use once_cell::sync::Lazy;
use std::env;
use std::fmt::Display;

use crate::http::{HttpMethod, RequestResponse};

static DEV_MODE: Lazy<bool> = Lazy::new(|| {
    env::var("ENV")
        .unwrap_or_else(|_| "development".to_string())
        .to_lowercase()
        == "development"
});

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Info,
    Debug,
    Warning,
    Error,
    Application,
}

static LOGGER_INIT: Lazy<()> = Lazy::new(|| {
    if !*DEV_MODE {
        println!("Note: Development logger is disabled in production mode");
    } else {
        println!("Development logger enabled, to disable set ENV=production");
    }
});

#[derive(Default, Debug, Clone)]
pub struct Logger {
    _private: (),
}

impl Logger {
    pub fn new() -> Self {
        Lazy::force(&LOGGER_INIT);
        Self { _private: () }
    }

    fn format_status(status: u16) -> Option<String> {
        if !*DEV_MODE {
            return None;
        }

        let color = match status {
            s if s >= 500 => ColorCode::BG_RED,
            s if s >= 400 => ColorCode::BG_YELLOW,
            s if s >= 300 => ColorCode::BG_CYAN,
            s if s >= 200 => ColorCode::BG_GREEN,
            _ => ColorCode::BG_BLACK,
        };

        Some(format!("{} {} {}", color.0, status, ColorCode::RESET.0))
    }

    fn format_method(method: HttpMethod) -> Option<String> {
        if !*DEV_MODE {
            return None;
        }

        let (color, padding) = match method {
            HttpMethod::Get => (ColorCode::BG_BLUE, "      "),
            HttpMethod::Post => (ColorCode::BG_CYAN, "     "),
            HttpMethod::Put => (ColorCode::BG_YELLOW, "      "),
            HttpMethod::Patch => (ColorCode::BG_MAGENTA, "    "),
            HttpMethod::Delete => (ColorCode::BG_RED, "   "),
            HttpMethod::Unknown => (ColorCode::BG_BLACK, ""),
        };

        Some(format!(
            "{} {}{} => {}",
            color.0,
            method,
            padding,
            ColorCode::RESET.0
        ))
    }

    pub fn log_request(&self, method: HttpMethod, path: &str, status: u16) {
        if let (Some(method_str), Some(status_str)) =
            (Self::format_method(method), Self::format_status(status))
        {
            println!("{} {} {}", method_str, path, status_str);
        }
    }

    pub fn log_http(request: &RequestResponse) {
        if !*DEV_MODE {
            return;
        }

        let method_str =
            Self::format_method(request.method).unwrap_or_else(|| request.method.to_string());
        let status_str =
            Self::format_status(request.status).unwrap_or_else(|| request.status.to_string());

        println!(
            "{} {} | {} | {} | {}ms",
            method_str,
            request.path,
            request.ip,
            status_str,
            request.duration.as_millis()
        );
    }

    pub fn panic(&self, message: &str) -> ! {
        let (bg_color, fg_color, label) = (ColorCode::BG_RED, ColorCode::FG_RED, "ERROR");
        println!(
            "{} {} {} {} {} {}",
            bg_color.0,
            label,
            ColorCode::RESET.0,
            fg_color.0,
            message,
            ColorCode::RESET.0
        );
        panic!();
    }

    pub fn log(&self, level: LogLevel, message: &str) {
        let (bg_color, fg_color, label) = match level {
            LogLevel::Info => (
                ColorCode::BG_GREEN,
                ColorCode::FG_GREEN,
                format!("{} ", &level.as_str()),
            ),
            LogLevel::Debug => (
                ColorCode::BG_BLUE,
                ColorCode::FG_BLUE,
                format!("{}", &level.as_str()),
            ),
            LogLevel::Warning => (
                ColorCode::BG_YELLOW,
                ColorCode::FG_YELLOW,
                format!("{} ", &level.as_str()),
            ),
            LogLevel::Error => (
                ColorCode::BG_RED,
                ColorCode::FG_RED,
                format!("{}", &level.as_str()),
            ),
            LogLevel::Application => return println!("{}", message),
        };

        if !*DEV_MODE {
            return;
        }

        println!(
            "{} {} {} {} {} {}",
            bg_color.0,
            label,
            ColorCode::RESET.0,
            fg_color.0,
            message,
            ColorCode::RESET.0
        );
    }
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Warning => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Application => "APPLICATION",
        }
    }
}

impl Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

struct ColorCode(&'static str);

impl ColorCode {
    const BG_BLACK: ColorCode = ColorCode("\x1b[40m");
    const BG_RED: ColorCode = ColorCode("\x1b[41m");
    const BG_GREEN: ColorCode = ColorCode("\x1b[42m");
    const BG_YELLOW: ColorCode = ColorCode("\x1b[43m");
    const BG_BLUE: ColorCode = ColorCode("\x1b[44m");
    const BG_MAGENTA: ColorCode = ColorCode("\x1b[45m");
    const BG_CYAN: ColorCode = ColorCode("\x1b[46m");
    const FG_RED: ColorCode = ColorCode("\x1b[31m");
    const FG_GREEN: ColorCode = ColorCode("\x1b[32m");
    const FG_YELLOW: ColorCode = ColorCode("\x1b[33m");
    const FG_BLUE: ColorCode = ColorCode("\x1b[34m");
    const RESET: ColorCode = ColorCode("\x1b[0m");
}
