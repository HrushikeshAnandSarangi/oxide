
# Oxide — Remaining Work (Detailed Implementation Roadmap)

This document describes **all remaining work required to make Oxide a fully operational and production-capable Platform-as-a-Service**.

The current system already includes the core architecture (builder, runtime, proxy, control plane, API, and database).  
However, several **critical platform capabilities, infrastructure guarantees, and operational features** still need to be implemented.

This document enumerates those tasks in full detail.

---

# 1. Dependency Wiring and Application Bootstrapping

Although all crates exist, the complete system initialization flow must still be finalized.

The application entrypoint must construct and wire all subsystems together.

Required initialization sequence:

```

Create PostgreSQL connection pool
↓
Initialize database repositories
↓
Create Builder service
↓
Create Runtime service
↓
Initialize Proxy state
↓
Construct Control Plane with dependencies
↓
Start Proxy server
↓
Start API server

```

Responsibilities include:

- Dependency injection across crates
- shared runtime state initialization
- system lifecycle management

The application must ensure that both the **API server** and **reverse proxy server** run concurrently within the same async runtime.

---

# 2. Deployment Queue System

Currently, deployments may run directly through API calls.  
A proper **asynchronous deployment queue** must be implemented.

## Objectives

- Prevent system overload during concurrent deployments
- Allow deployments to run independently from HTTP request lifecycle
- Support retries and failure recovery
- Enable future horizontal scaling

## Required Components

### Deployment Job Model

A job must include:

```

deployment_id
project_id
repo_url
requested_at
status

```

### Queue Implementation

An internal async queue should be implemented using:

```

tokio::mpsc

```

Queue responsibilities:

```

enqueue deployment jobs
dispatch jobs to worker tasks
manage job lifecycle

```

### Worker System

Workers must:

```

listen for deployment jobs
execute deployment pipeline
update deployment status
handle failures

```

Workers will execute the deployment pipeline independently of API requests.

---

# 3. Deployment Status Lifecycle Management

Deployment status transitions must be persisted and updated throughout the deployment process.

## Required Status States

```

Queued
Building
BuildFailed
ImageBuilding
ContainerStarting
Running
Crashed
Stopped
Failed

```

## Status Update Points

During deployment:

```

Queued
↓
Building
↓
ImageBuilding
↓
ContainerStarting
↓
Running

```

Failure states must update deployment records accordingly.

The database must be updated at each stage of the deployment lifecycle.

---

# 4. Deployment Environment Snapshot System

Each deployment must store a snapshot of environment variables used during runtime.

## Purpose

- Immutable deployment configuration
- reproducible deployments
- safe rollback capability
- debugging support

## Required Database Table

```

## deployment_env_vars

id
deployment_id
key
value_encrypted

```

## Snapshot Process

During deployment:

```

Fetch project environment variables
↓
Copy values into deployment_env_vars
↓
Associate snapshot with deployment

```

Runtime will read these values when launching containers.

---

# 5. Environment Variable Encryption

Environment variables must not be stored in plaintext.

## Encryption Requirements

Secrets must be stored using symmetric encryption.

Recommended approach:

```

AES-GCM encryption

```

Stored format:

```

encrypted_value
nonce
authentication_tag

```

## Decryption Flow

At runtime:

```

retrieve encrypted secret
↓
decrypt in control plane
↓
inject into container environment

```

Secrets must never be exposed through API responses or logs.

---

# 6. Runtime Environment Variable Injection

Containers must receive environment variables at runtime.

Runtime container creation must include:

```

env variables injected via Docker API

```

Example:

```

DATABASE_URL
API_KEY
JWT_SECRET

```

These must be retrieved from `deployment_env_vars` before container startup.

---

# 7. Docker Container Lifecycle Management

Container management must be extended beyond basic startup.

## Required Features

### Container Stop

Ability to stop containers when deployments are removed or replaced.

```

docker stop

```

### Container Removal

Cleanup unused containers.

```

docker rm

```

### Container Restart

Automatic restart of failed containers.

Runtime should monitor container health and restart when necessary.

---

# 8. Container Health Monitoring

The platform must detect unhealthy containers.

## Health Check Model

Applications should expose:

```

/health

```

endpoint.

Runtime health checks must periodically verify container availability.

Monitoring cycle:

```

check container health
↓
detect failure
↓
restart container

```

---

# 9. Docker Event Monitoring

The runtime must subscribe to Docker events to detect container state changes.

Events of interest include:

```

container die
container stop
container start

```

These events should update deployment status in the database.

Docker event monitoring enables:

