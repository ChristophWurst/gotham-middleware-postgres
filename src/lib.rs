extern crate futures;
extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate tokio_core;
extern crate tokio_postgres;

use std::io;

use futures::future::Future;
use gotham::handler::HandlerFuture;
use gotham::middleware::{Middleware, NewMiddleware};
use gotham::state::State;
use tokio_core::reactor::Handle;
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
        state.put(PostgresMiddlewareData::new(self.database.clone()));

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
    database: String,
}

impl PostgresMiddlewareData {
    pub fn new<S>(database: S) -> Self
    where
        S: Into<String>,
    {
        PostgresMiddlewareData {
            database: database.into(),
        }
    }

    pub fn connect<CB>(
        &self,
        handle: &Handle,
        cb: CB,
    ) -> Box<Future<Item = (), Error = tokio_postgres::Error>>
    where
        CB: FnOnce(Connection) -> Box<Future<Item = (), Error = tokio_postgres::Error>> + 'static,
    {
        let connect = Connection::connect(self.database.as_str(), TlsMode::None, handle);

        Box::new(connect.and_then(cb))
    }
}
