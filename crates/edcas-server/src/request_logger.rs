use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Data, Request, Response};

pub struct RequestLogger;

#[rocket::async_trait]
impl Fairing for RequestLogger {
    fn info(&self) -> Info {
        Info {
            name: "Request Logger",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _: &mut Data<'_>) {
        tracing::info!("→ {} {}", request.method(), request.uri());
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        tracing::info!("← {} {} {}", request.method(), request.uri(), response.status());
    }
}
