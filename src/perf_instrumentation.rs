use std::sync::Mutex;
use std::time::{Duration};

///! Call this example to print results
///! #[cfg(feature = "performance_stamp")]
///!	PerformanceStamp::print_results()

pub static PERF: Mutex<Vec<PerformanceStamp>> = Mutex::new(vec![]);

pub struct PerformanceStamp {
	pub description: &'static str,
	pub duration: Duration,
}

impl PerformanceStamp {
	pub fn print_results() {
		let perf = PERF.lock().unwrap();

		for result in perf.iter() {
			eprintln!("{} ran for {:?}", result.description, result.duration);
		}
	}
}

/// Creates a performance-timestamp measuring time from the last time this instant was created
#[macro_export]
macro_rules! stamp {
    ($desc:expr, $time_var:ident) => {
		#[cfg(feature = "performance_stamp")]
			{
				crate::perf_instrumentation::PERF.lock().unwrap().push(crate::perf_instrumentation::PerformanceStamp {description: $desc, duration: $time_var.elapsed()});
				$time_var = Instant::now();
		}
	};
}