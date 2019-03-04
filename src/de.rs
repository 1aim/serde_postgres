//! Deserialize postgres rows into a Rust data structure.
use serde::de::{
    self,
    Deserialize,
    Visitor,
    IntoDeserializer,
    value::SeqDeserializer
};

use postgres::rows::{Row, Rows};

use error::{Error, Result};

/// A structure that deserialize Postgres rows into Rust values.
pub struct Deserializer<'a> {
    input: Row<'a>,
    index: usize,
}

impl<'a> Deserializer<'a> {
    /// Create a `Row` deserializer from a `Row`.
    pub fn from_row(input: Row<'a>) -> Self {
        Self { index: 0, input }
    }
}

/// Attempt to deserialize from a single `Row`.
pub fn from_row<'a, T: Deserialize<'a>>(input: Row) -> Result<T> {
    let mut deserializer = Deserializer::from_row(input);
    Ok(T::deserialize(&mut deserializer)?)
}

/// Attempt to deserialize from `Rows`.
pub fn from_rows<'a, T: Deserialize<'a>>(input: &'a Rows) -> Result<Vec<T>> {
    input.into_iter().map(|row| {
        let mut deserializer = Deserializer::from_row(row);
        T::deserialize(&mut deserializer)
    }).collect()
}

macro_rules! unsupported_type {
    ($($fn_name:ident),*,) => {
        $(
            fn $fn_name<V: Visitor<'de>>(self, _: V) -> Result<V::Value> {
                Err(Error::UnsupportedType)
            }
        )*
    }
}

macro_rules! get_value {
    ($this:ident, $v:ident, $fn_call:ident, $ty:ty) => {{
        $v.$fn_call($this.input.get_opt::<_, $ty>($this.index)
            .unwrap()
            .map_err(|_| Error::InvalidType)?)
    }}
}

impl<'de, 'a, 'b> de::Deserializer<'de> for &'b mut Deserializer<'a> {
    type Error = Error;

    unsupported_type! {
        deserialize_any,
        deserialize_u8,
        deserialize_u16,
        deserialize_u64,
        deserialize_char,
        deserialize_str,
        deserialize_bytes,
        deserialize_unit,
        deserialize_identifier,
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_unit()
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        get_value!(self, visitor, visit_bool, bool)
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        get_value!(self, visitor, visit_i8, i8)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        get_value!(self, visitor, visit_i16, i16)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        get_value!(self, visitor, visit_i32, i32)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        get_value!(self, visitor, visit_i64, i64)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        get_value!(self, visitor, visit_u32, u32)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        get_value!(self, visitor, visit_f32, f32)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        get_value!(self, visitor, visit_f64, f64)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        get_value!(self, visitor, visit_string, String)
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        get_value!(self, visitor, visit_byte_buf, Vec<u8>)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V)
        -> Result<V::Value>
    {

        if self.input.get_bytes(self.index).is_some() {
            visitor.visit_some(self)
        } else {
            visitor.visit_none()
        }
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let raw = self.input.get_opt::<_, Vec<u8>>(self.index)
            .unwrap()
            .map_err(|_| Error::InvalidType)?;

        visitor.visit_seq(SeqDeserializer::new(raw.into_iter()))
    }


    fn deserialize_enum<V: Visitor<'de>>(self,
                                         _: &str,
                                         _: &[&str],
                                         _visitor: V)
        -> Result<V::Value>
    {
        //visitor.visit_enum(self)
        Err(Error::UnsupportedType)
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(self, _: &str, _: V)
        -> Result<V::Value>
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(self, _: &str, _: V)
        -> Result<V::Value>
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, _: usize, _: V)
        -> Result<V::Value>
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(self,
                                                 _: &str,
                                                 _: usize,
                                                 _: V)
        -> Result<V::Value>
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_map(self)
    }

    fn deserialize_struct<V: Visitor<'de>>(self, _: &'static str, _: &'static [&'static str], v: V) -> Result<V::Value> {
        self.deserialize_map(v)
    }
}

impl<'de, 'a> de::MapAccess<'de> for Deserializer<'a> {
    type Error = Error;

