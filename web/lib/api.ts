export interface CreateProjectRequest {
  name: string;
  subdomain: string;
  repo_url?: string;
  auto_generate_flake?: boolean;
}

export interface CreateProjectResponse {
  message: string;
  project_id: string;
}

export interface DeploymentRequest {
  subdomain: string;
}

export interface DeploymentResponse {
  message: string;
  deployment_id: string;
}

export interface Project {
  id: string;
  name: string;
  subdomain: string;
  repo_url: string | null;
  install_command: string | null;
  build_command: string | null;
  run_command: string | null;
  root_directory: string | null;
  auto_generate_flake: boolean;
  active_deployment_id: string | null;
  created_at: string;
}

export type DeploymentStatus =
  | "Queued"
  | "Building"
  | "BuildFailed"
  | "ImageBuilding"
  | "ContainerStarting"
  | "Running"
  | "Crashed"
  | "Stopped";

export interface Deployment {
  id: string;
  project_id: string;
  version: string;
  artifact_path: string | null;
  docker_image: string | null;
  container_id: string | null;
  container_port: number | null;
  status: DeploymentStatus;
  created_at: string;
}

export interface DeploymentLogs {
  status: DeploymentStatus;
  build_log: string | null;
  container_log: string | null;
}

async function unwrap<T>(res: Response, fallbackError: string): Promise<T> {
  if (!res.ok) {
    const errText = await res.text();
    throw new Error(errText || fallbackError);
  }
  return res.json();
}

export const api = {
  checkHealth: async (): Promise<boolean> => {
    try {
      const res = await fetch("/api/health");
      return res.ok;
    } catch {
      return false;
    }
  },

  createProject: async (data: CreateProjectRequest): Promise<CreateProjectResponse> => {
    const res = await fetch("/api/project", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(data),
    });
    return unwrap(res, "Failed to create project");
  },

  deployProject: async (data: DeploymentRequest): Promise<DeploymentResponse> => {
    const res = await fetch("/api/deploy", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(data),
    });
    return unwrap(res, "Failed to start deployment");
  },

  listProjects: async (): Promise<Project[]> => {
    const res = await fetch("/api/projects");
    return unwrap(res, "Failed to load projects");
  },

  listDeployments: async (): Promise<Deployment[]> => {
    const res = await fetch("/api/deployments");
    return unwrap(res, "Failed to load deployments");
  },

  deleteDeployment: async (id: string): Promise<void> => {
    const res = await fetch(`/api/deployments/${id}`, { method: "DELETE" });
    if (!res.ok) {
      const errText = await res.text();
      throw new Error(errText || "Failed to delete deployment");
    }
  },

  getDeploymentLogs: async (id: string): Promise<DeploymentLogs> => {
    const res = await fetch(`/api/deployments/${id}/logs`);
    return unwrap(res, "Failed to load logs");
  },

  rollback: async (data: DeploymentRequest): Promise<DeploymentResponse> => {
    const res = await fetch("/api/rollback", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(data),
    });
    return unwrap(res, "Failed to start rollback");
  },
};
