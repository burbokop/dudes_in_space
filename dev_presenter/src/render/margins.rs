use crate::render::{Distance, Relative};
use std::convert::Into;

pub(crate) struct Margins {
    pub(crate) left: Distance,
    pub(crate) right: Distance,
    pub(crate) top: Distance,
    pub(crate) bottom: Distance,
}

impl Margins {
    pub(crate) fn vertical(margin: Distance) -> Self {
        Self {
            top: margin,
            bottom: margin,
            left: DEFAULT_MARGIN,
            right: DEFAULT_MARGIN,
        }
    }

    pub(crate) fn horisontal(margin: Distance) -> Self {
        Self {
            top: DEFAULT_MARGIN,
            bottom: DEFAULT_MARGIN,
            left: margin,
            right: margin,
        }
    }
}

pub static DEFAULT_MARGIN: Distance = Relative(0.05).into();
pub static OLD_DEFAULT_MARGIN: Distance = Relative(0.95).into();

impl Default for Margins {
    fn default() -> Self {
        Self {
            top: DEFAULT_MARGIN,
            bottom: DEFAULT_MARGIN,
            left: DEFAULT_MARGIN,
            right: DEFAULT_MARGIN,
        }
    }
}