    fn next_key_seed<T: de::DeserializeSeed<'de>>(&mut self, seed: T)
        -> Result<Option<T::Value>>
    {
        if self.index >= self.input.columns().len() {
            return Ok(None)
        }

        self.input.columns()
            .get(self.index)
            .ok_or(Error::UnknownField)
            .map(|c| c.name().to_owned().into_deserializer())
            .and_then(|n| seed.deserialize(n).map(Some))

    }

    fn next_value_seed<T: de::DeserializeSeed<'de>>(&mut self, seed: T)
        -> Result<T::Value>
    {
        let result = seed.deserialize(&mut *self);
        self.index += 1;
        result
    }
}

/*
impl<'de, 'a, 'b> de::EnumAccess<'de> for &'b mut Deserializer<'a> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V: de::DeserializeSeed<'de>>(self, seed: V)
        -> Result<(V::Value, Self::Variant)>
    {
        let value = seed.deserialize(self);
    }
}

impl<'de, 'a, 'b> de::VariantAccess<'de> for &'b mut Deserializer<'a> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T: de::DeserializeSeed<'de>>(self, seed: T)
        -> Result<T::Value>
    {
        self.input.get_opt::<_, T::Value>(self.index)
            .unwrap()
            .map_err(|_| Error::InvalidType)
    }

    fn tuple_variant<V: Visitor<'de>>(self, _: usize, _: V)
        -> Result<V::Value>
    {
        unimplemented!("tuple_variant")
    }

    fn struct_variant<V: Visitor<'de>>(self, _: &[&str], _: V)
        -> Result<V::Value>
    {
        unimplemented!("struct_variant")
    }
}
*/

#[cfg(test)]
mod tests {
    use std::env;

    use serde_derive::Deserialize;

    use postgres::Connection;

    fn setup_and_connect_to_db() -> Connection {
        let user = env::var("PGUSER").unwrap_or("postgres".into());
        let pass = env::var("PGPASSWORD").map(|p| format!("{}", p)).unwrap_or("postgres".into());
        let addr = env::var("PGADDR").unwrap_or("localhost".into());
        let port = env::var("PGPORT").unwrap_or("5432".into());
        let url = format!("postgres://{user}:{pass}@{addr}:{port}", user = user, pass = pass, addr = addr, port = port);
        Connection::connect(url, postgres::TlsMode::None).unwrap()
    }

    #[test]
    fn non_null() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Buu {
            wants_candy: bool,
            width: i16,
            amount_eaten: i32,
            amount_want_to_eat: i64,
            speed: f32,
            weight: f64,
            catchphrase: String,
            stomach_contents: Vec<u8>,
        }

        let connection = setup_and_connect_to_db();

        connection.execute("CREATE TABLE IF NOT EXISTS Buu (
                    wants_candy BOOL NOT NULL,
                    width SMALLINT NOT NULL,
                    amount_eaten INT NOT NULL,
                    amount_want_to_eat BIGINT NOT NULL,
                    speed REAL NOT NULL,
                    weight DOUBLE PRECISION NOT NULL,
                    catchphrase VARCHAR NOT NULL,
                    stomach_contents BYTEA NOT NULL
        )", &[]).unwrap();

        connection.execute("INSERT INTO Buu (
            wants_candy,
            width,
            amount_eaten,
            amount_want_to_eat,
            speed,
            weight,
            catchphrase,
            stomach_contents
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        &[&true, &20i16, &1000i32, &1000_000i64, &99.99f32, &9999.9999f64, &String::from("Woo Woo"), &vec![1u8, 2, 3, 4, 5, 6]]).unwrap();

        let results = connection.query("SELECT wants_candy,
            width,
            amount_eaten,
            amount_want_to_eat,
            speed,
            weight,
            catchphrase,
            stomach_contents
 FROM Buu", &[]).unwrap();

        let row = results.get(0);

        let buu: Buu = super::from_row(row).unwrap();

        assert_eq!(true, buu.wants_candy);
        assert_eq!(20, buu.width);
        assert_eq!(1000, buu.amount_eaten);
        assert_eq!(1000_000, buu.amount_want_to_eat);
        assert_eq!(99.99, buu.speed);
        assert_eq!(9999.9999, buu.weight);
        assert_eq!("Woo Woo", buu.catchphrase);
        assert_eq!(vec![1,2,3,4,5,6], buu.stomach_contents);

        connection.execute("DROP TABLE Buu", &[]).unwrap();
    }

