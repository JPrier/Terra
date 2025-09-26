import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';

interface Manufacturer {
  id: string;
  name: string;
  city?: string;
  state?: string;
  categories: string[];
  capabilities?: string[];
  logo?: string;
}

const ManufacturerList: React.FC = () => {
  const [manufacturers, setManufacturers] = useState<Manufacturer[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // For MVP, we'll use mock data
    // In production, this would fetch from the catalog API
    setTimeout(() => {
      const mockData: Manufacturer[] = [
        {
          id: 'mfg_001',
          name: 'Precision Manufacturing Co.',
          city: 'Columbus',
          state: 'OH',
          categories: ['machining', 'prototyping'],
          capabilities: ['cnc_milling', '5_axis_machining'],
        },
        {
          id: 'mfg_002', 
          name: 'Advanced Plastics Inc.',
          city: 'Austin',
          state: 'TX',
          categories: ['injection_molding', 'plastics'],
          capabilities: ['injection_molding', 'overmolding'],
        },
        {
          id: 'mfg_003',
          name: 'Metal Works LLC',
          city: 'Denver',
          state: 'CO', 
          categories: ['sheet_metal', 'fabrication'],
          capabilities: ['laser_cutting', 'welding', 'powder_coating'],
        },
      ];
      setManufacturers(mockData);
      setLoading(false);
    }, 1000);
  }, []);

  if (loading) {
    return <div className="loading">Loading manufacturers...</div>;
  }

  if (error) {
    return <div className="error">Error: {error}</div>;
  }

  return (
    <div>
      <h2>US Manufacturers</h2>
      <p>Browse our directory of verified manufacturers. Click on any manufacturer to view details and submit an RFQ.</p>
      
      <div className="grid">
        {manufacturers.map((manufacturer) => (
          <div key={manufacturer.id} className="card">
            <h3>{manufacturer.name}</h3>
            {manufacturer.city && manufacturer.state && (
              <p><strong>Location:</strong> {manufacturer.city}, {manufacturer.state}</p>
            )}
            <p><strong>Categories:</strong> {manufacturer.categories.join(', ')}</p>
            {manufacturer.capabilities && (
              <p><strong>Capabilities:</strong> {manufacturer.capabilities.join(', ')}</p>
            )}
            <div style={{ marginTop: '1rem' }}>
              <Link to={`/manufacturers/${manufacturer.id}`} className="btn">
                View Details
              </Link>
              <Link 
                to={`/submit-rfq/${manufacturer.id}`} 
                className="btn btn-primary" 
                style={{ marginLeft: '1rem' }}
              >
                Submit RFQ
              </Link>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default ManufacturerList;