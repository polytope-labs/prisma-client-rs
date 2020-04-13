use datamodel::{parse_datamodel, parse_configuration, Source};
use query_core::{
	SupportedCapabilities, QuerySchemaBuilder, BuildMode, QuerySchema,
	executor::{InterpretingExecutor, QueryExecutor},
	query_document::*,
};
use url::Url;
use rust_decimal::Decimal;
use prisma_models::DatamodelConverter;
use std::{sync::Arc, collections::HashMap};
use std::{path::PathBuf, str::FromStr, collections::BTreeMap};
use serde::{de::DeserializeOwned, Serialize, Deserialize};
mod serialize;
use serialize::to_query_args;
use sql_query_connector::{Mysql, PostgreSql, Sqlite, FromSource};
use query_connector::Connector;
use graphql_parser::{
	self as gql,
	query::{
		Definition, Document, OperationDefinition, Selection as GqlSelection, SelectionSet, Value,
	}
};

// ====================== Enums ==========================
#[derive(Serialize, Debug)]
pub enum UserOrderByInput {
	#[serde(rename = "id_ASC")]
	IdASC,
	#[serde(rename = "id_DESC")]
	IdDESC,
	#[serde(rename = "name_ASC")]
	NameASC,
	#[serde(rename = "name_DESC")]
	NameDESC,
	#[serde(rename = "email_ASC")]
	EmailASC,
	#[serde(rename = "email_DESC")]
	EmailDESC,
}

// ======================================================================================

// ====================================== Input Types ===================================
#[derive(Default, Serialize, Debug)]
pub struct UserWhereInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "AND")]
    pub and: Option<Vec<UserWhereInput>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "OR")]
    pub or: Option<Vec<UserWhereInput>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "NOT")]
    pub not: Option<Vec<UserWhereInput>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "id")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "id_not")]
    pub id_not: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "id_in")]
    pub id_in: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "id_not_in")]
    pub id_not_in: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "id_lt")]
    pub id_lt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "id_lte")]
    pub id_lte: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "id_gt")]
    pub id_gt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "id_gte")]
    pub id_gte: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "id_contains")]
    pub id_contains: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "id_not_contains")]
    pub id_not_contains: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "id_starts_with")]
    pub id_starts_with: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "id_not_starts_with")]
    pub id_not_starts_with: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "id_ends_with")]
    pub id_ends_with: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "id_not_ends_with")]
    pub id_not_ends_with: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "name")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "name_not")]
    pub name_not: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "name_in")]
    pub name_in: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "name_not_in")]
    pub name_not_in: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "name_lt")]
    pub name_lt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "name_lte")]
    pub name_lte: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "name_gt")]
    pub name_gt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "name_gte")]
    pub name_gte: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "name_contains")]
    pub name_contains: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "name_not_contains")]
    pub name_not_contains: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "name_starts_with")]
    pub name_starts_with: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "name_not_starts_with")]
    pub name_not_starts_with: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "name_ends_with")]
    pub name_ends_with: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "name_not_ends_with")]
    pub name_not_ends_with: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "email")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "email_not")]
    pub email_not: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "email_in")]
    pub email_in: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "email_not_in")]
    pub email_not_in: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "email_lt")]
    pub email_lt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "email_lte")]
    pub email_lte: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "email_gt")]
    pub email_gt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "email_gte")]
    pub email_gte: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "email_contains")]
    pub email_contains: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "email_not_contains")]
    pub email_not_contains: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "email_starts_with")]
    pub email_starts_with: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "email_not_starts_with")]
    pub email_not_starts_with: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "email_ends_with")]
    pub email_ends_with: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "email_not_ends_with")]
    pub email_not_ends_with: Option<String>,
}
#[derive(Default, Serialize, Debug)]
pub struct UserWhereUniqueInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "id")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "email")]
    pub email: Option<String>,
}
#[derive(Default, Serialize, Debug)]
pub struct UserCreateInput {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "email")]
    pub email: String,
}
#[derive(Default, Serialize, Debug)]
pub struct UserUpdateInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "id")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "name")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "email")]
    pub email: Option<String>,
}
#[derive(Default, Serialize, Debug)]
pub struct UserUpdateManyMutationInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "id")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "name")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "email")]
    pub email: Option<String>,
}
// ======================================================================================

