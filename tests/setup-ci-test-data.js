#!/usr/bin/env node
/**
 * Simplified setup script for CI environments
 * Creates mock data without requiring full LocalStack setup
 */

import { writeFileSync, mkdirSync } from 'fs';
import { dirname } from 'path';

const TEST_DATA_DIR = '/tmp/test-data';

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
  }
];

async function setupCITestData() {
  console.log('🚀 Setting up test data for CI environment...');
  
  try {
    // Create test data directory
    mkdirSync(TEST_DATA_DIR, { recursive: true });
    
    // Create manufacturer data files
    for (const manufacturer of sampleManufacturers) {
      const filePath = `${TEST_DATA_DIR}/manufacturer-${manufacturer.id}.json`;
      writeFileSync(filePath, JSON.stringify(manufacturer, null, 2));
      console.log(`✅ Created test data: ${filePath}`);
    }
    
    // Create catalog data
    const catalogData = {
      category: 'machining',
      count: sampleManufacturers.length,
      manufacturers: sampleManufacturers
    };
    
    const catalogPath = `${TEST_DATA_DIR}/catalog-machining.json`;
    writeFileSync(catalogPath, JSON.stringify(catalogData, null, 2));
    console.log(`✅ Created catalog data: ${catalogPath}`);
    
    console.log('🎉 CI test data setup complete!');
  } catch (error) {
    console.error('❌ Failed to setup CI test data:', error.message);
    process.exit(1);
  }
}

setupCITestData().catch(console.error);