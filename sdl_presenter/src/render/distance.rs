use dudes_in_space_api::utils::utils::Float;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub(crate) struct Relative(pub Float);

impl Relative {
    pub(crate) fn value(&self) -> Float {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub(crate) struct Pix(pub Float);

impl Pix {
    pub(crate) fn value(&self) -> Float {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub(crate) enum Distance {
    Relative(Relative),
    Pix(Pix),
}

impl Distance {
    pub(crate) fn assume_relative(self) -> Option<Relative> {
        match self {
            Distance::Relative(r) => Some(r),
            Distance::Pix(_) => None,
        }
    }

    pub(crate) fn assume_pix(self) -> Option<Pix> {
        match self {
            Distance::Relative(_) => None,
            Distance::Pix(r) => Some(r),
        }
    }
}

impl const From<Relative> for Distance {
    fn from(value: Relative) -> Self {
        Self::Relative(value)
    }
}

impl const From<Pix> for Distance {
    fn from(value: Pix) -> Self {
        Self::Pix(value)
    }
}
