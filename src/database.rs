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

    let idle_timeout = env::var("IDLE_TIMEOUT")
        .expect("IDLE_TIMEOUT not found!")
        .parse::<u64>()
        .unwrap(); // I can unwrap_or() but I prefer variable, same for below

    let max_lifetime = env::var("MAX_LIFETIME")
        .expect("MAX_LIFETIME not found!")
        .parse::<u64>()
        .unwrap();

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(min_connections)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(idle_timeout))
        .max_lifetime(Duration::from_secs(max_lifetime))
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
            app_id VARCHAR(255) NOT NULL REFERENCES apps(unique_id) ON DELETE CASCADE,
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
            workspace_id INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
            unique_id VARCHAR(255) NOT NULL UNIQUE,
            title TEXT NOT NULL,
            field_type VARCHAR(100) NOT NULL,
            position INTEGER DEFAULT 0,
            is_system BOOLEAN DEFAULT false,
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
            workspace_id INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
            unique_id VARCHAR(255) NOT NULL UNIQUE,
            created_by INTEGER REFERENCES users(id),
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
            row_id VARCHAR(255) NOT NULL REFERENCES rows(unique_id) ON DELETE CASCADE,
            field_id VARCHAR(255) NOT NULL REFERENCES fields(unique_id) ON DELETE CASCADE,
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
        CREATE INDEX IF NOT EXISTS idx_workspaces_id
        ON workspaces(unique_id);
    ",
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        "
    CREATE INDEX IF NOT EXISTS idx_fields_workspace_id
    ON fields(workspace_id);
    ",
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        "
    CREATE INDEX IF NOT EXISTS idx_rows_workspace_id
    ON rows(workspace_id);
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
