use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct TaskExecuteError {
    details: String,
}

#[allow(dead_code)]
impl TaskExecuteError {
    pub fn new(msg: &str) -> TaskExecuteError {
        TaskExecuteError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for TaskExecuteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for TaskExecuteError {
    fn description(&self) -> &str {
        &self.details
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    // this brings everything from parent's scope into this scope
    use super::*;

    #[test]
    fn err_pass() {
        let err = TaskExecuteError::new(&format!("testing error {}", "123456".to_string()));
        assert_eq!(err.to_string(), "testing error 123456");
        assert_eq!(err.description(), "testing error 123456");
    }
}
