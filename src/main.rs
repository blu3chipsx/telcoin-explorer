#![allow(non_snake_case)]

mod components;
mod pages;
mod router;
mod services;

use dioxus::prelude::*;
use router::Route;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}
