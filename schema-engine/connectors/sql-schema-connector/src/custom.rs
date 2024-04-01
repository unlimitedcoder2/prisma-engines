use schema_connector::DatabaseSchema;

use crate::{sql_schema_calculator, SqlSchemaConnector};

/// Create a schema from a prisma source string.
pub fn create_schema(source: &str) -> Result<DatabaseSchema, String> {
    let pg_connector = SqlSchemaConnector::new_postgres();
    let schema = psl::parse_schema(source)?;

    match pg_connector.flavour.check_schema_features(&schema) {
        Ok(_) => (),
        Err(e) => {
            return Err(format!("Error checking schema features: {}", e));
        }
    };

    let schema = sql_schema_calculator::calculate_sql_schema(&schema, pg_connector.flavour.as_ref());
    Ok(DatabaseSchema::from(schema))
}
