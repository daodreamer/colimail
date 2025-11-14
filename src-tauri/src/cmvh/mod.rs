pub mod mime;
pub mod parser;
pub mod signer;
pub mod types;
pub mod verifier;

pub use mime::build_raw_email_with_cmvh;
pub use parser::{parse_cmvh_headers, validate_cmvh_headers};
pub use signer::{derive_address, sign_email};
pub use types::{CMVHHeaders, EmailContent, VerificationResult};
pub use verifier::verify_signature;
