use colored::*;

pub fn log_error(message: &str) {
	println!("{} {}", "[Error]".red(), message);
}

pub fn log_warning(message: &str) {
	println!("{} {}", "[Warning]".yellow(), message);
}

pub fn log_info(message: &str) {
	println!("{} {}", "[Info]", message);
}