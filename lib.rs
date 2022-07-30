///
use chrono::{DateTime, Utc};

mod prisma;
mod serialization;

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
pub trait Queryable {
	fn query() -> String;
}

macro_rules! generate_queryable_impl {
	(
		$($y:ty),+
	) => {
		$(impl Queryable for $y {
			fn query() -> String {
				String::new()
			}
		})+
	}
}

generate_queryable_impl!(i64, i32, i16, i8, f64, f32, &str, bool, String, DateTime<Utc>);

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
