#[macro_use]
extern crate rocket;

extern crate image;

extern crate tokio;

use rocket::serde::json::{json, Value};
use rocket::serde::{Deserialize, Serialize};

mod lrucache;
mod serve;
mod utils;

// #[cfg(test)]
// mod tests;

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Message {
    message: String,
}

#[get("/")]
fn index() -> Value {
    json!({ "message": "Welcome to the Rust API!" })
}

#[get("/hello")]
fn hello() -> Value {
    json!({ "message": "Hello, world!" })
}

#[get("/world")]
fn world() -> Value {
    json!({ "message": "Goodbye, world!" })
}

#[catch(404)]
fn not_found() -> Value {
    json!({
        "status": 404,
        "reason": "Resource was not found."
    })
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, hello, world])
        .attach(serve::stage())
        .register("/", catchers![not_found])
}
