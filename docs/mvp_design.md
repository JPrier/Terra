# US Manufacturing Directory & RFQ Platform — Design Document (MVP → V1)

## 1) Introduction

### 1.0) Purpose

Deliver a minimal viable product that connects buyers with US manufacturers. Optimize for **lowest possible cost per interaction**, **fast UX**, and **straightforward operations**.

## 1.1) Guiding Principles

* **Lowest cost:** every request and stored byte should be as cheap as possible.
* **Low latency:** common operations feel snappy (<200ms p95 in us-east-1).
* **Simple first:** only build what’s necessary now; add later as adoption proves it.
* **Customer experience first:** usefulness for buyers/manufacturers before monetization.
* **Operational minimalism:** few moving parts; low ops burden.
* **Scalable foundation:** MVP choices should evolve without rewrites.

## 1.2) Glossary

* **MVP:** Minimum Viable Product — smallest feature set to validate value.
* **RFQ:** Request for Quote — buyer sends specs to a manufacturer for pricing/lead time.
* **Tenant:** logical grouping (here, manufacturers) used to scope data and permissions.
* **SPA:** Single Page Application — frontend in browser, calls APIs.
* **S3:** Simple Storage Service — object storage for JSON and files.
* **CDN / CloudFront:** globally caches static content and public assets.
* **SES:** Simple Email Service.
* **CDK:** Cloud Development Kit (IaC).
* **Lambda:** AWS serverless function runtime.
* **Image Manifest:** JSON that lists available image variants for responsive loading.
* **Immutable URL:** filename includes content hash so it never changes, enabling long cache lifetimes.

---

## 2) High-Level Architecture

```
Buyers/Manufacturers
        │
        ▼
+-------------------+       +-----------------------+
|   GitHub Pages    |  ---> |   CloudFront (CDN)   |
|   Static SPA      |       |   (fronts Pages+S3)  |
+---------+---------+       +-----------+-----------+
          | HTTPS                         | HTTPS
          v                               v
    +-----------+                  +------------------+
    |  S3       |  (public)        | API Gateway      |
    |  derived  |  catalog/images  | HTTP API         |
    +-----------+                  +--------+---------+
                                            |
                                            v
                                       +---------+
                                       | Lambda  |
                                       | (Rust)  |
                                       +----+----+
                                            |
                                            v
                               +--------------------------+
                               | S3 (private)             |
                               | raw uploads, RFQs/events |
                               +--------------------------+
                                            |
                                            v
                                         +------+
                                         | SES  |
                                         | mail |
                                         +------+
```

**Notes**

* **Reads S3/CDN-first:** catalog slices and image variants are served directly from S3 via CloudFront.
* **Writes are append-only:** RFQs & messages are event files in S3; avoid server-side joins on hot path.
* **“Realtime” via polling:** `?since` parameter on events; upgrade later if needed.

---

## 3) Requirements

### 3.1 User Types

* **Buyer:** discovers manufacturers, submits RFQs, reads/sends messages.
* **Manufacturer:** manages profile & offerings, responds to RFQs.
* **Admin (Operator):** moderates listings, regenerates catalog slices.

### 3.2 UI Pages

**Public (no login)**

* Landing
* Browse manufacturers by category/state
* Manufacturer profile
* Submit RFQ form

**Buyer (light auth or magic-link)**

* RFQ detail (events/messages)

**Manufacturer**

* Profile management (info, images, offerings)
* RFQ inbox (list & detail)

**Admin**

* Approvals dashboard (signup moderation, catalog rebuilds)

### 3.3 Information to Store

* **Tenants/Manufacturers:** name, logo, location, capabilities, categories, contact.
* **Offerings:** title, materials, lead time, media refs.
* **RFQs:** buyer contact, manufacturer id, subject/body, status, timestamps.
* **Events:** rfq id, timestamp, type (`message|status|attachment`), author, payload.
* **Images:** manifest describing AVIF/WebP/JPEG variants.
* **Catalog slices:** JSON groupings by category, state, or both.

### 3.4 Uploads, Edits, Deletions

* **Buyer:** create RFQs, send messages, upload optional attachments; cannot delete RFQs (immutable record).
* **Manufacturer:** edit profile, add/update/delete offerings & images.
* **Admin:** edit/delete any manufacturer data or RFQs (moderation).

---

## 4) Infrastructure

