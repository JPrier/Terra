import React, { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';

const SubmitRFQ: React.FC = () => {
  const { manufacturerId } = useParams<{ manufacturerId: string }>();
  const navigate = useNavigate();
  
  const [formData, setFormData] = useState({
    buyerName: '',
    buyerEmail: '',
    subject: '',
    description: '',
  });
  
  const [submitting, setSubmitting] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);

    // In production, this would call the API
    // For MVP, we'll simulate the submission
    try {
      await new Promise(resolve => setTimeout(resolve, 2000)); // Simulate API call
      
      // Mock response
      const rfqId = 'r_' + Math.random().toString(36).substr(2, 8).toUpperCase();
      
      alert(`RFQ submitted successfully! RFQ ID: ${rfqId}`);
      navigate(`/rfq/${rfqId}`);
    } catch (error) {
      alert('Error submitting RFQ. Please try again.');
    } finally {
      setSubmitting(false);
    }
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
    setFormData({
      ...formData,
      [e.target.name]: e.target.value,
    });
  };

  return (
    <div>
      <h2>Submit Request for Quote</h2>
      <p>Send your requirements directly to the manufacturer and get a quick response.</p>

      <form onSubmit={handleSubmit} className="card">
        <div className="form-group">
          <label htmlFor="buyerName">Your Name</label>
          <input
            type="text"
            id="buyerName"
            name="buyerName"
            value={formData.buyerName}
            onChange={handleChange}
            required
          />
        </div>

        <div className="form-group">
          <label htmlFor="buyerEmail">Email Address</label>
          <input
            type="email"
            id="buyerEmail"
            name="buyerEmail"
            value={formData.buyerEmail}
            onChange={handleChange}
            required
          />
        </div>

        <div className="form-group">
          <label htmlFor="subject">Project Title</label>
          <input
            type="text"
            id="subject"
            name="subject"
            value={formData.subject}
            onChange={handleChange}
            placeholder="e.g., CNC milling prototype"
            required
          />
        </div>

        <div className="form-group">
          <label htmlFor="description">Project Description</label>
          <textarea
            id="description"
            name="description"
            value={formData.description}
            onChange={handleChange}
            placeholder="Describe your project requirements, materials, quantities, timeline, etc."
            required
          />
        </div>

        <div style={{ textAlign: 'center' }}>
          <button 
            type="submit" 
            className="btn btn-primary"
            disabled={submitting}
            style={{ fontSize: '1.1rem', padding: '1rem 2rem' }}
          >
            {submitting ? 'Submitting...' : 'Submit RFQ'}
          </button>
        </div>
      </form>
    </div>
  );
};

export default SubmitRFQ;