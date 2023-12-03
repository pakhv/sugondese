# sugondese

This is a naive Web Api library implementation. It supports `Get`, `Post`, `Put` and `Delete` methods.

Crate `sugondese` contains api to build and run web server and crate `ligma` contains attribute macro to decorate user defined handlers.

User defined handler:
- Optionally accepts `Route`, `Query` params (both tuple structs `HashMap<String, String>`) and type `T`, that body will be deserialized into (`T` must derive `Deserialize` from `serde` crate);
- Must return `Response<T>` (`T` must derive `Serialize` from `serde` crate);
- Must be decorated with `http_handler` attribute macro.

Crate `serde_json` is used for serialization and deserialization.

Examples of valid http handlers:

```rust
#[http_handler("/")]
fn hello_handler() -> Response<String> {
    Response {
        status: HttpStatus::Ok,
        data: Some("hello from handler".to_string()),
    }
}

struct RequestStruct { /**...**/ }
struct ResponseStruct { /**...**/ }

#[http_handler("/")]
fn post_handler(body: RequestStruct) -> Response<ResponseStruct> {
    Response {
        status: HttpStatus::Ok,
        data: Some(ResponseStruct {
            field_1: 420,
            field_2: format!("{} {}", body.field_1, body.field_2),
        }),
    }
}

#[http_handler("/route_params/{param_1}/{param_2}/hello")]
fn route_params_handler(route_params: Route, query_params: Query) -> Response<String> {
    println!("{route_params:?}");
    println!("{query_params:?}");

    Response {
        status: HttpStatus::Ok,
        data: Some("hello from handler".to_string()),
    }
}
```

Build and run a web server:

```rust
fn main() {
    let _ = WebApi::new("172.17.0.2:42069", 5)
        .get(hello_handler)
        .put(route_params_handler)
        .post(post_handler)
        .run();
}
```

Snippet above starts a tcp listener on port `42069` and spawn 5 threads for handling http requests. Methods `get`, `post`, `put`, `delete` used to add user defined http handlers.
