use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub supabase_url: String,
    pub supabase_anon_key: String,
    pub supabase_jwt_secret: String,
    pub tavily_api_key: String,
    pub exa_api_key: String,
    pub firecrawl_api_key: String,
    pub port: u16,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            supabase_url: env::var("SUPABASE_URL").expect("SUPABASE_URL required"),
            supabase_anon_key: env::var("SUPABASE_ANON_KEY").expect("SUPABASE_ANON_KEY required"),
            supabase_jwt_secret: env::var("SUPABASE_JWT_SECRET")
                .expect("SUPABASE_JWT_SECRET required"),
            tavily_api_key: env::var("TAVILY_API_KEY").expect("TAVILY_API_KEY required"),
            exa_api_key: env::var("EXA_API_KEY").expect("EXA_API_KEY required"),
            firecrawl_api_key: env::var("FIRECRAWL_API_KEY").expect("FIRECRAWL_API_KEY required"),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("PORT must be a number"),
        }
    }
}

#[cfg(test)]
impl AppConfig {
    pub fn test_config() -> Self {
        Self {
            supabase_url: "http://localhost:54321".to_string(),
            supabase_anon_key: "test-anon-key".to_string(),
            supabase_jwt_secret: "test-jwt-secret-at-least-32-chars-long".to_string(),
            tavily_api_key: "test-tavily-key".to_string(),
            exa_api_key: "test-exa-key".to_string(),
            firecrawl_api_key: "test-firecrawl-key".to_string(),
            port: 0,
        }
    }
}
