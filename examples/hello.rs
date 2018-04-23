extern crate futures;
extern crate futures_state_stream;
extern crate gotham;
extern crate gotham_middleware_postgres;
extern crate hyper;
extern crate mime;
extern crate tokio_core;

use std::error::Error;

use futures::Future;
use gotham::handler::HandlerFuture;
use gotham::http::response::create_response;
use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::router::Router;
use gotham::router::builder::*;
use gotham::state::{FromState, State};
use gotham_middleware_postgres::{PostgresMiddleware, PostgresMiddlewareData};
use hyper::StatusCode;
use futures_state_stream::StateStream;

pub fn say_hello(state: State) -> Box<HandlerFuture> {
    let f = {
        let postgres = PostgresMiddlewareData::borrow_from(&state);

        postgres.connect(|connection| {
            println!("connected to postgres db");

            let f = connection
                .prepare("SELECT 1")
                .and_then(|(select, connection)| {
                    connection.query(&select, &[]).for_each(|row| {
                        println!("result: {}", row.get::<i32, usize>(0 as usize));
                    })
                })
                .and_then(|_| Ok("Hello, Postgres!".to_owned()))
                .map_err(|(err, _)| err);

            Box::new(f)
        })
    }.then(|res| {
        let text = match res {
            Ok(text) => text,
            Err(err) => format!("Error: {}", err.description()),
        };

        let res = create_response(
            &state,
            StatusCode::Ok,
            Some((text.into_bytes(), mime::TEXT_PLAIN)),
        );

        Ok((state, res))
    });

    Box::new(f)
}

fn router() -> Router {
    // docker run --name gotham-middleware-postgres -e POSTGRES_PASSWORD=mysecretpassword -p 5432:5432 -d postgres
    let (chain, pipelines) = single_pipeline(
        new_pipeline()
            .add(PostgresMiddleware::new(
                "postgresql://postgres:mysecretpassword@localhost:5432",
            ))
            .build(),
    );

    build_router(chain, pipelines, |route| {
        route.get("/").to(say_hello);
    })
}

pub fn main() {
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router())
}
