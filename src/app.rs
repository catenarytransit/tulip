use chrono::offset::Utc;
use chrono::prelude::*;
use chrono::DateTime;
use leptos::logging::*;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="AdobeFonts" href="https://use.typekit.net/nhx0pgc.css"/>
        <Stylesheet id="leptos" href="/pkg/catenarytulip.css"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Router>
            <Routes>
                <Route path="/" view=move || view! { <Home/> }/>
                <Route path="/realtimekeys" view=move || view! { <RealtimeKeys

                    /> }/>
                <Route path="/leptosexample" view=move || view! { <LeptosExample
                    /> }/>
                <Route path="/404.html" view=move || view! { <NotFound/> }/>
            </Routes>
        </Router>
    }
}

#[component]
fn Nav() -> impl IntoView {
    view! {
        <div class="w-full border-b-2 border-gray-200 drop-shadow-2xl flex flex-row align-middle mx-2 my-2 items-center">
            <a href="https://catenarymaps.org" target="_blank">
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
        <main class="mx-2">
        <h1>"Administration Links:"</h1>
        <ul>
            <li><a href="/realtimekeys" class="text-blue-500 underline">"Realtime Keys"</a></li>
        </ul>
        </main>
    }
}

#[derive(Serialize, Clone, Deserialize, Debug, Hash, PartialEq, Eq)]
pub struct PasswordFormat {
    pub key_formats: Vec<KeyFormat>,
    pub passwords: Vec<PasswordInfo>,
    pub override_schedule_url: Option<String>,
    pub override_realtime_vehicle_positions: Option<String>,
    pub override_realtime_trip_updates: Option<String>,
    pub override_alerts: Option<String>,
}

#[derive(Serialize, Clone, Deserialize, Debug, Hash, PartialEq, Eq)]
pub enum KeyFormat {
    Header(String),
    UrlQuery(String),
}

#[derive(Serialize, Clone, Deserialize, Debug, Hash, PartialEq, Eq)]
pub struct PasswordInfo {
    pub password: Vec<String>,
    pub creator_email: String,
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct EachPasswordRow {
    pub passwords: Option<PasswordFormat>,
    pub fetch_interval_ms: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyResponse {
    pub passwords: BTreeMap<String, EachPasswordRow>,
}

#[server]
async fn load_realtime_keys(
    master_email: String,
    master_password: String,
) -> Result<Option<KeyResponse>, ServerFnError> {
    let client = reqwest::Client::new();

    let response = client
        .get("https://birch.catenarymaps.org/getrealtimekeys/")
        .header("email", master_email)
        .header("password", master_password)
        .send()
        .await?;

    println!("Sending to Birch");
    match response.status() {
        reqwest::StatusCode::OK => {
            let response_text = response.text().await?;
            println!("Recieved response back from birch");

            let key_response: KeyResponse = serde_json::from_str(&response_text)?;

            Ok(Some(key_response))
        }
        reqwest::StatusCode::UNAUTHORIZED => {
            println!("Unauthorized");
            Ok(None)
        }
        _ => {
            println!("Error, {}", response.status());
            Err(ServerFnError::new("Data did not load correctly"))
        }
    }
}

#[component]
fn RealtimeKeys() -> impl IntoView {
    let (master_email, set_master_email) = create_signal(String::from(""));
    let (master_password, set_master_password) = create_signal(String::from(""));
    let (master_creds, set_master_creds) = create_signal((String::from(""), String::from("")));

    let (original_keys, set_original_keys) =
        create_signal::<BTreeMap<String, EachPasswordRow>>(BTreeMap::new());

    let (authorised, set_authorised) = create_signal(false);

    let (count, set_count) = create_signal(0);

    let async_data_load = create_resource(
        move || (master_email.get(), master_password.get(), count.get()),
        |(master_email, master_password, _)| async move {
            load_realtime_keys(master_email.clone(), master_password.clone()).await
        },
    );

    create_effect(move |_| {
        async_data_load.and_then(|data| {
            if let Some(data) = data {
                set_original_keys(data.passwords.clone());
                set_authorised(true);
            } else {
                set_authorised(false);
            }
        });
    });

    view! {
        <Nav/>
        <main class="mx-4">
            <h1 class="text-lg font-bold">"Realtime Keys"</h1>

            <p>"Enter the master password to view the realtime keys."</p>

            <p>"Master Email"</p>

            <input
                type="email"
                prop:value=move || master_email.get()
                class= "shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                on:input=move |event| {
                    set_master_email(event_target_value(&event));
                    set_master_creds((event_target_value(&event), master_password.get()));
                }
            />

            <p>"Master Password"</p>

            <input
                type="password"
                prop:value=move || master_password.get()
                class= "shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                on:input=move |event| {
                    set_master_password(event_target_value(&event));
                    set_master_creds((master_email.get(), event_target_value(&event)));
                }
            />

            <br/>

            //load button
            <button class="bg-blue-500 text-white border font-bold py-2 px-4 rounded"
            on:input=move |event| set_count.update(|count| *count += 1)
            >"Load"</button>

            <br/>


                <div>
                <Transition fallback=move || view! {<p>"Loading..."</p> }>
            <ErrorBoundary fallback=|errors| view!{<p>"There was an error"</p>}>

                <div>
                    <h2>"Realtime Keys"</h2>
                    {
                        if authorised.get() {
                            view! {
                                <p>"Authorised"</p>
                            }
                        } else {
                            view! {
                                <p>"Not authorised"</p>
                            }
                        }
                    }

                    <ul>
                        {original_keys.get().iter().map(|(key, value)| {
                            view! {
                                <li>
                                    <h3>{key}</h3>
                                    <p>{format!("{:?}", value)}</p>
                                </li>
                            }
                        }).collect_view()}
                    </ul>
                </div>

            </ErrorBoundary>
            </Transition>
                </div>

        </main>
    }
}

fn time_format_now() -> String {
    let system_time = Local::now();

    format!("{}", system_time.format("%A %+"))
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
