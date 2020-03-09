# Serde Postgres
[![Build status](https://img.shields.io/travis/1aim/serde_postgres.svg?branch=master)](https://travis-ci.org/1aim/serde_postgres)
[![Crate](https://img.shields.io/crates/d/serde_postgres.svg)](https://crates.io/crates/serde_postgres)
[![Lines Of Code](https://tokei.rs/b1/github/1aim/serde_postgres?category=code)](https://github.com/Aaronepower/tokei)
[![Documentation](https://docs.rs/serde_postgres/badge.svg)](https://docs.rs/serde_postgres/)

Easily deserialize rows from [`tokio-postgres`](//docs.rs/tokio-postgres) or [`postgres`](//docs.rs/postgres) into
arbitrary structs. (Only deserialization is supported).

## Examples

**`tokio-postgres`** (asynchronous)

```rust
use std::error::Error;
use serde::Deserialize;
use tokio_postgres::{connect, NoTls};

#[derive(Clone, Debug, Deserialize)]
struct Person {
    name: String,
    age: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (client, connection) = connect("postgres://postgres@localhost:5432", NoTls).await?;
    tokio::spawn(connection);

    client.execute("CREATE TABLE IF NOT EXISTS Person (
        name VARCHAR NOT NULL,
        age INT NOT NULL
    )", &[]).await?;

    client.execute("INSERT INTO Person (name, age) VALUES ($1, $2)",
                       &[&"Jane", &23i32]).await?;

    client.execute("INSERT INTO Person (name, age) VALUES ($1, $2)",
                       &[&"Alice", &32i32]).await?;

    let rows = client.query("SELECT name, age FROM Person", &[]).await?;

    let people: Vec<Person> = serde_postgres::from_rows(&rows)?;

    for person in people {
        println!("{:?}", person);
    }

    Ok(())
}
```

**`postgres`** (synchronous)

```rust
use postgres::{Client, NoTls};
use serde::Deserialize;
use std::error::Error;

#[derive(Clone, Debug, Deserialize)]
struct Person {
    name: String,
    age: i32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut client = Client::connect("postgres://postgres@localhost:5432", NoTls)?;

    client.execute("CREATE TABLE IF NOT EXISTS Person (
        name VARCHAR NOT NULL,
        age INT NOT NULL
    )", &[])?;

    client.execute("INSERT INTO Person (name, age) VALUES ($1, $2)",
                   &[&"Jane", &23i32])?;
    
    client.execute("INSERT INTO Person (name, age) VALUES ($1, $2)",
                   &[&"Alice", &32i32])?;

    let rows = client.query("SELECT name, age FROM Person", &[])?;

    let people: Vec<Person> = serde_postgres::from_rows(&rows)?;

    for person in people {
        println!("{:?}", person);
    }

    Ok(())
}
```