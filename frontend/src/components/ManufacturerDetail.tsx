import React from 'react';
import { useParams, Link } from 'react-router-dom';

const ManufacturerDetail: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  
  // Mock data for MVP - would fetch from API in production
  const manufacturer = {
    id: 'mfg_001',
    name: 'Precision Manufacturing Co.',
    description: 'Family-owned precision machining company specializing in aerospace and medical components. ISO 9001 certified with over 30 years of experience.',
    city: 'Columbus',
    state: 'OH',
    categories: ['machining', 'prototyping'],
    capabilities: ['CNC Milling', '5-Axis Machining', 'Precision Turning', 'Quality Inspection'],
    contact_email: 'quotes@precision-mfg.com',
  };

  return (
    <div>
      <div style={{ marginBottom: '2rem' }}>
        <Link to="/manufacturers" className="btn">← Back to Manufacturers</Link>
      </div>
      
      <div className="card">
        <h2>{manufacturer.name}</h2>
        <p><strong>Location:</strong> {manufacturer.city}, {manufacturer.state}</p>
        
        <div style={{ margin: '2rem 0' }}>
          <h3>About</h3>
          <p>{manufacturer.description}</p>
        </div>

        <div style={{ margin: '2rem 0' }}>
          <h3>Categories</h3>
          <p>{manufacturer.categories.join(', ')}</p>
        </div>

        <div style={{ margin: '2rem 0' }}>
          <h3>Capabilities</h3>
          <ul>
            {manufacturer.capabilities.map((capability, index) => (
              <li key={index}>{capability}</li>
            ))}
          </ul>
        </div>

        <div style={{ margin: '2rem 0' }}>
          <h3>Contact</h3>
          <p>Email: {manufacturer.contact_email}</p>
        </div>

        <div style={{ textAlign: 'center', marginTop: '3rem' }}>
          <Link to={`/submit-rfq/${manufacturer.id}`} className="btn btn-primary" style={{ fontSize: '1.2rem', padding: '1rem 2rem' }}>
            Submit RFQ to {manufacturer.name}
          </Link>
        </div>
      </div>
    </div>
  );
};

export default ManufacturerDetail;