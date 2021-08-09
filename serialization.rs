//! Graphql inline-argument serialization.
//!
//! this is entirely for serializing Structs to strings that can be inserted into a graphql query.
//!
//! imagine
//!
//! ```rust
//! use prisma::to_query_args;
//! #[derive(Serialize)]
//! struct User {
//!     id: String,
//!     name: String
//! }
//!
//! to_query_args(&User { id: "28375fb6gsd".into(), name: "Seun Lanlege".into() });
//! ```
//! This produces `{ id: "28375fb6gsd", name: "Seun Lanlege" }`
//!
//! notice the lack of surrounding quotes of Object keys.
use serde::{
	Serialize, Serializer, serde_if_integer128,
	ser::{self, SerializeSeq, Impossible}
};
use std::{fmt::Display, io, fmt};
use std::num::FpCategory;

pub fn to_query_args<T>(data: T) -> Result<String>
	where
		T: Serialize
{
	let mut serializer = QueryArgumentSerializer {
		writer: Vec::new(),
	};

	data.serialize(&mut serializer)?;

	Ok(unsafe { String::from_utf8_unchecked(serializer.writer) })
}

struct QueryArgumentSerializer {
	writer: Vec<u8>,
}

/// Serialization Errors
#[derive(derive_more::From, derive_more::Display, Debug)]
pub enum Error {
	KeyMustBeAString,
	IO(io::Error),
	Custom(String)
}

impl std::error::Error for Error {}

impl ser::Error for Error {
	fn custom<T>(msg: T) -> Self
		where
			T: Display
	{
		Error::Custom(format!("{}", msg))
	}
}

type Result<T> = std::result::Result<T, Error>;

impl<'a> Serializer for &'a mut QueryArgumentSerializer {
	type Ok = ();
	type Error = Error;

	type SerializeSeq = Compound<'a>;
	type SerializeTuple = Compound<'a>;
	type SerializeTupleStruct = Compound<'a>;
	type SerializeTupleVariant = Compound<'a>;
	type SerializeMap = Compound<'a>;
	type SerializeStruct = Compound<'a>;
	type SerializeStructVariant = Compound<'a>;

	#[inline]
	fn serialize_bool(self, value: bool) -> Result<()> {
		write_bool(&mut self.writer, value)?;
		Ok(())
	}

	#[inline]
	fn serialize_i8(self, value: i8) -> Result<()> {
		write_i8(&mut self.writer, value)?;
		Ok(())
	}

	#[inline]
	fn serialize_i16(self, value: i16) -> Result<()> {
		write_i16(&mut self.writer, value)?;
		Ok(())
	}

	#[inline]
	fn serialize_i32(self, value: i32) -> Result<()> {
		write_i32(&mut self.writer, value)?;
		Ok(())
	}

	#[inline]
	fn serialize_i64(self, value: i64) -> Result<()> {
		write_i64(&mut self.writer, value)?;
		Ok(())
	}

	serde_if_integer128! {
        fn serialize_i128(self, value: i128) -> Result<()> {
            write_number_str(&mut self.writer, &value.to_string())?;
            Ok(())
        }
    }

	#[inline]
	fn serialize_u8(self, value: u8) -> Result<()> {
		write_u8(&mut self.writer, value)?;
		Ok(())
	}

	#[inline]
	fn serialize_u16(self, value: u16) -> Result<()> {
		write_u16(&mut self.writer, value)?;
		Ok(())
	}

	#[inline]
	fn serialize_u32(self, value: u32) -> Result<()> {
		write_u32(&mut self.writer, value)?;
		Ok(())
	}

	#[inline]
	fn serialize_u64(self, value: u64) -> Result<()> {
		write_u64(&mut self.writer, value)?;
		Ok(())
	}

	serde_if_integer128! {
        fn serialize_u128(self, value: u128) -> Result<()> {
            write_number_str(&mut self.writer, &value.to_string())?;
            Ok(())
        }
    }

