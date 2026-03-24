# Client — Angular Frontend

Parent instructions: [../CLAUDE.md](../CLAUDE.md)

## Agent Prompt

When working in this directory, follow the agent persona and conventions described in [prompts/agent.md](prompts/agent.md).

## Overview

Angular TypeScript SPA for the SC e-commerce platform. Communicates with backend microservices through the nginx API gateway.

## Status

**Planned** — not yet scaffolded. When starting development, initialize with Angular CLI:

```bash
ng new sc-client --routing --style=scss
```

## Architecture

```
client/
├── src/
│   ├── app/
│   │   ├── core/           # Singleton services, guards, interceptors
│   │   │   ├── auth/       # JWT interceptor, auth guard, auth service
│   │   │   ├── services/   # API services (product, user, order, chat)
│   │   │   └── models/     # TypeScript interfaces matching backend DTOs
│   │   ├── shared/         # Reusable components, pipes, directives
│   │   ├── features/       # Feature modules (lazy-loaded)
│   │   │   ├── catalog/    # Product listing, filters, search
│   │   │   ├── product/    # Product detail page
│   │   │   ├── chat/       # AI chat interface
│   │   │   ├── admin/      # Admin dashboard, product management, stats
│   │   │   ├── auth/       # Login, registration
│   │   │   └── profile/    # User profile, addresses, orders
│   │   └── app.component.ts
│   ├── assets/
│   ├── environments/
│   └── styles/
├── angular.json
├── package.json
└── tsconfig.json
```

## Development

```bash
npm install          # Install dependencies
ng serve             # Dev server (default: http://localhost:4200)
ng build             # Production build
ng test              # Unit tests (Karma)
ng e2e               # E2E tests
```

## Environment

API calls are proxied to backend services via nginx. In development, configure a proxy in `proxy.conf.json`:

```json
{
  "/api": {
    "target": "http://localhost:8080",
    "secure": false
  }
}
```

## Features

All v1 requirements, pages, API contracts, and resolved decisions are specified in [prompts/v1.md](prompts/v1.md).
