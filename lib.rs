///! Prisma Client.
///
/// We re-export them under one crate
pub use prisma_derive::Query;
use prisma_derive::QueryInternal;
use chrono::{DateTime, Utc};

include!(concat!(env!("OUT_DIR"), "/prisma.rs"));

/// Qraphql inline-argument serialization.
///
/// this is entirely for serializing Structs to strings that can be inserted into a graphql query.
///
/// imagine
///
/// ```rust
/// use prisma_client_rs::to_query_args;
/// #[derive(Serialize)]
/// struct User {
///     id: String,
///     name: String
/// }
///
/// to_query_args(&User { id: "28375fb6gsd".into(), name: "Seun Lanlege".into() });
/// ```
/// This produces `{ id: "28375fb6gsd", name: "Seun Lanlege" }`
///
/// notice the lack of surrounding quotes of Object keys.
pub mod serialization;
pub use serialization::to_query_args;

/// This allows objects(structs) desrbibe what data they want want from the db.
///
/// ideally end users aren't deriving this directly, they're using the derive proc macro `Query`.
///
/// ```rust
/// #[derive(Query)]
/// struct User {
/// 	id: String,
/// 	name: String
/// }
///
/// User::query();
/// ```
/// Produces `{ id name }`, this is interpolated into a graphql query.
///
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
