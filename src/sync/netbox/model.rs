use serde::Deserialize;


#[derive(Debug, Deserialize)]
pub struct Pageination<T> {
    pub count: usize,
    pub next: Option<String>,
    pub results: Vec<T>,
}