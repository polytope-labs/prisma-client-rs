# RUST PRISMA CLIENT

An auto-generated and type-safe query builder  in rust for [Prisma](https://prisma.io)

## Getting Started

1. Create a valid prisma schema file `prisma/schema.prisma` in your project root i.e. the newly created folder `prisma` and your `src`
folder should be siblings.
2. Clone the prisma-client-rs repository into your project root using `git clone git@github.com:polytope-labs/prisma-client-rs.git`.
3. Update your dependencies in the `Cargo.toml` file to include `prisma-client-rs` and `serde`, ensure the path points to the location of the cloned prisma-client-rs repository.
    ```toml
    [dependencies]
   ...
    prisma-client = { path = "./prisma-client-rs" }
    serde = { version = "1.0", features = ["serde_derive"] }
    ```
4. Create a .env in your project root and add the following:
   ```
   PRISMA_SCHEMA="../prisma/schema.prisma"
   ```
5. Go through the example in the `example` folder to see how to use the client.   