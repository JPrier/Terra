// Terra Marketplace JavaScript

// Sample product data (in a real app, this would come from the backend API)
const sampleProducts = [
    {
        id: 1,
        title: "Wireless Bluetooth Headphones",
        price: 79.99,
        category: "electronics",
        rating: 4.5,
        reviews: 234,
        image: "🎧"
    },
    {
        id: 2,
        title: "Classic Cotton T-Shirt",
        price: 24.99,
        category: "fashion",
        rating: 4.2,
        reviews: 156,
        image: "👕"
    },
    {
        id: 3,
        title: "Smart Home Security Camera",
        price: 149.99,
        category: "electronics",
        rating: 4.7,
        reviews: 89,
        image: "📹"
    },
    {
        id: 4,
        title: "Ceramic Plant Pot Set",
        price: 34.99,
        category: "home",
        rating: 4.3,
        reviews: 67,
        image: "🪴"
    },
    {
        id: 5,
        title: "Programming for Beginners",
        price: 29.99,
        category: "books",
        rating: 4.6,
        reviews: 123,
        image: "📚"
    },
    {
        id: 6,
        title: "Gaming Mechanical Keyboard",
        price: 89.99,
        category: "gaming",
        rating: 4.8,
        reviews: 201,
        image: "⌨️"
    },
    {
        id: 7,
        title: "Yoga Mat Premium",
        price: 39.99,
        category: "sports",
        rating: 4.4,
        reviews: 98,
        image: "🧘"
    },
    {
        id: 8,
        title: "Vintage Leather Wallet",
        price: 59.99,
        category: "fashion",
        rating: 4.5,
        reviews: 175,
        image: "👛"
    }
];

// Shopping cart
let cart = [];

// DOM elements
const productsGrid = document.getElementById('products-grid');
const cartBtn = document.getElementById('cart-btn');
const cartCount = document.getElementById('cart-count');
const cartModal = document.getElementById('cart-modal');
const closeCartBtn = document.getElementById('close-cart');
const cartItems = document.getElementById('cart-items');
const cartTotal = document.getElementById('cart-total');
const searchInput = document.getElementById('search-input');
const searchBtn = document.getElementById('search-btn');
const sortFilter = document.getElementById('sort-filter');
const categoryFilter = document.getElementById('category-filter');

// Initialize the application
document.addEventListener('DOMContentLoaded', function() {
    loadProducts();
    setupEventListeners();
    updateCartUI();
});

// Event listeners
function setupEventListeners() {
    // Cart modal
    cartBtn.addEventListener('click', openCartModal);
    closeCartBtn.addEventListener('click', closeCartModal);
    
    // Close modal when clicking outside
    cartModal.addEventListener('click', function(e) {
        if (e.target === cartModal) {
            closeCartModal();
        }
    });

    // Search functionality
    searchBtn.addEventListener('click', handleSearch);
    searchInput.addEventListener('keypress', function(e) {
        if (e.key === 'Enter') {
            handleSearch();
        }
    });

    // Filters
    sortFilter.addEventListener('change', handleFilters);
    categoryFilter.addEventListener('change', handleFilters);

    // Smooth scrolling for navigation
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
        anchor.addEventListener('click', function (e) {
            e.preventDefault();
            const target = document.querySelector(this.getAttribute('href'));
            if (target) {
                target.scrollIntoView({
                    behavior: 'smooth',
                    block: 'start'
                });
            }
        });
    });

    // CTA button
    document.querySelector('.cta-button').addEventListener('click', function() {
        document.getElementById('products').scrollIntoView({
            behavior: 'smooth'
        });
    });

    // Category cards
    document.querySelectorAll('.category-card').forEach(card => {
        card.addEventListener('click', function() {
            const categoryName = this.querySelector('h3').textContent.toLowerCase();
            filterByCategory(categoryName);
        });
    });
}

// Load and display products
function loadProducts(products = sampleProducts) {
    productsGrid.innerHTML = '';
    
    if (products.length === 0) {
        productsGrid.innerHTML = '<div class="no-products">No products found matching your criteria.</div>';
        return;
    }

    products.forEach(product => {
        const productCard = createProductCard(product);
        productsGrid.appendChild(productCard);
    });
}

