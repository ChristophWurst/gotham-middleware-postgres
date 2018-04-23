extern crate futures;
extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate tokio_core;
extern crate tokio_postgres;

use std::io;
use std::panic::AssertUnwindSafe;

use futures::future::Future;
use gotham::handler::HandlerFuture;
use gotham::middleware::{Middleware, NewMiddleware};
use gotham::state::{FromState, State};
use tokio_core::reactor::{Handle, Remote};
use tokio_postgres::{Connection, TlsMode};

pub struct PostgresMiddleware {
    database: String,
}

impl PostgresMiddleware {
    pub fn new<S>(database: S) -> Self
    where
        S: Into<String>,
    {
        PostgresMiddleware {
            database: database.into(),
        }
    }
}

impl Middleware for PostgresMiddleware {
    fn call<Chain>(self, mut state: State, chain: Chain) -> Box<HandlerFuture>
    where
        Chain: FnOnce(State) -> Box<HandlerFuture> + 'static,
        Self: Sized,
    {
        let handle = { Handle::borrow_from(&state).remote().clone() };
        state.put(PostgresMiddlewareData::new(handle, self.database.clone()));

        chain(state)
    }
}

impl NewMiddleware for PostgresMiddleware {
    type Instance = PostgresMiddleware;

    fn new_middleware(&self) -> io::Result<Self::Instance> {
        Ok(PostgresMiddleware {
            database: self.database.clone(),
        })
    }
}

#[derive(StateData)]
pub struct PostgresMiddlewareData {
    handle: AssertUnwindSafe<Remote>,
    database: String,
}

impl PostgresMiddlewareData {
    pub fn new<S>(handle: Remote, database: S) -> Self
    where
        S: Into<String>,
    {
        PostgresMiddlewareData {
            handle: AssertUnwindSafe(handle),
            database: database.into(),
        }
    }

    pub fn connect<CB, R>(&self, cb: CB) -> Box<Future<Item = R, Error = tokio_postgres::Error>>
    where
        CB: FnOnce(Connection) -> Box<Future<Item = R, Error = tokio_postgres::Error>> + 'static,
        R: 'static,
    {
        let connect = Connection::connect(
            self.database.as_str(),
            TlsMode::None,
            &self.handle.handle().expect("no access to handle"),
        );

        Box::new(connect.and_then(cb))
    }
}
