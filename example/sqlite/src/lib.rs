#[cfg(test)]
mod tests {
    use prisma_client::{Prisma, Query, UserCreateInput};
    use serde::Deserialize;

    #[derive(Query, Deserialize, Debug)]
    struct  User {
        id: i32,
        email: String,
        name: String,
    }

    #[tokio::test]
    async fn basic_crud() {
        let client = Prisma::new(vec![]).await.unwrap();

        let user = client.create_user::<User>(
            UserCreateInput {
                name: Some("Seun Lanlege".into()),
                email: "seun@parity.io".into(),
                ..Default::default()
            }
        ).await.unwrap();

        println!("{:#?}", user);

        let users = client.users::<User>(Default::default()).await.unwrap();

        println!("{:#?}", users);
    }
}