// Create product card element
function createProductCard(product) {
    const card = document.createElement('div');
    card.className = 'product-card';
    card.innerHTML = `
        <div class="product-image">${product.image}</div>
        <div class="product-info">
            <h3 class="product-title">${product.title}</h3>
            <div class="product-price">$${product.price.toFixed(2)}</div>
            <div class="product-rating">
                <div class="stars">${generateStars(product.rating)}</div>
                <span class="rating-text">(${product.reviews} reviews)</span>
            </div>
            <button class="add-to-cart" onclick="addToCart(${product.id})">
                Add to Cart
            </button>
        </div>
    `;
    return card;
}

// Generate star rating HTML
function generateStars(rating) {
    const fullStars = Math.floor(rating);
    const hasHalfStar = rating % 1 !== 0;
    let starsHTML = '';

    for (let i = 0; i < fullStars; i++) {
        starsHTML += '<i class="fas fa-star"></i>';
    }

    if (hasHalfStar) {
        starsHTML += '<i class="fas fa-star-half-alt"></i>';
    }

    const emptyStars = 5 - Math.ceil(rating);
    for (let i = 0; i < emptyStars; i++) {
        starsHTML += '<i class="far fa-star"></i>';
    }

    return starsHTML;
}

// Add product to cart
function addToCart(productId) {
    const product = sampleProducts.find(p => p.id === productId);
    if (!product) return;

    const existingItem = cart.find(item => item.id === productId);
    
    if (existingItem) {
        existingItem.quantity += 1;
    } else {
        cart.push({
            ...product,
            quantity: 1
        });
    }

    updateCartUI();
    showNotification(`${product.title} added to cart!`);
}

// Remove product from cart
function removeFromCart(productId) {
    cart = cart.filter(item => item.id !== productId);
    updateCartUI();
    renderCartItems();
}

// Update cart quantity
function updateCartQuantity(productId, newQuantity) {
    const item = cart.find(item => item.id === productId);
    if (item) {
        if (newQuantity <= 0) {
            removeFromCart(productId);
        } else {
            item.quantity = newQuantity;
            updateCartUI();
            renderCartItems();
        }
    }
}

// Update cart UI
function updateCartUI() {
    const totalItems = cart.reduce((sum, item) => sum + item.quantity, 0);
    const totalPrice = cart.reduce((sum, item) => sum + (item.price * item.quantity), 0);
    
    cartCount.textContent = totalItems;
    cartTotal.textContent = totalPrice.toFixed(2);
    
    // Hide cart count if empty
    if (totalItems === 0) {
        cartCount.style.display = 'none';
    } else {
        cartCount.style.display = 'flex';
    }
}

// Open cart modal
function openCartModal() {
    cartModal.style.display = 'block';
    renderCartItems();
}

// Close cart modal
function closeCartModal() {
    cartModal.style.display = 'none';
}

// Render cart items in modal
function renderCartItems() {
    if (cart.length === 0) {
        cartItems.innerHTML = '<div class="empty-cart">Your cart is empty</div>';
        return;
    }

    cartItems.innerHTML = cart.map(item => `
        <div class="cart-item">
            <div class="cart-item-info">
                <h4>${item.title}</h4>
                <div class="cart-item-price">$${item.price.toFixed(2)} × 
                    <input type="number" value="${item.quantity}" min="1" 
                           style="width: 60px; padding: 2px 5px; border: 1px solid #ccc; border-radius: 4px;"
                           onchange="updateCartQuantity(${item.id}, parseInt(this.value))">
                    = $${(item.price * item.quantity).toFixed(2)}
                </div>
            </div>
            <button onclick="removeFromCart(${item.id})" 
                    style="background: #e53e3e; color: white; border: none; padding: 5px 10px; border-radius: 4px; cursor: pointer;">
                Remove
            </button>
        </div>
    `).join('');
}

// Search functionality
function handleSearch() {
    const searchTerm = searchInput.value.toLowerCase().trim();
    
    if (searchTerm === '') {
        loadProducts();
        return;
    }

    const filteredProducts = sampleProducts.filter(product =>
        product.title.toLowerCase().includes(searchTerm) ||
        product.category.toLowerCase().includes(searchTerm)
    );

    loadProducts(filteredProducts);
}

