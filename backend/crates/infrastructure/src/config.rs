use std::env;

pub struct Config {
    pub public_bucket: String,
    pub private_bucket: String,
    pub environment: String,
    pub region: String,
    pub aws_endpoint_url: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        let env = env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_string());

        Self {
            public_bucket: env::var("PUBLIC_BUCKET")
                .unwrap_or_else(|_| format!("app-public-{}", env)),
            private_bucket: env::var("PRIVATE_BUCKET")
                .unwrap_or_else(|_| format!("app-private-{}", env)),
            environment: env,
            region: env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            aws_endpoint_url: env::var("AWS_ENDPOINT_URL").ok(),
        }
    }

    /// Create AWS config with optional endpoint override for LocalStack
    pub async fn create_aws_config(&self) -> aws_config::SdkConfig {
        use aws_config::BehaviorVersion;

        let mut config_loader = aws_config::defaults(BehaviorVersion::latest());

        // Override endpoint for LocalStack if specified
        if let Some(endpoint_url) = &self.aws_endpoint_url {
            config_loader = config_loader.endpoint_url(endpoint_url.clone());
        }

        config_loader.load().await
    }
}
