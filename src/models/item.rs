use crate::schema::items;
// use crate::schema::items::columns::*;
use diesel::sqlite::SqliteConnection;
use crate::diesel::{ExpressionMethods, Insertable, QueryDsl, Queryable, RunQueryDsl, TextExpressionMethods}; 

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
    pub fn list(connection: &SqliteConnection, sql_filter: String) -> Result<ItemList, diesel::result::Error> {
        use crate::schema::items::dsl::{items, key};

        let result = 
            items
                .filter(key.like(sql_filter))
                .load::<Item>(connection)?;
                // .expect("Error loading items");

        Ok(ItemList(result))
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

    pub fn replace_into(id: &str, new_item: &NewItem, connection: &SqliteConnection) -> Result<(), diesel::result::Error> {
        diesel::replace_into(items::table)
            .values(new_item)
            .execute(connection)
            .expect("Error creating new item");
        Ok(())
    }
}