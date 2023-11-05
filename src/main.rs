use serde::{Deserialize, Serialize};
use std::io::Result;
use sugondese::{Query, Route};

#[derive(Serialize, Deserialize)]
struct TestStruct {}

fn hello_handler<'a>(route_params: Route, query_params: Query) -> &'a str {
    "hello from handler"
}

fn main() -> Result<()> {
    let _ = sugondese::WebApi::new("172.17.0.2:6080", 5)
        .get("/", Box::new(hello_handler))
        .run()?;

    Ok(())
}
