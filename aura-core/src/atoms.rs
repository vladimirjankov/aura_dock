
use std::error::Error;
use x11rb::protocol::xproto::{Atom, ConnectionExt};
use x11rb::rust_connection::RustConnection;
pub struct Atoms {
    pub client_list: Atom,
    pub active_window: Atom,
    pub wm_name: Atom,
    pub wm_class: Atom,
    pub utf8_string: Atom,
    pub string: Atom,
    pub net_wm_icon: Atom,
}

impl Atoms {
    pub fn new(conn: &RustConnection) -> Result<Box<Self>, Box<dyn Error>>{
    
        let client_list = conn.intern_atom(false, b"_NET_CLIENT_LIST")?;
        let active_window = conn.intern_atom(false, b"_NET_ACTIVE_WINDOW")?;
        let wm_name = conn.intern_atom(false, b"WM_NAME")?;
        let wm_class = conn.intern_atom(false, b"WM_CLASS")?;
        let utf8_string = conn.intern_atom(false, b"UTF8_STRING")?;
        let string_cookie = conn.intern_atom(false, b"STRING")?;
        let net_wm_icon = conn.intern_atom(false, b"_NET_WM_ICON")?;
        
        Ok(Box::new( Self{
            client_list: client_list.reply()?.atom,
            active_window: active_window.reply()?.atom,
            wm_name: wm_name.reply()?.atom,
            wm_class: wm_class.reply()?.atom,
            utf8_string: utf8_string.reply()?.atom,
            string: string_cookie.reply()?.atom,
            net_wm_icon: net_wm_icon.reply()?.atom,
        }))
    }
}