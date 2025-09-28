"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TerraStack = void 0;
const cdk = require("aws-cdk-lib");
const s3 = require("aws-cdk-lib/aws-s3");
const s3deploy = require("aws-cdk-lib/aws-s3-deployment");
const apigateway = require("aws-cdk-lib/aws-apigateway");
const lambda = require("aws-cdk-lib/aws-lambda");
const iam = require("aws-cdk-lib/aws-iam");
const cloudfront = require("aws-cdk-lib/aws-cloudfront");
const origins = require("aws-cdk-lib/aws-cloudfront-origins");
class TerraStack extends cdk.Stack {
    constructor(scope, id, props) {
        super(scope, id, props);
        const environment = this.node.tryGetContext('environment') || 'dev';
        // S3 Buckets as per design
        const publicBucket = new s3.Bucket(this, 'PublicBucket', {
            bucketName: `app-public-${environment}`,
            publicReadAccess: false,
            blockPublicAccess: s3.BlockPublicAccess.BLOCK_ALL,
            removalPolicy: cdk.RemovalPolicy.DESTROY,
            lifecycleRules: [
                {
                    id: 'delete-incomplete-multipart-uploads',
                    abortIncompleteMultipartUploadAfter: cdk.Duration.days(1),
                },
            ],
        });
        const privateBucket = new s3.Bucket(this, 'PrivateBucket', {
            bucketName: `app-private-${environment}`,
            publicReadAccess: false,
            blockPublicAccess: s3.BlockPublicAccess.BLOCK_ALL,
            removalPolicy: cdk.RemovalPolicy.DESTROY,
            lifecycleRules: [
                {
                    id: 'move-raw-uploads-to-ia',
                    transitions: [
                        {
                            storageClass: s3.StorageClass.INFREQUENT_ACCESS,
                            transitionAfter: cdk.Duration.days(30),
                        },
                    ],
                    prefix: 'tenants/',
                },
                {
                    id: 'delete-temp-files',
                    prefix: 'tmp/',
                    expiration: cdk.Duration.days(1),
                },
                {
                    id: 'delete-idempotency-markers',
                    prefix: 'idem/',
                    expiration: cdk.Duration.days(1),
                },
            ],
        });
        // Deploy test manufacturer data to public S3 bucket
        const testDataDeployment = new s3deploy.BucketDeployment(this, 'TestDataDeployment', {
            sources: [s3deploy.Source.asset('./data')],
            destinationBucket: publicBucket,
            destinationKeyPrefix: 'test-data/',
            // Replace existing data on redeploy
            prune: true,
            // Cache control for JSON data files
            cacheControl: [
                s3deploy.CacheControl.setPublic(),
                s3deploy.CacheControl.maxAge(cdk.Duration.hours(1)),
            ],
        });
        // Lambda execution role
        const lambdaRole = new iam.Role(this, 'LambdaRole', {
            assumedBy: new iam.ServicePrincipal('lambda.amazonaws.com'),
            managedPolicies: [
                iam.ManagedPolicy.fromAwsManagedPolicyName('service-role/AWSLambdaBasicExecutionRole'),
            ],
            inlinePolicies: {
                S3Policy: new iam.PolicyDocument({
                    statements: [
                        new iam.PolicyStatement({
                            effect: iam.Effect.ALLOW,
                            actions: [
                                's3:GetObject',
                                's3:PutObject',
                                's3:DeleteObject',
                                's3:ListBucket',
                            ],
                            resources: [
                                publicBucket.bucketArn,
                                `${publicBucket.bucketArn}/*`,
                                privateBucket.bucketArn,
                                `${privateBucket.bucketArn}/*`,
                            ],
                        }),
                    ],
                }),
                SESPolicy: new iam.PolicyDocument({
                    statements: [
                        new iam.PolicyStatement({
                            effect: iam.Effect.ALLOW,
                            actions: [
                                'ses:SendEmail',
                                'ses:SendTemplatedEmail',
                                'ses:SendRawEmail',
                            ],
                            resources: ['*'], // SES requires wildcard for sending emails
                        }),
                    ],
                }),
            },
        });
        // Lambda functions (simplified for MVP - would build from Rust binaries in production)
        const apiRfqsLambda = new lambda.Function(this, 'ApiRfqsLambda', {
            runtime: lambda.Runtime.PROVIDED_AL2,
            handler: 'bootstrap',
            code: lambda.Code.fromAsset('../backend/lambdas/api_rfqs'),
            role: lambdaRole,
            timeout: cdk.Duration.seconds(30),
            memorySize: 512,
            architecture: lambda.Architecture.ARM_64,
            environment: {
                ENVIRONMENT: environment,
                PUBLIC_BUCKET: publicBucket.bucketName,
                PRIVATE_BUCKET: privateBucket.bucketName,
                FROM_EMAIL: 'noreply@terra-platform.com',
                RUST_LOG: 'info',
            },
        });
        const apiUploadsLambda = new lambda.Function(this, 'ApiUploadsLambda', {
            runtime: lambda.Runtime.PROVIDED_AL2,
            handler: 'bootstrap',
            code: lambda.Code.fromAsset('../backend/lambdas/api_uploads'),
            role: lambdaRole,
            timeout: cdk.Duration.seconds(10),
            memorySize: 256,
            architecture: lambda.Architecture.ARM_64,
            environment: {
                ENVIRONMENT: environment,
                PUBLIC_BUCKET: publicBucket.bucketName,
                PRIVATE_BUCKET: privateBucket.bucketName,
                RUST_LOG: 'info',
            },
        });
        const apiManufacturersLambda = new lambda.Function(this, 'ApiManufacturersLambda', {
            runtime: lambda.Runtime.PROVIDED_AL2,
            handler: 'bootstrap',
            code: lambda.Code.fromAsset('../backend/lambdas/api_manufacturers'),
            role: lambdaRole,
            timeout: cdk.Duration.seconds(30),
            memorySize: 256,
            architecture: lambda.Architecture.ARM_64,
            environment: {
                ENVIRONMENT: environment,
                PUBLIC_BUCKET: publicBucket.bucketName,
                PRIVATE_BUCKET: privateBucket.bucketName,
                RUST_LOG: 'info',
            },
        });
        // Publisher Lambda for generating static HTML and JSON
        const publisherLambda = new lambda.Function(this, 'PublisherLambda', {
            runtime: lambda.Runtime.PROVIDED_AL2,
            handler: 'bootstrap',
            code: lambda.Code.fromAsset('../backend/lambdas/publisher'),
            role: lambdaRole,
            timeout: cdk.Duration.seconds(60),
            memorySize: 512,
            architecture: lambda.Architecture.ARM_64,
            environment: {
                ENVIRONMENT: environment,
                PUBLIC_BUCKET: publicBucket.bucketName,
                PRIVATE_BUCKET: privateBucket.bucketName,
                RUST_LOG: 'info',
            },
        });
        // Image processing Lambda
        const imageIngestLambda = new lambda.Function(this, 'ImageIngestLambda', {
            runtime: lambda.Runtime.PROVIDED_AL2,
            handler: 'bootstrap',
            code: lambda.Code.fromAsset('../backend/lambdas/image_ingest'),
            role: lambdaRole,
            timeout: cdk.Duration.seconds(120),
            memorySize: 1024,
            architecture: lambda.Architecture.ARM_64,
            environment: {
                ENVIRONMENT: environment,
                PUBLIC_BUCKET: publicBucket.bucketName,
                PRIVATE_BUCKET: privateBucket.bucketName,
                RUST_LOG: 'info',
            },
        });
        // API Gateway HTTP API as specified in design
        const api = new apigateway.RestApi(this, 'TerraApi', {
            restApiName: `terra-api-${environment}`,
            description: 'Terra Manufacturing Platform API',
            defaultCorsPreflightOptions: {
                allowOrigins: apigateway.Cors.ALL_ORIGINS,
                allowMethods: apigateway.Cors.ALL_METHODS,
                allowHeaders: [
                    'Content-Type',
                    'Authorization',
                    'Idempotency-Key',
                    'X-Request-Id',
                    'If-None-Match',
                ],
                exposeHeaders: [
                    'X-Request-Id',
                    'ETag',
                ],
            },
        });
        // API Gateway routes
        const v1 = api.root.addResource('v1');
        // RFQ endpoints
        const rfqs = v1.addResource('rfqs');
        rfqs.addMethod('POST', new apigateway.LambdaIntegration(apiRfqsLambda));
        const rfqById = rfqs.addResource('{id}');
        rfqById.addMethod('GET', new apigateway.LambdaIntegration(apiRfqsLambda));
        const rfqEvents = rfqById.addResource('events');
        rfqEvents.addMethod('GET', new apigateway.LambdaIntegration(apiRfqsLambda));
        const rfqMessages = rfqById.addResource('messages');
        rfqMessages.addMethod('POST', new apigateway.LambdaIntegration(apiRfqsLambda));
        // Upload endpoints
        const uploads = v1.addResource('uploads');
        const presign = uploads.addResource('presign');
        presign.addMethod('POST', new apigateway.LambdaIntegration(apiUploadsLambda));
        // Manufacturer endpoints (admin)
        const manufacturers = v1.addResource('manufacturers');
        manufacturers.addMethod('POST', new apigateway.LambdaIntegration(apiManufacturersLambda));
        // Add a manual trigger for the publisher Lambda (for demo purposes)
        const publisherTrigger = new lambda.Function(this, 'PublisherTriggerLambda', {
            runtime: lambda.Runtime.NODEJS_18_X,
            handler: 'index.handler',
            code: lambda.Code.fromInline(`
        const AWS = require('aws-sdk');
        const lambda = new AWS.Lambda();
        
        exports.handler = async (event) => {
          try {
            const result = await lambda.invoke({
              FunctionName: '${publisherLambda.functionName}',
              InvocationType: 'Event',
              Payload: JSON.stringify({ trigger: 'manual', timestamp: new Date().toISOString() })
            }).promise();
            
            return {
              statusCode: 200,
              body: JSON.stringify({ message: 'Catalog rebuild triggered', result })
            };
          } catch (error) {
            return {
              statusCode: 500,
              body: JSON.stringify({ error: error.message })
            };
          }
        };
      `),
            timeout: cdk.Duration.seconds(30),
        });
        // Publisher trigger endpoint (admin)
        const publisherResource = v1.addResource('publisher');
        const publisherTriggerResource = publisherResource.addResource('trigger');
        publisherTriggerResource.addMethod('POST', new apigateway.LambdaIntegration(publisherTrigger));
        // CloudFront distribution for public assets and static catalog pages
        const distribution = new cloudfront.Distribution(this, 'Distribution', {
            defaultBehavior: {
                origin: new origins.S3Origin(publicBucket),
                viewerProtocolPolicy: cloudfront.ViewerProtocolPolicy.REDIRECT_TO_HTTPS,
                cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
                compress: true,
                allowedMethods: cloudfront.AllowedMethods.ALLOW_GET_HEAD,
            },
            additionalBehaviors: {
                // Catalog JSON files - long cache with immutable content
                '/catalog/category/*': {
                    origin: new origins.S3Origin(publicBucket),
                    cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
                    viewerProtocolPolicy: cloudfront.ViewerProtocolPolicy.REDIRECT_TO_HTTPS,
                    responseHeadersPolicy: cloudfront.ResponseHeadersPolicy.CORS_ALLOW_ALL_ORIGINS,
                },
                '/manufacturer/*': {
                    origin: new origins.S3Origin(publicBucket),
                    cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
                    viewerProtocolPolicy: cloudfront.ViewerProtocolPolicy.REDIRECT_TO_HTTPS,
                    responseHeadersPolicy: cloudfront.ResponseHeadersPolicy.CORS_ALLOW_ALL_ORIGINS,
                },
                // Static HTML catalog pages - shorter cache for faster updates
                '/catalog/*': {
                    origin: new origins.S3Origin(publicBucket),
                    cachePolicy: new cloudfront.CachePolicy(this, 'CatalogHtmlCachePolicy', {
                        cachePolicyName: `terra-catalog-html-${environment}`,
                        defaultTtl: cdk.Duration.minutes(5),
                        maxTtl: cdk.Duration.hours(1),
                        minTtl: cdk.Duration.seconds(0),
                    }),
                    viewerProtocolPolicy: cloudfront.ViewerProtocolPolicy.REDIRECT_TO_HTTPS,
                    compress: true,
                },
                // CSS and other assets
                '/src/*': {
                    origin: new origins.S3Origin(publicBucket),
                    cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
                    viewerProtocolPolicy: cloudfront.ViewerProtocolPolicy.REDIRECT_TO_HTTPS,
                    compress: true,
                },
                // Images and derived assets
                '/tenants/*': {
                    origin: new origins.S3Origin(publicBucket),
                    cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
                    viewerProtocolPolicy: cloudfront.ViewerProtocolPolicy.REDIRECT_TO_HTTPS,
                },
            },
        });
        // Grant permissions for the trigger to invoke the publisher
        publisherLambda.grantInvoke(publisherTrigger);
        // Outputs
        new cdk.CfnOutput(this, 'ApiUrl', {
            value: api.url,
            description: 'API Gateway URL',
        });
        new cdk.CfnOutput(this, 'DistributionUrl', {
            value: `https://${distribution.distributionDomainName}`,
            description: 'CloudFront Distribution URL',
        });
        new cdk.CfnOutput(this, 'PublicBucketName', {
            value: publicBucket.bucketName,
            description: 'Public S3 Bucket Name',
        });
        new cdk.CfnOutput(this, 'PrivateBucketName', {
            value: privateBucket.bucketName,
            description: 'Private S3 Bucket Name',
        });
    }
}
exports.TerraStack = TerraStack;
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoidGVycmEtc3RhY2suanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyJ0ZXJyYS1zdGFjay50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiOzs7QUFBQSxtQ0FBbUM7QUFDbkMseUNBQXlDO0FBQ3pDLDBEQUEwRDtBQUMxRCx5REFBeUQ7QUFDekQsaURBQWlEO0FBQ2pELDJDQUEyQztBQUUzQyx5REFBeUQ7QUFDekQsOERBQThEO0FBRzlELE1BQWEsVUFBVyxTQUFRLEdBQUcsQ0FBQyxLQUFLO0lBQ3ZDLFlBQVksS0FBZ0IsRUFBRSxFQUFVLEVBQUUsS0FBc0I7UUFDOUQsS0FBSyxDQUFDLEtBQUssRUFBRSxFQUFFLEVBQUUsS0FBSyxDQUFDLENBQUM7UUFFeEIsTUFBTSxXQUFXLEdBQUcsSUFBSSxDQUFDLElBQUksQ0FBQyxhQUFhLENBQUMsYUFBYSxDQUFDLElBQUksS0FBSyxDQUFDO1FBRXBFLDJCQUEyQjtRQUMzQixNQUFNLFlBQVksR0FBRyxJQUFJLEVBQUUsQ0FBQyxNQUFNLENBQUMsSUFBSSxFQUFFLGNBQWMsRUFBRTtZQUN2RCxVQUFVLEVBQUUsY0FBYyxXQUFXLEVBQUU7WUFDdkMsZ0JBQWdCLEVBQUUsS0FBSztZQUN2QixpQkFBaUIsRUFBRSxFQUFFLENBQUMsaUJBQWlCLENBQUMsU0FBUztZQUNqRCxhQUFhLEVBQUUsR0FBRyxDQUFDLGFBQWEsQ0FBQyxPQUFPO1lBQ3hDLGNBQWMsRUFBRTtnQkFDZDtvQkFDRSxFQUFFLEVBQUUscUNBQXFDO29CQUN6QyxtQ0FBbUMsRUFBRSxHQUFHLENBQUMsUUFBUSxDQUFDLElBQUksQ0FBQyxDQUFDLENBQUM7aUJBQzFEO2FBQ0Y7U0FDRixDQUFDLENBQUM7UUFFSCxNQUFNLGFBQWEsR0FBRyxJQUFJLEVBQUUsQ0FBQyxNQUFNLENBQUMsSUFBSSxFQUFFLGVBQWUsRUFBRTtZQUN6RCxVQUFVLEVBQUUsZUFBZSxXQUFXLEVBQUU7WUFDeEMsZ0JBQWdCLEVBQUUsS0FBSztZQUN2QixpQkFBaUIsRUFBRSxF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RUFBRTtZQUMxQyxLQUFLLEVBQUUsWUFBWSxDQUFDLFVBQVU7WUFDOUIsV0FBVyxFQUFFLHVCQUF1QjtTQUNyQyxDQUFDLENBQUM7UUFFSCxJQUFJLEdBQUcsQ0FBQyxTQUFTLENBQUMsSUFBSSxFQUFFLG1CQUFtQixFQUFFO1lBQzNDLEtBQUssRUFBRSxhQUFhLENBQUMsVUFBVTtZQUMvQixXQUFXLEVBQUUsd0JBQXdCO1NBQ3RDLENBQUMsQ0FBQztJQUNMLENBQUM7Q0FDRjtBQTFWRCxnQ0EwVkMiLCJzb3VyY2VzQ29udGVudCI6WyJpbXBvcnQgKiBhcyBjZGsgZnJvbSAnYXdzLWNkay1saWInO1xuaW1wb3J0ICogYXMgczMgZnJvbSAnYXdzLWNkay1saWIvYXdzLXMzJztcbmltcG9ydCAqIGFzIHMzZGVwbG95IGZyb20gJ2F3cy1jZGstbGliL2F3cy1zMy1kZXBsb3ltZW50JztcbmltcG9ydCAqIGFzIGFwaWdhdGV3YXkgZnJvbSAnYXdzLWNkay1saWIvYXdzLWFwaWdhdGV3YXknO1xuaW1wb3J0ICogYXMgbGFtYmRhIGZyb20gJ2F3cy1jZGstbGliL2F3cy1sYW1iZGEnO1xuaW1wb3J0ICogYXMgaWFtIGZyb20gJ2F3cy1jZGstbGliL2F3cy1pYW0nO1xuaW1wb3J0ICogYXMgc2VzIGZyb20gJ2F3cy1jZGstbGliL2F3cy1zZXMnO1xuaW1wb3J0ICogYXMgY2xvdWRmcm9udCBmcm9tICdhd3MtY2RrLWxpYi9hd3MtY2xvdWRmcm9udCc7XG5pbXBvcnQgKiBhcyBvcmlnaW5zIGZyb20gJ2F3cy1jZGstbGliL2F3cy1jbG91ZGZyb250LW9yaWdpbnMnO1xuaW1wb3J0IHsgQ29uc3RydWN0IH0gZnJvbSAnY29uc3RydWN0cyc7XG5cbmV4cG9ydCBjbGFzcyBUZXJyYVN0YWNrIGV4dGVuZHMgY2RrLlN0YWNrIHtcbiAgY29uc3RydWN0b3Ioc2NvcGU6IENvbnN0cnVjdCwgaWQ6IHN0cmluZywgcHJvcHM/OiBjZGsuU3RhY2tQcm9wcykge1xuICAgIHN1cGVyKHNjb3BlLCBpZCwgcHJvcHMpO1xuXG4gICAgY29uc3QgZW52aXJvbm1lbnQgPSB0aGlzLm5vZGUudHJ5R2V0Q29udGV4dCgnZW52aXJvbm1lbnQnKSB8fCAnZGV2JztcblxuICAgIC8vIFMzIEJ1Y2tldHMgYXMgcGVyIGRlc2lnblxuICAgIGNvbnN0IHB1YmxpY0J1Y2tldCA9IG5ldyBzMy5CdWNrZXQodGhpcywgJ1B1YmxpY0J1Y2tldCcsIHtcbiAgICAgIGJ1Y2tldE5hbWU6IGBhcHAtcHVibGljLSR7ZW52aXJvbm1lbnR9YCxcbiAgICAgIHB1YmxpY1JlYWRBY2Nlc3M6IGZhbHNlLCAvLyBXaWxsIGJlIGFjY2Vzc2VkIHZpYSBDbG91ZEZyb250IE9BQ1xuICAgICAgYmxvY2tQdWJsaWNBY2Nlc3M6IHMzLkJsb2NrUHVibGljQWNjZXNzLkJMT0NLX0FMTCxcbiAgICAgIHJlbW92YWxQb2xpY3k6IGNkay5SZW1vdmFsUG9saWN5LkRFU1RST1ksIC8vIEZvciBkZXYvdGVzdGluZ1xuICAgICAgbGlmZWN5Y2xlUnVsZXM6IFtcbiAgICAgICAge1xuICAgICAgICAgIGlkOiAnZGVsZXRlLWluY29tcGxldGUtbXVsdGlwYXJ0LXVwbG9hZHMnLFxuICAgICAgICAgIGFib3J0SW5jb21wbGV0ZU11bHRpcGFydFVwbG9hZEFmdGVyOiBjZGsuRHVyYXRpb24uZGF5cygxKSxcbiAgICAgICAgfSxcbiAgICAgIF0sXG4gICAgfSk7XG5cbiAgICBjb25zdCBwcml2YXRlQnVja2V0ID0gbmV3IHMzLkJ1Y2tldCh0aGlzLCAnUHJpdmF0ZUJ1Y2tldCcsIHtcbiAgICAgIGJ1Y2tldE5hbWU6IGBhcHAtcHJpdmF0ZS0ke2Vudmlyb25tZW50fWAsXG4gICAgICBwdWJsaWNSZWFkQWNjZXNzOiBmYWxzZSxcbiAgICAgIGJsb2NrUHVibGljQWNjZXNzOiBzMy5CbG9ja1B1YmxpY0FjY2Vzcy5CTE9DS19BTEwsXG4gICAgICByZW1vdmFsUG9saWN5OiBjZGsuUmVtb3ZhbFBvbGljeS5ERVNUUk9ZLFxuICAgICAgbGlmZWN5Y2xlUnVsZXM6IFtcbiAgICAgICAge1xuICAgICAgICAgIGlkOiAnbW92ZS1yYXctdXBsb2Fkcy10by1pYScsXG4gICAgICAgICAgdHJhbnNpdGlvbnM6IFtcbiAgICAgICAgICAgIHtcbiAgICAgICAgICAgICAgc3RvcmFnZUNsYXNzOiBzMy5TdG9yYWdlQ2xhc3MuSU5GUkVRVUVOVF9BQ0NFU1MsXG4gICAgICAgICAgICAgIHRyYW5zaXRpb25BZnRlcjogY2RrLkR1cmF0aW9uLmRheXMoMzApLFxuICAgICAgICAgICAgfSxcbiAgICAgICAgICBdLFxuICAgICAgICAgIHByZWZpeDogJ3RlbmFudHMvJyxcbiAgICAgICAgfSxcbiAgICAgICAge1xuICAgICAgICAgIGlkOiAnZGVsZXRlLXRlbXAtZmlsZXMnLFxuICAgICAgICAgIHByZWZpeDogJ3RtcC8nLFxuICAgICAgICAgIGV4cGlyYXRpb246IGNkay5EdXJhdGlvbi5kYXlzKDEpLFxuICAgICAgICB9LFxuICAgICAgICB7XG4gICAgICAgICAgaWQ6ICdkZWxldGUtaWRlbXBvdGVuY3ktbWFya2VycycsXG4gICAgICAgICAgcHJlZml4OiAnaWRlbS8nLFxuICAgICAgICAgIGV4cGlyYXRpb246IGNkay5EdXJhdGlvbi5kYXlzKDEpLFxuICAgICAgICB9LFxuICAgICAgXSxcbiAgICB9KTtcblxuICAgIC8vIERlcGxveSB0ZXN0IG1hbnVmYWN0dXJlciBkYXRhIHRvIHB1YmxpYyBTMyBidWNrZXRcbiAgICBjb25zdCB0ZXN0RGF0YURlcGxveW1lbnQgPSBuZXcgczNkZXBsb3kuQnVja2V0RGVwbG95bWVudCh0aGlzLCAnVGVzdERhdGFEZXBsb3ltZW50Jywge1xuICAgICAgc291cmNlczogW3MzZGVwbG95LlNvdXJjZS5hc3NldCgnLi9kYXRhJyldLFxuICAgICAgZGVzdGluYXRpb25CdWNrZXQ6IHB1YmxpY0J1Y2tldCxcbiAgICAgIGRlc3RpbmF0aW9uS2V5UHJlZml4OiAndGVzdC1kYXRhLycsXG4gICAgICAvLyBSZXBsYWNlIGV4aXN0aW5nIGRhdGEgb24gcmVkZXBsb3lcbiAgICAgIHBydW5lOiB0cnVlLFxuICAgICAgLy8gQ2FjaGUgY29udHJvbCBmb3IgSlNPTiBkYXRhIGZpbGVzXG4gICAgICBjYWNoZUNvbnRyb2w6IFtcbiAgICAgICAgczNkZXBsb3kuQ2FjaGVDb250cm9sLnNldFB1YmxpYygpLFxuICAgICAgICBzM2RlcGxveS5DYWNoZUNvbnRyb2wubWF4QWdlKGNkay5EdXJhdGlvbi5ob3VycygxKSksXG4gICAgICBdLFxuICAgIH0pO1xuXG4gICAgLy8gTGFtYmRhIGV4ZWN1dGlvbiByb2xlXG4gICAgY29uc3QgbGFtYmRhUm9sZSA9IG5ldyBpYW0uUm9sZSh0aGlzLCAnTGFtYmRhUm9sZScsIHtcbiAgICAgIGFzc3VtZWRCeTogbmV3IGlhbS5TZXJ2aWNlUHJpbmNpcGFsKCdsYW1iZGEuYW1hem9uYXdzLmNvbScpLFxuICAgICAgbWFuYWdlZFBvbGljaWVzOiBbXG4gICAgICAgIGlhbS5NYW5hZ2VkUG9saWN5LmZyb21Bd3NNYW5hZ2VkUG9saWN5TmFtZSgnc2VydmljZS1yb2xlL0FXU0xhbWJkYUJhc2ljRXhlY3V0aW9uUm9sZScpLFxuICAgICAgXSxcbiAgICAgIGlubGluZVBvbGljaWVzOiB7XG4gICAgICAgIFMzUG9saWN5OiBuZXcgaWFtLlBvbGljeURvY3VtZW50KHtcbiAgICAgICAgICBzdGF0ZW1lbnRzOiBbXG4gICAgICAgICAgICBuZXcgaWFtLlBvbGljeVN0YXRlbWVudCh7XG4gICAgICAgICAgICAgIGVmZmVjdDogaWFtLkVmZmVjdC5BTExPVyxcbiAgICAgICAgICAgICAgYWN0aW9uczogW1xuICAgICAgICAgICAgICAgICdzMzpHZXRPYmplY3QnLFxuICAgICAgICAgICAgICAgICdzMzpQdXRPYmplY3QnLFxuICAgICAgICAgICAgICAgICdzMzpEZWxldGVPYmplY3QnLFxuICAgICAgICAgICAgICAgICdzMzpMaXN0QnVja2V0JyxcbiAgICAgICAgICAgICAgXSxcbiAgICAgICAgICAgICAgcmVzb3VyY2VzOiBbXG4gICAgICAgICAgICAgICAgcHVibGljQnVja2V0LmJ1Y2tldEFybixcbiAgICAgICAgICAgICAgICBgJHtwdWJsaWNCdWNrZXQuYnVja2V0QXJufS8qYCxcbiAgICAgICAgICAgICAgICBwcml2YXRlQnVja2V0LmJ1Y2tldEFybixcbiAgICAgICAgICAgICAgICBgJHtwcml2YXRlQnVja2V0LmJ1Y2tldEFybn0vKmAsXG4gICAgICAgICAgICAgIF0sXG4gICAgICAgICAgICB9KSxcbiAgICAgICAgICBdLFxuICAgICAgICB9KSxcbiAgICAgICAgU0VTUG9saWN5OiBuZXcgaWFtLlBvbGljeURvY3VtZW50KHtcbiAgICAgICAgICBzdGF0ZW1lbnRzOiBbXG4gICAgICAgICAgICBuZXcgaWFtLlBvbGljeVN0YXRlbWVudCh7XG4gICAgICAgICAgICAgIGVmZmVjdDogaWFtLkVmZmVjdC5BTExPVyxcbiAgICAgICAgICAgICAgYWN0aW9uczogW1xuICAgICAgICAgICAgICAgICdzZXM6U2VuZEVtYWlsJyxcbiAgICAgICAgICAgICAgICAnc2VzOlNlbmRUZW1wbGF0ZWRFbWFpbCcsXG4gICAgICAgICAgICAgICAgJ3NlczpTZW5kUmF3RW1haWwnLFxuICAgICAgICAgICAgICBdLFxuICAgICAgICAgICAgICByZXNvdXJjZXM6IFsnKiddLCAvLyBTRVMgcmVxdWlyZXMgd2lsZGNhcmQgZm9yIHNlbmRpbmcgZW1haWxzXG4gICAgICAgICAgICB9KSxcbiAgICAgICAgICBdLFxuICAgICAgICB9KSxcbiAgICAgIH0sXG4gICAgfSk7XG5cbiAgICAvLyBMYW1iZGEgZnVuY3Rpb25zIChzaW1wbGlmaWVkIGZvciBNVlAgLSB3b3VsZCBidWlsZCBmcm9tIFJ1c3QgYmluYXJpZXMgaW4gcHJvZHVjdGlvbilcbiAgICBjb25zdCBhcGlSZnFzTGFtYmRhID0gbmV3IGxhbWJkYS5GdW5jdGlvbih0aGlzLCAnQXBpUmZxc0xhbWJkYScsIHtcbiAgICAgIHJ1bnRpbWU6IGxhbWJkYS5SdW50aW1lLlBST1ZJREVEX0FMMixcbiAgICAgIGhhbmRsZXI6ICdib290c3RyYXAnLFxuICAgICAgY29kZTogbGFtYmRhLkNvZGUuZnJvbUFzc2V0KCcuLi9iYWNrZW5kL2xhbWJkYXMvYXBpX3JmcXMnKSwgLy8gV291bGQgYmUgYnVpbHQgYmluYXJ5XG4gICAgICByb2xlOiBsYW1iZGFSb2xlLFxuICAgICAgdGltZW91dDogY2RrLkR1cmF0aW9uLnNlY29uZHMoMzApLFxuICAgICAgbWVtb3J5U2l6ZTogNTEyLFxuICAgICAgYXJjaGl0ZWN0dXJlOiBsYW1iZGEuQXJjaGl0ZWN0dXJlLkFSTV82NCxcbiAgICAgIGVudmlyb25tZW50OiB7XG4gICAgICAgIEVOVklST05NRU5UOiBlbnZpcm9ubWVudCxcbiAgICAgICAgUFVCTElDX0JVQ0tFVDogcHVibGljQnVja2V0LmJ1Y2tldE5hbWUsXG4gICAgICAgIFBSSVZBVEVfQlVDS0VUOiBwcml2YXRlQnVja2V0LmJ1Y2tldE5hbWUsXG4gICAgICAgIEZST01fRU1BSUw6ICdub3JlcGx5QHRlcnJhLXBsYXRmb3JtLmNvbScsXG4gICAgICAgIFJVU1RfTE9HOiAnaW5mbycsXG4gICAgICB9LFxuICAgIH0pO1xuXG4gICAgY29uc3QgYXBpVXBsb2Fkc0xhbWJkYSA9IG5ldyBsYW1iZGEuRnVuY3Rpb24odGhpcywgJ0FwaVVwbG9hZHNMYW1iZGEnLCB7XG4gICAgICBydW50aW1lOiBsYW1iZGEuUnVudGltZS5QUk9WSURFRF9BTDIsXG4gICAgICBoYW5kbGVyOiAnYm9vdHN0cmFwJyxcbiAgICAgIGNvZGU6IGxhbWJkYS5Db2RlLmZyb21Bc3NldCgnLi4vYmFja2VuZC9sYW1iZGFzL2FwaV91cGxvYWRzJyksXG4gICAgICByb2xlOiBsYW1iZGFSb2xlLFxuICAgICAgdGltZW91dDogY2RrLkR1cmF0aW9uLnNlY29uZHMoMTApLFxuICAgICAgbWVtb3J5U2l6ZTogMjU2LFxuICAgICAgYXJjaGl0ZWN0dXJlOiBsYW1iZGEuQXJjaGl0ZWN0dXJlLkFSTV82NCxcbiAgICAgIGVudmlyb25tZW50OiB7XG4gICAgICAgIEVOVklST05NRU5UOiBlbnZpcm9ubWVudCxcbiAgICAgICAgUFVCTElDX0JVQ0tFVDogcHVibGljQnVja2V0LmJ1Y2tldE5hbWUsXG4gICAgICAgIFBSSVZBVEVfQlVDS0VUOiBwcml2YXRlQnVja2V0LmJ1Y2tldE5hbWUsXG4gICAgICAgIFJVU1RfTE9HOiAnaW5mbycsXG4gICAgICB9LFxuICAgIH0pO1xuXG4gICAgY29uc3QgYXBpTWFudWZhY3R1cmVyc0xhbWJkYSA9IG5ldyBsYW1iZGEuRnVuY3Rpb24odGhpcywgJ0FwaU1hbnVmYWN0dXJlcnNMYW1iZGEnLCB7XG4gICAgICBydW50aW1lOiBsYW1iZGEuUnVudGltZS5QUk9WSURFRF9BTDIsXG4gICAgICBoYW5kbGVyOiAnYm9vdHN0cmFwJyxcbiAgICAgIGNvZGU6IGxhbWJkYS5Db2RlLmZyb21Bc3NldCgnLi4vYmFja2VuZC9sYW1iZGFzL2FwaV9tYW51ZmFjdHVyZXJzJyksXG4gICAgICByb2xlOiBsYW1iZGFSb2xlLFxuICAgICAgdGltZW91dDogY2RrLkR1cmF0aW9uLnNlY29uZHMoMzApLFxuICAgICAgbWVtb3J5U2l6ZTogMjU2LFxuICAgICAgYXJjaGl0ZWN0dXJlOiBsYW1iZGEuQXJjaGl0ZWN0dXJlLkFSTV82NCxcbiAgICAgIGVudmlyb25tZW50OiB7XG4gICAgICAgIEVOVklST05NRU5UOiBlbnZpcm9ubWVudCxcbiAgICAgICAgUFVCTElDX0JVQ0tFVDogcHVibGljQnVja2V0LmJ1Y2tldE5hbWUsXG4gICAgICAgIFBSSVZBVEVfQlVDS0VUOiBwcml2YXRlQnVja2V0LmJ1Y2tldE5hbWUsXG4gICAgICAgIFJVU1RfTE9HOiAnaW5mbycsXG4gICAgICB9LFxuICAgIH0pO1xuXG4gICAgLy8gUHVibGlzaGVyIExhbWJkYSBmb3IgZ2VuZXJhdGluZyBzdGF0aWMgSFRNTCBhbmQgSlNPTlxuICAgIGNvbnN0IHB1Ymxpc2hlckxhbWJkYSA9IG5ldyBsYW1iZGEuRnVuY3Rpb24odGhpcywgJ1B1Ymxpc2hlckxhbWJkYScsIHtcbiAgICAgIHJ1bnRpbWU6IGxhbWJkYS5SdW50aW1lLlBST1ZJREVEX0FMMixcbiAgICAgIGhhbmRsZXI6ICdib290c3RyYXAnLFxuICAgICAgY29kZTogbGFtYmRhLkNvZGUuZnJvbUFzc2V0KCcuLi9iYWNrZW5kL2xhbWJkYXMvcHVibGlzaGVyJyksXG4gICAgICByb2xlOiBsYW1iZGFSb2xlLFxuICAgICAgdGltZW91dDogY2RrLkR1cmF0aW9uLnNlY29uZHMoNjApLFxuICAgICAgbWVtb3J5U2l6ZTogNTEyLFxuICAgICAgYXJjaGl0ZWN0dXJlOiBsYW1iZGEuQXJjaGl0ZWN0dXJlLkFSTV82NCxcbiAgICAgIGVudmlyb25tZW50OiB7XG4gICAgICAgIEVOVklST05NRU5UOiBlbnZpcm9ubWVudCxcbiAgICAgICAgUFVCTElDX0JVQ0tFVDogcHVibGljQnVja2V0LmJ1Y2tldE5hbWUsXG4gICAgICAgIFBSSVZBVEVfQlVDS0VUOiBwcml2YXRlQnVja2V0LmJ1Y2tldE5hbWUsXG4gICAgICAgIFJVU1RfTE9HOiAnaW5mbycsXG4gICAgICB9LFxuICAgIH0pO1xuXG4gICAgLy8gSW1hZ2UgcHJvY2Vzc2luZyBMYW1iZGFcbiAgICBjb25zdCBpbWFnZUluZ2VzdExhbWJkYSA9IG5ldyBsYW1iZGEuRnVuY3Rpb24odGhpcywgJ0ltYWdlSW5nZXN0TGFtYmRhJywge1xuICAgICAgcnVudGltZTogbGFtYmRhLlJ1bnRpbWUuUFJPVklERURfQUwyLFxuICAgICAgaGFuZGxlcjogJ2Jvb3RzdHJhcCcsXG4gICAgICBjb2RlOiBsYW1iZGEuQ29kZS5mcm9tQXNzZXQoJy4uL2JhY2tlbmQvbGFtYmRhcy9pbWFnZV9pbmdlc3QnKSxcbiAgICAgIHJvbGU6IGxhbWJkYVJvbGUsXG4gICAgICB0aW1lb3V0OiBjZGsuRHVyYXRpb24uc2Vjb25kcygxMjApLFxuICAgICAgbWVtb3J5U2l6ZTogMTAyNCxcbiAgICAgIGFyY2hpdGVjdHVyZTogbGFtYmRhLkFyY2hpdGVjdHVyZS5BUk1fNjQsXG4gICAgICBlbnZpcm9ubWVudDoge1xuICAgICAgICBFTlZJUk9OTUVOVDogZW52aXJvbm1lbnQsXG4gICAgICAgIFBVQkxJQ19CVUNLRVQ6IHB1YmxpY0J1Y2tldC5idWNrZXROYW1lLFxuICAgICAgICBQUklWQVRFX0JVQ0tFVDogcHJpdmF0ZUJ1Y2tldC5idWNrZXROYW1lLFxuICAgICAgICBSVVNUX0xPRzogJ2luZm8nLFxuICAgICAgfSxcbiAgICB9KTtcblxuICAgIC8vIEFQSSBHYXRld2F5IEhUVFAgQVBJIGFzIHNwZWNpZmllZCBpbiBkZXNpZ25cbiAgICBjb25zdCBhcGkgPSBuZXcgYXBpZ2F0ZXdheS5SZXN0QXBpKHRoaXMsICdUZXJyYUFwaScsIHtcbiAgICAgIHJlc3RBcGlOYW1lOiBgdGVycmEtYXBpLSR7ZW52aXJvbm1lbnR9YCxcbiAgICAgIGRlc2NyaXB0aW9uOiAnVGVycmEgTWFudWZhY3R1cmluZyBQbGF0Zm9ybSBBUEknLFxuICAgICAgZGVmYXVsdENvcnNQcmVmbGlnaHRPcHRpb25zOiB7XG4gICAgICAgIGFsbG93T3JpZ2luczogYXBpZ2F0ZXdheS5Db3JzLkFMTF9PUklHSU5TLCAvLyBUT0RPOiBSZXN0cmljdCB0byBHaXRIdWIgUGFnZXMgZG9tYWluIGluIHByb2R1Y3Rpb25cbiAgICAgICAgYWxsb3dNZXRob2RzOiBhcGlnYXRld2F5LkNvcnMuQUxMX01FVEhPRFMsXG4gICAgICAgIGFsbG93SGVhZGVyczogW1xuICAgICAgICAgICdDb250ZW50LVR5cGUnLFxuICAgICAgICAgICdBdXRob3JpemF0aW9uJyxcbiAgICAgICAgICAnSWRlbXBvdGVuY3ktS2V5JyxcbiAgICAgICAgICAnWC1SZXF1ZXN0LUlkJyxcbiAgICAgICAgICAnSWYtTm9uZS1NYXRjaCcsXG4gICAgICAgIF0sXG4gICAgICAgIGV4cG9zZUhlYWRlcnM6IFtcbiAgICAgICAgICAnWC1SZXF1ZXN0LUlkJyxcbiAgICAgICAgICAnRVRhZycsXG4gICAgICAgIF0sXG4gICAgICB9LFxuICAgIH0pO1xuXG4gICAgLy8gQVBJIEdhdGV3YXkgcm91dGVzXG4gICAgY29uc3QgdjEgPSBhcGkucm9vdC5hZGRSZXNvdXJjZSgndjEnKTtcbiAgICBcbiAgICAvLyBSRlEgZW5kcG9pbnRzXG4gICAgY29uc3QgcmZxcyA9IHYxLmFkZFJlc291cmNlKCdyZnFzJyk7XG4gICAgcmZxcy5hZGRNZXRob2QoJ1BPU1QnLCBuZXcgYXBpZ2F0ZXdheS5MYW1iZGFJbnRlZ3JhdGlvbihhcGlSZnFzTGFtYmRhKSk7XG4gICAgXG4gICAgY29uc3QgcmZxQnlJZCA9IHJmcXMuYWRkUmVzb3VyY2UoJ3tpZH0nKTtcbiAgICByZnFCeUlkLmFkZE1ldGhvZCgnR0VUJywgbmV3IGFwaWdhdGV3YXkuTGFtYmRhSW50ZWdyYXRpb24oYXBpUmZxc0xhbWJkYSkpO1xuICAgIFxuICAgIGNvbnN0IHJmcUV2ZW50cyA9IHJmcUJ5SWQuYWRkUmVzb3VyY2UoJ2V2ZW50cycpO1xuICAgIHJmcUV2ZW50cy5hZGRNZXRob2QoJ0dFVCcsIG5ldyBhcGlnYXRld2F5LkxhbWJkYUludGVncmF0aW9uKGFwaVJmcXNMYW1iZGEpKTtcbiAgICBcbiAgICBjb25zdCByZnFNZXNzYWdlcyA9IHJmcUJ5SWQuYWRkUmVzb3VyY2UoJ21lc3NhZ2VzJyk7XG4gICAgcmZxTWVzc2FnZXMuYWRkTWV0aG9kKCdQT1NUJywgbmV3IGFwaWdhdGV3YXkuTGFtYmRhSW50ZWdyYXRpb24oYXBpUmZxc0xhbWJkYSkpO1xuXG4gICAgLy8gVXBsb2FkIGVuZHBvaW50c1xuICAgIGNvbnN0IHVwbG9hZHMgPSB2MS5hZGRSZXNvdXJjZSgndXBsb2FkcycpO1xuICAgIGNvbnN0IHByZXNpZ24gPSB1cGxvYWRzLmFkZFJlc291cmNlKCdwcmVzaWduJyk7XG4gICAgcHJlc2lnbi5hZGRNZXRob2QoJ1BPU1QnLCBuZXcgYXBpZ2F0ZXdheS5MYW1iZGFJbnRlZ3JhdGlvbihhcGlVcGxvYWRzTGFtYmRhKSk7XG5cbiAgICAvLyBNYW51ZmFjdHVyZXIgZW5kcG9pbnRzIChhZG1pbilcbiAgICBjb25zdCBtYW51ZmFjdHVyZXJzID0gdjEuYWRkUmVzb3VyY2UoJ21hbnVmYWN0dXJlcnMnKTtcbiAgICBtYW51ZmFjdHVyZXJzLmFkZE1ldGhvZCgnUE9TVCcsIG5ldyBhcGlnYXRld2F5LkxhbWJkYUludGVncmF0aW9uKGFwaU1hbnVmYWN0dXJlcnNMYW1iZGEpKTtcblxuICAgIC8vIEFkZCBhIG1hbnVhbCB0cmlnZ2VyIGZvciB0aGUgcHVibGlzaGVyIExhbWJkYSAoZm9yIGRlbW8gcHVycG9zZXMpXG4gICAgY29uc3QgcHVibGlzaGVyVHJpZ2dlciA9IG5ldyBsYW1iZGEuRnVuY3Rpb24odGhpcywgJ1B1Ymxpc2hlclRyaWdnZXJMYW1iZGEnLCB7XG4gICAgICBydW50aW1lOiBsYW1iZGEuUnVudGltZS5OT0RFSlNfMThfWCxcbiAgICAgIGhhbmRsZXI6ICdpbmRleC5oYW5kbGVyJyxcbiAgICAgIGNvZGU6IGxhbWJkYS5Db2RlLmZyb21JbmxpbmUoYFxuICAgICAgICBjb25zdCBBV1MgPSByZXF1aXJlKCdhd3Mtc2RrJyk7XG4gICAgICAgIGNvbnN0IGxhbWJkYSA9IG5ldyBBV1MuTGFtYmRhKCk7XG4gICAgICAgIFxuICAgICAgICBleHBvcnRzLmhhbmRsZXIgPSBhc3luYyAoZXZlbnQpID0+IHtcbiAgICAgICAgICB0cnkge1xuICAgICAgICAgICAgY29uc3QgcmVzdWx0ID0gYXdhaXQgbGFtYmRhLmludm9rZSh7XG4gICAgICAgICAgICAgIEZ1bmN0aW9uTmFtZTogJyR7cHVibGlzaGVyTGFtYmRhLmZ1bmN0aW9uTmFtZX0nLFxuICAgICAgICAgICAgICBJbnZvY2F0aW9uVHlwZTogJ0V2ZW50JyxcbiAgICAgICAgICAgICAgUGF5bG9hZDogSlNPTi5zdHJpbmdpZnkoeyB0cmlnZ2VyOiAnbWFudWFsJywgdGltZXN0YW1wOiBuZXcgRGF0ZSgpLnRvSVNPU3RyaW5nKCkgfSlcbiAgICAgICAgICAgIH0pLnByb21pc2UoKTtcbiAgICAgICAgICAgIFxuICAgICAgICAgICAgcmV0dXJuIHtcbiAgICAgICAgICAgICAgc3RhdHVzQ29kZTogMjAwLFxuICAgICAgICAgICAgICBib2R5OiBKU09OLnN0cmluZ2lmeSh7IG1lc3NhZ2U6ICdDYXRhbG9nIHJlYnVpbGQgdHJpZ2dlcmVkJywgcmVzdWx0IH0pXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0gY2F0Y2ggKGVycm9yKSB7XG4gICAgICAgICAgICByZXR1cm4ge1xuICAgICAgICAgICAgICBzdGF0dXNDb2RlOiA1MDAsXG4gICAgICAgICAgICAgIGJvZHk6IEpTT04uc3RyaW5naWZ5KHsgZXJyb3I6IGVycm9yLm1lc3NhZ2UgfSlcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfVxuICAgICAgICB9O1xuICAgICAgYCksXG4gICAgICB0aW1lb3V0OiBjZGsuRHVyYXRpb24uc2Vjb25kcygzMCksXG4gICAgfSk7XG5cbiAgICAvLyBQdWJsaXNoZXIgdHJpZ2dlciBlbmRwb2ludCAoYWRtaW4pXG4gICAgY29uc3QgcHVibGlzaGVyUmVzb3VyY2UgPSB2MS5hZGRSZXNvdXJjZSgncHVibGlzaGVyJyk7XG4gICAgY29uc3QgcHVibGlzaGVyVHJpZ2dlclJlc291cmNlID0gcHVibGlzaGVyUmVzb3VyY2UuYWRkUmVzb3VyY2UoJ3RyaWdnZXInKTtcbiAgICBwdWJsaXNoZXJUcmlnZ2VyUmVzb3VyY2UuYWRkTWV0aG9kKCdQT1NUJywgbmV3IGFwaWdhdGV3YXkuTGFtYmRhSW50ZWdyYXRpb24ocHVibGlzaGVyVHJpZ2dlcikpO1xuXG4gICAgLy8gQ2xvdWRGcm9udCBkaXN0cmlidXRpb24gZm9yIHB1YmxpYyBhc3NldHMgYW5kIHN0YXRpYyBjYXRhbG9nIHBhZ2VzXG4gICAgY29uc3QgZGlzdHJpYnV0aW9uID0gbmV3IGNsb3VkZnJvbnQuRGlzdHJpYnV0aW9uKHRoaXMsICdEaXN0cmlidXRpb24nLCB7XG4gICAgICBkZWZhdWx0QmVoYXZpb3I6IHtcbiAgICAgICAgb3JpZ2luOiBuZXcgb3JpZ2lucy5TM09yaWdpbihwdWJsaWNCdWNrZXQpLFxuICAgICAgICB2aWV3ZXJQcm90b2NvbFBvbGljeTogY2xvdWRmcm9udC5WaWV3ZXJQcm90b2NvbFBvbGljeS5SRURJUkVDVF9UT19IVFRQUyxcbiAgICAgICAgY2FjaGVQb2xpY3k6IGNsb3VkZnJvbnQuQ2FjaGVQb2xpY3kuQ0FDSElOR19PUFRJTUlaRUQsXG4gICAgICAgIGNvbXByZXNzOiB0cnVlLFxuICAgICAgICBhbGxvd2VkTWV0aG9kczogY2xvdWRmcm9udC5BbGxvd2VkTWV0aG9kcy5BTExPV19HRVRfSEVBRCxcbiAgICAgIH0sXG4gICAgICBhZGRpdGlvbmFsQmVoYXZpb3JzOiB7XG4gICAgICAgIC8vIENhdGFsb2cgSlNPTiBmaWxlcyAtIGxvbmcgY2FjaGUgd2l0aCBpbW11dGFibGUgY29udGVudFxuICAgICAgICAnL2NhdGFsb2cvY2F0ZWdvcnkvKic6IHtcbiAgICAgICAgICBvcmlnaW46IG5ldyBvcmlnaW5zLlMzT3JpZ2luKHB1YmxpY0J1Y2tldCksXG4gICAgICAgICAgY2FjaGVQb2xpY3k6IGNsb3VkZnJvbnQuQ2FjaGVQb2xpY3kuQ0FDSElOR19PUFRJTUlaRUQsXG4gICAgICAgICAgdmlld2VyUHJvdG9jb2xQb2xpY3k6IGNsb3VkZnJvbnQuVmlld2VyUHJvdG9jb2xQb2xpY3kuUkVESVJFQ1RfVE9fSFRUUFMsXG4gICAgICAgICAgcmVzcG9uc2VIZWFkZXJzUG9saWN5OiBjbG91ZGZyb250LlJlc3BvbnNlSGVhZGVyc1BvbGljeS5DT1JTX0FMTE9XX0FMTF9PUklHSU5TLFxuICAgICAgICB9LFxuICAgICAgICAnL21hbnVmYWN0dXJlci8qJzoge1xuICAgICAgICAgIG9yaWdpbjogbmV3IG9yaWdpbnMuUzNPcmlnaW4ocHVibGljQnVja2V0KSxcbiAgICAgICAgICBjYWNoZVBvbGljeTogY2xvdWRmcm9udC5DYWNoZVBvbGljeS5DQUNISU5HX09QVElNSVpFRCxcbiAgICAgICAgICB2aWV3ZXJQcm90b2NvbFBvbGljeTogY2xvdWRmcm9udC5WaWV3ZXJQcm90b2NvbFBvbGljeS5SRURJUkVDVF9UT19IVFRQUyxcbiAgICAgICAgICByZXNwb25zZUhlYWRlcnNQb2xpY3k6IGNsb3VkZnJvbnQuUmVzcG9uc2VIZWFkZXJzUG9saWN5LkNPUlNfQUxMT1dfQUxMX09SSUdJTlMsXG4gICAgICAgIH0sXG4gICAgICAgIC8vIFN0YXRpYyBIVE1MIGNhdGFsb2cgcGFnZXMgLSBzaG9ydGVyIGNhY2hlIGZvciBmYXN0ZXIgdXBkYXRlc1xuICAgICAgICAnL2NhdGFsb2cvKic6IHtcbiAgICAgICAgICBvcmlnaW46IG5ldyBvcmlnaW5zLlMzT3JpZ2luKHB1YmxpY0J1Y2tldCksXG4gICAgICAgICAgY2FjaGVQb2xpY3k6IG5ldyBjbG91ZGZyb250LkNhY2hlUG9saWN5KHRoaXMsICdDYXRhbG9nSHRtbENhY2hlUG9saWN5Jywge1xuICAgICAgICAgICAgY2FjaGVQb2xpY3lOYW1lOiBgdGVycmEtY2F0YWxvZy1odG1sLSR7ZW52aXJvbm1lbnR9YCxcbiAgICAgICAgICAgIGRlZmF1bHRUdGw6IGNkay5EdXJhdGlvbi5taW51dGVzKDUpLFxuICAgICAgICAgICAgbWF4VHRsOiBjZGsuRHVyYXRpb24uaG91cnMoMSksXG4gICAgICAgICAgICBtaW5UdGw6IGNkay5EdXJhdGlvbi5zZWNvbmRzKDApLFxuICAgICAgICAgIH0pLFxuICAgICAgICAgIHZpZXdlclByb3RvY29sUG9saWN5OiBjbG91ZGZyb250LlZpZXdlclByb3RvY29sUG9saWN5LlJFRElSRUNUX1RPX0hUVFBTLFxuICAgICAgICAgIGNvbXByZXNzOiB0cnVlLFxuICAgICAgICB9LFxuICAgICAgICAvLyBDU1MgYW5kIG90aGVyIGFzc2V0c1xuICAgICAgICAnL3NyYy8qJzoge1xuICAgICAgICAgIG9yaWdpbjogbmV3IG9yaWdpbnMuUzNPcmlnaW4ocHVibGljQnVja2V0KSxcbiAgICAgICAgICBjYWNoZVBvbGljeTogY2xvdWRmcm9udC5DYWNoZVBvbGljeS5DQUNISU5HX09QVElNSVpFRCxcbiAgICAgICAgICB2aWV3ZXJQcm90b2NvbFBvbGljeTogY2xvdWRmcm9udC5WaWV3ZXJQcm90b2NvbFBvbGljeS5SRURJUkVDVF9UT19IVFRQUyxcbiAgICAgICAgICBjb21wcmVzczogdHJ1ZSxcbiAgICAgICAgfSxcbiAgICAgICAgLy8gSW1hZ2VzIGFuZCBkZXJpdmVkIGFzc2V0c1xuICAgICAgICAnL3RlbmFudHMvKic6IHtcbiAgICAgICAgICBvcmlnaW46IG5ldyBvcmlnaW5zLlMzT3JpZ2luKHB1YmxpY0J1Y2tldCksXG4gICAgICAgICAgY2FjaGVQb2xpY3k6IGNsb3VkZnJvbnQuQ2FjaGVQb2xpY3kuQ0FDSElOR19PUFRJTUlaRUQsXG4gICAgICAgICAgdmlld2VyUHJvdG9jb2xQb2xpY3k6IGNsb3VkZnJvbnQuVmlld2VyUHJvdG9jb2xQb2xpY3kuUkVESVJFQ1RfVE9fSFRUUFMsXG4gICAgICAgIH0sXG4gICAgICB9LFxuICAgIH0pO1xuXG4gICAgLy8gR3JhbnQgcGVybWlzc2lvbnMgZm9yIHRoZSB0cmlnZ2VyIHRvIGludm9rZSB0aGUgcHVibGlzaGVyXG4gICAgcHVibGlzaGVyTGFtYmRhLmdyYW50SW52b2tlKHB1Ymxpc2hlclRyaWdnZXIpO1xuXG4gICAgLy8gT3V0cHV0c1xuICAgIG5ldyBjZGsuQ2ZuT3V0cHV0KHRoaXMsICdBcGlVcmwnLCB7XG4gICAgICB2YWx1ZTogYXBpLnVybCxcbiAgICAgIGRlc2NyaXB0aW9uOiAnQVBJIEdhdGV3YXkgVVJMJyxcbiAgICB9KTtcblxuICAgIG5ldyBjZGsuQ2ZuT3V0cHV0KHRoaXMsICdEaXN0cmlidXRpb25VcmwnLCB7XG4gICAgICB2YWx1ZTogYGh0dHBzOi8vJHtkaXN0cmlidXRpb24uZGlzdHJpYnV0aW9uRG9tYWluTmFtZX1gLFxuICAgICAgZGVzY3JpcHRpb246ICdDbG91ZEZyb250IERpc3RyaWJ1dGlvbiBVUkwnLFxuICAgIH0pO1xuXG4gICAgbmV3IGNkay5DZm5PdXRwdXQodGhpcywgJ1B1YmxpY0J1Y2tldE5hbWUnLCB7XG4gICAgICB2YWx1ZTogcHVibGljQnVja2V0LmJ1Y2tldE5hbWUsXG4gICAgICBkZXNjcmlwdGlvbjogJ1B1YmxpYyBTMyBCdWNrZXQgTmFtZScsXG4gICAgfSk7XG5cbiAgICBuZXcgY2RrLkNmbk91dHB1dCh0aGlzLCAnUHJpdmF0ZUJ1Y2tldE5hbWUnLCB7XG4gICAgICB2YWx1ZTogcHJpdmF0ZUJ1Y2tldC5idWNrZXROYW1lLFxuICAgICAgZGVzY3JpcHRpb246ICdQcml2YXRlIFMzIEJ1Y2tldCBOYW1lJyxcbiAgICB9KTtcbiAgfVxufSJdfQ==