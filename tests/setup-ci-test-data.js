#!/usr/bin/env node
/**
 * Simplified setup script for CI environments
 * Creates mock data without requiring full LocalStack setup
 */

import { writeFileSync, mkdirSync } from 'fs';
import { dirname } from 'path';

const TEST_DATA_DIR = '/tmp/test-data';

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