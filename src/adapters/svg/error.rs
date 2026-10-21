//! What the translator refuses, and where.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SvgError {
    pub line: usize,
    pub problem: String,
}

impl SvgError {
    pub fn new(line: usize, problem: impl Into<String>) -> Self {
        Self {
            line,
            problem: problem.into(),
        }
    }
}

impl std::fmt::Display for SvgError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "line {}: {}", self.line, self.problem)
    }
}
