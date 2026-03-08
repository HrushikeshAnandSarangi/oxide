# Oxide — MVP Priority Roadmap (Internship Deployment)

This document contains **only what needs to be done before deploying to a VM and adding Oxide to a resume**.
Everything else is post-internship-application work.

---

## 🔴 PHASE 1 — Blocking (System Won't Run Without These)

These must be completed first. Nothing works until these are done.

---

### 1. Dependency Wiring & Application Bootstrapping

Wire all subsystems together in the main entrypoint.

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
Start Proxy server (concurrent)
↓
Start API server (concurrent)
```

- Dependency injection across all crates
- Both API server and proxy must run concurrently in the same Tokio runtime
- System lifecycle must start cleanly from a single binary

---

### 2. Deployment Status Lifecycle Persistence

Status transitions must be written to the database throughout the pipeline.

Required status states:

```
Queued → Building → ImageBuilding → ContainerStarting → Running
                 ↘ BuildFailed
                                                       ↘ Crashed / Failed
```

- Update deployment row in DB at each pipeline stage
- Failure states must be persisted, not silently dropped
- Control plane must call the deployment repository at every transition point

---

### 3. Proxy State Recovery on Restart

Currently routing state lives only in memory — a VM reboot wipes it.

On startup, the proxy must:

```
Query DB for all Running deployments
↓
Load subdomain → container_port mappings
↓
Repopulate DashMap routing table
```

SQL needed:
```sql
SELECT subdomain, container_port
FROM deployments
WHERE status = 'Running'
```

---

## 🟡 PHASE 2 — Demo Quality (1–2 Days, Makes It Presentable)

System runs after Phase 1. These make it stable and demoable.

---

### 4. Deployment Queue (Basic)

Replace direct API-triggered deployments with an async queue.

- Implement using `tokio::mpsc`
- API enqueues a job and returns immediately with `deployment_id`
- Background worker picks up job and runs the pipeline
- Prevents blocking HTTP requests and race conditions during live demos

Job model:
```
deployment_id
project_id
repo_url
requested_at
status
```

---

### 5. Container Stop & Remove

Prevent zombie containers accumulating across redeployments.

- `docker stop` on old container when a new deployment for the same project goes live
- `docker rm` after stopping
- Hook this into the deployment pipeline before starting a new container

---

### 6. Deployment History API Endpoints

Give interviewers something visible to inspect beyond raw deploys.

Endpoints to add:
```
GET /projects              → list all projects
GET /deployments           → list all deployments
GET /deployments/{id}      → single deployment with status + metadata
```

Responses must include: `id`, `status`, `version`, `created_at`, `project_id`

---

## 🟢 PHASE 3 — VM Deployment Checklist

Once Phase 1 and 2 are complete, deploy to a single VM.

- [ ] Provision a VM (any cloud — DigitalOcean, Hetzner, etc.)
- [ ] Install: Docker, PostgreSQL, Nix
- [ ] Run DB migrations via SQLx
- [ ] Configure wildcard DNS: `*.oxide.yourdomain.com → VM IP`
- [ ] Start Oxide binary (API on :3001, Proxy on :80)
- [ ] Test end-to-end: POST /deploy with a real repo URL
- [ ] Verify subdomain routes to running container
- [ ] Keep a demo repo ready (simple HTTP server with /health endpoint)

---

## ⚫ POST-APPLICATION (Do Later)

Do not touch these before submitting internship applications.

| Item | Todo # |
|---|---|
| Environment variable encryption | #5 |
| Env var injection into containers | #6 |
| Container health monitoring | #8 |
| Docker event monitoring | #9 |
| Git webhook integration | #11 |
| Deployment log collection | #12 |
| Build timeout enforcement | #13 |
| Build/artifact cleanup | #14, #15 |
| TLS termination | #16 |
| Rollback capability | #22 |
| Platform CLI | #24 |
| Observability/metrics | #20 |

---

## Summary

| Phase | Items | Est. Time |
|---|---|---|
| 🔴 Phase 1 — Blocking | 3 items | 2–4 days |
| 🟡 Phase 2 — Demo Quality | 3 items | 1–2 days |
| 🟢 Phase 3 — Deploy to VM | Checklist | 1 day |
| **Total** | | **~1 week** |

Once Phase 3 is live, Oxide is resume-ready and demoable.