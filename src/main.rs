#[macro_use]
extern crate rocket;

use askama::Template;

#[derive(Template)]
#[template(path = "front_page.html")]
struct FrontPageTemplate {}

#[get("/")]
fn front() -> FrontPageTemplate {
    FrontPageTemplate {}
}

#[derive(Template)]
#[template(path = "mirror.html")]

struct MirrorTemplate {
    original_title: String,
    original_url: String,
    mirrored_content: String,
}

#[get("/<account>/<repository>/wiki")]
fn mirror_home<'a>(account: &'a str, repository: &'a str) -> MirrorTemplate {
    mirror_page(account, repository, "Home")
}

#[get("/<account>/<repository>/wiki/<page>")]
fn mirror_page<'a>(account: &'a str, repository: &'a str, page: &'a str) -> MirrorTemplate {
    let url = format!(
        "https://github.com/{}/{}/wiki/{}",
        account, repository, page,
    );
    let title = page.replace("-", " ");

    MirrorTemplate {
        original_title: title,
        original_url: url,
        mirrored_content: "blah blah".to_string(),
    }
}

#[launch]
fn rocket() -> _ {
    // Mount front Page

    // Mount Mirror
    rocket::build()
        .mount("/m", routes![mirror_home, mirror_page,])
        .mount("/", routes![front,])
}