	#[inline]
	fn serialize_f32(self, value: f32) -> Result<()> {
		match value.classify() {
			FpCategory::Nan | FpCategory::Infinite => {
                write_null(&mut self.writer)?;
			}
			_ => {
                write_f32(&mut self.writer, value)?;
			}
		}
		Ok(())
	}

	#[inline]
	fn serialize_f64(self, value: f64) -> Result<()> {
		match value.classify() {
			FpCategory::Nan | FpCategory::Infinite => {
                write_null(&mut self.writer)?;
			}
			_ => {
                write_f64(&mut self.writer, value)?;
			}
		}
		Ok(())
	}

	#[inline]
	fn serialize_char(self, value: char) -> Result<()> {
		// A char encoded as UTF-8 takes 4 bytes at most.
		let mut buf = [0; 4];
		self.serialize_str(value.encode_utf8(&mut buf))
	}

	#[inline]
	fn serialize_str(self, value: &str) -> Result<()> {
		format_escaped_str(&mut self.writer, value)?;
		Ok(())
	}

	#[inline]
	fn serialize_bytes(self, value: &[u8]) -> Result<()> {
		let mut seq = self.serialize_seq(Some(value.len()))?;
		for byte in value {
			seq.serialize_element(byte)?;
		}
		SerializeSeq::end(seq)
	}

	#[inline]
	fn serialize_none(self) -> Result<()> {
		self.serialize_unit()
	}

	#[inline]
	fn serialize_some<T>(self, value: &T) -> Result<()>
		where
			T: ?Sized + Serialize,
	{
		value.serialize(self)
	}

	#[inline]
	fn serialize_unit(self) -> Result<()> {
		write_null(&mut self.writer)?;
		Ok(())
	}

