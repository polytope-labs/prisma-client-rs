use std::env;

fn main() {
	// get the prisma schema path from .env
	let prisma_schema = env::var("PRISMA_SCHEMA")
		.expect("Please set the enviroment variable to the absolute path of your prisma schema");
	prisma_codegen::generate(&prisma_schema);
	println!("cargo:rerun-if-changed={}", prisma_schema);
}