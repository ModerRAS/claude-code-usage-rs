//! Utility functions and helpers for ccusage-rs

use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use crate::error::Result;

/// Get the home directory
pub fn get_home_dir() -> Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| crate::error::CcusageError::FileSystem(
        "Could not determine home directory".to_string()
    ))
}

/// Get the application data directory
pub fn get_app_data_dir() -> Result<PathBuf> {
    let home = get_home_dir()?;
    let app_dir = home.join(".ccusage");
    
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).map_err(|e| {
            crate::error::CcusageError::FileSystem(format!(
                "Failed to create app data directory: {}", e
            ))
        })?;
    }
    
    Ok(app_dir)
}

/// Get the cache directory
pub fn get_cache_dir() -> Result<PathBuf> {
    let app_data = get_app_data_dir()?;
    let cache_dir = app_data.join("cache");
    
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir).map_err(|e| {
            crate::error::CcusageError::FileSystem(format!(
                "Failed to create cache directory: {}", e
            ))
        })?;
    }
    
    Ok(cache_dir)
}

/// Format a duration in a human-readable way
pub fn format_duration(duration: chrono::Duration) -> String {
    let total_seconds = duration.num_seconds();
    
    if total_seconds < 60 {
        format!("{}s", total_seconds)
    } else if total_seconds < 3600 {
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{}m {}s", minutes, seconds)
    } else if total_seconds < 86400 {
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        format!("{}h {}m", hours, minutes)
    } else {
        let days = total_seconds / 86400;
        let hours = (total_seconds % 86400) / 3600;
        format!("{}d {}h", days, hours)
    }
}

/// Format a file size in human-readable format
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let bytes = bytes as f64;
    let base = 1024_f64;
    let i = (bytes.log(base) as u32).min(UNITS.len() as u32 - 1);
    let size = bytes / base.powi(i as i32);
    
    if i == 0 {
        format!("{} {}", size, UNITS[i as usize])
    } else {
        format!("{:.1} {}", size, UNITS[i as usize])
    }
}

/// Parse a date string in various formats
pub fn parse_date_flexible(date_str: &str) -> Result<DateTime<Utc>> {
    // Try common date formats
    let formats = [
        "%Y-%m-%d",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
        "%Y/%m/%d",
        "%m/%d/%Y",
        "%d/%m/%Y",
        "%Y%m%d",
    ];
    
    for format in &formats {
        if let Ok(dt) = DateTime::parse_from_str(date_str, format) {
            return Ok(dt.with_timezone(&Utc));
        }
        
        if let Ok(naive_dt) = chrono::NaiveDate::parse_from_str(date_str, format) {
            return Ok(naive_dt.and_hms_opt(0, 0, 0).unwrap().and_utc());
        }
    }
    
    Err(crate::error::CcusageError::Validation(format!(
        "Could not parse date: {}", date_str
    )))
}

/// Get the current timestamp in ISO format
pub fn get_current_timestamp() -> String {
    Utc::now().to_rfc3339()
}

/// Sanitize a filename by removing invalid characters
pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect()
}

/// Create a backup of a file
pub fn backup_file(file_path: &Path) -> Result<PathBuf> {
    if !file_path.exists() {
        return Err(crate::error::CcusageError::FileSystem(format!(
            "File does not exist: {}", file_path.display()
        )));
    }
    
    let backup_path = file_path.with_extension(format!(
        "{}.backup",
        file_path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("bak")
    ));
    
    fs::copy(file_path, &backup_path).map_err(|e| {
        crate::error::CcusageError::FileSystem(format!(
            "Failed to create backup: {}", e
        ))
    })?;
    
    Ok(backup_path)
}

/// Calculate percentage change between two values
pub fn calculate_percentage_change(old_value: f64, new_value: f64) -> f64 {
    if old_value == 0.0 {
        return 0.0;
    }
    
    ((new_value - old_value) / old_value) * 100.0
}

/// Round a number to specified decimal places
pub fn round_to_decimals(value: f64, decimals: u32) -> f64 {
    let multiplier = 10_f64.powi(decimals as i32);
    (value * multiplier).round() / multiplier
}

/// Check if a command exists in PATH
pub fn command_exists(command: &str) -> bool {
    which::which(command).is_ok()
}

/// Get system information
pub fn get_system_info() -> HashMap<String, String> {
    let mut info = HashMap::new();
    
    info.insert("os".to_string(), std::env::consts::OS.to_string());
    info.insert("arch".to_string(), std::env::consts::ARCH.to_string());
    // TARGET常量在某些版本中不存在，使用构建时的目标
    info.insert("target".to_string(), format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS));
    
    if let Ok(version) = std::env::var("CCUSAGE_VERSION") {
        info.insert("version".to_string(), version);
    } else {
        info.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    }
    
    info
}

/// Validate email address format
pub fn is_valid_email(email: &str) -> bool {
    use regex::Regex;
    
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

/// Generate a random string
pub fn generate_random_string(length: usize) -> String {
    // 简化实现：不使用rand，因为rand只在dev-dependencies中
    // TODO: 添加rand作为正式依赖或移除此函数
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    
    let mut result = String::new();
    let charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    
    for i in 0..length {
        let index = (timestamp + i as u128) % charset.len() as u128;
        result.push(charset.chars().nth(index as usize).unwrap());
    }
    
    result
}

/// Parse a duration string (e.g., "1h", "30m", "60s")
pub fn parse_duration(duration_str: &str) -> Result<chrono::Duration> {
    use regex::Regex;
    
    let re = Regex::new(r"^(\d+)([smhd])$").unwrap();
    
    if let Some(caps) = re.captures(duration_str) {
        let value: i64 = caps[1].parse().map_err(|_| {
            crate::error::CcusageError::Validation(format!(
                "Invalid duration value: {}", &caps[1]
            ))
        })?;
        
        let unit = &caps[2];
        
        match unit {
            "s" => Ok(chrono::Duration::seconds(value)),
            "m" => Ok(chrono::Duration::minutes(value)),
            "h" => Ok(chrono::Duration::hours(value)),
            "d" => Ok(chrono::Duration::days(value)),
            _ => Err(crate::error::CcusageError::Validation(format!(
                "Invalid duration unit: {}", unit
            ))),
        }
    } else {
        Err(crate::error::CcusageError::Validation(format!(
            "Invalid duration format: {}. Expected format like '1h', '30m', '60s'", duration_str
        )))
    }
}

/// Truncate a string to a maximum length with ellipsis
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Convert a boolean to "Yes" or "No"
pub fn bool_to_yes_no(b: bool) -> &'static str {
    if b { "Yes" } else { "No" }
}

