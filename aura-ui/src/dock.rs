use gtk::prelude::*;
use gtk::{Box, Orientation, Image, Button, Window};
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use aura_core::WindowInfo;
use crate::search::create_search_bar;
use crate::app_grid::{create_app_grid_button, create_app_grid_window};

/// Creates the dock container with search bar and app grid button
pub fn create_dock_container() -> (Box, Window) {
    let hbox = Box::new(Orientation::Horizontal, 8);
    hbox.set_widget_name("dock-container");
    hbox.set_halign(gtk::Align::Center);
    hbox.set_valign(gtk::Align::End);
    hbox.set_margin_bottom(10);

    // Create app grid window
    let app_grid_window = create_app_grid_window();
    let grid_window_rc = Rc::new(RefCell::new(Some(app_grid_window.clone())));

    // Add app grid button
    let app_grid_button = create_app_grid_button(grid_window_rc);
    hbox.append(&app_grid_button);

    // Add search bar
    let search_entry = create_search_bar();
    hbox.append(&search_entry);

    (hbox, app_grid_window)
}

/// Creates an icon widget from window info
fn create_icon_widget(info: &WindowInfo) -> Image {
    let icon_widget = if let Some(path) = &info.icon_path {
        Image::from_file(path)
    } else if let Some(raw) = &info.icon_data {
        let bytes = glib::Bytes::from(&raw.data);
        let pixbuf = gtk::gdk_pixbuf::Pixbuf::from_bytes(
            &bytes,
            gtk::gdk_pixbuf::Colorspace::Rgb,
            true,
            8,
            raw.width as i32,
            raw.height as i32,
            (raw.width * 4) as i32
        );
        let texture = gtk::gdk::Texture::for_pixbuf(&pixbuf);
        Image::from_paintable(Some(&texture))
    } else {
        Image::from_icon_name("application-x-executable")
    };
    
    icon_widget.set_pixel_size(48);
    icon_widget
}

/// Adds a new window item to the dock
pub fn add_window_item(widgets: &mut HashMap<u32, gtk::Widget>, hbox: &Box, info: WindowInfo) {
    let xid = info.xid;
    let icon_widget = create_icon_widget(&info);

    let tooltip = format!(
        "Title: {}\nClass: {}\nIcon: {:?}",
        info.title, info.class, info.icon_path
    );

    let button = Button::builder()
        .child(&icon_widget)
        .has_frame(false)
        .css_classes(["dock-item"])
        .tooltip_text(&tooltip)
        .build();

    button.connect_clicked(move |_| {
        if let Err(e) = aura_core::activate_window(xid) {
            eprintln!("Failed to activate window: {}", e);
        }
    });

    hbox.append(&button);
    widgets.insert(xid, button.upcast());
}

/// Removes a window item from the dock
pub fn remove_window_item(widgets: &mut HashMap<u32, gtk::Widget>, hbox: &Box, id: u32) -> bool {
    if let Some(widget) = widgets.remove(&id) {
        hbox.remove(&widget);
        true
    } else {
        false
    }
}

/// Updates focus styling on window items
pub fn update_focus(widgets: &HashMap<u32, gtk::Widget>, focused_id: u32) {
    for (w_id, widget) in widgets {
        if *w_id == focused_id {
            widget.add_css_class("active-window");
        } else {
            widget.remove_css_class("active-window");
        }
    }
}
