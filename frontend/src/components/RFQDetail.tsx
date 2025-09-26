import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';

interface RFQEvent {
  id: string;
  ts: string;
  by: string;
  type: 'message' | 'status' | 'attachment';
  body?: string;
  status?: string;
}

const RFQDetail: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const [events, setEvents] = useState<RFQEvent[]>([]);
  const [newMessage, setNewMessage] = useState('');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // Mock data for MVP - would fetch from API in production
    const mockEvents: RFQEvent[] = [
      {
        id: '1',
        ts: new Date().toISOString(),
        by: 'system',
        type: 'status',
        status: 'rfq_created'
      },
      {
        id: '2', 
        ts: new Date().toISOString(),
        by: 'buyer',
        type: 'message',
        body: 'Looking for CNC milling of 2 aluminum parts, 6061 material, 7-day turnaround needed.'
      }
    ];
    
    setTimeout(() => {
      setEvents(mockEvents);
      setLoading(false);
    }, 1000);
  }, [id]);

  const handleSubmitMessage = (e: React.FormEvent) => {
    e.preventDefault();
    if (!newMessage.trim()) return;

    const newEvent: RFQEvent = {
      id: Date.now().toString(),
      ts: new Date().toISOString(),
      by: 'buyer', // In production, this would be determined by auth
      type: 'message',
      body: newMessage
    };

    setEvents([...events, newEvent]);
    setNewMessage('');
  };

  const formatTimestamp = (ts: string) => {
    return new Date(ts).toLocaleDateString() + ' ' + new Date(ts).toLocaleTimeString();
  };

  if (loading) {
    return <div className="loading">Loading RFQ details...</div>;
  }

  return (
    <div>
      <h2>RFQ Details - {id}</h2>
      
      <div className="card">
        <h3>RFQ Information</h3>
        <p><strong>Status:</strong> Open</p>
        <p><strong>Created:</strong> {formatTimestamp(events[0]?.ts || '')}</p>
        <p><strong>Manufacturer:</strong> Precision Manufacturing Co.</p>
      </div>

      <div className="card">
        <h3>Conversation</h3>
        <div style={{ maxHeight: '400px', overflowY: 'auto', marginBottom: '1rem' }}>
          {events.map((event) => (
            <div 
              key={event.id} 
              style={{ 
                padding: '1rem', 
                borderBottom: '1px solid #eee',
                backgroundColor: event.by === 'buyer' ? '#f0f8ff' : '#f9f9f9'
              }}
            >
              <div style={{ fontSize: '0.9rem', color: '#666', marginBottom: '0.5rem' }}>
                <strong>{event.by === 'buyer' ? 'You' : event.by}</strong> - {formatTimestamp(event.ts)}
              </div>
              {event.type === 'message' && event.body && (
                <div>{event.body}</div>
              )}
              {event.type === 'status' && (
                <div style={{ fontStyle: 'italic', color: '#666' }}>
                  Status: {event.status?.replace('_', ' ')}
                </div>
              )}
            </div>
          ))}
        </div>

        <form onSubmit={handleSubmitMessage}>
          <div className="form-group">
            <label htmlFor="message">Send a message:</label>
            <textarea
              id="message"
              value={newMessage}
              onChange={(e) => setNewMessage(e.target.value)}
              placeholder="Type your message here..."
              rows={3}
            />
          </div>
          <button type="submit" className="btn btn-primary">Send Message</button>
        </form>
      </div>
    </div>
  );
};

export default RFQDetail;