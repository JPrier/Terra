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

// Simple HTML template function to generate category pages
function generateCategoryHTML(category, manufacturers) {
  const title = `${category.charAt(0).toUpperCase() + category.slice(1)} Manufacturers`;
  
  const manufacturerCards = manufacturers.map(m => `
    <article class="card manufacturer-card">
      <div class="manufacturer-info">
        <h3><a href="/catalog/manufacturer/${m.id}/">${m.name}</a></h3>
        <p class="location">${m.city}, ${m.state}</p>
        <p class="categories">Categories: ${m.categories.join(', ')}</p>
        <p class="capabilities">Capabilities: ${m.capabilities.join(', ')}</p>
        <div class="actions">
          <a href="/catalog/manufacturer/${m.id}/" class="btn">View Details</a>
          <a href="/rfq/submit?mfg=${m.id}" class="btn btn-primary">Submit RFQ</a>
        </div>
      </div>
    </article>
  `).join('\n');
  
  return `<!doctype html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>${title} - Terra</title>
  <link rel="stylesheet" href="/src/styles/globals.css">
  <meta name="description" content="Find verified ${category} manufacturers" />
</head>
<body>
  <header class="header">
    <div class="header-content">
      <h1>Terra</h1>
      <p class="tagline">US Manufacturing Directory & RFQ Platform</p>
      <nav class="nav">
        <a href="/">Home</a>
        <a href="/catalog/machining/">Browse Manufacturers</a>
      </nav>
    </div>
  </header>
  
  <main class="main">
    <h2>${title}</h2>
    <p>Find qualified ${category} manufacturers and submit RFQs directly.</p>
    
    <div class="catalog-stats">
      <p><strong>${manufacturers.length}</strong> verified manufacturers in ${category}</p>
    </div>
    
    <div class="grid" id="manufacturer-grid">
      ${manufacturerCards}
    </div>
  </main>
  
  <footer style="text-align: center; padding: 2rem; color: #7f8c8d; border-top: 1px solid #eee; margin-top: 4rem;">
    <p>&copy; 2024 Terra Manufacturing Platform. Built for American manufacturing.</p>
  </footer>
</body>
</html>`;
}

