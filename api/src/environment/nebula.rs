use crate::utils::math::Point;
use crate::utils::utils::Float;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Nebula {
    bound: Vec<Point<Float>>,
}

impl Nebula {
    pub fn new(bound: Vec<Point<Float>>) -> Self {
        Self { bound }   
    }
}
