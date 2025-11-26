# MCP Toolkit - Test Suite

## Monorepo Location

This package contains all end-to-end tests for MCP Toolkit, organized as part of the pnpm monorepo.

```
packages/
├── app/          # Frontend application
├── backend/      # Tauri Backend
├── mcp/          # MCP Library
└── tests/        # 👈 E2E Tests (this package)
```

## Installation

From the project root:

```bash
# Install all monorepo dependencies
pnpm install

# Install Playwright browsers
cd packages/tests
pnpm exec playwright install chromium
```

## Running Tests

### Option 1: From project root

```bash
# Run all tests
pnpm test -F @mcp-toolkit/tests

# Interactive UI mode
pnpm test:ui -F @mcp-toolkit/tests

# Headed mode (visible browser)
pnpm test:headed -F @mcp-toolkit/tests

# View report
pnpm test:report -F @mcp-toolkit/tests
```

### Option 2: From packages/tests

```bash
cd packages/tests

# Run all tests
pnpm test

# Interactive UI mode (recommended)
pnpm test:ui

# Headed mode (visible browser)
pnpm test:headed

# Debug mode
pnpm test:debug

# View HTML report
pnpm test:report
```

## Prerequisites

**Before running tests, the development server must be running:**

```bash
# From project root
pnpm dev
```

This will start the Tauri server at `http://localhost:1420`. Tests will connect automatically.

## Test Structure

```
packages/tests/
├── package.json              # Package dependencies
├── playwright.config.ts      # Playwright configuration
├── generate-report.ts        # Report generator
├── README.md                 # This documentation
├── e2e/                      # End-to-end tests
│   ├── 01-projects.spec.ts       # Project management (8 tests)
│   ├── 02-mcp-catalog.spec.ts    # MCP Catalog (12 tests) ⭐ KEY
│   ├── 03-bindings.spec.ts       # Project-MCP Bindings (8 tests)
│   ├── 04-secrets.spec.ts        # Secrets management (11 tests)
│   └── 05-integration.spec.ts    # Complete workflows (4 tests)
└── test-configs/             # Test configurations
    ├── standard-binary.json      # ✅ Should work
    ├── npx-based.json            # 🔍 Verify support
    ├── uv-python.json            # 🔍 Verify support
    ├── docker-based.json         # ⚠️ Partial support
    ├── http-based.json           # ⚠️ Partial support
    ├── complex-nested.json       # ❌ Likely not supported
    ├── alternative-fields.json   # ❌ Likely not supported
    ├── multiple-servers.json     # ✅ Should work
    ├── minimal-config.json       # ✅ Should work
    └── invalid-config.json       # ❌ Should fail
```

## Included Tests

### Total: 43 automated tests

| File | Tests | Description |
|---------|---------|-------------|
| `01-projects.spec.ts` | 8 | Create, list, delete projects |
| `02-mcp-catalog.spec.ts` | 12 | **Import MCP configurations** ⭐ |
| `03-bindings.spec.ts` | 8 | Activate MCPs, configure overrides |
| `04-secrets.spec.ts` | 11 | Create and manage secrets |
| `05-integration.spec.ts` | 4 | Complete workflows |

## Main Objective: Identify Unsupported Configurations

The **`02-mcp-catalog.spec.ts`** file is the most important for discovering which MCP configuration formats are supported.

### During execution, you will see in the console:

```
✅ Standard Binary MCP: SUPPORTED
❌ Alternative field names: NOT SUPPORTED
⚠️ Docker MCP: PARTIAL (parsing only)
🔍 NPX-based MCP: Testing...
```

### Expected Results:

**✅ Supported:**
- Standard binary configurations (`command` + `args` + `env`)
- Multiple server import
- Minimal configurations

**⚠️ Partial Support:**
- Docker MCPs (imported but execution might not be implemented)
- HTTP MCPs (imported but execution might not be implemented)

**❌ Likely Not Supported:**
- Alternative field names (`executable`, `arguments`, `environment`)
- Complex nested configurations (`transport`, `initializationOptions`)
- Non-standard formats

## Useful Commands

```bash
# Run specific test
pnpm exec playwright test e2e/02-mcp-catalog.spec.ts

# Run with name filter
pnpm exec playwright test --grep "import NPX"

# View last run report
pnpm test:report

# Generate support report
pnpm report
```

## Integration with Project Scripts

You can add these scripts to the root `package.json`:

```json
{
  "scripts": {
    "test": "pnpm -F @mcp-toolkit/tests test",
    "test:ui": "pnpm -F @mcp-toolkit/tests test:ui"
  }
}
```

Then run from root:

```bash
pnpm test
pnpm test:ui
```

## Test Results

After running tests, you will find:

- **HTML Report**: `packages/tests/test-results/html-report/`
- **Screenshots**: Screenshots of failures
- **Videos**: Recordings of failed tests
- **JSON**: `packages/tests/test-results/results.json`

## CI/CD Configuration

### GitHub Actions

```yaml
name: E2E Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: pnpm/action-setup@v2
        with:
          version: 8
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '20'
          cache: 'pnpm'
      
      - name: Install dependencies
        run: pnpm install
      
      - name: Install Playwright
        run: pnpm -F @mcp-toolkit/tests exec playwright install --with-deps
      
      - name: Run tests
        run: pnpm -F @mcp-toolkit/tests test
      
      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: test-results
          path: packages/tests/test-results/
```

## Troubleshooting

### Error: "Cannot connect to localhost:1420"

**Solution:** Ensure dev server is running:

```bash
pnpm dev
```

### Error: "Cannot find module '@playwright/test'"

**Solution:** Install dependencies:

```bash
pnpm install
```

### Tests fail randomly

**Solution:** 
1. There might be database state conflicts
2. Increase timeouts in `playwright.config.ts`
3. Run tests individually for debugging

### __dirname is not defined

This is normal in ES modules. Tests will work correctly when run with Playwright, which handles this automatically.

## Adding New Tests

1. Create file in `packages/tests/e2e/`
2. Follow existing file pattern
3. Use `test.describe()` and `test()`
4. Add test configurations in `test-configs/` if necessary

Example:

```typescript
import { test, expect } from '@playwright/test';

test.describe('New Functionality', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should do something', async ({ page }) => {
    // Your test here
  });
});
```

## Support

For issues with tests:

1. Check `test-results/html-report` for visual details
2. Examine console output for support messages
3. Run in `--debug` mode for step-by-step debugging
4. Check screenshots and videos of failures

## Contributing

When adding new tests:

1. Follow existing naming conventions
2. Add console logs for important discoveries
3. Document expected vs actual behavior
4. Update this documentation with new findings
