use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use crate::nav::Nav;
use chrono::{DateTime, Utc};
use leptos_router::hooks::use_params_map;

fn format_elapsed_time(timestamp_ms: i64) -> String {
    let now = Utc::now().timestamp_millis();
    let elapsed_ms = now - timestamp_ms;
    if elapsed_ms < 0 {
        return "".to_string();
    }
    let elapsed_secs = elapsed_ms / 1000;
    let days = elapsed_secs / 86400;
    let hours = (elapsed_secs % 86400) / 3600;
    let mins = (elapsed_secs % 3600) / 60;
    
    let mut parts = Vec::new();
    if days > 0 { parts.push(format!("{}d", days)); }
    if hours > 0 { parts.push(format!("{}h", hours)); }
    if mins > 0 { parts.push(format!("{}m", mins)); }
    if parts.is_empty() { parts.push("<1m".to_string()); }
    
    format!(" ({} ago)", parts.join(" "))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IngestedStatic {
    pub onestop_feed_id: String,
    pub ingest_start_unix_time_ms: i64,
    pub ingest_end_unix_time_ms: i64,
    pub ingest_duration_ms: i32,
    pub file_hash: String,
    pub attempt_id: String,
    pub ingesting_in_progress: bool,
    pub ingestion_successfully_finished: bool,
    pub ingestion_errored: bool,
    pub production: bool,
    pub deleted: bool,
    pub feed_expiration_date: Option<chrono::NaiveDate>,
    pub feed_start_date: Option<chrono::NaiveDate>,
    pub ingestion_version: i32,
    pub default_lang: Option<String>,
    pub languages_avaliable: Vec<Option<String>>,
    pub hash_of_file_contents: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StaticDownloadAttempt {
    pub onestop_feed_id: String,
    pub file_hash: Option<String>,
    pub downloaded_unix_time_ms: i64,
    pub ingested: bool,
    pub url: String,
    pub failed: bool,
    pub ingestion_version: i32,
    pub mark_for_redo: bool,
    pub http_response_code: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FeedMetadataResponse {
    pub ingested_static: Vec<IngestedStatic>,
    pub static_download_attempts: Vec<StaticDownloadAttempt>,
}

#[server(endpoint = "get_feed_metadata")]
pub async fn get_feed_metadata(feed_id: String) -> Result<FeedMetadataResponse, ServerFnError> {
    let client = reqwest::Client::new();
    let url = format!("https://birch.catenarymaps.org/feed_metadata?feed_id={}", feed_id);
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
        
    let metadata: FeedMetadataResponse = response.json().await.map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(metadata)
}

#[component]
pub fn FeedMetadata() -> impl IntoView {
    let params = use_params_map();
    let feed_id = move || params.with(|p| p.get("feed_id").unwrap_or_default());
    
    let metadata_resource = Resource::new(feed_id, |id| async move {
        if id.is_empty() { return None; }
        get_feed_metadata(id).await.ok()
    });

    view! {
        <Nav/>
        <main class="m-8 text-black dark:text-tulip text-left">
            <h1 class="text-2xl font-bold mb-4">"Feed Metadata: " {feed_id}</h1>
            <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                {move || metadata_resource.get().flatten().map(|metadata| {
                    view! {
                        <div class="flex flex-col gap-6">
                            <div>
                                <h2 class="text-xl font-semibold mb-2">"Ingested Static"</h2>
                                <table class="table-auto w-full border-collapse border border-gray-400">
                                    <thead>
                                        <tr class="bg-gray-200 dark:bg-gray-800 text-left">
                                            <th class="border border-gray-400 px-4 py-2">"Attempt ID"</th>
                                            <th class="border border-gray-400 px-4 py-2">"Hash"</th>
                                            <th class="border border-gray-400 px-4 py-2">"Time (UTC)"</th>
                                            <th class="border border-gray-400 px-4 py-2">"Duration"</th>
                                            <th class="border border-gray-400 px-4 py-2">"Status"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {metadata.ingested_static.into_iter().map(|ingest| {
                                            view! {
                                                <tr>
                                                    <td class="border border-gray-400 px-4 py-2">{ingest.attempt_id}</td>
                                                    <td class="border border-gray-400 px-4 py-2">
                                                        <span class="truncate block max-w-xs">{ingest.file_hash}</span>
                                                    </td>
                                                    <td class="border border-gray-400 px-4 py-2">{DateTime::from_timestamp_millis(ingest.ingest_start_unix_time_ms).map(|dt| dt.to_string()).unwrap_or_default()}{format_elapsed_time(ingest.ingest_start_unix_time_ms)}</td>
                                                    <td class="border border-gray-400 px-4 py-2">
                                                        {format!("{} ms", ingest.ingest_duration_ms)}
                                                    </td>
                                                    <td class="border border-gray-400 px-4 py-2">
                                                        {if ingest.ingesting_in_progress { "In Progress" } else if ingest.ingestion_successfully_finished { "Success" } else if ingest.ingestion_errored { "Errored" } else { "Unknown" }}
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            </div>
                            
                            <div>
                                <h2 class="text-xl font-semibold mb-2">"Static Download Attempts"</h2>
                                <table class="table-auto w-full border-collapse border border-gray-400">
                                    <thead>
                                        <tr class="bg-gray-200 dark:bg-gray-800 text-left">
                                            <th class="border border-gray-400 px-4 py-2">"URL"</th>
                                            <th class="border border-gray-400 px-4 py-2">"Status"</th>
                                            <th class="border border-gray-400 px-4 py-2">"Response Code"</th>
                                            <th class="border border-gray-400 px-4 py-2">"Time (UTC)"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {metadata.static_download_attempts.into_iter().map(|attempt| {
                                            view! {
                                                <tr>
                                                    <td class="border border-gray-400 px-4 py-2 truncate max-w-xs">{attempt.url}</td>
                                                    <td class="border border-gray-400 px-4 py-2">{if attempt.failed { "Failed" } else { "Success" }}</td>
                                                    <td class="border border-gray-400 px-4 py-2">{attempt.http_response_code.unwrap_or_default()}</td>
                                                    <td class="border border-gray-400 px-4 py-2">{DateTime::from_timestamp_millis(attempt.downloaded_unix_time_ms).map(|dt| dt.to_string()).unwrap_or_default()}{format_elapsed_time(attempt.downloaded_unix_time_ms)}</td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    }
                })}
            </Suspense>
        </main>
    }
}
