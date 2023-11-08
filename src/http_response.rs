#[derive(Debug)]
pub struct HttpResponse {
    pub status: HttpStatus,
    pub body: Option<String>,
}

#[derive(Debug)]
pub enum HttpStatus {
    Ok,
    BadRequest,
    NotFound,
    InternalServerError,
}

#[derive(Debug)]
pub struct HttpStatusDescription {
    pub status_code: usize,
    pub status_text: String,
}

impl HttpResponse {
    pub fn ok(body: Option<String>) -> HttpResponse {
        HttpResponse {
            status: HttpStatus::Ok,
            body,
        }
    }
}

impl HttpStatus {
    pub fn get_status_info(&self) -> HttpStatusDescription {
        match self {
            HttpStatus::Ok => HttpStatusDescription {
                status_code: 200,
                status_text: String::from("OK"),
            },
            HttpStatus::BadRequest => HttpStatusDescription {
                status_code: 400,
                status_text: String::from("Bad Request"),
            },
            HttpStatus::NotFound => HttpStatusDescription {
                status_code: 404,
                status_text: String::from("Not Found"),
            },
            HttpStatus::InternalServerError => HttpStatusDescription {
                status_code: 500,
                status_text: String::from("Internal Server Error"),
            },
        }
    }
}
