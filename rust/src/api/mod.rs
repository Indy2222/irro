//! This module implements REST API running on Irro's onboard computer.
//! See [API documentation](http://irro.mgn.cz/api.html).

use crate::arduino::binary::Message;
use crate::arduino::cmd::led::LedMask;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use std::io;
use std::sync::mpsc::Sender;

/// Start HTTP API server in blocking mode.
///
/// # Arguments
///
/// * `sender` - A channel for communication with Arduino via serial port.
pub fn run_http_server(sender: Sender<Message>) -> io::Result<()> {
    HttpServer::new(move || {
        let scope_low = web::scope("/low").route("/led/{id}", web::put().to(put_led));

        App::new()
            .data(sender.clone())
            .service(scope_low)
            .default_service(web::route().to(default_handler))
    })
    .keep_alive(120)
    .bind("0.0.0.0:8080")
    .unwrap()
    .run()
}

fn default_handler() -> impl Responder {
    HttpResponse::NotFound().body(
        "API endpoint does not exist. Please visit \
         documentation at http://irro.mgn.cz.",
    )
}

fn put_led(
    data: web::Data<Sender<Message>>,
    req: HttpRequest,
    value: web::Json<bool>,
) -> impl Responder {
    let led_id = req.match_info().get("id").unwrap();
    let led_id: u32 = match led_id.parse() {
        Err(reason) => {
            return HttpResponse::BadRequest()
                .body(format!("LED ID must be a non-negative integer: {}", reason));
        }
        Ok(value) => value,
    };

    if led_id != 0 {
        return HttpResponse::NotFound().body(format!("LED \"{}\" does not exist.", led_id));
    }

    LedMask::from_bools(vec![value.into_inner()]).send(data.get_ref());
    HttpResponse::Ok().json(())
}