// Filter and sort functionality
function handleFilters() {
    let filteredProducts = [...sampleProducts];
    
    // Apply category filter
    const selectedCategory = categoryFilter.value;
    if (selectedCategory !== 'all') {
        filteredProducts = filteredProducts.filter(product => 
            product.category === selectedCategory
        );
    }

    // Apply search filter if there's a search term
    const searchTerm = searchInput.value.toLowerCase().trim();
    if (searchTerm !== '') {
        filteredProducts = filteredProducts.filter(product =>
            product.title.toLowerCase().includes(searchTerm) ||
            product.category.toLowerCase().includes(searchTerm)
        );
    }

    // Apply sorting
    const sortOption = sortFilter.value;
    switch (sortOption) {
        case 'price-low':
            filteredProducts.sort((a, b) => a.price - b.price);
            break;
        case 'price-high':
            filteredProducts.sort((a, b) => b.price - a.price);
            break;
        case 'rating':
            filteredProducts.sort((a, b) => b.rating - a.rating);
            break;
        default:
            // Keep original order for 'featured'
            break;
    }

    loadProducts(filteredProducts);
}

// Filter by category (used by category cards)
function filterByCategory(categoryName) {
    // Map category names to filter values
    const categoryMap = {
        'fashion': 'fashion',
        'electronics': 'electronics',
        'home & garden': 'home',
        'books': 'books',
        'gaming': 'gaming',
        'sports': 'sports'
    };

    const filterValue = categoryMap[categoryName] || categoryName;
    categoryFilter.value = filterValue;
    handleFilters();
    
    // Scroll to products section
    document.getElementById('products').scrollIntoView({
        behavior: 'smooth'
    });
}

// Show notification
function showNotification(message) {
    // Create notification element
    const notification = document.createElement('div');
    notification.textContent = message;
    notification.style.cssText = `
        position: fixed;
        top: 100px;
        right: 20px;
        background: #48bb78;
        color: white;
        padding: 15px 20px;
        border-radius: 8px;
        box-shadow: 0 4px 15px rgba(0,0,0,0.2);
        z-index: 3000;
        font-weight: 600;
        transform: translateX(100%);
        transition: transform 0.3s ease;
    `;

    document.body.appendChild(notification);

    // Animate in
    setTimeout(() => {
        notification.style.transform = 'translateX(0)';
    }, 100);

    // Animate out and remove
    setTimeout(() => {
        notification.style.transform = 'translateX(100%)';
        setTimeout(() => {
            document.body.removeChild(notification);
        }, 300);
    }, 3000);
}

// Simulate API calls for future backend integration
class MarketplaceAPI {
    static async getProducts(filters = {}) {
        // Simulate API call delay
        await new Promise(resolve => setTimeout(resolve, 500));
        
        // In a real app, this would make an HTTP request to the Rust backend
        return {
            success: true,
            data: sampleProducts,
            total: sampleProducts.length
        };
    }

    static async addToCart(productId, quantity = 1) {
        // Simulate API call
        await new Promise(resolve => setTimeout(resolve, 200));
        
        return {
            success: true,
            message: 'Product added to cart'
        };
    }

    static async updateCart(cartItems) {
        // Simulate API call
        await new Promise(resolve => setTimeout(resolve, 300));
        
        return {
            success: true,
            message: 'Cart updated'
        };
    }

    static async checkout(cartItems) {
        // Simulate API call
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        return {
            success: true,
            orderId: 'ORDER_' + Date.now(),
            message: 'Order placed successfully'
        };
    }
}

// Initialize lazy loading for future use
function initializeLazyLoading() {
    const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                // Load more products when user scrolls near bottom
                console.log('Load more products...');
            }
        });
    });

    const loading = document.getElementById('loading');
    if (loading) {
        observer.observe(loading);
    }
}

// Add smooth scroll behavior for better UX
function addSmoothScrollBehavior() {
    // Add smooth scrolling to all anchor links
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
        anchor.addEventListener('click', function (e) {
            e.preventDefault();
            const targetId = this.getAttribute('href');
            const targetElement = document.querySelector(targetId);
            
            if (targetElement) {
                const offsetTop = targetElement.offsetTop - 80; // Account for fixed header
                window.scrollTo({
                    top: offsetTop,
                    behavior: 'smooth'
                });
            }
        });
    });
}

// Initialize additional features
document.addEventListener('DOMContentLoaded', function() {
    initializeLazyLoading();
    addSmoothScrollBehavior();
});

// Export functions for potential module use
if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
        MarketplaceAPI,
        addToCart,
        removeFromCart,
        updateCartQuantity
    };
}