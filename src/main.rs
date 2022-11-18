#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_include_static_resources;

use rocket::State;

use rocket_include_static_resources::{EtagIfNoneMatch, StaticContextManager, StaticResponse};

static_response_handler! {
    "/favicon.ico" => favicon => "favicon",
    "/favicon.svg" => favicon_svg => "favicon-svg",
}

#[get("/")]
fn index() -> &'static str {
    "你好, world!"
}

#[get("/readme")]
fn readme(
    static_resources: &State<StaticContextManager>,
    etag_if_none_match: EtagIfNoneMatch,
) -> StaticResponse {
    static_resources.build(&etag_if_none_match, "readme")
}

#[launch]
fn rocket() -> _ {

    rocket::build()
        .attach(static_resources_initializer!(
            "favicon" => "static/favicon.ico",
            "favicon-svg" => "static/favicon.svg",
            "readme" => ("README.md"),
        ))
        .mount("/", routes![favicon, favicon_svg])
        .mount("/", routes![index])
        .mount("/", routes![readme])


}