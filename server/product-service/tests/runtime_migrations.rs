#[tokio::test]
#[ignore = "requires a temporary Postgres database; run via tests/sql/run_sql_tests.sh"]
async fn runtime_migrations_apply_through_sqlx_migrator() {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must point at the temporary SQL migration test database");

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to the temporary SQL migration test database");

    sqlx::migrate!("./migrations/runtime")
        .run(&pool)
        .await
        .expect("runtime migrations must apply via sqlx::migrate!()");

    sqlx::migrate!("./migrations/runtime")
        .run(&pool)
        .await
        .expect("runtime migrations must be clean after first sqlx application");
}
