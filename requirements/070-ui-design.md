# Web UI Design - Phase 1

## Overview

Simple, functional web UI for managing simulator configuration. Served from control plane on **port 8081**.

**Design Philosophy**:

- **Simple**: Single-page app, minimal JavaScript
- **Functional**: Upload, validate, download, view endpoints
- **Clear**: Show validation errors prominently
- **Fast**: Lightweight, no heavy frameworks

**Technology**: Plain HTML + CSS + minimal JavaScript (no framework needed for Phase 1)

---

## Main Page Layout (Text Mockup)

```
┌────────────────────────────────────────────────────────────────────┐
│  WEB SIMULANT                                     [Status: Running] │
│                                                                      │
│  Engine: http://localhost:8080                                      │
│  Control Plane: http://localhost:8080                               │
├────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  CONFIGURATION                                                       │
│                                                                      │
│  Current: user-api-basic (5 endpoints)                              │
│  Loaded: 2026-02-11 10:15:23                                        │
│                                                                      │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐ │
│  │ [Upload Config]  │  │ [Download Config]│  │ [Validate Only]  │ │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘ │
│                                                                      │
├────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ACTIVE ENDPOINTS (5)                                [Refresh]      │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ ID: get-users                                                │  │
│  │ GET /api/users                                               │  │
│  │ Latency: normal (mean: 50ms, stddev: 10ms)                  │  │
│  │ Errors: 0.1% → [500]                                         │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ ID: create-user                                              │  │
│  │ POST /api/users                                              │  │
│  │ Latency: normal (mean: 150ms, stddev: 30ms)                 │  │
│  │ Errors: 2% → [400, 500]                                      │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ ID: update-user                                              │  │
│  │ PUT /api/users/me                                            │  │
│  │ Latency: normal (mean: 120ms, stddev: 25ms)                 │  │
│  │ Errors: 1% → [409, 500]                                      │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                      │
│  ... (2 more endpoints)                                              │
│                                                                      │
└────────────────────────────────────────────────────────────────────┘
```

---

## Upload Configuration Flow

### Step 1: Click "Upload Config"

File picker dialog opens:

- Accept: `.yaml`, `.yml`, `.json`
- Single file selection

### Step 2: Uploading State

```
┌────────────────────────────────────────────────────────────────────┐
│  UPLOADING CONFIGURATION...                                          │
│                                                                      │
│  Validating configuration file...                                   │
│  [████████████████████░░░░] 85%                                     │
└────────────────────────────────────────────────────────────────────┘
```

### Step 3a: Success

```
┌────────────────────────────────────────────────────────────────────┐
│  ✓ CONFIGURATION UPLOADED SUCCESSFULLY                               │
│                                                                      │
│  Configuration: ecommerce-mixed                                      │
│  Loaded 6 endpoints:                                                 │
│    • get-products (GET /api/products)                                │
│    • search-products (GET /api/search)                               │
│    • get-cart (GET /api/cart)                                        │
│    • add-to-cart (POST /api/cart/items)                              │
│    • checkout (POST /api/checkout)                                   │
│    • get-order (GET /api/orders/latest)                              │
│                                                                      │
│  Engine ready at http://localhost:8080                               │
│                                                                      │
│  [OK]                                                                │
└────────────────────────────────────────────────────────────────────┘
```

### Step 3b: Validation Errors

```
┌────────────────────────────────────────────────────────────────────┐
│  ✗ CONFIGURATION VALIDATION FAILED                                   │
│                                                                      │
│  Found 3 errors:                                                     │
│                                                                      │
│  1. endpoints[1].id                                                  │
│     Duplicate endpoint ID 'get-users'                                │
│     Location: line 15                                                │
│                                                                      │
│  2. endpoints[2].latency.params.mean_ms                              │
│     Value must be >= 0, got -10                                      │
│     Location: line 28                                                │
│                                                                      │
│  3. endpoints[4].error_profile.rate                                  │
│     Error rate must be between 0.0 and 1.0, got 1.5                  │
│     Location: line 52                                                │
│                                                                      │
│  Previous configuration retained.                                    │
│                                                                      │
│  [Close] [Download Errors as JSON]                                  │
└────────────────────────────────────────────────────────────────────┘
```

