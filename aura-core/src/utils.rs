use x11rb::rust_connection::RustConnection;
use x11rb::protocol::xproto::{Atom, ConnectionExt};
use std::error::Error;

pub fn get_string_property(conn: &RustConnection, window: u32, property: Atom, type_atom: Atom) -> Result<String, Box<dyn Error>> {
    let reply = conn.get_property(false, window, property, type_atom, 0, 4096)?.reply()?;

    if reply.format != 8 {
        return Err("Invalid property format (expected 8 bytes)".into());
    }
    let value = String::from_utf8_lossy(&reply.value).to_string();

    Ok(value)
}

pub fn get_u32_vector_property(
    conn: &RustConnection,
    window: u32,
    property: Atom,
    type_atom: Atom, 
) -> Result<Vec<u32>, Box<dyn Error>> {
    let reply = conn.get_property(false, window, property, type_atom, 0, 4096)?.reply()?;

    // 2. Validate format (Must be 32-bit data for Window IDs)
    if reply.format != 32 {
        // If the list is empty/missing, just return an empty Vec instead of erroring
        if reply.value_len == 0 {
            return Ok(Vec::new());
        }
        return Err(format!("Invalid property format (expected 32-bit, got {})", reply.format).into());
    }

    // 3. Use the value32() iterator to handle Endianness automatically
    let list: Vec<u32> = reply.value32().ok_or("Failed to parse 32-bit values")?.collect();

    Ok(list)
}