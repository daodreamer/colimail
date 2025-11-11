pub mod parser;
pub mod types;
pub mod verifier;

pub use parser::{parse_cmvh_headers, validate_cmvh_headers};
pub use types::{CMVHHeaders, EmailContent, VerificationResult};
pub use verifier::{hash_email, verify_signature};
