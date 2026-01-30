use std::collections::HashSet;
use tokio::sync::mpsc::Sender;
use std::error::Error;
use x11rb::rust_connection::RustConnection;
use crate::atoms::Atoms;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt, EventMask};
use crate::utils::{get_string_property, get_u32_vector_property};
use x11rb::protocol::Event;
use crate::{SensorEvent, WindowInfo, RawIcon};
use crate::filter::should_skip_window;

use aura_assets::lookup_icon;

pub fn run_sensor_loop( tx: Sender<SensorEvent>) -> Result<(), Box<dyn Error>> {

    // x11 connection
    let (conn, screen_num) = RustConnection::connect(None)?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    let atoms = Atoms::new(&conn)?;

    // listen for property changes on the root window
    conn.change_window_attributes(
        root,
        &x11rb::protocol::xproto::ChangeWindowAttributesAux::new()
            .event_mask(EventMask::PROPERTY_CHANGE),
    )?;
    conn.flush()?;

    // set of windows that are known to be open
    let mut known_windows: HashSet<u32> = HashSet::new();

    // get all windows that are open
    let initial_ids = get_u32_vector_property(&conn, root, atoms.client_list, x11rb::protocol::xproto::AtomEnum::WINDOW.into())?;
    for &id in &initial_ids {
        if let Ok(info) = fetch_window_info(&conn, &atoms, id) {
            known_windows.insert(id);
            // Filter out system windows at the source
            if !should_skip_window(&info) {
                let _ = tx.blocking_send(SensorEvent::WindowOpen(info));
            }
        }
    }


    loop {
        let event = match conn.wait_for_event() {
            Ok(event) => event,
            Err(_) => {
                break;
            }
        };

        match event {
            Event::PropertyNotify(e) => {
                if e.atom == atoms.client_list {
                    let current_ids = get_u32_vector_property(&conn, root, atoms.client_list, x11rb::protocol::xproto::AtomEnum::WINDOW.into())
                        .unwrap_or_default();

                    let current_set: HashSet<u32> = current_ids.iter().cloned().collect();

                    // Detect Opened Windows
                    for &id in &current_ids {
                        if !known_windows.contains(&id) {
                            if let Ok(info) = fetch_window_info(&conn, &atoms, id) {
                                known_windows.insert(id);
                                // Filter out system windows at the source
                                if !should_skip_window(&info) {
                                    let _ = tx.blocking_send(SensorEvent::WindowOpen(info));
                                }
                            }
                        }
                    }

                    let to_remove: Vec<u32> = known_windows.difference(&current_set).cloned().collect();
                    for id in to_remove {
                        known_windows.remove(&id);
                        let _ = tx.blocking_send(SensorEvent::WindowClose(id));
                    }
                }

                if e.atom == atoms.active_window {
                    let active_ids = get_u32_vector_property(&conn, root, atoms.active_window, x11rb::protocol::xproto::AtomEnum::WINDOW.into())
                        .unwrap_or_default();
                    
                    if let Some(&active_id) = active_ids.first() {
                        let _ = tx.blocking_send(SensorEvent::FocusChange(active_id));
                    }
                }
            }

            _ => {
                // we dont care.. 
            }

        }
    }

    Ok(())

}

fn fetch_window_info(conn: &RustConnection, atoms: &Atoms, window: u32) -> Result<WindowInfo, Box<dyn Error>> {
    // Fetch Title
    let title = get_string_property(conn, window, atoms.wm_name, atoms.utf8_string)
        .or_else(|_| get_string_property(conn, window, atoms.wm_name, atoms.string))
        .unwrap_or_else(|_| "Unknown".to_string());

    // Fetch Class (App Name)
    // WM_CLASS returns "InstanceName\0ClassName\0"
    let raw_class = get_string_property(conn, window, atoms.wm_class, x11rb::protocol::xproto::AtomEnum::STRING.into())
        .unwrap_or_default();

    // Parse the second part of the string for the class name (e.g., "Firefox")
    let class = raw_class.split('\0')
        .nth(1) // Get the second element
        .unwrap_or(raw_class.split('\0').next().unwrap_or(""))
        .to_string();

    let icon_path = lookup_icon(&class);
    
    // Fetch _NET_WM_ICON if path lookup failed
    let icon_data = if icon_path.is_none() {
        get_net_wm_icon(conn, window, atoms.net_wm_icon).ok().flatten()
    } else {
        None
    };

    Ok(WindowInfo {
        xid: window,
        title,
        class,
        is_active: false,
        icon_path,
        icon_data,
    })
}

fn get_net_wm_icon(conn: &RustConnection, window: u32, atom: x11rb::protocol::xproto::Atom) -> Result<Option<RawIcon>, Box<dyn Error>> {
    let reply = conn.get_property(
        false, 
        window, 
        atom, 
        x11rb::protocol::xproto::AtomEnum::CARDINAL, 
        0, 
        u32::MAX // Read plenty of data
    )?.reply()?;

    if reply.format != 32 || reply.value_len == 0 {
        return Ok(None);
    }

    // Parse CARDINAL array
    let data: Vec<u32> = reply.value32().ok_or("Invalid value32")?.collect();
    
    // Format: width, height, pixels...
    // We can have multiple icons. Let's find the biggest one or closest to 48x48.
    let mut cursor = 0;
    let mut best_icon: Option<RawIcon> = None;
    let mut best_score = 0;

    while cursor < data.len() {
        if cursor + 2 > data.len() { break; }
        let width = data[cursor];
        let height = data[cursor+1];
        let size = (width * height) as usize;
        
        if cursor + 2 + size > data.len() { break; }
        
        // Basic score: prefer larger icons, but maybe closest to 48?
        // Let's just pick the largest one for now to ensure quality.
        let score = width * height;
        if score > best_score {
            let pixels = &data[cursor+2 .. cursor+2+size];
            
            // Convert ARGB (u32) to RGBA (u8 bytes)
            let mut rgba = Vec::with_capacity(size * 4);
            for &p in pixels {
                let a = (p >> 24) as u8;
                let r = (p >> 16) as u8;
                let g = (p >> 8) as u8;
                let b = p as u8;
                rgba.extend_from_slice(&[r, g, b, a]);
            }

            best_icon = Some(RawIcon {
                width,
                height,
                data: rgba,
            });
            best_score = score;
        }

        cursor += 2 + size;
    }

    Ok(best_icon)
}


// pub struct WindowInfo {
//     pub xid: u32,
//     pub title: String,
//     pub class: String, // for instance chrome or some window that is open.. 
//     pub is_active: bool, 
// }
