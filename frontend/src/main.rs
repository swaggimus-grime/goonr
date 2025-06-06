use yew::prelude::*;
use yew_router::{BrowserRouter, Switch};
use yew_router::prelude::Link;

mod components;
mod route;

use components::*;
use crate::route::Route;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}