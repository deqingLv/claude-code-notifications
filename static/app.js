// Global state
let config = null;
let availableChannels = [];

// Channel icons and display names (sorted alphabetically except system first)
const channelInfo = {
    system: { icon: '💻', name: 'System Notification', description: 'Desktop notifications' },
    dingtalk: { icon: '📢', name: 'DingTalk', description: 'DingTalk notifications' },
    feishu: { icon: '🚀', name: 'Feishu/Lark', description: 'Feishu/Lark notifications' },
    wechat: { icon: '💬', name: 'WeChat Work', description: 'Enterprise WeChat notifications' }
};

// Helper function to get display name for a channel
function getChannelDisplayName(channelId, channelConfig) {
    if (!channelConfig) return channelId;
    const channelType = channelConfig.channel_type || channelId;
    const info = channelInfo[channelType] || { icon: '🔔', name: channelType, description: '' };
    return channelConfig.name || info.name;
}

// Helper function to get sorted channel IDs (system first, then alphabetically by display name)
function getSortedChannelIds() {
    if (!config || !config.channels) return [];

    const channelTypes = ['system', 'dingtalk', 'feishu', 'wechat'];
    return Object.keys(config.channels).sort((a, b) => {
        const configA = config.channels[a];
        const configB = config.channels[b];

        // Get channel type (default to channel_id for backward compatibility)
        const typeA = configA.channel_type || a;
        const typeB = configB.channel_type || b;

        // System always first
        if (typeA === 'system') return -1;
        if (typeB === 'system') return 1;

        // Then sort by type
        const indexA = channelTypes.indexOf(typeA);
        const indexB = channelTypes.indexOf(typeB);
        if (indexA !== indexB) {
            return (indexA === -1 ? 999 : indexA) - (indexB === -1 ? 999 : indexB);
        }

        // Same type, sort by display name
        const nameA = getChannelDisplayName(a, configA);
        const nameB = getChannelDisplayName(b, configB);
        return nameA.localeCompare(nameB);
    });
}

// Initialize on page load
document.addEventListener('DOMContentLoaded', async () => {
    await loadAvailableChannels();
    await loadConfiguration();
});

// Load available channels from the API
async function loadAvailableChannels() {
    try {
        const response = await fetch('/api/channels');
        const data = await response.json();
        availableChannels = data.channels;
        console.log('Available channels:', availableChannels);
    } catch (error) {
        console.error('Failed to load channels:', error);
        showStatus('Failed to load channels', 'error');
    }
}

// Load configuration from the API
async function loadConfiguration() {
    showStatus('Loading configuration...', 'loading');

    try {
        const response = await fetch('/api/config');
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }

        config = await response.json();
        console.log('Configuration loaded:', config);

        renderChannels();
        renderRoutingRules();
        renderTemplates();

        showStatus('Configuration loaded successfully', 'success');
    } catch (error) {
        console.error('Failed to load configuration:', error);
        showStatus('Failed to load configuration: ' + error.message, 'error');
    }
}

