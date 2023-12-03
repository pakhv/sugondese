use crate::http_response::HttpResponse;
use crate::uri_params::{Query, Route};

pub struct HttpHandlerInfo {
    pub handler: Box<dyn Fn(Route, Query, Option<String>) -> HttpResponse + 'static>,
    pub route: String,
}
