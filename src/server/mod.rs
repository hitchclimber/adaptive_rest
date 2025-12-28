use actix_web::{
    App as ServerApp, HttpRequest, HttpResponse, HttpServer, Responder, get,
    http::Method,
    middleware::Logger,
    mime::APPLICATION_JSON,
    web::{self, Bytes, Data, to},
};
use std::{
    io,
    sync::{Arc, RwLock, RwLockWriteGuard},
};

pub mod endpoint;
use crate::{
    command::EndpointAction,
    server::endpoint::{EndpointEntry, EndpointStore, HandlerResult},
    util::{error::InternalError, result::InternalResult},
};

#[derive(Debug)]
pub struct ServerState {
    pub endpoints: RwLock<EndpointStore>,
}

#[get("/api/health")]
async fn health() -> impl Responder {
    "OK"
}

macro_rules! json_error {
    ($val:expr) => {
        serde_json::json!({"error": $val})
    };
}
pub async fn run_server(state: Arc<ServerState>, addr: &str) -> io::Result<()> {
    HttpServer::new(move || {
        ServerApp::new()
            .wrap(Logger::default())
            .app_data(Data::new(state.clone()))
            .service(health)
            .default_service(to(catch_all))
    })
    .bind(addr)?
    .run()
    .await
}

async fn catch_all(
    req: HttpRequest,
    state: web::Data<Arc<ServerState>>,
    body: Bytes,
) -> impl Responder {
    match state.endpoints.write() {
        Ok(mut store) => HttpResponse::from(store.handle(
            req.method(),
            req.path(),
            if body.is_empty() { None } else { Some(&body) },
        )),
        Err(_) => HttpResponse::InternalServerError().json(json_error!("internal server error")),
    }
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            endpoints: RwLock::new(EndpointStore::default()),
        }
    }

    pub fn list_endpoints(&self) -> InternalResult<Vec<EndpointEntry>> {
        let endpoints = self
            .endpoints
            .read()
            .map_err(|_| InternalError::LockFailed)?;
        Ok(endpoints.entries())
    }

    pub fn add_endpoint(&self, path: &str, body: String) -> InternalResult<()> {
        let valid_path = if path.starts_with("/") {
            path.to_owned()
        } else {
            format!("/{}", path)
        };
        let log_msg = format!("endpoint {} -> {}", &valid_path, &body);
        let was_updated = self.endpoints_mut()?.add(&valid_path, Bytes::from(body));

        log::info!(
            "{}{}",
            if was_updated { "Updated " } else { "Inserted " },
            log_msg
        );
        Ok(())
    }

    fn endpoints_mut(&self) -> InternalResult<RwLockWriteGuard<'_, EndpointStore>> {
        self.endpoints
            .write()
            .map_err(|_| InternalError::LockFailed)
    }

    pub fn delete_endpoint(&self, path: &str) -> InternalResult<()> {
        self.endpoints_mut()?
            .delete(path)
            .ok_or_else(|| InternalError::EndpointNotFound(path.to_owned()))?;
        log::info!("Removed endpoint {}", path);
        Ok(())
    }

    pub fn handle(&self, action: EndpointAction) -> InternalResult<()> {
        match action {
            EndpointAction::Add { path, response } => self.add_endpoint(&path, response),
            EndpointAction::Allow { method, path } => {
                self.endpoints_mut()?.allow(&path, Method::from(method))
            }
            EndpointAction::Delete { path } => self.delete_endpoint(&path),
            EndpointAction::Deny { method, path } => {
                self.endpoints_mut()?.deny(&path, &Method::from(method))
            }
            _ => Ok(()),
        }
    }
}

impl From<HandlerResult> for HttpResponse {
    fn from(value: HandlerResult) -> Self {
        match value {
            HandlerResult::OkEmpty => HttpResponse::Ok().finish(),
            HandlerResult::Created => HttpResponse::Created().finish(),
            HandlerResult::MethodNotAllowed => {
                HttpResponse::MethodNotAllowed().json(json_error!("method not allowed"))
            }
            HandlerResult::NotFound => HttpResponse::NotFound().json(json_error!("not found")),
            HandlerResult::Ok(body) => HttpResponse::Ok().content_type(APPLICATION_JSON).body(body),
            HandlerResult::Conflict => HttpResponse::Conflict().json(json_error!("conflict")),
            HandlerResult::BadRequest => HttpResponse::BadRequest().json("bad request"),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn test_state() -> Arc<ServerState> {
        Arc::new(ServerState::new())
    }

    #[test]
    fn test_add_endpoint() {
        let state = test_state();
        state.add_endpoint("/test", "response".into()).unwrap();

        state
            .add_endpoint("no_leading_slash", "still_valid".into())
            .unwrap();

        let endpoints = state.endpoints.read().unwrap();
        assert_eq!(
            endpoints.get("/test").map(|b| b.as_ref()),
            Some(b"response".as_ref())
        );
        assert_eq!(
            endpoints.get("/no_leading_slash").map(|b| b.as_ref()),
            Some(b"still_valid".as_ref())
        );
    }

    #[test]
    fn test_delete_endpoint() {
        let state = test_state();
        state
            .add_endpoint("/test/nested", "'{id: 123456}'".into())
            .unwrap();
        state.delete_endpoint("/test/nested").unwrap();

        let endpoints = state.endpoints.read().unwrap();
        assert!(endpoints.get("/test/nested").is_none());
    }

    #[test]
    fn test_delete_nonexistent_endpoint() {
        let state = test_state();
        let result = state.delete_endpoint("/nonexistent");

        assert!(matches!(result, Err(InternalError::EndpointNotFound(_))));
    }
}
