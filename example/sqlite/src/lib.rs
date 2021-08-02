#[cfg(test)]
mod tests {
    use prisma_client::{Prisma, Query, UserCreateInput, FindManyUserArgs, UserWhereInput, UserWhereInputId, IntFilter};
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
                email: "seun@squarelabs.i".into(),
                ..Default::default()
            })
            .await
            .unwrap();

        println!("{:#?}", user);

        let users = client.users::<User>(
            FindManyUserArgs {
                filter: Some(UserWhereInput {
                    id: Some(UserWhereInputId::IntFilter(IntFilter {
                        within: Some(vec![1, 3, 5, 7]),
                        ..Default::default()
                    })),
                    ..Default::default()
                }),
                ..Default::default()
            }
        ).await.unwrap();

        println!("{:#?}", users);
    }
}
