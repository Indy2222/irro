//! This module implements REST API running on Irro's onboard computer.
//! See [API documentation](https://irro.cz/api.html).

use crate::arduino::binary::Message;
use crate::arduino::cmd::led::LedMask;
use crate::arduino::cmd::motor::MotorPowerRatio;
use actix_web::{middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use log::{info, warn};
use serde::Deserialize;
use std::io;
use std::sync::mpsc::Sender;

const SERVER_ADDRESS: &str = "0.0.0.0:8080";

/// Start HTTP API server in blocking mode.
///
/// # Arguments
///
/// * `sender` - A channel for communication with Arduino via serial port.
pub fn run_http_server(sender: Sender<Message>) -> io::Result<()> {
    info!("Starting HTTP server on {}...", SERVER_ADDRESS);

    HttpServer::new(move || {
        let scope_low = web::scope("/low")
            .route("/led/{id}", web::put().to(put_led))
            .route("/led", web::get().to(get_leds))
            .route("/motor/power/ratio", web::post().to(post_motor_power_ratio));

        App::new()
            .wrap(Logger::default())
            .data(sender.clone())
            .service(scope_low)
            .default_service(web::route().to(default_handler))
    })
    .keep_alive(120)
    .bind(SERVER_ADDRESS)
    .unwrap()
    .run()
}

fn default_handler(req: HttpRequest) -> impl Responder {
    warn!("A non-existing endpoint was requested: {}", req.path());

    HttpResponse::NotFound().body(
        "API endpoint does not exist. Please visit \
         documentation at https://irro.cz.",
    )
}

fn get_leds(data: web::Data<Sender<Message>>) -> impl Responder {
    let led_states: Vec<bool> = LedMask::read(data.get_ref()).into();
    HttpResponse::Ok().json(led_states)
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

#[derive(Deserialize)]
struct MotorRatio {
    left: f32,
    right: f32,
}

fn post_motor_power_ratio(
    data: web::Data<Sender<Message>>,
    value: web::Json<MotorRatio>,
) -> impl Responder {
    let motor_ratio = value.into_inner();
    let (left, right) = (motor_ratio.left, motor_ratio.right);

    if !left.is_finite() || !right.is_finite() || left.abs() > 1.0 || right.abs() > 1.0 {
        // Don't use is_infinite() as it doesn't include NaNs
        return HttpResponse::BadRequest().body(
            "Left and right motor power ratios have to be values between -1.0 \
             and 1.0.",
        );
    }

    let command = MotorPowerRatio::from_floats(left, right);
    command.send(data.get_ref());
    HttpResponse::Ok().json(())
}