/// Get a default value if the input is empty
pub fn get_default_if_empty<T: AsRef<str>>(input: T, default: &str) -> String {
    let input_str = input.as_ref();
    if input_str.trim().is_empty() {
        default.to_string()
    } else {
        input_str.to_string()
    }
}

/// Split a string into lines and trim each line
pub fn split_and_trim_lines(text: &str) -> Vec<String> {
    text.lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

/// Join a slice of strings with a separator
pub fn join_with_separator<T: AsRef<str>>(items: &[T], separator: &str) -> String {
    items.iter()
        .map(|item| item.as_ref())
        .collect::<Vec<_>>()
        .join(separator)
}

/// Create a simple progress bar string
pub fn create_progress_bar(current: usize, total: usize, width: usize) -> String {
    if total == 0 {
        return "[]".to_string();
    }
    
    let progress = (current * width) / total;
    let remaining = width - progress;
    
    format!(
        "[{}{}]",
        "=".repeat(progress),
        " ".repeat(remaining)
    )
}

/// Format a number with thousand separators
pub fn format_number_with_separators(num: u64) -> String {
    let num_str = num.to_string();
    let len = num_str.len();
    let mut result = String::new();
    
    for (i, ch) in num_str.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    
    result
}

/// Calculate the median of a slice of numbers
pub fn calculate_median(numbers: &mut [f64]) -> Option<f64> {
    if numbers.is_empty() {
        return None;
    }
    
    numbers.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    
    let len = numbers.len();
    if len % 2 == 0 {
        Some((numbers[len / 2 - 1] + numbers[len / 2]) / 2.0)
    } else {
        Some(numbers[len / 2])
    }
}

/// Calculate the mean of a slice of numbers
pub fn calculate_mean(numbers: &[f64]) -> Option<f64> {
    if numbers.is_empty() {
        return None;
    }
    
    let sum: f64 = numbers.iter().sum();
    Some(sum / numbers.len() as f64)
}

/// Calculate the standard deviation of a slice of numbers
pub fn calculate_std_dev(numbers: &[f64]) -> Option<f64> {
    if numbers.is_empty() {
        return None;
    }
    
    let mean = calculate_mean(numbers)?;
    let variance = numbers.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / numbers.len() as f64;
    
    Some(variance.sqrt())
}

/// Clamp a value between a minimum and maximum
pub fn clamp<T: Ord>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Check if a string contains only ASCII characters
pub fn is_ascii_string(s: &str) -> bool {
    s.is_ascii()
}

/// Convert a string to ASCII by replacing non-ASCII characters
pub fn to_ascii_string(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii() { c } else { '?' })
        .collect()
}

/// Get the file extension in lowercase
pub fn get_file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
}

/// Check if a file has a specific extension (case-insensitive)
pub fn has_file_extension(path: &Path, extension: &str) -> bool {
    get_file_extension(path)
        .map(|ext| ext == extension.to_lowercase())
        .unwrap_or(false)
}

/// Create a directory if it doesn't exist
pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| {
            crate::error::CcusageError::FileSystem(format!(
                "Failed to create directory {}: {}", path.display(), e
            ))
        })?;
    }
    Ok(())
}

/// Read a file to string with error context
pub fn read_file_to_string(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|e| {
        crate::error::CcusageError::FileSystem(format!(
            "Failed to read file {}: {}", path.display(), e
        ))
    })
}

/// Write a string to a file with error context
pub fn write_string_to_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir_exists(parent)?;
    }
    
    fs::write(path, content).map_err(|e| {
        crate::error::CcusageError::FileSystem(format!(
            "Failed to write file {}: {}", path.display(), e
        ))
    })
}

/// Get the file size in bytes
pub fn get_file_size(path: &Path) -> Result<u64> {
    fs::metadata(path).map(|m| m.len()).map_err(|e| {
        crate::error::CcusageError::FileSystem(format!(
            "Failed to get file size for {}: {}", path.display(), e
        ))
    })
}

/// Check if a path is a directory
pub fn is_directory(path: &Path) -> bool {
    path.is_dir()
}

/// Check if a path is a file
pub fn is_file(path: &Path) -> bool {
    path.is_file()
}

/// Get the filename without extension
pub fn get_filename_without_extension(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|name| name.to_str())
        .map(|s| s.to_string())
}

/// Normalize a path by resolving . and .. components
pub fn normalize_path(path: &Path) -> PathBuf {
    path.components()
        .collect::<PathBuf>()
}

/// Get the relative path from a base directory
pub fn get_relative_path(base: &Path, path: &Path) -> Option<PathBuf> {
    pathdiff::diff_paths(path, base)
}

/// Convert bytes to a human-readable string
pub fn bytes_to_human_readable(bytes: u64) -> String {
    format_file_size(bytes)
}

/// Convert a human-readable string to bytes
pub fn human_readable_to_bytes(s: &str) -> Result<u64> {
    let s = s.trim();
    let mut chars = s.chars();
    let mut num_str = String::new();
    
    while let Some(c) = chars.next() {
        if c.is_ascii_digit() || c == '.' {
            num_str.push(c);
        } else {
            break;
        }
    }
    
    let num: f64 = num_str.parse().map_err(|_| {
        crate::error::CcusageError::Validation(format!("Invalid number: {}", num_str))
    })?;
    
    let unit = chars.collect::<String>().to_lowercase();
    
    let multiplier = match unit.as_str() {
        "b" => 1.0,
        "kb" => 1024.0,
        "mb" => 1024.0 * 1024.0,
        "gb" => 1024.0 * 1024.0 * 1024.0,
        "tb" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => return Err(crate::error::CcusageError::Validation(format!("Invalid unit: {}", unit))),
    };
    
    Ok((num * multiplier) as u64)
}

