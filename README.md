# Tulip

Tulip (Transport Unification Live Infrastructure Portal) allows users to browse a human readable version of Catenary data and access API endpoints.

https://tulip.catenarymaps.org

It also allows administrators on the Catenary team to debug and make changes to Catenary's infrastructure and data in real-time, without having to access the Kubernetes / Distributed system database directly. 
This is a powerful tool to manage and diagnose the vast complexity that exists on Catenary Transit Initiatives cloud system.

### Original Leptos Template

This is a template for use with the [Leptos](https://github.com/leptos-rs/leptos) web framework and the [cargo-leptos](https://github.com/akesson/cargo-leptos) tool.

## CLI

If you don't have `cargo-leptos` installed you can install it with

`cargo install cargo-leptos`

## Running your project

`cargo leptos watch`  
By default, you can access your local project at `http://localhost:3000`

## Installing Additional Tools

By default, `cargo-leptos` uses `nightly` Rust, `cargo-generate`, and `sass`. If you run into any trouble, you may need to install one or more of these tools.

1. `rustup toolchain install nightly --allow-downgrade` - make sure you have Rust nightly
2. `rustup target add wasm32-unknown-unknown` - add the ability to compile Rust to WebAssembly
3. `cargo install cargo-generate` - install `cargo-generate` binary (should be installed automatically in future)
4. `npm install -g sass` - install `dart-sass` (should be optional in future)

## Executing a Server on a Remote Machine Without the Toolchain
After running a `cargo leptos build --release` the minimum files needed are:

1. The server binary located in `target/server/release`
2. The `site` directory and all files within located in `target/site`

Copy these files to your remote server. The directory structure should be:
```text
catenarytulip
site/
```
Set the following environment variables (updating for your project as needed):
```sh
export LEPTOS_OUTPUT_NAME="catenarytulip"
export LEPTOS_SITE_ROOT="site"
export LEPTOS_SITE_PKG_DIR="pkg"
export LEPTOS_SITE_ADDR="127.0.0.1:3000"
export LEPTOS_RELOAD_PORT="3001"
```
Finally, run the server binary.

## Notes about CSR and Trunk:
Although it is not recommended, you can also run your project without server integration using the feature `csr` and `trunk serve`:

`trunk serve --open --features csr`

This may be useful for integrating external tools which require a static site, e.g. `tauri`.

## Common Error Debugging

If compilation complains about bad wasm bindgen version, run `cargo install cargo-leptos`
