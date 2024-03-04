use crate::diesel::{ExpressionMethods, Insertable, QueryDsl, Queryable, RunQueryDsl, TextExpressionMethods};
use crate::schema::items;
use chrono::{DateTime, Utc};

use diesel::pg::PgConnection;

pub struct ItemList(pub Vec<Item>);

#[derive(Queryable)]
pub struct Item {
    pub key: String,
    pub val: String,
    pub updated_at: DateTime<Utc>,
    pub namespace: String,
}

#[derive(Insertable)]
#[diesel(table_name = items)]
pub struct NewItem<'a> {
    pub key: &'a str,
    pub val: &'a str,
    pub updated_at: &'a DateTime<Utc>,
    pub namespace: &'a str,
}

impl ItemList {
    pub fn list(connection: &mut PgConnection, sql_filter: String, namespace_id: &str) -> Result<std::vec::Vec<Item>, diesel::result::Error> {
        use crate::schema::items::dsl::{items, key, updated_at, namespace};

        let f = format!("%{}%", sql_filter);
        let result = 
            items
                .filter(key.like(f))
                .filter(namespace.eq(namespace_id))
                .order_by((key, updated_at.desc()))
                .distinct_on(key)
                .load::<Item>(connection)?;

        Ok(result)
    }
}

impl Item {
    pub fn find(key_id: &str, namespace_id: &str, connection: &mut PgConnection) -> Result<Item, diesel::result::Error> {
        use crate::schema::items::dsl::{items, key, updated_at, namespace};

        items.filter(key.eq(key_id))
            .filter(namespace.eq(namespace_id))
            .order_by(updated_at.desc())
            .first(connection)
    }
    
    pub fn history(key_id: &str, namespace_id: &str, connection: &mut PgConnection) -> Result<Vec<Item>, diesel::result::Error> {
        use crate::schema::items::dsl::{items, key, updated_at, namespace};

        items.filter(key.eq(key_id))
            .filter(namespace.eq(namespace_id))
            .order_by(updated_at.desc())
            .get_results(connection)
    }

    pub fn destroy(key_id: &str, namespace_id: &str, connection: &mut PgConnection) -> Result<usize, diesel::result::Error> {
        use crate::schema::items::dsl::{items, key, namespace};

        let delete_count = diesel::delete(
                items.filter(key.eq(key_id))
                    .filter(namespace.eq(namespace_id))
            )
            .execute(connection).unwrap_or_default();
        Ok(delete_count)
    }

    pub fn replace_into(key_id: &str, value: &str, namespace_id: &str, connection: &mut PgConnection) -> Result<(), diesel::result::Error> {
        let q = format!("INSERT INTO items(key, val, namespace) VALUES ('{0}', '{1}', '{2}')",
                        key_id, value, namespace_id);
        diesel::sql_query(q).execute(connection)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::util::get_database;

    use super::*;
    use diesel::prelude::*;
    // Helper function to establish a database connection
    fn establish_connection() -> PgConnection {
        let database_url = get_database();
        PgConnection::establish(&database_url).expect("Failed to connect to database")
    }

    #[test]
    fn test_list_items() {
        let mut connection = establish_connection();
        let sql_filter = String::from("example");
        let namespace_id = "my_namespace";

        let result = ItemList::list(&mut connection, sql_filter, namespace_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_history_items() {
        let mut connection = establish_connection();
        let key_id = "example_key";
        let namespace_id = "my_namespace";

        let result = Item::history(key_id, namespace_id, &mut connection);
        assert!(result.is_ok());
    }

    #[test]
    fn test_destroy_item() {
        let mut connection = establish_connection();
        let key_id = "example_key";
        let namespace_id = "my_namespace";

        let result = Item::destroy(key_id, namespace_id, &mut connection);
        assert!(result.is_ok());
    }

    #[test]
    fn test_replace_into_item_and_find() {
        let mut connection = establish_connection();
        let key_id = "example_key";
        let value: &str = "example_value";
        let namespace_id = "my_namespace";

        let result = Item::replace_into(key_id, value, namespace_id, &mut connection);
        assert!(result.is_ok());

        let result = Item::find(key_id, namespace_id, &mut connection);
        assert!(result.is_ok());
    }
}
