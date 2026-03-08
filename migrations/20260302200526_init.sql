CREATE TABLE projects (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    subdomain TEXT UNIQUE NOT NULL,
    active_deployment_id UUID,
    created_at TIMESTAMP NOT NULL
);

CREATE TABLE deployments (
    id UUID PRIMARY KEY,
    project_id UUID REFERENCES projects(id),
    version TEXT NOT NULL,
    artifact_path TEXT,
    docker_image TEXT,
    container_id TEXT,
    status TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL
);
