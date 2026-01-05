use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Movie {
    pub id: i32,
    pub title: String,
    pub tagline: Option<String>,
    pub vote_average: Option<f64>,
    pub vote_count: Option<i32>,
    pub popularity: Option<f64>,
    pub release_date: Option<String>,
}