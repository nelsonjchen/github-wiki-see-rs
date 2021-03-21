use actix_web::{
    get, middleware::Logger, post, web, App, HttpResponse, HttpServer, Responder, Result,
};
mod scraper;

#[get("/mirror/{account}/{repository}")] // <- define path parameters
async fn mirror(web::Path((account, repository)): web::Path<(String, String)>) -> Result<String> {
    Ok(format!("Account: {}, Repository {}!", account, repository))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .service(mirror)
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
