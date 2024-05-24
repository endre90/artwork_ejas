pub mod core;
pub use crate::core::structs::*;
pub use crate::core::enums::*;
// pub use crate::core::anonymize::*;

// pub mod records;
// pub use crate::records::year_2023::*;

pub mod algorithms;
pub use crate::algorithms::static_assignment::*;
pub use crate::algorithms::external_assignment::*;
pub use crate::algorithms::fair_horizon_assignment::*;
pub use crate::algorithms::fair_historic_assignment::*;