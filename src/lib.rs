pub mod core;
pub use crate::core::structs::*;

pub mod utils;
pub use utils::loading::*; 
pub use utils::errors::*; 

pub mod algorithms;
pub use crate::algorithms::static_new::*;
pub use crate::algorithms::external_new::*;
pub use crate::algorithms::historic_new::*;
pub use crate::algorithms::ergonomic_new::*;