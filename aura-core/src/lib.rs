
pub mod atoms;
pub mod utils;
pub mod notification_loop;
pub mod filter;

use tokio::sync::mpsc;
use std::thread;
use std::path::PathBuf;
use std::error::Error;
use x11rb::rust_connection::RustConnection;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt, EventMask, ClientMessageEvent};

#[derive(Debug, Clone)]
pub struct RawIcon {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // RGBA bytes
}

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub xid: u32,
    pub title: String,
    pub class: String, // for instance chrome or some window that is open.. 
    pub is_active: bool, 
    pub icon_path: Option<PathBuf>,
    pub icon_data: Option<RawIcon>,
}
#[derive(Debug)]
pub enum SensorEvent {
    FullScan(Vec<WindowInfo>),
    WindowOpen(WindowInfo),
    WindowClose(u32),
    FocusChange(u32),
}

pub struct Sensor;

impl Sensor {
    pub fn spawn(tx: mpsc::Sender<SensorEvent>) {
        thread::spawn(move || {
            if let Err(e) = notification_loop::run_sensor_loop(tx) {
                eprintln!("CRITICAL: Aura Sensor died! Reason: {}", e);
            }
        });
    }
}

pub fn activate_window(window_id: u32) -> Result<(), Box<dyn Error>> {
    let (conn, _) = RustConnection::connect(None)?;
    let screen = &conn.setup().roots[0];
    let root = screen.root;

    let active_window_atom = conn.intern_atom(false, b"_NET_ACTIVE_WINDOW")?.reply()?.atom;

    let event = ClientMessageEvent {
        response_type: x11rb::protocol::xproto::CLIENT_MESSAGE_EVENT,
        format: 32,
        window: window_id,
        type_: active_window_atom,
        data: x11rb::protocol::xproto::ClientMessageData::from([1, x11rb::CURRENT_TIME, 0, 0, 0]),
        sequence: 0,
    };

    conn.send_event(
        false, 
        root, 
        EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY, 
        event
    )?;
    conn.flush()?;
    Ok(())
}

/// Set a window to be "always on top" using _NET_WM_STATE_ABOVE
pub fn set_always_on_top(window_id: u32) -> Result<(), Box<dyn Error>> {
    let (conn, _) = RustConnection::connect(None)?;
    let screen = &conn.setup().roots[0];
    let root = screen.root;

    let net_wm_state = conn.intern_atom(false, b"_NET_WM_STATE")?.reply()?.atom;
    let net_wm_state_above = conn.intern_atom(false, b"_NET_WM_STATE_ABOVE")?.reply()?.atom;

    // _NET_WM_STATE message: [action, first_property, second_property, source_indication, 0]
    // action: 1 = _NET_WM_STATE_ADD
    let event = ClientMessageEvent {
        response_type: x11rb::protocol::xproto::CLIENT_MESSAGE_EVENT,
        format: 32,
        window: window_id,
        type_: net_wm_state,
        data: x11rb::protocol::xproto::ClientMessageData::from([1u32, net_wm_state_above, 0, 1, 0]),
        sequence: 0,
    };

    conn.send_event(
        false,
        root,
        EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
        event
    )?;
    conn.flush()?;
    Ok(())
}

/// Find a window by title and set it to always-on-top
pub fn set_always_on_top_by_name(title: &str) -> Result<(), Box<dyn Error>> {
    let (conn, screen_num) = RustConnection::connect(None)?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    let atoms = atoms::Atoms::new(&conn)?;
    
    // First try _NET_CLIENT_LIST
    let client_list = utils::get_u32_vector_property(
        &conn, 
        root, 
        atoms.client_list, 
        x11rb::protocol::xproto::AtomEnum::WINDOW.into()
    ).unwrap_or_default();

    for &window_id in &client_list {
        let win_title = utils::get_string_property(&conn, window_id, atoms.wm_name, atoms.utf8_string)
            .or_else(|_| utils::get_string_property(&conn, window_id, atoms.wm_name, atoms.string))
            .unwrap_or_default();

        if win_title == title {
            return set_always_on_top(window_id);
        }
    }

    // Fallback: query all children of root window (for windows not in client list)
    if let Ok(reply) = conn.query_tree(root)?.reply() {
        for &child in &reply.children {
            // Check direct children
            let win_title = utils::get_string_property(&conn, child, atoms.wm_name, atoms.utf8_string)
                .or_else(|_| utils::get_string_property(&conn, child, atoms.wm_name, atoms.string))
                .unwrap_or_default();

            if win_title == title {
                return set_always_on_top(child);
            }

            // Check grandchildren (frame windows often wrap the actual window)
            if let Ok(child_reply) = conn.query_tree(child)?.reply() {
                for &grandchild in &child_reply.children {
                    let win_title = utils::get_string_property(&conn, grandchild, atoms.wm_name, atoms.utf8_string)
                        .or_else(|_| utils::get_string_property(&conn, grandchild, atoms.wm_name, atoms.string))
                        .unwrap_or_default();

                    if win_title == title {
                        // Set on the frame (parent), not the grandchild
                        return set_always_on_top(child);
                    }
                }
            }
        }
    }

    Err(format!("Window with title '{}' not found", title).into())
}

