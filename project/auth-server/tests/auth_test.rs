use auth_server::db::AppDb;
use auth_server::models::*;

#[test]
fn test_create_and_get_user() {
    let db = AppDb::new(":memory:").unwrap();

    let user = db
        .create_user(&CreateUserRequest {
            email: "test@example.com".to_string(),
            password: Some("hash123".to_string()),
            name: "Test User".to_string(),
            provider: "email".to_string(),
            provider_id: "test@example.com".to_string(),
            avatar_url: None,
            role: None,
        })
        .unwrap();

    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.name, "Test User");
    assert_eq!(user.provider, "email");

    let found = db.get_user_by_email("test@example.com").unwrap().unwrap();
    assert_eq!(found.0.id, user.id);
    assert_eq!(found.1, Some("hash123".to_string()));
}

#[test]
fn test_create_duplicate_email_fails() {
    let db = AppDb::new(":memory:").unwrap();
    db.create_user(&CreateUserRequest {
        email: "dup@example.com".to_string(),
        password: Some("hash".to_string()),
        name: "Dup".to_string(),
        provider: "email".to_string(),
        provider_id: "dup@example.com".to_string(),
        avatar_url: None,
        role: None,
    })
    .unwrap();

    let result = db.create_user(&CreateUserRequest {
        email: "dup@example.com".to_string(),
        password: Some("hash2".to_string()),
        name: "Dup2".to_string(),
        provider: "email".to_string(),
        provider_id: "dup@example.com".to_string(),
        avatar_url: None,
        role: None,
    });
    assert!(result.is_err());
}

#[test]
fn test_get_nonexistent_user() {
    let db = AppDb::new(":memory:").unwrap();
    let result = db.get_user_by_email("noone@example.com").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_refresh_token_flow() {
    let db = AppDb::new(":memory:").unwrap();
    let user = db
        .create_user(&CreateUserRequest {
            email: "token@test.com".to_string(),
            password: None,
            name: "Token Test".to_string(),
            provider: "google".to_string(),
            provider_id: "google-123".to_string(),
            avatar_url: None,
            role: None,
        })
        .unwrap();

    db.store_refresh_token(&user.id, "refresh-token-123")
        .unwrap();
    let result = db.validate_refresh_token("refresh-token-123").unwrap();
    assert_eq!(result, Some(user.id));

    db.delete_refresh_token("refresh-token-123").unwrap();
    let result = db.validate_refresh_token("refresh-token-123").unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_get_user_by_provider() {
    let db = AppDb::new(":memory:").unwrap();
    db.create_user(&CreateUserRequest {
        email: "google@test.com".to_string(),
        password: None,
        name: "Google User".to_string(),
        provider: "google".to_string(),
        provider_id: "google-sub-123".to_string(),
        avatar_url: Some("https://pic.com/avatar.png".to_string()),
        role: None,
    }).unwrap();

    let found = db.get_user_by_provider("google", "google-sub-123").unwrap();
    assert!(found.is_some());
    let u = found.unwrap();
    assert_eq!(u.email, "google@test.com");
    assert_eq!(u.role, "editor");

    let not_found = db.get_user_by_provider("google", "nonexistent").unwrap();
    assert!(not_found.is_none());
}

#[test]
fn test_expired_refresh_token_is_cleaned_up() {
    let db = AppDb::new(":memory:").unwrap();
    let user = db
        .create_user(&CreateUserRequest {
            email: "expired@test.com".to_string(),
            password: None,
            name: "Expired".to_string(),
            provider: "email".to_string(),
            provider_id: "expired@test.com".to_string(),
            avatar_url: None,
            role: None,
        })
        .unwrap();

    // Insert an already-expired token directly
    {
        use rusqlite::params;
        let conn = db.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO refresh_tokens (id, user_id, token, expires_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                "expired-id",
                user.id,
                "expired-token-999",
                "2020-01-01T00:00:00+00:00",  // past date
                "2020-01-01T00:00:00+00:00",
            ],
        ).unwrap();
    }

    // validate should return None AND delete the expired row
    let result = db.validate_refresh_token("expired-token-999").unwrap();
    assert_eq!(result, None, "expired token should not validate");

    // confirm the row was deleted
    let result = db.validate_refresh_token("expired-token-999").unwrap();
    assert_eq!(result, None, "expired token should be deleted after validation");
}