// ====================================== Outpu Types ===================================
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
}
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct AggregateUser {
    pub count: u64,
}
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct BatchPayload {
    pub count: u64,
}
// ======================================================================================


// ======================================= Argument Types ==========================================
#[derive(Default, Serialize, Debug)]
pub struct FindManyUserArgs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<UserWhereInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<UserOrderByInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<UserWhereUniqueInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<UserWhereUniqueInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last: Option<u64>,
}
#[derive(Default, Serialize, Debug)]
pub struct UpdateOneUserArgs {
    pub data: UserUpdateInput,
    pub filter: UserWhereUniqueInput,
}
#[derive(Default, Serialize, Debug)]
pub struct UpsertOneUserArgs {
    pub filter: UserWhereUniqueInput,
    pub create: UserCreateInput,
    pub update: UserUpdateInput,
}
#[derive(Default, Serialize, Debug)]
pub struct UpdateManyUserArgs {
    pub data: UserUpdateManyMutationInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<UserWhereInput>,
}
// ============================================================================================


pub trait Queryable {
	fn query() -> String;
}

impl<T: Queryable> Queryable for Vec<T> {
	fn query() -> String {
		T::query()
	}
}

impl Queryable for u128 {
	fn query() -> String {
		String::new()
	}
}

impl Queryable for u64 {
	fn query() -> String {
		String::new()
	}
}

impl Queryable for u32 {
	fn query() -> String {
		String::new()
	}
}

impl Queryable for u16 {
	fn query() -> String {
		String::new()
	}
}

impl Queryable for u8 {
	fn query() -> String {
		String::new()
	}
}

impl Queryable for i128 {
	fn query() -> String {
		String::new()
	}
}

impl Queryable for i64 {
	fn query() -> String {
		String::new()
	}
}

impl Queryable for i32 {
	fn query() -> String {
		String::new()
	}
}

impl Queryable for i16 {
	fn query() -> String {
		String::new()
	}
}

impl Queryable for i8 {
	fn query() -> String {
		String::new()
	}
}

impl Queryable for &str {
	fn query() -> String {
		String::new()
	}
}

impl Queryable for String {
	fn query() -> String {
		String::new()
	}
}


pub struct Prisma {
	executor: Box<dyn QueryExecutor + Send + Sync + 'static>,
	query_schema: Arc<QuerySchema>,
}

impl Prisma {
	pub async fn init(force_transactions: bool) -> Result<Self, Error> {
		let datamodel_str = r###"datasource pg {
	provider = "postgres"
	url = "postgres://root:prisma@localhost:5432/default@default"
}

model User {
    id String @id
    name String
    email String @unique
}"###;
		let config = parse_configuration(datamodel_str).unwrap();
		let source = config.datasources.first()
			.expect("Please supply a datasource in your datamodel.prisma file");

		let model = parse_datamodel(datamodel_str).unwrap();
		let (db, executor) = load_executor(&**source, force_transactions).await?;

		let internal_model = DatamodelConverter::convert(&model)
			.build(db);
		let cap = SupportedCapabilities::empty();
		let schema_builder = QuerySchemaBuilder::new(
			&internal_model,
			&cap,
			BuildMode::Modern,
			false,
		);
		let query_schema = Arc::new(schema_builder.build());