/// Set a window to skip taskbar and pager
pub fn set_skip_taskbar(window_id: u32) -> Result<(), Box<dyn Error>> {
    let (conn, _) = RustConnection::connect(None)?;
    let screen = &conn.setup().roots[0];
    let root = screen.root;

    let net_wm_state = conn.intern_atom(false, b"_NET_WM_STATE")?.reply()?.atom;
    let skip_taskbar = conn.intern_atom(false, b"_NET_WM_STATE_SKIP_TASKBAR")?.reply()?.atom;
    let skip_pager = conn.intern_atom(false, b"_NET_WM_STATE_SKIP_PAGER")?.reply()?.atom;

    // Add SKIP_TASKBAR
    let event1 = ClientMessageEvent {
        response_type: x11rb::protocol::xproto::CLIENT_MESSAGE_EVENT,
        format: 32,
        window: window_id,
        type_: net_wm_state,
        data: x11rb::protocol::xproto::ClientMessageData::from([1u32, skip_taskbar, 0, 1, 0]),
        sequence: 0,
    };

    conn.send_event(
        false,
        root,
        EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
        event1
    )?;

    // Add SKIP_PAGER
    let event2 = ClientMessageEvent {
        response_type: x11rb::protocol::xproto::CLIENT_MESSAGE_EVENT,
        format: 32,
        window: window_id,
        type_: net_wm_state,
        data: x11rb::protocol::xproto::ClientMessageData::from([1u32, skip_pager, 0, 1, 0]),
        sequence: 0,
    };

    conn.send_event(
        false,
        root,
        EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
        event2
    )?;

    conn.flush()?;
    Ok(())
}

/// Find a window by title and set it to skip taskbar/pager
pub fn set_skip_taskbar_by_name(title: &str) -> Result<(), Box<dyn Error>> {
    let (conn, screen_num) = RustConnection::connect(None)?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    let atoms = atoms::Atoms::new(&conn)?;
    
    // First try _NET_CLIENT_LIST
    let client_list = utils::get_u32_vector_property(
        &conn, 
        root, 
        atoms.client_list, 
        x11rb::protocol::xproto::AtomEnum::WINDOW.into()
    ).unwrap_or_default();

    for &window_id in &client_list {
        let win_title = utils::get_string_property(&conn, window_id, atoms.wm_name, atoms.utf8_string)
            .or_else(|_| utils::get_string_property(&conn, window_id, atoms.wm_name, atoms.string))
            .unwrap_or_default();

        if win_title == title {
            return set_skip_taskbar(window_id);
        }
    }

    // Fallback: query all children of root window
    if let Ok(reply) = conn.query_tree(root)?.reply() {
        for &child in &reply.children {
            let win_title = utils::get_string_property(&conn, child, atoms.wm_name, atoms.utf8_string)
                .or_else(|_| utils::get_string_property(&conn, child, atoms.wm_name, atoms.string))
                .unwrap_or_default();

            if win_title == title {
                return set_skip_taskbar(child);
            }

            if let Ok(child_reply) = conn.query_tree(child)?.reply() {
                for &grandchild in &child_reply.children {
                    let win_title = utils::get_string_property(&conn, grandchild, atoms.wm_name, atoms.utf8_string)
                        .or_else(|_| utils::get_string_property(&conn, grandchild, atoms.wm_name, atoms.string))
                        .unwrap_or_default();

                    if win_title == title {
                        return set_skip_taskbar(child);
                    }
                }
            }
        }
    }

    Err(format!("Window with title '{}' not found", title).into())
}