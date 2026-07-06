//! Safety layer: WriteToken, confirmation phrases, and disk risk assessment.
//!
//! [`assess_disk()`] evaluates write risk for a disk path.
//! [`confirm_write()`] validates confirmation phrases and produces [`WriteToken`].
//! [`disk_confirmation_phrase()`] generates the required confirmation string.
//!
//! Writes are blocked by default — explicit user confirmation is required.

pub mod confirmation;
pub mod danger;
pub mod error;
pub mod token;

pub use confirmation::{confirm_write, disk_confirmation_phrase, ConfirmationOptions};
pub use danger::{assess_disk, validate_write_risk, RiskAssessment, RiskLevel};
pub use error::{SafetyError, SafetyResult};
pub use token::WriteToken;