* **Frontend:** Static SPA on **GitHub Pages**.
* **CDN:** **CloudFront** (fronts GitHub Pages + public S3). Use if it remains very cheap; otherwise Pages-only at day 1 is acceptable.
* **Backend:** **API Gateway (HTTP API)** → **AWS Lambda (Rust)** for APIs and background tasks.
* **Storage:** **S3-only** (no database in MVP).

  * **Public bucket:** derived images, image manifests, catalog JSON.
  * **Private bucket:** raw uploads, RFQ meta, RFQ events, small indexes.
* **Email:** **SES** notifications.
* **IaC:** **AWS CDK**.

Security & ops basics:

* Narrow IAM per Lambda; presigned uploads constrained to tenant prefixes and content-type/size.
* CORS: restrict to Pages domain.
* Long TTLs and immutable URLs for public assets.
* CloudWatch metrics/alarms for API 5xx, Lambda errors, SES bounce rate, budget alerts.

---

## 5) Data Model & S3 Layout

### 5.1 Entities

* **Tenant**: `id`, `name`, `plan`, `created_at`
* **Manufacturer**: `id`, `tenant_id`, `name`, `location`, `capabilities[]`, `categories[]`, `contact_email`, `media[]`
* **Offering**: `id`, `manufacturer_id`, `title`, `materials[]`, `lead_time_days[min,max]`, `media[]`
* **RFQ**: `id`, `tenant_id`, `buyer_contact`, `participants[]`, `status`, `last_event_ts`, `attachments[]`
* **Event**: `rfq_id`, `ts`, `type`, `by`, `body/meta`

### 5.2 S3 (concrete)

```
app-public/
  catalog/category/machining.json
  catalog/category_state/machining/OH.json
  manufacturer/mfg123.json
  tenants/t1/images/derived/img_ab12/w-320.avif
  tenants/t1/images/derived/img_ab12/w-320.webp
  tenants/t1/images/derived/img_ab12/w-320.jpg
  tenants/t1/manifests/img_ab12.json

app-private/
  tenants/t1/images/raw/uuid.jpg
  rfq/r_9Kc8/meta.json
  rfq/r_9Kc8/index.json                 # { last_event_ts, count }
  rfq/r_9Kc8/events/2025-09-25T12-00-01Z-uuid.json
  # optional compactions later: events-00001.jsonl
```

Lifecycle: move raw uploads to IA after ~30–60 days; auto-delete temp prefixes after 1 day.

---

## 6) APIs (API Gateway → Lambda)

**Base:** `/v1`

* `GET /catalog/...` → served from S3/CloudFront directly (no Lambda on hot path)
* `POST /manufacturers` (admin) → write source JSON, trigger catalog rebuild
* `POST /uploads/presign` → presigned URL for raw image/doc upload (tenant-scoped)
* `POST /rfqs` → create RFQ (meta + initial event), send SES notifications
* `GET /rfqs/{id}` → fetch RFQ meta (and `last_event_ts`)
* `GET /rfqs/{id}/events?since=ts` → list new events since timestamp
* `POST /rfqs/{id}/messages` → append message event; optional SES digest

Headers & caching: support `ETag`/`If-None-Match` for catalog JSON.

---

## 7) Background Processing

* **Image Ingest Lambda** (triggered on raw upload in private bucket)

  * Validate MIME/size; strip EXIF
  * Derive **AVIF/WebP/JPEG** at widths **320/640/1024/1600**
  * Write **manifest JSON** under public bucket
  * Set `Cache-Control: public, max-age=31536000, immutable` on derived assets
* **Publisher Lambda** (triggered by manufacturer/offer changes)

  * Compute affected slices (category/state/category_state)
  * Write compact slice arrays and optional per-slice search index

---

## 8) Frontend Notes

* SPA loads catalog slices from S3/CloudFront.
* Client-side filtering and typeahead using prebuilt slice JSON (MiniSearch/Lunr JSON per slice if helpful).
* Images via `<picture>` + `srcset/sizes`, `loading="lazy"`, IntersectionObserver; optional small SW cache (LRU ~200 items).

---

## 9) Project Layout (Repos)

