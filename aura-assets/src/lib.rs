use std::path::PathBuf;
use linicon::IconType;
use std::fs;
use std::env;
use std::process::Command;
use std::sync::OnceLock;
use std::collections::HashMap;
use walkdir::WalkDir;
use freedesktop_entry_parser::parse_entry;

/// Information about an installed application
#[derive(Debug, Clone)]
pub struct AppInfo {
    pub name: String,
    pub exec: String,
    pub icon_name: String,
    pub icon_path: Option<PathBuf>,
    pub desktop_file: PathBuf,
    pub categories: Vec<String>,
}

/// Returns a list of all installed applications
pub fn get_all_apps() -> Vec<AppInfo> {
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    let local_apps = std::path::Path::new(&home).join(".local/share/applications");
    
    let dirs = [
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
        local_apps,
    ];

    let mut apps = Vec::new();
    let mut seen_names: HashMap<String, bool> = HashMap::new();

    for dir in dirs {
        if !dir.exists() { continue; }
        
        for entry in WalkDir::new(dir).max_depth(1) {
            let Ok(entry) = entry else { continue };
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) != Some("desktop") {
                continue;
            }

            if let Ok(desktop_entry) = parse_entry(path) {
                let section = desktop_entry.section("Desktop Entry");
                
                // Skip if NoDisplay or Hidden
                if section.attr("NoDisplay").map(|v| v == "true").unwrap_or(false) {
                    continue;
                }
                if section.attr("Hidden").map(|v| v == "true").unwrap_or(false) {
                    continue;
                }
                
                // Only include Application type
                let entry_type = section.attr("Type").unwrap_or("Application");
                if entry_type != "Application" {
                    continue;
                }

                let name = section.attr("Name").unwrap_or("Unknown").to_string();
                let exec_raw = section.attr("Exec").unwrap_or("").to_string();
                let icon_name = section.attr("Icon").unwrap_or("application-x-executable").to_string();
                let categories_str = section.attr("Categories").unwrap_or("");
                
                // Skip duplicates (prefer earlier entries)
                if seen_names.contains_key(&name) {
                    continue;
                }
                seen_names.insert(name.clone(), true);

                // Clean up exec command - remove field codes
                let exec = clean_exec_command(&exec_raw);
                
                if exec.is_empty() {
                    continue;
                }

                // Parse categories
                let categories: Vec<String> = categories_str
                    .split(';')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();

                // Resolve icon path
                let icon_path = find_in_theme(&icon_name);

                apps.push(AppInfo {
                    name,
                    exec,
                    icon_name,
                    icon_path,
                    desktop_file: path.to_path_buf(),
                    categories,
                });
            }
        }
    }

    // Sort alphabetically by name
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}

/// Cleans an Exec command by removing field codes (%f, %F, %u, %U, etc.)
fn clean_exec_command(exec: &str) -> String {
    let mut result = exec.to_string();
    
    // Remove common field codes
    for code in &["%f", "%F", "%u", "%U", "%d", "%D", "%n", "%N", "%i", "%c", "%k", "%v", "%m"] {
        result = result.replace(code, "");
    }
    
    // Clean up extra whitespace
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn lookup_icon(app_class: &str) -> Option<PathBuf> {
    // 1. Try generic names
    let mut names_to_try = vec![
        app_class.to_string(),
        app_class.to_lowercase(),
    ];

    // 2. Try looking up in the desktop file map
    if let Some(mapped_name) = get_icon_map().get(app_class) {
        names_to_try.insert(0, mapped_name.clone());
    } else if let Some(mapped_name) = get_icon_map().get(&app_class.to_lowercase()) {
        names_to_try.insert(0, mapped_name.clone());
    }

    for name in names_to_try {
        if let Some(path) = find_in_theme(&name) {
            return Some(path);
        }
    }

    None
}

fn get_icon_map() -> &'static HashMap<String, String> {
    static MAP: OnceLock<HashMap<String, String>> = OnceLock::new();
    
    MAP.get_or_init(|| {
        let mut map = HashMap::new();
        
        let home = env::var("HOME").unwrap_or_else(|_| ".".into());
        let local_apps = std::path::Path::new(&home).join(".local/share/applications");
        
        let dirs = [
            PathBuf::from("/usr/share/applications"),
            PathBuf::from("/usr/local/share/applications"),
            local_apps,
        ];

        for dir in dirs {
            if !dir.exists() { continue; }
            
            for entry in WalkDir::new(dir).max_depth(1) {
                let Ok(entry) = entry else { continue };
                let path = entry.path();
                
                if path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                    if let Ok(entry) = parse_entry(path) {
                        let section = entry.section("Desktop Entry");
                        if let Some(icon) = section.attr("Icon") {
                            let icon_name = icon.to_string();
                            
                            // Map StartupWMClass -> Icon
                            if let Some(wm_class) = section.attr("StartupWMClass") {
                                map.insert(wm_class.to_string(), icon_name.clone());
                                map.insert(wm_class.to_lowercase(), icon_name.clone());
                            }
                            
                            // Map Filename (e.g. firefox.desktop -> firefox) -> Icon
                            if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                                map.insert(file_stem.to_string(), icon_name.clone());
                                map.insert(file_stem.to_lowercase(), icon_name.clone());
                            }
                            
                            // Map Name -> Icon
                            if let Some(name) = section.attr("Name") {
                                map.insert(name.to_string(), icon_name.clone());
                                map.insert(name.to_lowercase(), icon_name.clone());
                            }
                        }
                    }
                }
            }
        }
        map
    })
}

fn find_in_theme(name: &str) -> Option<PathBuf> {
    // Check if the name is already an absolute path (some desktop files point to /path/to/icon.png)
    let path = PathBuf::from(name);
    if path.is_absolute() && path.exists() {
        return Some(path);
    }

    let current_theme = get_current_icon_theme();
    let themes = [current_theme, "hicolor"];
    let size: u16 = 48;
    let scale: u16 = 1;

    for theme_name in themes {
        let found = linicon::lookup_icon(theme_name, name, size, scale)
            .ok()
            .into_iter()
            .flat_map(|iter| iter)
            .filter_map(|r| r.ok())
            .find(|icon| matches!(icon.icon_type, IconType::SVG | IconType::PNG))
            .map(|icon| icon.path);

        if let Some(path) = found {
            return Some(path);
        }
    }
    None
}

fn get_current_icon_theme() -> &'static str {
    static THEME: OnceLock<String> = OnceLock::new();
    
    THEME.get_or_init(|| {
        // Try gsettings first
        if let Ok(output) = Command::new("gsettings")
            .args(&["get", "org.gnome.desktop.interface", "icon-theme"])
            .output() 
        {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return s.trim_matches('\'').to_string();
            }
        }

        // Try to find the config file
        let config_dir = env::var("XDG_CONFIG_HOME")
            .map(|p| PathBuf::from(p))
            .unwrap_or_else(|_| {
                let home = env::var("HOME").unwrap_or_else(|_| ".".into());
                std::path::Path::new(&home).join(".config")
            });

        let settings_path = config_dir.join("gtk-3.0/settings.ini");

        if let Ok(content) = fs::read_to_string(settings_path) {
            for line in content.lines() {
                if let Some(val) = line.trim().strip_prefix("gtk-icon-theme-name=") {
                    return val.trim().to_string();
                }
            }
        }
        
        "hicolor".to_string()
    })
}
