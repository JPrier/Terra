// API route to fetch available categories from S3

export async function GET() {
  const S3_BASE_URL = import.meta.env.PUBLIC_S3_URL || 'https://your-cloudfront-domain.cloudfront.net';
  
  try {
    const response = await fetch(`${S3_BASE_URL}/test-data/catalog/categories.json`);
    
    if (!response.ok) {
      // Fallback categories if the index is not available
      return new Response(JSON.stringify({
        categories: ['machining', 'molding', 'fabrication', 'additive', 'electronics'],
        source: 'fallback'
      }), {
        status: 200,
        headers: {
          'Content-Type': 'application/json',
          'Cache-Control': 'public, max-age=300' // Cache for 5 minutes
        }
      });
    }
    
    const data = await response.json();
    
    return new Response(JSON.stringify({
      categories: data.categories || [],
      last_updated: data.last_updated,
      source: 'dynamic'
    }), {
      status: 200,
      headers: {
        'Content-Type': 'application/json',
        'Cache-Control': 'public, max-age=300' // Cache for 5 minutes
      }
    });
  } catch (error) {
    console.error('Error fetching categories:', error);
    
    // Return fallback categories on error
    return new Response(JSON.stringify({
      categories: ['machining', 'molding', 'fabrication', 'additive', 'electronics'],
      source: 'fallback',
      error: error.message
    }), {
      status: 200,
      headers: {
        'Content-Type': 'application/json',
        'Cache-Control': 'public, max-age=60' // Shorter cache on error
      }
    });
  }
}