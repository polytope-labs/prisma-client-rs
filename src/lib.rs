use std::{env, fs, path::PathBuf};
use datamodel::{parse_datamodel, parse_configuration};
use prisma::dmmf::render_dmmf;
use query_core::{QueryDocument, SupportedCapabilities, QuerySchemaBuilder, BuildMode};
use itertools::Itertools;
use prisma_models::DatamodelConverter;
use std::sync::Arc;
use inflector::Inflector;
use prisma::dmmf::schema::{DMMFTypeInfo, DMMFArgument, DMMFField};


const SERIALIZE_ARGS: &'static str = include_str!("serialization.rs.template");

// todo: mechanism for checking if db file has changed
pub fn generate() {
	let mut manifest_dir = PathBuf::from(
		env::var("CARGO_MANIFEST_DIR").expect("`CARGO_MANIFEST_DIR` is always set by cargo.")
	);
	let cargo_toml = r#"
[package]
name = "prisma-client"
version = "0.1.0"
authors = ["Seun <seunlanlege@gmail.com>"]
edition = "2018"

[dependencies]
query-core = { path = "../../../prisma-engines/query-engine/core" }
prisma-models = { path = "../../../prisma-engines/libs/prisma-models" }
prisma = { path = "../../../prisma-engines/query-engine/prisma" }
datamodel = { path = "../../../prisma-engines/libs/datamodel/core" }
itertools = "0.9.0"
serde_json = "1.0.50"
Inflector = "0.11"
serde = { version = "1.0.106", features = ["serde_derive"] }
derive_more = "0.99.5"
itoa = "0.4.5"
ryu = "1.0.3"
"#;
	let model = fs::read_to_string(manifest_dir.join("./datamodel.prisma")).unwrap();
	fs::create_dir_all(manifest_dir.join("prisma-client/src")).unwrap();
	fs::write(manifest_dir.join("prisma-client/Cargo.toml"), cargo_toml).unwrap();
	fs::write(manifest_dir.join("prisma-client/src/serialize.rs"), SERIALIZE_ARGS).unwrap();
	fs::write(manifest_dir.join("prisma-client/src/lib.rs"), generate_client(&model)).unwrap();
}