		Ok(Self { executor, query_schema })
	}
    pub async fn users<T>(&self, data: FindManyUserArgs) -> Result<Vec<T>, Error>
        where
            T: Queryable + DeserializeOwned,
    {
        let query = String::from("query { findManyUser");
    	let mut operation_args = String::new();
        if data.filter.is_some() {
        	operation_args.push_str(&format!(
        		"where: {},",
        		to_query_args(data.filter.as_ref().unwrap()).unwrap()
        	));
        }
        if data.order_by.is_some() {
        	operation_args.push_str(&format!(
        		"orderBy: {},",
        		to_query_args(data.order_by.as_ref().unwrap()).unwrap()
        	));
        }
        if data.skip.is_some() {
        	operation_args.push_str(&format!(
        		"skip: {},",
        		to_query_args(data.skip.as_ref().unwrap()).unwrap()
        	));
        }
        if data.after.is_some() {
        	operation_args.push_str(&format!(
        		"after: {},",
        		to_query_args(data.after.as_ref().unwrap()).unwrap()
        	));
        }
        if data.before.is_some() {
        	operation_args.push_str(&format!(
        		"before: {},",
        		to_query_args(data.before.as_ref().unwrap()).unwrap()
        	));
        }
        if data.first.is_some() {
        	operation_args.push_str(&format!(
        		"first: {},",
        		to_query_args(data.first.as_ref().unwrap()).unwrap()
        	));
        }
        if data.last.is_some() {
        	operation_args.push_str(&format!(
        		"last: {},",
        		to_query_args(data.last.as_ref().unwrap()).unwrap()
        	));
        }
    	if !operation_args.is_empty() {
    		operation_args = format!("({})", operation_args);
    	}
    	let mut query = format!(r#"
    		{} {} {}"#,
    		query,
    		operation_args,
    		T::query(),
    	);
    	query.push_str("}");
    	let doc = gql::parse_query(&query)?;
    	let operation = convert(doc)?;
    	let schema = self.query_schema.clone();
    	let response = self.executor.execute(operation, schema).await?;
    	if response.errors.len() > 0 {
    	    // return the errors.
    	    todo!("proper error")
    	}

    	// serialize from map to Queryable.
    	let value = serde_json::to_value(&response.data.get("findManyUser").unwrap())
    		.expect("Deserializing to serde_json::Value should be infallible");
    	let data = serde_json::from_value(value)
    		.expect("Validation is done by prisma_client_derive::Queryable, this is infallible");
    	Ok(data)
    }
    pub async fn aggregate_users(&self) -> Result<BatchPayload, Error>
        
    {
        let query = String::from("query { aggregateUser");
    	let mut operation_args = String::new();
    	if !operation_args.is_empty() {
    		operation_args = format!("({})", operation_args);
    	}
    	let mut query = format!(r#"
    		{} {} {}"#,
    		query,
    		operation_args,
    		"{ count }",
    	);
    	query.push_str("}");
    	let doc = gql::parse_query(&query)?;
    	let operation = convert(doc)?;
    	let schema = self.query_schema.clone();
    	let response = self.executor.execute(operation, schema).await?;
    	if response.errors.len() > 0 {
    	    // return the errors.
    	    todo!("proper error")
    	}

    	// serialize from map to Queryable.
    	let value = serde_json::to_value(&response.data.get("aggregateUser").unwrap())
    		.expect("Deserializing to serde_json::Value should be infallible");
    	let data = serde_json::from_value(value)
    		.expect("Validation is done by prisma_client_derive::Queryable, this is infallible");
    	Ok(data)
    }
    pub async fn user<T>(&self, filter: UserWhereUniqueInput) -> Result<T, Error>
        where
            T: Queryable + DeserializeOwned,
    {
        let query = String::from("query { findOneUser");
    	let mut operation_args = String::new();
        operation_args.push_str(&format!(
        	"where: {},",
        	to_query_args(filter).unwrap()
        ));
    	if !operation_args.is_empty() {
    		operation_args = format!("({})", operation_args);
    	}
    	let mut query = format!(r#"
    		{} {} {}"#,
    		query,
    		operation_args,
    		T::query(),
    	);
    	query.push_str("}");
    	let doc = gql::parse_query(&query)?;
    	let operation = convert(doc)?;
    	let schema = self.query_schema.clone();
    	let response = self.executor.execute(operation, schema).await?;
    	if response.errors.len() > 0 {
    	    // return the errors.
    	    todo!("proper error")
    	}

    	// serialize from map to Queryable.
    	let value = serde_json::to_value(&response.data.get("findOneUser").unwrap())
    		.expect("Deserializing to serde_json::Value should be infallible");
    	let data = serde_json::from_value(value)
    		.expect("Validation is done by prisma_client_derive::Queryable, this is infallible");
    	Ok(data)
    }
    pub async fn create_user<T>(&self, data: UserCreateInput) -> Result<T, Error>
        where
            T: Queryable + DeserializeOwned,
    {
        let query = String::from("mutation { createOneUser");
    	let mut operation_args = String::new();
        operation_args.push_str(&format!(
        	"data: {},",
        	to_query_args(data).unwrap()
        ));
    	if !operation_args.is_empty() {
    		operation_args = format!("({})", operation_args);
    	}
    	let mut query = format!(r#"
    		{} {} {}"#,
    		query,
    		operation_args,
    		T::query(),
    	);
    	query.push_str("}");
    	let doc = gql::parse_query(&query)?;
    	let operation = convert(doc)?;
    	let schema = self.query_schema.clone();
    	let response = self.executor.execute(operation, schema).await?;
    	if response.errors.len() > 0 {
    	    // return the errors.
    	    todo!("proper error")
    	}

    	// serialize from map to Queryable.
    	let value = serde_json::to_value(&response.data.get("createOneUser").unwrap())
    		.expect("Deserializing to serde_json::Value should be infallible");
    	let data = serde_json::from_value(value)
    		.expect("Validation is done by prisma_client_derive::Queryable, this is infallible");
    	Ok(data)
    }
    pub async fn delete_user<T>(&self, filter: UserWhereUniqueInput) -> Result<T, Error>
        where
            T: Queryable + DeserializeOwned,
    {
        let query = String::from("mutation { deleteOneUser");
    	let mut operation_args = String::new();
        operation_args.push_str(&format!(
        	"where: {},",
        	to_query_args(filter).unwrap()
        ));
    	if !operation_args.is_empty() {
    		operation_args = format!("({})", operation_args);
    	}
    	let mut query = format!(r#"
    		{} {} {}"#,
    		query,
    		operation_args,
    		T::query(),
    	);
    	query.push_str("}");
    	let doc = gql::parse_query(&query)?;
    	let operation = convert(doc)?;
    	let schema = self.query_schema.clone();
    	let response = self.executor.execute(operation, schema).await?;
    	if response.errors.len() > 0 {
    	    // return the errors.
    	    todo!("proper error")
    	}

    	// serialize from map to Queryable.
    	let value = serde_json::to_value(&response.data.get("deleteOneUser").unwrap())
    		.expect("Deserializing to serde_json::Value should be infallible");
    	let data = serde_json::from_value(value)
    		.expect("Validation is done by prisma_client_derive::Queryable, this is infallible");
    	Ok(data)
    }
    pub async fn update_user<T>(&self, data: UpdateOneUserArgs) -> Result<T, Error>
        where
            T: Queryable + DeserializeOwned,
    {
        let query = String::from("mutation { updateOneUser");
    	let mut operation_args = String::new();
        operation_args.push_str(&format!(
        	"data: {},",
        	to_query_args(&data.data).unwrap()
        ));
        operation_args.push_str(&format!(
        	"where: {},",
        	to_query_args(&data.filter).unwrap()
        ));
    	if !operation_args.is_empty() {
    		operation_args = format!("({})", operation_args);
    	}
    	let mut query = format!(r#"
    		{} {} {}"#,
    		query,
    		operation_args,
    		T::query(),
    	);
    	query.push_str("}");
    	let doc = gql::parse_query(&query)?;
    	let operation = convert(doc)?;
    	let schema = self.query_schema.clone();
    	let response = self.executor.execute(operation, schema).await?;
    	if response.errors.len() > 0 {
    	    // return the errors.
    	    todo!("proper error")
    	}

    	// serialize from map to Queryable.
    	let value = serde_json::to_value(&response.data.get("updateOneUser").unwrap())
    		.expect("Deserializing to serde_json::Value should be infallible");
    	let data = serde_json::from_value(value)
    		.expect("Validation is done by prisma_client_derive::Queryable, this is infallible");
    	Ok(data)
    }
    pub async fn upsert_user<T>(&self, data: UpsertOneUserArgs) -> Result<T, Error>
        where
            T: Queryable + DeserializeOwned,
    {
        let query = String::from("mutation { upsertOneUser");
    	let mut operation_args = String::new();
        operation_args.push_str(&format!(
        	"where: {},",
        	to_query_args(&data.filter).unwrap()
        ));
        operation_args.push_str(&format!(
        	"create: {},",
        	to_query_args(&data.create).unwrap()
        ));
        operation_args.push_str(&format!(
        	"update: {},",
        	to_query_args(&data.update).unwrap()
        ));
    	if !operation_args.is_empty() {
    		operation_args = format!("({})", operation_args);
    	}
    	let mut query = format!(r#"
    		{} {} {}"#,
    		query,
    		operation_args,
    		T::query(),
    	);
    	query.push_str("}");
    	let doc = gql::parse_query(&query)?;
    	let operation = convert(doc)?;
    	let schema = self.query_schema.clone();
    	let response = self.executor.execute(operation, schema).await?;
    	if response.errors.len() > 0 {
    	    // return the errors.
    	    todo!("proper error")
    	}

    	// serialize from map to Queryable.
    	let value = serde_json::to_value(&response.data.get("upsertOneUser").unwrap())
    		.expect("Deserializing to serde_json::Value should be infallible");
    	let data = serde_json::from_value(value)
    		.expect("Validation is done by prisma_client_derive::Queryable, this is infallible");
    	Ok(data)
    }
    pub async fn update_users(&self, data: UpdateManyUserArgs) -> Result<BatchPayload, Error>
        
    {
        let query = String::from("mutation { updateManyUser");
    	let mut operation_args = String::new();
        operation_args.push_str(&format!(
        	"data: {},",
        	to_query_args(&data.data).unwrap()
        ));
        if data.filter.is_some() {
        	operation_args.push_str(&format!(
        		"where: {},",
        		to_query_args(data.filter.as_ref().unwrap()).unwrap()
        	));
        }
    	if !operation_args.is_empty() {
    		operation_args = format!("({})", operation_args);
    	}
    	let mut query = format!(r#"
    		{} {} {}"#,
    		query,
    		operation_args,
    		"{ count }",
    	);
    	query.push_str("}");
    	let doc = gql::parse_query(&query)?;
    	let operation = convert(doc)?;
    	let schema = self.query_schema.clone();
    	let response = self.executor.execute(operation, schema).await?;
    	if response.errors.len() > 0 {
    	    // return the errors.
    	    todo!("proper error")
    	}

    	// serialize from map to Queryable.
    	let value = serde_json::to_value(&response.data.get("updateManyUser").unwrap())
    		.expect("Deserializing to serde_json::Value should be infallible");
    	let data = serde_json::from_value(value)
    		.expect("Validation is done by prisma_client_derive::Queryable, this is infallible");
    	Ok(data)
    }
    pub async fn delete_users(&self, filter: Option<UserWhereInput>) -> Result<BatchPayload, Error>
        
    {
        let query = String::from("mutation { deleteManyUser");
    	let mut operation_args = String::new();
        if filter.is_some() {
        	operation_args.push_str(&format!(
        		"where: {},",
        		to_query_args(filter.as_ref().unwrap()).unwrap()
        	));
        }
    	if !operation_args.is_empty() {
    		operation_args = format!("({})", operation_args);
    	}
    	let mut query = format!(r#"
    		{} {} {}"#,
    		query,
    		operation_args,
    		"{ count }",
    	);
    	query.push_str("}");
    	let doc = gql::parse_query(&query)?;
    	let operation = convert(doc)?;
    	let schema = self.query_schema.clone();
    	let response = self.executor.execute(operation, schema).await?;
    	if response.errors.len() > 0 {
    	    // return the errors.
    	    todo!("proper error")
    	}

    	// serialize from map to Queryable.
    	let value = serde_json::to_value(&response.data.get("deleteManyUser").unwrap())
    		.expect("Deserializing to serde_json::Value should be infallible");
    	let data = serde_json::from_value(value)
    		.expect("Validation is done by prisma_client_derive::Queryable, this is infallible");
    	Ok(data)
    }
}

