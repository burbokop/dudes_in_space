#![deny(warnings)]
#![allow(unused_variables)]
#![allow(dead_code)]

mod commands;
mod components;
mod logger;
mod utils;

use crate::commands::{Args, Command};
use crate::components::core_components;
use crate::utils::{load, save};
use clap::Parser as _;
use std::env::home_dir;

fn main() {
    let save_path = home_dir().unwrap().join(".dudes_in_space/save.json");
    let components = core_components();
    let mut environment = load(&components, save_path.clone());

    match Args::parse().command.unwrap_or_default() {
        Command::Step(command) => command.exec(&components, &mut environment),
        Command::Status(command) => command.exec(&environment),
    };

    save(environment, save_path)
}
