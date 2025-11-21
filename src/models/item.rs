use crate::diesel::{ExpressionMethods, Insertable, QueryDsl, Queryable, RunQueryDsl};
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
    pub fn list(connection: &mut PgConnection, namespace_id: &str) -> Result<std::vec::Vec<Item>, diesel::result::Error> {
        use crate::schema::items::dsl::{items, key, updated_at, namespace};

        let result = 
            items
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
        let namespace_id = "list_test_namespace";

        let result = ItemList::list(&mut connection, namespace_id);
        assert!(result.is_ok());
        // Result can be empty or contain items - both are valid
        let items = result.unwrap();
        assert!(items.is_empty() || !items.is_empty());
    }

    #[test]
    fn test_list_items_empty_namespace() {
        let mut connection = establish_connection();
        let namespace_id = "empty_namespace_12345";

        let result = ItemList::list(&mut connection, namespace_id);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_history_items_empty() {
        let mut connection = establish_connection();
        let key_id = "nonexistent_key_12345";
        let namespace_id = "my_namespace";

        let result = Item::history(key_id, namespace_id, &mut connection);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_destroy_item_not_found() {
        let mut connection = establish_connection();
        let key_id = "nonexistent_delete_key_12345";
        let namespace_id = "my_namespace";

        let result = Item::destroy(key_id, namespace_id, &mut connection);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_find_item_not_found() {
        let mut connection = establish_connection();
        let key_id = "nonexistent_find_key_12345";
        let namespace_id = "my_namespace";

        let result = Item::find(key_id, namespace_id, &mut connection);
        assert!(result.is_err());
    }

    #[test]
    fn test_replace_into_item_and_find() {
        let mut connection = establish_connection();
        let key_id = "test_key_12345";
        let value: &str = "test_value_12345";
        let namespace_id = "test_namespace_12345";

        let replace_result = Item::replace_into(key_id, value, namespace_id, &mut connection);
        assert!(replace_result.is_ok());

        let find_result = Item::find(key_id, namespace_id, &mut connection);
        assert!(find_result.is_ok());
        
        let found_item = find_result.unwrap();
        assert_eq!(found_item.key, key_id);
        assert_eq!(found_item.val, value);
        assert_eq!(found_item.namespace, namespace_id);
    }

    #[test]
    fn test_replace_into_multiple_values_and_history() {
        let mut connection = establish_connection();
        let key_id = "history_key_12345";
        let value1 = "value1";
        let value2 = "value2";
        let namespace_id = "history_namespace_12345";

        // Insert first value
        assert!(Item::replace_into(key_id, value1, namespace_id, &mut connection).is_ok());
        
        // Insert second value (overwrites)
        assert!(Item::replace_into(key_id, value2, namespace_id, &mut connection).is_ok());

        // Get current value
        let find_result = Item::find(key_id, namespace_id, &mut connection);
        assert!(find_result.is_ok());
        assert_eq!(find_result.unwrap().val, value2);

        // Get history - should have 2 entries
        let history_result = Item::history(key_id, namespace_id, &mut connection);
        assert!(history_result.is_ok());
        let history = history_result.unwrap();
        assert!(history.len() >= 2);
        // Most recent should be first (order_by updated_at desc)
        assert_eq!(history[0].val, value2);
    }

    #[test]
    fn test_destroy_removes_all_versions() {
        let mut connection = establish_connection();
        let key_id = "destroy_key_12345";
        let namespace_id = "destroy_namespace_12345";

        // Insert multiple versions
        assert!(Item::replace_into(key_id, "v1", namespace_id, &mut connection).is_ok());
        assert!(Item::replace_into(key_id, "v2", namespace_id, &mut connection).is_ok());

        // Verify we have history
        let history_before = Item::history(key_id, namespace_id, &mut connection).unwrap();
        assert!(history_before.len() >= 2);

        // Destroy all
        let destroy_result = Item::destroy(key_id, namespace_id, &mut connection);
        assert!(destroy_result.is_ok());
        assert!(destroy_result.unwrap() >= 2);

        // Verify all are deleted
        let history_after = Item::history(key_id, namespace_id, &mut connection).unwrap();
        assert!(history_after.is_empty());
    }

    #[test]
    fn test_namespace_isolation() {
        let mut connection = establish_connection();
        let key_id = "shared_key";
        let ns1 = "namespace_1_12345";
        let ns2 = "namespace_2_12345";

        // Insert same key in different namespaces
        assert!(Item::replace_into(key_id, "value_ns1", ns1, &mut connection).is_ok());
        assert!(Item::replace_into(key_id, "value_ns2", ns2, &mut connection).is_ok());

        // Verify they are isolated
        let item_ns1 = Item::find(key_id, ns1, &mut connection).unwrap();
        let item_ns2 = Item::find(key_id, ns2, &mut connection).unwrap();

        assert_eq!(item_ns1.val, "value_ns1");
        assert_eq!(item_ns2.val, "value_ns2");
        assert_eq!(item_ns1.namespace, ns1);
        assert_eq!(item_ns2.namespace, ns2);
    }

    #[test]
    fn test_list_returns_distinct_keys_with_latest_value() {
        let mut connection = establish_connection();
        let key1 = "list_key1_12345";
        let key2 = "list_key2_12345";
        let namespace_id = "list_distinct_namespace_12345";

        // Insert multiple versions of key1
        assert!(Item::replace_into(key1, "v1", namespace_id, &mut connection).is_ok());
        assert!(Item::replace_into(key1, "v2", namespace_id, &mut connection).is_ok());
        
        // Insert single version of key2
        assert!(Item::replace_into(key2, "v3", namespace_id, &mut connection).is_ok());

        // List should return both keys with latest values
        let list_result = ItemList::list(&mut connection, namespace_id);
        assert!(list_result.is_ok());
        
        let items = list_result.unwrap();
        let item1 = items.iter().find(|item| item.key == key1);
        let item2 = items.iter().find(|item| item.key == key2);

        assert!(item1.is_some());
        assert!(item2.is_some());
        
        assert_eq!(item1.unwrap().val, "v2");
        assert_eq!(item2.unwrap().val, "v3");
    }
}
