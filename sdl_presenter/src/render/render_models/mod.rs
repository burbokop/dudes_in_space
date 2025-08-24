mod environment_render_model;
mod module_render_model;
mod person_render_model;
mod vessel_render_model;

#[allow(unused_imports)]
pub use environment_render_model::*;
#[allow(unused_imports)]
pub use module_render_model::*;
#[allow(unused_imports)]
pub use person_render_model::*;
#[allow(unused_imports)]
pub use vessel_render_model::*;

use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum RenderError {}

impl Display for RenderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for RenderError {}