    #[test]
    fn nullable() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Buu {
            wants_candy: Option<bool>,
            width: Option<i16>,
            amount_eaten: Option<i32>,
            amount_want_to_eat: Option<i64>,
            speed: Option<f32>,
            weight: Option<f64>,
            catchphrase: Option<String>,
            stomach_contents: Option<Vec<u8>>,
        }

        let connection = setup_and_connect_to_db();

        connection.execute("CREATE TABLE IF NOT EXISTS NullBuu (
                    wants_candy BOOL,
                    width SMALLINT,
                    amount_eaten INT,
                    amount_want_to_eat BIGINT,
                    speed REAL,
                    weight DOUBLE PRECISION,
                    catchphrase VARCHAR,
                    stomach_contents BYTEA
        )", &[]).unwrap();

        connection.execute("INSERT INTO NullBuu (
            wants_candy,
            width,
            amount_eaten,
            amount_want_to_eat,
            speed,
            weight,
            catchphrase,
            stomach_contents
        ) VALUES (
            NULL,
            NULL,
            NULL,
            NULL,
            NULL,
            NULL,
            NULL,
            NULL)",
        &[]).unwrap();

        let results = connection.query("SELECT wants_candy,
            width,
            amount_eaten,
            amount_want_to_eat,
            speed,
            weight,
            catchphrase,
            stomach_contents
 FROM NullBuu", &[]).unwrap();

        let row = results.get(0);

        let buu: Buu = super::from_row(row).unwrap();

        assert_eq!(None, buu.wants_candy);
        assert_eq!(None, buu.width);
        assert_eq!(None, buu.amount_eaten);
        assert_eq!(None, buu.amount_want_to_eat);
        assert_eq!(None, buu.speed);
        assert_eq!(None, buu.weight);
        assert_eq!(None, buu.catchphrase);
        assert_eq!(None, buu.stomach_contents);

        connection.execute("DROP TABLE NullBuu", &[]).unwrap();
    }

    #[test]
    fn mispelled_field_name() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Buu {
            wants_candie: bool,
        }

        let connection = setup_and_connect_to_db();

        connection.execute("CREATE TABLE IF NOT EXISTS SpellBuu (
                    wants_candy BOOL NOT NULL
        )", &[]).unwrap();

        connection.execute("INSERT INTO SpellBuu (
            wants_candy
        ) VALUES ($1)",
        &[&true]).unwrap();

        let results = connection.query("SELECT wants_candy FROM SpellBuu", &[]).unwrap();

        let row = results.get(0);

        assert_eq!(
            super::from_row::<Buu>(row),
            Err(super::Error::Message(String::from("missing field `wants_candie`"))));

        connection.execute("DROP TABLE SpellBuu", &[]).unwrap();
    }

    /*
    use postgres_derive::FromSql;
    #[test]
    fn enums() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Goku {
            hair: HairColour,
        }

        #[derive(Debug, Deserialize, FromSql, PartialEq)]
        #[postgres(name = "hair_colour")]
        enum HairColour {
            #[postgres(name = "black")]
            Black,
            #[postgres(name = "yellow")]
            Yellow,
            #[postgres(name = "blue")]
            Blue,
        }

        let connection = setup_and_connect_to_db();

        connection.execute("CREATE TYPE hair_colour as ENUM (
            'black',
            'yellow',
            'blue'
        )", &[]).unwrap();

        connection.execute("CREATE TABLE Gokus (hair hair_colour)",
        &[]).unwrap();

        connection.execute("INSERT INTO Gokus VALUES ('black')", &[])
            .unwrap();

        let results = connection.query("SELECT * FROM Gokus", &[])
            .unwrap();

        let row = results.get(0);

        let goku: Goku = super::from_row(row).unwrap();

        assert_eq!(HairColour::Black, goku.hair);

        connection.execute("DROP TABLE Gokus", &[]).unwrap();
        connection.execute("DROP TYPE hair_colour", &[]).unwrap();
    }
    */
}
