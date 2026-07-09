use sqlx::{FromRow, MySql, MySqlConnection,Error};

pub mod user;




pub trait GeneralDbModelsTrait {
    type Model: for<'r> FromRow<'r, sqlx::mysql::MySqlRow> + Send + Unpin;

    fn table_name() -> &'static str;

    fn find_by_id<'a>(
        id: i32,
        conn: &'a mut MySqlConnection,
    ) -> impl Future<Output = Result<Option<Self::Model>, Error>> + 'a {
        let query = format!("SELECT * FROM {} WHERE id = ?", Self::table_name());
        async move {
            sqlx::query_as::<MySql, Self::Model>(&query)
                .bind(id)
                .fetch_optional(conn)
                .await
        }
    }
}