use std::{fs, path::PathBuf};
use datamodel::parse_datamodel;
use query_engine::dmmf::{render_dmmf, schema::{DMMFTypeInfo, DMMFOutputType}};
use query_core::{SupportedCapabilities, QuerySchemaBuilder, BuildMode};
use prisma_models::DatamodelConverter;
use std::sync::Arc;
use inflector::Inflector;
use serde_json::{json, Value};

/// Generates the client.
pub fn generate(datamodel: &str, out_dir: &str) {
	// todo: mechanism for checking if model file has changed
	let model = fs::read_to_string(PathBuf::from(datamodel))
		.expect("failed to read .prisma file");

	let out_dir = PathBuf::from(out_dir);
	fs::create_dir_all(out_dir.join("prisma-client/src")).unwrap();
	fs::write(
		out_dir.join("prisma-client/Cargo.toml"),
		include_str!("templates/Cargo.toml.template")
	).unwrap();
	fs::write(
		out_dir.join("prisma-client/src/serialize.rs"),
		include_str!("templates/serialization.rs.template")
	).unwrap();
	fs::write(
		out_dir.join("prisma-client/src/lib.rs"),
		generate_client(&model)
	).unwrap();
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
	let mut tt = tinytemplate::TinyTemplate::new();
	tt.add_template("client", include_str!("./templates/lib.rs.template")).unwrap();

	let enums = dmmf.schema.enums
		.iter()
		.map(|enu| {
			json!({
				"name": enu.name,
				"variants": enu.values.iter()
				.map(|v| {
					json!({
						"render": v.to_class_case(),
						"actual": v,
					})
				})
				.collect::<Vec<_>>(),
			})
		})
		.collect::<Vec<_>>();

	let inputs = dmmf.schema.input_types.iter()
		.map(|typ| {
			let fields = typ.fields.iter()
				.map(|field| {
					json!({
						"is_required": field.input_type.is_required,
						"name": json!({
							"render": field.name.to_snake_case(),
							"actual": field.name,
						}),
						"type": format_to_rust_type(&field.input_type),
					})
				})
				.collect::<Vec<_>>();

			json!({
				"name": typ.name.to_pascal_case(),
				"fields": fields,
			})
		})
		.collect::<Vec<_>>();

	let (query, mutation) = (
		dmmf.schema.root_query_type,
		dmmf.schema.root_mutation_type
	);

	let mut operations = Vec::new();
	let outputs = dmmf.schema.output_types.iter()
		.filter_map(|typ| {
			if typ.name == query || typ.name == mutation {
				let op = build_operation(typ);
				operations.push(op);
				return None
			}

			let fields = typ.fields.iter()
				.map(|field| {
					json!({
						"is_required": field.output_type.is_required,
						"name": field.name,
						"type": format_to_rust_type(&field.output_type),
					})
				})
				.collect::<Vec<_>>();

			Some(json!({
				"name": typ.name.to_pascal_case(),
				"fields": fields,
			}))
		})
		.collect::<Vec<_>>();

	let data = json!({
		"operations": operations,
		"inputs": inputs,
		"outputs": outputs,
		"enums": enums,
		"datamodel": model_str,
	});

	tt.render("client", &data).unwrap()
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

fn build_operation(out: &DMMFOutputType) -> Value {
	let operation = out.name.to_lowercase();
	let (methods, outputs) = out.fields
		.iter()
		.fold((Vec::new(), Vec::new()), |(mut methods, mut outputs), field| {
			let mut arg = format!(", data: {}", format_arg_name(&field.name));

			let only = field.args.len() == 1;
			let args = field.args
				.iter()
				.map(|arg| {
					json!({
						"is_required": arg.input_type.is_required,
						"name": arg.name,
						"type": format_to_rust_type(&arg.input_type),
						"name_filtered": if arg.name == "where" {
							"filter"
						} else if arg.name == "orderBy" {
							"order_by"
						} else {
							&arg.name
						},
						"only": only,
					})
				})
				.collect::<Vec<_>>();

			if only {
				let a = args.first().unwrap();
				arg = format!(
					", {}: {}",
					a["name_filtered"].as_str().unwrap(),
					a["type"].as_str().unwrap()
				);
			} else if !field.name.contains("aggregate") {
				let output = json!({
					"name": format_arg_name(&field.name),
					"fields": args,
				});
				outputs.push(output);
			}

			if field.name.contains("aggregate") {
				arg = "".into();
			}

			let use_batch = field.name.contains("deleteMany") || field.name.contains("updateMany")
				|| field.name.contains("aggregate");

			let generics = if !use_batch {
				"<T>"
			} else {
				""
			};

			let return_ty = if use_batch {
				"BatchPayload"
			} else if field.name.contains("findOne") || field.name.contains("deleteOne")   {
				"Option<T>"
			} else {
				"T"
			};

			let method = json!({
				"fn_name": format_method_name(field.name.clone()),
				"query_name": field.name,
				"args": args,
				"arg": arg,
				"generics": generics,
				"return_optional": field.name.contains("findOne"),
				"is_batch": use_batch,
				"query": if use_batch { r#""{ count }""# } else { "T::query()" },
				"return": return_ty
			});

			methods.push(method);

			(methods, outputs)
		});

	json!({
		"name": operation,
		"methods": methods,
		"outputs": outputs,
	})
}

fn format_arg_name(name: &str) -> String {
	format!("{}Args", name.to_pascal_case())
}

/// formats method name from
/// `findManyUser` to `users`
/// `findOneUser` to `user`
/// `deleteOneUser` to `delete_user`, updateOneUser` to `update_user`,
/// `deleteManyUser` to `delete_users`, updateManyUser` to `update_users`,
fn format_method_name(name: String) -> String {
	if name.contains("findMany") {
		return name.replace("findMany", " ").to_snake_case().to_lowercase().to_plural()
	}

	if name.contains("findOne") {
		return name.replace("findOne", "").to_snake_case().to_lowercase()
	}

	if name.contains("One") {
		return name.replace("One", " ").to_snake_case().to_lowercase()
	}

	name.replace("Many", " ").to_snake_case().to_lowercase().to_plural()
}

#[cfg(test)]
mod test {
	#[test]
	fn generate_client() {
		let out = super::generate_client(r##"datasource pg {
	provider = "mysql"
	url = "mysql://root:prisma@localhost:3306/default@default"
}

model User {
    id String @id @default(cuid())
    name String
    email String @unique
    password String
    subaccount String
}"##);
		println!("{}", out);
	}
}