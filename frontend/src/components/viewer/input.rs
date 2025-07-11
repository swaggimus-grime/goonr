use gloo_events::EventListener;
use web_sys::{KeyboardEvent, MouseEvent, HtmlCanvasElement};
use std::rc::Rc;
use std::cell::RefCell;
use glam::{Vec2, Vec3};
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
        listeners.push(EventListener::new(&canvas, "keydown", move |e| {
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
        listeners.push(EventListener::new(&canvas, "keyup", move |e| {
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
            s.mouse_delta = current - s.last_mouse_pos;
            s.last_mouse_pos = current;
        }));

        Self {
            state,
            _listeners: listeners,
        }
    }

    //pub fn update_camera(&self, camera: &mut Camera, dt: f32) {
    //    let mut state = self.state.borrow_mut();
    //    let speed = 2.0;
//
    //    let dir = camera.forward(); // assume your camera has forward/right/up methods
    //    let right = camera.right();
    //    let mut move_vec = Vec3::ZERO;
//
    //    if state.forward {
    //        move_vec += dir;
    //    }
    //    if state.backward {
    //        move_vec -= dir;
    //    }
    //    if state.left {
    //        move_vec -= right;
    //    }
    //    if state.right {
    //        move_vec += right;
    //    }
//
    //    camera.position += move_vec * speed * dt;
//
    //    if state.mouse_pressed {
    //        let delta = state.mouse_delta;
    //        camera.yaw += delta.x * 0.002;
    //        camera.pitch -= delta.y * 0.002;
    //        camera.clamp_pitch();
    //    }
//
    //    state.mouse_delta = Vec2::ZERO;
    //}
}
