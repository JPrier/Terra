/**
 * S3 HTML Prefetching Utility
 * 
 * Automatically prefetches pre-rendered HTML pages from S3 for faster navigation.
 * Scans the current page for internal links and prefetches their corresponding S3 HTML.
 */

// Cache to avoid duplicate prefetch requests
const prefetchCache = new Set();

// Get S3 base URL from environment or use fallback
const getS3BaseUrl = () => {
  // Try to get from meta tag first (set by Astro build)
  const metaTag = document.querySelector('meta[name="s3-base-url"]');
  if (metaTag) {
    return metaTag.getAttribute('content');
  }
  
  // Fallback to environment variable pattern used in the app
  return window.PUBLIC_S3_URL || 'https://your-cloudfront-domain.cloudfront.net';
};

/**
 * Maps internal application routes to their corresponding S3 HTML paths
 */
const mapRouteToS3Path = (pathname) => {
  // Remove leading/trailing slashes and base path for processing
  const cleanPath = pathname.replace(/^\/|\/$/g, '');
  const basePath = window.BASE_URL ? window.BASE_URL.replace(/^\/|\/$/g, '') : '';
  const routePath = cleanPath.startsWith(basePath) 
    ? cleanPath.substring(basePath.length).replace(/^\/|\/$/g, '') 
    : cleanPath;

  // Handle root/home page - no S3 equivalent needed (it's static)
  if (!routePath || routePath === '') {
    return null;
  }

  // Handle catalog category pages: /catalog/{category}/ -> /test-data/catalog/{category}/index.html
  const categoryMatch = routePath.match(/^catalog\/([^\/]+)\/?$/);
  if (categoryMatch) {
    const category = categoryMatch[1];
    return `/test-data/catalog/${category}/index.html`;
  }

  // Handle manufacturer detail pages: /catalog/manufacturer/{id}/ -> /test-data/catalog/manufacturer/{id}/index.html
  const manufacturerMatch = routePath.match(/^catalog\/manufacturer\/([^\/]+)\/?$/);
  if (manufacturerMatch) {
    const manufacturerId = manufacturerMatch[1];
    return `/test-data/catalog/manufacturer/${manufacturerId}/index.html`;
  }

  // Handle RFQ pages - these are static, no S3 equivalent needed
  if (routePath.startsWith('rfq/')) {
    return null;
  }

  // Handle API routes - skip prefetching
  if (routePath.startsWith('api/')) {
    return null;
  }

  // For other routes, try a generic mapping
  return `/test-data/${routePath}/index.html`;
};

/**
 * Prefetches HTML content from S3 for a given route
 */
const prefetchRoute = async (pathname) => {
  const s3Path = mapRouteToS3Path(pathname);
  
  if (!s3Path) {
    console.debug(`Skipping prefetch for ${pathname} - no S3 mapping`);
    return;
  }

  const s3Url = `${getS3BaseUrl()}${s3Path}`;
  
  if (prefetchCache.has(s3Url)) {
    console.debug(`Already prefetched: ${s3Url}`);
    return;
  }

  prefetchCache.add(s3Url);
  
  try {
    console.debug(`Prefetching: ${s3Url}`);
    const response = await fetch(s3Url, {
      method: 'GET',
      // Use no-cache to ensure we get fresh content but store in browser cache
      cache: 'no-cache'
    });
    
    if (response.ok) {
      // Browser will cache this for future requests
      await response.text();
      console.debug(`Successfully prefetched: ${pathname} -> ${s3Url}`);
    } else {
      console.debug(`Failed to prefetch ${pathname}: HTTP ${response.status}`);
    }
  } catch (error) {
    console.debug(`Error prefetching ${pathname}:`, error.message);
  }
};

/**
 * Scans the page for internal links and prefetches their HTML
 */
const prefetchPageLinks = () => {
  // Find all links on the page
  const links = document.querySelectorAll('a[href]');
  const currentOrigin = window.location.origin;
  
  const routesToPrefetch = new Set();
  
  links.forEach(link => {
    const href = link.getAttribute('href');
    
    if (!href) return;
    
    // Handle relative URLs and same-origin absolute URLs
    let url;
    try {
      url = new URL(href, window.location.href);
    } catch (e) {
      return; // Invalid URL, skip
    }
    
    // Only prefetch internal links (same origin)
    if (url.origin === currentOrigin) {
      routesToPrefetch.add(url.pathname);
    }
  });

  // Prefetch all discovered routes
  routesToPrefetch.forEach(pathname => {
    // Small delay to avoid overwhelming the server
    setTimeout(() => prefetchRoute(pathname), Math.random() * 1000);
  });
  
  console.debug(`Found ${routesToPrefetch.size} internal links to prefetch`);
};

/**
 * Initialize prefetching on page load
 */
const initializePrefetching = () => {
  // Prefetch links when DOM is ready
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', prefetchPageLinks);
  } else {
    prefetchPageLinks();
  }
  
  // Also prefetch when new content is added dynamically
  const observer = new MutationObserver(mutations => {
    let hasNewLinks = false;
    mutations.forEach(mutation => {
      if (mutation.type === 'childList') {
        mutation.addedNodes.forEach(node => {
          if (node.nodeType === Node.ELEMENT_NODE) {
            if (node.tagName === 'A' || node.querySelector('a')) {
              hasNewLinks = true;
            }
          }
        });
      }
    });
    
    if (hasNewLinks) {
      // Debounce to avoid excessive prefetching
      clearTimeout(window.prefetchTimeout);
      window.prefetchTimeout = setTimeout(prefetchPageLinks, 500);
    }
  });
  
  observer.observe(document.body, {
    childList: true,
    subtree: true
  });
  
  console.debug('S3 HTML prefetching initialized');
};

// Auto-initialize when script loads
if (typeof window !== 'undefined') {
  initializePrefetching();
}

// Export for manual use if needed
export { prefetchRoute, prefetchPageLinks, mapRouteToS3Path };