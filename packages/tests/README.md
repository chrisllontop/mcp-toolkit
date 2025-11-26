# MCP Toolkit - Test Suite

## Ubicación en Monorepo

Este paquete contiene todas las pruebas end-to-end para MCP Toolkit, organizado como parte del monorepo con pnpm.

```
packages/
├── app/          # Aplicación frontend
├── backend/      # Backend Tauri
├── mcp/          # Librería MCP
└── tests/        # 👈 Pruebas E2E (este paquete)
```

## Instalación

Desde la raíz del proyecto:

```bash
# Instalar todas las dependencias del monorepo
pnpm install

# Instalar navegadores de Playwright
cd packages/tests
pnpm exec playwright install chromium
```

## Ejecución de Pruebas

### Opción 1: Desde la raíz del proyecto

```bash
# Ejecutar todas las pruebas
pnpm test -F @mcp-toolkit/tests

# Modo UI interactivo
pnpm test:ui -F @mcp-toolkit/tests

# Con navegador visible
pnpm test:headed -F @mcp-toolkit/tests

# Ver reporte
pnpm test:report -F @mcp-toolkit/tests
```

### Opción 2: Desde packages/tests

```bash
cd packages/tests

# Ejecutar todas las pruebas
pnpm test

# Modo UI interactivo (recomendado)
pnpm test:ui

# Con navegador visible
pnpm test:headed

# Modo debug
pnpm test:debug

# Ver reporte HTML
pnpm test:report
```

## Prerequisitos

**Antes de ejecutar las pruebas, el servidor de desarrollo debe estar corriendo:**

```bash
# Desde la raíz del proyecto
pnpm dev
```

Esto iniciará el servidor Tauri en `http://localhost:1420`. Las pruebas se conectarán automáticamente.

## Estructura de Pruebas

```
packages/tests/
├── package.json              # Dependencias del paquete
├── playwright.config.ts      # Configuración de Playwright
├── generate-report.ts        # Generador de reportes
├── README.md                 # Esta documentación
├── e2e/                      # Pruebas end-to-end
│   ├── 01-projects.spec.ts       # Gestión de proyectos (8 tests)
│   ├── 02-mcp-catalog.spec.ts    # Catálogo MCP (12 tests) ⭐ CLAVE
│   ├── 03-bindings.spec.ts       # Bindings proyecto-MCP (8 tests)
│   ├── 04-secrets.spec.ts        # Gestión de secretos (11 tests)
│   └── 05-integration.spec.ts    # Workflows completos (4 tests)
└── test-configs/             # Configuraciones de prueba
    ├── standard-binary.json      # ✅ Debería funcionar
    ├── npx-based.json            # 🔍 Verificar soporte
    ├── uv-python.json            # 🔍 Verificar soporte
    ├── docker-based.json         # ⚠️ Soporte parcial
    ├── http-based.json           # ⚠️ Soporte parcial
    ├── complex-nested.json       # ❌ Probablemente no soportado
    ├── alternative-fields.json   # ❌ Probablemente no soportado
    ├── multiple-servers.json     # ✅ Debería funcionar
    ├── minimal-config.json       # ✅ Debería funcionar
    └── invalid-config.json       # ❌ Debe fallar
```

## Pruebas Incluidas

### Total: 43 pruebas automatizadas

| Archivo | Pruebas | Descripción |
|---------|---------|-------------|
| `01-projects.spec.ts` | 8 | Crear, listar, eliminar proyectos |
| `02-mcp-catalog.spec.ts` | 12 | **Importar configuraciones MCP** ⭐ |
| `03-bindings.spec.ts` | 8 | Activar MCPs, configurar overrides |
| `04-secrets.spec.ts` | 11 | Crear y gestionar secretos |
| `05-integration.spec.ts` | 4 | Flujos de trabajo completos |

## Objetivo Principal: Identificar Configuraciones No Soportadas

El archivo **`02-mcp-catalog.spec.ts`** es el más importante para descubrir qué formatos de configuración MCP están soportados.

### Durante la ejecución, verás en consola:

```
✅ Standard Binary MCP: SUPPORTED
❌ Alternative field names: NOT SUPPORTED
⚠️ Docker MCP: PARTIAL (solo parsing)
🔍 NPX-based MCP: Testing...
```

### Resultados Esperados:

**✅ Soportados:**
- Configuraciones binarias estándar (`command` + `args` + `env`)
- Importación múltiple de servidores
- Configuraciones mínimas

**⚠️ Soporte Parcial:**
- Docker MCPs (se importan pero la ejecución puede no estar implementada)
- HTTP MCPs (se importan pero la ejecución puede no estar implementada)

**❌ Probablemente No Soportados:**
- Nombres de campos alternativos (`executable`, `arguments`, `environment`)
- Configuraciones complejas anidadas (`transport`, `initializationOptions`)
- Formatos no estándar

## Comandos Útiles

```bash
# Ejecutar prueba específica
pnpm exec playwright test e2e/02-mcp-catalog.spec.ts

# Ejecutar con filtro de nombre
pnpm exec playwright test --grep "import NPX"

# Ver reporte de última ejecución
pnpm test:report

# Generar reporte de soporte
pnpm report
```

## Integración con Scripts del Proyecto

Puedes añadir estos scripts al `package.json` raíz:

```json
{
  "scripts": {
    "test": "pnpm -F @mcp-toolkit/tests test",
    "test:ui": "pnpm -F @mcp-toolkit/tests test:ui"
  }
}
```

Luego ejecutar desde la raíz:

```bash
pnpm test
pnpm test:ui
```

## Resultados de las Pruebas

Después de ejecutar las pruebas, encontrarás:

- **Reporte HTML**: `packages/tests/test-results/html-report/`
- **Screenshots**: Capturas de pantalla de fallos
- **Videos**: Grabaciones de pruebas fallidas
- **JSON**: `packages/tests/test-results/results.json`

## Configuración de CI/CD

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

**Solución:** Asegúrate de que el servidor dev esté corriendo:

```bash
pnpm dev
```

### Error: "Cannot find module '@playwright/test'"

**Solución:** Instala las dependencias:

```bash
pnpm install
```

### Los tests fallan aleatoriamente

**Solución:** 
1. Puede haber conflictos de estado de base de datos
2. Aumenta timeouts en `playwright.config.ts`
3. Ejecuta pruebas individualmente para debug

### __dirname is not defined

Esto es normal en módulos ES. Las pruebas funcionarán correctamente cuando se ejecuten con Playwright, que maneja esto automáticamente.

## Añadir Nuevas Pruebas

1. Crear archivo en `packages/tests/e2e/`
2. Seguir el patrón de los archivos existentes
3. Usar `test.describe()` y `test()`
4. Añadir configuraciones de prueba en `test-configs/` si es necesario

Ejemplo:

```typescript
import { test, expect } from '@playwright/test';

test.describe('Nueva Funcionalidad', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('debe hacer algo', async ({ page }) => {
    // Tu prueba aquí
  });
});
```

## Soporte

Para problemas con las pruebas:

1. Revisa `test-results/html-report` para detalles visuales
2. Examina la salida de consola para mensajes de soporte
3. Ejecuta en modo `--debug` para depuración paso a paso
4. Revisa screenshots y videos de fallos

## Contribuir

Al añadir nuevas pruebas:

1. Sigue las convenciones de nombres existentes
2. Añade logs de consola para descubrimientos importantes
3. Documenta comportamiento esperado vs actual
4. Actualiza esta documentación con nuevos hallazgos
