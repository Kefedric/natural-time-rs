use chrono::{Datelike, TimeZone, Timelike, Utc};
use solar_calendar_events::{AnnualSolarEvent, DecemberSolstice};
use sun::{SunPhase, time_at_phase};

const MILLISECONDS_PER_DAY: i64 = 86_400_000;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <latitude> <longitude> [precision]", args[0]);
        eprintln!("Example: {} 48.8566 2.3522", args[0]);
        std::process::exit(1);
    }

    let latitude: f64 = args[1].parse().unwrap_or(0.0);
    let mut longitude: f64 = args[2].parse().unwrap_or(0.0);
    let precision: usize = if args.len() > 3 {
        args[3].parse().unwrap_or(0)
    } else {
        0
    };

    // Normalize longitude to [-180, 180]
    longitude = ((longitude + 180.0) % 360.0) - 180.0;

    let now = Utc::now();
    let unix_ms = now.timestamp_millis();

    let natural = NaturalDate::new(unix_ms, latitude, longitude, precision);
    println!("{}", natural);
}

/// Natural Time representation (v0.2)
#[derive(Debug)]
#[allow(dead_code)]
struct NaturalDate {
    latitude: f64,
    longitude: f64,
    precision: usize,
    effective_longitude: f64,

    year: i32,
    moon: u32,
    day_of_moon: u32,
    day_of_year: u32,
    is_rainbow_day: bool,

    time_deg: f64,
    day_of_week: u32,        // 1 to 7

    sunrise_deg: i32,
    sunset_deg: i32,
}

impl NaturalDate {
    fn new(unix_ms: i64, latitude: f64, longitude: f64, precision: usize) -> Self {
        // Effective longitude for Natural Time zone display
        let effective_lon = if precision == 0 {
            longitude.trunc()
        } else if precision == usize::MAX {
            longitude
        } else {
            let factor = 10f64.powi(precision as i32);
            (longitude * factor).trunc() / factor
        };

        let utc_now = Utc.timestamp_millis_opt(unix_ms).unwrap();
        let greg_year = utc_now.year();

        // Astronomical year context
        let mut year_context = calculate_year_context(greg_year - 1, effective_lon);
        let ms_in_year = (year_context.duration * MILLISECONDS_PER_DAY as f64) as i64;

        if unix_ms >= year_context.start + ms_in_year {
            year_context = calculate_year_context(greg_year, effective_lon);
        }

        let year_start = year_context.start;
        let time_since = (unix_ms as f64 - year_start as f64) / MILLISECONDS_PER_DAY as f64;

        // Date calculations
        let day_of_year = time_since.floor() as u32 + 1;
        let is_rainbow = day_of_year > 364;

        let moon = if is_rainbow {
            14
        } else {
            (time_since / 28.0).floor() as u32 + 1
        };

        let day_of_moon = if is_rainbow {
            day_of_year - 364
        } else {
            (time_since % 28.0).floor() as u32 + 1
        };

        let day_of_week = (time_since % 7.0).floor() as u32 + 1; // 1-7

        // Current Natural Time
        let nadir = year_start + time_since.floor() as i64 * MILLISECONDS_PER_DAY;
        let time_deg = ((unix_ms as f64 - nadir as f64) * 360.0 / MILLISECONDS_PER_DAY as f64)
            .rem_euclid(360.0);

        let natural_year = Utc.timestamp_millis_opt(year_start).unwrap().year() - 2012 + 1;

        // Sunrise & Sunset
        let (sunrise_deg, sunset_deg) = calculate_sunrise_sunset_deg(latitude, longitude, nadir);

        Self {
            latitude,
            longitude,
            precision,
            effective_longitude: effective_lon,
            year: natural_year,
            moon,
            day_of_moon,
            day_of_year,
            is_rainbow_day: is_rainbow,
            time_deg,
            day_of_week,
            sunrise_deg,
            sunset_deg,
        }
    }

    fn date_string(&self) -> String {
        if self.is_rainbow_day {
            let plus = if self.day_of_year >= 366 { "+" } else { "" };
            format!("{:03})RAINBOW{}", self.year.abs(), plus)
        } else {
            format!("{:03}){:02}){:02}", self.year.abs(), self.moon, self.day_of_moon)
        }
    }

    fn time_string(&self, decimals: usize) -> String {
        let int_part = self.time_deg.floor() as i32;
        let frac = (self.time_deg.fract() * 10f64.powi(decimals as i32)).round() as i32;
        format!("{:03}°{:0width$}", int_part, frac, width = decimals)
    }

    fn longitude_string(&self) -> String {
        if self.effective_longitude.abs() < 0.1 {
            return "Z".to_string();
        }
        let sign = if self.effective_longitude >= 0.0 { "+" } else { "-" };
        let abs_lon = self.effective_longitude.abs();

        if self.precision == 0 {
            format!("{}{}", sign, abs_lon as i32)
        } else {
            format!("{}{:.prec$}", sign, abs_lon, prec = self.precision)
        }
    }
}

impl std::fmt::Display for NaturalDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} NT{} DOW{} ↑{:03}° ↓{:03}°",
            self.date_string(),
            self.time_string(2),
            self.longitude_string(),
            self.day_of_week,
            self.sunrise_deg,
            self.sunset_deg
        )
    }
}

// ===================================================================
// Astronomical Year
// ===================================================================

#[derive(Clone, Copy)]
struct YearContext {
    start: i64,
    duration: f64,
}

fn calculate_year_context(greg_year: i32, effective_lon: f64) -> YearContext {
    let this_solstice = DecemberSolstice::for_year(greg_year)
        .expect("Failed to calculate December Solstice");

    let next_solstice = DecemberSolstice::for_year(greg_year + 1)
        .expect("Failed to calculate next December Solstice");

    let solstice_dt = this_solstice.date_time();
    let day_adjust = if solstice_dt.hour() >= 12 { 1 } else { 0 };

    let base = Utc
        .with_ymd_and_hms(
            solstice_dt.year(),
            solstice_dt.month(),
            solstice_dt.day() + day_adjust,
            12, 0, 0,
        )
        .unwrap()
        .timestamp_millis();

    let offset_ms = ((-effective_lon + 180.0) * MILLISECONDS_PER_DAY as f64 / 360.0) as i64;
    let start = base + offset_ms;

    let duration_ms = next_solstice.date_time().timestamp_millis()
        - this_solstice.date_time().timestamp_millis();
    let duration_days = duration_ms as f64 / MILLISECONDS_PER_DAY as f64;

    YearContext { start, duration: duration_days }
}

// ===================================================================
// Sunrise / Sunset
// ===================================================================

fn calculate_sunrise_sunset_deg(lat: f64, lon: f64, nadir_ms: i64) -> (i32, i32) {
    let sunrise_ms = time_at_phase(nadir_ms, SunPhase::Sunrise, lat, lon, 0.0);
    let sunset_ms = time_at_phase(nadir_ms, SunPhase::Sunset, lat, lon, 0.0);

    let sunrise_deg = convert_to_natural_deg(sunrise_ms, nadir_ms);
    let sunset_deg = convert_to_natural_deg(sunset_ms, nadir_ms);

    (sunrise_deg.round() as i32, sunset_deg.round() as i32)
}

fn convert_to_natural_deg(event_ms: i64, reference_ms: i64) -> f64 {
    let delta_ms = event_ms - reference_ms;
    let delta_days = delta_ms as f64 / MILLISECONDS_PER_DAY as f64;
    (delta_days * 360.0).rem_euclid(360.0)
}
