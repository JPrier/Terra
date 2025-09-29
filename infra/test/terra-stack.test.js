"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const terra_stack_1 = require("../lib/terra-stack");
const aws_cdk_lib_1 = require("aws-cdk-lib");
const assertions_1 = require("aws-cdk-lib/assertions");
describe('TerraStack', () => {
    test('Stack creates S3 buckets', () => {
        const app = new aws_cdk_lib_1.App();
        const stack = new terra_stack_1.TerraStack(app, 'TestStack', {
            env: {
                account: '123456789012',
                region: 'us-east-1',
            },
        });
        const template = assertions_1.Template.fromStack(stack);
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
        const app = new aws_cdk_lib_1.App();
        const stack = new terra_stack_1.TerraStack(app, 'TestStack', {
            env: {
                account: '123456789012',
                region: 'us-east-1',
            },
        });
        const template = assertions_1.Template.fromStack(stack);
        // Verify API Gateway exists
        template.hasResourceProperties('AWS::ApiGateway::RestApi', {
            Name: 'terra-api-dev',
        });
    });
    test('Stack creates CloudFront distribution', () => {
        const app = new aws_cdk_lib_1.App();
        const stack = new terra_stack_1.TerraStack(app, 'TestStack', {
            env: {
                account: '123456789012',
                region: 'us-east-1',
            },
        });
        const template = assertions_1.Template.fromStack(stack);
        // Verify CloudFront distribution exists
        template.hasResourceProperties('AWS::CloudFront::Distribution', {
            DistributionConfig: {
                Enabled: true,
            },
        });
    });
    test('Stack has required outputs', () => {
        const app = new aws_cdk_lib_1.App();
        const stack = new terra_stack_1.TerraStack(app, 'TestStack', {
            env: {
                account: '123456789012',
                region: 'us-east-1',
            },
        });
        const template = assertions_1.Template.fromStack(stack);
        // Verify required outputs exist
        template.hasOutput('ApiUrl', {});
        template.hasOutput('DistributionUrl', {});
        template.hasOutput('PublicBucketName', {});
        template.hasOutput('PrivateBucketName', {});
    });
});
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoidGVycmEtc3RhY2sudGVzdC5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbInRlcnJhLXN0YWNrLnRlc3QudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6Ijs7QUFBQSxvREFBZ0Q7QUFDaEQsNkNBQWtDO0FBQ2xDLHVEQUFrRDtBQUVsRCxRQUFRLENBQUMsWUFBWSxFQUFFLEdBQUcsRUFBRTtJQUMxQixJQUFJLENBQUMsMEJBQTBCLEVBQUUsR0FBRyxFQUFFO1FBQ3BDLE1BQU0sR0FBRyxHQUFHLElBQUksaUJBQUcsRUFBRSxDQUFDO1FBQ3RCLE1BQU0sS0FBSyxHQUFHLElBQUksd0JBQVUsQ0FBQyxHQUFHLEVBQUUsV0FBVyxFQUFFO1lBQzdDLEdBQUcsRUFBRTtnQkFDSCxPQUFPLEVBQUUsY0FBYztnQkFDdkIsTUFBTSxFQUFFLFdBQVc7YUFDcEI7U0FDRixDQUFDLENBQUM7UUFFSCxNQUFNLFFBQVEsR0FBRyxxQkFBUSxDQUFDLFNBQVMsQ0FBQyxLQUFLLENBQUMsQ0FBQztRQUUzQyw4QkFBOEI7UUFDOUIsUUFBUSxDQUFDLHFCQUFxQixDQUFDLGlCQUFpQixFQUFFO1lBQ2hELFVBQVUsRUFBRSxnQkFBZ0I7U0FDN0IsQ0FBQyxDQUFDO1FBRUgsK0JBQStCO1FBQy9CLFFBQVEsQ0FBQyxxQkFBcUIsQ0FBQyxpQkFBaUIsRUFBRTtZQUNoRCxVQUFVLEVBQUUsaUJBQWlCO1NBQzlCLENBQUMsQ0FBQztJQUNMLENBQUMsQ0FBQyxDQUFDO0lBRUgsSUFBSSxDQUFDLDJCQUEyQixFQUFFLEdBQUcsRUFBRTtRQUNyQyxNQUFNLEdBQUcsR0FBRyxJQUFJLGlCQUFHLEVBQUUsQ0FBQztRQUN0QixNQUFNLEtBQUssR0FBRyxJQUFJLHdCQUFVLENBQUMsR0FBRyxFQUFFLFdBQVcsRUFBRTtZQUM3QyxHQUFHLEVBQUU7Z0JBQ0gsT0FBTyxFQUFFLGNBQWM7Z0JBQ3ZCLE1BQU0sRUFBRSxXQUFXO2FBQ3BCO1NBQ0YsQ0FBQyxDQUFDO1FBRUgsTUFBTSxRQUFRLEdBQUcscUJBQVEsQ0FBQyxTQUFTLENBQUMsS0FBSyxDQUFDLENBQUM7UUFFM0MsNEJBQTRCO1FBQzVCLFFBQVEsQ0FBQyxxQkFBcUIsQ0FBQywwQkFBMEIsRUFBRTtZQUN6RCxJQUFJLEVBQUUsZUFBZTtTQUN0QixDQUFDLENBQUM7SUFDTCxDQUFDLENBQUMsQ0FBQztJQUVILElBQUksQ0FBQyx1Q0FBdUMsRUFBRSxHQUFHLEVBQUU7UUFDakQsTUFBTSxHQUFHLEdBQUcsSUFBSSxpQkFBRyxFQUFFLENBQUM7UUFDdEIsTUFBTSxLQUFLLEdBQUcsSUFBSSx3QkFBVSxDQUFDLEdBQUcsRUFBRSxXQUFXLEVBQUU7WUFDN0MsR0FBRyxFQUFFO2dCQUNILE9BQU8sRUFBRSxjQUFjO2dCQUN2QixNQUFNLEVBQUUsV0FBVzthQUNwQjtTQUNGLENBQUMsQ0FBQztRQUVILE1BQU0sUUFBUSxHQUFHLHFCQUFRLENBQUMsU0FBUyxDQUFDLEtBQUssQ0FBQyxDQUFDO1FBRTNDLHdDQUF3QztRQUN4QyxRQUFRLENBQUMscUJBQXFCLENBQUMsK0JBQStCLEVBQUU7WUFDOUQsa0JBQWtCLEVBQUU7Z0JBQ2xCLE9BQU8sRUFBRSxJQUFJO2FBQ2Q7U0FDRixDQUFDLENBQUM7SUFDTCxDQUFDLENBQUMsQ0FBQztJQUVILElBQUksQ0FBQyw0QkFBNEIsRUFBRSxHQUFHLEVBQUU7UUFDdEMsTUFBTSxHQUFHLEdBQUcsSUFBSSxpQkFBRyxFQUFFLENBQUM7UUFDdEIsTUFBTSxLQUFLLEdBQUcsSUFBSSx3QkFBVSxDQUFDLEdBQUcsRUFBRSxXQUFXLEVBQUU7WUFDN0MsR0FBRyxFQUFFO2dCQUNILE9BQU8sRUFBRSxjQUFjO2dCQUN2QixNQUFNLEVBQUUsV0FBVzthQUNwQjtTQUNGLENBQUMsQ0FBQztRQUVILE1BQU0sUUFBUSxHQUFHLHFCQUFRLENBQUMsU0FBUyxDQUFDLEtBQUssQ0FBQyxDQUFDO1FBRTNDLGdDQUFnQztRQUNoQyxRQUFRLENBQUMsU0FBUyxDQUFDLFFBQVEsRUFBRSxFQUFFLENBQUMsQ0FBQztRQUNqQyxRQUFRLENBQUMsU0FBUyxDQUFDLGlCQUFpQixFQUFFLEVBQUUsQ0FBQyxDQUFDO1FBQzFDLFFBQVEsQ0FBQyxTQUFTLENBQUMsa0JBQWtCLEVBQUUsRUFBRSxDQUFDLENBQUM7UUFDM0MsUUFBUSxDQUFDLFNBQVMsQ0FBQyxtQkFBbUIsRUFBRSxFQUFFLENBQUMsQ0FBQztJQUM5QyxDQUFDLENBQUMsQ0FBQztBQUNMLENBQUMsQ0FBQyxDQUFDIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IHsgVGVycmFTdGFjayB9IGZyb20gJy4uL2xpYi90ZXJyYS1zdGFjayc7XG5pbXBvcnQgeyBBcHAgfSBmcm9tICdhd3MtY2RrLWxpYic7XG5pbXBvcnQgeyBUZW1wbGF0ZSB9IGZyb20gJ2F3cy1jZGstbGliL2Fzc2VydGlvbnMnO1xuXG5kZXNjcmliZSgnVGVycmFTdGFjaycsICgpID0+IHtcbiAgdGVzdCgnU3RhY2sgY3JlYXRlcyBTMyBidWNrZXRzJywgKCkgPT4ge1xuICAgIGNvbnN0IGFwcCA9IG5ldyBBcHAoKTtcbiAgICBjb25zdCBzdGFjayA9IG5ldyBUZXJyYVN0YWNrKGFwcCwgJ1Rlc3RTdGFjaycsIHtcbiAgICAgIGVudjoge1xuICAgICAgICBhY2NvdW50OiAnMTIzNDU2Nzg5MDEyJyxcbiAgICAgICAgcmVnaW9uOiAndXMtZWFzdC0xJyxcbiAgICAgIH0sXG4gICAgfSk7XG5cbiAgICBjb25zdCB0ZW1wbGF0ZSA9IFRlbXBsYXRlLmZyb21TdGFjayhzdGFjayk7XG5cbiAgICAvLyBWZXJpZnkgcHVibGljIGJ1Y2tldCBleGlzdHNcbiAgICB0ZW1wbGF0ZS5oYXNSZXNvdXJjZVByb3BlcnRpZXMoJ0FXUzo6UzM6OkJ1Y2tldCcsIHtcbiAgICAgIEJ1Y2tldE5hbWU6ICdhcHAtcHVibGljLWRldicsXG4gICAgfSk7XG5cbiAgICAvLyBWZXJpZnkgcHJpdmF0ZSBidWNrZXQgZXhpc3RzXG4gICAgdGVtcGxhdGUuaGFzUmVzb3VyY2VQcm9wZXJ0aWVzKCdBV1M6OlMzOjpCdWNrZXQnLCB7XG4gICAgICBCdWNrZXROYW1lOiAnYXBwLXByaXZhdGUtZGV2JyxcbiAgICB9KTtcbiAgfSk7XG5cbiAgdGVzdCgnU3RhY2sgY3JlYXRlcyBBUEkgR2F0ZXdheScsICgpID0+IHtcbiAgICBjb25zdCBhcHAgPSBuZXcgQXBwKCk7XG4gICAgY29uc3Qgc3RhY2sgPSBuZXcgVGVycmFTdGFjayhhcHAsICdUZXN0U3RhY2snLCB7XG4gICAgICBlbnY6IHtcbiAgICAgICAgYWNjb3VudDogJzEyMzQ1Njc4OTAxMicsXG4gICAgICAgIHJlZ2lvbjogJ3VzLWVhc3QtMScsXG4gICAgICB9LFxuICAgIH0pO1xuXG4gICAgY29uc3QgdGVtcGxhdGUgPSBUZW1wbGF0ZS5mcm9tU3RhY2soc3RhY2spO1xuXG4gICAgLy8gVmVyaWZ5IEFQSSBHYXRld2F5IGV4aXN0c1xuICAgIHRlbXBsYXRlLmhhc1Jlc291cmNlUHJvcGVydGllcygnQVdTOjpBcGlHYXRld2F5OjpSZXN0QXBpJywge1xuICAgICAgTmFtZTogJ3RlcnJhLWFwaS1kZXYnLFxuICAgIH0pO1xuICB9KTtcblxuICB0ZXN0KCdTdGFjayBjcmVhdGVzIENsb3VkRnJvbnQgZGlzdHJpYnV0aW9uJywgKCkgPT4ge1xuICAgIGNvbnN0IGFwcCA9IG5ldyBBcHAoKTtcbiAgICBjb25zdCBzdGFjayA9IG5ldyBUZXJyYVN0YWNrKGFwcCwgJ1Rlc3RTdGFjaycsIHtcbiAgICAgIGVudjoge1xuICAgICAgICBhY2NvdW50OiAnMTIzNDU2Nzg5MDEyJyxcbiAgICAgICAgcmVnaW9uOiAndXMtZWFzdC0xJyxcbiAgICAgIH0sXG4gICAgfSk7XG5cbiAgICBjb25zdCB0ZW1wbGF0ZSA9IFRlbXBsYXRlLmZyb21TdGFjayhzdGFjayk7XG5cbiAgICAvLyBWZXJpZnkgQ2xvdWRGcm9udCBkaXN0cmlidXRpb24gZXhpc3RzXG4gICAgdGVtcGxhdGUuaGFzUmVzb3VyY2VQcm9wZXJ0aWVzKCdBV1M6OkNsb3VkRnJvbnQ6OkRpc3RyaWJ1dGlvbicsIHtcbiAgICAgIERpc3RyaWJ1dGlvbkNvbmZpZzoge1xuICAgICAgICBFbmFibGVkOiB0cnVlLFxuICAgICAgfSxcbiAgICB9KTtcbiAgfSk7XG5cbiAgdGVzdCgnU3RhY2sgaGFzIHJlcXVpcmVkIG91dHB1dHMnLCAoKSA9PiB7XG4gICAgY29uc3QgYXBwID0gbmV3IEFwcCgpO1xuICAgIGNvbnN0IHN0YWNrID0gbmV3IFRlcnJhU3RhY2soYXBwLCAnVGVzdFN0YWNrJywge1xuICAgICAgZW52OiB7XG4gICAgICAgIGFjY291bnQ6ICcxMjM0NTY3ODkwMTInLFxuICAgICAgICByZWdpb246ICd1cy1lYXN0LTEnLFxuICAgICAgfSxcbiAgICB9KTtcblxuICAgIGNvbnN0IHRlbXBsYXRlID0gVGVtcGxhdGUuZnJvbVN0YWNrKHN0YWNrKTtcblxuICAgIC8vIFZlcmlmeSByZXF1aXJlZCBvdXRwdXRzIGV4aXN0XG4gICAgdGVtcGxhdGUuaGFzT3V0cHV0KCdBcGlVcmwnLCB7fSk7XG4gICAgdGVtcGxhdGUuaGFzT3V0cHV0KCdEaXN0cmlidXRpb25VcmwnLCB7fSk7XG4gICAgdGVtcGxhdGUuaGFzT3V0cHV0KCdQdWJsaWNCdWNrZXROYW1lJywge30pO1xuICAgIHRlbXBsYXRlLmhhc091dHB1dCgnUHJpdmF0ZUJ1Y2tldE5hbWUnLCB7fSk7XG4gIH0pO1xufSk7Il19