fn generate_client(model_str: &str) -> String {
	let model = parse_datamodel(model_str).unwrap();
	let internal_model = DatamodelConverter::convert(&model).build("".into());
	let cap = SupportedCapabilities::empty();
	let schema_builder = QuerySchemaBuilder::new(
		&internal_model,
		&cap,
		BuildMode::Modern,
		false,
	);
	let query_schema = Arc::new(schema_builder.build());
	let dmmf = render_dmmf(&model, query_schema);
	let enums = dmmf.schema.enums
		.iter()
		.map(|enu| {
			format!(
				r#"
#[derive(Serialize)]
pub enum {} {{
	{}
}}"#,
				enu.name,
				// todo: fix enum value
				enu.values.iter().join(",\n\t")
			)
		})
		.join("\n");

	let inputs = dmmf.schema.input_types.iter()
		.map(|typ| {
			let fields = typ.fields.iter()
				.map(|field| {
					let skip = if field.input_type.is_required {
						"    "
					} else {
						r#"    #[serde(skip_serializing_if = "Option::is_none")]
	"#
					};
					format!(
						r#"{}pub {}: {},"#,
						skip,
						field.name,
						format_to_rust_type(&field.input_type)
					)
				})
				.join("\n");
			format!(
				r#"
#[derive(Default, Serialize)]
pub struct {} {{
{}
}}"#,
				typ.name.to_pascal_case(),
				fields
			)
		})
		.join("\n");

	let (query, mutation) = (
		dmmf.schema.root_query_type,
		dmmf.schema.root_mutation_type
	);

	let (mut methods, mut argument_definitions) = (String::new(), String::new());
	let outputs = dmmf.schema.output_types.iter()
		.filter_map(|typ| {
			if typ.name == query || typ.name == mutation {
				let (m, ad) = build_root_fields(typ);
				methods.push_str(&m);
				argument_definitions.push_str(&ad);
				return None
			}

			let fields = typ.fields.iter()
				.map(|field| {
					format!("\tpub {}: {},", field.name, format_to_rust_type(&field.output_type))
				})
				.join("\n");
			Some(format!(
				r#"
pub struct {} {{
{}
}}
"#,
				typ.name,
				fields
			))
		})
		.join("\n");

	let client = format!(r###"
use datamodel::{{parse_datamodel, parse_configuration}};
use prisma::{{
	context::{{PrismaContext, ContextBuilder}},
	dmmf::render_dmmf, request_handlers::{{graphql::*, PrismaRequest}}
}};
use query_core::{{QueryDocument, SupportedCapabilities, QuerySchemaBuilder, BuildMode}};
use itertools::Itertools;
use prisma_models::DatamodelConverter;
use std::{{sync::Arc, collections::HashMap}};
use std::convert::TryFrom;
use std::convert::AsRef;
use serde::{{de::DeserializeOwned, Serialize}};
mod serialize;
use serialize::to_query_args;

{}
{}
{}
{}
pub trait Queryable: DeserializeOwned {{
	fn query() -> String;
}}

impl<T: Queryable> Queryable for Vec<T> {{
	fn query() -> String {{
		T::query()
	}}
}}

impl Queryable for u128 {{
	fn query() -> String {{
		String::new()
	}}
}}

impl Queryable for u64 {{
	fn query() -> String {{
		String::new()
	}}
}}

impl Queryable for u32 {{
	fn query() -> String {{
		String::new()
	}}
}}

impl Queryable for u16 {{
	fn query() -> String {{
		String::new()
	}}
}}

impl Queryable for u8 {{
	fn query() -> String {{
		String::new()
	}}
}}

impl Queryable for i128 {{
	fn query() -> String {{
		String::new()
	}}
}}

impl Queryable for i64 {{
	fn query() -> String {{
		String::new()
	}}
}}

impl Queryable for i32 {{
	fn query() -> String {{
		String::new()
	}}
}}

impl Queryable for i16 {{
	fn query() -> String {{
		String::new()
	}}
}}

impl Queryable for i8 {{
	fn query() -> String {{
		String::new()
	}}
}}

impl Queryable for &str {{
	fn query() -> String {{
		String::new()
	}}
}}

pub struct Prisma {{
	context: PrismaContext,
}}

#[derive(derive_more::From, derive_more::Display, Debug)]
pub enum Error {{
	PrismaError(prisma::error::PrismaError),
	QueryError(query_core::error::CoreError)
}}

impl Prisma {{
	pub async fn init() -> Result<Self, Error> {{
		let datamodel_str = r#"{}"#;
		let model = parse_datamodel(datamodel_str).unwrap();
		let config = parse_configuration(datamodel_str).unwrap();
		let context = PrismaContext::builder(config, model)
		    .enable_raw_queries(true)
		    .build()
		    .await?;
		Ok(Self {{ context }})
	}}
{}
}}
	"###, enums, inputs, outputs, argument_definitions, model_str, methods);
	println!("{}", client);
	client
}

/// converts DMMFTypeInfo to a rust type.
fn format_to_rust_type(typ: &DMMFTypeInfo) -> String {
	let typ_name = match typ.typ.as_str() {
		"Int" => "u64",
		_ => &typ.typ,
	};
	let formatted = if typ.is_list {
		format!("Vec<{}>", typ_name)
	} else {
		typ_name.to_owned()
	};

	if !typ.is_required {
		format!("Option<{}>", formatted)
	} else {
		formatted
	}
}

fn build_root_fields(out: &prisma::dmmf::schema::DMMFOutputType) -> (String, String) {
	let (methods, method_arg_def) = (Vec::new(), Vec::new());
	let (a, b) = out.fields.iter()
		.fold((methods, method_arg_def), |(mut a, mut b), field| {
			let args = if field.args.len() > 0 {
				let formated = field.args.iter()
					.map(|arg| {
						if !arg.input_type.is_required {
							format!(
								r#"
		if data.{filter}.is_some() {{
			let x = "{name}";
			operation_args.push_str(&format!(
				"{{}}: {{}},",
				x,
				to_query_args(data.{filter}.as_ref().unwrap()).unwrap()
			));
		}}
"#,
								name=arg.name,
								filter=if arg.name == "where" { "filter" } else { &arg.name }
							)
						} else {
							format!(
								r#"
		let x = "{name}";
		operation_args.push_str(&format!(
			"{{}}: {{}},",
			x,
			to_query_args(&data.{filter}).unwrap()
		));
"#,
								name=arg.name,
								filter=if arg.name == "where" { "filter" } else { &arg.name }
							)
						}

					})
					.join("\n");
				formated
			} else {
				"".into()
			};

			let operation = out.name.to_lowercase();
			a.push(format!(
				r###"
	pub async fn {method_name}<T: Queryable>(&self, {method_arg}) -> Result<T, Error> {{
		let mut query = String::from("{operation} {{\n {method_name}");
		let mut operation_args = String::new();
		{args}
		if !operation_args.is_empty() {{
			operation_args = format!("({{}})", operation_args);
		}}
		let mut query = format!(r#"
			{{}} {{}} {{}}
			"#,
			query,
			operation_args,
			T::query(),
		);
		query.push_str("}}");
		println!("{{}}", query);
		let mut query = SingleQuery {{
		    query,
		    // note: prisma doesn't yet support variables.
		    variables: HashMap::new(),
		    operation_name: None,
		}};
		let body = GraphQlBody::Single(query);
		let doc = match QueryDocument::try_from(body).unwrap() {{
		    QueryDocument::Single(op) => op,
		    _ => unreachable!("body is a single query."),
		}};
		let schema = Arc::clone(self.context.query_schema());

		let response = self.context.executor.execute(doc, schema).await?;
		if response.errors.len() > 0 {{
		    // return the errors.
		    todo!("proper error")
		}}

		// serialize from map to Queryable.
		let value = serde_json::to_value(&response.data["{method_name}"])
			.expect("Deserializing to serde_json::Value should be infallible");
		let data = serde_json::from_value(value)
			.expect("Validation is done by prisma_client_derive::Queryable, this is infallible");
		Ok(data)
	}}"###,
				operation=operation,
				method_name=field.name,
				args=args,
				method_arg=format!("data: {}", format_arg_name(&field.name)),
			));
			b.push(build_args_struct(&field.name, &out.name, &field.args));
			(a, b)
		});
	(a.join("\n"), b.join("\n"))
}

fn format_arg_name(name: &str) -> String {
	format!("{}Args", name.to_pascal_case())
}

fn format_args(args: &Vec<DMMFArgument>, filter: bool) -> String {
	args.iter()
		.map(|arg| {
			format!(
				"\tpub {}: {},",
				if filter && arg.name == "where" { "filter" } else { &arg.name },
				format_to_rust_type(&arg.input_type)
			)
		})
		.join("\n")
}

fn build_args_struct(name: &str, kind: &str, args: &Vec<DMMFArgument>) -> String {
	// todo: fix name
	format!(r#"
#[derive(Default, Serialize)]
pub struct {} {{
{}
}}"#, format_arg_name(name), format_args(args, true))
}

#[cfg(test)]
mod test {
	#[test]
	fn generate_client() {}
}