use crate::diesel::{ExpressionMethods, Insertable, QueryDsl, Queryable, RunQueryDsl, TextExpressionMethods}; 
use crate::schema::items;

use diesel::sqlite::SqliteConnection;

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
    pub fn list(connection: &SqliteConnection, sql_filter: String) -> Result<std::vec::Vec<Item>, diesel::result::Error> {
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
    pub fn find(id: &str, connection: &SqliteConnection) -> Result<Item, diesel::result::Error> {
        use crate::schema::items::dsl::{items, key};

        items.filter(key.eq(id))
            .first(connection)
    }

    pub fn destroy(id: &str, connection: &SqliteConnection) -> Result<usize, diesel::result::Error> {
        use crate::schema::items::dsl::{items, key};

        let delete_count = diesel::delete(
                items.filter(
                    key.eq(id)
                )
            )
            .execute(connection).unwrap_or_default();
        Ok(delete_count)
    }

    pub fn replace_into(id: &str, value: &str, connection: &SqliteConnection) -> Result<(), diesel::result::Error> {
        let new_item = NewItem {
            key: id,
            val: value,
        };

        diesel::replace_into(items::table)
            .values(new_item)
            .execute(connection)
            .expect("Error creating new item");
        Ok(())
    }
}