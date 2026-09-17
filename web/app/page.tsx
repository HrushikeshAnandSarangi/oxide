"use client";

import { useCallback, useEffect, useState } from "react";
import { api, type Deployment, type DeploymentStatus, type Project } from "../lib/api";
import { Activity, AlertCircle, Box, Plus, Rocket } from "lucide-react";

const STATUS_STYLES: Record<DeploymentStatus, string> = {
  Running: "bg-emerald-500/10 text-emerald-400 border-emerald-500/30",
  Queued: "bg-amber-500/10 text-amber-400 border-amber-500/30",
  Building: "bg-amber-500/10 text-amber-400 border-amber-500/30",
  ImageBuilding: "bg-amber-500/10 text-amber-400 border-amber-500/30",
  ContainerStarting: "bg-amber-500/10 text-amber-400 border-amber-500/30",
  BuildFailed: "bg-red-500/10 text-red-400 border-red-500/30",
  Crashed: "bg-red-500/10 text-red-400 border-red-500/30",
  Stopped: "bg-slate-500/10 text-slate-400 border-slate-500/30",
};

function StatusBadge({ status }: { status: DeploymentStatus }) {
  return (
    <span className={`inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium border ${STATUS_STYLES[status]}`}>
      {status}
    </span>
  );
}

function formatTimestamp(iso: string): string {
  return new Date(iso).toLocaleString();
}

