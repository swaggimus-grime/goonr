use yew::prelude::*;

mod components;
mod route;
mod pages;

use components::*;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}