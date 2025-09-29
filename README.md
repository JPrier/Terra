# Terra - US Manufacturing Directory & RFQ Platform

A complete MVP implementation of a US manufacturing directory and RFQ (Request for Quote) platform, built according to the comprehensive design document in `docs/mvp_design.md`.

## Architecture Overview

Terra follows a serverless, event-driven architecture optimized for ultra-fast performance and lowest cost:

- **Backend**: Rust Lambda functions with clean architecture (domain/application/infrastructure/presentation layers)
- **Storage**: S3-only data model with event sourcing for RFQs 
- **API**: AWS API Gateway HTTP API with rate limiting and CORS
- **Frontend**: Astro static site generator with server-rendered HTML and Svelte islands
- **Infrastructure**: AWS CDK for Infrastructure as Code
- **Distribution**: CloudFront CDN with optimized caching for HTML and JSON content

## Project Structure

```
/docs/                 # Design documentation
/infra/                # CDK infrastructure code  
/frontend/             # Astro static site generator
  /src/
    /components/       # ManufacturerCard.astro, FilterControls.svelte
    /layouts/          # BaseLayout.astro
    /pages/            # index.astro, rfq/submit.astro
    /styles/           # globals.css
  astro.config.mjs
/backend/
  /lambdas/            # Lambda function implementations
    api_rfqs/          # RFQ CRUD operations
    api_uploads/       # Presigned URL generation
    api_manufacturers/ # Admin manufacturer management
    image_ingest/      # Image processing pipeline
    publisher/         # Catalog rebuilding + HTML generation
  /crates/             # Shared Rust libraries
    domain/            # Business entities and rules
    application/       # Use cases and services  
    infrastructure/    # S3/SES implementations
    presentation/      # HTTP handlers
```

## Features Implemented

### Core MVP Features
- ✅ **Ultra-Fast Catalog Browsing**: Pre-rendered HTML pages for instant loading (sub-100ms)
- ✅ **Manufacturer Directory**: Browse verified US manufacturers by category/state
- ✅ **RFQ System**: Submit requests for quotes with real-time messaging
- ✅ **Event Sourcing**: Immutable RFQ history with message/status/attachment events
- ✅ **File Uploads**: Secure presigned URLs for project attachments
- ✅ **Email Notifications**: SES-powered notifications for RFQ activity
- ✅ **Admin Interface**: Manufacturer profile management
- ✅ **Publisher-Generated HTML**: Lambda generates static catalog pages for lightning speed

### Frontend Architecture - Astro + Islands
- ✅ **Static Site Generation**: Astro framework for minimal JavaScript footprint
- ✅ **Server-Rendered Catalog**: Publisher Lambda generates static HTML for categories and manufacturer profiles
- ✅ **Svelte Islands**: Client-side filtering components hydrated only where needed
- ✅ **Smart Caching**: Differential cache policies (5 min for HTML, 1 year for JSON)
- ✅ **SEO Optimized**: Server-side rendering for search engine visibility
- ✅ **Progressive Enhancement**: Core functionality works without JavaScript

### Technical Features  
- ✅ **Idempotency**: Duplicate request prevention with S3 markers
- ✅ **Rate Limiting**: API Gateway usage plans
- ✅ **CORS**: Proper cross-origin request handling
- ✅ **Request Validation**: JSON schema validation on all endpoints
- ✅ **Error Handling**: Consistent error envelope format
- ✅ **Multi-Layer Caching**: CloudFront CDN with optimized cache behaviors
- ✅ **Security**: Narrow IAM permissions and comprehensive input validation

## Quick Start

### Prerequisites
- AWS CLI configured with appropriate permissions
- Node.js 18+ for frontend and infrastructure
- Rust 1.70+ for backend development
- Docker (for Lambda builds)

### Backend Development

```bash
# Check backend compilation
cd backend
cargo check --workspace

# Run Lambda locally (example)
cd lambdas/api_rfqs
cargo lambda start
```

### Infrastructure Deployment

**Automated Deployment (Recommended):**
The repository includes automated CDK deployment via GitHub Actions that runs on every push to the main branch. See [CDK Deployment Guide](docs/CDK_DEPLOYMENT.md) for setup instructions.

**Manual Deployment:**
```bash
cd infra
npm install
npm run deploy
```

**Testing Infrastructure:**
```bash
cd infra
npm test          # Run CDK unit tests
npm run synth     # Validate CloudFormation synthesis
```

### Development Setup

**First-time setup:**
```bash
# Install git hooks for automatic code formatting
./scripts/setup-hooks.sh
```

This installs a pre-commit hook that automatically runs `cargo fmt --all` before each commit to ensure consistent Rust code formatting.

### Frontend Development  

```bash
cd frontend
npm install

# Build static site
npm run build

# Development server
npm run dev
```

### GitHub Pages Deployment

The repository is configured to automatically deploy the frontend to GitHub Pages when changes are pushed to the main branch. The deployment workflow:

