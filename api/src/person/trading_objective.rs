use std::error::Error;
use std::fmt::{Display, Formatter};
use std::marker::PointeeSized;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TradingObjective {
    i: Option<u8>,
}

#[derive(Debug)]
pub(crate) enum TradingObjectiveError {

}

impl Display for TradingObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for TradingObjectiveError {
    
}
