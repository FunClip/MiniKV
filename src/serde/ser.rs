use serde::{ser, Serialize};
use sloggers::syslog::format;

use crate::{KvsError, Result};

pub struct Serializer {
    /// Serialized string
    output: String,
}

pub fn to_string<T>(value: &T) ->Result<String>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        output: String::new(),
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl<'ser> ser::Serializer for &'ser mut Serializer {
    type Ok = ();

    type Error = KvsError;

    type SerializeSeq = Self;

    type SerializeTuple = Self;

    type SerializeTupleStruct = Self;

    type SerializeTupleVariant = Self;

    type SerializeMap = Self;

    type SerializeStruct = Self;

    type SerializeStructVariant = Self;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok> {
        unimplemented!("Unsupported type `bool`")
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok> {
        unimplemented!("Unsupported type `i8`")
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok> {
        unimplemented!("Unsupported type `i16`")
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok> {
        unimplemented!("Unsupported type `i32`")
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok> {
        unimplemented!("Unsupported type `i64`")
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok> {
        unimplemented!("Unsupported type `u8`")
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok> {
        unimplemented!("Unsupported type `u16`")
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok> {
        unimplemented!("Unsupported type `u32`")
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok> {
        unimplemented!("Unsupported type `u64`")
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok> {
        unimplemented!("Unsupported type `f32`")
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok> {
        unimplemented!("Unsupported type `f64`")
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok> {
        unimplemented!("Unsupported type `char`")
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.output += format!("+{}+{}", v.len(), v).as_ref();
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok> {
        unimplemented!("Unsupported type `&[u8]`")
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.output += "+0+";
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        unimplemented!("Unsupported type `()`")
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok> {
        unimplemented!("Unsupported type `{}()`", name)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.output += variant;
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize {
        self.output += name;
        self.output += ":";
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize {
        self.output += variant;
        self.output += ":";
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        unimplemented!("Unsupported type `seq`")
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        unimplemented!("Unsupported type `tuple`")
    }

    fn serialize_tuple_struct(self, name: &'static str, _len: usize) -> Result<Self::SerializeTupleStruct> {
        unimplemented!("Unsupported type `tuple_struct: {}`", name)
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        unimplemented!("Unsupported type `tuple_variant: {}`", name)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        unimplemented!("Unsupported type `map`")
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.output += variant;
        self.output += "#\r\n";
        Ok(self)
    }
}

impl<'ser> ser::SerializeSeq for &'ser mut Serializer {
    type Ok = ();

    type Error = KvsError;

    fn serialize_element<T: ?Sized>(&mut self, _value: &T) -> Result<()>
    where
        T: Serialize {
        unimplemented!("Unsupported type `seq`")
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!("Unsupported type `seq`")
    }
}

impl<'ser> ser::SerializeTuple for &'ser mut Serializer {
    type Ok = ();

    type Error = KvsError;

    fn serialize_element<T: ?Sized>(&mut self, _value: &T) -> Result<()>
    where
        T: Serialize {
        unimplemented!("Unsupported type `tuple`")
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!("Unsupported type `tuple`")
    }
}

impl<'ser> ser::SerializeTupleStruct for &'ser mut Serializer {
    type Ok = ();

    type Error = KvsError;

    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<()>
    where
        T: Serialize {
        unimplemented!("Unsupported type `tuple_struct`")
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!("Unsupported type `tuple_struct`")
    }
}

impl<'ser> ser::SerializeTupleVariant for &'ser mut Serializer {
    type Ok = ();

    type Error = KvsError;

    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<()>
    where
        T: Serialize {
        unimplemented!("Unsupported type `tuple_variant`")
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!("Unsupported type `tuple_variant`")
    }
}

impl<'ser> ser::SerializeMap for &'ser mut Serializer {
    type Ok = ();

    type Error = KvsError;

    fn serialize_key<T: ?Sized>(&mut self, _key: &T) -> Result<()>
    where
        T: Serialize {
        unimplemented!("Unsupported type `map`")
    }

    fn serialize_value<T: ?Sized>(&mut self, _value: &T) -> Result<()>
    where
        T: Serialize {
        unimplemented!("Unsupported type `map`")
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!("Unsupported type `map`")
    }
}

impl<'ser> ser::SerializeStruct for &'ser mut Serializer {
    type Ok = ();

    type Error = KvsError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: Serialize {
        self.output += key;
        self.output += ":";
        value.serialize(&mut **self)?;
        self.output += "\r\n";
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.output += "\r\n";
        Ok(())
    }
}

impl<'ser> ser::SerializeStructVariant for &'ser mut Serializer {
    type Ok = ();

    type Error = KvsError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: Serialize {
        self.output += key;
        self.output += ":";
        value.serialize(&mut **self)?;
        self.output += "\r\n";
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.output += "\r\n";
        Ok(())
    }
}

#[test]
fn test_request() {
    use crate::Request;

    let r = Request::Set{key: "hello".to_owned(), value: "world".to_owned()};
    let s = "Set#\r\nkey:+5+hello\r\nvalue:+5+world\r\n\r\n";
    println!("{}",to_string(&r).unwrap());

    assert_eq!(to_string(&r).unwrap(),s.to_owned());
}