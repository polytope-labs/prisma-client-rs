/// Prisma Client.
///
/// We re-export them under one crate
pub use prisma_codegen::generate;
pub use prisma_derive::Query;
use chrono::{DateTime, Utc};

pub trait Queryable {
	fn query() -> String;
}

impl Queryable for DateTime<Utc> {
	fn query() -> String {
		String::new()
	}
}

impl<T: Queryable> Queryable for Vec<T> {
	fn query() -> String {
		T::query()
	}
}

impl<T: Queryable> Queryable for Option<T> {
	fn query() -> String {
		T::query()
	}
}

impl<T: Queryable> Queryable for Box<T> {
	fn query() -> String {
		T::query()
	}
}

impl Queryable for u128 {
	fn query() -> String {
		String::new()
	}
}

impl Queryable for u64 {
	fn query() -> String {
		String::new()
	}
}

impl Queryable for u32 {
	fn query() -> String {
		String::new()
	}
}

impl Queryable for u16 {
	fn query() -> String {
		String::new()
	}
}

impl Queryable for u8 {
	fn query() -> String {
		String::new()
	}
}

impl Queryable for i128 {
	fn query() -> String {
		String::new()
	}
}

impl Queryable for i64 {
	fn query() -> String {
		String::new()
	}
}

impl Queryable for i32 {
	fn query() -> String {
		String::new()
	}
}

impl Queryable for i16 {
	fn query() -> String {
		String::new()
	}
}

impl Queryable for i8 {
	fn query() -> String {
		String::new()
	}
}

impl Queryable for &str {
	fn query() -> String {
		String::new()
	}
}

impl Queryable for String {
	fn query() -> String {
		String::new()
	}
}

impl Queryable for bool {
	fn query() -> String {
		String::new()
	}
}
