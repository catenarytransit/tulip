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
