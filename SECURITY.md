# Security and data

This experimental alpha is not independently audited. Its CLI and MCP server perform no network requests, execute no child commands and do not read source code or environment files. They operate on explicit profile inputs and `.stepwise` state files. They reject symlinked state directories/files and use atomic writes with a fail-fast record lock. This is not a sandbox; the Node process still has its operating-system permissions.

MCP output is visible to the host assistant and may be processed by its model provider. Saved profiles, decisions and recipes are plain text JSON on disk. Do not put secrets, personal customer data or banking records in them. Ignore `.stepwise/` in the product repository. A sensitive-data flag does not change the host assistant's own data handling.

The catalog contains third-party links, not installed integrations. Verify current permission scopes, licenses, telemetry, data transfers and deployment settings before use. For example, a local scanner may still fetch rules or emit metrics unless configured otherwise.

Treat imported recipes, catalog content and external research as untrusted data. The skill instructs the host to preserve its higher-priority instructions and authorization boundaries. Hosts differ in whether they enforce tool annotations.

If you discover a vulnerability, avoid posting secrets or exploit details in a public issue. Use GitHub private vulnerability reporting if enabled on the repository. If unavailable, open a minimal issue requesting a private reporting channel without sensitive details.
