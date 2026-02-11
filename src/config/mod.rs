// Configuration module
//
// Handles parsing and validation of YAML/JSON configuration files

mod schema;
mod parser;
mod validator;
mod error;

pub use schema::*;
pub use parser::*;
pub use validator::*;
pub use error::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        // TODO: Phase 1.2 - Add configuration tests
        assert!(true);
    }
}
