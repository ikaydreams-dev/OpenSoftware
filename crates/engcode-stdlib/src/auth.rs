use crate::database::{Database, DatabaseError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub token: String,
    pub user_id: String,
    pub created_at: u64,
    pub expires_at: u64,
}

pub struct AuthSystem {
    db: Box<Database>,
    sessions: HashMap<String, Session>,
    secret_key: String,
}

impl AuthSystem {
    pub fn new(db: Database, secret_key: String) -> Self {
        Self {
            db: Box::new(db),
            sessions: HashMap::new(),
            secret_key,
        }
    }

    pub fn database_name(&self) -> String {
        self.db.name().to_string()
    }

    pub fn secret_key_name(&self) -> String {
        self.secret_key.clone()
    }

    pub fn signup(&mut self, username: &str, email: &str, password: &str) -> Result<User, DatabaseError> {
        // Check if user already exists
        let check_query = format!(
            r#"{{"username": "{}"}}"#,
            username
        );

        let existing = crate::database::query_data(&self.db, "users", &check_query)?;
        if !existing.is_empty() {
            return Err(DatabaseError::QueryError("Username already exists".to_string()));
        }

        // Hash password (simple hash for now - in production use bcrypt)
        let password_hash = self.simple_hash(password);

        // Generate user ID
        let user_id = format!("user_{}", self.generate_id());

        // Get current timestamp
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create user
        let user = User {
            id: user_id.clone(),
            username: username.to_string(),
            email: email.to_string(),
            password_hash,
            created_at,
        };

        // Insert into database
        let user_json = serde_json::to_string(&user)
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        crate::database::insert_data(&mut *self.db, "users", &user_json)?;

        Ok(user)
    }

    pub fn login(&mut self, username: &str, password: &str) -> Result<Session, DatabaseError> {
        // Find user
        let check_query = format!(
            r#"{{"username": "{}"}}"#,
            username
        );

        let users = crate::database::query_data(&self.db, "users", &check_query)?;
        if users.is_empty() {
            return Err(DatabaseError::QueryError("Invalid username or password".to_string()));
        }

        let user: User = serde_json::from_str(&users[0])
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        // Verify password
        let password_hash = self.simple_hash(password);
        if user.password_hash != password_hash {
            return Err(DatabaseError::QueryError("Invalid username or password".to_string()));
        }

        // Create session
        let session = self.create_session(&user.id)?;

        Ok(session)
    }

    pub fn logout(&mut self, token: &str) -> Result<(), DatabaseError> {
        self.sessions.remove(token);
        Ok(())
    }

    pub fn verify_session(&self, token: &str) -> Result<Session, DatabaseError> {
        if let Some(session) = self.sessions.get(token) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            if session.expires_at > now {
                return Ok(session.clone());
            }
        }

        Err(DatabaseError::QueryError("Invalid or expired session".to_string()))
    }

    pub fn get_user_by_id(&self, user_id: &str) -> Result<User, DatabaseError> {
        let query = format!(r#"{{"id": "{}"}}"#, user_id);
        let users = crate::database::query_data(&self.db, "users", &query)?;

        if users.is_empty() {
            return Err(DatabaseError::QueryError("User not found".to_string()));
        }

        let user: User = serde_json::from_str(&users[0])
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(user)
    }

    pub fn update_password(&mut self, user_id: &str, new_password: &str) -> Result<(), DatabaseError> {
        let new_hash = self.simple_hash(new_password);

        let update_data = format!(r#"{{"password_hash": "{}"}}"#, new_hash);
        let condition = format!(r#"{{"id": "{}"}}"#, user_id);

        crate::database::update_data(&self.db, "users", &update_data, &condition)?;

        Ok(())
    }

    fn create_session(&mut self, user_id: &str) -> Result<Session, DatabaseError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let token = self.generate_token(user_id);
        let expires_at = now + (24 * 60 * 60); // 24 hours

        let session = Session {
            token: token.clone(),
            user_id: user_id.to_string(),
            created_at: now,
            expires_at,
        };

        self.sessions.insert(token.clone(), session.clone());

        Ok(session)
    }

    fn generate_token(&self, user_id: &str) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let data = format!("{}{}{}", user_id, now, self.secret_key);
        self.simple_hash(&data)
    }

    fn generate_id(&self) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        format!("{:x}", now)
    }

    fn simple_hash(&self, data: &str) -> String {
        // Simple hash for demonstration - in production use bcrypt or argon2
        let mut hash: u64 = 5381;
        for byte in data.bytes() {
            hash = ((hash << 5).wrapping_add(hash)).wrapping_add(byte as u64);
        }
        format!("{:x}", hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::create_database;

    #[test]
    fn test_signup_and_login() {
        let _ = std::fs::remove_file("databases/auth_test.db");
        let mut db = create_database("auth_test").unwrap();
        db.create_collection("users").unwrap();
        let mut auth = AuthSystem::new(db, "test_secret".to_string());

        // Signup
        let user = auth.signup("alice", "alice@example.com", "password123").unwrap();
        assert_eq!(user.username, "alice");
        assert_eq!(user.email, "alice@example.com");

        // Login
        let session = auth.login("alice", "password123").unwrap();
        assert_eq!(session.user_id, user.id);

        // Verify session
        let verified = auth.verify_session(&session.token).unwrap();
        assert_eq!(verified.user_id, user.id);
    }

    #[test]
    fn test_login_failure() {
        let _ = std::fs::remove_file("databases/auth_test2.db");
        let mut db = create_database("auth_test2").unwrap();
        db.create_collection("users").unwrap();
        let mut auth = AuthSystem::new(db, "test_secret".to_string());

        auth.signup("bob", "bob@example.com", "password123").unwrap();

        // Wrong password
        let result = auth.login("bob", "wrongpassword");
        assert!(result.is_err());

        // Wrong username
        let result = auth.login("nonexistent", "password123");
        assert!(result.is_err());
    }

    #[test]
    fn test_logout() {
        let _ = std::fs::remove_file("databases/auth_test3.db");
        let mut db = create_database("auth_test3").unwrap();
        db.create_collection("users").unwrap();
        let mut auth = AuthSystem::new(db, "test_secret".to_string());

        auth.signup("charlie", "charlie@example.com", "password123").unwrap();
        let session = auth.login("charlie", "password123").unwrap();

        // Verify session exists
        assert!(auth.verify_session(&session.token).is_ok());

        // Logout
        auth.logout(&session.token).unwrap();

        // Session should be invalid
        assert!(auth.verify_session(&session.token).is_err());
    }
}
