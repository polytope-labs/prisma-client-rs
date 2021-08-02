//! Given a prisma datamodel, generates a full prisma client with all the appropriate methods and types.

use std::{fs, path::PathBuf, sync::Arc};
use serde::Serialize;
use serde_json::{json, Value};
use inflector::Inflector;

use datamodel::parse_datamodel;
use prisma_models::{dml::Field, DatamodelConverter};
use query_core::{BuildMode, schema_builder};
use datamodel_connector::ConnectorCapabilities;
use request_handlers::dmmf::{
    render_dmmf, schema::{DmmfTypeReference, TypeLocation, DmmfInputField, DmmfOutputType},
};

#[derive(Debug, Serialize, Clone)]
struct TypeName {
    rename: bool,
    render: String,
    actual: String,
}

#[derive(Debug, Serialize, Clone)]
struct Enum {
    name: String,
    variants: Vec<TypeName>,
}

#[derive(Debug, Serialize, Clone)]
struct TypeField {
    is_required: bool,
    r#type: String,
    name: TypeName,
}

#[derive(Debug, Serialize, Clone)]
struct Type {
    name: String,
    fields: Vec<TypeField>,
}

/// Generates the client.
pub fn write_to_dir(datamodel: &str, path: PathBuf) {
	let model_str = fs::read_to_string(PathBuf::from(datamodel))
		.expect("failed to read .prisma file");
    fs::write(path, generate(&model_str))
        .expect("Error while writing to prisma.rs");
}

