use async_trait::async_trait;
use core::option::Option;
use sqlx::{MySqlPool, QueryBuilder};

#[async_trait]
pub trait Repository: Send + Sync {
    async fn get<T>(&self) -> Result<Vec<T>, sqlx::Error>
    where
        T: SqlObject + Send + Unpin + for<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow>;
    async fn batch(&self, vec: &Vec<LegoSet>) -> Option<String>;
}

pub struct SqlxRepository {
    pub pool: sqlx::MySqlPool,
}

#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct LegoSet {
    pub set_id: String,
    pub name: String,
    pub year: String,
    pub theme: String,
}

pub trait SqlObject {
    fn print(&self) -> String {
        "Object:".to_string()
    }
}

impl SqlObject for LegoSet {}

impl SqlxRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository for SqlxRepository {
    async fn get<T>(&self) -> Result<Vec<T>, sqlx::Error>
    where
        T: Send + Unpin + for<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow>,
    {
        let mut query_builder = QueryBuilder::new(
            "SELECT set_id, name, year, theme FROM ecommerce.lego_sets WHERE set_id = ",
        );
        query_builder.push_bind("10097775");
        let sets = query_builder
            .build_query_as::<T>()
            .fetch_all(&self.pool)
            .await?;

        Ok(sets)
    }

    async fn batch(&self, items: &Vec<LegoSet>) -> Option<String> {
        if items.is_empty() {
            return Some("1".to_string());
        }
        let mut query_builder =
            QueryBuilder::new("INSERT INTO lego_sets (set_id, name, year, theme)");

        query_builder.push_values(items, |mut b, item| {
            b.push_bind(item.set_id.clone())
                .push_bind(item.name.clone())
                .push_bind(item.year.clone())
                .push_bind(item.theme.clone());
        });
        let query = query_builder.build();

        let result = query.execute(&self.pool).await;
        match result {
            Ok(re) => println!("Result: {}", re.rows_affected()),
            Err(e) => {
                eprintln!("BATCH FAILED: {}", e);
            }
        }
        Some("0".to_string())
    }
}