```
/infra/             # CDK app (buckets, API, Lambdas, CloudFront, SES)
/frontend/          # SPA
/backend/
  /lambdas/
    api_manufacturers/
    api_rfqs/
    api_uploads/
    publisher/
    image_ingest/
  /crates/
    domain/         # entities & rules (no AWS code)
    application/    # use-cases (services)
    infrastructure/ # S3/SES clients & adapters
    presentation/   # Lambda HTTP handlers
```

Rust patterns: `serde`/`serde_json`, `aws-sdk-s3`, `aws-sdk-sesv2`, `tracing`. Small, idempotent POSTs via `Idempotency-Key` stored as tiny S3 markers.

---

## 10) Cost Posture (MVP ballpark)

* S3 GETs ≈ micro-dollars each; PUT/LIST more expensive but still tiny at MVP scale.
* API Gateway (HTTP) and Lambda (Rust, arm64, low memory) are pennies per million.
* SES ≈ $0.10 / 1k emails.
* With S3-only and CDN-cached reads, target **≪ $0.00001 per interaction**.

---

## 11) Security & Compliance

* **AuthN:** browsing is public; admin endpoints protected via simple key/JWT initially.
* **AuthZ:** tenant prefix enforced in presign; server-side checks on RFQ ownership.
* **PII minimization:** store minimal buyer info; remove EXIF; S3 SSE-S3 for rest encryption.
* **Observability:** CloudWatch metrics/alarms; sampled logs with `request_id`, `tenant_id`, `rfq_id`.

---

# Definitive Model Design

## A) S3 Object Shapes

### A.1 Buckets

* **Public bucket**: `app-public-<env>`

  * Purpose: catalog JSON, image manifests, derived image variants
  * Access: CloudFront OAC only
  * Headers:

    * `Cache-Control: public, max-age=31536000, immutable` (derived, manifests, catalog)
    * `Content-Type` according to object (e.g., `application/json`, `image/avif`)
* **Private bucket**: `app-private-<env>`

  * Purpose: raw uploads, RFQ meta, RFQ events, indexes
  * Access: Lambdas via IAM; uploads via presigned URLs
  * Headers:

    * `Cache-Control: private, max-age=0, no-store` (for RFQ JSON)
    * `x-amz-meta-tenant: <tenant_id>` (for uploads)

### A.2 Key Conventions

```
# PUBLIC
app-public-<env>/
  catalog/category/{category}.json
  catalog/category_state/{category}/{state}.json
  manufacturer/{manufacturer_id}.json
  tenants/{tenant_id}/manifests/{image_id}.json
  tenants/{tenant_id}/images/derived/{image_id}/w-{width}.{avif|webp|jpg}

# PRIVATE
app-private-<env>/
  tenants/{tenant_id}/images/raw/{uuid}.{ext}
  rfq/{rfq_id}/meta.json
  rfq/{rfq_id}/index.json
  rfq/{rfq_id}/events/{ts}-{uuid}.json     # ts = ISO8601 with : -> -
  tmp/{uuid}                                # auto-deleted in 24h (lifecycle)
```

### A.3 JSON Schemas (TypeScript-style for readability)

> These are **storage shapes**—API DTOs mirror these but may redact internal fields (e.g., internal notes).

#### A.3.1 `catalog/category/{category}.json`

```ts
type CatalogManufacturerSummary = {
  id: string;
  name: string;
  city?: string;
  state?: string;            // "OH", "CA"...
  categories: string[];      // normalized slugs
  capabilities?: string[];   // optional facets
  logo?: string;             // public URL via CloudFront
};

type CategorySlice = {
  category: string;          // slug
  generated_at: string;      // ISO8601
  items: CatalogManufacturerSummary[];
};
```

#### A.3.2 `manufacturer/{manufacturer_id}.json`

```ts
type ManufacturerProfile = {
  id: string;
  tenant_id: string;
  name: string;
  description?: string;
  location?: { city?: string; state?: string; country?: string; lat?: number; lng?: number };
  categories: string[];
  capabilities?: string[];
  contact_email?: string;    // public if manufacturer opts-in
  media?: { image_manifest_id: string; alt?: string }[];
  offerings?: {
    id: string;
    title: string;
    materials?: string[];
    lead_time_days?: { min?: number; max?: number };
    media?: { image_manifest_id: string; alt?: string }[];
  }[];
  updated_at: string;        // ISO8601
};
```

#### A.3.3 Image manifest `tenants/{tenant_id}/manifests/{image_id}.json`

