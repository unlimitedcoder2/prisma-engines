use std::{fs, sync::Arc};

use psl::builtin_connectors::PostgresType;
use psl::SourceFile;
use schema_connector::{DiffTarget, SchemaConnector};
use serde::{Deserialize, Serialize};
use sql_schema_connector::database_schema::SqlDatabaseSchema;
use sql_schema_connector::SqlSchemaConnector;
use sql_schema_describer::{
    ColumnArity, ColumnTypeFamily, EnumId, ForeignKeyAction, ForeignKeyWalker, IndexType, IndexWalker, SqlSchema,
    TableColumnWalker, TableWalker,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "t", content = "c", rename_all = "snake_case")]
enum DatabaseType {
    SmallInt,
    Int,
    BigInt,
    Float,
    Decimal,
    Boolean,
    Binary,
    Enum(EnumId),
    Unsupported(String),
    Money,
    Inet,
    Oid,
    Citext,
    // Real,
    Double,
    VarChar(Option<u32>),
    Char(Option<u32>),
    Text,
    ByteA,
    Timestamp(Option<u32>),
    Timestamptz(Option<u32>),
    Date,
    Time(Option<u32>),
    Timetz(Option<u32>),
    /* Bit(Option<u32>),
    VarBit(Option<u32>), */
    Uuid,
    Xml,
    Json,
    JsonB,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct DatabaseColumn {
    name: String,
    arity: ColumnArity,
    tpe: DatabaseType,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct DatabaseForeignKey {
    name: String,
    table_name: String,
    referenced_table_name: String,
    column_names: Vec<String>,
    referenced_column_names: Vec<String>,
    on_delete_action: ForeignKeyAction,
}

impl Into<DatabaseForeignKey> for ForeignKeyWalker<'_> {
    fn into(self) -> DatabaseForeignKey {
        DatabaseForeignKey {
            name: self.constraint_name().expect("No constraint name").to_string(),
            table_name: self.table().name().to_string(),
            referenced_table_name: self.referenced_table_name().to_string(),
            column_names: self
                .constrained_columns()
                .map(|c| c.name().to_string())
                .collect::<Vec<String>>(),
            referenced_column_names: self
                .referenced_columns()
                .map(|c| c.name().to_string())
                .collect::<Vec<String>>(),
            on_delete_action: self.on_delete_action(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct DatabaseTable {
    name: String,
    columns: Vec<DatabaseColumn>,
    indexes: Vec<DatabaseIndex>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct DatabaseIndex {
    name: String,
    tpe: IndexType,
    columns: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct DatabaseSchema {
    tables: Vec<DatabaseTable>,
    foreign_keys: Vec<DatabaseForeignKey>,
}

impl Into<DatabaseColumn> for TableColumnWalker<'_> {
    fn into(self) -> DatabaseColumn {
        DatabaseColumn {
            name: self.name().to_string(),
            arity: self.column_type().arity,
            tpe: get_column_type(
                self.column_native_type::<PostgresType>(),
                self.column_type().family.clone(),
            ),
        }
    }
}

fn get_column_type(postgres_type: Option<&PostgresType>, sql_type: ColumnTypeFamily) -> DatabaseType {
    if let Some(pg_type) = postgres_type {
        match pg_type {
            PostgresType::SmallInt => DatabaseType::SmallInt,
            PostgresType::Integer => DatabaseType::Int,
            PostgresType::BigInt => DatabaseType::BigInt,
            PostgresType::Decimal(_) => DatabaseType::Decimal,
            PostgresType::Money => DatabaseType::Money,
            PostgresType::Inet => DatabaseType::Inet,
            PostgresType::Oid => DatabaseType::Oid,
            PostgresType::Citext => DatabaseType::Citext,
            PostgresType::Real => DatabaseType::Decimal,
            PostgresType::DoublePrecision => DatabaseType::Double,
            PostgresType::VarChar(len) => DatabaseType::VarChar(*len),
            PostgresType::Char(len) => DatabaseType::Char(*len),
            PostgresType::Text => DatabaseType::Text,
            PostgresType::ByteA => DatabaseType::ByteA,
            PostgresType::Timestamp(precision) => DatabaseType::Timestamp(*precision),
            PostgresType::Timestamptz(precision) => DatabaseType::Timestamptz(*precision),
            PostgresType::Date => DatabaseType::Date,
            PostgresType::Time(precision) => DatabaseType::Time(*precision),
            PostgresType::Timetz(precision) => DatabaseType::Timetz(*precision),
            PostgresType::Boolean => DatabaseType::Boolean,
            PostgresType::Bit(_) =>
            /* DatabaseType::Bit(*len) */
            {
                DatabaseType::Binary
            }
            PostgresType::VarBit(_) =>
            /* DatabaseType::VarBit(*len) */
            {
                DatabaseType::Binary
            }
            PostgresType::Uuid => DatabaseType::Uuid,
            PostgresType::Xml => DatabaseType::Xml,
            PostgresType::Json => DatabaseType::Json,
            PostgresType::JsonB => DatabaseType::JsonB,
        }
    } else {
        match sql_type {
            ColumnTypeFamily::Int => DatabaseType::Int,
            ColumnTypeFamily::BigInt => DatabaseType::BigInt,
            ColumnTypeFamily::Float => DatabaseType::Float,
            ColumnTypeFamily::Decimal => DatabaseType::Decimal,
            ColumnTypeFamily::Boolean => DatabaseType::Boolean,
            ColumnTypeFamily::String => DatabaseType::Text,
            ColumnTypeFamily::DateTime => DatabaseType::Timestamp(None),
            ColumnTypeFamily::Binary => DatabaseType::Binary,
            ColumnTypeFamily::Json => DatabaseType::Json,
            ColumnTypeFamily::Uuid => DatabaseType::Uuid,
            ColumnTypeFamily::Enum(id) => DatabaseType::Enum(id),
            ColumnTypeFamily::Unsupported(t) => DatabaseType::Unsupported(t),
        }
    }
}

impl Into<DatabaseTable> for TableWalker<'_> {
    fn into(self) -> DatabaseTable {
        DatabaseTable {
            name: self.name().to_string(),
            columns: self.columns().map(|c| c.into()).collect::<Vec<DatabaseColumn>>(),
            indexes: self.indexes().map(|i| i.into()).collect::<Vec<DatabaseIndex>>(),
        }
    }
}

impl Into<DatabaseIndex> for IndexWalker<'_> {
    fn into(self) -> DatabaseIndex {
        DatabaseIndex {
            name: self.name().to_string(),
            tpe: self.index_type(),
            columns: self.columns().map(|c| c.name().to_string()).collect::<Vec<String>>(),
        }
    }
}

impl Into<DatabaseSchema> for SqlSchema {
    fn into(self) -> DatabaseSchema {
        DatabaseSchema {
            tables: self.table_walkers().map(|t| t.into()).collect::<Vec<DatabaseTable>>(),
            foreign_keys: self
                .walk_foreign_keys()
                .map(|f| f.into())
                .collect::<Vec<DatabaseForeignKey>>(),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let f = if cfg!(target_os = "windows") {
        fs::read_to_string("C:/Users/alex/dev/projects/rokkit/rokkit/core/schema.prisma")?
    } else {
        fs::read_to_string("/home/alex/dev/projects/rokkit/core/schema.prisma")?
    };

    let source = SourceFile::new_allocated(Arc::from(f));
    let mut pg_connector = SqlSchemaConnector::new_postgres();

    let schema = pg_connector
        .database_schema_from_diff_target(DiffTarget::Datamodel(source), None, None)
        .await?
        .downcast::<SqlDatabaseSchema>();

    let schema = schema.describer_schema;

    fs::write("schema.json", serde_json::to_string_pretty(&schema)?.as_str())?;

    let db_schema: DatabaseSchema = schema.into();

    fs::write("dbschema.json", serde_json::to_string_pretty(&db_schema)?.as_str())?;

    Ok(())
}
