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