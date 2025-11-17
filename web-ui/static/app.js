// Toyota MyT2ABRP Dashboard JavaScript

// Section navigation
function showSection(sectionId) {
    // Hide all sections
    document.querySelectorAll('.section').forEach(section => {
        section.classList.remove('active');
    });

    // Remove active state from all nav buttons
    document.querySelectorAll('.nav-btn').forEach(btn => {
        btn.classList.remove('active');
    });

    // Show selected section
    document.getElementById(sectionId).classList.add('active');

    // Highlight active nav button
    event.target.classList.add('active');
}

// Show notification toast
function showNotification(message, type = 'success') {
    const notification = document.getElementById('notification');
    notification.textContent = message;
    notification.className = `notification ${type} show`;

    setTimeout(() => {
        notification.classList.remove('show');
    }, 3000);
}

// Format time remaining
function formatTimeRemaining(minutes) {
    if (minutes < 60) {
        return `${minutes} min`;
    }
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;
    return `${hours}h ${mins}m`;
}

// Format battery percentage with icon
function getBatteryIcon(percentage) {
    if (percentage >= 80) return '🔋';
    if (percentage >= 50) return '🔋';
    if (percentage >= 20) return '🪫';
    return '🪫';
}

// Get charging speed color
function getChargingSpeedColor(kw) {
    if (kw >= 100) return 'var(--success-color)';  // DC Fast
    if (kw >= 7) return 'var(--warning-color)';     // AC Fast
    return 'var(--danger-color)';                    // Slow
}

// Auto-refresh logic
document.addEventListener('DOMContentLoaded', () => {
    console.log('🚗 Toyota MyT2ABRP Dashboard loaded');

    // Listen for HTMX events
    document.body.addEventListener('htmx:afterSwap', (event) => {
        console.log('Content updated:', event.detail.target.id);
    });

    document.body.addEventListener('htmx:responseError', (event) => {
        console.error('Error loading content:', event.detail);
        showNotification('Failed to load data. Retrying...', 'error');
    });

    // Handle form submissions
    document.body.addEventListener('htmx:afterRequest', (event) => {
        if (event.detail.successful) {
            const target = event.detail.target;
            if (target.tagName === 'FORM') {
                showNotification('Settings saved successfully!');
            }
        }
    });

    // Initialize tooltips and other UI enhancements
    initializeUI();
});

function initializeUI() {
    // Add smooth scroll behavior
    document.documentElement.style.scrollBehavior = 'smooth';

    // Add keyboard shortcuts
    document.addEventListener('keydown', (e) => {
        // Alt+1/2/3/4 for quick navigation
        if (e.altKey) {
            switch(e.key) {
                case '1':
                    document.querySelector('.nav-btn:nth-child(1)').click();
                    break;
                case '2':
                    document.querySelector('.nav-btn:nth-child(2)').click();
                    break;
                case '3':
                    document.querySelector('.nav-btn:nth-child(3)').click();
                    break;
                case '4':
                    document.querySelector('.nav-btn:nth-child(4)').click();
                    break;
            }
        }
    });

    // Add touch gestures for mobile (swipe between sections)
    let touchStartX = 0;
    let touchEndX = 0;

    document.addEventListener('touchstart', e => {
        touchStartX = e.changedTouches[0].screenX;
    });

    document.addEventListener('touchend', e => {
        touchEndX = e.changedTouches[0].screenX;
        handleSwipe();
    });

    function handleSwipe() {
        const swipeThreshold = 50;
        const diff = touchStartX - touchEndX;

        if (Math.abs(diff) > swipeThreshold) {
            const sections = ['dashboard', 'alerts', 'analytics', 'settings'];
            const currentSection = document.querySelector('.section.active').id;
            const currentIndex = sections.indexOf(currentSection);

            if (diff > 0 && currentIndex < sections.length - 1) {
                // Swipe left - next section
                showSection(sections[currentIndex + 1]);
            } else if (diff < 0 && currentIndex > 0) {
                // Swipe right - previous section
                showSection(sections[currentIndex - 1]);
            }
        }
    }
}

// PWA Service Worker registration (for offline support)
if ('serviceWorker' in navigator) {
    window.addEventListener('load', () => {
        navigator.serviceWorker.register('/sw.js')
            .then(registration => console.log('SW registered:', registration))
            .catch(err => console.log('SW registration failed:', err));
    });
}

// Web Notifications API
function requestNotificationPermission() {
    if ('Notification' in window && Notification.permission === 'default') {
        Notification.requestPermission().then(permission => {
            if (permission === 'granted') {
                showNotification('Push notifications enabled!');
            }
        });
    }
}

// Show system notification
function showSystemNotification(title, options = {}) {
    if ('Notification' in window && Notification.permission === 'granted') {
        new Notification(title, {
            icon: '/icon-192.png',
            badge: '/badge-96.png',
            ...options
        });
    }
}

// Export for use in HTMX responses
window.dashboardUtils = {
    showNotification,
    formatTimeRemaining,
    getBatteryIcon,
    getChargingSpeedColor,
    showSystemNotification,
    requestNotificationPermission
};
