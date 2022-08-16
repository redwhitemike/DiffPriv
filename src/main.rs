use diff_priv::test::tests::start_tests;
use std::env;

const DEFAULT_CONFIGFILENAME: &str = "application.conf";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    let args: Vec<String> = env::args().collect();
    let conf_file: String = match args.as_slice() {
        [_, a0] => a0.to_owned(),
        _ => DEFAULT_CONFIGFILENAME.to_string(),
    };

    println!("conf_file = {}", conf_file);
    start_tests(&conf_file);
    Ok(())
}
