# Migrations
Run the following
```
sqlx db create --database-url sqlite:database.sqlite
sqlx migrate run --databse-url sqlite:database.sqlite
```

# Env
Create a .env file with the following content - `DATABASE_URL="sqlite:database.sqlite"`

# Run
Run with `cargo run`