	#[inline]
	fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
		self.serialize_unit()
	}

	#[inline]
	fn serialize_unit_variant(
		self,
		_name: &'static str,
		_variant_index: u32,
		variant: &'static str,
	) -> Result<()> {
		self.serialize_str(variant)
	}

	/// Serialize newtypes without an object wrapper.
	#[inline]
	fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
		where
			T: ?Sized + Serialize,
	{
		value.serialize(self)
	}

	#[inline]
	fn serialize_newtype_variant<T>(
		self,
		_name: &'static str,
		_variant_index: u32,
		variant: &'static str,
		value: &T,
	) -> Result<()>
		where
			T: ?Sized + Serialize,
	{
		begin_object(&mut self.writer)?;
		begin_object_key(&mut self.writer, true)?;
		self.serialize_str(variant)?;
		end_object_key(&mut self.writer)?;
		begin_object_value(&mut self.writer)?;
		value.serialize(&mut *self)?;
		end_object_value(&mut self.writer)?;
		end_object(&mut self.writer)?;
		Ok(())
	}

	#[inline]
	fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
		if len == Some(0) {
			begin_array(&mut self.writer)?;
			end_array(&mut self.writer)?;
			Ok(Compound {
				ser: self,
				state: State::Empty,
			})
		} else {
			begin_array(&mut self.writer)?;
			Ok(Compound {
				ser: self,
				state: State::First,
			})
		}
	}

	#[inline]
	fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
		self.serialize_seq(Some(len))
	}

	#[inline]
	fn serialize_tuple_struct(
		self,
		_name: &'static str,
		len: usize,
	) -> Result<Self::SerializeTupleStruct> {
		self.serialize_seq(Some(len))
	}

	#[inline]
	fn serialize_tuple_variant(
		self,
		_name: &'static str,
		_variant_index: u32,
		variant: &'static str,
		len: usize,
	) -> Result<Self::SerializeTupleVariant> {
		begin_object(&mut self.writer)?;
		begin_object_key(&mut self.writer, true)?;
		self.serialize_str(variant)?;
		end_object_key(&mut self.writer)?;
		begin_object_value(&mut self.writer)?;
		self.serialize_seq(Some(len))
	}

	#[inline]
	fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
		if len == Some(0) {
			begin_object(&mut self.writer)?;
			end_object(&mut self.writer)?;
			Ok(Compound {
				ser: self,
				state: State::Empty,
			})
		} else {
			begin_object(&mut self.writer)?;
			Ok(Compound {
				ser: self,
				state: State::First,
			})
		}
	}

	#[inline]
	fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
		match name {
			#[cfg(feature = "arbitrary_precision")]
			crate::number::TOKEN => Ok(Compound::Number { ser: self }),
			#[cfg(feature = "raw_value")]
			crate::raw::TOKEN => Ok(Compound::RawValue { ser: self }),
			_ => self.serialize_map(Some(len)),
		}
	}

	#[inline]
	fn serialize_struct_variant(
		self,
		_name: &'static str,
		_variant_index: u32,
		variant: &'static str,
		len: usize,
	) -> Result<Self::SerializeStructVariant> {
		begin_object(&mut self.writer)?;
		begin_object_key(&mut self.writer, true)?;
		self.serialize_str(variant)?;
		end_object_key(&mut self.writer)?;
		begin_object_value(&mut self.writer)?;
		self.serialize_map(Some(len))
	}

	fn collect_str<T>(self, value: &T) -> Result<()>
		where
			T: ?Sized + Display,
	{
		use self::fmt::Write;

		struct Adapter<'ser, W: 'ser> {
			writer: &'ser mut W,
			error: Option<io::Error>,
		}

		impl<'ser, W> fmt::Write for Adapter<'ser, W>
			where
				W: io::Write,
		{
			fn write_str(&mut self, s: &str) -> fmt::Result {
				assert!(self.error.is_none());
				match format_escaped_str_contents(self.writer, s) {
					Ok(()) => Ok(()),
					Err(err) => {
						self.error = Some(err);
						Err(fmt::Error)
					}
				}
			}
		}

		begin_string(&mut self.writer)?;
		{
			let mut adapter = Adapter {
				writer: &mut self.writer,
				error: None,
			};
			match write!(adapter, "{}", value) {
				Ok(()) => assert!(adapter.error.is_none()),
				Err(fmt::Error) => {
					return Err(Error::IO(adapter.error.expect("there should be an error")));
				}
			}
		}
		end_string(&mut self.writer)?;
		Ok(())
	}
}

// Not public API. Should be pub(crate).
#[doc(hidden)]
#[derive(Eq, PartialEq)]
enum State {
	Empty,
	First,
	Rest,
}

// Not public API. Should be pub(crate).
#[doc(hidden)]
struct Compound<'a> {
	ser: &'a mut QueryArgumentSerializer,
	state: State,
}

impl<'a> ser::SerializeSeq for Compound<'a> {
	type Ok = ();
	type Error = Error;

	#[inline]
	fn serialize_element<T>(&mut self, value: &T) -> Result<()>
		where
			T: ?Sized + Serialize,
	{
		begin_array_value(&mut self.ser.writer, self.state == State::First)?;
		self.state = State::Rest;
		value.serialize(&mut *self.ser)?;
		end_array_value(&mut self.ser.writer)?;
		Ok(())
	}

	#[inline]
	fn end(self) -> Result<()> {
		match self.state {
			State::Empty => {}
			_ => end_array(&mut self.ser.writer)?,
		}
		Ok(())
	}
}

impl<'a> ser::SerializeTuple for Compound<'a> {
	type Ok = ();
	type Error = Error;

	#[inline]
	fn serialize_element<T>(&mut self, value: &T) -> Result<()>
		where
			T: ?Sized + Serialize,
	{
		ser::SerializeSeq::serialize_element(self, value)
	}

	#[inline]
	fn end(self) -> Result<()> {
		ser::SerializeSeq::end(self)
	}
}

