use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box};
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use aura_core::{set_always_on_top_by_name, set_skip_taskbar_by_name};

pub struct ScreenGeometry {
    pub width: i32,
    pub height: i32,
}

/// Gets the primary monitor's geometry
pub fn get_screen_geometry() -> ScreenGeometry {
    let display = gtk::gdk::Display::default().expect("No display");
    let monitors = display.monitors();
    let monitor = monitors.item(0).unwrap().downcast::<gtk::gdk::Monitor>().unwrap();
    let geometry = monitor.geometry();
    
    ScreenGeometry {
        width: geometry.width(),
        height: geometry.height(),
    }
}

/// Creates and configures the main dock window
pub fn create_dock_window(app: &Application, geometry: &ScreenGeometry) -> ApplicationWindow {
    let window = ApplicationWindow::new(app);
    window.set_default_size(geometry.width, geometry.height);
    window.set_title(Some("Aura Dock"));
    window.set_decorated(false);
    window.set_resizable(false);
    window.set_widget_name("transparent-window");
    
    window
}

/// Sets up X11 window hints (always-on-top, skip-taskbar) after window is realized
pub fn setup_window_hints(window: &ApplicationWindow) {
    window.connect_realize(|_win| {
        let retry_count = Rc::new(Cell::new(0));
        let skip_taskbar_done = Rc::new(Cell::new(false));
        let always_on_top_done = Rc::new(Cell::new(false));
        
        glib::timeout_add_local(Duration::from_millis(500), move || {
            // Try to set skip taskbar
            if !skip_taskbar_done.get() {
                if set_skip_taskbar_by_name("Aura Dock").is_ok() {
                    skip_taskbar_done.set(true);
                }
            }
            
            // Try to set always on top
            if !always_on_top_done.get() {
                if set_always_on_top_by_name("Aura Dock").is_ok() {
                    always_on_top_done.set(true);
                }
            }
            
            // Check if both done or max retries reached
            if skip_taskbar_done.get() && always_on_top_done.get() {
                glib::ControlFlow::Break
            } else {
                let count = retry_count.get();
                if count < 5 {
                    retry_count.set(count + 1);
                    glib::ControlFlow::Continue
                } else {
                    if !skip_taskbar_done.get() {
                        eprintln!("Warning: Could not set skip-taskbar after retries");
                    }
                    if !always_on_top_done.get() {
                        eprintln!("Warning: Could not set always-on-top after retries");
                    }
                    glib::ControlFlow::Break
                }
            }
        });
    });
}

/// Input region updater that can be cloned and called
#[derive(Clone)]
pub struct InputRegionUpdater {
    window_weak: glib::WeakRef<ApplicationWindow>,
    hbox_weak: glib::WeakRef<Box>,
    screen_height: i32,
}

impl InputRegionUpdater {
    pub fn new(window: &ApplicationWindow, hbox: &Box, screen_height: i32) -> Self {
        Self {
            window_weak: window.downgrade(),
            hbox_weak: hbox.downgrade(),
            screen_height,
        }
    }

    pub fn update(&self) -> glib::ControlFlow {
        if let (Some(window), Some(hbox)) = (self.window_weak.upgrade(), self.hbox_weak.upgrade()) {
            if let Some(surface) = window.surface() {
                let (x, y) = hbox.translate_coordinates(&window, 0.0, 0.0).unwrap_or((0.0, 0.0));
                let width = hbox.width();
                let height = hbox.height();

                let region = gtk::cairo::Region::create_rectangle(
                    &gtk::cairo::RectangleInt::new(x as i32, y as i32, width, height)
                );
                
                // Add a thin strip at the bottom for hover detection
                let activation_strip = gtk::cairo::RectangleInt::new(
                    0, 
                    self.screen_height - 10, 
                    window.width(), 
                    10
                );
                region.union_rectangle(&activation_strip).ok();

                surface.set_input_region(&region);
            }
        }
        glib::ControlFlow::Break
    }
}
