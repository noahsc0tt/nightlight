use nightlight::{NightLight, Schedule, Status, Time};
#[cfg(target_os = "macos")]
use nightlight::ColorFilters;
use std::env::args;
use std::process::exit;

fn print_usage(program: &String) {
    println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    println!("  {}\n", env!("CARGO_PKG_DESCRIPTION"));
    println!("usage:\n  {} [--help] [--filters|-f] <command> [<args>]\n", program);
    println!("Available Commands By Category:");
    println!("\nmanual on/off control (Night Shift by default, Color Filters with -f):");
    println!("  on                       Turn on");
    println!("  off                      Turn off");
    println!("  status                   View current on/off status");
    println!("  toggle                   Toggle on or off based on current status");
    println!("  <noargs>                 Toggle on or off based on current status");
    println!("\nnight shift temperature:");
    println!("  temp                     View temperature preference");
    println!("  temp <0-100|3500K-6500K> Set temperature preference (does not affect on/off)");
    println!("\nautomated schedule (Night Shift only):");
    println!("  schedule                 View the current schedule");
    println!("  schedule start           Start schedule from sunset to sunrise");
    println!("  schedule <from> <to>     Start a custom schedule (12 or 24-hour time format)");
    println!("  schedule stop            Stop the current schedule");
    println!("\ncolor filters (use with -f/--filters):");
    println!("  intensity                View intensity (0-100)");
    println!("  intensity <0-100>        Set intensity (fixed orange tint)");
    println!("  help                     Print this help menu");
}

fn print_status(client: NightLight) -> Result<(), String> {
    let schedule = client.get_schedule()?;
    let status = client.status()?;

    let off_at = match schedule {
        Schedule::SunsetToSunrise => " until sunrise".to_string(),
        Schedule::Off => "".to_string(),
        Schedule::Custom(from_time, to_time) => match status {
            Status::On => format!(" until {}", to_time),
            Status::Off => format!(" until {}", from_time),
        },
    };
    Ok(println!("{}{}", status, off_at))
}

fn toggle(client: NightLight) -> Result<(), String> {
    let status = client.status()?;

    match status {
        Status::On => client.off(),
        Status::Off => client.on(),
    }
}

fn main() {
    let args: Vec<String> = args().collect();

    // Flag detection: -f or --filters switches mode to Color Filters
    let (use_filters, idx) = if args.len() > 1 && (args[1] == "-f" || args[1] == "--filters") {
        (true, 2)
    } else {
        (false, 1)
    };

    if use_filters {
        let filters = ColorFilters::new();
        if args.len() == idx {
            filters.toggle().unwrap_or_else(|e| error(e));
        } else if args.len() == idx + 1 && args[idx] == "on" {
            filters.on().unwrap_or_else(|e| error(e));
        } else if args.len() == idx + 1 && args[idx] == "off" {
            filters.off().unwrap_or_else(|e| error(e));
        } else if args.len() == idx + 1 && args[idx] == "toggle" {
            filters.toggle().unwrap_or_else(|e| error(e));
        } else if args.len() == idx + 1 && args[idx] == "status" {
            match filters.status() { Ok(s) => println!("{}", s), Err(e) => error(e) }
        } else if args.len() == idx + 1 && args[idx] == "intensity" {
            match filters.get_intensity() { Ok(v) => println!("{}", v), Err(e) => error(e) }
        } else if args.len() == idx + 2 && args[idx] == "intensity" {
            let val = args[idx+1].parse::<i32>().unwrap_or(-1);
            filters.set_intensity(val).unwrap_or_else(|e| error(e));
        } else if args.len() == idx + 1 && args[idx] == "help" {
            print_usage(&args[0]);
        } else {
            print_usage(&args[0]);
        }
        return;
    }

    let client = NightLight::new();
    if args.len() == 1 {
        toggle(client).unwrap_or_else(|e| error(e));
    } else if args.len() == 2 && args[1] == "on" {
        client.on().unwrap_or_else(|e| error(e));
    } else if args.len() == 2 && args[1] == "off" {
        client.off().unwrap_or_else(|e| error(e));
    } else if args.len() == 2 && args[1] == "toggle" {
        toggle(client).unwrap_or_else(|e| error(e));
    } else if args.len() == 2 && args[1] == "schedule" {
        match client.get_schedule() {
            Ok(schedule) => println!("{}", schedule),
            Err(e) => error(e),
        }
    } else if args.len() == 3 && args[1] == "schedule" && args[2] == "start" {
        client
            .set_schedule(Schedule::SunsetToSunrise)
            .unwrap_or_else(|e| error(e));
    } else if args.len() == 4 && args[1] == "schedule" {
        set_custom_schedule(client, &args[2], &args[3]).unwrap_or_else(|e| error(e));
    } else if args.len() == 3 && args[1] == "schedule" && args[2] == "stop" {
        client
            .set_schedule(Schedule::Off)
            .unwrap_or_else(|e| error(e));
    } else if args.len() == 2 && args[1] == "status" {
        print_status(client).unwrap_or_else(|e| error(e))
    } else if args.len() == 2 && args[1] == "temp" {
        match client.get_temp() {
            Ok(temp) => println!("{}", temp),
            Err(e) => error(e),
        }
    } else if args.len() == 3 && args[1] == "temp" {
        let temp = temp_userinput(args[2].clone());
        client.set_temp(temp).unwrap_or_else(|e| error(e));
    } else if args.len() == 2 && args[1] == "help" {
        print_usage(&args[0]);
    } else {
        print_usage(&args[0]);
    }
}

fn temp_userinput(input: String) -> i32 {
    if let Some(temp) = input.parse().ok() {
        temp
    } else {
        const KELVIN_LOWER: f64 = 3500.0;
        const KELVIN_UPPER: f64 = 6000.0;

        if input.to_ascii_uppercase().ends_with("K") {
            match input[..(input.len() - 1)].parse::<f64>().ok() {
                Some(kelvin_input) => {
                    // Map kelvin value to 0-100
                    if kelvin_input < KELVIN_LOWER || kelvin_input > KELVIN_UPPER {
                        -1
                    }
                    else {
                        (((kelvin_input - KELVIN_LOWER) / (KELVIN_UPPER - KELVIN_LOWER)) * 100.0) as i32
                    }
                }
                None => -1,
            }
        } else {
            -1
        }
    }
}

fn set_custom_schedule(client: NightLight, from: &String, to: &String) -> Result<(), String> {
    let from = Time::parse(from)?;
    let to = Time::parse(to)?;

    client.set_schedule(Schedule::Custom(from, to))
}

fn error(text: String) {
    eprintln!("{}", text);
    exit(1)
}
