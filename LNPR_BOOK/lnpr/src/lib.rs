pub mod base;
pub mod plotters_ext;
pub mod robot;

pub mod prelude {
    pub use crate::base::*;
    pub use crate::plotters_ext::*;
    pub use crate::robot::*;
}
