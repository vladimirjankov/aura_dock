use gtk::prelude::*;
use gtk::Box;
use tokio::sync::mpsc;
use std::collections::HashMap;
use std::time::Duration;

use aura_core::{Sensor, SensorEvent};
use crate::dock::{add_window_item, remove_window_item, update_focus};
use crate::window::InputRegionUpdater;

/// Starts the sensor and spawns the event handling loop
pub fn start_sensor_loop(hbox: &Box, region_updater: InputRegionUpdater) {
    let (tx, mut rx) = mpsc::channel(32);
    Sensor::spawn(tx);

    let hbox_weak = hbox.downgrade();

    glib::MainContext::default().spawn_local(async move {
        let mut widgets: HashMap<u32, gtk::Widget> = HashMap::new();

        while let Some(event) = rx.recv().await {
            let Some(hbox) = hbox_weak.upgrade() else { break };
            let mut changed = false;

            match event {
                SensorEvent::FullScan(windows) => {
                    for info in windows {
                        if !widgets.contains_key(&info.xid) {
                            add_window_item(&mut widgets, &hbox, info);
                            changed = true;
                        }
                    }
                }
                SensorEvent::WindowOpen(info) => {
                    add_window_item(&mut widgets, &hbox, info);
                    changed = true;
                }
                SensorEvent::WindowClose(id) => {
                    if remove_window_item(&mut widgets, &hbox, id) {
                        changed = true;
                    }
                }
                SensorEvent::FocusChange(id) => {
                    update_focus(&widgets, id);
                }
            }

            if changed {
                let updater = region_updater.clone();
                glib::timeout_add_local(Duration::from_millis(50), move || updater.update());
            }
        }
    });
}
