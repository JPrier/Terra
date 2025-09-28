// Static manufacturer data for fallback when S3 is not available
// This data matches the CDK-deployed S3 data structure

export const MANUFACTURERS = [
  {
    id: "precision-aerospace-cnc",
    name: "Precision Aerospace CNC",
    city: "Seattle",
    state: "WA",
    logoUrl: null,
    categories: ["CNC Machining", "Aerospace", "Precision Manufacturing"],
    capabilities: [
      "5-axis CNC Machining",
      "Titanium Alloys",
      "Aluminum 7075",
      "Stainless Steel 316L",
      "AS9100 Certified",
      "Prototype to Production",
      "Tight Tolerances ±0.0005\"",
      "Complex Geometries"
    ],
    description: "Premier aerospace precision machining company specializing in critical flight components. AS9100 and ISO 9001 certified with over 25 years of experience serving Boeing, Lockheed Martin, and other aerospace leaders. Expertise in complex 5-axis machining of exotic materials with tolerances as tight as ±0.0005 inches.",
    contact_email: "quotes@precision-aerospace-cnc.com",
    certifications: ["AS9100", "ISO 9001", "ITAR Registered"],
    materials: ["Titanium Ti-6Al-4V", "Aluminum 7075-T6", "Stainless 316L", "Inconel 625"],
    min_order_quantity: "Prototype to 10K+ units",
    lead_time_weeks: "2-8 weeks"
  },
  {
    id: "advanced-molding-systems",
    name: "Advanced Molding Systems",
    city: "Austin",
    state: "TX",
    logoUrl: null,
    categories: ["Injection Molding", "Plastics", "Consumer Electronics"],
    capabilities: [
      "High-Volume Injection Molding",
      "Multi-Cavity Tooling",
      "Overmolding",
      "Insert Molding",
      "Medical Grade Materials",
      "Clean Room Production",
      "Secondary Operations",
      "Assembly Services"
    ],
    description: "Leading injection molding manufacturer serving consumer electronics, medical devices, and automotive industries. State-of-the-art facility with 50+ molding machines ranging from 50-1000 tons. ISO 13485 certified for medical devices with Class 7 clean room capabilities. Specializes in high-volume production and complex multi-material assemblies.",
    contact_email: "info@advanced-molding-systems.com",
    certifications: ["ISO 13485", "ISO 9001", "FDA Registered", "TS 16949"],
    materials: ["PC", "ABS", "PP", "PE", "TPU", "PEEK", "Medical Grade Resins"],
    min_order_quantity: "1,000 to 10M+ units",
    lead_time_weeks: "4-12 weeks"
  },
  {
    id: "industrial-metal-fabricators",
    name: "Industrial Metal Fabricators",
    city: "Pittsburgh",
    state: "PA",
    logoUrl: null,
    categories: ["Sheet Metal", "Fabrication", "Welding", "Industrial Equipment"],
    capabilities: [
      "Laser Cutting up to 1\" Steel",
      "Press Brake Forming",
      "TIG/MIG Welding",
      "Powder Coating",
      "CNC Punching",
      "Assembly Services",
      "Heavy Fabrication",
      "Custom Enclosures"
    ],
    description: "Full-service metal fabrication shop specializing in custom industrial equipment, enclosures, and structural components. 40,000 sq ft facility with advanced laser cutting, forming, and welding capabilities. Serving oil & gas, power generation, and industrial automation industries with projects ranging from prototypes to large-scale production runs.",
    contact_email: "projects@industrial-metal-fab.com",
    certifications: ["AWS D1.1", "ASME U-Stamp", "ISO 9001"],
    materials: ["Carbon Steel", "Stainless Steel", "Aluminum", "Galvanized Steel"],
    min_order_quantity: "1 to 1000+ units",
    lead_time_weeks: "1-6 weeks"
  },
  {
    id: "rapid-additive-solutions",
    name: "Rapid Additive Solutions",
    city: "Denver",
    state: "CO",
    logoUrl: null,
    categories: ["3D Printing", "Additive Manufacturing", "Prototyping", "Low-Volume Production"],
    capabilities: [
      "SLA High-Resolution Printing",
      "SLS Nylon Production",
      "FDM Engineering Plastics",
      "Metal 3D Printing (DMLS)",
      "Post-Processing Services",
      "Design for Additive",
      "Rapid Prototyping",
      "Bridge Manufacturing"
    ],
    description: "Cutting-edge additive manufacturing facility with 20+ industrial 3D printers including metal DMLS systems. Specializes in rapid prototyping, low-volume production, and complex geometries impossible with traditional manufacturing. Serving aerospace, medical, automotive, and consumer product industries with same-day to 2-week turnarounds.",
    contact_email: "orders@rapid-additive.com",
    certifications: ["ISO 9001", "AS9100 (in progress)"],
    materials: ["PLA", "ABS", "PETG", "Nylon PA12", "Titanium Ti64", "Aluminum AlSi10Mg", "Stainless 316L"],
    min_order_quantity: "1 to 500 units",
    lead_time_weeks: "0.1-2 weeks"
  },
  {
    id: "precision-electronics-assembly",
    name: "Precision Electronics Assembly",
    city: "San Jose",
    state: "CA",
    logoUrl: null,
    categories: ["Electronics Assembly", "PCB Assembly", "Contract Manufacturing", "Testing"],
    capabilities: [
      "Surface Mount Technology (SMT)",
      "Through-Hole Assembly",
      "Mixed Technology PCBs",
      "BGA/μBGA Assembly",
      "Functional Testing",
      "Box Build Assembly",
      "Cable Assembly",
      "Quality Inspection (AOI/X-ray)"
    ],
    description: "Premier electronics contract manufacturer in Silicon Valley serving high-tech, medical, and aerospace industries. IPC-610 Class 3 certified facility with advanced SMT lines, automated optical inspection, and comprehensive testing capabilities. Expertise in complex mixed-technology PCBs, rigid-flex assemblies, and full box-build services.",
    contact_email: "sales@precision-ems.com",
    certifications: ["IPC-610 Class 3", "ISO 9001", "ISO 13485", "ITAR Registered"],
    materials: ["FR4 PCBs", "Flex Circuits", "Ceramic Substrates", "Components 0201-BGA"],
    min_order_quantity: "10 to 100K+ units",
    lead_time_weeks: "2-8 weeks"
  }
];

// Category mapping for filtering manufacturers
export const CATEGORY_FILTERS = {
  machining: ["CNC Machining"],
  molding: ["Injection Molding"],
  fabrication: ["Sheet Metal", "Fabrication"],
  additive: ["3D Printing", "Additive Manufacturing"],
  electronics: ["Electronics Assembly", "PCB Assembly"]
};

// Utility function to get manufacturers by category
export function getManufacturersByCategory(category: string) {
  const filters = CATEGORY_FILTERS[category as keyof typeof CATEGORY_FILTERS];
  if (!filters) {
    return [];
  }
  
  return MANUFACTURERS.filter(manufacturer => 
    manufacturer.categories.some(cat => filters.includes(cat))
  );
}

// Utility function to create fallback catalog data structure
export function createFallbackCatalogData(category: string) {
  const manufacturers = getManufacturersByCategory(category);
  return {
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
  };
}