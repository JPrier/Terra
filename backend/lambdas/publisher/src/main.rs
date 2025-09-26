use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{Value, json};
use serde::{Deserialize, Serialize};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use aws_sdk_s3::Client as S3Client;
use aws_config;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
struct Manufacturer {
    id: String,
    name: String,
    city: Option<String>,
    state: Option<String>,
    logo_url: Option<String>,
    categories: Vec<String>,
    capabilities: Option<Vec<String>>,
    description: Option<String>,
    contact_email: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct CategorySlice {
    category: String,
    state: Option<String>,
    manufacturers: Vec<Manufacturer>,
    last_updated: String,
}

struct HtmlTemplate;

impl HtmlTemplate {
    fn category_page(slice: &CategorySlice) -> String {
        let title = match &slice.state {
            Some(state) => format!("{} Manufacturers in {}", slice.category, state),
            None => format!("{} Manufacturers", slice.category),
        };
        
        let manufacturer_cards: String = slice.manufacturers.iter().map(|m| {
            format!(
                r#"<article class="card manufacturer-card">
                    {}
                    <div class="manufacturer-info">
                        <h3><a href="/catalog/manufacturer/{}/">{}</a></h3>
                        {}
                        <p class="categories">Categories: {}</p>
                        {}
                        <div class="actions">
                            <a href="/catalog/manufacturer/{}/" class="btn">View Details</a>
                            <a href="/rfq/submit?mfg={}" class="btn btn-primary">Submit RFQ</a>
                        </div>
                    </div>
                </article>"#,
                m.logo_url.as_ref().map(|url| 
                    format!(r#"<img src="{}" alt="{}" class="manufacturer-logo" loading="lazy" width="160" height="100">"#, url, m.name)
                ).unwrap_or_default(),
                m.id,
                m.name,
                m.city.as_ref().zip(m.state.as_ref()).map(|(city, state)| 
                    format!(r#"<p class="location">{}, {}</p>"#, city, state)
                ).unwrap_or_default(),
                m.categories.join(", "),
                m.capabilities.as_ref().map(|caps| 
                    format!(r#"<p class="capabilities">Capabilities: {}</p>"#, caps.join(", "))
                ).unwrap_or_default(),
                m.id,
                m.id
            )
        }).collect::<Vec<String>>().join("\n");

        format!(
            r#"<!doctype html>
<html lang="en">
<head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{}</title>
    <link rel="stylesheet" href="/src/styles/globals.css">
    <meta name="description" content="Find verified {} manufacturers{}" />
</head>
<body>
    <header class="header">
        <div class="header-content">
            <h1>Terra</h1>
            <p class="tagline">US Manufacturing Directory & RFQ Platform</p>
            <nav class="nav">
                <a href="/">Home</a>
                <a href="/catalog/machining/">Browse Manufacturers</a>
            </nav>
        </div>
    </header>
    
    <main class="main">
        <h2>{}</h2>
        <p>Find qualified {} manufacturers{} and submit RFQs directly.</p>
        
        <!-- Client-side filtering island -->
        <div id="filters" data-items='{}'>
            <div class="loading">Loading filters...</div>
        </div>
        
        <div class="grid" id="manufacturer-grid">
            {}
        </div>
        
        <script type="module">
            // Initialize Svelte filter component
            const filtersEl = document.getElementById('filters');
            const itemsData = JSON.parse(filtersEl.getAttribute('data-items'));
            
            // This would be dynamically loaded in a real implementation
            console.log('Manufacturers data:', itemsData);
        </script>
    </main>
    
    <footer style="text-align: center; padding: 2rem; color: #7f8c8d; border-top: 1px solid #eee; margin-top: 4rem;">
        <p>&copy; 2024 Terra Manufacturing Platform. Built for American manufacturing.</p>
    </footer>
</body>
</html>"#,
            title,
            slice.category,
            slice.state.as_ref().map(|s| format!(" in {}", s)).unwrap_or_default(),
            title,
            slice.category,
            slice.state.as_ref().map(|s| format!(" in {}", s)).unwrap_or_default(),
            serde_json::to_string(&slice.manufacturers).unwrap_or_default(),
            manufacturer_cards
        )
    }

    fn manufacturer_detail(manufacturer: &Manufacturer) -> String {
        format!(
            r#"<!doctype html>
<html lang="en">
<head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{} - Terra Manufacturing</title>
    <link rel="stylesheet" href="/src/styles/globals.css">
    <meta name="description" content="{}" />
</head>
<body>
    <header class="header">
        <div class="header-content">
            <h1>Terra</h1>
            <p class="tagline">US Manufacturing Directory & RFQ Platform</p>
            <nav class="nav">
                <a href="/">Home</a>
                <a href="/catalog/machining/">Browse Manufacturers</a>
            </nav>
        </div>
    </header>
    
    <main class="main">
        <div style="margin-bottom: 2rem;">
            <a href="/catalog/{}/" class="btn">← Back to {} Manufacturers</a>
        </div>
        
        <div class="card">
            <div style="display: flex; align-items: flex-start; gap: 2rem; margin-bottom: 2rem;">
                {}
                <div style="flex: 1;">
                    <h2>{}</h2>
                    {}
                    <p><strong>Categories:</strong> {}</p>
                    {}
                    {}
                </div>
            </div>
            
            {}
            
            <div style="text-align: center; margin-top: 3rem;">
                <a href="/rfq/submit?mfg={}" class="btn btn-primary btn-large">
                    Submit RFQ to {}
                </a>
            </div>
        </div>
    </main>
    
    <footer style="text-align: center; padding: 2rem; color: #7f8c8d; border-top: 1px solid #eee; margin-top: 4rem;">
        <p>&copy; 2024 Terra Manufacturing Platform. Built for American manufacturing.</p>
    </footer>
</body>
</html>"#,
            manufacturer.name,
            manufacturer.description.as_deref().unwrap_or(&format!("{} - US Manufacturing", manufacturer.name)),
            manufacturer.categories.first().unwrap_or(&"manufacturing".to_string()),
            manufacturer.categories.first().unwrap_or(&"Manufacturing".to_string()),
            manufacturer.logo_url.as_ref().map(|url| 
                format!(r#"<img src="{}" alt="{}" style="width: 200px; height: 150px; object-fit: contain; border-radius: 8px; background: #f8f9fa;">"#, url, manufacturer.name)
            ).unwrap_or_default(),
            manufacturer.name,
            manufacturer.city.as_ref().zip(manufacturer.state.as_ref()).map(|(city, state)| 
                format!(r#"<p><strong>Location:</strong> {}, {}</p>"#, city, state)
            ).unwrap_or_default(),
            manufacturer.categories.join(", "),
            manufacturer.capabilities.as_ref().map(|caps| 
                format!(r#"<p><strong>Capabilities:</strong> {}</p>"#, caps.join(", "))
            ).unwrap_or_default(),
            manufacturer.contact_email.as_ref().map(|email| 
                format!(r#"<p><strong>Contact:</strong> {}</p>"#, email)
            ).unwrap_or_default(),
            manufacturer.description.as_ref().map(|desc| 
                format!(r#"<div style="margin: 2rem 0;"><h3>About</h3><p>{}</p></div>"#, desc)
            ).unwrap_or_default(),
            manufacturer.id,
            manufacturer.name
        )
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    lambda_runtime::run(service_fn(function_handler)).await
}

async fn function_handler(event: LambdaEvent<Value>) -> Result<Value, lambda_runtime::Error> {
    let (event, _context) = event.into_parts();
    
    tracing::info!("Processing catalog rebuild event: {}", event);
    
    // Initialize AWS clients
    let config = aws_config::load_from_env().await;
    let s3_client = S3Client::new(&config);
    let bucket = std::env::var("PUBLIC_BUCKET").unwrap_or_else(|_| "app-public-dev".to_string());
    
    // Mock data for MVP demonstration
    let sample_manufacturers = vec![
        Manufacturer {
            id: "mfg_001".to_string(),
            name: "Precision Manufacturing Co.".to_string(),
            city: Some("Columbus".to_string()),
            state: Some("OH".to_string()),
            logo_url: None,
            categories: vec!["machining".to_string(), "prototyping".to_string()],
            capabilities: Some(vec!["cnc_milling".to_string(), "5_axis_machining".to_string()]),
            description: Some("Family-owned precision machining company specializing in aerospace and medical components. ISO 9001 certified with over 30 years of experience.".to_string()),
            contact_email: Some("quotes@precision-mfg.com".to_string()),
        },
        Manufacturer {
            id: "mfg_002".to_string(), 
            name: "Advanced Plastics Inc.".to_string(),
            city: Some("Austin".to_string()),
            state: Some("TX".to_string()),
            logo_url: None,
            categories: vec!["injection_molding".to_string(), "plastics".to_string()],
            capabilities: Some(vec!["injection_molding".to_string(), "overmolding".to_string()]),
            description: Some("Leading plastic injection molding company serving automotive and consumer electronics industries.".to_string()),
            contact_email: Some("info@advancedplastics.com".to_string()),
        },
        Manufacturer {
            id: "mfg_003".to_string(),
            name: "Metal Works LLC".to_string(),
            city: Some("Denver".to_string()),
            state: Some("CO".to_string()),
            logo_url: None,
            categories: vec!["sheet_metal".to_string(), "fabrication".to_string()],
            capabilities: Some(vec!["laser_cutting".to_string(), "welding".to_string(), "powder_coating".to_string()]),
            description: Some("Custom sheet metal fabrication and finishing services for industrial and architectural applications.".to_string()),
            contact_email: Some("sales@metalworks-co.com".to_string()),
        },
    ];

    let mut rebuilt_slices = Vec::new();
    let now = chrono::Utc::now().to_rfc3339();

    // Group manufacturers by category
    let mut category_groups: HashMap<String, Vec<Manufacturer>> = HashMap::new();
    for manufacturer in &sample_manufacturers {
        for category in &manufacturer.categories {
            category_groups.entry(category.clone()).or_default().push(manufacturer.clone());
        }
    }

    // Generate category slices and HTML pages
    for (category, manufacturers) in category_groups {
        // Generate JSON slice
        let slice = CategorySlice {
            category: category.clone(),
            state: None,
            manufacturers: manufacturers.clone(),
            last_updated: now.clone(),
        };
        
        let json_key = format!("catalog/category/{}.json", category);
        let html_key = format!("catalog/{}/index.html", category);
        
        // Upload JSON slice
        let json_body = serde_json::to_string(&slice).unwrap();
        s3_client.put_object()
            .bucket(&bucket)
            .key(&json_key)
            .body(json_body.into_bytes().into())
            .content_type("application/json")
            .cache_control("public, max-age=31536000, immutable")
            .send()
            .await
            .map_err(|e| lambda_runtime::Error::from(format!("Failed to upload JSON: {}", e)))?;

        // Upload HTML page
        let html_body = HtmlTemplate::category_page(&slice);
        s3_client.put_object()
            .bucket(&bucket)
            .key(&html_key)
            .body(html_body.into_bytes().into())
            .content_type("text/html")
            .cache_control("public, max-age=60")
            .send()
            .await
            .map_err(|e| lambda_runtime::Error::from(format!("Failed to upload HTML: {}", e)))?;

        rebuilt_slices.push(format!("{} (JSON + HTML)", category));

        // Generate state-specific slices
        let mut state_groups: HashMap<String, Vec<Manufacturer>> = HashMap::new();
        for manufacturer in &manufacturers {
            if let Some(state) = &manufacturer.state {
                state_groups.entry(state.clone()).or_default().push(manufacturer.clone());
            }
        }

        for (state, state_manufacturers) in state_groups {
            let state_slice = CategorySlice {
                category: category.clone(),
                state: Some(state.clone()),
                manufacturers: state_manufacturers,
                last_updated: now.clone(),
            };

            let state_json_key = format!("catalog/category_state/{}/{}.json", category, state);
            let state_html_key = format!("catalog/{}/{}/index.html", category, state);

            // Upload state JSON slice
            let state_json_body = serde_json::to_string(&state_slice).unwrap();
            s3_client.put_object()
                .bucket(&bucket)
                .key(&state_json_key)
                .body(state_json_body.into_bytes().into())
                .content_type("application/json")
                .cache_control("public, max-age=31536000, immutable")
                .send()
                .await
                .map_err(|e| lambda_runtime::Error::from(format!("Failed to upload state JSON: {}", e)))?;

            // Upload state HTML page
            let state_html_body = HtmlTemplate::category_page(&state_slice);
            s3_client.put_object()
                .bucket(&bucket)
                .key(&state_html_key)
                .body(state_html_body.into_bytes().into())
                .content_type("text/html")
                .cache_control("public, max-age=60")
                .send()
                .await
                .map_err(|e| lambda_runtime::Error::from(format!("Failed to upload state HTML: {}", e)))?;

            rebuilt_slices.push(format!("{}/{} (JSON + HTML)", category, state));
        }
    }

    // Generate individual manufacturer detail pages
    for manufacturer in &sample_manufacturers {
        let detail_key = format!("catalog/manufacturer/{}/index.html", manufacturer.id);
        let detail_json_key = format!("manufacturer/{}.json", manufacturer.id);
        
        // Upload manufacturer JSON
        let mfg_json = serde_json::to_string(&manufacturer).unwrap();
        s3_client.put_object()
            .bucket(&bucket)
            .key(&detail_json_key)
            .body(mfg_json.into_bytes().into())
            .content_type("application/json")
            .cache_control("public, max-age=31536000, immutable")
            .send()
            .await
            .map_err(|e| lambda_runtime::Error::from(format!("Failed to upload manufacturer JSON: {}", e)))?;

        // Upload manufacturer HTML
        let detail_html = HtmlTemplate::manufacturer_detail(&manufacturer);
        s3_client.put_object()
            .bucket(&bucket)
            .key(&detail_key)
            .body(detail_html.into_bytes().into())
            .content_type("text/html")
            .cache_control("public, max-age=60")
            .send()
            .await
            .map_err(|e| lambda_runtime::Error::from(format!("Failed to upload manufacturer HTML: {}", e)))?;

        rebuilt_slices.push(format!("manufacturer/{} (JSON + HTML)", manufacturer.id));
    }
    
    tracing::info!("Catalog rebuild completed. Rebuilt: {:?}", rebuilt_slices);
    
    Ok(json!({
        "message": "Catalog rebuild completed",
        "rebuilt_slices": rebuilt_slices,
        "timestamp": now
    }))
}