fn generate(model_str: &str) -> String {
    let model = parse_datamodel(&model_str).unwrap();
    let internal_model = DatamodelConverter::convert(&model.subject).build("".into());
    let query_schema = Arc::new(schema_builder::build(
        internal_model,
        BuildMode::Modern,
        true,
        ConnectorCapabilities::empty(),
        vec![]
    ));
    let mut dmmf = render_dmmf(&model.subject, query_schema);

    let models = model.subject
        .models
        .into_iter()
        .map(|m| {
            m.fields
                .into_iter()
                .filter_map(|f| {
                    if f.is_relation() {
                        Some(f)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .flatten()
        .collect::<Vec<_>>();

    let enums = dmmf.schema
        .enum_types
        .remove("prisma")
        .unwrap()
        .into_iter()
        .map(|enu| {
            let variants = enu
                .values
                .iter()
                .map(|v| TypeName {
                    render: v.to_class_case(),
                    rename: true,
                    actual: v.clone(),
                })
                .collect::<Vec<_>>();

            Enum {
                name: enu.name,
                variants,
            }
        })
        .collect::<Vec<_>>();

    let inputs = dmmf.schema.input_object_types
        .remove("prisma")
        .unwrap()
        .into_iter()
        .filter_map(|input_type| {
            if !input_type.name.contains("Unchecked") {
                Some((input_type.name, input_type.fields))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let (inputs, inputs_enums) = build_inputs(inputs, &models);

    let (outputs, others) = dmmf.schema.output_object_types
        .remove("prisma")
        .unwrap()
        .into_iter()
        .partition(|output_type| {
            if output_type.name == "Query" || output_type.name == "Mutation" {
                false
            } else {
                true
            }
        });
    let outputs = build_outupts(outputs, &models);

    let operations: Vec<Value> = others
        .into_iter()
        .filter_map(|typ| build_operation(typ, &models))
        .collect();

    let data = json!({
        "operations": operations,
        "inputs": inputs,
        "outputs": outputs,
        "enums": enums,
        "input_enums": inputs_enums,
        "datamodel": model_str,
    });

    let mut tt = tinytemplate::TinyTemplate::new();
    tt.add_template("client", include_str!("./prisma.rs.template"))
        .expect("Couldn't add Template");
    tt.render("client", &data).expect("Couldn't write to template")
}

/// Convert input objects to TypeField
fn build_inputs(inputs: Vec<(String, Vec<DmmfInputField>)>, models: &Vec<Field>) -> (Vec<Type>, Vec<Enum>) {
    let mut inputs_enums = vec![];
    let types = inputs
        .into_iter()
        .map(|(input_name, input_type)| {
            let fields = input_type
                .iter()
                .filter_map(|field| {
                    if field.deprecation.is_some() || input_name.contains("Unchecked") {
                        // no point generating unchecked input types
                        return None
                    }
                    let name = match &*field.name {
                        "where" => "filter".to_owned(),
                        "in" => "within".to_owned(),
                        _ => field.name.to_snake_case()
                    };

                    let is_relation = is_relation(models, &field.name);
                    let filtered_types = field.input_types.iter()
                        .filter_map(|typ_ref| {
                            if typ_ref.is_list ||
                                typ_ref.typ.contains("Unchecked") ||
                                &typ_ref.typ == "Null"
                            {
                                None
                            } else {
                                Some(typ_ref)
                            }
                        })
                        .collect::<Vec<_>>();

                    if filtered_types.len() > 1 {
                        inputs_enums.push(Enum {
                            name: format!("{}{}", &input_name.to_pascal_case(), field.name.to_pascal_case()),
                            variants: filtered_types.iter()
                                .map(|type_ref| {
                                    let typ = dmmf_type_to_rust(&type_ref, false);
                                    if type_ref.typ == "Null" {
                                        TypeName {
                                            render: "Null".into(),
                                            rename: true,
                                            actual: "null".into(),
                                        }
                                    } else {
                                        TypeName {
                                            render: format!("{}({})", type_ref.typ, typ),
                                            rename: false,
                                            actual: format!("")
                                        }
                                    }
                                })
                                .collect::<Vec<_>>(),
                        });
                    }

                    let type_field = TypeField {
                        is_required: field.is_required,
                        name: TypeName {
                            render: name,
                            rename: true,
                            actual: field.name.clone(),
                        },
                        r#type: format(&field, &input_name, is_relation)
                    };

                    Some(type_field)
                })
                .collect::<Vec<_>>();

            Type {
                name: input_name.to_pascal_case(),
                fields,
            }
        })
        .collect::<Vec<_>>();

    (types, inputs_enums)
}

/// Convert output objects to TypeField
fn build_outupts(outputs: Vec<DmmfOutputType>, models: &Vec<Field>) -> Vec<Type> {
    outputs.iter()
        .map(|output_type| {
            let fields = output_type.fields
                .iter()
                .filter_map(|field| {
                    if field.deprecation.is_some() {
                        return None
                    }
                    let is_relation = is_relation(&models, &field.name);
                    let formatted = dmmf_type_to_rust(&field.output_type, is_relation);
                    let formatted = if field.is_nullable {
                        format!("Option<{}>", formatted)
                    } else {
                        formatted
                    };
                    Some(TypeField {
                        is_required: !field.is_nullable,
                        name: TypeName {
                            render: field.name.to_snake_case(),
                            rename: true,
                            actual: field.name.clone(),
                        },
                        r#type: formatted,
                    })
                })
                .collect::<Vec<_>>();

            Type {
                name: output_type.name.to_pascal_case(),
                fields,
            }
        })
        .collect::<Vec<_>>()
}

fn is_relation(models: &Vec<Field>, name: &str) -> bool {
    models
        .iter()
        .filter_map(|f| {
            if name.contains(&f.name()) {
                Some(())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .len()
        > 0
}

fn format(input: &DmmfInputField, name: &str, needs_box: bool) -> String {
    let is_update = name.contains("UpdateInput");
    let needs_box = needs_box || name.to_lowercase().contains("nested");

    let without_unchecked_input = input.input_types.iter()
        .filter_map(|typ_ref| {
            if typ_ref.typ.contains("Unchecked") || &typ_ref.typ == "Null" {
                None
            } else {
                Some(typ_ref)
            }
        })
        .collect::<Vec<_>>();

    // we want to know if there are only 2 possible input types and one is a list.
    let has_list_variant = if without_unchecked_input.len() == 2 {
        input.input_types.iter()
            .find(|typ_ref| typ_ref.is_list)
    } else {
        None
    };

    // if there's a list in the input types, default to it
    let formatted = if let Some(list) = has_list_variant {
        dmmf_type_to_rust(list, needs_box)
    } else if without_unchecked_input.len() > 1 {
        let mut typ_name = format!("{}{}", name.to_pascal_case(), input.name.to_pascal_case());
        if needs_box {
            typ_name = format!("Box<{}>", typ_name);
        }
        typ_name
    } else {
        dmmf_type_to_rust(&without_unchecked_input[0], needs_box)
    };

    if input.is_nullable && is_update {
        format!("Option<Option<{}>>", formatted)
    } else if !input.is_required {
        format!("Option<{}>", formatted)
    } else {
        formatted
    }
}

/// converts DmmfTypeReference to a rust type
fn dmmf_type_to_rust(type_ref: &DmmfTypeReference, needs_box: bool) -> String {
    let formatted = match type_ref.typ.as_str() {
        // graphql scalar types.
        "Int" => "i32",
        "Float" => "f32",
        "Boolean" => "bool",
        "DateTime" => "chrono::DateTime<chrono::Utc>",
        "Json" => "serde_json::Value",
        _ => &type_ref.typ,
    };

    let formatted = match type_ref.location {
        TypeLocation::InputObjectTypes | TypeLocation::OutputObjectTypes if needs_box => format!("Box<{}>", formatted),
        _ => formatted.to_string(),
    };

    if type_ref.is_list {
        format!("Vec<{}>", formatted)
    } else {
        formatted
    }
}

fn build_operation(out: DmmfOutputType, models: &Vec<Field>) -> Option<Value> {
    let operation = out.name.to_lowercase();

    let (input_types, input_enums, methods) = out.fields
        .into_iter()
        .map(|field| {
            // this operation takes more than one argyment, bundle it all into a struct.
            let fn_arg = if field.args.len() > 1 {
                format!("data: {}Args", field.name.to_class_case())
            } else {
                // otherwise, just use the default input object
                format!("data: {}", field.args[0].input_types[0].typ)
            };

            let args = field
                .args
                .iter()
                .map(|arg| TypeField {
                    is_required: arg.is_required || field.args.len() == 1,
                    name: TypeName {
                        render: {
                            let name = match arg.name.as_str() {
                                "where" => "filter".to_owned(),
                                "orderBy" => "order_by".to_owned(),
                                _ => arg.name.clone(),
                            };

                            // if its not the only arg, then it's in a struct
                            if field.args.len() > 1 {
                                format!("data.{}", name)
                            } else {
                                "data".to_string()
                            }
                        },
                        rename: true,
                        actual: arg.name.clone(),
                    },
                    r#type: String::new(),
                })
                .collect::<Vec<_>>();

                let mut return_ty = String::from("T");
                if field.output_type.is_list {
                    return_ty = format!("Vec<{}>", return_ty)
                }
                if field.is_nullable {
                    return_ty = format!("Option<{}>", return_ty)
                }

                let query_name = field.name.clone();

                let method = json!({
                    "fn_name": format_method_name(field.name.clone()),
                    "fn_return": return_ty,
                    "fn_arg": fn_arg,
                    "query_name": query_name,
                    "query_args": args,
                });

            let (input_type, input_enums) = if field.args.len() > 1 {
                build_inputs(vec![(format!("{}Args", field.name), field.args)], models)
            } else {
                (vec![], vec![])
            };

            (input_type, input_enums, method)
        })
        .fold((vec![], vec![], vec![]), |mut acc, (input_type, input_enum, method)| {
            acc.0.extend_from_slice(&input_type);
            acc.1.extend_from_slice(&input_enum);
            acc.2.push(method);
            acc
        });

    Some(json!({
        "name": operation,
        "methods": methods,
        "input_types": input_types,
        "input_enums": input_enums,
    }))
}

/// formats method name from
/// findFirstUser - first_user
/// findManyUser - users
/// aggregateUser - aggregate_users
/// groupByUser - group_users
/// updateOneUser - update_user
/// upsertOneUser - upsert_user
/// updateManyUser - update_users
/// findUniqueUser - user
/// createOneUser - create_user
/// deleteManyUser - delete_users
/// deleteOneUser - delete_user
fn format_method_name(name: String) -> String {
    if name.contains("findMany") {
        return name
            .replace("findMany", "")
            .to_snake_case()
            .to_lowercase()
            .to_plural();
    }

    if name.contains("findFirst") {
        return name
            .replace("findFirst", "first ")
            .to_snake_case()
            .to_lowercase()
    }

    if name.contains("aggregate") {
        return name
            .replace("aggregate", "aggregate ")
            .to_lowercase()
            .to_snake_case()
            .to_plural()
    }

    if name.contains("groupBy") {
        return name
            .replace("groupBy", "group ")
            .to_lowercase()
            .to_snake_case()
            .to_plural()
    }

    if name.contains("findUnique") {
        return name.replace("findUnique", "").to_snake_case().to_lowercase();
    }

    if name.contains("One") {
        return name.replace("One", " ").to_snake_case().to_lowercase();
    }

    name.replace("Many", " ")
        .to_snake_case()
        .to_lowercase()
        .to_plural()
}

#[cfg(test)]
mod test {
    #[test]
    fn generate_client() {
        let out = super::generate(r##"
            datasource pg {
	            provider = "mysql"
	            url = "mysql://root:prisma@localhost:3306/default@default"
            }

            model User {
              id    Int     @id @default(autoincrement())
              email String  @unique
              name  String?
              posts Post[]
            }

            model Post {
              id        Int      @id @default(autoincrement())
              createdAt DateTime @default(now())
              updatedAt DateTime @updatedAt
              title     String
              content   String?
              published Boolean  @default(false)
              viewCount Int      @default(0)
              author    User?    @relation(fields: [authorId], references: [id])
              authorId  Int?
            }
        "##);

        // println!("{}", out);
    }
}