#[derive(derive_more::From, derive_more::Display, Debug)]
pub enum Error {
	QueryError(query_core::error::CoreError),
	GraphqlParseError(gql::query::ParseError),
	UrlParseError(url::ParseError),
	QueryConnector(query_connector::error::ConnectorError),
	Other(String),
}

async fn load_executor(
	source: &(dyn Source + Send + Sync),
	force_transactions: bool,
) -> Result<(String, Box<dyn QueryExecutor + Send + Sync + 'static>), Error> {
	match source.connector_type() {
		"sqlite" => {
			log::info!("Loading SQLite connector.");
			let sqlite = Sqlite::from_source(source).await?;
			log::info!("Loaded SQLite connector.");

			let path = PathBuf::from(sqlite.file_path());
			let db_name = path.file_stem().unwrap().to_str().unwrap().to_owned(); // Safe due to previous validations.
			let executor = sql_executor("sqlite", sqlite, false);
			Ok((db_name, executor))
		},
		"mysql" => {
			log::info!("Loading MySQL connector.");
			let mysql = Mysql::from_source(source).await?;
			log::info!("Loaded MySQL connector.");
			let executor = sql_executor("mysql", mysql, false);
			let url = Url::parse(&source.url().value)?;
			let err_str = "No database found in connection string";

			let mut db_name = url
				.path_segments()
				.ok_or_else(|| Error::Other(err_str.into()))?;

			let db_name = db_name.next().expect(err_str).to_owned();
			Ok((db_name, executor))
		},
		"postgresql" => {
			let url = Url::parse(&source.url().value)?;
			let params: HashMap<String, String> = url.query_pairs().into_owned().collect();

			let db_name = params
				.get("schema")
				.map(ToString::to_string)
				.unwrap_or_else(|| String::from("public"));

			log::info!("Loading Postgres connector.");
			let psql = PostgreSql::from_source(source).await?;
			log::info!("Loaded Postgres connector.");
			let executor = sql_executor("postgres", psql, force_transactions);

			Ok((db_name, executor))
		},
		x => Err(Error::Other(format!(
			"Unsupported connector type: {}",
			x
		))),
	}
}

