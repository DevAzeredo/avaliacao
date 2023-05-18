use actix_files::NamedFile;
use actix_web::{
    error::InternalError,
    get,
    http::{Method, StatusCode},
    middleware, web, App, Either, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use chrono::{DateTime, Local};
use sailfish::TemplateOnce;
use serde::{Deserialize, Serialize};
use surrealdb::{engine::any::Any, opt::auth::Root, sql::Thing, Surreal};
#[derive(TemplateOnce)]
#[template(path = "index.stpl")]
struct Index;

async fn default_handler() -> Result<impl Responder> {
    let body = Index {}
        .render_once()
        .map_err(|e| InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body))
}

#[derive(TemplateOnce, Deserialize, Debug)]
#[template(path = "relatorio.stpl")]
struct Relatorio {
    datainicio: String,
    datafim: String,
}

async fn relatorio(req: HttpRequest) -> Result<impl Responder> {
    let query_string = req.query_string();
    let params: Vec<&str> = query_string.split('&').collect();

    let mut datainicio = None;
    let mut datafim = None;

    for param in params {
        let parts: Vec<&str> = param.split('=').collect();
        if parts.len() != 2 {
            continue;
        }
        match parts[0] {
            "datainicio" => datainicio = Some(parts[1]),
            "datafim" => datafim = Some(parts[1]),
            _ => (),
        }
    }

    if datainicio.is_none() || datafim.is_none() {
        let body = Relatorio {
            datainicio: "0".to_owned(),
            datafim: "0".to_owned(),
        }
        .render_once()
        .map_err(|e| InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;
        Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(body))
    } else {
        let body = Relatorio {
            datainicio: datainicio.unwrap().to_owned(),
            datafim: datafim.unwrap().to_owned(),
        }
        .render_once()
        .map_err(|e| InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;
        Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(body))
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
#[derive(Debug, Serialize, Deserialize)]
struct Avaliacao {
    textarea: String,
    nota: String,
}
#[derive(Debug, Deserialize)]
struct Record {
    #[allow(dead_code)]
    id: Thing,
}

async fn avaliacao(req: HttpRequest) -> Result<impl Responder> {
    println!("entrou aqui");

    let created: Record = DB
        .create("Avaliacao")
        .content(Avaliacao {
            textarea: req.match_info().query("text").to_string(),
            nota: req.match_info().query("nota").to_string(),
        })
        .await
        .unwrap();

    return default_handler().await;
}

static DB: Surreal<Any> = Surreal::init();
#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Iniciando...");
    println!("Conectando ao banco...");
    DB.connect("file://db.db").await?;

    DB.use_ns("namespace").use_db("database").await?;
    let a: Vec<Avaliacao> = DB.select("Avaliacao").await.unwrap();
    println!("{:?}", a);
    println!("Conectado!");
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    println!("starting HTTP server at http://192.168.1.18:64555");

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(web::resource("/bulma").route(web::get().to(bulma_css)))
            .service(web::resource("/all").route(web::get().to(all_css)))
            .service(web::resource("/relatorio").route(web::get().to(relatorio)))
            .service(web::resource("/avaliacao/{text}/{nota}").route(web::get().to(avaliacao)))
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
