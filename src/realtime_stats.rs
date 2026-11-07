use crate::chateaux::ChateauToSendNoGeom;
use crate::nav::Nav;
use chrono::{DateTime, Utc};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub fn stats_url(feed_id: &str) -> String {
    format!("/gtfs-rt-stats?feed_id={}", urlencoding::encode(feed_id))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FeedTimestamps {
    pub vehicle_positions: Option<u64>,
    pub trip_updates: Option<u64>,
    pub alerts: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChateauStats {
    pub chateau: ChateauToSendNoGeom,
    pub entities: Result<BTreeMap<String, u64>, String>,
    pub authoritative: Result<BTreeMap<String, u64>, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FeedStats {
    pub timestamps: Result<FeedTimestamps, String>,
    pub chateaux: Result<Vec<ChateauStats>, String>,
}

#[cfg(feature = "ssr")]
async fn fetch_json<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    path: &str,
) -> Result<T, String> {
    let response = client
        .get(format!("https://birch.catenarymaps.org{path}"))
        .timeout(std::time::Duration::from_secs(4))
        .header("Cache-Control", "no-cache")
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;
    let response = response
        .error_for_status()
        .map_err(|e| format!("Birch returned an error: {e}"))?;
    response
        .json()
        .await
        .map_err(|e| format!("Invalid response: {e}"))
}

#[server(endpoint = "get_gtfs_rt_stats")]
pub async fn get_gtfs_rt_stats(
    feed_id: String,
    known_chateaux: Option<Vec<ChateauToSendNoGeom>>,
) -> Result<FeedStats, ServerFnError> {
    if feed_id.trim().is_empty() {
        return Err(ServerFnError::new("A feed_id is required"));
    }
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    let client = CLIENT.get_or_init(reqwest::Client::new);
    let timestamps_path = format!(
        "/gtfs_rt_timestamps?feed_id={}",
        urlencoding::encode(&feed_id)
    );
    let timestamps = fetch_json(client, &timestamps_path);
    let counts = async {
        let chateaux = match known_chateaux {
            Some(chateaux) => chateaux,
            None => fetch_json::<Vec<ChateauToSendNoGeom>>(client, "/getchateausnogeom")
                .await?
                .into_iter()
                .filter(|c| c.realtime_feeds.contains(&feed_id))
                .collect(),
        };
        Ok(
            futures::future::join_all(chateaux.into_iter().map(|chateau| async move {
                let id = urlencoding::encode(&chateau.chateau);
                let entities_path = format!("/chateau_gtfs_rt_counts/{id}");
                let authoritative_path = format!("/chateau_authoritative_counts/{id}");
                let (entities, authoritative) = futures::join!(
                    fetch_json(client, &entities_path),
                    fetch_json(client, &authoritative_path)
                );
                ChateauStats {
                    chateau,
                    entities,
                    authoritative,
                }
            }))
            .await,
        )
    };
    let (timestamps, chateaux) = futures::join!(timestamps, counts);
    Ok(FeedStats {
        timestamps,
        chateaux,
    })
}

fn timestamp_text(timestamp: Option<u64>) -> String {
    match timestamp {
        None => "null (unavailable)".to_owned(),
        Some(timestamp) => i64::try_from(timestamp)
            .ok()
            .and_then(|seconds| DateTime::<Utc>::from_timestamp(seconds, 0))
            .map(|dt| format!("{} ({timestamp})", dt.format("%Y-%m-%d %H:%M:%S UTC")))
            .unwrap_or_else(|| timestamp.to_string()),
    }
}

fn age_text(timestamp: Option<u64>, now: i64) -> String {
    match timestamp {
        None => "—".to_owned(),
        Some(timestamp) => {
            let age = i128::from(now) - i128::from(timestamp);
            if age >= 0 {
                format!("{age} s ago")
            } else {
                format!("in {} s", -age)
            }
        }
    }
}

#[component]
pub fn GtfsRtStats() -> impl IntoView {
    let query = use_query_map();
    let feed_id = move || query.with(|q| q.get("feed_id").unwrap_or_default());
    view! {
        <Nav/>
        <main class="m-4 md:m-8 text-black dark:text-gray-100">
            <a href="/chateaux" class="underline text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-300">"Châteaux"</a>
            // Key by feed ID so changing the query disposes the old polling loop and data.
            <For each=move || vec![feed_id()] key=|id| id.clone() children=|id| view! {
                <RealtimeFeedStats feed_id=id/>
            }/>
        </main>
    }
}

#[component]
fn RealtimeFeedStats(feed_id: String) -> impl IntoView {
    let stats = RwSignal::new(None::<FeedStats>);
    let now = RwSignal::new(0_i64);
    let last_poll = RwSignal::new(None::<i64>);
    let poll_error = RwSignal::new(None::<String>);
    let valid_id = !feed_id.trim().is_empty();

    #[cfg(feature = "hydrate")]
    if valid_id {
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        };
        let id = feed_id.clone();
        let disposed = Arc::new(AtomicBool::new(false));
        let in_flight = Arc::new(AtomicBool::new(false));
        let known_chateaux = RwSignal::new(None::<(i64, Vec<ChateauToSendNoGeom>)>);
        let cleanup_disposed = disposed.clone();
        Effect::new(move |_| {
            let disposed = disposed.clone();
            let in_flight = in_flight.clone();
            let id = id.clone();
            let poll = move || {
                now.set(Utc::now().timestamp());
                // Slow requests never overlap. Ages keep advancing on every tick.
                if in_flight.swap(true, Ordering::Relaxed) {
                    return;
                }
                let id = id.clone();
                let disposed = disposed.clone();
                let in_flight = in_flight.clone();
                let known = known_chateaux
                    .get_untracked()
                    .and_then(|(loaded_at, chateaux)| {
                        (Utc::now().timestamp() - loaded_at < 60).then_some(chateaux)
                    });
                let refresh_directory = known.is_none();
                leptos::task::spawn_local(async move {
                    let result = get_gtfs_rt_stats(id, known).await;
                    in_flight.store(false, Ordering::Relaxed);
                    if disposed.load(Ordering::Relaxed) {
                        return;
                    }
                    match result {
                        Ok(snapshot) => {
                            if refresh_directory {
                                if let Ok(chateaux) = &snapshot.chateaux {
                                    if !chateaux.is_empty() {
                                        known_chateaux.set(Some((
                                            Utc::now().timestamp(),
                                            chateaux.iter().map(|c| c.chateau.clone()).collect(),
                                        )));
                                    }
                                }
                            }
                            stats.set(Some(snapshot));
                            last_poll.set(Some(Utc::now().timestamp()));
                            poll_error.set(None);
                        }
                        Err(error) => poll_error.set(Some(error.to_string())),
                    }
                });
            };
            poll();
            let handle = leptos::leptos_dom::helpers::set_interval_with_handle(
                poll,
                std::time::Duration::from_secs(1),
            );
            match handle {
                Ok(handle) => on_cleanup(move || handle.clear()),
                Err(_) => poll_error.set(Some(
                    "Could not start automatic refresh. Reload to retry.".to_owned(),
                )),
            }
        });
        on_cleanup(move || cleanup_disposed.store(true, Ordering::Relaxed));
    }

    view! {
        <h1 class="text-2xl font-bold text-tulip dark:text-tulip-300 mt-4 mb-2">"GTFS-RT stats"</h1>
        <p class="font-mono break-all mb-4 text-gray-800 dark:text-gray-200">{feed_id}</p>
        {if !valid_id { Some(view! {
            <p role="alert" class="text-gray-700 dark:text-gray-300">"Provide a feed ID using /gtfs-rt-stats?feed_id=…"</p>
        }) } else { None }}
        <Show when=move || valid_id>
            <p class="mb-2 text-gray-700 dark:text-gray-300">"Refreshes every second. Timestamps are publisher times in Unix seconds."</p>
            <p class="text-sm mb-4 text-gray-600 dark:text-gray-400">{move || last_poll.get().map(|ts| format!("Last response: {}", timestamp_text(Some(ts as u64)))).unwrap_or_else(|| "Loading stats…".to_owned())}</p>
            {move || poll_error.get().map(|error| view! {
                <p role="alert" class="text-red-600 dark:text-red-400 mb-4">{format!("Refresh failed: {error}. Any displayed data is from the last response; retrying automatically.")}</p>
            })}
            {move || stats.get().map(|snapshot| view! {
                <section class="mb-6">
                    <h2 class="text-xl font-semibold mb-2 text-gray-900 dark:text-gray-100">"Feed update timestamps"</h2>
                    {match snapshot.timestamps {
                        Err(error) => view! { <p role="alert" class="text-red-600 dark:text-red-400">{format!("Timestamps unavailable: {error}")}</p> }.into_any(),
                        Ok(ts) => view! {
                            <div class="overflow-x-auto">
                                <table class="table-auto w-full border-collapse">
                                    <thead><tr class="text-left border-b border-gray-400 dark:border-gray-600 text-gray-800 dark:text-gray-200">
                                        <th class="p-2">"Category"</th><th class="p-2">"Last updated (UTC / Unix seconds)"</th><th class="p-2">"Age"</th>
                                    </tr></thead>
                                    <tbody>{[
                                        ("Vehicle positions", ts.vehicle_positions),
                                        ("Trip updates", ts.trip_updates),
                                        ("Alerts", ts.alerts),
                                    ].into_iter().map(|(label, timestamp)| view! {
                                        <tr class="border-b border-gray-200 dark:border-gray-800 text-gray-900 dark:text-gray-100">
                                            <th scope="row" class="p-2 text-left font-normal">{label}</th>
                                            <td class="p-2 font-mono">{timestamp_text(timestamp)}</td>
                                            <td class="p-2 whitespace-nowrap">{move || age_text(timestamp, now.get())}</td>
                                        </tr>
                                    }).collect_view()}</tbody>
                                </table>
                            </div>
                        }.into_any(),
                    }}
                </section>
                <section>
                    <h2 class="text-xl font-semibold mb-2 text-gray-900 dark:text-gray-100">"Chateau entity counts and metadata"</h2>
                    <p class="mb-4 text-gray-700 dark:text-gray-300">"Counts cover all realtime feeds in each chateau, including any other feeds listed below."</p>
                    {match snapshot.chateaux {
                        Err(error) => view! { <p role="alert" class="text-red-600 dark:text-red-400">{format!("Chateau metadata unavailable: {error}")}</p> }.into_any(),
                        Ok(chateaux) if chateaux.is_empty() => view! {
                            <p class="text-gray-700 dark:text-gray-300">"No matching chateau was found for this realtime feed."</p>
                        }.into_any(),
                        Ok(chateaux) => chateaux.into_iter().map(|c| view! {
                            <section class="border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-900 rounded-lg p-4 mb-4 shadow-sm">
                                <h3 class="text-lg font-bold mb-2 text-gray-900 dark:text-gray-100">{c.chateau.chateau.clone()}</h3>
                                <p class="break-all text-gray-700 dark:text-gray-300">"Realtime feeds: "{c.chateau.realtime_feeds.join(", ")}</p>
                                <p class="break-all text-gray-700 dark:text-gray-300">"Schedule feeds: "{c.chateau.schedule_feeds.join(", ")}</p>
                                <p class="text-gray-700 dark:text-gray-300">"Languages: "{c.chateau.languages_avaliable.join(", ")}</p>
                                <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 mt-4">
                                    <CountTable title="GTFS-RT entities" counts=c.entities/>
                                    <CountTable title="Aspen authoritative store" counts=c.authoritative/>
                                </div>
                            </section>
                        }).collect_view().into_any(),
                    }}
                </section>
            })}
        </Show>
    }
}

#[component]
fn CountTable(title: &'static str, counts: Result<BTreeMap<String, u64>, String>) -> impl IntoView {
    view! {
        <section>
            <h4 class="font-semibold mb-2 text-gray-900 dark:text-gray-100">{title}</h4>
            {match counts {
                Err(error) => view! { <p role="alert" class="text-red-600 dark:text-red-400">{format!("Unavailable: {error}")}</p> }.into_any(),
                Ok(counts) if counts.is_empty() => view! { <p class="text-gray-600 dark:text-gray-400">"No counters returned."</p> }.into_any(),
                Ok(counts) => view! {
                    <div class="overflow-x-auto">
                        <table class="table-auto w-full border-collapse text-sm">
                            <thead><tr class="border-b border-gray-400 dark:border-gray-600 text-left text-gray-800 dark:text-gray-200"><th class="p-2">"Metric"</th><th class="p-2 text-right">"Count"</th></tr></thead>
                            <tbody>{counts.into_iter().map(|(key, count)| view! {
                                <tr class="border-b border-gray-200 dark:border-gray-800 text-gray-900 dark:text-gray-100">
                                    <th scope="row" class="p-2 text-left font-mono break-all font-normal">{key}</th>
                                    <td class="p-2 text-right font-mono">{count}</td>
                                </tr>
                            }).collect_view()}</tbody>
                        </table>
                    </div>
                }.into_any(),
            }}
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamps_use_seconds_and_preserve_null_and_future_values() {
        assert_eq!(age_text(Some(100), 112), "12 s ago");
        assert_eq!(age_text(Some(115), 112), "in 3 s");
        assert_eq!(age_text(None, 112), "—");
        assert_eq!(timestamp_text(None), "null (unavailable)");
        assert!(timestamp_text(Some(0)).contains("1970-01-01 00:00:00 UTC"));
        assert!(timestamp_text(Some(u64::MAX)).contains(&u64::MAX.to_string()));
    }

    #[test]
    fn stats_links_encode_the_entire_feed_id() {
        assert_eq!(
            stats_url("f-test &/東京"),
            "/gtfs-rt-stats?feed_id=f-test%20%26%2F%E6%9D%B1%E4%BA%AC"
        );
    }
}