fn sql_executor<T>(
	primary_connector: &'static str,
	connector: T,
	force_transactions: bool,
) -> Box<dyn QueryExecutor + Send + Sync + 'static>
	where
		T: Connector + Send + Sync + 'static,
{
	Box::new(InterpretingExecutor::new(
		connector,
		primary_connector,
		force_transactions,
	))
}

fn convert(gql_doc: Document<String>) -> Result<Operation, Error> {
	let mut operations: Vec<Operation> = gql_doc
		.definitions
		.into_iter()
		.map(convert_definition)
		.collect::<Result<Vec<Vec<Operation>>, Error>>()
		.map(|r| r.into_iter().flatten().collect::<Vec<Operation>>())?;

	let operation = operations
		.pop()
		.ok_or_else(|| Error::Other("Document contained no operations.".into()))?
		.dedup_selections();

	Ok(operation)
}

fn convert_definition(def: Definition<String>) -> Result<Vec<Operation>, Error> {
	match def {
		Definition::Fragment(f) => Err(Error::Other(
			format!("Fragment '{}', at position {}.", f.name, f.position),
		)),
		Definition::Operation(op) => match op {
			OperationDefinition::Subscription(s) => Err(Error::Other(
				format!("At position {}.", s.position),
			)),
			OperationDefinition::SelectionSet(s) => convert_query(s),
			OperationDefinition::Query(q) => convert_query(q.selection_set),
			OperationDefinition::Mutation(m) => convert_mutation(m.selection_set),
		},
	}
}