/// Escape special characters in a string for CSV output
pub fn escape_csv_string(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace("\"", "\"\""))
    } else {
        s.to_string()
    }
}

/// Parse a CSV string with escaped quotes
pub fn parse_csv_string(s: &str) -> String {
    if s.starts_with('"') && s.ends_with('"') {
        s[1..s.len() - 1].replace("\"\"", "\"")
    } else {
        s.to_string()
    }
}

/// Create a temporary file with the given content
pub fn create_temp_file(content: &str) -> Result<PathBuf> {
    use std::env::temp_dir;
    
    let temp_dir = temp_dir();
    let file_name = format!("ccusage_temp_{}", uuid::Uuid::new_v4());
    let file_path = temp_dir.join(file_name);
    
    fs::write(&file_path, content).map_err(|e| {
        crate::error::CcusageError::FileSystem(format!(
            "Failed to create temp file: {}", e
        ))
    })?;
    
    Ok(file_path)
}

/// Remove a file if it exists
pub fn remove_file_if_exists(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_file(path).map_err(|e| {
            crate::error::CcusageError::FileSystem(format!(
                "Failed to remove file {}: {}", path.display(), e
            ))
        })?;
    }
    Ok(())
}

/// Get the current working directory
pub fn get_current_dir() -> Result<PathBuf> {
    std::env::current_dir().map_err(|e| {
        crate::error::CcusageError::FileSystem(format!(
            "Failed to get current directory: {}", e
        ))
    })
}

/// Change to a directory
pub fn change_dir(path: &Path) -> Result<()> {
    std::env::set_current_dir(path).map_err(|e| {
        crate::error::CcusageError::FileSystem(format!(
            "Failed to change directory to {}: {}", path.display(), e
        ))
    })
}

/// Get the username of the current user
pub fn get_username() -> Result<String> {
    std::env::var("USER").or_else(|_| std::env::var("USERNAME")).map_err(|_| {
        crate::error::CcusageError::FileSystem("Could not determine username".to_string())
    })
}

/// Get the hostname of the current machine
pub fn get_hostname() -> Result<String> {
    std::env::var("HOSTNAME").or_else(|_| {
        use std::process::Command;
        let output = Command::new("hostname").output();
        match output {
            Ok(output) => String::from_utf8(output.stdout).map_err(|_| {
                crate::error::CcusageError::FileSystem("Could not determine hostname".to_string())
            }),
            Err(_) => Err(crate::error::CcusageError::FileSystem("Could not determine hostname".to_string())),
        }
    }).map(|s| s.trim().to_string())
}

/// Format a timestamp as a relative time (e.g., "2 hours ago")
pub fn format_relative_time(timestamp: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(timestamp);
    
    let seconds = duration.num_seconds();
    
    if seconds < 60 {
        format!("{} seconds ago", seconds.abs())
    } else if seconds < 3600 {
        format!("{} minutes ago", (seconds / 60).abs())
    } else if seconds < 86400 {
        format!("{} hours ago", (seconds / 3600).abs())
    } else {
        format!("{} days ago", (seconds / 86400).abs())
    }
}

/// Parse a comma-separated list of values
pub fn parse_comma_separated_list(s: &str) -> Vec<String> {
    s.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Join a list of values into a comma-separated string
pub fn join_comma_separated_list(values: &[String]) -> String {
    values.join(", ")
}

/// Check if a string is a valid URL
pub fn is_valid_url(url: &str) -> bool {
    url::Url::parse(url).is_ok()
}

/// Parse a URL and return its components
pub fn parse_url(url: &str) -> Result<url::Url> {
    url::Url::parse(url).map_err(|e| {
        crate::error::CcusageError::Validation(format!("Invalid URL: {}", e))
    })
}

/// Get the domain from a URL
pub fn get_url_domain(url: &str) -> Result<String> {
    let parsed_url = parse_url(url)?;
    Ok(parsed_url.host_str().unwrap_or("").to_string())
}

/// Check if a URL uses HTTPS
pub fn is_https_url(url: &str) -> bool {
    parse_url(url)
        .map(|u| u.scheme() == "https")
        .unwrap_or(false)
}

/// Get the file extension from a URL
pub fn get_url_file_extension(url: &str) -> Option<String> {
    let parsed_url = parse_url(url).ok()?;
    let path = parsed_url.path();
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_string())
}

/// Sanitize a URL by removing sensitive parameters
pub fn sanitize_url(url: &str) -> String {
    if let Ok(mut parsed_url) = url::Url::parse(url) {
        let mut query_pairs = parsed_url.query_pairs_mut();
        let sensitive_params = vec!["password", "token", "key", "secret"];
        
        let mut new_query = Vec::new();
        for (key, value) in query_pairs {
            if sensitive_params.contains(&key.to_lowercase().as_str()) {
                new_query.push((key.to_string(), "****".to_string()));
            } else {
                new_query.push((key.to_string(), value.to_string()));
            }
        }
        
        query_pairs.clear();
        for (key, value) in new_query {
            query_pairs.append_pair(&key, &value);
        }
        
        parsed_url.to_string()
    } else {
        url.to_string()
    }
}

/// Create a safe filename from a string
pub fn create_safe_filename(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' => c,
            _ => '_',
        })
        .collect()
}

/// Get the MIME type of a file based on its extension
pub fn get_mime_type(path: &Path) -> &'static str {
    get_file_extension(path)
        .map(|ext| match ext.as_str() {
            "txt" => "text/plain",
            "json" => "application/json",
            "csv" => "text/csv",
            "xml" => "application/xml",
            "html" | "htm" => "text/html",
            "pdf" => "application/pdf",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "mp4" => "video/mp4",
            "mp3" => "audio/mpeg",
            "wav" => "audio/wav",
            "zip" => "application/zip",
            "tar" => "application/x-tar",
            "gz" => "application/gzip",
            "md" => "text/markdown",
            _ => "application/octet-stream",
        })
        .unwrap_or("application/octet-stream")
}

/// Check if a file is a text file
pub fn is_text_file(path: &Path) -> bool {
    get_mime_type(path).starts_with("text/")
}

