//! `ampel-admin` — small operational CLI for recovering account access.
//!
//! Ampel hashes passwords with Argon2id, so a forgotten password can never be
//! read back out of the database. The only safe recovery path is to set a NEW
//! password, which this tool does. It is intended for local / docker-compose
//! setups where an operator with database access needs to get a locked-out user
//! signed in again.
//!
//! Usage:
//!   DATABASE_URL=postgres://... ampel-admin list-users
//!   DATABASE_URL=postgres://... ampel-admin reset-password --email user@example.com
//!   DATABASE_URL=postgres://... ampel-admin reset-password --id <uuid> --password 'new-secret'
//!
//! In docker compose, run it inside the API container, which already has
//! DATABASE_URL set, e.g.:
//!   docker compose exec api ampel-admin list-users
//!   docker compose exec api ampel-admin reset-password --email user@example.com

use ampel_core::services::AuthService;
use ampel_db::init_database;
use ampel_db::queries::UserQueries;
use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand};
use uuid::Uuid;

/// Minimum password length, kept in sync with `CreateUserRequest` validation.
const MIN_PASSWORD_LEN: usize = 8;

#[derive(Parser)]
#[command(
    name = "ampel-admin",
    about = "Ampel admin utility: list users and reset forgotten passwords.",
    long_about = "Passwords are stored as Argon2id hashes and cannot be recovered. \
This tool lists users and sets a NEW password for a locked-out account. \
Requires DATABASE_URL to point at the Ampel database."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List all users (id, email, display name, language, created-at).
    ListUsers,

    /// Set a new password for an existing user, identified by email or id.
    ResetPassword {
        /// Email address of the user to reset.
        #[arg(long, conflicts_with = "id")]
        email: Option<String>,

        /// User id (UUID) of the user to reset.
        #[arg(long, conflicts_with = "email")]
        id: Option<Uuid>,

        /// New password. If omitted, you are prompted interactively (input hidden).
        #[arg(long)]
        password: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Pick up DATABASE_URL from a local .env if present (mirrors the server).
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    let database_url = std::env::var("DATABASE_URL").map_err(|_| {
        anyhow!("DATABASE_URL must be set (e.g. postgres://ampel:ampel@localhost:5432/ampel)")
    })?;

    let db = init_database(&database_url)
        .await
        .context("failed to connect to the database")?;

    match cli.command {
        Command::ListUsers => list_users(&db).await,
        Command::ResetPassword {
            email,
            id,
            password,
        } => reset_password(&db, email, id, password).await,
    }
}

async fn list_users(db: &sea_orm::DatabaseConnection) -> Result<()> {
    let users = UserQueries::list(db)
        .await
        .context("failed to query users")?;

    if users.is_empty() {
        println!("No users found.");
        return Ok(());
    }

    println!(
        "{:<38} {:<32} {:<24} {:<6} CREATED",
        "ID", "EMAIL", "DISPLAY NAME", "LANG"
    );
    for user in &users {
        println!(
            "{:<38} {:<32} {:<24} {:<6} {}",
            user.id,
            user.email,
            user.display_name.as_deref().unwrap_or("-"),
            user.language.as_deref().unwrap_or("-"),
            user.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        );
    }
    println!("\n{} user(s).", users.len());
    Ok(())
}

async fn reset_password(
    db: &sea_orm::DatabaseConnection,
    email: Option<String>,
    id: Option<Uuid>,
    password: Option<String>,
) -> Result<()> {
    // Locate the target user.
    let user = match (email, id) {
        (Some(email), None) => UserQueries::find_by_email(db, &email)
            .await
            .context("failed to look up user by email")?
            .ok_or_else(|| anyhow!("no user found with email '{email}'"))?,
        (None, Some(id)) => UserQueries::find_by_id(db, id)
            .await
            .context("failed to look up user by id")?
            .ok_or_else(|| anyhow!("no user found with id '{id}'"))?,
        (None, None) => bail!("provide either --email or --id to identify the user"),
        (Some(_), Some(_)) => bail!("provide only one of --email or --id, not both"),
    };

    // Obtain the new password: from --password, or an interactive hidden prompt
    // with confirmation.
    let new_password = match password {
        Some(pw) => pw,
        None => {
            let pw = rpassword::prompt_password(format!(
                "New password for {} ({}): ",
                user.email, user.id
            ))
            .context("failed to read password")?;
            let confirm = rpassword::prompt_password("Confirm new password: ")
                .context("failed to read password")?;
            if pw != confirm {
                bail!("passwords did not match");
            }
            pw
        }
    };

    if new_password.len() < MIN_PASSWORD_LEN {
        bail!("password must be at least {MIN_PASSWORD_LEN} characters");
    }

    // The JWT settings are irrelevant for hashing; hash_password only uses Argon2.
    let auth = AuthService::new(String::from("unused-for-hashing"), 15, 7);
    let password_hash = auth
        .hash_password(&new_password)
        .map_err(|e| anyhow!("failed to hash password: {e}"))?;

    UserQueries::update_password(db, user.id, password_hash)
        .await
        .context("failed to update password")?;

    println!(
        "Password reset for {} ({}). They can now sign in with the new password.",
        user.email, user.id
    );
    Ok(())
}