impl<'a> ser::SerializeTupleStruct for Compound<'a> {
	type Ok = ();
	type Error = Error;

	#[inline]
	fn serialize_field<T>(&mut self, value: &T) -> Result<()>
		where
			T: ?Sized + Serialize,
	{
		ser::SerializeSeq::serialize_element(self, value)
	}

	#[inline]
	fn end(self) -> Result<()> {
		ser::SerializeSeq::end(self)
	}
}

impl<'a> ser::SerializeTupleVariant for Compound<'a> {
	type Ok = ();
	type Error = Error;

	#[inline]
	fn serialize_field<T>(&mut self, value: &T) -> Result<()>
		where
			T: ?Sized + Serialize,
	{
		ser::SerializeSeq::serialize_element(self, value)
	}

	#[inline]
	fn end(self) -> Result<()> {
		match self.state {
			State::Empty => {}
			_ => end_array(&mut self.ser.writer)?,
		}
		end_object_value(&mut self.ser.writer)?;
		end_object(&mut self.ser.writer)?;
		Ok(())
	}
}

impl<'a> ser::SerializeMap for Compound<'a> {
	type Ok = ();
	type Error = Error;

	#[inline]
	fn serialize_key<T>(&mut self, key: &T) -> Result<()>
		where
			T: ?Sized + Serialize,
	{
		begin_object_key(&mut self.ser.writer, self.state == State::First)?;
		self.state = State::Rest;
		key.serialize(MapKeySerializer { ser: &mut *self.ser })?;
		end_object_key(&mut self.ser.writer)?;
		Ok(())
	}

	#[inline]
	fn serialize_value<T>(&mut self, value: &T) -> Result<()>
		where
			T: ?Sized + Serialize,
	{
		begin_object_value(&mut self.ser.writer)?;
		value.serialize(&mut *self.ser)?;
		end_object_value(&mut self.ser.writer)?;
		Ok(())
	}

	#[inline]
	fn end(self) -> Result<()> {
		match self.state {
			State::Empty => {}
			_ => end_object(&mut self.ser.writer)?,
		}
		Ok(())
	}
}

impl<'a> ser::SerializeStruct for Compound<'a> {
	type Ok = ();
	type Error = Error;

	#[inline]
	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
		where
			T: ?Sized + Serialize,
	{
		ser::SerializeMap::serialize_entry(self, key, value)
	}

	#[inline]
	fn end(self) -> Result<()> {
		ser::SerializeMap::end(self)
	}
}

impl<'a> ser::SerializeStructVariant for Compound<'a> {
	type Ok = ();
	type Error = Error;

	#[inline]
	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
		where
			T: ?Sized + Serialize,
	{
		ser::SerializeStruct::serialize_field(self, key, value)
	}

	#[inline]
	fn end(self) -> Result<()> {
		match self.state {
			State::Empty => {}
			_ => end_object(&mut self.ser.writer)?,
		}
		end_object_value(&mut self.ser.writer)?;
		end_object(&mut self.ser.writer)?;
		Ok(())
	}
}

struct MapKeySerializer<'a> {
	ser: &'a mut QueryArgumentSerializer,
}

impl<'a> ser::Serializer for MapKeySerializer<'a> {
	type Ok = ();
	type Error = Error;

	/// NOTE: this is the whole reason this lib exists
	/// to format string object keys without surrounding quotes.
	#[inline]
	fn serialize_str(self, value: &str) -> Result<()> {
		format_escaped_str_contents(&mut self.ser.writer, value)?;
		Ok(())
	}

	#[inline]
	fn serialize_unit_variant(
		self,
		_name: &'static str,
		_variant_index: u32,
		variant: &'static str,
	) -> Result<()> {
		self.ser.serialize_str(variant)
	}

	#[inline]
	fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
		where
			T: ?Sized + Serialize,
	{
		value.serialize(self)
	}

