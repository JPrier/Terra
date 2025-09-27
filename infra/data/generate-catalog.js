#!/usr/bin/env node
/**
 * Script to generate catalog files from manufacturer data for CDK deployment
 */

const fs = require('fs');
const path = require('path');

// Read manufacturer data
const manufacturersPath = path.join(__dirname, 'manufacturers.json');
const categoriesPath = path.join(__dirname, 'categories.json');

if (!fs.existsSync(manufacturersPath)) {
  console.error('❌ manufacturers.json not found');
  process.exit(1);
}

if (!fs.existsSync(categoriesPath)) {
  console.error('❌ categories.json not found');
  process.exit(1);
}

const manufacturers = JSON.parse(fs.readFileSync(manufacturersPath, 'utf8'));
const categories = JSON.parse(fs.readFileSync(categoriesPath, 'utf8'));

// Create manufacturer directory structure
const manufacturerDir = path.join(__dirname, 'manufacturer');
const catalogDir = path.join(__dirname, 'catalog', 'category');

// Ensure directories exist
fs.mkdirSync(manufacturerDir, { recursive: true });
fs.mkdirSync(catalogDir, { recursive: true });

// Create individual manufacturer files
console.log('🔧 Creating manufacturer profile files...');
manufacturers.forEach(manufacturer => {
  const manufacturerFile = path.join(manufacturerDir, `${manufacturer.id}.json`);
  fs.writeFileSync(manufacturerFile, JSON.stringify(manufacturer, null, 2));
  console.log(`✅ Created ${manufacturer.id}.json`);
});

// Create catalog files by category
console.log('🔧 Creating catalog category files...');
Object.entries(categories).forEach(([categoryKey, categoryInfo]) => {
  const categoryManufacturers = manufacturers.filter(m => 
    categoryInfo.manufacturer_ids.includes(m.id)
  );

  const catalogData = {
    category: categoryKey,
    name: categoryInfo.name,
    description: categoryInfo.description,
    count: categoryManufacturers.length,
    manufacturers: categoryManufacturers.map(m => ({
      id: m.id,
      name: m.name,
      city: m.city,
      state: m.state,
      categories: m.categories,
      capabilities: m.capabilities
    }))
  };

  const catalogFile = path.join(catalogDir, `${categoryKey}.json`);
  fs.writeFileSync(catalogFile, JSON.stringify(catalogData, null, 2));
  console.log(`✅ Created catalog/${categoryKey}.json`);
});

console.log('🎉 CDK deployment data generation complete!');