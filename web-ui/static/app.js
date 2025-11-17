// Toyota MyT2ABRP Dashboard JavaScript
// Clean JSON API architecture with client-side rendering

// ============================================================================
// API Fetching Functions
// ============================================================================

/**
 * Generic fetch wrapper with error handling
 */
async function fetchJSON(url, options = {}) {
    try {
        const response = await fetch(url, {
            ...options,
            headers: {
                'Content-Type': 'application/json',
                ...options.headers
            }
        });

        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        return await response.json();
    } catch (error) {
        console.error(`Failed to fetch ${url}:`, error);
        showNotification(`Failed to load data from ${url}`, 'error');
        throw error;
    }
}

/**
 * Fetch and render vehicle status
 */
async function loadVehicleStatus() {
    const data = await fetchJSON('/api/vehicle/status');
    const html = `
        <div>
            <h2>Vehicle Status</h2>
            <div class="battery-indicator">
                <div class="battery-icon">
                    <div class="battery-level" style="width: ${data.battery_level}%"></div>
                </div>
                <div class="battery-percentage">${data.battery_level}%</div>
            </div>
            <div class="status-grid">
                <div class="stat">
                    <div class="stat-label">VIN</div>
                    <div class="stat-value">${data.vin}</div>
                </div>
                <div class="stat">
                    <div class="stat-label">Range</div>
                    <div class="stat-value">${data.range_km} km</div>
                </div>
                <div class="stat">
                    <div class="stat-label">Status</div>
                    <div class="stat-value">${data.is_charging ? '⚡ Charging' : '🅿️ Parked'}</div>
                </div>
            </div>
        </div>
    `;
    document.getElementById('vehicle-status').innerHTML = html;
}

/**
 * Fetch and render charging status
 */
async function loadChargingStatus() {
    const data = await fetchJSON('/api/charging/status');
    const timeRemaining = data.time_remaining_minutes
        ? formatTimeRemaining(data.time_remaining_minutes)
        : 'N/A';

    const html = `
        <div>
            <h2>Charging Status</h2>
            <div class="charging-progress">
                <div class="progress-bar">
                    <div class="progress-fill" style="width: ${data.current_level}%"></div>
                </div>
                <div style="margin-top: 10px; font-size: 14px;">
                    ${data.current_level}% → ${data.target_level}% target
                </div>
            </div>
            <div class="charging-stats">
                <div class="stat">
                    <div class="stat-value">${data.power_kw.toFixed(1)} kW</div>
                    <div class="stat-label">Power</div>
                </div>
                <div class="stat">
                    <div class="stat-value">${timeRemaining}</div>
                    <div class="stat-label">Time Left</div>
                </div>
                <div class="stat">
                    <div class="stat-value">${data.charge_rate_kwh.toFixed(1)} kWh</div>
                    <div class="stat-label">Rate</div>
                </div>
            </div>
        </div>
    `;
    document.getElementById('charging-status').innerHTML = html;
}

/**
 * Fetch and render range information
 */
async function loadRange() {
    const data = await fetchJSON('/api/range');
    const html = `
        <div class="range-info">
            <div>
                <div class="stat-label">Estimated Range</div>
                <div class="range-value">${data.estimated_range_km} km</div>
            </div>
            <div>
                <div class="stat-label">Range @ 80%</div>
                <div class="stat-value">${data.range_at_80_percent_km} km</div>
            </div>
        </div>
    `;
    document.getElementById('range-info').innerHTML = html;
}

/**
 * Fetch and render battery health
 */
async function loadBatteryHealth() {
    const data = await fetchJSON('/api/battery/health');
    const html = `
        <div>
            <div class="health-status" style="text-align: center; margin: 20px 0;">
                <div style="font-size: 48px; font-weight: 700; color: var(--success-color);">
                    ${data.capacity_percentage}%
                </div>
                <div style="font-size: 18px; margin-top: 10px;">
                    ${data.health_status} Health
                </div>
            </div>
            <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 15px;">
                <div class="stat">
                    <div class="stat-label">Charge Cycles</div>
                    <div class="stat-value">${data.cycles}</div>
                </div>
                <div class="stat">
                    <div class="stat-label">Temperature</div>
                    <div class="stat-value">${data.temperature_celsius.toFixed(1)}°C</div>
                </div>
            </div>
        </div>
    `;
    document.getElementById('battery-health').innerHTML = html;
}

