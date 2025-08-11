use std::fmt::{self, Display};

use crate::helixc::generator::{
    return_values::ReturnValue,
    statements::Statement,
    utils::{EmbedData, GeneratedType},
};

pub struct Query {
    pub embedding_model_to_use: Option<String>,
    pub mcp_handler: Option<String>,
    pub name: String,
    pub statements: Vec<Statement>,
    pub parameters: Vec<Parameter>, // iterate through and print each one
    pub sub_parameters: Vec<(String, Vec<Parameter>)>,
    pub return_values: Vec<ReturnValue>,
    pub is_mut: bool,
    pub hoisted_embedding_calls: Vec<EmbedData>,
}
impl Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // prints sub parameter structs (e.g. (docs: {doc: String, id: String}))
        for (name, parameters) in &self.sub_parameters {
            writeln!(f, "#[derive(Serialize, Deserialize)]")?;
            writeln!(f, "pub struct {name} {{")?;
            for parameter in parameters {
                writeln!(f, "    pub {}: {},", parameter.name, parameter.field_type)?;
            }
            writeln!(f, "}}")?;
        }
        // prints top level parameters (e.g. (docs: {doc: String, id: String}))
        // if !self.parameters.is_empty() {
        writeln!(f, "#[derive(Serialize, Deserialize)]")?;
        writeln!(f, "pub struct {}Input {{\n", self.name)?;
        write!(
            f,
            "{}",
            self.parameters
                .iter()
                .map(|p| format!("{p}"))
                .collect::<Vec<_>>()
                .join(",\n")
        )?;
        write!(f, "\n}}\n")?;
        // }

        if let Some(mcp_handler) = &self.mcp_handler {
            writeln!(
                f,
                "#[tool_call({}, {})]",
                mcp_handler,
                match self.is_mut {
                    true => "with_write",
                    false => "with_read",
                }
            )?;
        }
        // writeln!(
        //     f,
        //     "#[handler({})]",
        //     match self.is_mut {
        //         true => "with_write",
        //         false => "with_read",
        //     }
        // )?; // Handler macro

        // prints the function signature
        writeln!(
            f,
            "pub fn {} (input: HandlerInput) -> Result<Response, GraphError> {{",
            self.name
        )?;

        if !self.hoisted_embedding_calls.is_empty() {
            writeln!(
                f,
                "Err(IoContFn::create_err(|__internal_cont_tx, __internal_ret_chan| Box::pin(async move {{"
            )?;
            // ((({ })))

            for (i, embed) in self.hoisted_embedding_calls.iter().enumerate() {
                let name = EmbedData::name_from_index(i);
                writeln!(f, "let {name} = {embed};")?;
            }

            writeln!(
                f,
                "__internal_cont_tx.send_async((__internal_ret_chan, Box::new(move || {{"
            )?;
            // ((({ }))).await.expect("Cont Channel should be alive")
        }

        writeln!(
            f,
            "let data = input.request.in_fmt.deserialize::<{}Input>(&input.request.body)?;",
            self.name
        )?;
        writeln!(f, "let mut remapping_vals = RemappingMap::new();")?;
        writeln!(f, "let db = Arc::clone(&input.graph.storage);")?;

        match self.is_mut {
            true => writeln!(f, "let mut txn = db.graph_env.write_txn().unwrap();")?,
            false => writeln!(f, "let txn = db.graph_env.read_txn().unwrap();")?,
        }

        // prints each statement
        for statement in &self.statements {
            writeln!(f, "    {statement};")?;
        }

        // commit the transaction
        // writeln!(f, "    txn.commit().unwrap();")?;

        // create the return values
        writeln!(
            f,
            "let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();"
        )?;
        if !self.return_values.is_empty() {
            for return_value in &self.return_values {
                writeln!(f, "    {return_value}")?;
            }
        }

        writeln!(f, "txn.commit().unwrap();")?;
        writeln!(f, "Ok(input.request.out_fmt.create_response(&return_vals))")?;

        if !self.hoisted_embedding_calls.is_empty() {
            writeln!(f, r#"}}))).await.expect("Cont Channel should be alive")"#)?;
            writeln!(f, "}})))")?;
        }

        writeln!(f, "}}")?;

        writeln!(f, "#[doc(hidden)]")?;
        writeln!(f, "#[used]")?;
        writeln!(
            f,
            "static _MAIN_HANDLER_REGISTRATION_{}: () = {{",
            self.name.to_uppercase()
        )?;
        writeln!(f, "inventory::submit! {{")?;
        writeln!(
            f,
            "::helix_db::helix_gateway::router::router::HandlerSubmission("
        )?;
        writeln!(
            f,
            r#"::helix_db::helix_gateway::router::router::Handler::new( "{}", {}))}}}}; "#,
            self.name, self.name
        )?;

        Ok(())
    }
}
impl Default for Query {
    fn default() -> Self {
        Self {
            embedding_model_to_use: None,
            mcp_handler: None,
            name: "".to_string(),
            statements: vec![],
            parameters: vec![],
            sub_parameters: vec![],
            return_values: vec![],
            is_mut: false,
            hoisted_embedding_calls: vec![],
        }
    }
}

impl Query {
    pub fn add_hoisted_embed(&mut self, embed_data: EmbedData) -> String {
        let name = EmbedData::name_from_index(self.hoisted_embedding_calls.len());
        self.hoisted_embedding_calls.push(embed_data);
        name
    }
}

pub struct Parameter {
    pub name: String,
    pub field_type: GeneratedType,
    pub is_optional: bool,
}
impl Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.is_optional {
            true => write!(f, "pub {}: Option<{}>", self.name, self.field_type),
            false => write!(f, "pub {}: {}", self.name, self.field_type),
        }
    }
}
