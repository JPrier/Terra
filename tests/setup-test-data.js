#!/usr/bin/env node
/**
 * Setup script to initialize test data in LocalStack
 */

import axios from 'axios';

const LOCALSTACK_URL = process.env.LOCALSTACK_URL || 'http://localhost:4566';
const AWS_ACCESS_KEY_ID = 'test';
const AWS_SECRET_ACCESS_KEY = 'test';
const AWS_REGION = 'us-east-1';

const PUBLIC_BUCKET = 'app-public-test';
const PRIVATE_BUCKET = 'app-private-test';

// Sample manufacturer data
const sampleManufacturers = [
  {
    id: 'precision-parts-co',
    name: 'Precision Parts Co',
    city: 'Detroit', 
    state: 'MI',
    categories: ['CNC Machining', 'Precision Manufacturing'],
    capabilities: ['5-axis CNC', 'Aluminum', 'Steel', 'Prototype to Production'],
    description: 'Family-owned precision machining company specializing in aerospace and medical components. ISO 9001 certified with over 30 years of experience.',
    contact_email: 'quotes@precision-parts.com'
  },
  {
    id: 'midwest-machining', 
    name: 'Midwest Machining Solutions',
    city: 'Chicago',
    state: 'IL',
    categories: ['CNC Machining', 'Turning', 'Milling'],
    capabilities: ['CNC Turning', 'CNC Milling', 'Aerospace Grade', 'ISO 9001'],
    description: 'Leading machining company serving automotive and aerospace industries.',
    contact_email: 'info@midwest-machining.com'
  },
  {
    id: 'advanced-plastics-inc',
    name: 'Advanced Plastics Inc',
    city: 'Austin',
    state: 'TX', 
    categories: ['Injection Molding', 'Plastics'],
    capabilities: ['Injection Molding', 'Overmolding', 'Medical Grade', 'Food Safe'],
    description: 'Leading plastic injection molding company serving automotive and consumer electronics industries.',
    contact_email: 'info@advancedplastics.com'
  }
];

async function createS3Buckets() {
  console.log('Creating S3 buckets...');
  
  try {
    // Create public bucket
    await axios.put(`${LOCALSTACK_URL}/${PUBLIC_BUCKET}`, null, {
      headers: {
        'Authorization': `AWS4-HMAC-SHA256 Credential=${AWS_ACCESS_KEY_ID}/${new Date().toISOString().split('T')[0]}/${AWS_REGION}/s3/aws4_request`,
      }
    });
    
    // Create private bucket  
    await axios.put(`${LOCALSTACK_URL}/${PRIVATE_BUCKET}`, null, {
      headers: {
        'Authorization': `AWS4-HMAC-SHA256 Credential=${AWS_ACCESS_KEY_ID}/${new Date().toISOString().split('T')[0]}/${AWS_REGION}/s3/aws4_request`,
      }
    });
    
    console.log('✅ S3 buckets created successfully');
  } catch (error) {
    console.error('❌ Failed to create S3 buckets:', error.message);
  }
}

async function createTestManufacturers() {
  console.log('Creating test manufacturers...');
  
  for (const manufacturer of sampleManufacturers) {
    try {
      // Store manufacturer profile in public bucket
      const manufacturerKey = `manufacturer/${manufacturer.id}.json`;
      await axios.put(`${LOCALSTACK_URL}/${PUBLIC_BUCKET}/${manufacturerKey}`, 
        JSON.stringify(manufacturer), {
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `AWS4-HMAC-SHA256 Credential=${AWS_ACCESS_KEY_ID}/${new Date().toISOString().split('T')[0]}/${AWS_REGION}/s3/aws4_request`,
        }
      });
      
      console.log(`✅ Created manufacturer: ${manufacturer.name}`);
    } catch (error) {
      console.error(`❌ Failed to create manufacturer ${manufacturer.name}:`, error.message);
    }
  }
}

async function createCatalogPages() {
  console.log('Creating catalog pages...');
  
  const categories = {
    'machining': sampleManufacturers.filter(m => m.categories.includes('CNC Machining')),
    'molding': sampleManufacturers.filter(m => m.categories.includes('Injection Molding')),
    'fabrication': [] // Empty for testing
  };
  
  for (const [category, manufacturers] of Object.entries(categories)) {
    try {
      const catalogKey = `catalog/category/${category}.json`;
      await axios.put(`${LOCALSTACK_URL}/${PUBLIC_BUCKET}/${catalogKey}`, 
        JSON.stringify({
          category,
          count: manufacturers.length,
          manufacturers: manufacturers.map(m => ({
            id: m.id,
            name: m.name,
            city: m.city,
            state: m.state,
            categories: m.categories,
            capabilities: m.capabilities
          }))
        }), {
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `AWS4-HMAC-SHA256 Credential=${AWS_ACCESS_KEY_ID}/${new Date().toISOString().split('T')[0]}/${AWS_REGION}/s3/aws4_request`,
        }
      });
      
      console.log(`✅ Created catalog for category: ${category}`);
    } catch (error) {
      console.error(`❌ Failed to create catalog for ${category}:`, error.message);
    }
  }
}

async function setupSESEmail() {
  console.log('Setting up SES email...');
  
  try {
    // Verify sender email
    await axios.post(`${LOCALSTACK_URL}/`, 
      'Action=VerifyEmailIdentity&EmailAddress=test@terra.local&Version=2010-12-01', {
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
        'Authorization': `AWS4-HMAC-SHA256 Credential=${AWS_ACCESS_KEY_ID}/${new Date().toISOString().split('T')[0]}/${AWS_REGION}/ses/aws4_request`,
      }
    });
    
    console.log('✅ SES email verification setup');
  } catch (error) {
    console.error('❌ Failed to setup SES email:', error.message);
  }
}

async function main() {
  console.log('🚀 Setting up test data for Terra platform...');
  
  // Wait for LocalStack to be ready
  let retries = 30;
  while (retries > 0) {
    try {
      await axios.get(`${LOCALSTACK_URL}/_localstack/health`);
      console.log('✅ LocalStack is ready');
      break;
    } catch (error) {
      console.log(`⏳ Waiting for LocalStack... (${retries} retries left)`);
      await new Promise(resolve => setTimeout(resolve, 2000));
      retries--;
    }
  }
  
  if (retries === 0) {
    console.error('❌ LocalStack not ready, exiting');
    process.exit(1);
  }
  
  await createS3Buckets();
  await createTestManufacturers();
  await createCatalogPages();
  await setupSESEmail();
  
  console.log('🎉 Test data setup complete!');
}

main().catch(console.error);