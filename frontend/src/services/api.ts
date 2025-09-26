// API service for Terra frontend
// This would connect to the actual API in production

const API_BASE_URL = process.env.REACT_APP_API_URL || 'https://api.terra-platform.com/v1';

export interface CreateRfqRequest {
  tenant_id: string;
  manufacturer_id: string;
  buyer: { email: string; name?: string };
  subject: string;
  body: string;
  attachments?: Array<{
    upload_key: string;
    file_name: string;
    content_type: string;
    size_bytes: number;
  }>;
}

export interface CreateRfqResponse {
  id: string;
  last_event_ts: string;
}

export interface RfqEvent {
  id: string;
  rfq_id: string;
  ts: string;
  by: string;
  type: 'message' | 'status' | 'attachment';
  body?: string;
  status?: string;
  note?: string;
  attachments?: Array<{
    id: string;
    file_name: string;
    content_type: string;
    size_bytes: number;
    key: string;
  }>;
}

export interface ListEventsResponse {
  items: RfqEvent[];
  next_since?: string;
}

class TerraApiService {
  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${API_BASE_URL}${endpoint}`;
    
    const response = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({}));
      throw new Error(error.message || `HTTP ${response.status}: ${response.statusText}`);
    }

    return response.json();
  }

  // RFQ Operations
  async createRfq(request: CreateRfqRequest, idempotencyKey?: string): Promise<CreateRfqResponse> {
    const headers: Record<string, string> = {};
    if (idempotencyKey) {
      headers['Idempotency-Key'] = idempotencyKey;
    }

    return this.request<CreateRfqResponse>('/rfqs', {
      method: 'POST',
      headers,
      body: JSON.stringify(request),
    });
  }

  async getRfq(rfqId: string) {
    return this.request(`/rfqs/${rfqId}`);
  }

  async listRfqEvents(
    rfqId: string,
    since?: string,
    limit?: number
  ): Promise<ListEventsResponse> {
    const params = new URLSearchParams();
    if (since) params.append('since', since);
    if (limit) params.append('limit', limit.toString());
    
    const query = params.toString();
    const endpoint = `/rfqs/${rfqId}/events${query ? `?${query}` : ''}`;
    
    return this.request<ListEventsResponse>(endpoint);
  }

  async postMessage(
    rfqId: string,
    message: { by: string; body: string; attachments?: any[] },
    idempotencyKey?: string
  ) {
    const headers: Record<string, string> = {};
    if (idempotencyKey) {
      headers['Idempotency-Key'] = idempotencyKey;
    }

    return this.request(`/rfqs/${rfqId}/messages`, {
      method: 'POST',
      headers,
      body: JSON.stringify(message),
    });
  }

  // Upload Operations
  async getPresignedUploadUrl(request: {
    tenant_id: string;
    pathType: string;
    content_type: string;
    size_bytes: number;
  }) {
    return this.request<{
      url: string;
      key: string;
      expires_in: number;
    }>('/uploads/presign', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  }

  // Manufacturer Operations (would fetch from S3/CloudFront in production)
  async getManufacturers() {
    // For MVP, return empty array - would fetch catalog from CloudFront
    return [];
  }

  async getManufacturer(manufacturerId: string) {
    // For MVP, return null - would fetch from S3/CloudFront
    return null;
  }
}

export const terraApi = new TerraApiService();