// Sample manufacturer data (Enhanced 5-manufacturer dataset)
const sampleManufacturers = [
  {
    id: 'precision-aerospace-cnc',
    name: 'Precision Aerospace CNC',
    city: 'Seattle',
    state: 'WA',
    categories: ['CNC Machining', 'Aerospace', 'Precision Manufacturing'],
    capabilities: ['5-axis CNC Machining', 'Titanium Alloys', 'Aluminum 7075', 'Stainless Steel 316L', 'AS9100 Certified', 'Prototype to Production', 'Tight Tolerances ±0.0005"', 'Complex Geometries'],
    description: 'Premier aerospace precision machining company specializing in critical flight components. AS9100 and ISO 9001 certified with over 25 years of experience serving Boeing, Lockheed Martin, and other aerospace leaders. Expertise in complex 5-axis machining of exotic materials with tolerances as tight as ±0.0005 inches.',
    contact_email: 'quotes@precision-aerospace-cnc.com'
  },
  {
    id: 'advanced-molding-systems',
    name: 'Advanced Molding Systems',
    city: 'Austin',
    state: 'TX',
    categories: ['Injection Molding', 'Plastics', 'Consumer Electronics'],
    capabilities: ['High-Volume Injection Molding', 'Multi-Cavity Tooling', 'Overmolding', 'Insert Molding', 'Medical Grade Materials', 'Clean Room Production', 'Secondary Operations', 'Assembly Services'],
    description: 'Leading injection molding manufacturer serving consumer electronics, medical devices, and automotive industries. State-of-the-art facility with 50+ molding machines ranging from 50-1000 tons. ISO 13485 certified for medical devices with Class 7 clean room capabilities. Specializes in high-volume production and complex multi-material assemblies.',
    contact_email: 'info@advanced-molding-systems.com'
  },
  {
    id: 'industrial-metal-fabricators',
    name: 'Industrial Metal Fabricators',
    city: 'Pittsburgh',
    state: 'PA',
    categories: ['Sheet Metal', 'Fabrication', 'Welding', 'Industrial Equipment'],
    capabilities: ['Laser Cutting up to 1" Steel', 'Press Brake Forming', 'TIG/MIG Welding', 'Powder Coating', 'CNC Punching', 'Assembly Services', 'Heavy Fabrication', 'Custom Enclosures'],
    description: 'Full-service metal fabrication shop specializing in custom industrial equipment, enclosures, and structural components. 40,000 sq ft facility with advanced laser cutting, forming, and welding capabilities. Serving oil & gas, power generation, and industrial automation industries with projects ranging from prototypes to large-scale production runs.',
    contact_email: 'projects@industrial-metal-fab.com'
  },
  {
    id: 'rapid-additive-solutions',
    name: 'Rapid Additive Solutions',
    city: 'Denver',
    state: 'CO',
    categories: ['3D Printing', 'Additive Manufacturing', 'Prototyping', 'Low-Volume Production'],
    capabilities: ['SLA High-Resolution Printing', 'SLS Nylon Production', 'FDM Engineering Plastics', 'Metal 3D Printing (DMLS)', 'Post-Processing Services', 'Design for Additive', 'Rapid Prototyping', 'Bridge Manufacturing'],
    description: 'Cutting-edge additive manufacturing facility with 20+ industrial 3D printers including metal DMLS systems. Specializes in rapid prototyping, low-volume production, and complex geometries impossible with traditional manufacturing. Serving aerospace, medical, automotive, and consumer product industries with same-day to 2-week turnarounds.',
    contact_email: 'orders@rapid-additive.com'
  },
  {
    id: 'precision-electronics-assembly',
    name: 'Precision Electronics Assembly',
    city: 'San Jose',
    state: 'CA',
    categories: ['Electronics Assembly', 'PCB Assembly', 'Contract Manufacturing', 'Testing'],
    capabilities: ['Surface Mount Technology (SMT)', 'Through-Hole Assembly', 'Mixed Technology PCBs', 'BGA/μBGA Assembly', 'Functional Testing', 'Box Build Assembly', 'Cable Assembly', 'Quality Inspection (AOI/X-ray)'],
    description: 'Premier electronics contract manufacturer in Silicon Valley serving high-tech, medical, and aerospace industries. IPC-610 Class 3 certified facility with advanced SMT lines, automated optical inspection, and comprehensive testing capabilities. Expertise in complex mixed-technology PCBs, rigid-flex assemblies, and full box-build services.',
    contact_email: 'sales@precision-ems.com'
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
    'fabrication': sampleManufacturers.filter(m => m.categories.includes('Sheet Metal') || m.categories.includes('Fabrication')),
    'additive': sampleManufacturers.filter(m => m.categories.includes('3D Printing') || m.categories.includes('Additive Manufacturing')),
    'electronics': sampleManufacturers.filter(m => m.categories.includes('Electronics Assembly') || m.categories.includes('PCB Assembly'))
  };
  
  // Create categories index for frontend discovery
  try {
    const categoriesIndex = {
      categories: Object.keys(categories),
      last_updated: new Date().toISOString()
    };
    
    const categoriesKey = `catalog/categories.json`;
    await axios.put(`${LOCALSTACK_URL}/${PUBLIC_BUCKET}/${categoriesKey}`, 
      JSON.stringify(categoriesIndex), {
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `AWS4-HMAC-SHA256 Credential=${AWS_ACCESS_KEY_ID}/${new Date().toISOString().split('T')[0]}/${AWS_REGION}/s3/aws4_request`,
      }
    });
    
    console.log(`✅ Created categories index with categories: ${Object.keys(categories).join(', ')}`);
  } catch (error) {
    console.error(`❌ Failed to create categories index:`, error.message);
  }
  
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
      
      // Also create pre-rendered HTML page for this category
      const htmlContent = generateCategoryHTML(category, manufacturers);
      const htmlKey = `catalog/${category}/index.html`;
      
      await axios.put(`${LOCALSTACK_URL}/${PUBLIC_BUCKET}/${htmlKey}`, 
        htmlContent, {
        headers: {
          'Content-Type': 'text/html',
          'Authorization': `AWS4-HMAC-SHA256 Credential=${AWS_ACCESS_KEY_ID}/${new Date().toISOString().split('T')[0]}/${AWS_REGION}/s3/aws4_request`,
        }
      });
      
      console.log(`✅ Created pre-rendered HTML for category: ${category}`);
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