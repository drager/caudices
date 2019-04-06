extern crate caudices;
extern crate console_log;
extern crate env_logger;
extern crate log;
use log::Level;

fn main() {
    env_logger::init();
    console_log::init_with_level(Level::Debug);
    caudices::start()
}
