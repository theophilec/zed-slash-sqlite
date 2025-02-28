use zed_extension_api::{
    self as zed, SlashCommand, SlashCommandOutput, SlashCommandOutputSection, Worktree,
};
struct SlashSqliteSchemaExtension;
use rusqlite::{Connection, Result};

pub struct SqliteDatabaseInfo {
    pub name: String,
    pub tables: Vec<TableInfo>,
}

pub struct TableInfo {
    pub name: String,
    pub type_: String,
    // notnull: bool,
    // dflt_value: Option<String>,
    // primary_key: bool,
}

pub fn schema(path: &str) -> Result<SqliteDatabaseInfo> {
    let conn = Connection::open(path)?;

    let mut tables_query = conn.prepare("SELECT name FROM sqlite_master WHERE type='table';")?;

    let rows = tables_query.query_map([], |row| row.get(0))?;

    let table_names = rows.collect::<Result<Vec<String>, _>>()?;

    let mut tables_info = Vec::new();

    for table_name in &table_names {
        let query = format!("SELECT * FROM pragma_table_info({table_name:?});");
        let mut table_query = conn.prepare(&query)?;
        let _ = table_query.query_map([], |row| {
            let table_info = TableInfo {
                name: row.get(1)?,
                type_: row.get(2)?,
            };
            tables_info.push(table_info);
            Ok(())
        })?;
    }
    Ok(SqliteDatabaseInfo {
        name: path.to_string(),
        tables: tables_info,
    })
}

impl zed::Extension for SlashSqliteSchemaExtension {
    fn new() -> Self {
        SlashSqliteSchemaExtension
    }

    fn complete_slash_command_argument(
        &self,
        command: SlashCommand,
        _args: Vec<String>,
    ) -> Result<Vec<zed_extension_api::SlashCommandArgumentCompletion>, String> {
        match command.name.as_str() {
            "schema" => Ok(vec![]),
            command => Err(format!("unknown slash command: \"{command}\"")),
        }
    }

    fn run_slash_command(
        &self,
        command: SlashCommand,
        args: Vec<String>,
        _worktree: Option<&Worktree>,
    ) -> Result<SlashCommandOutput, String> {
        match command.name.as_str() {
            "schema" => {
                if args.is_empty() {
                    return Err("need path to db".to_string());
                }

                let path = &args[0];

                let schema = schema(path.as_str());

                let schema = schema.unwrap();
                let mut result = String::new();
                for table in schema.tables {
                    result.push_str(&format!("Table: {} Type: {}\n", table.name, table.type_));
                }
                Ok(SlashCommandOutput {
                    sections: vec![SlashCommandOutputSection {
                        range: (0.."toto".to_string().len()).into(),
                        label: path.clone(),
                    }],
                    text: "toto".to_string(),
                })
            }
            command => Err(format!("unknown slash command: \"{command}\"")),
        }
    }
}

zed::register_extension!(SlashSqliteSchemaExtension);