```ts
type ImageManifest = {
  id: string;                // content-hash or stable id
  w: number;                 // original width
  h: number;                 // original height
  variants: Array<{
    w: number;
    t: "image/avif" | "image/webp" | "image/jpeg";
    k: string;               // S3 key in public bucket
  }>;
  lqip?: string;             // data URI (optional)
  created_at: string;        // ISO8601
};
```

#### A.3.4 RFQ meta `rfq/{rfq_id}/meta.json`

```ts
type RfqMeta = {
  id: string;
  tenant_id: string;         // manufacturer tenant receiving the RFQ
  manufacturer_id: string;
  buyer: { email: string; name?: string };
  subject: string;
  status: "open" | "archived" | "closed";
  created_at: string;        // ISO8601
  last_event_ts: string;     // ISO8601
  participants: Array<{ role: "buyer" | "manufacturer"; email: string; name?: string }>;
  attachments?: AttachmentRef[]; // convenience cache of first message attachments
};

type AttachmentRef = {
  id: string;                // event_id or logical id
  file_name: string;
  content_type: string;
  size_bytes: number;
  key: string;               // S3 private key
};
```

#### A.3.5 RFQ index `rfq/{rfq_id}/index.json`

```ts
type RfqIndex = { last_event_ts: string; count: number };
```

#### A.3.6 RFQ event `rfq/{rfq_id}/events/{ts}-{uuid}.json`

```ts
type RfqEventBase = {
  id: string;                // UUID of the event file
  rfq_id: string;
  ts: string;                // ISO8601
  by: "buyer" | "manufacturer" | "system";
  type: "message" | "status" | "attachment";
};

type MessageEvent = RfqEventBase & {
  type: "message";
  body: string;              // markdown-safe plain text
};

type StatusEvent = RfqEventBase & {
  type: "status";
  status:
    | "rfq_created"
    | "vendor_viewed"
    | "vendor_replied"
    | "buyer_viewed"
    | "closed"
    | "archived";
  note?: string;
};

type AttachmentEvent = RfqEventBase & {
  type: "attachment";
  attachments: AttachmentRef[];
};

type RfqEvent = MessageEvent | StatusEvent | AttachmentEvent;
```

---

## B) Event & Message Model (Validation Rules)

* **Ordering:** events are strictly ordered by `ts`; consumers should **sort by (ts, id)** to be stable.
* **Immutability:** events are append-only. If a mistake occurs, append a correcting event (no deletes).
* **Size limits:**

  * Message body ≤ **8,000 chars**
  * Attachments per event ≤ **10**
  * Attachment size ≤ **15 MB** (enforced at presign)
* **Content types:** allow common images (`image/jpeg`, `image/png`, `image/webp`, `image/avif`) and PDFs; others rejected at presign.
* **HTML/Markdown:** store text as plain UTF-8; rendering layer may treat as basic Markdown; **no HTML allowed in storage**.
* **Privacy:** PII kept to minimum; **never** store IPs/user agents in event bodies.

---

## C) API Definitions

**Base URL:** `/v1` (API Gateway HTTP API)
**Auth:** Public GETs for catalog; admin POSTs protected (Bearer); RFQ create/messages are public for MVP with rate limits + CAPTCHA (optional).
**Common Headers**

* `Idempotency-Key` (POSTs) → de-duplicate within 24h
* `X-Request-Id` (response)
* `ETag`/`If-None-Match` (catalog JSON)

### C.1 `POST /rfqs`

Create RFQ (meta + initial events).

**Request**

```json
{
  "tenant_id": "t1",
  "manufacturer_id": "mfg_123",
  "buyer": { "email": "a@b.com", "name": "Alice" },
  "subject": "CNC milling prototype",
  "body": "2 parts, 6061, 7-day turn",
  "attachments": [
    { "upload_key": "tenants/t1/images/raw/550e...-a.jpg", "file_name": "drawing.jpg", "content_type": "image/jpeg", "size_bytes": 540000 }
  ]
}
```

* `attachments[].upload_key` is the **private S3 key** previously uploaded via presign.

**Response**

`201 Created`

```json
{ "id": "r_9Kc8", "last_event_ts": "2025-09-25T12:00:01Z" }
```

**Errors**

* `400` invalid input (schema/size)
* `409` idempotency conflict (same `Idempotency-Key` different payload)
* `429` rate limited
* `500` operational

### C.2 `GET /rfqs/{id}`

