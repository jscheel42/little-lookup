use super::schema::items;

#[derive(Queryable)]
pub struct Item {
    pub key: String,
    pub val: String
}

#[derive(Insertable)]
#[table_name = "items"]
pub struct NewItem<'a> {
    pub key: &'a str,
    pub val: &'a str
}