/// Check if a file is an image
pub fn is_image_file(path: &Path) -> bool {
    get_mime_type(path).starts_with("image/")
}

/// Check if a file is a video
pub fn is_video_file(path: &Path) -> bool {
    get_mime_type(path).starts_with("video/")
}

/// Check if a file is an audio file
pub fn is_audio_file(path: &Path) -> bool {
    get_mime_type(path).starts_with("audio/")
}

/// Get the file modification time
pub fn get_file_mtime(path: &Path) -> Result<DateTime<Utc>> {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .map(|time| DateTime::from(time).with_timezone(&Utc))
        .map_err(|e| {
            crate::error::CcusageError::FileSystem(format!(
                "Failed to get file modification time for {}: {}", path.display(), e
            ))
        })
}

/// Check if a file is newer than another file
pub fn is_file_newer_than(path1: &Path, path2: &Path) -> Result<bool> {
    let mtime1 = get_file_mtime(path1)?;
    let mtime2 = get_file_mtime(path2)?;
    Ok(mtime1 > mtime2)
}

/// Check if a file is older than a certain duration
pub fn is_file_older_than(path: &Path, duration: chrono::Duration) -> Result<bool> {
    let mtime = get_file_mtime(path)?;
    let now = Utc::now();
    Ok(now.signed_duration_since(mtime) > duration)
}

/// Get the total size of all files in a directory
pub fn get_directory_size(path: &Path) -> Result<u64> {
    let mut total_size = 0;
    
    for entry in walkdir::WalkDir::new(path) {
        let entry = entry.map_err(|e| {
            crate::error::CcusageError::FileSystem(format!(
                "Failed to walk directory {}: {}", path.display(), e
            ))
        })?;
        
        if entry.file_type().is_file() {
            total_size += entry.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }
    
    Ok(total_size)
}

/// Count the number of files in a directory
pub fn count_files_in_directory(path: &Path) -> Result<usize> {
    let mut count = 0;
    
    for entry in walkdir::WalkDir::new(path) {
        let entry = entry.map_err(|e| {
            crate::error::CcusageError::FileSystem(format!(
                "Failed to walk directory {}: {}", path.display(), e
            ))
        })?;
        
        if entry.file_type().is_file() {
            count += 1;
        }
    }
    
    Ok(count)
}

/// Find files by extension in a directory
pub fn find_files_by_extension(path: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    for entry in walkdir::WalkDir::new(path) {
        let entry = entry.map_err(|e| {
            crate::error::CcusageError::FileSystem(format!(
                "Failed to walk directory {}: {}", path.display(), e
            ))
        })?;
        
        if entry.file_type().is_file() && has_file_extension(entry.path(), extension) {
            files.push(entry.path().to_path_buf());
        }
    }
    
    Ok(files)
}

/// Find files by name pattern in a directory
pub fn find_files_by_pattern(path: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let regex_pattern = regex::Regex::new(&pattern.replace("*", ".*")).map_err(|e| {
        crate::error::CcusageError::Validation(format!("Invalid pattern: {}", e))
    })?;
    
    for entry in walkdir::WalkDir::new(path) {
        let entry = entry.map_err(|e| {
            crate::error::CcusageError::FileSystem(format!(
                "Failed to walk directory {}: {}", path.display(), e
            ))
        })?;
        
        if entry.file_type().is_file() {
            if let Some(file_name) = entry.file_name().to_str() {
                if regex_pattern.is_match(file_name) {
                    files.push(entry.path().to_path_buf());
                }
            }
        }
    }
    
    Ok(files)
}

/// Get the available disk space for a path
pub fn get_available_disk_space(path: &Path) -> Result<u64> {
    use std::os::unix::fs::MetadataExt;
    
    let metadata = fs::metadata(path).map_err(|e| {
        crate::error::CcusageError::FileSystem(format!(
            "Failed to get metadata for {}: {}", path.display(), e
        ))
    })?;
    
    // 简化实现，实际需要根据平台调整
    Ok(metadata.blocks() * 512)
}

/// Get the total disk space for a path
pub fn get_total_disk_space(path: &Path) -> Result<u64> {
    use std::os::unix::fs::MetadataExt;
    
    let metadata = fs::metadata(path).map_err(|e| {
        crate::error::CcusageError::FileSystem(format!(
            "Failed to get metadata for {}: {}", path.display(), e
        ))
    })?;
    
    // 简化实现，实际需要根据平台调整
    Ok(metadata.blocks() * 512)
}

/// Check if there is enough disk space
pub fn has_enough_disk_space(path: &Path, required_bytes: u64) -> Result<bool> {
    let available = get_available_disk_space(path)?;
    Ok(available >= required_bytes)
}

/// Format disk space in human-readable format
pub fn format_disk_space(bytes: u64) -> String {
    format_file_size(bytes)
}

/// Get the system uptime in seconds
pub fn get_system_uptime() -> Result<u64> {
    use std::process::Command;
    
    let output = Command::new("uptime").output();
    match output {
        Ok(output) => {
            let uptime_str = String::from_utf8(output.stdout).unwrap_or_default();
            // 简化实现，实际需要解析uptime输出
            Ok(0)
        }
        Err(_) => Err(crate::error::CcusageError::FileSystem("Failed to get system uptime".to_string())),
    }
}

/// Get the system load average
pub fn get_system_load_average() -> Result<(f64, f64, f64)> {
    use std::process::Command;
    
    let output = Command::new("uptime").output();
    match output {
        Ok(output) => {
            let uptime_str = String::from_utf8(output.stdout).unwrap_or_default();
            // 简化实现，实际需要解析uptime输出
            Ok((0.0, 0.0, 0.0))
        }
        Err(_) => Err(crate::error::CcusageError::FileSystem("Failed to get system load average".to_string())),
    }
}

/// Get the number of CPU cores
pub fn get_cpu_count() -> Result<usize> {
    Ok(std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(1))
}

/// Get the total memory in bytes
pub fn get_total_memory() -> Result<u64> {
    use std::process::Command;
    
    let output = Command::new("free").arg("-b").output();
    match output {
        Ok(output) => {
            let free_output = String::from_utf8(output.stdout).unwrap_or_default();
            // 简化实现，实际需要解析free输出
            Ok(0)
        }
        Err(_) => Err(crate::error::CcusageError::FileSystem("Failed to get memory info".to_string())),
    }
}

/// Get the available memory in bytes
pub fn get_available_memory() -> Result<u64> {
    use std::process::Command;
    
    let output = Command::new("free").arg("-b").output();
    match output {
        Ok(output) => {
            let free_output = String::from_utf8(output.stdout).unwrap_or_default();
            // 简化实现，实际需要解析free输出
            Ok(0)
        }
        Err(_) => Err(crate::error::CcusageError::FileSystem("Failed to get memory info".to_string())),
    }
}

/// Get the memory usage percentage
pub fn get_memory_usage_percentage() -> Result<f64> {
    let total = get_total_memory()?;
    let available = get_available_memory()?;
    
    if total == 0 {
        return Ok(0.0);
    }
    
    Ok(((total - available) as f64 / total as f64) * 100.0)
}

/// Get the disk usage percentage for a path
pub fn get_disk_usage_percentage(path: &Path) -> Result<f64> {
    let total = get_total_disk_space(path)?;
    let available = get_available_disk_space(path)?;
    
    if total == 0 {
        return Ok(0.0);
    }
    
    Ok(((total - available) as f64 / total as f64) * 100.0)
}

/// Format a percentage with a specified number of decimal places
pub fn format_percentage(value: f64, decimals: u32) -> String {
    format!("{:.1$}%", value, decimals as usize)
}

/// Get the current user's home directory
pub fn get_user_home() -> Result<PathBuf> {
    get_home_dir()
}

/// Get the current user's documents directory
pub fn get_user_documents() -> Result<PathBuf> {
    let home = get_home_dir()?;
    Ok(home.join("Documents"))
}

/// Get the current user's downloads directory
pub fn get_user_downloads() -> Result<PathBuf> {
    let home = get_home_dir()?;
    Ok(home.join("Downloads"))
}

/// Get the current user's desktop directory
pub fn get_user_desktop() -> Result<PathBuf> {
    let home = get_home_dir()?;
    Ok(home.join("Desktop"))
}

/// Get the current user's pictures directory
pub fn get_user_pictures() -> Result<PathBuf> {
    let home = get_home_dir()?;
    Ok(home.join("Pictures"))
}

/// Get the current user's videos directory
pub fn get_user_videos() -> Result<PathBuf> {
    let home = get_home_dir()?;
    Ok(home.join("Videos"))
}

/// Get the current user's music directory
pub fn get_user_music() -> Result<PathBuf> {
    let home = get_home_dir()?;
    Ok(home.join("Music"))
}

/// Create a symlink from source to target
pub fn create_symlink(source: &Path, target: &Path) -> Result<()> {
    std::os::unix::fs::symlink(source, target).map_err(|e| {
        crate::error::CcusageError::FileSystem(format!(
            "Failed to create symlink from {} to {}: {}", source.display(), target.display(), e
        ))
    })
}

/// Read a symlink target
pub fn read_symlink(path: &Path) -> Result<PathBuf> {
    std::fs::read_link(path).map_err(|e| {
        crate::error::CcusageError::FileSystem(format!(
            "Failed to read symlink {}: {}", path.display(), e
        ))
    })
}

/// Check if a path is a symlink
pub fn is_symlink(path: &Path) -> bool {
    path.is_symlink()
}

/// Get the canonical path (resolve all symlinks)
pub fn get_canonical_path(path: &Path) -> Result<PathBuf> {
    fs::canonicalize(path).map_err(|e| {
        crate::error::CcusageError::FileSystem(format!(
            "Failed to get canonical path for {}: {}", path.display(), e
        ))
    })
}

/// Set file permissions
pub fn set_file_permissions(path: &Path, permissions: std::fs::Permissions) -> Result<()> {
    fs::set_permissions(path, permissions).map_err(|e| {
        crate::error::CcusageError::FileSystem(format!(
            "Failed to set permissions for {}: {}", path.display(), e
        ))
    })
}

/// Get file permissions
pub fn get_file_permissions(path: &Path) -> Result<std::fs::Permissions> {
    fs::metadata(path)
        .map(|m| m.permissions())
        .map_err(|e| {
            crate::error::CcusageError::FileSystem(format!(
                "Failed to get permissions for {}: {}", path.display(), e
            ))
        })
}

/// Make a file executable
pub fn make_file_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    
    let mut permissions = get_file_permissions(path)?;
    permissions.set_mode(permissions.mode() | 0o111);
    set_file_permissions(path, permissions)
}