/**
 * Fetch and render charging history
 */
async function loadChargingHistory() {
    const data = await fetchJSON('/api/charging/history');
    const items = data.map(session => `
        <div class="history-item">
            <div><strong>${session.date}</strong></div>
            <div>${session.start_level}% → ${session.end_level}% | ${formatTimeRemaining(session.duration_minutes)} | ${session.energy_kwh} kWh</div>
        </div>
    `).join('');

    const html = `
        <div class="history-list">
            ${items}
        </div>
        <style>
            .history-list { display: flex; flex-direction: column; gap: 10px; }
            .history-item { padding: 12px; background: var(--background); border-radius: 8px; }
            .history-item div:last-child { font-size: 13px; color: var(--text-secondary); margin-top: 5px; }
        </style>
    `;
    document.getElementById('charging-history').innerHTML = html;
}

/**
 * Fetch and render active alerts
 */
async function loadActiveAlerts() {
    const data = await fetchJSON('/api/alerts/active');
    const items = data.map(alert => `
        <div class="alert-item ${alert.type}">
            <div><strong>${alert.title}</strong></div>
            <div>${alert.message}</div>
            <div style="font-size: 12px; color: var(--text-secondary); margin-top: 5px;">${alert.time_ago}</div>
        </div>
    `).join('');

    const html = `
        <div class="alerts-list">
            ${items}
        </div>
        <style>
            .alerts-list { display: flex; flex-direction: column; gap: 10px; }
            .alert-item.success { border-left-color: var(--success-color); }
        </style>
    `;
    document.getElementById('active-alerts').innerHTML = html;
}

/**
 * Fetch and render weekly analytics
 */
async function loadWeeklyAnalytics() {
    const data = await fetchJSON('/api/analytics/weekly');
    const html = `
        <div>
            <h3>Weekly Charging Stats</h3>
            <div style="margin: 20px 0;">
                <div class="stat">
                    <div class="stat-value">${data.charging_sessions}</div>
                    <div class="stat-label">Charging Sessions</div>
                </div>
                <div class="stat" style="margin-top: 15px;">
                    <div class="stat-value">${data.total_energy_kwh} kWh</div>
                    <div class="stat-label">Total Energy</div>
                </div>
                <div class="stat" style="margin-top: 15px;">
                    <div class="stat-value">${formatTimeRemaining(data.avg_duration_minutes)}</div>
                    <div class="stat-label">Avg. Duration</div>
                </div>
            </div>
        </div>
    `;
    document.getElementById('weekly-analytics').innerHTML = html;
}

/**
 * Fetch and render cost analytics
 */
async function loadCostAnalytics() {
    const data = await fetchJSON('/api/analytics/costs');
    const html = `
        <div>
            <h3>Cost Analysis</h3>
            <div style="margin: 20px 0;">
                <div class="stat">
                    <div class="stat-value">€${data.this_week_cost.toFixed(2)}</div>
                    <div class="stat-label">This Week</div>
                </div>
                <div class="stat" style="margin-top: 15px;">
                    <div class="stat-value">€${data.per_session_avg.toFixed(2)}</div>
                    <div class="stat-label">Per Session Avg.</div>
                </div>
                <div class="stat" style="margin-top: 15px;">
                    <div class="stat-value">€${data.avg_price_per_kwh.toFixed(3)}/kWh</div>
                    <div class="stat-label">Avg. Price</div>
                </div>
            </div>
        </div>
    `;
    document.getElementById('cost-analytics').innerHTML = html;
}

/**
 * Fetch and render efficiency analytics
 */
async function loadEfficiencyAnalytics() {
    const data = await fetchJSON('/api/analytics/efficiency');
    const html = `
        <div>
            <h3>Efficiency Metrics</h3>
            <div style="margin: 20px 0;">
                <div class="stat">
                    <div class="stat-value">${data.charging_efficiency_percent}%</div>
                    <div class="stat-label">Charging Efficiency</div>
                </div>
                <div class="stat" style="margin-top: 15px;">
                    <div class="stat-value">${data.avg_consumption_kwh_per_100km} kWh/100km</div>
                    <div class="stat-label">Avg. Consumption</div>
                </div>
                <div class="stat" style="margin-top: 15px;">
                    <div class="stat-value">${data.battery_health_percent}%</div>
                    <div class="stat-label">Battery Health</div>
                </div>
            </div>
        </div>
    `;
    document.getElementById('efficiency-analytics').innerHTML = html;
}

