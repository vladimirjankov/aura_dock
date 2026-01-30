use gtk::prelude::*;
use gtk::{ApplicationWindow, Box};
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

const HIDE_THRESHOLD: i32 = 100;  // Pixels from bottom to trigger show
const HIDE_DELAY_MS: u64 = 500;   // Delay before hiding

/// State for auto-hide functionality
pub struct AutoHideState {
    pub is_visible: Rc<Cell<bool>>,
    pub should_hide: Rc<Cell<bool>>,
}

impl AutoHideState {
    pub fn new() -> Self {
        Self {
            is_visible: Rc::new(Cell::new(true)),
            should_hide: Rc::new(Cell::new(false)),
        }
    }
}

/// Sets up the periodic hide checker
pub fn setup_hide_checker(hbox: &Box, state: &AutoHideState) {
    let hbox_weak = hbox.downgrade();
    let is_visible = state.is_visible.clone();
    let should_hide = state.should_hide.clone();
    
    glib::timeout_add_local(Duration::from_millis(100), move || {
        if should_hide.get() && is_visible.get() {
            if let Some(hbox) = hbox_weak.upgrade() {
                hbox.add_css_class("dock-hidden");
                is_visible.set(false);
                should_hide.set(false);
            }
        }
        glib::ControlFlow::Continue
    });
}

/// Sets up the motion controller for auto-hide
pub fn setup_motion_controller(window: &ApplicationWindow, hbox: &Box, state: &AutoHideState, screen_height: i32) {
    let motion_controller = gtk::EventControllerMotion::new();
    
    let hbox_weak = hbox.downgrade();
    let is_visible = state.is_visible.clone();
    let should_hide = state.should_hide.clone();

    motion_controller.connect_motion(move |_ctrl, _x, y| {
        let near_bottom = y > (screen_height - HIDE_THRESHOLD) as f64;

        if let Some(hbox) = hbox_weak.upgrade() {
            if near_bottom {
                // Cancel pending hide and show the dock
                should_hide.set(false);
                if !is_visible.get() {
                    hbox.remove_css_class("dock-hidden");
                    is_visible.set(true);
                }
            } else if is_visible.get() {
                // Schedule hide
                let should_hide_delay = should_hide.clone();
                glib::timeout_add_local(Duration::from_millis(HIDE_DELAY_MS), move || {
                    should_hide_delay.set(true);
                    glib::ControlFlow::Break
                });
            }
        }
    });

    window.add_controller(motion_controller);
}
