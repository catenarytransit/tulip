// Copyright Kyler Chin <kyler@catenarymaps.org>
// Catenary Transit Initiatives
// Attribution cannot be removed

#[cfg(feature = "ssr")]
async fn robots(req: actix_web::HttpRequest) -> impl actix_web::Responder {
    let banned_bots = vec![
        "CCBot",
        "ChatGPT-User",
        "GPTBot",
        "Google-Extended",
        "anthropic-ai",
        "ClaudeBot",
        "Omgilibot",
        "Omgili",
        "FacebookBot",
        "Diffbot",
        "Bytespider",
        "ImagesiftBot",
        "cohere-ai",
    ];

    let robots_banned_bots = banned_bots
        .into_iter()
        .map(|x| format!("User-agent: {}\nDisallow: /", x))
        .collect::<Vec<String>>()
        .join("\n\n");

    actix_web::HttpResponse::Ok()
        .insert_header(("Content-Type", "text/plain"))
        .body(robots_banned_bots)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_files::Files;
    use actix_web::*;
    use leptos::prelude::get_configuration;
    use leptos::*;
    use leptos_actix::{LeptosRoutes, generate_route_list};
    use leptos_meta::MetaTags;
    use tulip::app::*;
    // Setting this to None means we'll be using cargo-leptos and its env vars.
    let conf = get_configuration(None).unwrap();

    println!("Configuration {:?}", conf);

    let addr = conf.leptos_options.site_addr;

    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = &leptos_options.site_root;
        let routes = &routes;
        actix_web::App::new()
            .wrap(actix_block_ai_crawling::BlockAi)
            .leptos_routes(routes.to_owned(),
            {let leptos_options = leptos_options.clone();
                 move || {
                use leptos::prelude::*;
                use leptos_meta::*;

                view! {
                    <!DOCTYPE html>
                    <html lang="en">
                        <head>
                            <meta charset="utf-8" />
                            <meta name="viewport" content="width=device-width, initial-scale=1" />
                            <AutoReload options=leptos_options.clone() />
                            <HydrationScripts options=leptos_options.clone()/>
                            <MetaTags/>
                        </head>
                        <body><App />
                        </body>
                        </html>
                }
            }
            })
            .route("robots.txt", web::get().to(robots))
            .service(Files::new("/", site_root.to_string()))
            .wrap(middleware::Compress::default())
    })
    .bind(&addr)?
    .run()
    .await
}
