//! Utility functions
use crate::system::{
    devices::raw::{Result as DeviceResult, Wakeup},
    UEventAction,
};
use std::{collections::HashMap, error::Error, fs, io::prelude::*, path::Path, time::Duration};

/// Technically Linux requires sysfs to be at `/sys`, calling it a system
/// configuration error otherwise.
///
/// But theres an upcoming distro planning to experiment with filesystem layout
/// changes, including of `/sys`, so do this to allow easily changing it.
pub const SYSFS_PATH: &str = "/sys";

/// Kernel Module location. Same reasons as [`SYSFS_PATH`].
pub const MODULE_PATH: &str = "/lib/modules";

/// Device file location. Same reasons as [`SYSFS_PATH`].
pub const DEV_PATH: &str = "/dev";

/// Read a uevent file
///
/// # Arguments
///
/// - `path`, path to the uevent file.
pub fn read_uevent(path: &Path) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in fs::read_to_string(path).unwrap().split_terminator('\n') {
        let line: &str = line;
        let mut i = line.split('=');
        let key = i.next().unwrap().into();
        let val = i.next().unwrap().into();
        map.insert(key, val);
    }
    map
}

/// Write a uevent file
///
/// # Arguments
///
/// - `path`, path to the uevent file.
pub fn write_uevent(
    path: &Path,
    action: UEventAction,
    uuid: Option<String>,
    args: HashMap<String, String>,
) {
    let mut data = String::new();
    match action {
        UEventAction::Add => data.push_str("add"),
        UEventAction::Change => data.push_str("change"),
        UEventAction::Remove => data.push_str("remove"),
    }
    data.push(' ');
    if let Some(uuid) = uuid {
        data.push_str(&uuid);
        data.push(' ');
    }
    for (k, v) in args {
        data.push_str(&k);
        data.push('=');
        data.push_str(&v);
        data.push(' ');
    }
    //
    let mut f = fs::OpenOptions::new().write(true).open(path).unwrap();
    f.write_all(data.trim().as_bytes()).unwrap();
}

pub fn read_power_autosuspend_delay(path: &Path) -> DeviceResult<Option<Duration>> {
    Ok(fs::read_to_string(path.join("power/autosuspend_delay_ms"))
        .map(|s| s.trim().parse())?
        .map(Duration::from_millis)
        .ok())
}

pub fn read_power_async(path: &Path) -> DeviceResult<bool> {
    fs::read_to_string(path.join("power/async")).map(|s| match s.trim() {
        "enabled" => Ok(true),
        "disabled" => Ok(false),
        _ => Err(Box::<dyn Error>::from("")),
    })?
}

pub fn read_power_wakeup(path: &Path) -> DeviceResult<Option<Wakeup>> {
    Ok(Some(Wakeup {
        can_wakeup: fs::read_to_string(path.join("power/wakeup")).map(|s| match s.trim() {
            "enabled" => Ok(true),
            "disabled" => Ok(false),
            _ => Err(Box::<dyn Error>::from("")),
        })??,
        count: fs::read_to_string(path.join("power/wakeup_count"))?
            .trim()
            .parse()
            .map_err(|_| Box::<dyn Error>::from(""))?,
        count_active: fs::read_to_string(path.join("power/wakeup_active_count"))?
            .trim()
            .parse()
            .map_err(|_| Box::<dyn Error>::from(""))?,
    }))
}
