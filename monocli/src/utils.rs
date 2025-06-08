pub fn log_message(message: &str) {
    println!("[INFO] {}", message);
}

pub fn log_error(message: &str) {
    eprintln!("[ERROR] {}", message);
}

// Helper function to pad a string with leading zeros
pub trait PadLeft {
    fn pad_left(&self, width: usize, pad: char) -> String;
}

impl PadLeft for str {
    fn pad_left(&self, width: usize, pad: char) -> String {
        format!("{:>width$}", self, width = width).replace(' ', &pad.to_string())
    }
}
