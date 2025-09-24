# Terra Marketplace Backend Makefile

.PHONY: help build build-release test lint clean deploy local install

# Default target
help:
	@echo "Terra Marketplace Backend"
	@echo ""
	@echo "Available targets:"
	@echo "  help          - Show this help message"
	@echo "  install       - Install dependencies and tools"
	@echo "  build         - Build for development"
	@echo "  build-release - Build for production release"
	@echo "  test          - Run all tests"
	@echo "  lint          - Run linter checks"
	@echo "  clean         - Clean build artifacts"
	@echo "  local         - Run local development server"
	@echo "  deploy        - Deploy to AWS (requires AWS CLI and SAM CLI)"
	@echo "  deploy-dev    - Deploy to development environment"
	@echo "  deploy-prod   - Deploy to production environment"

# Install dependencies and tools
install:
	@echo "Installing Rust dependencies..."
	cargo fetch
	@echo "Installing cargo-lambda for AWS Lambda builds..."
	cargo install cargo-lambda
	@echo "Installing AWS SAM CLI (if not already installed)..."
	@command -v sam >/dev/null 2>&1 || { \
		echo "AWS SAM CLI not found. Please install it from https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-cli-install.html"; \
		exit 1; \
	}

# Build for development
build:
	@echo "Building Terra backend for development..."
	cargo build

# Build for production release
build-release:
	@echo "Building Terra backend for production..."
	cargo build --release

# Build Lambda functions
build-lambda:
	@echo "Building Lambda functions..."
	mkdir -p target/lambda
	cargo lambda build --release
	@echo "Copying Lambda binaries..."
	mkdir -p target/lambda/products && cp target/lambda/products/bootstrap target/lambda/products/
	mkdir -p target/lambda/users && cp target/lambda/users/bootstrap target/lambda/users/
	mkdir -p target/lambda/auth && cp target/lambda/auth/bootstrap target/lambda/auth/
	mkdir -p target/lambda/orders && cp target/lambda/orders/bootstrap target/lambda/orders/

# Run tests
test:
	@echo "Running tests..."
	cargo test

# Run linter
lint:
	@echo "Running linter checks..."
	cargo clippy -- -D warnings
	cargo fmt --check

# Format code
format:
	@echo "Formatting code..."
	cargo fmt

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -rf target/lambda/

# Run local development server
local:
	@echo "Starting local development server..."
	sam local start-api --template-file template.yaml --port 8080

# Deploy to AWS
deploy: build-lambda
	@echo "Deploying to AWS..."
	sam deploy --guided

# Deploy to development environment
deploy-dev: build-lambda
	@echo "Deploying to development environment..."
	sam deploy \
		--template-file template.yaml \
		--stack-name terra-backend-dev \
		--parameter-overrides \
			Environment=dev \
			DynamoDBTablePrefix=terra \
			JWTSecret=$(JWT_SECRET) \
			CorsOrigins="https://jprier.github.io,http://localhost:3000" \
		--capabilities CAPABILITY_IAM \
		--region us-east-1 \
		--no-confirm-changeset

# Deploy to production environment
deploy-prod: build-lambda
	@echo "Deploying to production environment..."
	sam deploy \
		--template-file template.yaml \
		--stack-name terra-backend-prod \
		--parameter-overrides \
			Environment=prod \
			DynamoDBTablePrefix=terra \
			JWTSecret=$(JWT_SECRET) \
			CorsOrigins="https://jprier.github.io" \
		--capabilities CAPABILITY_IAM \
		--region us-east-1 \
		--no-confirm-changeset

# Validate SAM template
validate:
	@echo "Validating SAM template..."
	sam validate --template-file template.yaml

# Package for deployment
package: build-lambda
	@echo "Packaging application..."
	sam package \
		--template-file template.yaml \
		--s3-bucket terra-deployment-artifacts \
		--output-template-file packaged-template.yaml

# Run local DynamoDB
local-db:
	@echo "Starting local DynamoDB..."
	docker run -p 8000:8000 amazon/dynamodb-local

# Create DynamoDB tables locally
create-tables:
	@echo "Creating DynamoDB tables locally..."
	aws dynamodb create-table \
		--table-name terra_users_dev \
		--attribute-definitions \
			AttributeName=id,AttributeType=S \
			AttributeName=email,AttributeType=S \
			AttributeName=username,AttributeType=S \
		--key-schema AttributeName=id,KeyType=HASH \
		--billing-mode PAY_PER_REQUEST \
		--global-secondary-indexes \
			IndexName=email-index,KeySchema=[{AttributeName=email,KeyType=HASH}],Projection={ProjectionType=ALL} \
			IndexName=username-index,KeySchema=[{AttributeName=username,KeyType=HASH}],Projection={ProjectionType=ALL} \
		--endpoint-url http://localhost:8000

	aws dynamodb create-table \
		--table-name terra_products_dev \
		--attribute-definitions \
			AttributeName=id,AttributeType=S \
			AttributeName=seller_id,AttributeType=S \
			AttributeName=category,AttributeType=S \
			AttributeName=created_at,AttributeType=S \
		--key-schema AttributeName=id,KeyType=HASH \
		--billing-mode PAY_PER_REQUEST \
		--global-secondary-indexes \
			IndexName=seller-index,KeySchema=[{AttributeName=seller_id,KeyType=HASH},{AttributeName=created_at,KeyType=RANGE}],Projection={ProjectionType=ALL} \
			IndexName=category-index,KeySchema=[{AttributeName=category,KeyType=HASH},{AttributeName=created_at,KeyType=RANGE}],Projection={ProjectionType=ALL} \
		--endpoint-url http://localhost:8000

	aws dynamodb create-table \
		--table-name terra_orders_dev \
		--attribute-definitions \
			AttributeName=id,AttributeType=S \
			AttributeName=user_id,AttributeType=S \
			AttributeName=created_at,AttributeType=S \
		--key-schema AttributeName=id,KeyType=HASH \
		--billing-mode PAY_PER_REQUEST \
		--global-secondary-indexes \
			IndexName=user-index,KeySchema=[{AttributeName=user_id,KeyType=HASH},{AttributeName=created_at,KeyType=RANGE}],Projection={ProjectionType=ALL} \
		--endpoint-url http://localhost:8000

# Integration tests
integration-test:
	@echo "Running integration tests..."
	cargo test --test integration_tests

# Load test
load-test:
	@echo "Running load tests..."
	@command -v artillery >/dev/null 2>&1 || { \
		echo "Artillery not found. Install with: npm install -g artillery"; \
		exit 1; \
	}
	artillery run load-test.yml

# Generate API documentation
docs:
	@echo "Generating API documentation..."
	cargo doc --open

# Check security vulnerabilities
security-check:
	@echo "Checking for security vulnerabilities..."
	cargo audit