#[macro_use]
extern crate bson;
#[macro_use]
extern crate anyhow;

use crate::config::*;
use actix_web::{web, App, HttpServer};
use crate::module::article;

mod config;
mod middleware;
mod module;
mod common;

const DEFAULT_CONFIG_FILE: &str = "config.yml";
const CONFIG_FILE_ENV: &str = "MY_CONFIG";

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    config::init_logger();

    let config = my_config();
    log::info!("[load_config] {:?}", config);

    web::block(|| Result::<(), ()>::Ok(autowired::setup_submitted_beans())).await?.expect("TODO: panic message");
    log::info!("[beans] loaded: {:?}", autowired::list_bean_names());

    let binding_address = format!("{}:{}", config.host, config.port);
    HttpServer::new(|| {
        App::new()
            .app_data(web::JsonConfig::default().error_handler(|err, req| {
                    log::error!("json extractor error, path={}, {}", req.uri(), err);
                    BusinessError::ArgumentError.into()
                })
            )
            .service(
                web::scope("/articles")
                    .route("", web::get().to(article::list_article))
                    .route("", web::post().to(article::save_article))
                    .route("{id}", web::put().to(article::update_article))
                    .route("{id}", web::delete().to(article::remove_article)),
            )
    })
        .bind(&binding_address)
        .expect(&format!("Can not bind to {}", binding_address))
        .run()
        .await?;
    Ok(())
}