fn convert_query(selection_set: SelectionSet<String>) -> Result<Vec<Operation>, Error> {
	convert_selection_set(selection_set)
		.map(|fields| fields.into_iter().map(|field| Operation::Read(field)).collect())
}

fn convert_mutation(selection_set: SelectionSet<String>) -> Result<Vec<Operation>, Error> {
	convert_selection_set(selection_set).map(|fields| {
		fields
			.into_iter()
			.map(|selection| Operation::Write(selection))
			.collect()
	})
}

fn convert_selection_set(selection_set: SelectionSet<String>) -> Result<Vec<Selection>, Error> {
	selection_set
		.items
		.into_iter()
		.map(|item| match item {
			GqlSelection::Field(f) => {
				let arguments: Vec<(String, QueryValue)> = f
					.arguments
					.into_iter()
					.map(|(k, v)| Ok((k, convert_value(v)?)))
					.collect::<Result<Vec<_>, Error>>()?;

				let mut builder = Selection::builder(f.name);
				builder.set_arguments(arguments);
				builder.nested_selections(convert_selection_set(f.selection_set)?);

				if let Some(alias) = f.alias {
					builder.alias(alias);
				};

				Ok(builder.build())
			}

			GqlSelection::FragmentSpread(fs) => Err(Error::Other(
				format!("Fragment '{}', at position {}.", fs.fragment_name, fs.position),
			)),

			GqlSelection::InlineFragment(i) => Err(Error::Other(
				format!("At position {}.", i.position),
			)),
		})
		.collect()
}


