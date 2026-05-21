use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::{env, time::Duration};

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub record_limit: usize,
}

pub async fn setup_database() -> AppState {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not found!");
    let max_connections = env::var("MAX_CONNECTIONS")
        .expect("MAX_CONNECTIONS not found!")
        .parse::<u32>()
        .unwrap();
    let min_connections = env::var("MIN_CONNECTIONS")
        .expect("MIN_CONNECTIONS not found!")
        .parse::<u32>()
        .unwrap();

    let records_per_page = env::var("RECORDS_PER_PAGE")
        .expect("RECORDS_PER_PAGE not found!")
        .parse::<usize>()
        .unwrap_or(100);

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(min_connections)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .unwrap();

    let mut tx = pool.begin().await.unwrap();
    sqlx::query(
        "
        CREATE TABLE IF NOT EXISTS users(
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            username VARCHAR(255) NOT NULL UNIQUE,
            email VARCHAR(255) NOT NULL UNIQUE,
            password VARCHAR(255) NOT NULL,
            is_admin BOOLEAN DEFAULT false,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        ",
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        " CREATE TABLE IF NOT EXISTS apps(
            id SERIAL PRIMARY KEY,
            owner_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            unique_id VARCHAR(255) NOT NULL UNIQUE,
            title TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        " CREATE TABLE IF NOT EXISTS members(
            id SERIAL PRIMARY KEY,
            member_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            app_id VARCHAR(255) NOT NULL REFERENCES apps(unique_id) ON DELETE CASCADE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        " CREATE TABLE IF NOT EXISTS tokens(
            id SERIAL PRIMARY KEY,
            owner_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            app_id VARCHAR(255) NOT NULL REFERENCES apps(unique_id) ON DELETE CASCADE,
            unique_id VARCHAR(255) NOT NULL UNIQUE,
            token TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        "
        CREATE TABLE IF NOT EXISTS workspaces(
            id SERIAL PRIMARY KEY,
            app_id INTEGER NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
            unique_id VARCHAR(255) NOT NULL UNIQUE,
            title TEXT NOT NULL,
            position INTEGER DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
    ",
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        "
        CREATE TABLE IF NOT EXISTS tables(
            id BIGSERIAL PRIMARY KEY,
            workspace_id INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
            unique_id VARCHAR(255) NOT NULL UNIQUE,
            title TEXT NOT NULL,
            position INTEGER DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
    ",
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        "
        CREATE TABLE IF NOT EXISTS fields(
            id BIGSERIAL PRIMARY KEY,
            table_id INTEGER NOT NULL REFERENCES tables(id) ON DELETE CASCADE,
            unique_id VARCHAR(255) NOT NULL UNIQUE,
            title TEXT NOT NULL,
            field_type VARCHAR(100) NOT NULL,
            position INTEGER DEFAULT 0,
            settings JSONB,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
    ",
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        "
        CREATE TABLE IF NOT EXISTS rows(
            id BIGSERIAL PRIMARY KEY,
            table_id INTEGER NOT NULL REFERENCES tables(id) ON DELETE CASCADE,
            unique_id VARCHAR(255) NOT NULL UNIQUE,
            created_by INTEGER REFERENCES users(id),
            position BIGINT DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
    ",
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        "
        CREATE TABLE IF NOT EXISTS cells(
            id BIGSERIAL PRIMARY KEY,
            row_id BIGINT NOT NULL REFERENCES rows(id) ON DELETE CASCADE,
            field_id INTEGER NOT NULL REFERENCES fields(id) ON DELETE CASCADE,
            value_text TEXT,
            value_number DOUBLE PRECISION,
            value_boolean BOOLEAN,
            value_date TIMESTAMP,
            value_json JSONB,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(row_id, field_id)
        );
    ",
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        "
        CREATE UNIQUE INDEX IF NOT EXISTS idx_tokens_id ON tokens(unique_id);
    ",
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        "
    CREATE INDEX IF NOT EXISTS idx_apps_owner_id
    ON apps(owner_id);
    ",
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        "
        CREATE INDEX IF NOT EXISTS idx_member_app_id
        ON members(app_id);
    ",
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        "
        CREATE INDEX IF NOT EXISTS idx_member_user_id
        ON members(member_id);
    ",
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        "
        CREATE INDEX IF NOT EXISTS idx_workspaces_app_id
        ON workspaces(app_id);
    ",
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        "
        CREATE INDEX IF NOT EXISTS idx_tables_workspace_id
        ON tables(workspace_id);
    ",
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        "
        CREATE INDEX IF NOT EXISTS idx_fields_table_id
        ON fields(table_id);
    ",
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        "
        CREATE INDEX IF NOT EXISTS idx_rows_table_id
        ON rows(table_id);
    ",
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        "
        CREATE INDEX IF NOT EXISTS idx_cells_row_id
        ON cells(row_id);
    ",
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        "
        CREATE INDEX IF NOT EXISTS idx_cells_field_id
        ON cells(field_id);
    ",
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        "
        CREATE INDEX IF NOT EXISTS idx_cells_field_text
        ON cells(field_id, value_text);
    ",
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        "
        CREATE INDEX IF NOT EXISTS idx_cells_field_number
        ON cells(field_id, value_number);
    ",
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        "
        CREATE INDEX IF NOT EXISTS idx_cells_field_date
        ON cells(field_id, value_date);
    ",
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    tx.commit().await.unwrap();

    AppState {
        pool: pool,
        record_limit: records_per_page,
    }
}
