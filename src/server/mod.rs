use actix_web::{
    App as ServerApp, HttpRequest, HttpResponse, HttpServer, Responder, get,
    http::Method,
    middleware::Logger,
    web::{self, Bytes, Data, to},
};
use std::{
    io,
    sync::{Arc, RwLock},
};

mod endpoint;
use crate::{
    server::endpoint::EndpointStore,
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

async fn catch_all(req: HttpRequest, state: web::Data<Arc<ServerState>>) -> impl Responder {
    let path = req.path();
    let endpoints = match state.endpoints.read() {
        Ok(guard) => guard,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    match endpoints.get(req.method(), path) {
        Some(response) => HttpResponse::Ok().body(response.clone()),
        None => {
            HttpResponse::NotFound().json(serde_json::json!({"error": "not found", "path": path}))
        }
    }
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            endpoints: RwLock::new(EndpointStore::default()),
        }
    }

    pub fn list_endpoints(&self, by_method: Option<&Method>) -> InternalResult<()> {
        let endpoints = self
            .endpoints
            .read()
            .map_err(|_| InternalError::LockFailed)?;

        if endpoints.is_empty() {
            log::info!("No user defined endpoints currently available");
            return Ok(());
        }
        for (method, children) in endpoints.entries(by_method) {
            let entries: Vec<_> = children
                .iter()
                .map(|(path, content)| {
                    format!("  {} -> {}", path, String::from_utf8_lossy(content))
                })
                .collect();
            log::info!(
                "\t{}\n\t{}\n{}",
                method,
                "=".repeat(method.as_str().len()),
                entries.join("\n")
            );
        }
        Ok(())
    }

    pub fn add_endpoint(&self, method: Method, path: &str, body: String) -> InternalResult<()> {
        let valid_path = if path.starts_with("/") {
            path.to_owned()
        } else {
            format!("/{}", path)
        };
        let log_msg = format!("endpoint {} {} -> {}", method, &valid_path, &body);
        let was_updated = self
            .endpoints
            .write()
            .map_err(|_| InternalError::LockFailed)?
            .add(method, &valid_path, Bytes::from(body));

        log::info!(
            "{}{}",
            if was_updated { "Updated " } else { "Inserted " },
            log_msg
        );
        Ok(())
    }

    pub fn delete_endpoint(&self, method: &Method, path: &str) -> InternalResult<()> {
        self.endpoints
            .write()
            .map_err(|_| InternalError::LockFailed)?
            .delete(method, path)
            .ok_or_else(|| InternalError::EndpointNotFound(path.to_owned()))?;
        log::info!("Removed endpoint {}", path);
        Ok(())
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
        state
            .add_endpoint(Method::GET, "/test", "response".into())
            .unwrap();

        state
            .add_endpoint(Method::GET, "no_leading_slash", "still_valid".into())
            .unwrap();

        let endpoints = state.endpoints.read().unwrap();
        assert_eq!(
            endpoints.get(&Method::GET, "/test").map(|b| b.as_ref()),
            Some(b"response".as_ref())
        );
        assert_eq!(
            endpoints
                .get(&Method::GET, "/no_leading_slash")
                .map(|b| b.as_ref()),
            Some(b"still_valid".as_ref())
        );
    }

    #[test]
    fn test_delete_endpoint() {
        let state = test_state();
        state
            .add_endpoint(Method::GET, "/test/nested", "'{id: 123456}'".into())
            .unwrap();
        state.delete_endpoint(&Method::GET, "/test/nested").unwrap();

        let endpoints = state.endpoints.read().unwrap();
        assert!(endpoints.get(&Method::GET, "/test/nested").is_none());
    }

    #[test]
    fn test_delete_nonexistent_endpoint() {
        let state = test_state();
        let result = state.delete_endpoint(&Method::GET, "/nonexistent");

        assert!(matches!(result, Err(InternalError::EndpointNotFound(_))));
    }
}
