#[macro_use]
extern crate rocket;

extern crate image;

extern crate tokio;

// use rocket::serde::{Deserialize, Serialize};

mod lrucache;
mod reqimage;
mod serve;
mod utils;

// #[cfg(test)]
// mod tests;

// #[derive(Serialize, Deserialize)]
// #[serde(crate = "rocket::serde")]
// struct Message {
//     message: String,
// }

// #[get("/")]
// fn index() -> Value {
//     json!({ "message": "Welcome to the Rust API!" })
// }

// #[get("/hello")]
// fn hello() -> Value {
//     json!({ "message": "Hello, world!" })
// }

#[catch(404)]
fn not_found() -> utils::InvalidRequest {
    utils::send_404_response("Resource was not found.".to_string())
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        // .mount("/", routes![index, hello])
        .attach(serve::main())
        .register("/", catchers![not_found])
}
