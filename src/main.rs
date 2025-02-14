use clap::Parser;
use config::{Case, Config, Environment, File, FileFormat};
use model::config::AppConfig;
use serde::Deserialize;
use std::error::Error;
use diesel::PgConnection;
use rusqlite::config::DbConfig;
use crate::model::config::DBConfig;

#[derive(Parser, Deserialize)]
pub struct Args {
    #[arg(short, long)]
    cfg_path: String,
}

pub mod model;

fn main() -> () {
    let args = match Args::try_parse() {
        Ok(v) => v,
        Err(e) => {
            let msg = e.to_string();
            print!("{msg}");
            return;
        }
    };
    let cfg_path = args.cfg_path;
    let raw_cfg = Config::builder()
        .add_source(
            Environment::default()
                .convert_case(Case::Lower)
                .separator("_"),
        )
        .add_source(File::new(cfg_path.as_str(), FileFormat::Yaml))
        .build()
        .expect("cannot parse config");
    let app_config: AppConfig = raw_cfg
        .try_deserialize()
        .unwrap();
    let connection = establish_connection(app_config.db())
    println!("Hello, world!");
}

pub fn establish_connection(db_config: &DBConfig)
{
    PgConnection::establish(db_config.)
}
