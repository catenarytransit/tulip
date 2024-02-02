use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="AdobeFonts" href="https://use.typekit.net/nhx0pgc.css"/>
        <Stylesheet id="leptos" href="/pkg/leptos_start.css"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Router>
            <Routes>
                <Route path="/" view=  move || view! { <Home/> }/>
                <Route path="/gtfsingesterrors" view=  move || view! { <GtfsIngestErrors/> }/>
                <Route path="/kactustimes" view=  move || view! { <KactusTimes/> }/>
                <Route path="/leptosexample" view=  move || view! { <LeptosExample/> }/>
                <Route path="/404.html" view=move || view! { <NotFound/> }/>
            </Routes>
        </Router>
    }
}


#[component]
fn Nav() -> impl IntoView {
    view! {
        <div class="w-full border-b-2 border-gray-200 drop-shadow-2xl flex flex-row align-middle mx-2 my-2 items-center">
        <img src="/LogoSharpCorners.png" class="h-8 "/>
            <img src="/tulip-logo.png" class="h-12"/>
            <p class="text-3xl italic bigmoore align-middle">"Tulip"</p>
        </div>
    }
}

#[component]
fn Home() -> impl IntoView {

    view! {
        
        <Nav/>
        <main>
        </main>

    }
}

#[component]
fn KactusTimes() -> impl IntoView {

    view! {
        
        <Nav/>
        <main>

        </main>

    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct GtfsIngestErrorRow {
    onestop_feed_id: String,
    error: String
}

type FetchErrorsOutput = Result<Vec<GtfsIngestErrorRow>, Box<dyn std::error::Error>>;

async fn fetch_gtfs_ingest_errors() -> FetchErrorsOutput {
    let res = reqwasm::http::Request::get("https://backend.catenarymaps.org/gtfsingesterrors")
    .send().await;

    match res {
        Ok(res) => {
            let output = res.json::<Vec<GtfsIngestErrorRow>>().await.unwrap();

            Ok(output)
        },
        Err(err) => Err(Box::new(err))
    }
}

#[component]
fn GtfsIngestErrors() -> impl IntoView {

    let (gtfs_ingest_errors, set_gtfs_ingest_errors) = create_signal::<Option<Vec<GtfsIngestErrorRow>>>(None);

    let async_data = create_resource(
        gtfs_ingest_errors,
        |value| async move {
            let data_from_backend = fetch_gtfs_ingest_errors().await;

            match data_from_backend {
                Ok(data_from_backend) => {
                    Some(data_from_backend)
                },
                _ => {
                    None
                }
            }
        }
    );

    view! {
        
        <Nav/>
        <main>
        
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
            <p class="px-10 pb-10 text-left">"Tailwind will scan your Rust files for Tailwind class names and compile them into a CSS file."</p>
            <button
                class="bg-amber-600 hover:bg-sky-700 px-5 py-3 text-white rounded-lg"
                on:click=move |_| set_count.update(|count| *count += 1)
            >
                "Something's here | "
                {move || if count() == 0 {
                    "Click me!".to_string()
                } else {
                    count().to_string()
                }}
                " | Some more text"
            </button>
        </main>
    }
}