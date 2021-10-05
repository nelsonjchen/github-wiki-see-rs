#[macro_use] extern crate rocket;

use askama::Template;

#[derive(Template)]
#[template(path = "front_page.html")]
struct FrontPageTemplate {}

#[get("/")]
fn front() -> FrontPageTemplate {
    FrontPageTemplate {}
}

#[get("/<username>/<repository>/wiki")]
fn mirror_home(username: &str, repository: &str ) -> String {
    format!("Grabbing {} from {}!", username, repository)
}

#[launch]
fn rocket() -> _ {
    // Mount front Page

    // Mount Mirror
    rocket::build().mount("/m", routes![
        mirror_home,
    ]).mount("/", routes![
        front,
    ])
}