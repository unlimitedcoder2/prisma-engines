use schema_connector::DatabaseSchema;
use sql_schema_describer::{self as sql, SqlSchema};

#[derive(Default, Debug)]
#[allow(missing_docs)]
pub struct SqlDatabaseSchema {
    pub describer_schema: SqlSchema,
    /// A _sorted_ array of column ids with prisma-level defaults.
    pub prisma_level_defaults: Vec<sql::TableColumnId>,
}

impl SqlDatabaseSchema {
    pub fn from_erased(erased: DatabaseSchema) -> Box<Self> {
        erased.downcast()
    }

    pub fn walk<I>(&self, id: I) -> sql::Walker<'_, I> {
        self.describer_schema.walk(id)
    }
}

impl From<SqlSchema> for SqlDatabaseSchema {
    fn from(describer_schema: SqlSchema) -> Self {
        SqlDatabaseSchema {
            describer_schema,
            ..Default::default()
        }
    }
}

impl From<SqlDatabaseSchema> for DatabaseSchema {
    fn from(s: SqlDatabaseSchema) -> Self {
        DatabaseSchema::new(s)
    }
}
