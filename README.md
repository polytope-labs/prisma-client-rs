# RUST PRISMA CLIENT

An auto-generated and type-safe query builder  in rust for [Prisma](https://prisma.io)

## Getting Started

1. Create a valid prisma schema file `prisma/schema.prisma` in your project root i.e. the newly created folder `prisma` and your `src`
folder should be siblings.

2. Update your dependencies in the `Cargo.toml` file to include `prisma-client-rs` and `serde`.
    ```toml
    [dependencies]
   ...
    prisma-client = { git = "https://github.com/polytope-labs/prisma-client-rs", branch = "master" }
    serde = { version = "1.0", features = ["serde_derive"] }
    ```
3. Create the `.cargo/config.toml` in your project root and add the following:
   ```
   [env]
   PRISMA_SCHEMA=ABSOLUTE/PATH/TO/SCHEMA/FILE/HERE
   ```
4. Go through the example in the `example` folder to see how to use the client.   