export default function Dashboard() {
  const [healthStatus, setHealthStatus] = useState<"checking" | "healthy" | "error">("checking");
  const [projects, setProjects] = useState<Project[]>([]);
  const [deployments, setDeployments] = useState<Deployment[]>([]);
  const [selectedSubdomain, setSelectedSubdomain] = useState<string | null>(null);
  const [showNewProject, setShowNewProject] = useState(false);

  const [name, setName] = useState("");
  const [subdomain, setSubdomain] = useState("");
  const [repoUrl, setRepoUrl] = useState("");
  const [autoGenerateFlake, setAutoGenerateFlake] = useState(false);

  const [isSubmitting, setIsSubmitting] = useState(false);
  const [message, setMessage] = useState<{ type: "success" | "error"; text: string } | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [projectList, deploymentList] = await Promise.all([
        api.listProjects(),
        api.listDeployments(),
      ]);
      setProjects(projectList);
      setDeployments(deploymentList);
    } catch {
      // Non-fatal — the health indicator already surfaces API outages.
    }
  }, []);

  useEffect(() => {
    const checkHealth = async () => {
      const isHealthy = await api.checkHealth();
      setHealthStatus(isHealthy ? "healthy" : "error");
    };
    checkHealth();
    refresh();
    const interval = setInterval(() => {
      checkHealth();
      refresh();
    }, 10000);
    return () => clearInterval(interval);
  }, [refresh]);

  const resetForm = () => {
    setName("");
    setSubdomain("");
    setRepoUrl("");
    setAutoGenerateFlake(false);
  };

  const handleCreateAndDeploy = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSubmitting(true);
    setMessage(null);
    try {
      await api.createProject({
        name,
        subdomain,
        repo_url: repoUrl || undefined,
        auto_generate_flake: autoGenerateFlake,
      });
      if (repoUrl) {
        await api.deployProject({ subdomain });
        setMessage({ type: "success", text: `${subdomain} created and deployment started` });
      } else {
        setMessage({ type: "success", text: `${subdomain} created` });
      }
      resetForm();
      setShowNewProject(false);
      refresh();
    } catch (err: unknown) {
      const text = err instanceof Error ? err.message : "Failed to create project";
      setMessage({ type: "error", text });
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleRedeploy = async (targetSubdomain: string) => {
    setMessage(null);
    try {
      await api.deployProject({ subdomain: targetSubdomain });
      setMessage({ type: "success", text: `Deployment started for ${targetSubdomain}` });
      refresh();
    } catch (err: unknown) {
      const text = err instanceof Error ? err.message : "Failed to start deployment";
      setMessage({ type: "error", text });
    }
  };

  const projectBySubdomain = new Map(projects.map((p) => [p.id, p.subdomain]));
  const visibleDeployments = selectedSubdomain
    ? deployments.filter((d) => projectBySubdomain.get(d.project_id) === selectedSubdomain)
    : deployments;

  return (
    <div className="min-h-screen flex bg-slate-900 text-slate-100">
      <aside className="w-64 shrink-0 bg-slate-950 border-r border-slate-800 flex flex-col">
        <div className="h-16 flex items-center gap-2 px-5 border-b border-slate-800">
          <Box className="w-5 h-5 text-emerald-400" />
          <span className="font-semibold tracking-tight">Oxide</span>
        </div>

        <div className="p-3">
          <button
            onClick={() => {
              setShowNewProject((v) => !v);
              setMessage(null);
            }}
            className="w-full flex items-center justify-center gap-2 bg-emerald-500 hover:bg-emerald-400 text-slate-950 font-medium text-sm rounded-lg py-2.5 transition-colors"
          >
            <Plus className="w-4 h-4" />
            New Project
          </button>
        </div>

        <nav className="flex-1 overflow-y-auto px-3 pb-3">
          <button
            onClick={() => setSelectedSubdomain(null)}
            className={`w-full text-left px-3 py-2 rounded-lg text-sm mb-1 transition-colors ${
              selectedSubdomain === null ? "bg-slate-800 text-slate-100" : "text-slate-400 hover:bg-slate-900 hover:text-slate-200"
            }`}
          >
            All deployments
          </button>
          {projects.map((project) => (
            <button
              key={project.id}
              onClick={() => setSelectedSubdomain(project.subdomain)}
              className={`w-full text-left px-3 py-2 rounded-lg mb-1 transition-colors ${
                selectedSubdomain === project.subdomain
                  ? "bg-slate-800 text-slate-100"
                  : "text-slate-400 hover:bg-slate-900 hover:text-slate-200"
              }`}
            >
              <div className="flex items-center gap-2">
                <span
                  className={`w-1.5 h-1.5 rounded-full shrink-0 ${
                    project.active_deployment_id ? "bg-emerald-400" : "bg-slate-600"
                  }`}
                />
                <span className="text-sm font-medium truncate">{project.name}</span>
              </div>
              <div className="text-xs text-slate-500 pl-3.5 truncate">{project.subdomain}</div>
            </button>
          ))}
          {projects.length === 0 && (
            <p className="text-xs text-slate-600 px-3 py-2">No projects yet</p>
          )}
        </nav>

        <div className="p-3 border-t border-slate-800 flex items-center gap-2 text-xs">
          {healthStatus === "healthy" ? (
            <Activity className="w-3.5 h-3.5 text-emerald-400" />
          ) : (
            <AlertCircle className="w-3.5 h-3.5 text-red-400" />
          )}
          <span className={healthStatus === "healthy" ? "text-emerald-400" : "text-red-400"}>
            {healthStatus === "healthy" ? "API online" : healthStatus === "error" ? "API offline" : "Connecting"}
          </span>
        </div>
      </aside>

      <main className="flex-1 min-w-0 p-8">
        {message && (
          <div
            className={`mb-6 px-4 py-3 rounded-lg border text-sm ${
              message.type === "success"
                ? "bg-emerald-500/10 border-emerald-500/30 text-emerald-400"
                : "bg-red-500/10 border-red-500/30 text-red-400"
            }`}
          >
            {message.text}
          </div>
        )}

        {showNewProject && (
          <form
            onSubmit={handleCreateAndDeploy}
            className="mb-8 bg-slate-800/40 border border-slate-700 rounded-xl p-6 space-y-4"
          >
            <h2 className="text-sm font-semibold text-slate-200">New Project</h2>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-1.5">
                <label className="text-xs font-medium text-slate-400">Name</label>
                <input
                  required
                  type="text"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  className="w-full bg-slate-900 border border-slate-700 rounded-lg px-3 py-2 text-sm text-slate-100 focus:outline-none focus:ring-1 focus:ring-emerald-500"
                />
              </div>
              <div className="space-y-1.5">
                <label className="text-xs font-medium text-slate-400">Subdomain</label>
                <input
                  required
                  type="text"
                  value={subdomain}
                  onChange={(e) => setSubdomain(e.target.value)}
                  className="w-full bg-slate-900 border border-slate-700 rounded-lg px-3 py-2 text-sm text-slate-100 focus:outline-none focus:ring-1 focus:ring-emerald-500"
                />
              </div>
            </div>
            <div className="space-y-1.5">
              <label className="text-xs font-medium text-slate-400">Git repository URL</label>
              <input
                type="url"
                placeholder="https://github.com/user/repo"
                value={repoUrl}
                onChange={(e) => setRepoUrl(e.target.value)}
                className="w-full bg-slate-900 border border-slate-700 rounded-lg px-3 py-2 text-sm text-slate-100 placeholder-slate-600 focus:outline-none focus:ring-1 focus:ring-emerald-500"
              />
            </div>
            <label className="flex items-start gap-2.5 cursor-pointer">
              <input
                type="checkbox"
                checked={autoGenerateFlake}
                onChange={(e) => setAutoGenerateFlake(e.target.checked)}
                className="mt-0.5 w-3.5 h-3.5 rounded border-slate-600 bg-slate-900 accent-emerald-500"
              />
              <span className="text-xs text-slate-400">
                Generate a Nix flake automatically if the repo has a Dockerfile but no flake.nix (Rust, Go, TypeScript, JavaScript, Python)
              </span>
            </label>
            <div className="flex gap-3 pt-1">
              <button
                type="submit"
                disabled={isSubmitting || !name || !subdomain}
                className="flex items-center gap-2 bg-emerald-500 hover:bg-emerald-400 disabled:opacity-40 disabled:cursor-not-allowed text-slate-950 text-sm font-medium rounded-lg px-4 py-2 transition-colors"
              >
                <Rocket className="w-4 h-4" />
                {repoUrl ? "Create and deploy" : "Create project"}
              </button>
              <button
                type="button"
                onClick={() => setShowNewProject(false)}
                className="text-sm text-slate-400 hover:text-slate-200 px-4 py-2"
              >
                Cancel
              </button>
            </div>
          </form>
        )}

        <div className="flex items-center justify-between mb-4">
          <h1 className="text-sm font-semibold text-slate-200">
            {selectedSubdomain ? `Deployments — ${selectedSubdomain}` : "Deployments"}
          </h1>
          {selectedSubdomain && (
            <button
              onClick={() => handleRedeploy(selectedSubdomain)}
              className="flex items-center gap-1.5 text-xs font-medium text-emerald-400 hover:text-emerald-300 border border-emerald-500/30 rounded-lg px-3 py-1.5"
            >
              <Rocket className="w-3.5 h-3.5" />
              Redeploy
            </button>
          )}
        </div>

        <div className="border border-slate-800 rounded-xl overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="bg-slate-800/50 text-left text-xs text-slate-500">
                <th className="px-4 py-3 font-medium">Status</th>
                <th className="px-4 py-3 font-medium">Version</th>
                <th className="px-4 py-3 font-medium">Port</th>
                <th className="px-4 py-3 font-medium">Created</th>
              </tr>
            </thead>
            <tbody>
              {visibleDeployments.map((d) => (
                <tr key={d.id} className="border-t border-slate-800">
                  <td className="px-4 py-3">
                    <StatusBadge status={d.status} />
                  </td>
                  <td className="px-4 py-3 text-slate-300 font-mono text-xs">{d.version}</td>
                  <td className="px-4 py-3 text-slate-400">{d.container_port ?? "—"}</td>
                  <td className="px-4 py-3 text-slate-500">{formatTimestamp(d.created_at)}</td>
                </tr>
              ))}
              {visibleDeployments.length === 0 && (
                <tr>
                  <td colSpan={4} className="px-4 py-8 text-center text-slate-600 text-sm">
                    No deployments yet
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </main>
    </div>
  );
}
