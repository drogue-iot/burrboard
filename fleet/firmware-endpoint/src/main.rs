use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use oci_distribution::client;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct PollResponse {
    /// Current expected version
    pub current: Option<Metadata>,

    /// Poll interval
    pub interval: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub version: String,
    pub size: String,
}

#[derive(Serialize, Deserialize)]
pub struct FirmwareResponse {
    pub metadata: Metadata,
    pub payload: Vec<u8>,
}

#[get("/v1/poll/{image}")]
async fn poll(oci: web::Data<OciClient>, image: web::Path<String>) -> impl Responder {
    format!("Return metadata for image {}!", &image);
    HttpResponse::Ok().json(PollResponse {
        current: None,
        interval: Some(30),
    })
}

#[get("/v1/fetch/{image}/{version}")]
async fn fetch(
    oci: web::Data<OciClient>,
    image: web::Path<String>,
    version: web::Path<String>,
) -> impl Responder {
    format!("Return metadata for image {}!", &image);
    let metadata = Metadata {
        version: version.to_string(),
        size: "0".to_string(),
    };
    let payload = Vec::new();
    HttpResponse::Ok().json(FirmwareResponse { metadata, payload })
}

pub struct OciClient {
    client: Mutex<client::Client>,
}

impl OciClient {
    pub async fn fetch_latest_metadata(&self, image: &str) -> Result<Metadata, anyhow::Error> {
        todo!()
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let oci_config = client::ClientConfig {
        protocol: client::ClientProtocol::Http,
        accept_invalid_hostnames: true,
        accept_invalid_certificates: true,
        extra_root_certificates: Vec::new(),
    };

    let oci = web::Data::new(OciClient {
        client: Mutex::new(client::Client::new(oci_config)),
    });
    HttpServer::new(move || App::new().data(oci.clone()).service(poll).service(fetch))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
