pub mod app;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;
    use leptos::logging;
    console_error_panic_hook::set_once();

    logging::log!("hydrate mode - hydrating");

    leptos::mount::hydrate_body(App);
}
