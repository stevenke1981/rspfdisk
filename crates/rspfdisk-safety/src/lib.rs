//! Safety layer: WriteToken and confirmation.

pub mod confirmation;
pub mod danger;
pub mod error;
pub mod token;

pub use confirmation::{confirm_write, disk_confirmation_phrase, ConfirmationOptions};
pub use danger::{assess_disk, validate_write_risk, RiskAssessment, RiskLevel};
pub use error::{SafetyError, SafetyResult};
pub use token::WriteToken;
