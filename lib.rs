///
use chrono::{DateTime, Utc};

mod serialization;
mod prisma;

pub use datamodel::common::preview_features::PreviewFeature;

pub use prisma::*;
pub use prisma_derive::Query;


/// This allows objects(structs) describe what data they want want from the db.
///
/// ideally you aren't deriving this directly, you're using the derive proc macro `Query`.
///
/// ```rust
/// #[derive(Query)]
/// struct User {
/// 	id: String,
/// 	name: String
/// }
///
/// User::query(); // Produces `{ id name }`, which is then interpolated into a graphql query.
/// ```
///
pub trait Queryable {
	fn query() -> String;
}

// TODO: use macro
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

impl Queryable for f32 {
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
