fn main() {
	// build script generates the prisma client code
	// from the datamodel file.
	prisma_codegen::generate("./datamodel.prisma", "./")
}