### Step 3c: Parse Error

```
┌────────────────────────────────────────────────────────────────────┐
│  ✗ FAILED TO PARSE CONFIGURATION                                     │
│                                                                      │
│  Invalid YAML syntax:                                                │
│  expected a mapping at line 5, column 3                              │
│                                                                      │
│  Please check your YAML/JSON syntax and try again.                   │
│                                                                      │
│  [Close]                                                             │
└────────────────────────────────────────────────────────────────────┘
```

---

## Download Configuration Flow

### Click "Download Config"

Browser automatically downloads:

- Filename: `simulation-config-2026-02-11-101523.yaml` (timestamp)
- Format: YAML (default) or JSON (if dropdown selected)

**Format selector** (optional enhancement):

```
Download Config ▼
  ├─ As YAML (.yaml)
  └─ As JSON (.json)
```

---

## Validate Only Flow

### Step 1: Click "Validate Only"

File picker dialog opens (same as upload)

### Step 2: Validation in progress

```
┌────────────────────────────────────────────────────────────────────┐
│  VALIDATING CONFIGURATION...                                         │
│                                                                      │
│  Checking syntax, structure, and constraints...                      │
└────────────────────────────────────────────────────────────────────┘
```

### Step 3a: Valid

```
┌────────────────────────────────────────────────────────────────────┐
│  ✓ CONFIGURATION IS VALID                                            │
│                                                                      │
│  Configuration: payment-api                                          │
│  Contains 3 endpoints                                                │
│                                                                      │
│  This configuration can be uploaded successfully.                    │
│                                                                      │
│  [Upload Now] [Close]                                                │
└────────────────────────────────────────────────────────────────────┘
```

### Step 3b: Invalid

Same error display as upload validation failure (Step 3b above)

---

## Endpoint List View (Detailed)

### Collapsed View (Default)

```
┌──────────────────────────────────────────────────────────────────┐
│ ID: get-users                                          [Expand ▼] │
│ GET /api/users                                                   │
│ Latency: normal (mean: 50ms, stddev: 10ms)                      │
│ Errors: 0.1% → [500]                                             │
└──────────────────────────────────────────────────────────────────┘
```

### Expanded View

```
┌──────────────────────────────────────────────────────────────────┐
│ ID: get-users                                        [Collapse ▲] │
│ GET /api/users                                                   │
│                                                                  │
│ LATENCY                                                          │
│   Distribution: normal                                           │
│   Mean: 50ms                                                     │
│   Std Dev: 10ms                                                  │
│                                                                  │
│ RESPONSE (Success)                                               │
│   Status: 200 OK                                                 │
│   Headers:                                                       │
│     • Content-Type: application/json                             │
│     • Cache-Control: max-age=60                                  │
│   Body:                                                          │
│     {                                                            │
│       "users": [                                                 │
│         {"id": "u1", "name": "Alice Anderson", ...}              │
│         ...                                                      │
│       ]                                                          │
│     }                                                            │
│                                                                  │
│ ERROR PROFILE                                                    │
│   Rate: 0.1% (1 in 1000 requests)                                │
│   Codes: 500                                                     │
│   Body: {"error": "Database connection timeout"}                 │
│                                                                  │
│ REQUEST MATCHING                                                 │
│   Body match: any (matches all requests)                         │
│                                                                  │
│ TEST THIS ENDPOINT                                               │
│   curl http://localhost:8080/api/users                           │
│   [Copy Command]                                                 │
└──────────────────────────────────────────────────────────────────┘
```

---

## No Configuration State

When no configuration is loaded:

