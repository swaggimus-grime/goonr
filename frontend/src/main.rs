use stylist::yew::styled_component;
use uuid::Uuid;
use yew::prelude::*;

mod components;
mod route;

use components::*;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}