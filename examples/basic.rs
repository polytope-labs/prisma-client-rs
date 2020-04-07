use prisma_client_derive::Query;
use prisma_client::Queryable;

#[derive(Query)]
struct Test {
	id: String,
	user: Human,
}

#[derive(Query)]
struct Human {
	name: String,
	age: String
}

fn main() {
	println!("{}", Test::query())
}