fn convert_value(value: Value<String>) -> Result<QueryValue, Error> {
	match value {
		Value::Variable(name) => Err(Error::Other(
			format!("Variable '{}'.", name),
		)),
		Value::Int(i) => match i.as_i64() {
			Some(i) => Ok(QueryValue::Int(i)),
			None => Err(Error::Other(format!(
				"Invalid 64 bit integer: {:?}",
				i
			))),
		},
		// We can't use Decimal::from_f64 here due to a bug in rust_decimal.
		// Issue: https://github.com/paupino/rust-decimal/issues/228<Paste>
		Value::Float(f) => match Decimal::from_str(&f.to_string()).ok() {
			Some(dec) => Ok(QueryValue::Float(dec)),
			None => Err(Error::Other(format!(
				"invalid 64-bit float: {:?}",
				f
			))),
		},
		Value::String(s) => Ok(QueryValue::String(s)),
		Value::Boolean(b) => Ok(QueryValue::Boolean(b)),
		Value::Null => Ok(QueryValue::Null),
		Value::Enum(e) => Ok(QueryValue::Enum(e)),
		Value::List(values) => {
			let values: Vec<QueryValue> = values
				.into_iter()
				.map(convert_value)
				.collect::<Result<Vec<QueryValue>, Error>>()?;

			Ok(QueryValue::List(values))
		}
		Value::Object(map) => {
			let values = map
				.into_iter()
				.map(|(k, v)| convert_value(v).map(|v| (k, v)))
				.collect::<Result<BTreeMap<String, QueryValue>, Error>>()?;

			Ok(QueryValue::Object(values))
		}
	}
}
