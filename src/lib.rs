//! # Serde Postgres
//!
//! Easily deserialize rows from [`postgres`](//docs.rs/postgres) into
//! arbitrary structs. (Only deserialization is supported).
//!
//! ```rust,no_run
//! extern crate serde;
//! extern crate serde_derive;
//! extern crate serde_postgres;
//! extern crate postgres;
//!
//! use std::error::Error;
//!
//! use serde_derive::Deserialize;
//! use postgres::{Connection, TlsMode};
//!
//! #[derive(Clone, Debug, Deserialize)]
//! struct Person {
//!     name: String,
//!     age: i32,
//! }
//!
//! fn main() -> Result<(), Box<Error>> {
//!     let connection = Connection::connect("postgres://postgres@localhost:5432", TlsMode::None)?;
//!
//!     connection.execute("CREATE TABLE IF NOT EXISTS Person (
//!     name VARCHAR NOT NULL,
//!     age INT NOT NULL
//!     )", &[])?;
//!
//!     connection.execute("INSERT INTO Person (name, age) VALUES ($1, $2)",
//!     &[&"Jane", &23])?;
//!
//!     connection.execute("INSERT INTO Person (name, age) VALUES ($1, $2)",
//!     &[&"Alice", &32])?;
//!
//!     let rows = connection.query("SELECT name, age FROM Person", &[])?;
//!
//!     let people: Vec<Person> = serde_postgres::from_rows(&rows)?;
//!
//!     for person in people {
//!         println!("{:?}", person);
//!     }
//!
//!     Ok(())
//! }
//! ```
#![deny(missing_docs)]

pub mod de;
pub mod error;
mod raw;

pub use de::{from_row, from_rows, Deserializer};
pub use error::{Error, Result};
