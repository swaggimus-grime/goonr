use gloo_events::EventListener;
use web_sys::{KeyboardEvent, MouseEvent, HtmlCanvasElement};
use std::rc::Rc;
use std::cell::RefCell;
use glam::{Vec2, Vec3};
use gloo::utils::window;
use gloo_console::log;
use wasm_bindgen::JsCast;

#[derive(Default)]
pub struct InputState {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub mouse_pressed: bool,
    pub last_mouse_pos: Vec2,
    pub mouse_delta: Vec2,
}

pub struct InputController {
    pub state: Rc<RefCell<InputState>>,
    _listeners: Vec<EventListener>,
}

impl InputController {
    pub fn new(canvas: HtmlCanvasElement) -> Self {
        let state = Rc::new(RefCell::new(InputState::default()));
        let mut listeners = Vec::new();

        // Clone handles for events
        let state_kb = state.clone();
        listeners.push(EventListener::new(&window(), "keydown", move |e| {
            let event = e.dyn_ref::<KeyboardEvent>().unwrap();
            let mut s = state_kb.borrow_mut();
            match event.key().as_str() {
                "w" | "ArrowUp" => s.forward = true,
                "s" | "ArrowDown" => s.backward = true,
                "a" | "ArrowLeft" => s.left = true,
                "d" | "ArrowRight" => s.right = true,
                _ => {}
            }
        }));

        let state_kb = state.clone();
        listeners.push(EventListener::new(&window(), "keyup", move |e| {
            let event = e.dyn_ref::<KeyboardEvent>().unwrap();
            let mut s = state_kb.borrow_mut();
            match event.key().as_str() {
                "w" | "ArrowUp" => s.forward = false,
                "s" | "ArrowDown" => s.backward = false,
                "a" | "ArrowLeft" => s.left = false,
                "d" | "ArrowRight" => s.right = false,
                _ => {}
            }
        }));

        // Mouse down
        let state_mouse = state.clone();
        listeners.push(EventListener::new(&canvas, "mousedown", move |_e| {
            state_mouse.borrow_mut().mouse_pressed = true;
        }));

        // Mouse up
        let state_mouse = state.clone();
        listeners.push(EventListener::new(&canvas, "mouseup", move |_e| {
            state_mouse.borrow_mut().mouse_pressed = false;
        }));

        // Mouse move
        let state_mouse = state.clone();
        listeners.push(EventListener::new(&canvas, "mousemove", move |e| {
            let event = e.dyn_ref::<MouseEvent>().unwrap();
            let mut s = state_mouse.borrow_mut();
            let current = Vec2::new(event.client_x() as f32, event.client_y() as f32);
            if s.mouse_pressed {
                // Calculate delta only if pressed, otherwise zero delta (or ignore)
                s.mouse_delta = current - s.last_mouse_pos;
            } else {
                s.mouse_delta = Vec2::ZERO;
            }
            s.last_mouse_pos = current;
        }));

        Self {
            state,
            _listeners: listeners,
        }
    }
}
