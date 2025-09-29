# CDK Deployment Setup

This document explains how to set up automated CDK deployment for the Terra platform.

## Overview

The `deploy-cdk.yml` workflow provides automated infrastructure deployment with the following features:

- **Validation First**: Runs comprehensive validation before deployment
- **Mainline Only**: Only deploys from the `main` branch
- **Change Detection**: Only runs when infrastructure files change
- **Secure Authentication**: Uses OIDC for AWS authentication
- **Comprehensive Checks**: Validates CDK build, synthesis, and Lambda compilation

## Prerequisites

### AWS Account Setup

1. **Create an OIDC Identity Provider** in your AWS account:
   - Provider URL: `https://token.actions.githubusercontent.com`
   - Audience: `sts.amazonaws.com`

2. **Create an IAM Role** for GitHub Actions with the following trust policy:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": {
        "Federated": "arn:aws:iam::YOUR-ACCOUNT-ID:oidc-provider/token.actions.githubusercontent.com"
      },
      "Action": "sts:AssumeRoleWithWebIdentity",
      "Condition": {
        "StringEquals": {
          "token.actions.githubusercontent.com:aud": "sts.amazonaws.com"
        },
        "StringLike": {
          "token.actions.githubusercontent.com:sub": "repo:JPrier/Terra:ref:refs/heads/main"
        }
      }
    }
  ]
}
```

3. **Attach Required Policies** to the role:
   - `PowerUserAccess` (or more restrictive policies as needed)
   - Custom policy for CDK operations

### GitHub Repository Setup

1. **Add Repository Secrets**:
   - `AWS_ROLE_TO_ASSUME`: The ARN of the IAM role created above
   - `AWS_ACCOUNT_ID`: Your AWS account ID (12-digit number)

2. **Configure Environment Protection** (recommended):
   - Go to Settings > Environments
   - Create a `production` environment
   - Add protection rules (e.g., required reviewers)

## Workflow Triggers

The deployment workflow triggers on:

- **Push to main branch**: Automatic deployment when infrastructure changes
- **Manual dispatch**: Can be triggered manually via GitHub Actions UI

## Workflow Stages

### 1. Validation Stage
- ✅ Detects changes to infrastructure files
- ✅ Builds CDK TypeScript code
- ✅ Runs CDK tests (if present)
- ✅ Validates CDK synthesis
- ✅ Checks Lambda function compilation

### 2. Deployment Stage (only if validation passes)
- ✅ Configures AWS credentials via OIDC
- ✅ Bootstraps CDK environment (if needed)
- ✅ Deploys infrastructure stack
- ✅ Reports deployment status

### 3. Notification Stage
- ✅ Reports success, failure, or skip status

## Change Detection

The workflow uses path filtering to only run when these files change:
- `infra/**` - Any CDK infrastructure files
- `.github/workflows/deploy-cdk.yml` - The workflow itself

## Security Features

- **OIDC Authentication**: No long-lived AWS credentials stored
- **Mainline Only**: Only deploys from the main branch
- **Environment Protection**: Uses GitHub environment protection
- **Least Privilege**: Role permissions can be scoped as needed

## Local Development

To test CDK changes locally:

```bash
cd infra
npm install
npm run build
npm run synth
npm run deploy  # Only if you have AWS credentials configured
```

## Troubleshooting

### Common Issues

1. **OIDC Authentication Fails**:
   - Verify the IAM role trust policy
   - Check that `AWS_ROLE_TO_ASSUME` secret is set correctly

2. **CDK Bootstrap Required**:
   - The workflow will automatically bootstrap if needed
   - For manual bootstrap: `cdk bootstrap`

3. **Permission Denied**:
   - Review IAM role permissions
   - Ensure the role has necessary CDK and service permissions

### Monitoring

- Check GitHub Actions logs for detailed output
- AWS CloudTrail for API activity
- CloudFormation console for stack status

## Customization

To customize the deployment:

1. **Change Environment**: Modify the `environment` in the deploy job
2. **Add Tests**: Add CDK unit tests in the `infra/test/` directory
3. **Notifications**: Add Slack/email notifications in the notify job
4. **Multi-Stage**: Add staging environment deployment

## Manual Deployment

For manual deployment outside GitHub Actions:

```bash
# Set environment variables
export CDK_DEFAULT_ACCOUNT=123456789012
export CDK_DEFAULT_REGION=us-east-1

# Deploy
cd infra
npm run deploy
```