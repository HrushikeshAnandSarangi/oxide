export interface CreateProjectRequest {
  name: string;
  subdomain: string;
}

export interface CreateProjectResponse {
  message: string;
}

export interface DeploymentRequest {
  subdomain: string;
  repo_url: string;
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

  createProject: async (
    data: CreateProjectRequest
  ): Promise<CreateProjectResponse> => {
    const res = await fetch("/api/project", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(data),
    });

    if (!res.ok) {
      const errText = await res.text();
      throw new Error(errText || "Failed to create project");
    }

    return res.json();
  },

  deployProject: async (data: DeploymentRequest): Promise<string> => {
    const res = await fetch("/api/deploy", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(data),
    });

    if (!res.ok) {
      const errText = await res.text();
      throw new Error(errText || "Failed to deploy project");
    }

    const value = await res.json();
    return value as string;
  },
};