	type SerializeSeq = Impossible<(), Error>;
	type SerializeTuple = Impossible<(), Error>;
	type SerializeTupleStruct = Impossible<(), Error>;
	type SerializeTupleVariant = Impossible<(), Error>;
	type SerializeMap = Impossible<(), Error>;
	type SerializeStruct = Impossible<(), Error>;
	type SerializeStructVariant = Impossible<(), Error>;

	fn serialize_bool(self, _value: bool) -> Result<()> {
		Err(Error::KeyMustBeAString)
	}

	fn serialize_i8(self, value: i8) -> Result<()> {
		begin_string(&mut self.ser.writer)?;
		write_i8(&mut self.ser.writer, value)?;
		end_string(&mut self.ser.writer)?;
		Ok(())
	}

	fn serialize_i16(self, value: i16) -> Result<()> {
		begin_string(&mut self.ser.writer)?;
		write_i16(&mut self.ser.writer, value)?;
		end_string(&mut self.ser.writer)?;
		Ok(())
	}

	fn serialize_i32(self, value: i32) -> Result<()> {
		begin_string(&mut self.ser.writer)?;
		write_i32(&mut self.ser.writer, value)?;
		end_string(&mut self.ser.writer)?;
		Ok(())
	}

	fn serialize_i64(self, value: i64) -> Result<()> {
		begin_string(&mut self.ser.writer)?;
		write_i64(&mut self.ser.writer, value)?;
		end_string(&mut self.ser.writer)?;
		Ok(())
	}

	serde_if_integer128! {
        fn serialize_i128(self, value: i128) -> Result<()> {
            begin_string(&mut self.ser.writer)?;
            write_number_str(&mut self.ser.writer, &value.to_string())?;
            end_string(&mut self.ser.writer)?;
            Ok(())
        }
    }

	fn serialize_u8(self, value: u8) -> Result<()> {
		begin_string(&mut self.ser.writer)?;
		write_u8(&mut self.ser.writer, value)?;
		end_string(&mut self.ser.writer)?;
		Ok(())
	}

	fn serialize_u16(self, value: u16) -> Result<()> {
		begin_string(&mut self.ser.writer)?;
		write_u16(&mut self.ser.writer, value)?;
		end_string(&mut self.ser.writer)?;
		Ok(())
	}

	fn serialize_u32(self, value: u32) -> Result<()> {
		begin_string(&mut self.ser.writer)?;
		write_u32(&mut self.ser.writer, value)?;
		end_string(&mut self.ser.writer)?;
		Ok(())
	}

	fn serialize_u64(self, value: u64) -> Result<()> {
		begin_string(&mut self.ser.writer)?;
		write_u64(&mut self.ser.writer, value)?;
		end_string(&mut self.ser.writer)?;
		Ok(())
	}

	serde_if_integer128! {
        fn serialize_u128(self, value: u128) -> Result<()> {
            begin_string(&mut self.ser.writer)?;
            write_number_str(&mut self.ser.writer, &value.to_string())?;
            end_string(&mut self.ser.writer)?;
            Ok(())
        }
    }

	fn serialize_f32(self, _value: f32) -> Result<()> {
		Err(Error::KeyMustBeAString)
	}

	fn serialize_f64(self, _value: f64) -> Result<()> {
		Err(Error::KeyMustBeAString)
	}

	fn serialize_char(self, value: char) -> Result<()> {
		self.ser.serialize_str(&value.to_string())
	}

	fn serialize_bytes(self, _value: &[u8]) -> Result<()> {
		Err(Error::KeyMustBeAString)
	}

	fn serialize_unit(self) -> Result<()> {
		Err(Error::KeyMustBeAString)
	}

	fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
		Err(Error::KeyMustBeAString)
	}

	fn serialize_newtype_variant<T>(
		self,
		_name: &'static str,
		_variant_index: u32,
		_variant: &'static str,
		_value: &T,
	) -> Result<()>
		where
			T: ?Sized + Serialize,
	{
		Err(Error::KeyMustBeAString)
	}

	fn serialize_none(self) -> Result<()> {
		Err(Error::KeyMustBeAString)
	}

	fn serialize_some<T>(self, _value: &T) -> Result<()>
		where
			T: ?Sized + Serialize,
	{
		Err(Error::KeyMustBeAString)
	}

	fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
		Err(Error::KeyMustBeAString)
	}

	fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
		Err(Error::KeyMustBeAString)
	}

	fn serialize_tuple_struct(
		self,
		_name: &'static str,
		_len: usize,
	) -> Result<Self::SerializeTupleStruct> {
		Err(Error::KeyMustBeAString)
	}

	fn serialize_tuple_variant(
		self,
		_name: &'static str,
		_variant_index: u32,
		_variant: &'static str,
		_len: usize,
	) -> Result<Self::SerializeTupleVariant> {
		Err(Error::KeyMustBeAString)
	}

	fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
		Err(Error::KeyMustBeAString)
	}

	fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
		Err(Error::KeyMustBeAString)
	}

	fn serialize_struct_variant(
		self,
		_name: &'static str,
		_variant_index: u32,
		_variant: &'static str,
		_len: usize,
	) -> Result<Self::SerializeStructVariant> {
		Err(Error::KeyMustBeAString)
	}

	fn collect_str<T>(self, value: &T) -> Result<()>
		where
			T: ?Sized + Display,
	{
		self.ser.collect_str(value)
	}
}

/// Writes a `null` value to the specified writer.
#[inline]
fn write_null<W>(writer: &mut W) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	writer.write_all(b"null")
}

/// Writes a `true` or `false` value to the specified writer.
#[inline]
fn write_bool<W>(writer: &mut W, value: bool) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	let s = if value {
		b"true" as &[u8]
	} else {
		b"false" as &[u8]
	};
	writer.write_all(s)
}

/// Writes an integer value like `-123` to the specified writer.
#[inline]
fn write_i8<W>(writer: &mut W, value: i8) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	let mut buffer = itoa::Buffer::new();
	let s = buffer.format(value);
	writer.write_all(s.as_bytes())
}

/// Writes an integer value like `-123` to the specified writer.
#[inline]
fn write_i16<W>(writer: &mut W, value: i16) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	let mut buffer = itoa::Buffer::new();
	let s = buffer.format(value);
	writer.write_all(s.as_bytes())
}

/// Writes an integer value like `-123` to the specified writer.
#[inline]
fn write_i32<W>(writer: &mut W, value: i32) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	let mut buffer = itoa::Buffer::new();
	let s = buffer.format(value);
	writer.write_all(s.as_bytes())
}

/// Writes an integer value like `-123` to the specified writer.
#[inline]
fn write_i64<W>(writer: &mut W, value: i64) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	let mut buffer = itoa::Buffer::new();
	let s = buffer.format(value);
	writer.write_all(s.as_bytes())
}

/// Writes an integer value like `123` to the specified writer.
#[inline]
fn write_u8<W>(writer: &mut W, value: u8) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	let mut buffer = itoa::Buffer::new();
	let s = buffer.format(value);
	writer.write_all(s.as_bytes())
}

/// Writes an integer value like `123` to the specified writer.
#[inline]
fn write_u16<W>(writer: &mut W, value: u16) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	let mut buffer = itoa::Buffer::new();
	let s = buffer.format(value);
	writer.write_all(s.as_bytes())
}

/// Writes an integer value like `123` to the specified writer.
#[inline]
fn write_u32<W>(writer: &mut W, value: u32) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	let mut buffer = itoa::Buffer::new();
	let s = buffer.format(value);
	writer.write_all(s.as_bytes())
}