Returns RFQ meta (no event list).

`200 OK`

```json
{
  "id": "r_9Kc8",
  "tenant_id": "t1",
  "manufacturer_id": "mfg_123",
  "buyer": { "email": "a@b.com", "name": "Alice" },
  "subject": "CNC milling prototype",
  "status": "open",
  "created_at": "2025-09-25T12:00:01Z",
  "last_event_ts": "2025-09-25T12:05:10Z",
  "participants": [
    { "role": "buyer", "email": "a@b.com", "name": "Alice" },
    { "role": "manufacturer", "email": "shop@example.com", "name": "Acme" }
  ]
}
```

**Errors**: `404`, `403` (future), `500`.

### C.3 `GET /rfqs/{id}/events?since=<ISO8601>&limit=<n>`

Poll incremental events.

* `since` optional (default: start)
* `limit` optional, **default 50**, **max 200**
* Returns sorted list and a `next_since` cursor (the `ts` of the last event).

`200 OK`

```json
{
  "items": [
    { "id": "01HX3", "rfq_id": "r_9Kc8", "ts": "2025-09-25T12:05:10Z", "by": "manufacturer", "type": "message", "body": "Can do 10-day" }
  ],
  "next_since": "2025-09-25T12:05:10Z"
}
```

**Errors**: `400` bad `since`, `404`, `500`.

### C.4 `POST /rfqs/{id}/messages`

Append a message event.

**Request**

```json
{ "by": "buyer", "body": "Can you do anodizing?", "attachments": [] }
```

**Response** `201 Created`

```json
{ "ts": "2025-09-25T12:07:31Z" }
```

**Errors**: `400` invalid size/types, `404`, `409` idempotency mismatch, `429`, `500`.

### C.5 `POST /uploads/presign`

Create a constrained presigned URL.

**Request**

```json
{ "tenant_id": "t1", "pathType": "imageRaw", "content_type": "image/jpeg", "size_bytes": 540000 }
```

**Response** `200 OK`

```json
{
  "url": "https://app-private-.../tenants/t1/images/raw/550e...-a.jpg?X-Amz-Algorithm=...",
  "key": "tenants/t1/images/raw/550e...-a.jpg",
  "expires_in": 600
}
```

**Server constraints**

* Valid MIME (`image/*` or `application/pdf`)
* Max size 15 MB
* Key prefix locked to `tenants/{tenant_id}/images/raw/`
* Expiry 5–10 minutes

**Errors**: `400`, `403` (tenant mismatch), `429`, `500`.

### C.6 `POST /manufacturers` (admin)

Create/update manufacturer; triggers catalog rebuild.

**Request**

```json
{
  "id": "mfg_123",
  "tenant_id": "t1",
  "name": "Acme",
  "categories": ["machining"],
  "capabilities": ["cnc_milling"],
  "contact_email": "shop@example.com",
  "media": [{ "image_manifest_id": "img_ab12", "alt": "Shop" }]
}
```

**Response** `202 Accepted`

```json
{ "ok": true, "rebuild": ["category/machining", "category_state/machining/OH", "manufacturer/mfg_123"] }
```

**Errors**: `400`, `401/403`, `500`.

### C.7 `GET /catalog/...`

* Served **directly** from S3/CloudFront.
* Supports `ETag` and `If-None-Match` (returns `304 Not Modified`).

---

## D) Idempotency, Rate Limits, and Error Envelope

* **Idempotency:** For POSTs, if `Idempotency-Key` header is present:

  * First success stores a tiny marker at `app-private-<env>/idem/{sha256(key)}.json` with a hash of the request body.
  * Subsequent requests with the **same key and same body** → **200/201** replay response.
  * Same key **different body** → **409 Conflict**.
* **Rate Limits (API Gateway usage plan):**

  * Anonymous POSTs: **10 req/min/IP**, burst **20**
  * GET catalog/events: **60 req/min/IP**, burst **120**
* **Error envelope (uniform)**

```json
{ "code": "validation_error", "message": "Attachment too large", "details": { "max_bytes": 15728640 } }
```

---

## E) Lambda Functions (MVP)

> All Lambdas are **Rust**, **arm64**, **256–512 MB**, **timeout 10–30s** (image ingest 60–120s). Logging with `tracing`. Deployed via **CDK**.

### E.1 `api_uploads`

