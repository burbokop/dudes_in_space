use crate::utils::math::Point;
use crate::utils::utils::Float;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Nebula {
    bound: Vec<Point<Float>>,
}

impl Nebula {}