/// Writes an integer value like `123` to the specified writer.
#[inline]
fn write_u64<W>(writer: &mut W, value: u64) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	let mut buffer = itoa::Buffer::new();
	let s = buffer.format(value);
	writer.write_all(s.as_bytes())
}

/// Writes a floating point value like `-31.26e+12` to the specified writer.
#[inline]
fn write_f32<W>(writer: &mut W, value: f32) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	let mut buffer = ryu::Buffer::new();
	let s = buffer.format_finite(value);
	writer.write_all(s.as_bytes())
}

/// Writes a floating point value like `-31.26e+12` to the specified writer.
#[inline]
fn write_f64<W>(writer: &mut W, value: f64) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	let mut buffer = ryu::Buffer::new();
	let s = buffer.format_finite(value);
	writer.write_all(s.as_bytes())
}

/// Writes a number that has already been rendered to a string.
#[inline]
fn write_number_str<W>(writer: &mut W, value: &str) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	writer.write_all(value.as_bytes())
}

/// Called before each series of `write_string_fragment` and
/// `write_char_escape`.  Writes a `"` to the specified writer.
#[inline]
fn begin_string<W>(writer: &mut W) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	writer.write_all(b"\"")
}

/// Called after each series of `write_string_fragment` and
/// `write_char_escape`.  Writes a `"` to the specified writer.
#[inline]
fn end_string<W>(writer: &mut W) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	writer.write_all(b"\"")
}

/// Writes a string fragment that doesn't need any escaping to the
/// specified writer.
#[inline]
fn write_string_fragment<W>(writer: &mut W, fragment: &str) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	writer.write_all(fragment.as_bytes())
}


/// Represents a character escape code in a type-safe manner.
enum CharEscape {
	/// An escaped quote `"`
	Quote,
	/// An escaped reverse solidus `\`
	ReverseSolidus,
	/// An escaped backspace character (usually escaped as `\b`)
	Backspace,
	/// An escaped form feed character (usually escaped as `\f`)
	FormFeed,
	/// An escaped line feed character (usually escaped as `\n`)
	LineFeed,
	/// An escaped carriage return character (usually escaped as `\r`)
	CarriageReturn,
	/// An escaped tab character (usually escaped as `\t`)
	Tab,
	/// An escaped ASCII plane control character (usually escaped as
	/// `\u00XX` where `XX` are two hex characters)
	AsciiControl(u8),
}

impl CharEscape {
	#[inline]
	fn from_escape_table(escape: u8, byte: u8) -> CharEscape {
		match escape {
			self::BB => CharEscape::Backspace,
			self::TT => CharEscape::Tab,
			self::NN => CharEscape::LineFeed,
			self::FF => CharEscape::FormFeed,
			self::RR => CharEscape::CarriageReturn,
			self::QU => CharEscape::Quote,
			self::BS => CharEscape::ReverseSolidus,
			self::UU => CharEscape::AsciiControl(byte),
			_ => unreachable!(),
		}
	}
}


const BB: u8 = b'b'; // \x08
const TT: u8 = b't'; // \x09
const NN: u8 = b'n'; // \x0A
const FF: u8 = b'f'; // \x0C
const RR: u8 = b'r'; // \x0D
const QU: u8 = b'"'; // \x22
const BS: u8 = b'\\'; // \x5C
const UU: u8 = b'u'; // \x00...\x1F except the ones above
const __: u8 = 0;

// Lookup table of escape sequences. A value of b'x' at index i means that byte
// i is escaped as "\x" in JSON. A value of 0 means that byte i is not escaped.
static ESCAPE: [u8; 256] = [
	//   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
	UU, UU, UU, UU, UU, UU, UU, UU, BB, TT, NN, UU, FF, RR, UU, UU, // 0
	UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, // 1
	__, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
	__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
	__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
	__, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
	__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
	__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
	__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
	__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
	__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
	__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
	__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
	__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
	__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
	__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
];