* **Purpose:** Issue presigned URLs for uploads.
* **Triggers:** API Gateway `/v1/uploads/presign` (POST)
* **Input:** JSON (tenant_id, pathType, content_type, size_bytes)
* **Output:** `{ url, key, expires_in }`
* **IAM:** `s3:PutObject` on `app-private-<env>/tenants/${tenant}/images/raw/*`
* **Validation:** MIME whitelist, size ≤ 15 MB, prefix lock, expiry 5–10 min
* **Notes:** Adds `x-amz-meta-tenant`.

### E.2 `image_ingest`

* **Purpose:** On raw upload, create derivatives + manifest.
* **Triggers:** S3 (PUT) on `app-private-<env>/tenants/*/images/raw/*`
* **Input:** S3 event
* **Process:**

  1. Validate content type/size.
  2. Load image, strip EXIF.
  3. Generate AVIF/WebP/JPEG at widths 320/640/1024/1600.
  4. Compute `image_id` (content hash).
  5. Write variants to **public** bucket; write manifest JSON.
* **Output:** None (side-effects)
* **IAM:** Read private bucket raw; write public bucket manifests/variants.
* **Timeout/Memory:** 120s / 512MB
* **Failure handling:** On error, move raw object to `reingest/` or add a `.error.json` next to it.

### E.3 `api_rfqs`

* **Purpose:** Create RFQs, fetch RFQ meta, list/post events.
* **Triggers:** API Gateway:

  * `POST /v1/rfqs`
  * `GET /v1/rfqs/{id}`
  * `GET /v1/rfqs/{id}/events`
  * `POST /v1/rfqs/{id}/messages`
* **Input/Output:** As defined in Section C
* **Side-effects:** Writes `meta.json`, `index.json`, appends `events/*.json`; sends SES notifications.
* **IAM:** `s3:GetObject/PutObject` on `app-private-<env>/rfq/*`; `ses:SendEmail`/`SendTemplatedEmail`.
* **Notes:** Enforces idempotency; clamps message sizes; normalizes timestamps server-side.

### E.4 `api_manufacturers`

* **Purpose:** Admin CRUD for manufacturer profiles; store source; kick publisher.
* **Triggers:** API Gateway `POST /v1/manufacturers`
* **Process:** Validate payload → write `manufacturer/{id}.json` (public) and internal source in private if desired → enqueue rebuild (invoke `publisher` synchronously or async).
* **IAM:** Write public catalog files; optional private source area.
* **Auth:** Bearer token (static secret in SSM Param for MVP).

### E.5 `publisher`

* **Purpose:** Rebuild catalog slices affected by a manufacturer/offer change.
* **Triggers:** Direct invoke from `api_manufacturers` (or EventBridge later)
* **Process:** Load manufacturer(s) → compute impacted slices (category, category_state) → write compact slice arrays with ETags.
* **IAM:** Read manufacturer JSON; write catalog slices.
* **Perf:** Keep memory 256MB; typical runtime < 2s/slice.

### E.6 `notifier` (optional inline inside `api_rfqs`)

* **Purpose:** Wrap SES sends for RFQ create/reply acks.
* **Templates:** `rfq_created_manufacturer`, `rfq_created_buyer`, `rfq_new_message`.
* **Bounces/Complaints:** CloudWatch alarm; suppress list stored in private S3 (`ses/suppressions.json`).

---

## F) Validation & Security Details

* **CORS:** Only allow `https://<username>.github.io` (and custom domain if used).
* **Content Security Policy (SPA):** restrict to CloudFront domain + API domain.
* **Request validation:** JSON schema per route; reject unknown fields.
* **Path traversal/key injection:** keys are **server-generated**; clients never choose final S3 object names beyond a presigned **prefix**.
* **ETag behavior:** publisher and image_ingest should set deterministic content to keep ETags sticky.

---

## G) SDK/DTO Snippets (for frontend)

```ts
// Create RFQ
export type CreateRfqReq = {
  tenant_id: string;
  manufacturer_id: string;
  buyer: { email: string; name?: string };
  subject: string;
  body: string;
  attachments?: Array<{
    upload_key: string;
    file_name: string;
    content_type: string;
    size_bytes: number;
  }>;
};

export type CreateRfqResp = { id: string; last_event_ts: string };

// Poll events
export type ListEventsResp = { items: RfqEvent[]; next_since?: string };
```
