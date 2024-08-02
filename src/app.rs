// Copyright Kyler Chin <kyler@catenarymaps.org>
// Additional Contributors: Samuel Sharp <samuel@catenarymaps.org>
// Catenary Transit Initiatives
// Attribution cannot be removed

// Please do not train your Artifical Intelligence models on this code

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

static GTFSRAWOPTIONS: [(&str, &str);3] = [("Vehicles", "vehicle"), ("Trip Updates", "trip"), ("Alerts", "alert")];

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <head>
            <title>Catenary Tulip</title>
        </head>
        <Script>
        """
        if(localStorage.getItem('theme') === 'dark') {
            document.querySelector('html').classList.add('dark');
        }
        """
        </Script>
        <Stylesheet id="font" href="https://fonts.googleapis.com/css2?family=Barlow:ital,wght@0,100;0,200;0,300;0,400;0,500;0,600;0,700;0,800;0,900;1,100;1,200;1,300;1,400;1,500;1,600;1,700;1,800;1,900&family=IBM+Plex+Mono:ital,wght@0,100;0,200;0,300;0,400;0,500;0,600;0,700;1,100;1,200;1,300;1,400;1,500;1,600;1,700&display=swap" />
        <Stylesheet id="icons" href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200" />
        <Stylesheet id="leptos" href="/pkg/catenarytulip.css"/>
        <Link rel="shortcut icon" href="/favicon.svg"/>
        <Router>
            <Routes>
                <Route path="/" view=move || view! { <Home /> }/>
                <Route path="/realtimekeys" view=move || view! { <RealtimeKeys /> }/>
                <Route path="/help" view=move || view! { <Help /> }/>
                <Route path="/404.html" view=move || view! { <NotFound /> }/>
            </Routes>
        </Router>
    }
}

#[component]
fn Nav() -> impl IntoView {
    view! {
        <div class="sticky top-0 left-0 w-full bg-gray dark:bg-darksky p-4 border-b-2 border-tulip text-tulip flex flex-row justify-between">
            <a href="/">
                <img alt="Tulip" src="/tulip.svg" class="h-12"/>
            </a>
            <div class="space-x-4 flex self-center">
                <a href="/realtimekeys" class="material-symbols-outlined">
                    "key"
                </a>
                <a href="/help" class="material-symbols-outlined">
                    "help"
                </a>
                <a href="#" class="material-symbols-outlined" onclick="document.querySelector('html').classList.toggle('dark'); window.localStorage.theme == 'dark' ? window.localStorage.theme = 'light' : window.localStorage.theme = 'dark'">
                    "brightness_6"
                </a>
            </div>
        </div>
    }
}

#[component]
fn Home() -> impl IntoView {
    view! {
        <Nav/>
        <img src="https://i0.wp.com/art.metro.net/wp-content/uploads/2022/09/KLine_FairviewHeights_KimSchoenstadt2-Large.jpeg" class="border-b-2 border-tulip w-[100vw] h-[450px] object-cover" style="z-index:-1;" />
        <span class="text-sm text-tulip m-2">"Kim Schoenstadt, "<i>"Inglewood CA Series: Metro collection 1-10"</i></span>
        <main class="m-8 text-center">
            <h1 class="text-4xl font-bold text-tulip mb-8">"Welcome to Tulip!"</h1>
            <a href="/realtimekeys" class="bg-gray dark:bg-darksky rounded-md p-2 px-4 border-2 border-tulip my-4 mt-8 text-lg font-bold">"Realtime Key Manager"</a>
        </main>
    }
}