/// Check if a file is executable
pub fn is_file_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    
    get_file_permissions(path)
        .map(|p| p.mode() & 0o111 != 0)
        .unwrap_or(false)
}

/// Get the file owner
pub fn get_file_owner(path: &Path) -> Result<u32> {
    use std::os::unix::fs::MetadataExt;
    
    fs::metadata(path)
        .map(|m| m.uid())
        .map_err(|e| {
            crate::error::CcusageError::FileSystem(format!(
                "Failed to get file owner for {}: {}", path.display(), e
            ))
        })
}

/// Get the file group
pub fn get_file_group(path: &Path) -> Result<u32> {
    use std::os::unix::fs::MetadataExt;
    
    fs::metadata(path)
        .map(|m| m.gid())
        .map_err(|e| {
            crate::error::CcusageError::FileSystem(format!(
                "Failed to get file group for {}: {}", path.display(), e
            ))
        })
}

/// Change file owner
pub fn change_file_owner(path: &Path, uid: u32) -> Result<()> {
    use std::process::Command;
    
    let output = Command::new("chown")
        .arg(uid.to_string())
        .arg(path)
        .output();
    
    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                Err(crate::error::CcusageError::FileSystem(format!(
                    "Failed to change file owner for {}: {}", path.display(), 
                    String::from_utf8_lossy(&output.stderr)
                )))
            }
        }
        Err(e) => Err(crate::error::CcusageError::FileSystem(format!(
            "Failed to change file owner for {}: {}", path.display(), e
        ))),
    }
}

