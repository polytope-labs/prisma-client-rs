use std::env;

fn main() {
	let prisma_schema = "example/sqlite/prisma/schema.prisma";
	let out_dir = env::var_os("OUT_DIR").unwrap();
	prisma_codegen::generate_prisma(&prisma_schema, &out_dir.to_str().unwrap());
	println!("cargo:rerun-if-changed={}", prisma_schema);
}
