#[macro_use]
extern crate rocket;

use rocket::http::{ContentType, Status};
use rocket::response::Redirect;

use askama::Template;

#[derive(Template)]
#[template(path = "front_page.html")]
struct FrontPageTemplate {}

#[get("/")]
fn front() -> FrontPageTemplate {
    FrontPageTemplate {}
}

#[get("/favicon.ico")]
fn favicon() -> (Status, (ContentType, &'static [u8])) {
    (
        Status::Ok,
        (
            ContentType::Icon,
            include_bytes!("../templates/favicon.ico"),
        ),
    )
}

#[get("/<account>/<repository>/wiki")]
async fn mirror_home<'a>(account: &'a str, repository: &'a str) -> Redirect {
    mirror_page(account, repository, "Home").await
}

#[get("/<account>/<repository>/wiki/<page>")]
async fn mirror_page<'a>(account: &'a str, repository: &'a str, page: &'a str) -> Redirect {
    // Check github_assets
    let original_url_encoded = format!(
        "https://github.com/{}/{}/wiki/{}",
        account,
        repository,
        percent_encoding::utf8_percent_encode(page, percent_encoding::NON_ALPHANUMERIC),
    );

    Redirect::permanent(original_url_encoded)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/m", routes![mirror_home, mirror_page,])
        .mount("/", routes![front, favicon,])
}
