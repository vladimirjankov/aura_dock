use gtk::prelude::*;
use gtk::{Button, Image, Box, Orientation, ScrolledWindow, FlowBox, FlowBoxChild, SearchEntry, Label, Window};
use std::cell::RefCell;
use std::rc::Rc;
use std::process::Command;

use aura_assets::{get_all_apps, AppInfo};

/// Creates the app grid button for the dock
pub fn create_app_grid_button(grid_window: Rc<RefCell<Option<Window>>>) -> Button {
    let icon = Image::from_icon_name("view-app-grid-symbolic");
    icon.set_pixel_size(32);

    let button = Button::builder()
        .child(&icon)
        .has_frame(false)
        .css_classes(["dock-item", "app-grid-button"])
        .tooltip_text("Applications")
        .build();

    button.connect_clicked(move |_btn| {
        let window_ref = grid_window.borrow();
        
        if let Some(window) = window_ref.as_ref() {
            if window.is_visible() {
                window.set_visible(false);
            } else {
                window.set_visible(true);
                window.present();
            }
        }
    });

    button
}

/// Creates the app grid overlay window
pub fn create_app_grid_window() -> Window {
    let window = Window::builder()
        .title("Applications")
        .default_width(800)
        .default_height(600)
        .decorated(false)
        .resizable(false)
        .modal(false)
        .build();

    window.set_widget_name("app-grid-window");

    // Main container
    let main_box = Box::new(Orientation::Vertical, 12);
    main_box.set_margin_top(16);
    main_box.set_margin_bottom(16);
    main_box.set_margin_start(16);
    main_box.set_margin_end(16);
    main_box.add_css_class("app-grid-container");

    // Search entry
    let search_entry = SearchEntry::new();
    search_entry.set_placeholder_text(Some("Search applications..."));
    search_entry.add_css_class("app-grid-search");
    main_box.append(&search_entry);

    // Scrolled window for the grid
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .vexpand(true)
        .build();

    // FlowBox for the app grid
    let flow_box = FlowBox::new();
    flow_box.set_valign(gtk::Align::Start);
    flow_box.set_max_children_per_line(8);
    flow_box.set_min_children_per_line(4);
    flow_box.set_selection_mode(gtk::SelectionMode::None);
    flow_box.set_homogeneous(true);
    flow_box.set_row_spacing(8);
    flow_box.set_column_spacing(8);
    flow_box.add_css_class("app-grid-flow");

    // Load all apps
    let apps = get_all_apps();
    let apps_rc = Rc::new(apps);

    // Clone for search filter
    let flow_box_weak = flow_box.downgrade();
    let apps_for_search = apps_rc.clone();
    let window_weak = window.downgrade();

    // Populate the grid
    populate_app_grid(&flow_box, &apps_rc, window_weak.clone());

    // Search filter
    search_entry.connect_search_changed(move |entry| {
        let query = entry.text().to_lowercase();
        
        if let Some(flow_box) = flow_box_weak.upgrade() {
            // Remove all children
            while let Some(child) = flow_box.first_child() {
                flow_box.remove(&child);
            }

            // Filter and re-populate
            let filtered: Vec<&AppInfo> = apps_for_search
                .iter()
                .filter(|app| {
                    if query.is_empty() {
                        true
                    } else {
                        app.name.to_lowercase().contains(&query) ||
                        app.categories.iter().any(|c| c.to_lowercase().contains(&query))
                    }
                })
                .collect();

            for app in filtered {
                let child = create_app_item(app, window_weak.clone());
                flow_box.insert(&child, -1);
            }
        }
    });

    scrolled.set_child(Some(&flow_box));
    main_box.append(&scrolled);

    // Close button area - click outside to close
    let window_weak_close = window.downgrade();
    let gesture = gtk::GestureClick::new();
    gesture.connect_pressed(move |gesture, _n, x, y| {
        if let Some(window) = window_weak_close.upgrade() {
            // Check if click is outside the main content
            let (width, height) = (window.width() as f64, window.height() as f64);
            if x < 16.0 || x > width - 16.0 || y < 16.0 || y > height - 16.0 {
                window.set_visible(false);
            }
        }
        gesture.set_state(gtk::EventSequenceState::Claimed);
    });
    window.add_controller(gesture);

    // Handle Escape key to close
    let key_controller = gtk::EventControllerKey::new();
    let window_weak_key = window.downgrade();
    key_controller.connect_key_pressed(move |_controller, key, _code, _modifiers| {
        if key == gtk::gdk::Key::Escape {
            if let Some(window) = window_weak_key.upgrade() {
                window.set_visible(false);
            }
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    window.add_controller(key_controller);

    window.set_child(Some(&main_box));
    window
}

/// Populates the app grid with app items
fn populate_app_grid(flow_box: &FlowBox, apps: &[AppInfo], window_weak: glib::WeakRef<Window>) {
    for app in apps {
        let child = create_app_item(app, window_weak.clone());
        flow_box.insert(&child, -1);
    }
}

/// Creates a single app item widget
fn create_app_item(app: &AppInfo, window_weak: glib::WeakRef<Window>) -> FlowBoxChild {
    let item_box = Box::new(Orientation::Vertical, 4);
    item_box.set_halign(gtk::Align::Center);
    item_box.set_valign(gtk::Align::Start);
    item_box.add_css_class("app-grid-item");

    // Icon
    let icon = if let Some(path) = &app.icon_path {
        Image::from_file(path)
    } else {
        Image::from_icon_name(&app.icon_name)
    };
    icon.set_pixel_size(64);
    icon.add_css_class("app-grid-icon");

    // Name label
    let label = Label::new(Some(&app.name));
    label.set_max_width_chars(12);
    label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    label.set_wrap(true);
    label.set_wrap_mode(gtk::pango::WrapMode::Word);
    label.set_lines(2);
    label.add_css_class("app-grid-label");

    item_box.append(&icon);
    item_box.append(&label);

    // Make it clickable
    let button = Button::builder()
        .child(&item_box)
        .has_frame(false)
        .css_classes(["app-grid-button-item"])
        .tooltip_text(&app.name)
        .build();

    let exec = app.exec.clone();
    button.connect_clicked(move |_| {
        launch_app(&exec);
        // Close the grid after launching
        if let Some(window) = window_weak.upgrade() {
            window.set_visible(false);
        }
    });

    let child = FlowBoxChild::new();
    child.set_child(Some(&button));
    child
}

/// Launches an application from its exec command
fn launch_app(exec: &str) {
    let parts: Vec<&str> = exec.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    let program = parts[0];
    let args = &parts[1..];

    match Command::new(program).args(args).spawn() {
        Ok(_) => {}
        Err(e) => eprintln!("Failed to launch {}: {}", exec, e),
    }
}
