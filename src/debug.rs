use colored::*;

#[allow(dead_code)]
pub fn log_error(message: &str) {
    println!("{} {}", "[Error]".red(), message);
}

#[allow(dead_code)]
pub fn log_warning(message: &str) {
    println!("{} {}", "[Warning]".yellow(), message);
}

#[allow(dead_code)]
pub fn log_info(message: &str) {
    println!("{} {}", "[Info]", message);
}
