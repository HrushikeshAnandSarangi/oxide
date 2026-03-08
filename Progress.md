
# Oxide — Project Snapshot (Current State)

**Oxide** is a lightweight Rust-based Platform-as-a-Service designed to deploy and manage applications with deterministic builds and containerized runtime isolation. The system is architected as a modular Rust workspace composed of multiple crates, each responsible for a distinct layer of the platform.

This document captures the **current achieved architecture, implemented subsystems, and the interaction between components**.

---

# 1. Core Objective

Oxide provides a minimal but production-style infrastructure platform capable of:

- Deploying applications from Git repositories
- Building artifacts using deterministic Nix builds
- Running applications inside Docker containers
- Routing external traffic to deployed applications
- Managing deployments via a Rust control plane
- Exposing an HTTP API for project and deployment management
- Maintaining state via PostgreSQL

All components are designed to run on a **single host machine**, enabling a compact yet realistic platform architecture.

---

# 2. Workspace Architecture

The Oxide system is organized as a **Cargo workspace** with multiple crates representing infrastructure layers.

```

oxide/
├── Cargo.toml
├── common
├── domain
├── db
├── builder
├── runtime
├── proxy
├── controller
├── api
└── migrations

````

Each crate represents a specific layer in the platform stack.

---

# 3. Shared Foundation Layer

## 3.1 `common` Crate

This crate provides shared utilities used across the platform.

### Features

- Global logging setup using structured tracing
- Centralized error definitions
- Configuration loading utilities
- Shared `Result` type alias
- Error propagation patterns

### Logging

Logging is configured using the Rust tracing ecosystem.

Features:

- JSON structured logs
- Environment-based log filtering
- Async-friendly logging
- Compatible with observability pipelines

Example initialization:

```rust
tracing_subscriber::fmt()
    .with_env_filter(filter)
    .json()
    .init();
````

---

# 4. Domain Layer

## 4.1 `domain` Crate

The domain crate contains **pure business entities and state definitions** independent of infrastructure.

No external infrastructure dependencies exist here.

### Core Entities

#### Project

Represents an application hosted on Oxide.

Fields:

* `id`
* `name`
* `subdomain`
* `active_deployment_id`
* `created_at`

#### Deployment

Represents a deployed version of a project.

Fields:

* `id`
* `project_id`
* `version`
* `artifact_path`
* `docker_image`
* `container_id`
* `status`
* `created_at`

### Deployment Status Lifecycle

The deployment state machine is defined using an enum:

```
Building
BuildFailed
ImageBuilding
ContainerStarting
Running
Crashed
Stopped
```

This lifecycle models the entire deployment pipeline.

---

# 5. Database Layer

## 5.1 `db` Crate

The database crate manages persistence and database interaction.

### Technology

* PostgreSQL
* SQLx
* Async connection pooling

### Responsibilities

* Managing database connections
* Query execution
* Mapping database rows to domain models
* Repository pattern implementation

### Repositories

#### Project Repository

Handles project operations:

* Create project
* Fetch by subdomain
* Project persistence

#### Deployment Repository

Handles deployment operations:

* Create deployment records
* Store deployment metadata
* Track deployment lifecycle

### Database Schema

#### Projects Table

```
projects
---------
id
name
subdomain
active_deployment_id
created_at
```

#### Deployments Table

```
deployments
-----------
id
project_id
version
artifact_path
docker_image
container_id
status
created_at
```

Database migrations are managed using **SQLx migration tooling**.

---

# 6. Build System

## 6.1 `builder` Crate

The builder crate is responsible for converting Git repositories into runnable artifacts using deterministic builds.

### Responsibilities

* Clone Git repositories
* Execute Nix builds
* Produce deployable artifacts
* Manage build directories
* Return artifact paths to the control plane

### Build Flow

```
Git Repository
      ↓
Clone to Build Directory
      ↓
Nix Build Execution
      ↓
Artifact Produced
      ↓
Artifact Path Returned
```

### Build Directory Structure

```
/var/oxide/builds/{deployment_id}/
```

Each deployment receives an isolated build directory.

### Artifact Resolution

Nix builds produce a `result` symlink pointing to the Nix store.

The builder resolves this path to determine the final artifact location.

---

# 7. Runtime System

## 7.1 `runtime` Crate

The runtime crate manages container lifecycle using Docker.

### Technology

* Docker Engine
* Bollard (Rust Docker SDK)

### Responsibilities

* Build container images from artifacts
* Create containers
* Apply resource limits
* Start containers
* Inspect container networking
* Return runtime metadata

### Container Resource Constraints

Containers are created with limits:

```
Memory: 256MB
CPU: 0.5 core
```

### Image Build Process

```
Artifact Directory
      ↓
Docker Build Context
      ↓
Image Creation
```

Images are tagged using deployment identifiers.

### Container Startup

Containers expose port:

```
3000
```

Docker dynamically assigns host ports.

Runtime inspects the container to determine the mapped port.

### Returned Runtime Metadata

```
container_id
host_port
```

---

# 8. Reverse Proxy System

## 8.1 `proxy` Crate

The proxy crate implements a programmable reverse proxy.

### Technology

Pingora — a high-performance Rust reverse proxy framework originally developed by Cloudflare.

