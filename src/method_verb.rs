use std::str::FromStr;

pub enum MethodVerb {
    Get,
    Post,
    Delete,
    Put,
}

impl FromStr for MethodVerb {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(MethodVerb::Get),
            "POST" => Ok(MethodVerb::Post),
            "DELETE" => Ok(MethodVerb::Delete),
            "PUT" => Ok(MethodVerb::Put),
            _ => Err("Invalid request verb".to_string()),
        }
    }
}
