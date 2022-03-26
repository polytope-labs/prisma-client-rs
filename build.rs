use std::env;
use dotenv::dotenv;

fn main() {
	dotenv().expect("Please setup the .env file as detailed in the README");

	// get the prisma schema path from .env
	let prisma_schema = env::var("PRISMA_SCHEMA")
		.expect("Please set the enviroment variable to the absolute path of your prisma schema");
	prisma_codegen::generate_prisma(&prisma_schema, "./prisma.rs".into());
	println!("cargo:rerun-if-changed={}", prisma_schema);
}