```
┌────────────────────────────────────────────────────────────────────┐
│  WEB SIMULANT                                     [Status: Running] │
│                                                                      │
│  Engine: http://localhost:8080                                      │
│  Control Plane: http://localhost:8081                               │
├────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  NO CONFIGURATION LOADED                                             │
│                                                                      │
│  Upload a configuration to start simulating API endpoints.           │
│                                                                      │
│  ┌──────────────────┐  ┌──────────────────┐                        │
│  │ [Upload Config]  │  │ [Validate Only]  │                        │
│  └──────────────────┘  └──────────────────┘                        │
│                                                                      │
│  EXAMPLE CONFIGURATIONS                                              │
│                                                                      │
│  • 01-simple-health-check.yaml - Single endpoint                    │
│  • 02-user-api-basic.yaml - User management (5 endpoints)           │
│  • 03-ecommerce-mixed.yaml - E-commerce API (6 endpoints)           │
│  • 04-unreliable-external.yaml - High errors for resilience         │
│                                                                      │
│  [Browse Examples Folder]                                            │
│                                                                      │
└────────────────────────────────────────────────────────────────────┘
```

---

## Status Indicator

Top right corner shows system status:

### All Running

```
[Status: Running ●]
```

### Error State (if applicable)

```
[Status: Error ●]
```

### Loading

```
[Status: Loading... ●]
```

---

## Color Scheme (Simple)

