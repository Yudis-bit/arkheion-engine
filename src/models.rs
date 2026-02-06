use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};
use sqlx::{Pool, Sqlite};
use crate::engine::EngineMetrics;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub public_key: String,      
    pub api_key: String,         
    pub tier: String,           
    pub credits: i64,            
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub wallet: String,
    pub tier: String,
    pub usage_limit: String,
    pub status: String,
}

#[derive(Debug, FromRow)]
pub struct PaymentTx {
    pub id: i64,
    pub user_id: i64,
    pub signature: String,       
    pub amount_sol: f64,
    pub status: String,          
    pub created_at: DateTime<Utc>,
}
#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Sqlite>,
    pub metrics: Arc<Mutex<EngineMetrics>>,
}
