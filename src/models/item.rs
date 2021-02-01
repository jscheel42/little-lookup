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
}

#[derive(Insertable)]
#[table_name = "items"]
pub struct NewItem<'a> {
    pub key: &'a str,
    pub val: &'a str,
    pub updated_at: &'a DateTime<Utc>,
}

impl ItemList {
    pub fn list(connection: &PgConnection, sql_filter: String) -> Result<std::vec::Vec<Item>, diesel::result::Error> {
        use crate::schema::items::dsl::{items, key, updated_at};

        let f = format!("%{}%", sql_filter);
        let result = 
            items
                .filter(key.like(f))
                .order_by((key, updated_at.desc()))
                .distinct_on(key)
                .load::<Item>(connection)?;

        Ok(result)
    }
}

impl Item {
    pub fn find(id: &str, connection: &PgConnection) -> Result<Item, diesel::result::Error> {
        use crate::schema::items::dsl::{items, key, updated_at};

        items.filter(key.eq(id))
            .order_by(updated_at.desc())
            .first(connection)
    }
    
    pub fn history(id: &str, connection: &PgConnection) -> Result<Vec<Item>, diesel::result::Error> {
        use crate::schema::items::dsl::{items, key, updated_at};

        items.filter(key.eq(id))
            .order_by(updated_at.desc())
            .get_results(connection)
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
        let q = format!("INSERT INTO items(key, val) VALUES ('{0}', '{1}')",
                        id, value);
        diesel::sql_query(q).execute(connection)?;
        Ok(())
    }
}