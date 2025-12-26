use std::collections::{BTreeMap, HashMap};

use actix_web::{http::Method, web::Bytes};

#[derive(Debug, Default)]
pub struct PathNode {
    body: Option<Bytes>,
    children: BTreeMap<String, PathNode>,
}

impl PathNode {
    fn is_empty(&self) -> bool {
        self.body.is_none() && self.children.is_empty()
    }

    fn walk(&self, path: &str) -> Option<&PathNode> {
        let segments = path.trim_matches('/').split('/').filter(|s| !s.is_empty());

        let mut current = self;
        for segment in segments {
            current = current.children.get(segment)?;
        }
        Some(current)
    }

    /// Walk the path, creating nodes as needed. Always succeeds.
    fn walk_or_create(&mut self, path: &str) -> &mut PathNode {
        let segments = path.trim_matches('/').split('/').filter(|s| !s.is_empty());

        let mut current = self;
        for segment in segments {
            current = current.children.entry(segment.to_string()).or_default();
        }
        current
    }

    /// Recursively delete at path and prune empty nodes.
    /// Returns (removed_body, should_prune_self)
    fn delete_recursive(&mut self, segments: &[&str]) -> (Option<Bytes>, bool) {
        if segments.is_empty() {
            let body = self.body.take();
            return (body, self.is_empty());
        }

        let segment = segments[0];
        let rest = &segments[1..];

        if let Some(child) = self.children.get_mut(segment) {
            let (body, should_prune) = child.delete_recursive(rest);
            if should_prune {
                self.children.remove(segment);
            }
            return (body, self.is_empty());
        }

        (None, false)
    }
    fn collect_entries<'a>(&'a self, path: String, results: &mut Vec<(String, &'a Bytes)>) {
        if let Some(body) = &self.body {
            let full_path = if path.is_empty() {
                "/".to_string()
            } else {
                path.clone()
            };
            results.push((full_path, body));
        }
        for (segment, child) in &self.children {
            child.collect_entries(format!("{}/{}", path, segment), results);
        }
    }
}

#[derive(Debug, Default)]
pub struct EndpointStore {
    pub(crate) entries: HashMap<Method, PathNode>,
}

impl EndpointStore {
    /// Add or update an endpoint. Returns true if it was an update. *Note:* `method` needs to be
    /// owned for potential insertion (if not updating)
    pub fn add(&mut self, method: Method, path: &str, body: Bytes) -> bool {
        let root = self.entries.entry(method).or_default();
        let node = root.walk_or_create(path);
        let was_update = node.body.is_some();
        node.body = Some(body);
        was_update
    }

    pub fn get(&self, method: &Method, path: &str) -> Option<&Bytes> {
        self.entries.get(method)?.walk(path)?.body.as_ref()
    }

    /// Delete an endpoint. Returns the removed body if it existed.
    /// Prunes empty nodes up to (and including) the method root.
    pub fn delete(&mut self, method: &Method, path: &str) -> Option<Bytes> {
        let segments: Vec<&str> = path
            .trim_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        let root = self.entries.get_mut(method)?;
        let (body, should_prune_root) = root.delete_recursive(&segments);

        if should_prune_root {
            self.entries.remove(method);
        }

        body
    }

    fn entries_by(&self, method: &Method) -> Vec<(String, &Bytes)> {
        let mut results = Vec::new();
        if let Some(root) = self.entries.get(method) {
            root.collect_entries(String::new(), &mut results)
        }
        results
    }
    pub fn entries(&self, by_method: Option<&Method>) -> Vec<(&Method, Vec<(String, &Bytes)>)> {
        self.entries
            .keys()
            .filter(|k| by_method.is_none_or(|m| *k == m))
            .map(|m| (m, self.entries_by(m)))
            .collect()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_add_endpoint() {
        let mut store = EndpointStore::default();
        let was_update = store.add(Method::GET, "/users", Bytes::from("[]"));

        assert!(!was_update);
        assert!(store.get(&Method::GET, "/users").is_some());
    }

    #[test]
    fn test_add_updates_existing() {
        let mut store = EndpointStore::default();
        store.add(Method::GET, "/users", Bytes::from("[]"));
        let was_update = store.add(Method::GET, "/users", Bytes::from("[1,2,3]"));

        assert!(was_update);
        assert_eq!(
            store.get(&Method::GET, "/users").unwrap().as_ref(),
            b"[1,2,3]"
        );
    }

    #[test]
    fn test_different_methods_same_path() {
        let mut store = EndpointStore::default();
        store.add(Method::GET, "/users", Bytes::from("get"));
        store.add(Method::POST, "/users", Bytes::from("post"));

        assert_eq!(store.get(&Method::GET, "/users").unwrap().as_ref(), b"get");
        assert_eq!(
            store.get(&Method::POST, "/users").unwrap().as_ref(),
            b"post"
        );
    }

    #[test]
    fn test_get_nonexistent() {
        let store = EndpointStore::default();
        assert!(store.get(&Method::GET, "/nothing").is_none());
    }

    #[test]
    fn test_get_wrong_method() {
        let mut store = EndpointStore::default();
        store.add(Method::GET, "/users", Bytes::from("[]"));

        assert!(store.get(&Method::POST, "/users").is_none());
    }

    #[test]
    fn test_nested_paths() {
        let mut store = EndpointStore::default();
        store.add(Method::GET, "/users/123/posts", Bytes::from("[]"));

        assert!(store.get(&Method::GET, "/users/123/posts").is_some());
        assert!(store.get(&Method::GET, "/users/123").is_none());
        assert!(store.get(&Method::GET, "/users").is_none());
    }

    #[test]
    fn test_delete_existing() {
        let mut store = EndpointStore::default();
        store.add(Method::GET, "/users", Bytes::from("[]"));

        let removed = store.delete(&Method::GET, "/users");
        assert!(removed.is_some());
        assert!(store.get(&Method::GET, "/users").is_none());
    }

    #[test]
    fn test_delete_nonexistent() {
        let mut store = EndpointStore::default();
        let removed = store.delete(&Method::GET, "/nothing");
        assert!(removed.is_none());
    }

    #[test]
    fn test_delete_prunes_empty_nodes() {
        let mut store = EndpointStore::default();
        store.add(Method::GET, "/a/b/c", Bytes::from("deep"));
        store.delete(&Method::GET, "/a/b/c");

        // Method root should be pruned since no endpoints remain
        assert!(store.entries.is_empty());
    }

    #[test]
    fn test_delete_preserves_siblings() {
        let mut store = EndpointStore::default();
        store.add(Method::GET, "/users/1", Bytes::from("one"));
        store.add(Method::GET, "/users/2", Bytes::from("two"));
        store.delete(&Method::GET, "/users/1");

        assert!(store.get(&Method::GET, "/users/1").is_none());
        assert!(store.get(&Method::GET, "/users/2").is_some());
    }

    #[test]
    fn test_path_normalization() {
        let mut store = EndpointStore::default();
        store.add(Method::GET, "users", Bytes::from("[]"));

        assert!(store.get(&Method::GET, "/users").is_some());
        assert!(store.get(&Method::GET, "users").is_some());
        assert!(store.get(&Method::GET, "/users/").is_some());
    }

    #[test]
    fn test_root_path() {
        let mut store = EndpointStore::default();
        store.add(Method::GET, "/", Bytes::from("root"));

        assert_eq!(store.get(&Method::GET, "/").unwrap().as_ref(), b"root");
    }
}
