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
        <Stylesheet href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/default.min.css"/>
        <Script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js"/>
        <Script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/rust.min.js"/>
        <Script>
        "hljs.highlightAll();"
        </Script>
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

#[derive(Serialize, Clone, Deserialize, Debug, Hash, PartialEq, Eq, Default)]
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

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct EachPasswordRowInput {
    pub passwords: String,
    pub fetch_interval_ms: String,
    pub originals: EachPasswordRow,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyResponse {
    pub passwords: HashMap<String, EachPasswordRow>,
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

#[server]
async fn submit_data(
    master_email: String,
    master_password: String,
    feed_id: String,
    password: String,
    interval: String
) -> Result<bool, ServerFnError> {
    //post json EachPasswordRow to /setrealtimekey/{feed_id}/

    let client = reqwest::Client::new();

    let data_to_send = EachPasswordRow {
        passwords: ron::from_str(&password)?,
        fetch_interval_ms: ron::from_str(&interval)?,
    };

    let response = client
        .post(format!(
            "https://birch.catenarymaps.org/setrealtimekey/{}/",
            feed_id
        ))
        .header("email", master_email)
        .header("password", master_password)
        .body(ron::to_str(&data_to_send)?)
        .send()
        .await?;

        let status = response.status();

    match status {
        reqwest::StatusCode::OK => Ok(true),
        reqwest::StatusCode::UNAUTHORIZED => Ok(false),
        _ => {
            let text = response.text().await;

            match text {
                Ok(text) => {
                    println!("recieved strange answer from birch on setrealtimekey, {} text {}", status, text);
                },
                Err(err) => {
                    println!("error on birch setrealtimekey {} err {}", status, err);
                }
            }
            Err(ServerFnError::new("Data did not submit correctly"))},
    }
}

#[component]
fn RealtimeKeys() -> impl IntoView {
    let (master_email, set_master_email) = create_signal(String::from(""));
    let (master_password, set_master_password) = create_signal(String::from(""));
    let (master_creds, set_master_creds) = create_signal((String::from(""), String::from("")));

    let (form_feed_id, set_form_feed_id) = create_signal(String::from(""));
    let (form_password, set_form_password) = create_signal(String::from(""));
    let (form_interval_ms, set_form_interval_ms) = create_signal(String::from(""));

    // let new_keys = create_rw_signal::<HashMap<String, EachPasswordRowInput>>(HashMap::new());

    let original_keys = create_rw_signal::<HashMap<String, EachPasswordRow>>(HashMap::new());

    let (authorised, set_authorised) = create_signal(false);

    let (count, set_count) = create_signal(0);

    let async_data_load = create_resource(
        move || (master_email.get(), master_password.get(), count.get()),
        |(master_email, master_password, _)| async move {
            load_realtime_keys(master_email.clone(), master_password.clone()).await
        },
    );

    let feed_id_node_ref: NodeRef<html::Input> = create_node_ref();
    let password_node_ref: NodeRef<html::Textarea> = create_node_ref();
    let interval_ms_node_ref: NodeRef<html::Input> = create_node_ref();

    create_effect(move |_| {
        async_data_load.and_then(|data| {
            leptos::logging::log!("{:?}", data);
            if let Some(data) = data {
                original_keys.update(|x| *x = data.passwords.clone());
                set_authorised(true);
                /*  new_keys.update(|x| *x = data.passwords.clone().into_iter().map(
                    |(key, value)| {
                        (
                            key.clone(),
                            EachPasswordRowInput {
                                passwords: format!("{:#?}", value),
                                fetch_interval_ms: format!("{:?}", value.fetch_interval_ms),
                                originals: value.clone(),
                            },
                        )
                    },
                ).collect());*/
            } else {
                //set_authorised(false);
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
                {
                    move || if authorised.get() {
                        view! {
                            <p>"Authorised"</p>
                        }
                    } else {
                        view! {
                            <p>"Not authorised"</p>
                        }
                    }
                }

                <h2 class="text-xl font-semibold">"Instructions"</h2>

                <p> "Keys are defined as Option<PasswordFormat> as defined in this structure here:"</p>

                        //code=STRUCT_PASSWORD_TEXT.to_string()

                        <pre><code class="language-rust">{STRUCT_PASSWORD_TEXT.to_string()}</code></pre>

                    <p>"Every password entry is required to have the same length as key_format. Uploads will be blocked otherwise."</p>
                    <p>"The fetch interval is the number of milliseconds between fetches of the realtime data. Putting None will default the value to what Alpenrose has."</p>

                    <p>"Here's an imaginary api for Washington Metro Area Transit Authority"</p>

                    <pre><code class="language-rust">{format!("{}", ron::ser::to_string_pretty(&give_wmata_format(),
                    ron::ser::PrettyConfig::default()).unwrap())}</code></pre>

                    <p>"Here's an imaginary api for San Francisco Bay Area Transit Authority, but pretend we set the vehicle position url manually"</p>

                    <pre><code class="language-rust">{format!("{}", ron::ser::to_string_pretty(&give_sfbay_format(),
                        ron::ser::PrettyConfig::default()).unwrap())}</code></pre>
                    <div>
                    <h2 class="text-xl font-semibold">"Realtime Keys"</h2>

                    <ul>
                     {
                        move ||
                            original_keys.with(|keys| keys.iter().map(|(key, value)| {
                                view! {
                                    <li>
                                    <h3 class="text-lg font-semibold">{key.clone()}</h3>
                                        <p class="font-bold">"Current values"</p>
                                        <p class="font-semibold">"Passwords:"</p>
                                        <p class="bg-gray-100 font-mono">{format!("{}", ron::ser::to_string_pretty(&value.passwords,
                                            ron::ser::PrettyConfig::default()).unwrap())}</p>

                                        <p class="font-semibold">"Fetch Interval:"</p>
                                        <p>{format!("{:?}", value.fetch_interval_ms)}</p>
                                    </li>
                                }
                            }).collect_view())
                     }

                    </ul>
                </div>

                <div><h2 class="text-xl font-semibold">
                "Submission form"
                </h2>

                <div class="flex flex-row gap-x-2">
                     <button class="bg-blue-500 text-white border font-bold py-2 px-4 rounded"

                    on:click=move |_| {
                        set_form_feed_id(String::from(""));
                        set_form_interval_ms(String::from(""));
                        set_form_password(String::from(""));
                    }
                    disabled=move || !authorised.get()
                     >"Clear all fields"</button>

                        <button class="bg-blue-500 text-white border font-bold py-2 px-4 rounded"
                        on:click=move |_| {
                            set_form_password(format!("{}",

                                ron::ser::to_string_pretty(&Some(PasswordFormat::default()),
                                    ron::ser::PrettyConfig::default()).unwrap()
                        ));
                        }
                        disabled=move || !authorised.get()
                        >
                        "Fill with default password format"
                    </button>

                    <button class="bg-blue-500 text-white border font-bold py-2 px-4 rounded"
                        on:click=move |_| {
                            match original_keys.get().get(form_feed_id.get().as_str()) {
                                Some(original_data) => {
                                    set_form_password(
                                        //use ron
                                        ron::ser::to_string_pretty(&original_data.passwords,
                                            ron::ser::PrettyConfig::default()).unwrap(),
                                    );
                                    set_form_interval_ms(
                                        ron::ser::to_string_pretty(&original_data.fetch_interval_ms,
                                            ron::ser::PrettyConfig::default()).unwrap(),
                                    );
                                },
                                None => {
                                    set_form_password(String::from(""));
                                    set_form_interval_ms(String::from(""));
                                }
                            }
                        }
                        disabled=move || !authorised.get()
                        >
                        "Import using feed id"
                    </button>
                </div>

                <p>"feed id"</p>

                <input
                type="text"
                prop:value=move || form_feed_id.get()
                disabled=move || !authorised.get()
                class= "shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                on:input=move |event| {
                    set_form_feed_id(event_target_value(&event));
                }
                node_ref=feed_id_node_ref
            />

            //check if the feed id is in the original dataset

            {
                move || match original_keys.get().get(form_feed_id.get().as_str()) {
                    Some(_) => view! {
                        <p>"✅ Feed ID is valid"</p>
                    },
                    None => view! {
                        <p>"❌ Feed ID is invalid"</p>
                    }
                }
            }

            <p>"interval"</p>

            <input
                type="text"
                prop:value=move || form_interval_ms.get()
                class= "shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                disabled=move || !authorised.get()
                on:input=move |event| {
                    set_form_interval_ms(event_target_value(&event));
                }
                node_ref=interval_ms_node_ref
            />


             {
                move || match ron::from_str::<Option<u32>>(form_interval_ms.get().as_str()) {
                    Ok(_) => view! {
                        <p>"✅ Interval is valid"</p>
                    },
                    Err(_) => view! {
                        <p>"❌ Interval is invalid, must be Option<u32> like Some(1000) or None"</p>
                }
            }
             }

            <p>"password"</p>

            <textarea
                type="text"
                prop:value=move || form_password.get()
                disabled=move || !authorised.get()
                class= "shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                on:input=move |event| {
                    set_form_password(event_target_value(&event));
                }
                node_ref=password_node_ref
            />

            {
                move || match ron::from_str::<Option<PasswordFormat>>(form_password.get().as_str()) {
                    Ok(formatted_password) => {
                        //check if the password is the same length as the key format fo all passwords
                        match formatted_password {
                            Some(formatted_password) => {
                                let key_formats = formatted_password.key_formats.len();
                                let passwords = formatted_password.passwords.iter().map(|x| x.password.len()).collect::<Vec<_>>();
                                let all_same = passwords.iter().all(|x| *x == key_formats);

                                match all_same {
                                    true => view! {
                                        <> <p>"✅ Password is valid"</p></>
                                    },
                                    false => view! {
                                        <> <p>"❌ Password is invalid, must be the same length as key format"</p></>
                                    }
                                }
                            },
                            None => {
                                view! {
                                    <><p>"✅ Password is valid"</p></>
                                }
                            }
                        }
                    },
                    Err(err) => view! {
                        <>
                        <p>"❌ Password is invalid"</p>
                        <p class="font-mono">{format!("{:#?}", err)}</p>
                        </>
                }
            }
            }

            <button

                class="bg-blue-500 text-white border font-bold py-2 px-4 rounded"
                disabled=move || !authorised.get()
            on:click=move |e| {
              let master_creds = master_creds.get();
              let (form_feed_id, form_password, form_interval_ms) = (form_feed_id.get(),
              form_password.get(),
              form_interval_ms.get());

              spawn_local(async move {
                submit_data(master_creds.0, master_creds.1, form_feed_id, form_password, form_interval_ms).await;
              });
            }
                >"Submit"</button>

                </div>

        </main>

    <Script>
    "hljs.highlightAll();"
    </Script>
    }
}

const STRUCT_PASSWORD_TEXT: &str = r"#[derive(Serialize, Clone, Deserialize, Debug, Hash, PartialEq, Eq, Default)]
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
}";

fn give_wmata_format() -> Option<PasswordFormat> {
    Some(PasswordFormat {
        key_formats: vec![KeyFormat::Header("api_key".to_string())],
        passwords: vec![PasswordInfo {
            password: vec!["c3ab117ab77aa801f706e6bea12f5612".to_string()],
            creator_email: String::from("kyler@catenarymaps.org"),
        }],
        override_schedule_url: None,
        override_realtime_vehicle_positions: None,
        override_realtime_trip_updates: None,
        override_alerts: None,
    })
}

fn give_sfbay_format() -> Option<PasswordFormat> {
    Some(PasswordFormat {
        key_formats: vec![KeyFormat::UrlQuery("api_key".to_string())],
        passwords: vec![
            PasswordInfo {
                password: vec!["f8f683cc177053581ef9d425071eb6d1".to_string()],
                creator_email: String::from("kyler@catenarymaps.org"),
            },
            PasswordInfo {
                password: vec!["e6c335d9cab3bd41ac51bc6235ce966b".to_string()],
                creator_email: String::from("sam@catenarymaps.org"),
            },
        ],
        override_schedule_url: None,
        override_realtime_vehicle_positions: Some(String::from(
            "http://api.511.org/transit/vehiclepositions",
        )),
        override_realtime_trip_updates: None,
        override_alerts: None,
    })
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
