use crate::http_response::HttpResponse;
use crate::uri_params::{Query, Route};

pub struct HttpHandlerInfo {
    pub handler: fn(Route, Query, Option<String>) -> HttpResponse,
    pub route: String,
}
