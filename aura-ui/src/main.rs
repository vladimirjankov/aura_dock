mod app_grid;
mod autohide;
mod dock;
mod search;
mod sensor;
mod style;
mod window;

use gtk::prelude::*;
use gtk::Application;

const APP_ID: &str = "com.vladimir.aura";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    // Get screen geometry
    let geometry = window::get_screen_geometry();

    // Load CSS theme
    style::load_css();

    // Create main window
    let window = window::create_dock_window(app, &geometry);
    
    // Create dock container and app grid
    let (hbox, app_grid_window) = dock::create_dock_container();
    window.set_child(Some(&hbox));
    
    // Position app grid window (will be shown/hidden by button)
    app_grid_window.set_transient_for(Some(&window));

    // Setup X11 window hints (always-on-top, skip-taskbar)
    window::setup_window_hints(&window);

    // Setup auto-hide behavior
    let autohide_state = autohide::AutoHideState::new();
    autohide::setup_hide_checker(&hbox, &autohide_state);
    autohide::setup_motion_controller(&window, &hbox, &autohide_state, geometry.height);

    // Create input region updater for click-through
    let region_updater = window::InputRegionUpdater::new(&window, &hbox, geometry.height);

    // Start sensor and event loop
    sensor::start_sensor_loop(&hbox, region_updater);

    // Show window
    window.present();
}