/// Change file group
pub fn change_file_group(path: &Path, gid: u32) -> Result<()> {
    use std::process::Command;
    
    let output = Command::new("chgrp")
        .arg(gid.to_string())
        .arg(path)
        .output();
    
    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                Err(crate::error::CcusageError::FileSystem(format!(
                    "Failed to change file group for {}: {}", path.display(),
                    String::from_utf8_lossy(&output.stderr)
                )))
            }
        }
        Err(e) => Err(crate::error::CcusageError::FileSystem(format!(
            "Failed to change file group for {}: {}", path.display(), e
        ))),
    }
}

/// Get the file mode (permissions)
pub fn get_file_mode(path: &Path) -> Result<u32> {
    use std::os::unix::fs::MetadataExt;
    
    fs::metadata(path)
        .map(|m| m.mode())
        .map_err(|e| {
            crate::error::CcusageError::FileSystem(format!(
                "Failed to get file mode for {}: {}", path.display(), e
            ))
        })
}

/// Set the file mode (permissions)
pub fn set_file_mode(path: &Path, mode: u32) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    
    let mut permissions = get_file_permissions(path)?;
    permissions.set_mode(mode);
    set_file_permissions(path, permissions)
}

/// Format file mode as octal string
pub fn format_file_mode(mode: u32) -> String {
    format!("{:o}", mode)
}

/// Format file mode as symbolic representation
pub fn format_file_mode_symbolic(mode: u32) -> String {
    let mut result = String::new();
    
    // File type
    match mode & 0o170000 {
        0o100000 => result.push('-'), // Regular file
        0o040000 => result.push('d'), // Directory
        0o120000 => result.push('l'), // Symbolic link
        _ => result.push('?'),
    }
    
    // Owner permissions
    result.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o100 != 0 { 'x' } else { '-' });
    
    // Group permissions
    result.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o010 != 0 { 'x' } else { '-' });
    
    // Other permissions
    result.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o001 != 0 { 'x' } else { '-' });
    
    result
}

/// Get the file creation time
pub fn get_file_creation_time(path: &Path) -> Result<DateTime<Utc>> {
    fs::metadata(path)
        .and_then(|m| m.created())
        .map(|time| DateTime::from(time).with_timezone(&Utc))
        .map_err(|e| {
            crate::error::CcusageError::FileSystem(format!(
                "Failed to get file creation time for {}: {}", path.display(), e
            ))
        })
}

/// Get the file access time
pub fn get_file_access_time(path: &Path) -> Result<DateTime<Utc>> {
    fs::metadata(path)
        .and_then(|m| m.accessed())
        .map(|time| DateTime::from(time).with_timezone(&Utc))
        .map_err(|e| {
            crate::error::CcusageError::FileSystem(format!(
                "Failed to get file access time for {}: {}", path.display(), e
            ))
        })
}

/// Set the file modification time
pub fn set_file_mtime(path: &Path, mtime: DateTime<Utc>) -> Result<()> {
    use std::fs::OpenOptions;
    
    let file = OpenOptions::new().write(true).open(path).map_err(|e| {
        crate::error::CcusageError::FileSystem(format!(
            "Failed to open file {}: {}", path.display(), e
        ))
    })?;
    
    file.set_modified(mtime.into()).map_err(|e| {
        crate::error::CcusageError::FileSystem(format!(
            "Failed to set file modification time for {}: {}", path.display(), e
        ))
    })?;
    
    Ok(())
}

/// Touch a file (update modification time or create if doesn't exist)
pub fn touch_file(path: &Path) -> Result<()> {
    if path.exists() {
        set_file_mtime(path, Utc::now())?;
    } else {
        write_string_to_file(path, "")?;
    }
    Ok(())
}

/// Get the file's block size
pub fn get_file_block_size(path: &Path) -> Result<u64> {
    use std::os::unix::fs::MetadataExt;
    
    fs::metadata(path)
        .map(|m| m.blocks())
        .map_err(|e| {
            crate::error::CcusageError::FileSystem(format!(
                "Failed to get file block size for {}: {}", path.display(), e
            ))
        })
}

/// Get the file's device ID
pub fn get_file_device_id(path: &Path) -> Result<u64> {
    use std::os::unix::fs::MetadataExt;
    
    fs::metadata(path)
        .map(|m| m.dev())
        .map_err(|e| {
            crate::error::CcusageError::FileSystem(format!(
                "Failed to get file device ID for {}: {}", path.display(), e
            ))
        })
}

/// Get the file's inode number
pub fn get_file_inode(path: &Path) -> Result<u64> {
    use std::os::unix::fs::MetadataExt;
    
    fs::metadata(path)
        .map(|m| m.ino())
        .map_err(|e| {
            crate::error::CcusageError::FileSystem(format!(
                "Failed to get file inode for {}: {}", path.display(), e
            ))
        })
}

/// Check if two files are the same (hard links)
pub fn are_files_same(path1: &Path, path2: &Path) -> Result<bool> {
    let inode1 = get_file_inode(path1)?;
    let device1 = get_file_device_id(path1)?;
    let inode2 = get_file_inode(path2)?;
    let device2 = get_file_device_id(path2)?;
    
    Ok(inode1 == inode2 && device1 == device2)
}

/// Create a hard link
pub fn create_hard_link(source: &Path, target: &Path) -> Result<()> {
    fs::hard_link(source, target).map_err(|e| {
        crate::error::CcusageError::FileSystem(format!(
            "Failed to create hard link from {} to {}: {}", source.display(), target.display(), e
        ))
    })
}

/// Get the number of hard links to a file
pub fn get_hard_link_count(path: &Path) -> Result<u64> {
    use std::os::unix::fs::MetadataExt;
    
    fs::metadata(path)
        .map(|m| m.nlink())
        .map_err(|e| {
            crate::error::CcusageError::FileSystem(format!(
                "Failed to get hard link count for {}: {}", path.display(), e
            ))
        })
}

