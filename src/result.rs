use std::{ops::Div, time::Duration};

pub struct TestResult {
    title: String,
    n_tests: usize,
    num_elements: usize,
    elapsed: Duration,
    elapsed_mean: Duration,
}

impl TestResult {
    pub fn new(
        title: impl Into<String>,
        n_tests: usize,
        num_elements: usize,
        elapsed: Duration,
    ) -> Self {
        let elapsed_mean = elapsed.div(n_tests.try_into().unwrap());
        Self {
            title: title.into(),
            n_tests,
            num_elements,
            elapsed,
            elapsed_mean,
        }
    }
}

impl std::fmt::Display for TestResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} elements   \t| elapsed ({} messages)={:?}\t| elapsed (mean)={:?}",
            self.title, self.num_elements, self.n_tests, self.elapsed, self.elapsed_mean
        )
    }
}