### Responsibilities

* Accept incoming HTTP traffic
* Extract subdomain from host header
* Resolve container target
* Forward traffic to the correct container
* Maintain routing state

### Routing Model

```
{subdomain}.oxide.domain
```

Example:

```
portfolio.oxide.dev
```

Routes to:

```
127.0.0.1:{container_port}
```

### Routing Table

Routing state is stored in-memory using a concurrent map:

```
DashMap<String, u16>
```

Where:

```
subdomain → container_port
```

### Proxy Request Flow

```
Incoming HTTP Request
      ↓
Extract Host Header
      ↓
Determine Subdomain
      ↓
Lookup Container Port
      ↓
Forward Request
```

The proxy dynamically resolves upstream targets.

---

# 9. Controller

## 9.1 `controller` Crate

The control plane orchestrates the entire deployment pipeline.

### Responsibilities

* Manage deployments
* Coordinate build and runtime layers
* Update routing state
* Persist deployment metadata
* Execute deployment pipeline

### Core Dependencies

The controller integrates:

```
builder
runtime
proxy
database repositories
domain entities
```

### Deployment Service

The deployment service executes the deployment pipeline.

### Deployment Pipeline

```
Deployment Request
        ↓
Create Deployment ID
        ↓
Builder clones repository
        ↓
Nix build executed
        ↓
Artifact produced
        ↓
Runtime builds container image
        ↓
Container started
        ↓
Proxy routing updated
```

The controller acts as the orchestration layer for all infrastructure components.

---

# 10. API Layer

## 10.1 `api` Crate

The API crate exposes HTTP endpoints used to interact with the Oxide platform.

### Technology

Axum — asynchronous Rust web framework.

### Responsibilities

* Accept HTTP requests
* Validate payloads
* Call control plane services
* Return JSON responses

### API Server

The server runs on:

```
0.0.0.0:3001
```

### Shared Application State

API handlers share application state containing:

```
ControlPlane instance
```

---

# 11. API Endpoints

### Health Check

```
GET /health
```

Returns service status.

---

### Project Creation

```
POST /projects
```

Registers a new project.

Payload example:

```
{
  "name": "portfolio",
  "subdomain": "portfolio"
}
```

---

### Deployment Trigger

```
POST /deploy
```

Triggers a deployment pipeline.

Payload example:

```
{
  "subdomain": "portfolio",
  "repo_url": "https://github.com/user/repo"
}
```

---

# 12. End-to-End Deployment Flow

The complete deployment lifecycle currently operates as follows.

```
Client Request
      ↓
API Layer
      ↓
Control Plane
      ↓
Builder (Git + Nix)
      ↓
Artifact Produced
      ↓
Runtime (Docker)
      ↓
Container Started
      ↓
Proxy Route Updated
      ↓
Application Live
```

---

# 13. Deployment Routing Model

Example deployment:

```
Project: portfolio
Container Port: 32768
```

Proxy routing:

```
portfolio.oxide.dev
        ↓
127.0.0.1:32768
```

Traffic is dynamically forwarded through the Pingora proxy.

---

# 14. Logging and Observability

Oxide uses structured logging through the Rust tracing ecosystem.

Features:

* JSON logs
* async-safe logging
* request tracing support
* environment-filtered log levels

Logs are emitted throughout:

* builder
* runtime
* control_plane
* API handlers
* proxy routing

---

# 15. Deployment Isolation

Each deployment is isolated through multiple layers:

### Build Isolation

```
/var/oxide/builds/{deployment_id}
```

### Artifact Isolation

Artifacts generated through Nix are immutable.

### Container Isolation

Docker containers isolate runtime environments.

### Routing Isolation

Each project receives a unique subdomain mapping.

---

# 16. System Stack

The Oxide stack currently consists of the following technologies.

| Layer             | Technology          |
| ----------------- | ------------------- |
| API               | Axum                |
| Control Plane     | Rust async services |
| Proxy             | Pingora             |
| Container Runtime | Docker              |
| Build System      | Nix                 |
| Database          | PostgreSQL          |
| Async Runtime     | Tokio               |
| Logging           | Tracing             |

---

# 17. Platform Execution Model

All services run on a single host machine.

System services include:

```
Docker daemon
PostgreSQL
Oxide API
Oxide Proxy
```

The Rust application orchestrates the deployment lifecycle internally.

---

# 18. Architectural Characteristics

Oxide demonstrates several infrastructure design principles:

### Deterministic Builds

Applications are built using Nix for reproducibility.

### Containerized Runtime

All applications run inside Docker containers.

### Programmable Proxy

Routing is implemented using a Rust-native reverse proxy.

### Modular Architecture

Each infrastructure concern is isolated into independent crates.

### Async Control Plane

Deployment orchestration uses asynchronous Rust services.

---

# 19. Platform Outcome

The Oxide system currently implements a complete minimal PaaS architecture capable of:

* accepting deployment requests
* building applications from Git repositories
* producing deterministic artifacts
* running applications in containers
* dynamically routing traffic to deployed applications
* exposing an API for platform interaction
* maintaining deployment state through a database

The architecture follows modern infrastructure design patterns and demonstrates a modular Rust-based platform control plane.


