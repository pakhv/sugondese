use std::str::FromStr;

#[derive(Debug)]
pub enum HttpMethod {
    Get,
    Post,
    Delete,
    Put,
}

impl FromStr for HttpMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "DELETE" => Ok(HttpMethod::Delete),
            "PUT" => Ok(HttpMethod::Put),
            _ => Err("Invalid request verb".to_string()),
        }
    }
}
