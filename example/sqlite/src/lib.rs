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

    #[derive(Clone, Deserialize, Debug, Query)]
    pub struct Post {
        pub id: i64,
        pub title: String,
        pub published: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub content: Option<String>,
        #[query(rename = "viewCount")]
        #[serde(rename = "viewCount")]
        pub view_count: i64,
        #[query(rename = "createdAt")]
        #[serde(rename = "createdAt")]
        pub created_at: chrono::DateTime<chrono::Utc>,
        #[query(rename = "updatedAt")]
        #[serde(rename = "updatedAt")]
        pub updated_at: chrono::DateTime<chrono::Utc>,
    }

    #[derive(Deserialize, Debug)]
    struct Transaction {
        users: Vec<User>,
        posts: Vec<Post>
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

        let response = client.transaction()
            .users::<User>(FindManyUserArgs {
                filter: Some(UserWhereInput {
                    id: Some(UserWhereInputId::IntFilter(IntFilter {
                        within: Some(vec![1, 3, 5, 7]),
                        ..Default::default()
                    })),
                    ..Default::default()
                }),
                ..Default::default()
            }).unwrap()
            .posts::<Post>(Default::default()).unwrap()
            .execute::<Transaction>()
            .await;

        println!("{:#?}", response);


        // println!("{:#?}", users);
    }
}
