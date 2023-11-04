use crate::method_verb::MethodVerb;

#[derive(Debug)]
pub struct HttpRequest {
    pub method: MethodVerb,
    pub uri: String,
    pub body: Option<String>,
}
