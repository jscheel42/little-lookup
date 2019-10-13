use crate::diesel::{ExpressionMethods, Insertable, QueryDsl, Queryable, RunQueryDsl, TextExpressionMethods}; 
use crate::schema::items;

use diesel::pg::PgConnection;

pub struct ItemList(pub Vec<Item>);

#[derive(Queryable)]
pub struct Item {
    pub key: String,
    pub val: String,
}

#[derive(Insertable)]
#[table_name = "items"]
pub struct NewItem<'a> {
    pub key: &'a str,
    pub val: &'a str,
}

impl ItemList {
    pub fn list(connection: &PgConnection, sql_filter: String) -> Result<std::vec::Vec<Item>, diesel::result::Error> {
        use crate::schema::items::dsl::{items, key};

        let f = format!("%{}%", sql_filter);
        let result = 
            items
                .filter(key.like(f))
                .load::<Item>(connection)?;

        Ok(result)
    }
}

impl Item {
    pub fn find(id: &str, connection: &PgConnection) -> Result<Item, diesel::result::Error> {
        use crate::schema::items::dsl::{items, key};

        items.filter(key.eq(id))
            .first(connection)
    }

    pub fn destroy(id: &str, connection: &PgConnection) -> Result<usize, diesel::result::Error> {
        use crate::schema::items::dsl::{items, key};

        let delete_count = diesel::delete(
                items.filter(
                    key.eq(id)
                )
            )
            .execute(connection).unwrap_or_default();
        Ok(delete_count)
    }

    pub fn replace_into(id: &str, value: &str, connection: &PgConnection) -> Result<(), diesel::result::Error> {
        let q = format!("INSERT INTO items(key, val) VALUES ('{0}', '{1}') 
                        ON CONFLICT ON CONSTRAINT items_pkey
                        DO UPDATE SET val = '{1}';",
                        id, value);
        diesel::sql_query(q).execute(connection)?;
        Ok(())
    }
}