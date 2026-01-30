use crate::WindowInfo;

/// Determines if a window should be excluded from the dock
pub fn should_skip_window(info: &WindowInfo) -> bool {
    let class_lower = info.class.to_lowercase();
    
    // Skip the dock itself
    if info.title == "Aura Dock" 
        || class_lower == "aura-ui"
        || class_lower == "aura_ui" {
        return true;
    }
    
    // Skip GNOME system windows and background apps
    if class_lower == "gjs"  // GNOME JavaScript apps (extensions, shell components)
        || class_lower.contains("gnome-shell")
        || class_lower.contains("gsd-")
        || class_lower == "ibus-extension-gtk3"
        || class_lower == "ibus-ui-gtk3"
        || class_lower.contains("polkit")
        || (info.title.is_empty() && info.class.is_empty()) {
        return true;
    }
    
    false
}