// Render all channels
function renderChannels() {
    const container = document.getElementById('channels-container');
    container.innerHTML = '';

    // Get sorted channel IDs (system first, then by type and display name)
    const sortedChannelIds = getSortedChannelIds();

    for (const channelId of sortedChannelIds) {
        const channelConfig = config.channels[channelId];
        const channelType = channelConfig.channel_type || channelId;
        const info = channelInfo[channelType] || { icon: '🔔', name: channelType, description: '' };
        const displayName = getChannelDisplayName(channelId, channelConfig);

        const card = document.createElement('div');
        card.className = `channel-card ${!channelConfig.enabled ? 'disabled' : ''}`;
        card.innerHTML = `
            <div class="channel-header">
                <div class="channel-title">
                    <span class="channel-icon">${info.icon}</span>
                    <span>${displayName}</span>
                    <span style="font-size: 0.8rem; color: #9ca3af; margin-left: 8px;">(${channelId})</span>
                </div>
                <div class="checkbox-group">
                    <input
                        type="checkbox"
                        id="channel-${channelId}-enabled"
                        ${channelConfig.enabled ? 'checked' : ''}
                        onchange="toggleChannel('${channelId}')"
                    >
                    <label for="channel-${channelId}-enabled">Enabled</label>
                </div>
            </div>
            <p style="margin-bottom: 15px; color: #6b7280;">${info.description}</p>
            <div class="form-group">
                <label>Channel ID</label>
                <input
                    type="text"
                    id="channel-${channelId}-id"
                    value="${channelId}"
                    readonly
                    style="background: #f3f4f6; color: #6b7280;"
                >
            </div>
            <div class="form-group">
                <label>Channel Type</label>
                <input
                    type="text"
                    id="channel-${channelId}-type"
                    value="${channelType}"
                    readonly
                    style="background: #f3f4f6; color: #6b7280;"
                >
            </div>
            <div class="form-group">
                <label>Display Name</label>
                <input
                    type="text"
                    id="channel-${channelId}-name"
                    value="${channelConfig.name || ''}"
                    placeholder="Enter display name"
                    onchange="updateChannelConfig('${channelId}', 'name', this.value)"
                >
            </div>
            ${channelType !== 'system' ? `
                <div class="form-group">
                    <label>Webhook URL</label>
                    <input
                        type="url"
                        id="channel-${channelId}-webhook-url"
                        value="${channelConfig.webhook_url || ''}"
                        placeholder="https://..."
                        onchange="updateChannelConfig('${channelId}', 'webhook_url', this.value)"
                    >
                </div>
            ` : ''}
            ${channelType === 'dingtalk' ? `
                <div class="form-group">
                    <label>Secret (optional, for signing)</label>
                    <input
                        type="password"
                        id="channel-${channelId}-secret"
                        value="${channelConfig.secret || ''}"
                        placeholder="SEC..."
                        onchange="updateChannelConfig('${channelId}', 'secret', this.value)"
                    >
                </div>
            ` : ''}
            ${channelType === 'system' ? `
                <div class="form-group">
                    <label>Sound</label>
                    <input
                        type="text"
                        id="channel-${channelId}-sound"
                        value="${channelConfig.sound || 'Glass'}"
                        placeholder="Glass"
                        onchange="updateChannelConfig('${channelId}', 'sound', this.value)"
                    >
                </div>
            ` : ''}
            <div class="form-group">
                <label>Icon URL (optional)</label>
                <input
                    type="url"
                    id="channel-${channelId}-icon"
                    value="${channelConfig.icon || ''}"
                    placeholder="https://..."
                    onchange="updateChannelConfig('${channelId}', 'icon', this.value)"
                >
            </div>
            <div class="form-group">
                <label>Message Template Body</label>
                <textarea
                    id="channel-${channelId}-template-body"
                    placeholder="{{message}}"
                    onchange="updateChannelTemplate('${channelId}', 'body', this.value)"
                >${channelConfig.message_template?.body || ''}</textarea>
            </div>
            <div class="channel-actions">
                <button class="btn btn-test" onclick="testChannel('${channelId}', '${channelType}')">🧪 Test Channel</button>
            </div>
        `;
        container.appendChild(card);
    }
}

// Render routing rules
function renderRoutingRules() {
    const container = document.getElementById('routing-rules-container');
    container.innerHTML = '';

    if (config.routing_rules.length === 0) {
        container.innerHTML = '<p style="color: #6b7280; font-style: italic;">No routing rules configured. Add rules to control which channels receive notifications based on conditions.</p>';
        return;
    }

    config.routing_rules.forEach((rule, index) => {
        const ruleCard = document.createElement('div');
        ruleCard.className = 'routing-rule';
        ruleCard.innerHTML = `
            <div class="routing-rule-header">
                <div class="routing-rule-title">${rule.name}</div>
                <div>
                    <input
                        type="checkbox"
                        id="rule-${index}-enabled"
                        ${rule.enabled ? 'checked' : ''}
                        onchange="toggleRule(${index})"
                    >
                    <label for="rule-${index}-enabled">Enabled</label>
                    <button class="btn btn-danger" onclick="deleteRule(${index})" style="margin-left: 10px;">🗑️ Delete</button>
                </div>
            </div>
            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px;">
                <div>
                    <label>Channels</label>
                    <div style="font-size: 0.9rem;">${rule.channels.map(id => {
                        const channelConfig = config.channels[id];
                        return channelConfig ? getChannelDisplayName(id, channelConfig) + ` (${id})` : id;
                    }).join(', ')}</div>
                </div>
                <div>
                    <label>Hook Types</label>
                    <div style="font-size: 0.9rem;">${rule.match.hook_types.length > 0 ? rule.match.hook_types.join(', ') : 'All'}</div>
                </div>
                ${rule.match.message_pattern ? `
                    <div>
                        <label>Message Pattern</label>
                        <div style="font-size: 0.9rem; font-family: monospace;">${rule.match.message_pattern}</div>
                    </div>
                ` : ''}
                ${rule.match.tool_pattern ? `
                    <div>
                        <label>Tool Pattern</label>
                        <div style="font-size: 0.9rem; font-family: monospace;">${rule.match.tool_pattern}</div>
                    </div>
                ` : ''}
            </div>
        `;
        container.appendChild(ruleCard);
    });
}

