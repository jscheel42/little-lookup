use crate::schema::items;
use crate::schema::items::columns::*;
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
    pub fn list(connection: &SqliteConnection, sql_filter: String) -> Self {
        use diesel::RunQueryDsl;
        use diesel::QueryDsl;
        use crate::schema::items::dsl::*;

        let result = 
            items
                .filter(key.like(sql_filter))
                .load::<Item>(connection)
                .expect("Error loading items");

        ItemList(result)
    }
}

impl Item {
    pub fn find(id: &str, connection: &SqliteConnection) -> Result<Item, diesel::result::Error> {
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;

        items::table.filter(key.eq(id))
            .first(connection)
    }

    pub fn destroy(id: &str, connection: &SqliteConnection) -> Result<(), diesel::result::Error> {
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;
        use crate::schema::items::dsl;

        diesel::delete(dsl::items
            .filter(key.eq(id))
            .execute(connection))?;
        Ok(())
    }

    pub fn replace_into(id: &str, new_item: &NewItem, connection: &SqliteConnection) -> Result<(), diesel::result::Error> {
        // use diesel::QueryDsl;
        use diesel::RunQueryDsl;
        // use crate::schema::items::dsl;

        diesel::replace_into(items::table)
            .values(new_item)
            .execute(connection)
            .expect("Error creating new item");
        Ok(())
    }
}