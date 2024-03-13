#[macro_use]
extern crate rocket;
use env_logger;
mod fillup;

use rocket::fs::FileServer;

pub fn stage() -> rocket::fairing::AdHoc {
    rocket::fairing::AdHoc::on_ignite("Static File Server", |rocket| async {
        rocket.mount("/", FileServer::from("static"))
    })
}

#[launch]
fn rocket() -> _ {
    env_logger::init();
    rocket::build()
    .attach(stage())
    .attach(fillup::stage())
}
