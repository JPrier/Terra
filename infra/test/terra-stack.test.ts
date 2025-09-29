import { TerraStack } from '../lib/terra-stack';
import { App } from 'aws-cdk-lib';
import { Template } from 'aws-cdk-lib/assertions';

describe('TerraStack', () => {
  test('Stack creates S3 buckets', () => {
    const app = new App();
    const stack = new TerraStack(app, 'TestStack', {
      env: {
        account: '123456789012',
        region: 'us-east-1',
      },
    });

    const template = Template.fromStack(stack);

    // Verify public bucket exists
    template.hasResourceProperties('AWS::S3::Bucket', {
      BucketName: 'app-public-dev',
    });

    // Verify private bucket exists
    template.hasResourceProperties('AWS::S3::Bucket', {
      BucketName: 'app-private-dev',
    });
  });

  test('Stack creates API Gateway', () => {
    const app = new App();
    const stack = new TerraStack(app, 'TestStack', {
      env: {
        account: '123456789012',
        region: 'us-east-1',
      },
    });

    const template = Template.fromStack(stack);

    // Verify API Gateway exists
    template.hasResourceProperties('AWS::ApiGateway::RestApi', {
      Name: 'terra-api-dev',
    });
  });

  test('Stack creates CloudFront distribution', () => {
    const app = new App();
    const stack = new TerraStack(app, 'TestStack', {
      env: {
        account: '123456789012',
        region: 'us-east-1',
      },
    });

    const template = Template.fromStack(stack);

    // Verify CloudFront distribution exists
    template.hasResourceProperties('AWS::CloudFront::Distribution', {
      DistributionConfig: {
        Enabled: true,
      },
    });
  });

  test('Stack has required outputs', () => {
    const app = new App();
    const stack = new TerraStack(app, 'TestStack', {
      env: {
        account: '123456789012',
        region: 'us-east-1',
      },
    });

    const template = Template.fromStack(stack);

    // Verify required outputs exist
    template.hasOutput('ApiUrl', {});
    template.hasOutput('DistributionUrl', {});
    template.hasOutput('PublicBucketName', {});
    template.hasOutput('PrivateBucketName', {});
  });
});