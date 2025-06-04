use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    sync::Mutex,
};

use actix_files::NamedFile;
use actix_multipart::form::{MultipartForm, tempfile::TempFile};
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, get, post, web};
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
    // lock the mutex and generate fileid
    let mut generator = state.generator.lock().unwrap();
    let id = generator.next_int().to_string();

    // read form data
    let mut data = vec![];
    form.file.file.as_file().read_to_end(&mut data).unwrap();

    let filename = form.file.file_name.as_ref().unwrap();

    // get input extension, to string if ok, .bin if error
    let extension = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or(".bin");

    // write form data
    let mut written_file =
        File::create(Path::new("./files").join(format!("{}.{}", id, extension))).unwrap();
    written_file.write_all(&data).unwrap();

    println!("uploaded file {}.{}", id, extension);

    HttpResponse::Ok().json(json!({ "id": id, "ext": extension }))
}

#[get("/file/{filename}")]
async fn file(req: HttpRequest, filename: web::Path<String>) -> impl Responder {
    println!(
        "{} wants file {}",
        req.peer_addr().unwrap(),
        filename.to_string()
    );

    let path = Path::new("./files").join(filename.to_string());

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

#[get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let generator = ShortCodeGenerator::<char>::new_alphanumeric(4);
    let state = web::Data::new(AppState {
        generator: Mutex::new(generator),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(hello)
            .service(upload)
            .service(file)
            .service(actix_files::Files::new("/", "./web").index_file("index.html"))
    })
    .bind(("127.0.0.1", 80))?
    .run()
    .await
}
