use std::process::Command;
use std::thread;
use std::time::Duration;

const DOMAIN: &str = "com.apple.universalaccess";
const ARG_CURRENT_HOST: &str = "-currentHost";

pub struct Filters;

impl Filters {
    pub fn new() -> Filters { Filters }

    pub fn set_enabled(&self, enabled: bool) -> Result<(), String> {
        if enabled {
            // Ensure tint type/hue exist, then force an apply via off->on nudge
            self.set_fixed_orange_tint()?;
            write_bool("colorFilterEnabled", false)?;
            // brief delay gives the agent a chance to observe change
            thread::sleep(Duration::from_millis(150));
            write_bool("colorFilterEnabled", true)
        } else {
            write_bool("colorFilterEnabled", false)
        }
    }

    pub fn get_enabled(&self) -> Result<bool, String> { read_bool("colorFilterEnabled") }

    pub fn set_intensity(&self, value_0_1: f32) -> Result<(), String> {
        let clamped = if value_0_1 < 0.0 { 0.0 } else if value_0_1 > 1.0 { 1.0 } else { value_0_1 };
        // Write both the generic filter intensity and the color-tint-specific key (best effort)
        write_float("colorFilterIntensity", clamped)?;
        let _ = write_float("colorTintIntensity", clamped);
        // If enabled, nudge to apply immediately
        if self.get_enabled().unwrap_or(false) {
            write_bool("colorFilterEnabled", false)?;
            thread::sleep(Duration::from_millis(100));
            write_bool("colorFilterEnabled", true)?;
        }
        Ok(())
    }

    pub fn get_intensity_percent(&self) -> Result<i32, String> {
        match read_float("colorFilterIntensity") {
            Ok(v) => Ok((v * 100.0).round() as i32),
            Err(_) => read_float("colorTintIntensity").map(|v| (v * 100.0).round() as i32),
        }
    }

    pub fn set_fixed_orange_tint(&self) -> Result<(), String> {
        // Force the filter type to Color Tint (commonly 4)
        let _ = write_int("colorFilterType", 4);
        // Best-effort set hue to orange. Detect whether the stored hue appears to be degrees (>1.0) or normalized (0..1)
        let hue_key = "colorTintHue";
        let target_deg: f32 = 30.0; // orange-ish
        let target_norm: f32 = target_deg / 360.0;
        match read_float(hue_key) {
            Ok(current) if current > 1.5 => { let _ = write_float(hue_key, target_deg); }
            _ => { let _ = write_float(hue_key, target_norm); }
        }
        Ok(())
    }
}

fn write_bool(key: &str, val: bool) -> Result<(), String> {
    let val_str = if val { "true" } else { "false" };
    // Write both user and ByHost
    let s1 = Command::new("/usr/bin/defaults")
        .args(["write", DOMAIN, key, "-bool", val_str])
        .status()
        .map_err(|e| format!("Failed to execute defaults: {}", e))?;
    let s2 = Command::new("/usr/bin/defaults")
        .args([ARG_CURRENT_HOST, "write", DOMAIN, key, "-bool", val_str])
        .status()
        .map_err(|e| format!("Failed to execute defaults (ByHost): {}", e))?;
    if s1.success() || s2.success() { Ok(()) } else { Err(format!("defaults write failed for {}", key)) }
}

fn write_int(key: &str, val: i32) -> Result<(), String> {
    let s1 = Command::new("/usr/bin/defaults")
        .args(["write", DOMAIN, key, "-int", &val.to_string()])
        .status()
        .map_err(|e| format!("Failed to execute defaults: {}", e))?;
    let s2 = Command::new("/usr/bin/defaults")
        .args([ARG_CURRENT_HOST, "write", DOMAIN, key, "-int", &val.to_string()])
        .status()
        .map_err(|e| format!("Failed to execute defaults (ByHost): {}", e))?;
    if s1.success() || s2.success() { Ok(()) } else { Err(format!("defaults write failed for {}", key)) }
}

fn write_float(key: &str, val: f32) -> Result<(), String> {
    let s1 = Command::new("/usr/bin/defaults")
        .args(["write", DOMAIN, key, "-float", &val.to_string()])
        .status()
        .map_err(|e| format!("Failed to execute defaults: {}", e))?;
    let s2 = Command::new("/usr/bin/defaults")
        .args([ARG_CURRENT_HOST, "write", DOMAIN, key, "-float", &val.to_string()])
        .status()
        .map_err(|e| format!("Failed to execute defaults (ByHost): {}", e))?;
    if s1.success() || s2.success() { Ok(()) } else { Err(format!("defaults write failed for {}", key)) }
}

fn read_bool(key: &str) -> Result<bool, String> {
    let out1 = Command::new("/usr/bin/defaults")
        .args(["read", DOMAIN, key])
        .output()
        .map_err(|e| format!("Failed to execute defaults: {}", e))?;
    let out = if out1.status.success() { out1 } else {
        Command::new("/usr/bin/defaults")
            .args([ARG_CURRENT_HOST, "read", DOMAIN, key])
            .output()
            .map_err(|e| format!("Failed to execute defaults (ByHost): {}", e))?
    };
    if !out.status.success() {
        return Err(format!("defaults read failed for {}", key));
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_lowercase();
    Ok(s == "1" || s == "true")
}

fn read_float(key: &str) -> Result<f32, String> {
    let out1 = Command::new("/usr/bin/defaults")
        .args(["read", DOMAIN, key])
        .output()
        .map_err(|e| format!("Failed to execute defaults: {}", e))?;
    let out = if out1.status.success() { out1 } else {
        Command::new("/usr/bin/defaults")
            .args([ARG_CURRENT_HOST, "read", DOMAIN, key])
            .output()
            .map_err(|e| format!("Failed to execute defaults (ByHost): {}", e))?
    };
    if !out.status.success() {
        return Err(format!("defaults read failed for {}", key));
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    s.parse::<f32>().map_err(|_| format!("Unable to parse float for {}", key))
}