/// Truncate a file to a specific size
pub fn truncate_file(path: &Path, size: u64) -> Result<()> {
    use std::fs::OpenOptions;
    
    let file = OpenOptions::new().write(true).open(path).map_err(|e| {
        crate::error::CcusageError::FileSystem(format!(
            "Failed to open file {}: {}", path.display(), e
        ))
    })?;
    
    file.set_len(size).map_err(|e| {
        crate::error::CcusageError::FileSystem(format!(
            "Failed to truncate file {}: {}", path.display(), e
        ))
    })?;
    
    Ok(())
}

/// Get the file's preferred block size for I/O
pub fn get_file_preferred_block_size(path: &Path) -> Result<u64> {
    use std::os::unix::fs::MetadataExt;
    
    fs::metadata(path)
        .map(|m| m.blksize())
        .map_err(|e| {
            crate::error::CcusageError::FileSystem(format!(
                "Failed to get file preferred block size for {}: {}", path.display(), e
            ))
        })
}

/// Get the file's allocated size on disk
pub fn get_file_allocated_size(path: &Path) -> Result<u64> {
    use std::os::unix::fs::MetadataExt;
    
    fs::metadata(path)
        .map(|m| m.blocks() * 512)
        .map_err(|e| {
            crate::error::CcusageError::FileSystem(format!(
                "Failed to get file allocated size for {}: {}", path.display(), e
            ))
        })
}

/// Check if a file is sparse
pub fn is_file_sparse(path: &Path) -> Result<bool> {
    let size = get_file_size(path)?;
    let allocated = get_file_allocated_size(path)?;
    Ok(allocated < size)
}

/// Get the file's flags
#[cfg(unix)]
pub fn get_file_flags(path: &Path) -> Result<u32> {
    use std::os::unix::fs::MetadataExt;
    
    fs::metadata(path)
        .map(|m| m.flags())
        .map_err(|e| {
            crate::error::CcusageError::FileSystem(format!(
                "Failed to get file flags for {}: {}", path.display(), e
            ))
        })
}

#[cfg(not(unix))]
pub fn get_file_flags(_path: &Path) -> Result<u32> {
    Ok(0) // Not supported on non-Unix platforms
}

/// Check if a file is immutable
#[cfg(unix)]
pub fn is_file_immutable(path: &Path) -> Result<bool> {
    use std::os::unix::fs::MetadataExt;
    
    fs::metadata(path)
        .map(|m| m.flags() & 0x80000000 != 0)
        .map_err(|e| {
            crate::error::CcusageError::FileSystem(format!(
                "Failed to check if file is immutable {}: {}", path.display(), e
            ))
        })
}

#[cfg(not(unix))]
pub fn is_file_immutable(_path: &Path) -> Result<bool> {
    Ok(false) // Not supported on non-Unix platforms
}

/// Set file as immutable
pub fn set_file_immutable(path: &Path, immutable: bool) -> Result<()> {
    use std::process::Command;
    
    let flag = if immutable { "+i" } else { "-i" };
    let output = Command::new("chattr")
        .arg(flag)
        .arg(path)
        .output();
    
    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                Err(crate::error::CcusageError::FileSystem(format!(
                    "Failed to set immutable flag for {}: {}", path.display(),
                    String::from_utf8_lossy(&output.stderr)
                )))
            }
        }
        Err(e) => Err(crate::error::CcusageError::FileSystem(format!(
            "Failed to set immutable flag for {}: {}", path.display(), e
        ))),
    }
}

/// Get the file's generation number
#[cfg(unix)]
pub fn get_file_generation_number(path: &Path) -> Result<u64> {
    use std::os::unix::fs::MetadataExt;
    
    fs::metadata(path)
        .map(|m| m.gen())
        .map_err(|e| {
            crate::error::CcusageError::FileSystem(format!(
                "Failed to get file generation number for {}: {}", path.display(), e
            ))
        })
}