// Render templates
function renderTemplates() {
    const container = document.getElementById('templates-container');
    container.innerHTML = '';

    for (const [name, template] of Object.entries(config.global_templates)) {
        const item = document.createElement('div');
        item.className = 'template-item';
        item.innerHTML = `
            <div class="template-name">${name}</div>
            <div style="font-size: 0.9rem; color: #4b5563;">
                <strong>Title:</strong> ${template.title || '(not set)'}<br>
                <strong>Body:</strong> ${template.body || '(not set)'}
            </div>
        `;
        container.appendChild(item);
    }
}

// Toggle channel enabled state
function toggleChannel(channelType) {
    const checkbox = document.getElementById(`channel-${channelType}-enabled`);
    const enabled = checkbox.checked;

    if (!config.channels[channelType]) {
        config.channels[channelType] = { enabled: false };
    }

    config.channels[channelType].enabled = enabled;

    // Update visual state
    const card = checkbox.closest('.channel-card');
    if (enabled) {
        card.classList.remove('disabled');
    } else {
        card.classList.add('disabled');
    }

    console.log(`Channel ${channelType} ${enabled ? 'enabled' : 'disabled'}`);
}

// Update channel configuration
function updateChannelConfig(channelType, field, value) {
    if (!config.channels[channelType]) {
        config.channels[channelType] = {};
    }

    config.channels[channelType][field] = value;
    console.log(`Updated ${channelType}.${field} = ${value}`);
}

// Update channel template
function updateChannelTemplate(channelType, field, value) {
    if (!config.channels[channelType]) {
        config.channels[channelType] = {};
    }

    if (!config.channels[channelType].message_template) {
        config.channels[channelType].message_template = {};
    }

    config.channels[channelType].message_template[field] = value;
    console.log(`Updated ${channelType}.message_template.${field} = ${value}`);
}

// Toggle routing rule enabled state
function toggleRule(index) {
    const checkbox = document.getElementById(`rule-${index}-enabled`);
    config.routing_rules[index].enabled = checkbox.checked;
    renderRoutingRules();
    console.log(`Rule ${index} ${checkbox.checked ? 'enabled' : 'disabled'}`);
}

// Delete routing rule
function deleteRule(index) {
    if (confirm('Are you sure you want to delete this rule?')) {
        config.routing_rules.splice(index, 1);
        renderRoutingRules();
        console.log(`Rule ${index} deleted`);
    }
}

// Test a channel
async function testChannel(channelId, channelType) {
    const info = channelInfo[channelType] || { icon: '🔔', name: channelType, description: '' };
    const channelConfig = config.channels[channelId];
    const displayName = channelConfig?.name || info.name;

    showStatus(`Testing ${displayName}...`, 'loading');

    try {
        const response = await fetch(`/api/test/${channelId}`, {
            method: 'POST'
        });

        const result = await response.json();
        console.log('Test result:', result);

        if (result.status === 'ok') {
            showStatus(`${displayName} test successful! ${result.message}`, 'success');
        } else {
            showStatus(`${displayName} test failed: ${result.message}`, 'error');
        }
    } catch (error) {
        console.error('Test error:', error);
        showStatus(`Test failed: ${error.message}`, 'error');
    }
}

// Save configuration
async function saveConfiguration() {
    showStatus('Saving configuration...', 'loading');

    try {
        const response = await fetch('/api/config', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(config)
        });

        const result = await response.json();
        console.log('Save result:', result);

        if (response.ok) {
            showStatus('Configuration saved successfully! ✓', 'success');
        } else {
            showStatus('Failed to save: ' + result.error, 'error');
        }
    } catch (error) {
        console.error('Save error:', error);
        showStatus('Failed to save: ' + error.message, 'error');
    }
}

// Show status message
function showStatus(message, type = 'info') {
    const indicator = document.getElementById('status-indicator');
    const text = document.getElementById('status-text');

    indicator.className = `status-indicator status-${type}`;
    text.textContent = message;

    // Auto-hide success messages after 5 seconds
    if (type === 'success') {
        setTimeout(() => {
            if (text.textContent === message) {
                indicator.className = 'status-indicator';
                text.textContent = 'Ready';
            }
        }, 5000);
    }
}

// Add new routing rule (placeholder for now)
function addRoutingRule() {
    alert('Adding routing rules through the UI is not yet implemented. Please edit the configuration file directly for now.');
    console.log('Add routing rule - not yet implemented');
}
