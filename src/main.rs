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
async fn mirror_home<'a>(account: &'a str, repository: &'a str) -> MirrorTemplate {
    mirror_page(account, repository, "Home").await
}

#[get("/<account>/<repository>/wiki/<page>")]
async fn mirror_page<'a>(account: &'a str, repository: &'a str, page: &'a str) -> MirrorTemplate {
    let original_url = format!(
        "https://github.com/{}/{}/wiki/{}",
        account, repository, page,
    );

    let page_title = page.replace("-", " ");

    MirrorTemplate {
        original_title: page_title,
        original_url,
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
