use crate::method_verb::HttpMethod;

#[derive(Debug)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub uri: String,
    pub body: Option<String>,
}