1. **Automatic Deployment**: Pushes to `main` branch trigger GitHub Pages deployment
2. **Manual Deployment**: Can be triggered via GitHub Actions "Deploy to GitHub Pages" workflow
3. **Build Process**: Uses Astro static site generation with optimized assets
4. **Configuration**: Automatically adapts to GitHub Pages URL structure

To enable GitHub Pages deployment:
1. Go to repository Settings > Pages
2. Set Source to "GitHub Actions" 
3. The site will be available at `https://[username].github.io/Terra/`

### Content Generation

The Publisher Lambda generates both JSON data and static HTML pages:

```bash
# Trigger catalog rebuild (in production via API)
# Creates both JSON slices and pre-rendered HTML pages:
# - /catalog/machining/index.html
# - /catalog/machining/OH/index.html  
# - /catalog/manufacturer/mfg_001/index.html
```

## API Endpoints

Base URL: `/v1`

### RFQ Operations
- `POST /rfqs` - Create new RFQ
- `GET /rfqs/{id}` - Get RFQ metadata  
- `GET /rfqs/{id}/events` - List RFQ events (with polling support)
- `POST /rfqs/{id}/messages` - Post new message

### Upload Operations
- `POST /uploads/presign` - Generate presigned upload URL

### Admin Operations  
- `POST /manufacturers` - Create/update manufacturer (admin only)

### Public Catalog
- `GET /catalog/category/{category}.json` - Category listings (via S3/CloudFront)
- `GET /manufacturer/{id}.json` - Manufacturer profiles (via S3/CloudFront)

## Data Model

### S3 Storage Layout
```
# Public Bucket (app-public-{env})
catalog/category/{category}.json                    # Category manufacturer listings
catalog/category_state/{category}/{state}.json     # Category+state filtered listings  
manufacturer/{manufacturer_id}.json                # Public manufacturer profiles
tenants/{tenant_id}/manifests/{image_id}.json      # Image manifests for responsive loading
tenants/{tenant_id}/images/derived/{image_id}/     # Derived image variants (AVIF/WebP/JPEG)

# Private Bucket (app-private-{env})
tenants/{tenant_id}/images/raw/{uuid}.{ext}        # Raw uploaded images
rfq/{rfq_id}/meta.json                             # RFQ metadata
rfq/{rfq_id}/index.json                            # RFQ event index
rfq/{rfq_id}/events/{ts}-{uuid}.json               # Individual RFQ events
tmp/{uuid}                                         # Temporary files (auto-deleted)
idem/{sha256(key)}.json                           # Idempotency markers
```

### Event Sourcing
RFQs use immutable event streams:
- **Message Events**: Buyer/manufacturer text messages
- **Status Events**: RFQ lifecycle changes (created, viewed, replied, closed)  
- **Attachment Events**: File uploads with metadata

## Configuration

### Environment Variables
- `ENVIRONMENT`: Deployment environment (dev/staging/prod)
- `FROM_EMAIL`: SES sender email address
- `RUST_LOG`: Logging level for Lambda functions

### AWS Resources
- S3 buckets with lifecycle policies
- Lambda functions (ARM64, 256-512MB)
- API Gateway with usage plans
- CloudFront distribution with OAC
- SES for email notifications
- IAM roles with least privilege

## Design Principles

Following the MVP design document:

1. **Lowest Cost**: Every request optimized for minimal cost
2. **Low Latency**: <200ms p95 response times in us-east-1  
3. **Simple First**: Only essential features implemented
4. **Customer Experience**: Buyers/manufacturers before monetization
5. **Operational Minimalism**: Few moving parts, low ops burden
6. **Scalable Foundation**: Architecture supports growth without rewrites

## Security

- **Authentication**: Public catalog browsing, protected admin endpoints
- **Authorization**: Tenant-scoped data access with IAM enforcement
- **Validation**: JSON schema validation on all inputs
- **Rate Limiting**: API Gateway usage plans prevent abuse
- **Encryption**: S3 SSE-S3 for data at rest
- **CORS**: Restricted to approved origins

## Cost Optimization

Designed for micro-dollar per interaction costs:
- S3-only storage (no expensive databases)
- ARM64 Lambda functions for best price/performance
- CloudFront CDN reduces origin requests
- Efficient Rust implementation minimizes compute time
- Smart caching with immutable URLs

## Monitoring

Built-in observability:
- CloudWatch metrics and alarms for all services  
- Request IDs for tracing across services
- Structured logging with tenant/RFQ context
- SES bounce/complaint tracking

## Next Steps

For production deployment:
1. Set up custom domain and SSL certificates
2. Configure SES sending domain and verification
3. Build and deploy Rust Lambda binaries
4. Set up monitoring dashboards and alerting
5. Configure backup and disaster recovery
6. Implement comprehensive testing suite

## License

Apache 2.0 - See LICENSE file for details.