/// Writes a character escape code to the specified writer.
#[inline]
fn write_char_escape<W>(writer: &mut W, char_escape: CharEscape) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	use self::CharEscape::*;

	let s = match char_escape {
		Quote => b"\\\"",
		ReverseSolidus => b"\\\\",
		Backspace => b"\\b",
		FormFeed => b"\\f",
		LineFeed => b"\\n",
		CarriageReturn => b"\\r",
		Tab => b"\\t",
		AsciiControl(byte) => {
			static HEX_DIGITS: [u8; 16] = *b"0123456789abcdef";
			let bytes = &[
				b'\\',
				b'u',
				b'0',
				b'0',
				HEX_DIGITS[(byte >> 4) as usize],
				HEX_DIGITS[(byte & 0xF) as usize],
			];
			return writer.write_all(bytes);
		}
	};

	writer.write_all(s)
}

/// Called before every array.  Writes a `[` to the specified
/// writer.
#[inline]
fn begin_array<W>(writer: &mut W) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	writer.write_all(b"[")
}

/// Called after every array.  Writes a `]` to the specified
/// writer.
#[inline]
fn end_array<W>(writer: &mut W) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	writer.write_all(b"]")
}

/// Called before every array value.  Writes a `,` if needed to
/// the specified writer.
#[inline]
fn begin_array_value<W>(writer: &mut W, first: bool) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	if first {
		Ok(())
	} else {
		writer.write_all(b",")
	}
}

/// Called after every array value.
#[inline]
fn end_array_value<W>(_: &mut W) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	Ok(())
}

/// Called before every object.  Writes a `{` to the specified
/// writer.
#[inline]
fn begin_object<W>(writer: &mut W) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	writer.write_all(b"{")
}

/// Called after every object.  Writes a `}` to the specified
/// writer.
#[inline]
fn end_object<W>(writer: &mut W) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	writer.write_all(b"}")
}

/// Called before every object key.
#[inline]
fn begin_object_key<W>(writer: &mut W, first: bool) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	if first {
		Ok(())
	} else {
		writer.write_all(b",")
	}
}

/// Called after every object key.  A `:` should be written to the
/// specified writer by either this method or
/// `begin_object_value`.
#[inline]
fn end_object_key<W>(_: &mut W) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	Ok(())
}

/// Called before every object value.  A `:` should be written to
/// the specified writer by either this method or
/// `end_object_key`.
#[inline]
fn begin_object_value<W>(writer: &mut W) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	writer.write_all(b":")
}

/// Called after every object value.
#[inline]
fn end_object_value<W>(_: &mut W) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	Ok(())
}


fn format_escaped_str<W>(writer: &mut W, value: &str) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	begin_string(writer)?;
	format_escaped_str_contents(writer, value)?;
	end_string(writer)?;
	Ok(())
}

fn format_escaped_str_contents<W>(
	writer: &mut W,
	value: &str,
) -> io::Result<()>
	where
		W: ?Sized + io::Write,
{
	let bytes = value.as_bytes();

	let mut start = 0;

	for (i, &byte) in bytes.iter().enumerate() {
		let escape = ESCAPE[byte as usize];
		if escape == 0 {
			continue;
		}

		if start < i {
			write_string_fragment(writer, &value[start..i])?;
		}

		let char_escape = CharEscape::from_escape_table(escape, byte);
		write_char_escape(writer, char_escape)?;

		start = i + 1;
	}

	if start != bytes.len() {
		write_string_fragment(writer, &value[start..])?;
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde::{Serialize, Deserialize};

	#[derive(Serialize, Deserialize, Clone)]
	struct Person {
		name: String,
		age: u64,
	}

	#[test]
	fn it_works() {
		let person = Person {
			name: "Seun".into(),
			age: 22,
		};

		assert_eq!(
			r###"{name:"Seun",age:22}"###,
			to_query_args(person).unwrap())
	}
}
