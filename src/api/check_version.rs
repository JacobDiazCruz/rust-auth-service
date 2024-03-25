use actix_web::{ Responder, HttpResponse };

pub async fn check_version() -> impl Responder {
    HttpResponse::Ok().body("API version 1.0.0!")
}
