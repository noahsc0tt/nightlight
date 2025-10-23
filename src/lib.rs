#[cfg_attr(target_os = "linux", path = "linux/mod.rs")]
#[cfg_attr(target_os = "macos", path = "macos/mod.rs")]
mod os;
mod schedule;
mod status;
 
 pub use schedule::{Schedule, Time};
 pub use status::Status;
 
 pub struct NightLight {
     client: os::Client,
 }
 
#[cfg(target_os = "macos")]
pub struct ColorFilters {
    client: os::Filters,
}
 
#[cfg(target_os = "macos")]
impl ColorFilters {
    pub fn new() -> ColorFilters {
        ColorFilters { client: os::Filters::new() }
    }
 
    pub fn on(&self) -> Result<(), String> {
        self.client.set_fixed_orange_tint()?;
        self.client.set_enabled(true)
    }
 
    pub fn off(&self) -> Result<(), String> {
        self.client.set_enabled(false)
    }
 
    pub fn toggle(&self) -> Result<(), String> {
        match self.status()? { Status::On => self.off(), Status::Off => self.on() }
    }
 
    pub fn set_intensity(&self, percent: i32) -> Result<(), String> {
        if percent < 0 || percent > 100 { return Err("Intensity must be 0-100".to_string()); }
        self.client.set_fixed_orange_tint()?;
        self.client.set_intensity(percent as f32 / 100.0)
    }
 
    pub fn get_intensity(&self) -> Result<i32, String> {
        self.client.get_intensity_percent()
    }
 
    pub fn status(&self) -> Result<Status, String> {
        Ok(if self.client.get_enabled()? { Status::On } else { Status::Off })
    }
 }



impl NightLight {
    pub fn new() -> NightLight {
        NightLight {
            client: os::Client::new(),
        }
    }

    pub fn on(&self) -> Result<(), String> {
        self.toggle(Status::On)
    }

    pub fn off(&self) -> Result<(), String> {
        self.toggle(Status::Off)
    }

    pub fn toggle(&self, status: Status) -> Result<(), String> {
        match self.client.set_enabled(status.as_bool()) {
            Ok(_) => Ok(()),
            Err(_) => Err(format!("Failed to turn Night Shift {}", status).to_string()),
        }
    }

    pub fn set_schedule(&self, schedule: Schedule) -> Result<(), String> {
        let status = self.status()?;

        match schedule {
            Schedule::Off => self.client.set_mode(0)?,
            Schedule::SunsetToSunrise => self.client.set_mode(1)?,
            Schedule::Custom(from, to) => {
                self.client.set_mode(2)?;
                self.client.set_schedule(from.tuple(), to.tuple())?
            }
        }
        self.toggle(status)
    }

    pub fn get_schedule(&self) -> Result<Schedule, String> {
        let (from_time, to_time) = self.client.get_schedule()?;
        NightLight::schedule(self.client.get_mode()?, from_time, to_time)
    }

    pub fn set_temp(&self, temp: i32) -> Result<(), String> {
        if temp < 0 || temp > 100 {
            return Err("Color temperature must be a number from 0 to 100.".to_string());
        }

        self.client.set_strength(temp as f32 / 100.0)
    }

    pub fn get_temp(&self) -> Result<i32, String> {
        self.client.get_strength()
    }

    pub fn status(&self) -> Result<Status, String> {
        Ok(match self.client.get_enabled()? {
            true => Status::On,
            false => Status::Off,
        })
    }

    fn schedule(mode: i32, from: (u8, u8), to: (u8, u8)) -> Result<Schedule, String> {
        let from = Time::from_tuple(from)?;
        let to = Time::from_tuple(to)?;

        match mode {
            0 => Ok(Schedule::Off),
            2 => Ok(Schedule::Custom(from, to)),
            1 => Ok(Schedule::SunsetToSunrise),
            _ => Err("Unrecognized schedule type".to_string()),
        }
    }
}
