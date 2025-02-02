pub mod core;
pub use crate::core::structs::*;

pub mod utils;
pub use utils::loading::*; 
pub use utils::errors::*; 
pub use utils::matrix::*;
pub use utils::printing::*;

pub mod algorithms;
pub use crate::algorithms::static_new_2::*;
pub use crate::algorithms::external_new::*;
pub use crate::algorithms::historic_new::*;
pub use crate::algorithms::ergonomic_new::*;
pub use crate::algorithms::complete_new::*;
pub use crate::algorithms::incremental_new::*;