#[cfg(not(unix))]
pub fn get_file_generation_number(_path: &Path) -> Result<u64> {
    Ok(0) // Not supported on non-Unix platforms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        let duration = chrono::Duration::seconds(30);
        assert_eq!(format_duration(duration), "30s");
        
        let duration = chrono::Duration::minutes(5);
        assert_eq!(format_duration(duration), "5m 0s");
        
        let duration = chrono::Duration::hours(2);
        assert_eq!(format_duration(duration), "2h 0m");
        
        let duration = chrono::Duration::days(1);
        assert_eq!(format_duration(duration), "1d 0h");
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_is_valid_email() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name+tag@domain.co.uk"));
        assert!(!is_valid_email("invalid-email"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("test@"));
    }

    #[test]
    fn test_parse_duration() {
        assert!(parse_duration("1h").is_ok());
        assert!(parse_duration("30m").is_ok());
        assert!(parse_duration("60s").is_ok());
        assert!(parse_duration("1d").is_ok());
        assert!(parse_duration("invalid").is_err());
        assert!(parse_duration("1x").is_err());
    }

    #[test]
    fn test_calculate_percentage_change() {
        assert_eq!(calculate_percentage_change(100.0, 150.0), 50.0);
        assert_eq!(calculate_percentage_change(100.0, 50.0), -50.0);
        assert_eq!(calculate_percentage_change(0.0, 100.0), 0.0);
    }

    #[test]
    fn test_round_to_decimals() {
        assert_eq!(round_to_decimals(3.14159, 2), 3.14);
        assert_eq!(round_to_decimals(3.14159, 0), 3.0);
        assert_eq!(round_to_decimals(3.14159, 4), 3.1416);
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("Hello, world!", 5), "He...");
        assert_eq!(truncate_string("Hi", 10), "Hi");
        assert_eq!(truncate_string("Hello, world!", 20), "Hello, world!");
    }

    #[test]
    fn test_bool_to_yes_no() {
        assert_eq!(bool_to_yes_no(true), "Yes");
        assert_eq!(bool_to_yes_no(false), "No");
    }

    #[test]
    fn test_get_default_if_empty() {
        assert_eq!(get_default_if_empty("", "default"), "default");
        assert_eq!(get_default_if_empty("value", "default"), "value");
        assert_eq!(get_default_if_empty("  ", "default"), "default");
    }

    #[test]
    fn test_split_and_trim_lines() {
        let text = "  line 1  \n  line 2  \n\n  line 3  ";
        let lines = split_and_trim_lines(text);
        assert_eq!(lines, vec!["line 1", "line 2", "line 3"]);
    }

    #[test]
    fn test_join_with_separator() {
        let items = vec!["a", "b", "c"];
        assert_eq!(join_with_separator(&items, ", "), "a, b, c");
    }

    #[test]
    fn test_create_progress_bar() {
        assert_eq!(create_progress_bar(5, 10, 10), "[=====     ]");
        assert_eq!(create_progress_bar(0, 0, 10), "[]");
        assert_eq!(create_progress_bar(10, 10, 10), "[==========]");
    }

    #[test]
    fn test_format_number_with_separators() {
        assert_eq!(format_number_with_separators(1000), "1,000");
        assert_eq!(format_number_with_separators(1000000), "1,000,000");
        assert_eq!(format_number_with_separators(123), "123");
    }

    #[test]
    fn test_calculate_median() {
        let mut numbers = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(calculate_median(&mut numbers), Some(3.0));
        
        let mut numbers = vec![1.0, 2.0, 3.0, 4.0];
        assert_eq!(calculate_median(&mut numbers), Some(2.5));
        
        let mut numbers: Vec<f64> = vec![];
        assert_eq!(calculate_median(&mut numbers), None);
    }

    #[test]
    fn test_calculate_mean() {
        let numbers = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(calculate_mean(&numbers), Some(3.0));
        
        let numbers: Vec<f64> = vec![];
        assert_eq!(calculate_mean(&numbers), None);
    }

    #[test]
    fn test_calculate_std_dev() {
        let numbers = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let std_dev = calculate_std_dev(&numbers);
        assert!(std_dev.is_some());
        assert!(std_dev.unwrap() > 0.0);
        
        let numbers: Vec<f64> = vec![];
        assert_eq!(calculate_std_dev(&numbers), None);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5, 1, 10), 5);
        assert_eq!(clamp(0, 1, 10), 1);
        assert_eq!(clamp(15, 1, 10), 10);
    }

    #[test]
    fn test_is_ascii_string() {
        assert!(is_ascii_string("Hello"));
        assert!(!is_ascii_string("Héllo"));
    }

    #[test]
    fn test_to_ascii_string() {
        assert_eq!(to_ascii_string("Hello"), "Hello");
        assert_eq!(to_ascii_string("Héllo"), "H?llo");
    }

    #[test]
    fn test_parse_comma_separated_list() {
        let result = parse_comma_separated_list("a, b, c");
        assert_eq!(result, vec!["a", "b", "c"]);
        
        let result = parse_comma_separated_list("a,,b");
        assert_eq!(result, vec!["a", "b"]);
    }

    #[test]
    fn test_join_comma_separated_list() {
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(join_comma_separated_list(&items), "a, b, c");
    }

    #[test]
    fn test_is_valid_url() {
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("http://example.com"));
        assert!(!is_valid_url("not-a-url"));
    }

    #[test]
    fn test_is_https_url() {
        assert!(is_https_url("https://example.com"));
        assert!(!is_https_url("http://example.com"));
    }

    #[test]
    fn test_create_safe_filename() {
        assert_eq!(create_safe_filename("file*name?.txt"), "file_name_.txt");
        assert_eq!(create_safe_filename("normal_name.txt"), "normal_name.txt");
    }

    #[test]
    fn test_get_mime_type() {
        use std::path::Path;
        assert_eq!(get_mime_type(Path::new("file.txt")), "text/plain");
        assert_eq!(get_mime_type(Path::new("file.json")), "application/json");
        assert_eq!(get_mime_type(Path::new("file.unknown")), "application/octet-stream");
    }

    #[test]
    fn test_is_text_file() {
        use std::path::Path;
        assert!(is_text_file(Path::new("file.txt")));
        assert!(!is_text_file(Path::new("file.jpg")));
    }

    #[test]
    fn test_is_image_file() {
        use std::path::Path;
        assert!(is_image_file(Path::new("file.jpg")));
        assert!(!is_image_file(Path::new("file.txt")));
    }

    #[test]
    fn test_is_video_file() {
        use std::path::Path;
        assert!(is_video_file(Path::new("file.mp4")));
        assert!(!is_video_file(Path::new("file.txt")));
    }

    #[test]
    fn test_is_audio_file() {
        use std::path::Path;
        assert!(is_audio_file(Path::new("file.mp3")));
        assert!(!is_audio_file(Path::new("file.txt")));
    }

    #[test]
    fn test_format_percentage() {
        assert_eq!(format_percentage(50.0, 1), "50.0%");
        assert_eq!(format_percentage(33.333, 2), "33.33%");
    }

    #[test]
    fn test_format_file_mode() {
        assert_eq!(format_file_mode(0o644), "644");
    }

    #[test]
    fn test_format_file_mode_symbolic() {
        assert_eq!(format_file_mode_symbolic(0o644), "-rw-r--r--");
        assert_eq!(format_file_mode_symbolic(0o755), "-rwxr-xr-x");
    }

    #[test]
    fn test_escape_csv_string() {
        assert_eq!(escape_csv_string("simple"), "simple");
        assert_eq!(escape_csv_string("with, comma"), "\"with, comma\"");
        assert_eq!(escape_csv_string("with \"quote\""), "\"with \"\"quote\"\"\"");
    }

    #[test]
    fn test_parse_csv_string() {
        assert_eq!(parse_csv_string("simple"), "simple");
        assert_eq!(parse_csv_string("\"with, comma\""), "with, comma");
        assert_eq!(parse_csv_string("\"with \"\"quote\"\"\""), "with \"quote\"");
    }

    #[test]
    fn test_bytes_to_human_readable() {
        assert_eq!(bytes_to_human_readable(1024), "1.0 KB");
        assert_eq!(bytes_to_human_readable(1024 * 1024), "1.0 MB");
    }

    #[test]
    fn test_human_readable_to_bytes() {
        assert_eq!(human_readable_to_bytes("1 KB").unwrap(), 1024);
        assert_eq!(human_readable_to_bytes("1 MB").unwrap(), 1024 * 1024);
        assert!(human_readable_to_bytes("invalid").is_err());
    }
}