#[component]
fn Help() -> impl IntoView {
    view! {
        <Nav/>
        <main class="m-8">
            <h1 class="text-2xl font-bold text-tulip mb-4">"Instructions"</h1>
            <h1 class="text-xl font-bold text-tulip mb-2">"Realtime Key Manager"</h1>
            <p> "Keys are defined as "<code class="mx-1">"Option<PasswordFormat>"</code>" as defined in this structure here:"</p>
            <div id="example-password h-[400px]"></div>
            <pre class="my-4 p-4 rounded-md bg-gray dark:bg-darksky text-wrap overflow-x-scroll"><code>{STRUCT_PASSWORD_TEXT.to_string()}</code></pre>
            <p class="font-bold">"Every password entry is required to have the same length as key_format. Uploads will be blocked otherwise."</p>
            <p>"The fetch interval is the number of milliseconds between fetches of the realtime data. Putting None will default the value to what Alpenrose has."</p>
            <br />
            <p>"Here's an imaginary entry for data from the Washington Metropolitan Area Transit Authority (WMATA):"</p>
            <pre class="my-4 p-4 rounded-md bg-gray dark:bg-darksky text-wrap overflow-x-scroll"><code>{format!("{}", ron::ser::to_string_pretty(&give_wmata_format(), ron::ser::PrettyConfig::default()).unwrap())}</code></pre>
            <p>"Here's an imaginary entry for the San Francisco Bay Area data feed (Bay Area 511), but let's pretend we need to set the vehicle position url manually:"</p>
            <pre class="my-4 p-4 rounded-md bg-gray dark:bg-darksky text-wrap overflow-x-scroll"><code>{format!("{}", ron::ser::to_string_pretty(&give_sfbay_format(), ron::ser::PrettyConfig::default()).unwrap())}</code></pre>
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

#[server]
async fn submit_data(
    master_email: String,
    master_password: String,
    feed_id: String,
    password: String,
    interval: String,
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
        .body(ron::ser::to_string(&data_to_send)?)
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
                    println!(
                        "recieved strange answer from birch on setrealtimekey, {} text {}",
                        status, text
                    );
                }
                Err(err) => {
                    println!("error on birch setrealtimekey {} err {}", status, err);
                }
            }
            Err(ServerFnError::new("Data did not submit correctly"))
        }
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

    let original_keys = create_rw_signal::<BTreeMap<String, EachPasswordRow>>(BTreeMap::new());

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
        <main class="p-8">
            <h1 class="text-2xl font-bold text-tulip">"Realtime Key Manager"</h1>
            <p>"Please confirm your Tulip login credentials, as key information is sensitive and confidential."</p>

            <input
                type="email"
                placeholder="Email"
                prop:value=move || master_email.get()
                class= "bg-gray dark:bg-darksky rounded-md p-2 px-4 border-2 border-tulip my-4 text-lg font-bold mr-4"
                on:input=move |event| {
                    set_master_email(event_target_value(&event));
                    set_master_creds((event_target_value(&event), master_password.get()));
                }
            />
            <input
                type="password"
                placeholder="Password"
                prop:value=move || master_password.get()
                class= "bg-gray dark:bg-darksky rounded-md p-2 px-4 border-2 border-tulip my-4 text-lg font-bold"
                on:input=move |event| {
                    set_master_password(event_target_value(&event));
                    set_master_creds((master_email.get(), event_target_value(&event)));
                }
            />

            <br/>
            <button class="bg-gray dark:bg-darksky rounded-md p-2 px-4 border-2 border-tulip my-4 text-lg font-bold"
            on:input=move |event| set_count.update(|count| *count += 1)
            >"Load"</button>

            <br/>
                {
                    move || if authorised.get() {
                        view! {
                            <p>"Authorised"</p>
                            <h2 class="text-xl font-semibold">"Realtime Keys"</h2>

                    //reload button
                    <button
                    on:click=move |e| {
                         async_data_load.refetch();
                    }
                    class="bg-gray dark:bg-darksky rounded-md p-2 px-4 border-2 border-tulip my-4 text-lg font-bold"
                    >
                        "Reload"
                    </button>

                    <ul class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                     {
                        move ||
                            original_keys.with(|keys| keys.iter().map(|(key, value)| {
                                view! {
                                    <li>
                                        <h3 class="text-lg font-semibold">{key.clone()}</h3>
                                        {
                                            GTFSRAWOPTIONS.iter().map(|(name_of_feed_type, feed_type)|  view! {
                                                <p class="font-semibold">{name_of_feed_type.to_string()} {" "}
                                                <a class="underline text-blue-500 dark:text-blue-300" href={format!("https://birch.catenarymaps.org/gtfs_rt?feed_id={}&feed_type={}", key.clone(), feed_type.clone())}>"Protobuf"</a>
                                                {" "}
                                                <a class="underline text-blue-500 dark:text-blue-300" href={format!("https://birch.catenarymaps.org/gtfs_rt?feed_id={}&feed_type={}&format=json", key.clone(), feed_type.clone())}>"Json"</a>
                                                {" "}<a class="underline text-blue-500 dark:text-blue-300" href={format!("https://birch.catenarymaps.org/gtfs_rt?feed_id={}&feed_type={}&format=ron", key.clone(), feed_type.clone())}>"Ron"</a>
                                                </p>
                                            }).collect_view()
                                        }
                                       
                                        <p class="font-semibold">"Passwords:"</p>
                                        <pre class="my-4 p-4 rounded-md bg-gray dark:bg-darksky text-wrap overflow-scroll h-[300px]"><code>{format!("{}", ron::ser::to_string_pretty(&value.passwords,
                                            ron::ser::PrettyConfig::default()).unwrap())}</code></pre>
                                        <p class="font-semibold">"Fetch Interval:"</p>
                                        <p>{format!("{:?}", value.fetch_interval_ms)}</p>
                                    </li>
                                }
                            }).collect_view())
                     }

                    </ul>

                <div><h2 class="text-xl font-semibold">
                "Submission form"
                </h2></div>

                <div class="flex flex-row gap-x-2">
                     <button class="bg-gray dark:bg-darksky rounded-md p-2 px-4 border-2 border-tulip my-4 text-lg font-bold"

                    on:click=move |_| {
                        set_form_feed_id(String::from(""));
                        set_form_interval_ms(String::from(""));
                        set_form_password(String::from(""));
                    }
                    disabled=move || !authorised.get()
                     >"Clear all fields"</button>

                        <button class="bg-gray dark:bg-darksky rounded-md p-2 px-4 border-2 border-tulip my-4 text-lg font-bold"
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

                    <button class="bg-gray dark:bg-darksky rounded-md p-2 px-4 border-2 border-tulip my-4 text-lg font-bold"
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
                class= "bg-gray dark:bg-darksky rounded-md p-2 px-4 border-2 border-tulip my-4 text-lg font-bold"
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
                class="bg-gray dark:bg-darksky rounded-md p-2 px-4 border-2 border-tulip my-4 text-lg font-bold"
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
                class= "w-full bg-gray dark:bg-darksky rounded-md p-2 px-4 border-2 border-tulip my-4 text-base h-[400px]"
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

                class="bg-gray dark:bg-darksky rounded-md p-2 px-4 border-2 border-tulip my-4 text-lg font-bold"
                disabled=move || !authorised.get()
            on:click=move |e| {
              let master_creds = master_creds.get();
              let (form_feed_id, form_password, form_interval_ms) = (form_feed_id.get(),
              form_password.get(),
              form_interval_ms.get());

              spawn_local(async move {
                submit_data(master_creds.0, master_creds.1, form_feed_id, form_password, form_interval_ms).await;
                async_data_load.refetch();
              });
            }
                >"Submit"</button>
                        }
                    } else {
                        view! {
                            <>
                            <p>"Not authorised"</p>
                            </>
                        }
                    }
                }
        </main>
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
