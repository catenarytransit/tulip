

#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_files::Files;
    use actix_web::*;
    use leptos::*;
    use tulip::app::*;
        use leptos_actix::{generate_route_list, LeptosRoutes};
            // Setting this to None means we'll be using cargo-leptos and its env vars.
            let conf = get_configuration(None).await.unwrap();

            println!("Configuration {:?}", conf);

            let addr = conf.leptos_options.site_addr;

            // Generate the list of routes in your Leptos App
            let routes = generate_route_list(App);

            HttpServer::new(move || {
                let leptos_options = &conf.leptos_options;
                let site_root = &leptos_options.site_root;
                let routes = &routes;
                App::new()
                    .leptos_routes(
                        leptos_options.to_owned(),
                        routes.to_owned(),
                        App,
                    )
                    .service(Files::new("/", site_root))
                    .wrap(middleware::Compress::default())
            })
            .bind(&addr)?
            .run()
            .await
}

#[cfg(not(any(feature = "ssr", feature = "csr")))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
    // see optional feature `csr` instead
}

#[cfg(all(not(feature = "ssr"), feature = "csr"))]
pub fn main() {
    // a client-side main function is required for using `trunk serve`
    // prefer using `cargo leptos serve` instead
    // to run: `trunk serve --open --features csr`
    use kylerchinmusic::app::*;
    use leptos::*;
    use wasm_bindgen::prelude::wasm_bindgen;

    console_error_panic_hook::set_once();

    leptos::mount_to_body(App);
}
