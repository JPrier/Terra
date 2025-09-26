import React from 'react';
import { Link } from 'react-router-dom';

const Header: React.FC = () => {
  return (
    <header className="header">
      <div>
        <h1>Terra</h1>
        <p>US Manufacturing Directory & RFQ Platform</p>
        <nav className="nav">
          <Link to="/">Home</Link>
          <Link to="/manufacturers">Browse Manufacturers</Link>
        </nav>
      </div>
    </header>
  );
};

export default Header;