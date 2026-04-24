use std::time::Instant;

pub struct ThroughputMeter {
    start: Instant,
    bytes: u64,
}

impl ThroughputMeter {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            bytes: 0,
        }
    }

    pub fn add_bytes(&mut self, count: u64) {
        self.bytes += count;
    }

    pub fn bytes_per_second(&self) -> f64 {
        let elapsed = self.start.elapsed().as_secs_f64();
        if elapsed == 0.0 {
            return 0.0;
        }
        self.bytes as f64 / elapsed
    }

    pub fn total_bytes(&self) -> u64 {
        self.bytes
    }

    pub fn elapsed_secs(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }

    pub fn format_speed(&self) -> String {
        let bps = self.bytes_per_second();
        if bps < 1024.0 {
            format!("{:.0} B/s", bps)
        } else if bps < 1024.0 * 1024.0 {
            format!("{:.1} KB/s", bps / 1024.0)
        } else if bps < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1} MB/s", bps / 1024.0 / 1024.0)
        } else {
            format!("{:.1} GB/s", bps / 1024.0 / 1024.0 / 1024.0)
        }
    }

    pub fn reset(&mut self) {
        self.start = Instant::now();
        self.bytes = 0;
    }
}

impl Default for ThroughputMeter {
    fn default() -> Self {
        Self::new()
    }
}

pub fn format_throughput(bytes: u64, elapsed_secs: f64) -> String {
    if elapsed_secs == 0.0 {
        return "N/A".to_string();
    }
    let bps = bytes as f64 / elapsed_secs;
    if bps < 1024.0 {
        format!("{:.0} B/s", bps)
    } else if bps < 1024.0 * 1024.0 {
        format!("{:.1} KB/s", bps / 1024.0)
    } else {
        format!("{:.1} MB/s", bps / 1024.0 / 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_throughput_meter_new() {
        let meter = ThroughputMeter::new();
        assert_eq!(meter.total_bytes(), 0);
        assert_eq!(meter.bytes_per_second(), 0.0);
    }

    #[test]
    fn test_throughput_meter_add_bytes() {
        let mut meter = ThroughputMeter::new();
        meter.add_bytes(1024);
        assert_eq!(meter.total_bytes(), 1024);
    }

    #[test]
    fn test_throughput_meter_default() {
        let meter = ThroughputMeter::default();
        assert_eq!(meter.total_bytes(), 0);
    }

    #[test]
    fn test_throughput_meter_format_speed() {
        let mut meter = ThroughputMeter::new();
        meter.add_bytes(1048576);
        let speed = meter.format_speed();
        assert!(!speed.is_empty());
    }

    #[test]
    fn test_format_throughput_zero() {
        assert_eq!(format_throughput(0, 0.0), "N/A");
    }

    #[test]
    fn test_format_throughput_bytes() {
        let result = format_throughput(100, 1.0);
        assert!(result.contains("B/s"));
    }

    #[test]
    fn test_format_throughput_kilobytes() {
        let result = format_throughput(2048, 1.0);
        assert!(result.contains("KB/s"));
    }

    #[test]
    fn test_format_throughput_megabytes() {
        let result = format_throughput(2097152, 1.0);
        assert!(result.contains("MB/s"));
    }

    #[test]
    fn test_throughput_meter_reset() {
        let mut meter = ThroughputMeter::new();
        meter.add_bytes(1000);
        meter.reset();
        assert_eq!(meter.total_bytes(), 0);
    }

    #[test]
    fn test_throughput_meter_elapsed() {
        let meter = ThroughputMeter::new();
        let elapsed = meter.elapsed_secs();
        assert!(elapsed >= 0.0);
    }
}
