# Actix

## State

## Guards

## Configure

## Multi-Threading
Actix is multi-threaded by default. The server starts with several workers, the amount defaults to the number of logical CPUs in the system. The number can be overridden with the `HttpServer::workers()` method.

## Request Handlers
A request handler is an async function that handles a request. These handlers define endpoints. Their inputs/parameters can be extracted if they `impl FromRequest` and they must return a type that `impl Responder`.

Example of a simple request handler
```rust
async fn index(_req: HttpRequest) -> &'static str {
    "Hello world!"
}
```

Custom structs can be used in responses if you implement `Responder` for them.
```rust
#[derive(Serialize, Deserialize)]
pub struct Achievement {
    name: String
    // other fields go here
}

impl Responder for Achievement {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}
```

### Different Return Types
Most endpoints will need multiple responses, like a 200 or 400, depending on whether or not the provided input is valid. The `Either` enum can be used as a wrapper to give this ability.

This snippet takes an ID and queries our database for an achievement with the given ID. If it is found, the achievement is returned. If it is not found, a 404 is returned.
```rust
#[get("{id}")]
pub async fn get_individual_achievement(db: &SqliteConnection, path: web::Path<u32>) -> Either<Achievement, HttpResponse> {
    let unwrapped_id = path.into_inner();
    match db.get_achievement_by_id(unwrapped) {
        Some(achievement) => Either::Left(achievement),
        None => Either::Right(HttpResponse::NotFound().body(format!("Could not find achievement with id {}", unwrapped_id)))
    }
}
```

## Extractors
Extractors are type-safe request information access from requests (i.e. `impl FromRequest`).

### Path

The below snippet defines an endpoint that will match a request with a given URL and then passes the dynamic URL values to the handler.
```rust
#[get("/users/{user_id}/{friend}")] // This request will match a request that takes the form /users/some_u32/some_string
async fn index(path: web::Path<(u32, String)>) -> Result<String> {
    let (user_id, friend) = path.into_inner();
    Ok(format!("Welcome {}, user_id {}!", friend, user_id))
}
```

The next snippet will extract path information into a struct that implements `Deserialize`
```rust
#[derive(Deserialize)]
struct Info {
    user_id: u32,
    friend: String,
}

/// extract path info using serde
#[get("/users/{user_id}/{friend}")]
async fn index(info: web::Path<Info>) -> Result<String> {
    Ok(format!(
        "Welcome {}, user_id {}!",
        info.friend, info.user_id
    ))
}
```

### Query
The Query type extracts query params from a request and can deserialize them into a type that derives `Deserialize`. If the deserialization fails a 400 is returned.

### JSON
The Json type allows the request body to be deserialized into a struct that derives `Deserialize`. The Json extractor process can be configured when setting up the App object with options like setting the maximum payload size and a custom error handler.

## Errors
Actix has default error handling where default behavior is broken down by feature. You can use custom error types in handlers and configure the default behavior for a feature within actix. Default configs can be attached within the `App::new().app_data()` call in the main method. There are six configs that can be set; FormConfig, JsonConfig, PathConfig, QueryConfg, PayloadConfig, and ServiceConfig. Custom errors can be created that `impl ResponseError`.

## Logging
[Error Logging](https://actix.rs/docs/errors/)

[General Logging](https://actix.rs/docs/middleware/)

## URL Dispatch
URL dispatching maps URLs to handler code by pattern matching. Simple endpoints can be defined with `App::new().route("/some/route", some_method())` and complicated ones can be defined as an `App::service()`. A service can contain guards, methods that verify aspects of a request in order for it to match the pattern, and one handler which will be called if the pattern matches.

Example:
```rust
App::new().service(
    web::resource("/path").route(
        web::route()
            .guard(guard::Get())
            .guard(guard::Header("content-type", "text/plain"))
            .to(HttpResponse::Ok),
    ),
)
```
The above snippet defines an endpoint that will be matched if the request is to the path `/path`, is a GET, and has a `content-type` header with the value `text/plain`.

If a request does not match any defined patterns then a "NOT FOUND" is returned. Actix can use dynamic arguments in the path as well. A path of `/user/{user_id}/foo` can be used to reach the `foo` endpoint for an individual user by their ID.

Routes can be scoped to keep them organized.

```rust
#[get("/show")]
async fn show_users() -> HttpResponse {}

#[get("/show/{id}")]
async fn user_detail(path: web::Path<(u32,)>) -> HttpResponse {}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(
            web::scope("/users")
                .service(show_users)
                .service(user_detail),
        )
    })
    // boilerplate
}
```
In the above snippet, the `show_users` handler is decorated with a path of `/show` but is added as a service under a scope. To reach this endpoint a request must be made to `/users/show`.

Information about path segments are available in `HttpRequest::match_info`.

## Middleware
Middleware allows you to add behavior to request/response processing. It can hook into incoming requests and allow you to modify/halt request processing. Middleware is registered for each `App`, `scope`, or `Resource` and executed in opposite order as registration.
