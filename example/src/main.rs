use prisma_client::{Prisma, UserWhereUniqueInput};

/// Query derive generates the db query based on the fields in the struct.
/// TODO: validate it against prisma_client::User.
///
/// this would generate the query { id name }
#[derive(prisma_derive::Query, serde::Deserialize, Debug)]
struct User {
	id: String,
	name: String
}

#[tokio::main]
async fn main() {
	let db = Prisma::init(false).await.unwrap();
	let user = db.user::<User>(
		UserWhereUniqueInput {
			id: Some("sd".into()),
			// default here means, the rest is set to None.
			..Default::default()
		}
	).await.unwrap();

	println!("{:?}", user);
}