use chrono::offset::Utc;
use chrono::prelude::*;
use chrono::DateTime;
use leptos::logging::*;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="AdobeFonts" href="https://use.typekit.net/nhx0pgc.css"/>
        <Stylesheet id="leptos" href="/pkg/leptos_start.css"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Router>
            <Routes>
                <Route path="/" view=move || view! { <Home/> }/>
                <Route path="/gtfsingesterrors" view=move || view! { <GtfsIngestErrors/> }/>
                <Route path="/kactustimes" view=move || view! { <KactusTimes/> }/>
                <Route path="/leptosexample" view=move || view! { <LeptosExample/> }/>
                <Route path="/404.html" view=move || view! { <NotFound/> }/>
            </Routes>
        </Router>
    }
}

#[component]
fn Nav() -> impl IntoView {
    view! {
        <div class="w-full border-b-2 border-gray-200 drop-shadow-2xl flex flex-row align-middle mx-2 my-2 items-center">
            <a href="https://catenarymaps.org">
                <img src="/LogoSharpCorners.png" class="h-8 "/>
            </a>
            <a href="/">
                <img src="/tulip-logo.png" class="h-12"/>

            </a>
            <a href="/">
                <p class="text-3xl italic bigmoore align-middle">"Tulip"</p>
            </a>

        </div>
    }
}

#[component]
fn Home() -> impl IntoView {
    view! {
        <Nav/>
        <main>
            <a href="gtfsingesterrors">
                <p>"GTFS Ingest Errors"</p>
            </a>
        </main>
    }
}

#[component]
fn KactusTimes() -> impl IntoView {
    view! {
        <Nav/>
        <main></main>
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct GtfsIngestErrorRow {
    onestop_feed_id: String,
    error: String,
}

type FetchErrorsOutput = Result<Vec<GtfsIngestErrorRow>, Box<dyn std::error::Error>>;
async fn fetch_gtfs_ingest_errors() -> FetchErrorsOutput {
    //let res = reqwasm::http::Request::get("https://backend.catenarymaps.org/gtfsingesterrors")
    //.send().await;

    log!("Fetching errors list from backend");

    let res = reqwest_wasm::get("https://backend.catenarymaps.org/gtfsingesterrors").await;

    match res {
        Ok(res) => {
            let text = res.text().await.unwrap();
            log!("{}", text);

            let output = serde_json::from_str::<Vec<GtfsIngestErrorRow>>(text.as_str()).unwrap();

            for row in &output {
                log!("{}", row.error);
            }

             let output = output.into_iter().map(|x| GtfsIngestErrorRow {
                onestop_feed_id: x.onestop_feed_id,
                error: x.error.replace("\\n","\n")
            }).collect::<Vec<GtfsIngestErrorRow>>();

            Ok(output)
        }
        Err(err) => Err(Box::new(err)),
    }
}

fn time_format_now() -> String {
    let system_time = Local::now();

    format!("{}", system_time.format("%A %+"))
}

#[component]
fn GtfsIngestErrors() -> impl IntoView {
    let (request_time, set_request_time) = create_signal::<DateTime<Local>>(Local::now());

    let error_list: Resource<DateTime<Local>, Option<Vec<GtfsIngestErrorRow>>> = create_resource(
        move || request_time.get(),
        |value| async move {
            let data_from_backend = fetch_gtfs_ingest_errors().await;

            match data_from_backend {
                Ok(data_from_backend) => Some(data_from_backend),
                _ => None,
            }
        },
    );

    create_effect(move |_| {
        // immediately prints "Value: 0" and subscribes to `a`
        set_request_time(Local::now());
      });

    view! {
        <Nav/>
        <main class="mx-3">
            <h1 class="font-semibold text-2xl">"GTFS Ingest Errors"</h1>
            <button
                class="px-2 py-1 border border-black"
                on:click=move |_| set_request_time(Local::now())
            >
                "Reload"
            </button>

            <table>
                <thead>
                    <tr class="font-semibold">
                        <td>"onestop_feed_id"</td>
                        <td>"error"</td>
                    </tr>
                </thead>
                <Suspense fallback=move || {
                    view! { <p></p> }
                }>

                    {move || match error_list.get() {
                        Some(data) => {
                            match data {
                                Some(inner_data) => {
                                    view! {
                                        <For
                                            each=move || inner_data.clone()
                                            key=|row| row.clone().onestop_feed_id
                                            children=move |row: GtfsIngestErrorRow| {
                                                view! {
                                                    <tr>
                                                        <td class="font-mono border border-slate-500">
                                                            {row.onestop_feed_id}
                                                        </td>
                                                        <td class="font-mono border border-slate-500 whitespace-pre-wrap">
                                                            {row.error}
                                                        </td>
                                                    </tr>
                                                }
                                            }
                                        />
                                    }
                                }
                                None => view! { <p>"Nothing found"</p> }.into_view(),
                            }
                        }
                        None => view! { () }.into_view(),
                    }}

                </Suspense>
            </table>
        </main>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous
        // if it were async, we'd use a server function
        let resp = expect_context::<leptos_actix::ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <Nav/>
        <main class="h-full w-full">
            <div class="m-auto">"404, this page doesn't exist"</div>
        </main>
    }
}

#[component]
fn LeptosExample() -> impl IntoView {
    let (count, set_count) = create_signal(0);

    view! {
        <main class="my-0 mx-auto max-w-3xl text-center">
            <h2 class="p-6 text-4xl">"Welcome to Leptos with Tailwind"</h2>
            <p class="px-10 pb-10 text-left">
                "Tailwind will scan your Rust files for Tailwind class names and compile them into a CSS file."
            </p>
            <button
                class="bg-amber-600 hover:bg-sky-700 px-5 py-3 text-white rounded-lg"
                on:click=move |_| set_count.update(|count| *count += 1)
            >
                "Something's here | "
                {move || if count() == 0 { "Click me!".to_string() } else { count().to_string() }}

                " | Some more text"
            </button>
        </main>
    }
}
