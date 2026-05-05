use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use crate::nav::Nav;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChateauToSendNoGeom {
    pub chateau: String,
    pub realtime_feeds: Vec<String>,
    pub schedule_feeds: Vec<String>,
    pub languages_avaliable: Vec<String>,
}

#[server(endpoint = "get_chateaus_nogeom")]
pub async fn get_chateaus_nogeom() -> Result<Vec<ChateauToSendNoGeom>, ServerFnError> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://birch.catenarymaps.org/getchateausnogeom")
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
        
    let mut chateaus: Vec<ChateauToSendNoGeom> = response.json().await.map_err(|e| ServerFnError::new(e.to_string()))?;
    chateaus.sort_by(|a, b| a.chateau.cmp(&b.chateau));
    Ok(chateaus)
}

#[component]
pub fn Chateaux() -> impl IntoView {
    let chateaus_resource = Resource::new(|| (), |_| async move { get_chateaus_nogeom().await.unwrap_or_default() });

    view! {
        <Nav/>
        <main class="m-8 text-black dark:text-tulip text-left">
            <h1 class="text-2xl font-bold mb-4">"Châteaux"</h1>
            <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                <div class="flex flex-col gap-4">
                    {move || chateaus_resource.get().map(|chateaus| {
                        chateaus.into_iter().map(|c| {
                            view! {
                                <div class="p-4 border border-gray-300 rounded shadow">
                                    <h2 class="text-xl font-semibold">{c.chateau.clone()}</h2>
                                    <div>
                                        <h3 class="font-bold">"Schedule Feeds:"</h3>
                                        <ul class="list-disc pl-5">
                                            {c.schedule_feeds.into_iter().map(|f| {
                                                let link = format!("/debug/schedule/{}", f);
                                                view! {
                                                    <li>
                                                        <a href=link class="text-blue-500 hover:text-blue-700 underline">
                                                            {f}
                                                        </a>
                                                    </li>
                                                }
                                            }).collect_view()}
                                        </ul>
                                    </div>
                                    <div class="mt-2">
                                        <h3 class="font-bold">"Realtime Feeds:"</h3>
                                        <ul class="list-disc pl-5">
                                            {c.realtime_feeds.into_iter().map(|f| view! { <li>{f}</li> }).collect_view()}
                                        </ul>
                                    </div>
                                </div>
                            }
                        }).collect_view()
                    })}
                </div>
            </Suspense>
        </main>
    }
}
