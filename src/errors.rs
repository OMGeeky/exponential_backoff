use std::fmt;

#[derive(Debug, Clone)]
pub struct BackoffError {
    message: String,
}

impl BackoffError {
    pub fn new<S: Into<String>>(message: S) -> BackoffError {
        let message = message.into();
        BackoffError { message }
    }
}


// Generation of an error is completely separate from how it is displayed.
// There's no need to be concerned about cluttering complex logic with the display style.
//
// Note that we don't store any extra info about the errors. This means we can't state
// which string failed to parse without modifying our types to carry that information.
impl fmt::Display for BackoffError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", stringify!(BackoffError), self.message)
    }
}

impl From<&str> for BackoffError {
    fn from(s: &str) -> BackoffError {
        BackoffError {
            message: s.to_string(),
        }
    }
}

impl std::error::Error for BackoffError {
    fn description(&self) -> &str {
        self.message.as_str()
    }

}