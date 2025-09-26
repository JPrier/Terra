import React from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import Header from './components/Header';
import HomePage from './components/HomePage';
import ManufacturerList from './components/ManufacturerList';
import ManufacturerDetail from './components/ManufacturerDetail';
import SubmitRFQ from './components/SubmitRFQ';
import RFQDetail from './components/RFQDetail';
import './App.css';

function App() {
  return (
    <Router>
      <div className="App">
        <Header />
        <main>
          <Routes>
            <Route path="/" element={<HomePage />} />
            <Route path="/manufacturers" element={<ManufacturerList />} />
            <Route path="/manufacturers/:id" element={<ManufacturerDetail />} />
            <Route path="/submit-rfq/:manufacturerId" element={<SubmitRFQ />} />
            <Route path="/rfq/:id" element={<RFQDetail />} />
          </Routes>
        </main>
      </div>
    </Router>
  );
}

export default App;