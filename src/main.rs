use actix_files::NamedFile;
use actix_web::{
    get,
    http::{Method, StatusCode},
    middleware, web, App, Either, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use chrono::Local;
use serde::{Deserialize, Serialize};
use surrealdb::{
    dbs::Session,
    engine::{
        any::{self, Any},
        remote,
        remote::ws::{Client, Ws}, local::{File, RocksDb},
    },
    kvs::Datastore,
    Surreal,
};

use std::io;

async fn default_handler(req_method: Method) -> Result<impl Responder> {
    match req_method {
        Method::GET => {
            let file = NamedFile::open(r"src\index.html")?
                .customize()
                .with_status(StatusCode::FOUND);
            Ok(Either::Left(file))
        }
        _ => Ok(Either::Right(HttpResponse::MethodNotAllowed().finish())),
    }
}
async fn bulma_css(req_method: Method) -> Result<impl Responder> {
    match req_method {
        Method::GET => {
            let file = NamedFile::open(r"src\css\bulma.min.css")?.customize();
            Ok(Either::Left(file))
        }
        _ => Ok(Either::Right(HttpResponse::MethodNotAllowed().finish())),
    }
}

async fn all_css(req_method: Method) -> Result<impl Responder> {
    println!("asdsadsa");
    match req_method {
        Method::GET => {
            let file = NamedFile::open(r"src\css\all.min.css")?.customize();
            Ok(Either::Left(file))
        }
        _ => Ok(Either::Right(HttpResponse::MethodNotAllowed().finish())),
    }
}

#[get("/webfonts/{file}")]
async fn web_fonts(req: HttpRequest) -> Result<impl Responder> {
    match req.method().to_owned() {
        Method::GET => {
            let file_name = req.match_info().query("file");
            let file = NamedFile::open(r"src\webfonts\".to_owned() + file_name)?.customize();
            Ok(Either::Left(file))
        }
        _ => Ok(Either::Right(HttpResponse::MethodNotAllowed().finish())),
    }
}
#[get("/avaliacao/{text}/{idBotaoSelecionado}")]
async fn avaliacao(req: HttpRequest) -> Result<impl Responder> {
    /* match req.method().to_owned() {
        Method::GET => {

            Ok(Either::Left(file))
        }
        _ => Ok(Either::Right(HttpResponse::MethodNotAllowed().finish())),
    } */
    let textarea = req.match_info().query("text");
    let id_btn = req.match_info().query("idBotaoSelecionado");
    let now: chrono::DateTime<Local> = Local::now();
    return default_handler(req.method().to_owned()).await;
}
#[derive(Debug, Serialize)]
struct Avaliacao<'a> {
    text: &'a str,
    name: &'a str,
    datetime: chrono::DateTime<Local>,
}

static DB: Surreal<Any> = Surreal::init();
struct Person {
    id: i32,
    name: String,
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("criado ");

    println!("tentanu ");
    DB.connect("file://db.db").await?;
    println!("conectado");

    log::info!("Conectando no banco");
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    //DB.connect("ws://localhost:8000").await.unwrap();
    println!("asdsassssssssssssa");

    log::info!("starting HTTP server at http://192.168.1.18:64555");

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(web::resource("/bulma").route(web::get().to(bulma_css)))
            .service(web::resource("/all").route(web::get().to(all_css)))
            .service(web_fonts)
            // default
            .default_service(web::to(default_handler))
    })
    .bind(("192.168.1.18", 64555))?
    .workers(2)
    .run()
    .await?;
    Ok(())
}
