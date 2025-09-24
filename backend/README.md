# Terra Marketplace Backend

A serverless backend API for the Terra digital marketplace built with Rust and deployed on AWS Lambda.

## Features

- **Serverless Architecture**: Built for AWS Lambda with minimal cold start times
- **RESTful API**: Clean API design with proper HTTP methods and status codes
- **Authentication**: JWT-based authentication with secure password hashing
- **Database**: DynamoDB for scalable NoSQL data storage
- **File Storage**: S3 integration for product images and assets
- **Real-time**: Support for real-time features through WebSocket connections
- **Security**: Input validation, rate limiting, and CORS protection
- **Monitoring**: CloudWatch logs and metrics for observability

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Frontend      │    │   API Gateway   │    │   Lambda Fns    │
│   (GitHub Pages)│◄──►│                 │◄──►│   - Products    │
│                 │    │                 │    │   - Users       │
└─────────────────┘    └─────────────────┘    │   - Orders      │
                                               │   - Auth        │
                                               └─────────────────┘
                                                        ▲
                                                        │
                                               ┌─────────────────┐
                                               │   DynamoDB      │
                                               │   - Users       │
                                               │   - Products    │
                                               │   - Orders      │
                                               └─────────────────┘
```

## API Endpoints

### Authentication
- `POST /auth/register` - Register a new user
- `POST /auth/login` - Authenticate user and get JWT token
- `POST /auth/refresh` - Refresh JWT token

### Users
- `GET /users` - List users (admin only)
- `GET /users/{id}` - Get user profile
- `PUT /users/{id}` - Update user profile
- `DELETE /users/{id}` - Delete user account

### Products
- `GET /products` - List products with filtering and pagination
- `GET /products/{id}` - Get product details
- `POST /products` - Create new product (sellers only)
- `PUT /products/{id}` - Update product (owner only)
- `DELETE /products/{id}` - Delete product (owner only)

### Orders
- `GET /orders` - List user's orders
- `GET /orders/{id}` - Get order details
- `POST /orders` - Create new order
- `PUT /orders/{id}` - Update order status

## Development Setup

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [AWS CLI](https://aws.amazon.com/cli/) configured with appropriate credentials
- [AWS SAM CLI](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-cli-install.html)
- [Docker](https://www.docker.com/) (for local DynamoDB)

### Installation

1. **Clone the repository**:
   ```bash
   git clone https://github.com/JPrier/Terra.git
   cd Terra/backend
   ```

2. **Install dependencies**:
   ```bash
   make install
   ```

3. **Set up environment variables**:
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

4. **Build the project**:
   ```bash
   make build
   ```

5. **Run tests**:
   ```bash
   make test
   ```

### Local Development

1. **Start local DynamoDB**:
   ```bash
   make local-db
   ```

2. **Create tables**:
   ```bash
   make create-tables
   ```

3. **Start local API server**:
   ```bash
   make local
   ```

The API will be available at `http://localhost:8080`.

## Deployment

### Development Environment

```bash
export JWT_SECRET="your-jwt-secret-here"
make deploy-dev
```

### Production Environment

```bash
export JWT_SECRET="your-production-jwt-secret"
make deploy-prod
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `ENVIRONMENT` | Environment name (dev/staging/prod) | `dev` |
| `DYNAMODB_TABLE_PREFIX` | Prefix for DynamoDB tables | `terra` |
| `JWT_SECRET` | Secret key for JWT signing | *required* |
| `JWT_EXPIRATION_HOURS` | JWT token expiration time | `24` |
| `CORS_ORIGINS` | Comma-separated allowed origins | `*` |
| `AWS_REGION` | AWS region | `us-east-1` |

## Testing

### Unit Tests
```bash
make test
```

### Integration Tests
```bash
make integration-test
```

### Load Testing
```bash
make load-test
```

## API Usage Examples

### Register a new user
```bash
curl -X POST https://your-api-url/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "SecurePassword123",
    "username": "newuser",
    "first_name": "John",
    "last_name": "Doe"
  }'
```

### Login
```bash
curl -X POST https://your-api-url/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "SecurePassword123"
  }'
```

### Get products
```bash
curl -X GET "https://your-api-url/products?category=electronics&limit=10"
```

### Create a product (requires authentication)
```bash
curl -X POST https://your-api-url/products \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "title": "Amazing Product",
    "description": "This is an amazing product",
    "price": 29.99,
    "category": "electronics",
    "stock_quantity": 100,
    "images": ["https://example.com/image.jpg"],
    "tags": ["electronics", "gadget"]
  }'
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Run the test suite
6. Submit a pull request

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](../LICENSE) file for details.

## Security

If you discover a security vulnerability, please send an email to security@terra-marketplace.com instead of using the issue tracker.

## Support

For support and questions, please open an issue on GitHub or contact us at support@terra-marketplace.com.