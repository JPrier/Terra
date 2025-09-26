import * as cdk from 'aws-cdk-lib';
import * as s3 from 'aws-cdk-lib/aws-s3';
import * as apigateway from 'aws-cdk-lib/aws-apigateway';
import * as lambda from 'aws-cdk-lib/aws-lambda';
import * as iam from 'aws-cdk-lib/aws-iam';
import * as ses from 'aws-cdk-lib/aws-ses';
import * as cloudfront from 'aws-cdk-lib/aws-cloudfront';
import * as origins from 'aws-cdk-lib/aws-cloudfront-origins';
import { Construct } from 'constructs';

export class TerraStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const environment = this.node.tryGetContext('environment') || 'dev';

    // S3 Buckets as per design
    const publicBucket = new s3.Bucket(this, 'PublicBucket', {
      bucketName: `app-public-${environment}`,
      publicReadAccess: false, // Will be accessed via CloudFront OAC
      blockPublicAccess: s3.BlockPublicAccess.BLOCK_ALL,
      removalPolicy: cdk.RemovalPolicy.DESTROY, // For dev/testing
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
      code: lambda.Code.fromAsset('../backend/lambdas/api_rfqs'), // Would be built binary
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
        allowOrigins: apigateway.Cors.ALL_ORIGINS, // TODO: Restrict to GitHub Pages domain in production
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

    new cdk.CfnOutput(this, 'PublicBucket', {
      value: publicBucket.bucketName,
      description: 'Public S3 Bucket Name',
    });

    new cdk.CfnOutput(this, 'PrivateBucket', {
      value: privateBucket.bucketName,
      description: 'Private S3 Bucket Name',
    });
  }
}