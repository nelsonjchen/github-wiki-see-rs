use actix_web::{
    get,
    middleware::Logger,
    post,
    web::{self, scope},
    App, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use askama::Template;

mod scraper;

#[derive(Template)]
#[template(path = "front_page.html")]

struct FrontPageTemplate {}

async fn front_page(_req: HttpRequest) -> impl Responder {
    let hello = FrontPageTemplate {}; // instantiate your struct
    hello
        .render()
        .unwrap()
        .with_header("Content-Type", "text/html; charset=utf-8")
}

#[get("/{account}/{repository}")] // <- define path parameters
async fn mirror_root(
    web::Path((account, repository)): web::Path<(String, String)>,
) -> Result<String> {
    Ok(format!("Account: {}, Repository {}!", account, repository))
}

#[get("/{account}/{repository}/{page}")] // <- define path parameters
async fn mirror_page(
    web::Path((account, repository, page)): web::Path<(String, String, String)>,
) -> Result<String> {
    Ok(format!(
        "Account: {}, Repository {}, Page {}!",
        account, repository, page
    ))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(front_page))
            .service(scope("mirror").service(mirror_root).service(mirror_page))
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
