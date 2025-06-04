use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    sync::Mutex,
};

use actix_files::NamedFile;
use actix_multipart::form::{MultipartForm, tempfile::TempFile};
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, get, post, web};
use bytesize::ByteSize;
use serde_json::json;
use tiny_id::ShortCodeGenerator;

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(limit = "100MB")]
    file: TempFile,
}

pub struct AppState {
    pub generator: Mutex<ShortCodeGenerator<char>>,
}

#[post("/upload")]
async fn upload(
    state: web::Data<AppState>,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> impl Responder {
    // lock the mutex for other uploads rn and generate file id
    let mut generator = state.generator.lock().unwrap();
    let id = generator.next_int().to_string();

    // read form data
    let mut data = vec![];
    form.file.file.as_file().read_to_end(&mut data).unwrap();

    let filename = form.file.file_name.as_ref().unwrap();

    // get input extension, to string, .bin if error
    let extension = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or(".bin");

    // write form data
    let mut written_file =
        File::create(Path::new("./files").join(format!("{}.{}", id, extension))).unwrap();
    written_file.write_all(&data).unwrap();

    // respond with info
    println!("uploaded file {}.{}", id, extension);
    HttpResponse::Ok().json(json!({ "id": id, "ext": extension }))
}

#[get("/dl/{filename}")]
async fn dl(req: HttpRequest, filename: web::Path<String>) -> impl Responder {
    println!(
        "{} wants file {}",
        req.peer_addr().unwrap(),
        filename.to_string()
    );

    let path = Path::new("./files").join(filename.to_string());

    // handle if exists, does not exists or something else
    match std::fs::exists(path.clone()) {
        Ok(true) => NamedFile::open(path.clone()).unwrap().into_response(&req),
        Ok(false) => HttpResponse::BadRequest().json(json!({
            "error": "file was not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        })),
    }
}

#[get("/file/{filename}")]
async fn file(req: HttpRequest, filename: web::Path<String>) -> impl Responder {
    println!(
        "{} wants filepage {}",
        req.peer_addr().unwrap(),
        filename.to_string()
    );

    let path = Path::new("./files").join(filename.to_string());

    if std::fs::exists(path.clone()).unwrap() {
        let filestat = std::fs::metadata(path.clone()).unwrap();

        let template = std::fs::read_to_string("./web/file.html").unwrap();

        let filename_str = path.file_name().unwrap().to_string_lossy();
        let download_url = format!("/dl/{}", filename_str);

        let html = template
            .replace("{{filename}}", &filename_str)
            .replace("{{download_url}}", &download_url)
            .replace("{{filesize}}", &ByteSize(filestat.len()).to_string());

        HttpResponse::Ok().content_type("text/html").body(html)
    } else {
        HttpResponse::NotFound().body("File not found")
    }
}

#[get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // create dir for files
    // std::fs::create_dir_all("./files").unwrap();

    // create id generator for state to share between services
    let generator = ShortCodeGenerator::<char>::new_alphanumeric(4);
    let state = web::Data::new(AppState {
        generator: Mutex::new(generator),
    });

    // add endpoints (servicess)
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(hello)
            .service(upload)
            .service(dl)
            .service(file)
            .service(actix_files::Files::new("/", "./web").index_file("index.html"))
    })
    .bind(("127.0.0.1", 80))?
    .run()
    .await
}