```

automatic crash detection
automatic restarts
system observability

```

---

# 10. Reverse Proxy State Recovery

Currently, proxy routing state exists only in memory.

System restart would erase routing configuration.

## Required Startup Procedure

On system startup:

```

query running deployments
↓
load routing entries
↓
repopulate proxy routing table

```

Database query example:

```

SELECT subdomain, container_port
FROM deployments
WHERE status = 'Running'

```

This ensures routing survives system restarts.

---

# 11. Git Webhook Integration

A webhook endpoint must be implemented to automatically trigger deployments when repositories are updated.

## Endpoint

```

POST /webhook/github

```

## Webhook Flow

```

receive webhook
↓
verify signature
↓
extract repository info
↓
identify project
↓
enqueue deployment job

```

Webhook authentication should use GitHub signature verification.

---

# 12. Deployment Log Collection

Deployment logs must be captured and stored.

## Build Logs

Capture stdout and stderr from:

```

nix build

```

Logs must be streamed or buffered.

## Container Logs

Docker logs must be accessible for running containers.

Runtime must expose logs via:

```

docker logs

```

Logs should be associated with deployment records.

---

# 13. Build Timeout Enforcement

Build processes must have maximum execution time limits.

Example:

```

maximum build time: 10 minutes

```

If exceeded:

```

terminate build
mark deployment failed
cleanup resources

```

Timeout enforcement prevents system resource exhaustion.

---

# 14. Build Directory Management

Build directories must be managed to avoid disk exhaustion.

Current build structure:

```

/var/oxide/builds/{deployment_id}

```

Management tasks:

```

cleanup failed builds
remove stale directories
periodically purge unused builds

```

---

# 15. Artifact Retention Policy

Artifacts generated during builds must be managed to prevent disk growth.

Retention strategies may include:

```

keep last N deployments
delete old artifacts
remove unused Docker images

```

Artifact lifecycle management should be automated.

---

# 16. TLS Termination

The proxy currently supports only HTTP.

TLS must be added to enable secure connections.

## Requirements

- TLS certificate provisioning
- automatic certificate renewal
- secure HTTPS routing

Possible solutions include:

```

Pingora TLS integration
or
external TLS terminator

```

---

# 17. DNS Configuration Support

The platform requires wildcard DNS configuration.

Expected configuration:

```

*.oxide.domain → server IP

```

All project subdomains resolve to the Oxide proxy.

DNS configuration must be documented and integrated with routing.

---

# 18. Configuration System

System configuration should be externalized into configuration files.

Example:

```

config.toml

```

Configuration options may include:

```

server ports
build directories
database URL
docker network
proxy configuration

```

Configuration loading should support:

```

file-based config
environment variables

```

---

# 19. Security Hardening

Runtime isolation and resource control must be enforced.

## Required Controls

### Container Resource Limits

```

CPU quotas
memory limits

```

### Network Isolation

Containers must run in isolated networks.

### Privilege Restrictions

Containers must run without elevated privileges.

---

# 20. Observability Enhancements

The platform should expose runtime metrics.

Possible observability features include:

```

deployment metrics
container resource usage
request metrics
error tracking

```

Structured logging is already present but should be expanded with metrics.

---

# 21. Deployment History API

API endpoints should allow users to inspect deployment history.

Example endpoints:

```

GET /projects
GET /deployments
GET /deployments/{id}

```

Responses must include deployment status and metadata.

---

# 22. Rollback Capability

The platform should allow reverting to previous deployments.

Rollback flow:

```

select previous deployment
↓
restart associated container
↓
update proxy routing

```

Rollback relies on deployment snapshots and immutable artifacts.

---

# 23. Deployment Metadata Storage

Additional metadata should be stored for each deployment.

Recommended fields:

```

commit_sha
build_logs
runtime_logs
artifact_hash
deployment_duration

```

This improves debugging and traceability.

---

# 24. Platform CLI Tool

A command-line interface may be developed to interact with Oxide.

Possible commands:

```

oxide deploy
oxide logs
oxide status
oxide rollback

```

CLI tooling improves developer experience.

---

# 25. Operational Automation

Maintenance tasks should run automatically.

Examples include:

```

artifact cleanup
container pruning
log rotation
build directory cleanup

```

These tasks can run through background workers.

---

# Conclusion

The remaining work primarily focuses on:

- improving deployment orchestration
- strengthening runtime reliability
- securing secrets management
- enhancing observability
- ensuring operational stability

Completing these tasks will transform Oxide from a **functional platform prototype** into a **fully operational infrastructure platform capable of managing real application deployments**.
```