**Background**: Light gray (#f5f5f5)  
**Panels**: White (#ffffff)  
**Borders**: Light gray (#ddd)  
**Text**: Dark gray (#333)  
**Accent**: Blue (#4a90e2)  
**Success**: Green (#5cb85c)  
**Error**: Red (#d9534f)  
**Warning**: Orange (#f0ad4e)

---

## Responsive Layout (Phase 2)

Phase 1: Desktop only (min-width: 1024px)  
Phase 2: Add mobile/tablet support

---

## Key Interactions

### Upload Button

- Opens file picker
- Accepts `.yaml`, `.yml`, `.json`
- Shows progress during upload/validation
- Displays success or error modal
- Auto-refreshes endpoint list on success

### Download Button

- Triggers immediate download
- Filename includes timestamp
- Format: YAML (default)

### Validate Button

- Same file picker as upload
- Does not apply configuration
- Shows validation results
- Option to upload if valid

### Endpoint Cards

- Clickable to expand/collapse
- Show key info when collapsed
- Show full details when expanded
- Each card has "Copy curl command" button

### Refresh Button

- Manually refresh endpoint list
- Shows loading indicator
- Updates endpoint count

---

## Error Display Patterns

### Inline Error (in form)

```
┌──────────────────────────────────────┐
│ [Choose File]                        │
│ ✗ File must be .yaml, .yml, or .json│
└──────────────────────────────────────┘
```

### Modal Error (validation)

Overlay modal with:

- Error icon (✗)
- Error title
- Error list with locations
- Action buttons

### Toast Notification (minor errors)

```
┌─────────────────────────────────────┐
│ ⚠ Configuration saved to disk failed│
│   (retrying in background)          │
└─────────────────────────────────────┘
```

---

## HTML Structure (Simplified)

```html
<!DOCTYPE html>
<html>
  <head>
    <title>Web Simulant</title>
    <style>
      /* Embedded CSS */
    </style>
  </head>
  <body>
    <header>
      <h1>Web Simulant</h1>
      <div id="status">Status: Running</div>
    </header>

    <main>
      <section id="config-panel">
        <h2>Configuration</h2>
        <div id="current-config">
          <!-- Current config info -->
        </div>
        <div id="actions">
          <button id="upload-btn">Upload Config</button>
          <button id="download-btn">Download Config</button>
          <button id="validate-btn">Validate Only</button>
        </div>
      </section>

      <section id="endpoints-panel">
        <h2>Active Endpoints (<span id="endpoint-count">0</span>)</h2>
        <button id="refresh-btn">Refresh</button>
        <div id="endpoint-list">
          <!-- Endpoint cards inserted here -->
        </div>
      </section>
    </main>

    <!-- Modal for errors/success -->
    <div id="modal" class="hidden">
      <div class="modal-content">
        <!-- Dynamic content -->
      </div>
    </div>

    <script>
      // Minimal JavaScript for interactions
      // - File upload with fetch API
      // - Download trigger
      // - Endpoint list refresh
      // - Modal show/hide
      // - Expand/collapse cards
    </script>
  </body>
</html>
```

---

## JavaScript Functionality (Minimal)

### File Upload

```javascript
async function uploadConfig(file) {
  const formData = new FormData();
  formData.append("config", file);

  const response = await fetch("/api/config/import", {
    method: "POST",
    body: formData,
  });

  const result = await response.json();

  if (response.ok) {
    showSuccess(result);
    refreshEndpoints();
  } else {
    showErrors(result.errors);
  }
}
```

### Download Config

```javascript
function downloadConfig() {
  window.location.href = "/api/config/export?format=yaml";
}
```

### Refresh Endpoints

```javascript
async function refreshEndpoints() {
  const response = await fetch("/api/endpoints");
  const data = await response.json();

  renderEndpoints(data.endpoints);
  updateCount(data.endpoints_count);
}
```

### Show Modal

```javascript
function showModal(type, title, content) {
  const modal = document.getElementById("modal");
  modal.classList.remove("hidden");
  // ... populate modal content
}
```

---

## Performance Targets

- **Page load**: <200ms
- **Upload/validate**: <500ms (small configs), <2s (large configs)
- **Endpoint list refresh**: <100ms
- **Download trigger**: Immediate

---

## Accessibility Considerations

- Semantic HTML (header, main, section)
- ARIA labels on buttons
- Keyboard navigation support (Tab, Enter, Esc)
- Screen reader friendly error messages
- Focus management (modal open/close)
- Color contrast ratios meet WCAG AA

---

## Phase 2 Enhancements

- **Live updates**: WebSocket for real-time endpoint status
- **Metrics dashboard**: Request counts, latency histograms
- **Endpoint editor**: Edit individual endpoints without full config
- **Configuration history**: View and rollback to previous configs
- **Dark mode**: Toggle between light/dark themes
- **Mobile responsive**: Support tablets and phones
- **Search/filter**: Search endpoints by path, method, or ID
- **Export selected**: Download subset of endpoints

---

## Testing Plan

### Manual Testing

1. Upload valid config → verify success
2. Upload invalid config → verify errors displayed
3. Upload unparseable file → verify parse error
4. Download config → verify file downloaded
5. Validate only → verify no application
6. Refresh endpoints → verify list updates
7. Expand/collapse endpoint → verify details shown/hidden
8. Copy curl command → verify clipboard

### Browser Testing

- Chrome (latest)
- Firefox (latest)
- Safari (latest) - Phase 2
- Edge (latest) - Phase 2

### Integration Testing (with API)

1. Upload config via UI → verify engine endpoints active
2. Download config → re-upload → verify identical
3. Multiple uploads → verify previous config replaced
4. Upload error → verify previous config retained

---

## Implementation Notes

### File Size

- Target: <50KB for HTML + CSS + JS combined
- No external dependencies
- No build step required

### Code Organization

```
static/
  ├── index.html    # Main UI page (embedded CSS/JS)
  └── favicon.ico   # Optional favicon
```

Optional split (if file grows):

```
static/
  ├── index.html
  ├── style.css
  ├── app.js
  └── favicon.ico
```

### State Management

- No complex state management needed
- Fetch fresh data from API on refresh
- Modal state managed with CSS classes
- No client-side routing

### Build Process

Phase 1: No build process, plain files  
Phase 2: Consider bundler if adding dependencies

---

## Success Criteria

UI is successful if user can:

1. ✓ Upload configuration and see it applied
2. ✓ See clear validation errors if config is invalid
3. ✓ View all active endpoints with key details
4. ✓ Download current configuration
5. ✓ Validate without applying
6. ✓ Copy curl commands to test endpoints
7. ✓ Complete all tasks without reading documentation
