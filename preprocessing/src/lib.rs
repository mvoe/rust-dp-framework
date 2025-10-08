pub mod error;
pub mod adapters;
pub mod prelude {
    pub use crate::adapters::{
        map::Map, filter::Filter, clip::Clip, scale::Scale, zscore::ZScore, moving_avg::MovingAverage,
    };
    pub use crate::error::PrepError;
}