// ============================================================================
// Action Handlers
// ============================================================================

/**
 * Handle charging control actions
 */
async function handleAction(action, endpoint) {
    try {
        const result = await fetchJSON(endpoint, { method: 'POST' });
        if (result.success) {
            showNotification(result.message, 'success');
            // Refresh relevant data
            if (action.includes('charging')) {
                await loadChargingStatus();
            }
        } else {
            showNotification(result.message || 'Action failed', 'error');
        }
    } catch (error) {
        showNotification(`Failed to ${action}`, 'error');
    }
}

// ============================================================================
// UI Functions
// ============================================================================

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

    // Load section-specific data
    loadSectionData(sectionId);
}

/**
 * Load data for a specific section
 */
function loadSectionData(sectionId) {
    switch(sectionId) {
        case 'dashboard':
            loadVehicleStatus();
            loadChargingStatus();
            loadRange();
            loadBatteryHealth();
            loadChargingHistory();
            break;
        case 'alerts':
            loadActiveAlerts();
            break;
        case 'analytics':
            loadWeeklyAnalytics();
            loadCostAnalytics();
            loadEfficiencyAnalytics();
            break;
    }
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

// ============================================================================
// Auto-refresh and Initialization
// ============================================================================

// Auto-refresh intervals (in milliseconds)
const REFRESH_INTERVALS = {
    vehicleStatus: 5000,   // 5 seconds
    chargingStatus: 2000,  // 2 seconds
    range: 10000,          // 10 seconds
    batteryHealth: 60000,  // 60 seconds
    alerts: 3000,          // 3 seconds
};

let refreshTimers = {};

/**
 * Start auto-refresh for dashboard elements
 */
function startAutoRefresh() {
    // Clear any existing timers
    stopAutoRefresh();

    // Set up new refresh timers
    refreshTimers.vehicleStatus = setInterval(loadVehicleStatus, REFRESH_INTERVALS.vehicleStatus);
    refreshTimers.chargingStatus = setInterval(loadChargingStatus, REFRESH_INTERVALS.chargingStatus);
    refreshTimers.range = setInterval(loadRange, REFRESH_INTERVALS.range);
    refreshTimers.batteryHealth = setInterval(loadBatteryHealth, REFRESH_INTERVALS.batteryHealth);
    refreshTimers.alerts = setInterval(loadActiveAlerts, REFRESH_INTERVALS.alerts);
}

/**
 * Stop all auto-refresh timers
 */
function stopAutoRefresh() {
    Object.values(refreshTimers).forEach(timer => clearInterval(timer));
    refreshTimers = {};
}

/**
 * Initialize the dashboard
 */
document.addEventListener('DOMContentLoaded', () => {
    console.log('🚗 Toyota MyT2ABRP Dashboard loaded (JSON API mode)');

    // Load initial data for dashboard
    loadVehicleStatus();
    loadChargingStatus();
    loadRange();
    loadBatteryHealth();
    loadChargingHistory();

    // Start auto-refresh
    startAutoRefresh();

    // Set up action button handlers
    document.querySelectorAll('[data-action]').forEach(button => {
        button.addEventListener('click', async (e) => {
            const action = e.target.dataset.action;
            const endpoint = e.target.dataset.endpoint;
            await handleAction(action, endpoint);
        });
    });

    // Set up form submission handlers
    document.querySelectorAll('form[data-api]').forEach(form => {
        form.addEventListener('submit', async (e) => {
            e.preventDefault();
            const endpoint = form.dataset.api;
            const formData = new FormData(form);
            const data = Object.fromEntries(formData);

            try {
                const result = await fetchJSON(endpoint, {
                    method: 'POST',
                    body: JSON.stringify(data)
                });

                if (result.success) {
                    showNotification(result.message || 'Settings saved successfully!');
                } else {
                    showNotification(result.message || 'Failed to save settings', 'error');
                }
            } catch (error) {
                showNotification('Failed to save settings', 'error');
            }
        });
    });

    // Initialize UI enhancements
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

// Export for use in inline event handlers
window.showSection = showSection;
window.handleAction = handleAction;
window.dashboardUtils = {
    showNotification,
    formatTimeRemaining,
    getBatteryIcon,
    getChargingSpeedColor,
    showSystemNotification,
    requestNotificationPermission
};
