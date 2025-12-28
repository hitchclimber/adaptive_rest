use std::collections::{BTreeMap, HashSet};

use actix_web::{http::Method, web::Bytes};

use crate::util::{error::InternalError, result::InternalResult};

#[derive(Debug, Default)]
pub struct Endpoint {
    data: Bytes,
    whitelist: HashSet<Method>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct EndpointEntry {
    pub data: String,
    pub path: String,
    pub methods: HashSet<Method>,
}

#[derive(Debug, Default)]
pub struct EndpointStore {
    entries: BTreeMap<String, Endpoint>,
}

pub enum HandlerResult {
    Ok(Bytes),
    OkEmpty,
    Created,
    NotFound,
    Conflict,
    MethodNotAllowed,
    BadRequest,
}

impl EndpointStore {
    pub fn handle(&mut self, method: &Method, path: &str, body: Option<&Bytes>) -> HandlerResult {
        match *method {
            Method::GET | Method::HEAD => {
                if let Some(data) = self.get(path) {
                    if method == Method::HEAD {
                        HandlerResult::OkEmpty
                    } else {
                        HandlerResult::Ok(data.clone())
                    }
                } else {
                    HandlerResult::NotFound
                }
            }
            Method::POST | Method::PUT => {
                if let Some(request_data) = body {
                    let is_update = self.add(path, request_data.clone());
                    match *method {
                        Method::POST => {
                            if is_update {
                                HandlerResult::BadRequest
                            } else {
                                HandlerResult::Created
                            }
                        }
                        _ => {
                            if is_update {
                                HandlerResult::Created
                            } else {
                                HandlerResult::OkEmpty
                            }
                        }
                    }
                } else {
                    HandlerResult::BadRequest
                }
            }
            Method::DELETE => {
                // we assume always success
                let _ = self.delete(path);
                HandlerResult::OkEmpty
            }
            _ => HandlerResult::MethodNotAllowed,
        }
    }

    /// Add or update an endpoint. Returns true if it was an update. *Note:* `method` needs to be
    /// owned for potential insertion (if not updating)
    pub fn add(&mut self, path: &str, body: Bytes) -> bool {
        self.entries
            .insert(
                path.to_string(),
                Endpoint {
                    data: body,
                    whitelist: HashSet::from([
                        Method::GET,
                        Method::POST,
                        Method::PUT,
                        Method::DELETE,
                        Method::HEAD,
                    ]),
                },
            )
            .is_some()
    }

    pub fn get(&self, path: &str) -> Option<&Bytes> {
        self.entries.get(path).map(|ep| &ep.data)
    }

    /// Delete an endpoint. Returns the removed body if it existed.
    /// Prunes empty nodes up to (and including) the method root.
    pub fn delete(&mut self, path: &str) -> Option<Endpoint> {
        self.entries.remove(path)
    }

    pub fn entries(&self) -> Vec<EndpointEntry> {
        self.entries
            .iter()
            .map(|(k, v)| EndpointEntry {
                path: k.to_owned(),
                data: String::from_utf8_lossy(&v.data).to_string(),
                methods: v.whitelist.clone(),
            })
            .collect()
    }

    fn get_raw_mut(&mut self, path: &str) -> InternalResult<&mut Endpoint> {
        self.entries
            .get_mut(path)
            .ok_or_else(|| InternalError::EndpointNotFound(path.to_owned()))
    }

    pub fn allow(&mut self, path: &str, method: Method) -> InternalResult<()> {
        self.get_raw_mut(path)?.whitelist.insert(method);
        Ok(())
    }

    pub fn deny(&mut self, path: &str, method: &Method) -> InternalResult<()> {
        self.get_raw_mut(path)?.whitelist.remove(method);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_add_endpoint() {
        let mut store = EndpointStore::default();
        let was_update = store.add("/users", Bytes::from("[]"));

        assert!(!was_update);
        assert!(store.get("/users").is_some());
    }

    #[test]
    fn test_add_updates_existing() {
        let mut store = EndpointStore::default();
        store.add("/users", Bytes::from("[]"));
        let was_update = store.add("/users", Bytes::from("[1,2,3]"));

        assert!(was_update);
        assert_eq!(store.get("/users").unwrap().as_ref(), b"[1,2,3]");
    }

    #[test]
    fn test_get_nonexistent() {
        let store = EndpointStore::default();
        assert!(store.get("/nothing").is_none());
    }

    #[test]
    fn test_nested_paths() {
        let mut store = EndpointStore::default();
        store.add("/users/123/posts", Bytes::from("[]"));

        assert!(store.get("/users/123/posts").is_some());
        assert!(store.get("/users/123").is_none());
        assert!(store.get("/users").is_none());
    }

    #[test]
    fn test_delete_existing() {
        let mut store = EndpointStore::default();
        store.add("/users", Bytes::from("[]"));

        let removed = store.delete("/users");
        assert!(removed.is_some());
        assert!(store.get("/users").is_none());
    }

    #[test]
    fn test_delete_nonexistent() {
        let mut store = EndpointStore::default();
        let removed = store.delete("/nothing");
        assert!(removed.is_none());
    }

    #[test]
    fn test_delete_prunes_empty_nodes() {
        let mut store = EndpointStore::default();
        store.add("/a/b/c", Bytes::from("deep"));
        store.delete("/a/b/c");

        assert!(store.is_empty());
    }

    #[test]
    fn test_delete_preserves_siblings() {
        let mut store = EndpointStore::default();
        store.add("/users/1", Bytes::from("one"));
        store.add("/users/2", Bytes::from("two"));
        store.delete("/users/1");

        assert!(store.get("/users/1").is_none());
        assert!(store.get("/users/2").is_some());
    }

    #[test]
    fn test_root_path() {
        let mut store = EndpointStore::default();
        store.add("/", Bytes::from("root"));

        assert_eq!(store.get("/").unwrap().as_ref(), b"root");
    }
}
