use std::env;

pub struct Config {
    pub public_bucket: String,
    pub private_bucket: String,
    pub environment: String,
    pub region: String,
}

impl Config {
    pub fn from_env() -> Self {
        let env = env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_string());

        Self {
            public_bucket: format!("app-public-{}", env),
            private_bucket: format!("app-private-{}", env),
            environment: env,
            region: env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
        }
    }
}
