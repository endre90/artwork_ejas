pub mod core;
pub use crate::core::structs::*;

pub mod utils;
pub use utils::loading::*; 
pub use utils::errors::*; 
pub use utils::matrix::*;
pub use utils::printing::*;
pub use utils::visualization::*;

pub mod algorithms;
pub use crate::algorithms::r#static::*;
pub use crate::algorithms::external::*;
pub use crate::algorithms::historic::*;
pub use crate::algorithms::ergonomic::*;
pub use crate::algorithms::complete_new::*;
// pub use crate::algorithms::incremental_new::*;
pub use crate::algorithms::incremental::*;