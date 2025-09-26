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

    // CloudFront distribution for public assets
    const distribution = new cloudfront.Distribution(this, 'Distribution', {
      defaultBehavior: {
        origin: new origins.S3Origin(publicBucket),
        viewerProtocolPolicy: cloudfront.ViewerProtocolPolicy.REDIRECT_TO_HTTPS,
        cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
        compress: true,
        allowedMethods: cloudfront.AllowedMethods.ALLOW_GET_HEAD,
      },
      additionalBehaviors: {
        '/catalog/*': {
          origin: new origins.S3Origin(publicBucket),
          cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
          viewerProtocolPolicy: cloudfront.ViewerProtocolPolicy.REDIRECT_TO_HTTPS,
        },
      },
    });

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