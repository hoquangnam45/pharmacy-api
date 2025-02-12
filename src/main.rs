use clap::Parser;
use config::{Config, FileFormat};
use model::config::AppConfig;

#[derive(Parser)]
pub struct Args {
    cfg_path: String,
}

pub mod model;

fn main() {
    let args: Args = Args::try_parse().expect("cfg_path not set");
    let cfg_path: String = args.cfg_path;
    let app_cfg: AppConfig = Config::builder()
        .add_source(config::File::new(cfg_path.as_str(), FileFormat::Yaml))
        .build()
        .try_into()
        .expect(format!("config {cfg_path} not have the correct structure").as_str());
    println!("Hello, world!");
}
