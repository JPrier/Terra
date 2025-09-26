import React from 'react';
import { Link } from 'react-router-dom';

const HomePage: React.FC = () => {
  return (
    <div>
      <section className="hero">
        <h2>Connect with US Manufacturers</h2>
        <p>
          Find qualified manufacturers for your projects and submit RFQs directly.
          Fast, reliable, and built for American manufacturing.
        </p>
      </section>

      <div className="grid">
        <div className="card">
          <h3>Browse Manufacturers</h3>
          <p>Explore our directory of verified US manufacturers across all categories.</p>
          <Link to="/manufacturers" className="btn">Browse Now</Link>
        </div>

        <div className="card">
          <h3>Submit an RFQ</h3>
          <p>Get quotes quickly by submitting your requirements directly to manufacturers.</p>
          <Link to="/manufacturers" className="btn btn-primary">Get Started</Link>
        </div>

        <div className="card">
          <h3>Categories</h3>
          <p>CNC Machining • 3D Printing • Injection Molding • Sheet Metal • Electronics • More</p>
        </div>
      </div>

      <section style={{ textAlign: 'center', marginTop: '3rem' }}>
        <h3>Why Choose Terra?</h3>
        <div className="grid">
          <div>
            <h4>🇺🇸 US-Only</h4>
            <p>All manufacturers are verified US-based companies</p>
          </div>
          <div>
            <h4>⚡ Fast Quotes</h4>
            <p>Get responses from manufacturers in hours, not days</p>
          </div>
          <div>
            <h4>🔒 Secure</h4>
            <p>Your projects and data are protected</p>
          </div>
        </div>
      </section>
    </div>
  );
};

export default HomePage;