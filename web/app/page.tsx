"use client";

import { useEffect, useState } from "react";
import { api } from "../lib/api";
import {
  Activity,
  Server,
  Box,
  Rocket,
  PlusCircle,
  AlertCircle,
  CheckCircle2,
} from "lucide-react";

export default function Dashboard() {
  const [healthStatus, setHealthStatus] = useState<"checking" | "healthy" | "error">("checking");
  const [activeTab, setActiveTab] = useState<"create" | "deploy">("create");

  // Form states
  const [name, setName] = useState("");
  const [subdomain, setSubdomain] = useState("");
  const [deploySubdomain, setDeploySubdomain] = useState("");
  const [repoUrl, setRepoUrl] = useState("");

  const [isLoading, setIsLoading] = useState(false);
  const [message, setMessage] = useState<{ type: "success" | "error"; text: string } | null>(null);

  useEffect(() => {
    const checkHealth = async () => {
      const isHealthy = await api.checkHealth();
      setHealthStatus(isHealthy ? "healthy" : "error");
    };

    checkHealth();
    const interval = setInterval(checkHealth, 30000);
    return () => clearInterval(interval);
  }, []);

  const handleCreateProject = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);
    setMessage(null);
    try {
      const res = await api.createProject({ name, subdomain });
      setMessage({ type: "success", text: res.message || "Project created successfully!" });
      setName("");
      setSubdomain("");
    } catch (err: any) {
      setMessage({ type: "error", text: err.message || "Failed to create project" });
    } finally {
      setIsLoading(false);
    }
  };

  const handleDeployProject = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);
    setMessage(null);
    try {
      const res = await api.deployProject({ subdomain: deploySubdomain, repo_url: repoUrl });
      setMessage({ type: "success", text: res || "Deployment started successfully!" });
      setDeploySubdomain("");
      setRepoUrl("");
    } catch (err: any) {
      setMessage({ type: "error", text: err.message || "Failed to start deployment" });
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-neutral-950 text-neutral-100 font-sans selection:bg-indigo-500/30">
      
      {/* Decorative background gradients */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-[-10%] left-[-10%] w-[40%] h-[40%] rounded-full bg-indigo-600/20 blur-[120px]" />
        <div className="absolute bottom-[-10%] right-[-10%] w-[40%] h-[40%] rounded-full bg-blue-600/20 blur-[120px]" />
      </div>

      <nav className="relative z-10 border-b border-white/10 bg-black/40 backdrop-blur-xl">
        <div className="max-w-6xl mx-auto px-6 h-16 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Box className="w-6 h-6 text-indigo-500" />
            <span className="font-semibold text-xl tracking-tight">Oxide Platform</span>
          </div>
          <div className="flex items-center gap-3">
            <div
              className={`flex items-center gap-2 px-3 py-1.5 rounded-full text-xs font-medium border ${
                healthStatus === "healthy"
                  ? "bg-emerald-500/10 text-emerald-400 border-emerald-500/20"
                  : healthStatus === "error"
                  ? "bg-red-500/10 text-red-400 border-red-500/20"
                  : "bg-neutral-500/10 text-neutral-400 border-neutral-500/20"
              }`}
            >
              {healthStatus === "healthy" ? (
                <Activity className="w-3.5 h-3.5" />
              ) : healthStatus === "error" ? (
                <AlertCircle className="w-3.5 h-3.5" />
              ) : (
                <Activity className="w-3.5 h-3.5 animate-pulse" />
              )}
              {healthStatus === "healthy" ? "API Online" : healthStatus === "error" ? "API Offline" : "Connecting..."}
            </div>
          </div>
        </div>
      </nav>

      <main className="relative z-10 max-w-4xl mx-auto px-6 py-12">
        <div className="mb-12 text-center sm:text-left">
          <h1 className="text-4xl sm:text-5xl font-bold tracking-tight mb-4">
            Deploy your code
            <br />
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-indigo-400 to-blue-400">
              in seconds.
            </span>
          </h1>
          <p className="text-neutral-400 text-lg max-w-2xl">
            Oxide makes it trivial to spin up environments, configure routes, and deploy applications without the headache of manual orchestration.
          </p>
        </div>

        {/* Tab Navigation */}
        <div className="flex bg-neutral-900/50 p-1 rounded-xl border border-white/10 w-full sm:w-fit mb-8 backdrop-blur-md">
          <button
            onClick={() => { setActiveTab("create"); setMessage(null); }}
            className={`flex items-center gap-2 px-6 py-2.5 rounded-lg text-sm font-medium transition-all ${
              activeTab === "create"
                ? "bg-indigo-500 text-white shadow-lg"
                : "text-neutral-400 hover:text-white hover:bg-white/5"
            }`}
          >
            <PlusCircle className="w-4 h-4" />
            Create Project
          </button>
          <button
            onClick={() => { setActiveTab("deploy"); setMessage(null); }}
            className={`flex items-center gap-2 px-6 py-2.5 rounded-lg text-sm font-medium transition-all ${
              activeTab === "deploy"
                ? "bg-blue-500 text-white shadow-lg"
                : "text-neutral-400 hover:text-white hover:bg-white/5"
            }`}
          >
            <Rocket className="w-4 h-4" />
            Start Deployment
          </button>
        </div>

        {/* Form Container */}
        <div className="bg-neutral-900/40 border border-white/10 rounded-2xl p-6 sm:p-8 backdrop-blur-xl relative overflow-hidden group">
          <div className="absolute inset-0 bg-gradient-to-br from-white/[0.02] to-transparent pointer-events-none" />
          
          {message && (
            <div
              className={`mb-6 p-4 rounded-xl border flex items-start gap-3 ${
                message.type === "success"
                  ? "bg-emerald-500/10 border-emerald-500/20 text-emerald-400"
                  : "bg-red-500/10 border-red-500/20 text-red-400"
              }`}
            >
              {message.type === "success" ? (
                <CheckCircle2 className="w-5 h-5 shrink-0 mt-0.5" />
              ) : (
                <AlertCircle className="w-5 h-5 shrink-0 mt-0.5" />
              )}
              <div className="text-sm font-medium leading-relaxed">{message.text}</div>
            </div>
          )}

          {activeTab === "create" ? (
            <form onSubmit={handleCreateProject} className="space-y-5 relative">
              <div className="space-y-1.5">
                <label className="text-sm font-medium text-neutral-300">Project Name</label>
                <input
                  required
                  type="text"
                  placeholder="e.g., My Awesome App"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  className="w-full bg-black/50 border border-white/10 rounded-xl px-4 py-3 text-white placeholder-neutral-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 transition-all font-medium"
                />
              </div>
              <div className="space-y-1.5">
                <label className="text-sm font-medium text-neutral-300">Subdomain</label>
                <div className="flex">
                  <input
                    required
                    type="text"
                    placeholder="e.g., awesome-app"
                    value={subdomain}
                    onChange={(e) => setSubdomain(e.target.value)}
                    className="w-full bg-black/50 border border-white/10 border-r-0 rounded-l-xl px-4 py-3 text-white placeholder-neutral-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 transition-all font-medium"
                  />
                  <div className="bg-white/5 border border-white/10 rounded-r-xl px-4 py-3 text-neutral-400 flex items-center text-sm font-medium">
                    .oxide.dev
                  </div>
                </div>
              </div>
              <div className="pt-2">
                <button
                  type="submit"
                  disabled={isLoading || !name || !subdomain}
                  className="bg-white text-black hover:bg-neutral-200 disabled:opacity-50 disabled:cursor-not-allowed w-full rounded-xl py-3 font-semibold transition-all shadow-[0_0_20px_rgba(255,255,255,0.1)] hover:shadow-[0_0_25px_rgba(255,255,255,0.2)] flex justify-center items-center gap-2"
                >
                  {isLoading ? (
                    <div className="w-5 h-5 border-2 border-black/30 border-t-black rounded-full animate-spin" />
                  ) : (
                    <>
                      <PlusCircle className="w-5 h-5" />
                      Create Project
                    </>
                  )}
                </button>
              </div>
            </form>
          ) : (
            <form onSubmit={handleDeployProject} className="space-y-5 relative">
              <div className="space-y-1.5">
                <label className="text-sm font-medium text-neutral-300">Subdomain to Deploy</label>
                <input
                  required
                  type="text"
                  placeholder="e.g., awesome-app"
                  value={deploySubdomain}
                  onChange={(e) => setDeploySubdomain(e.target.value)}
                  className="w-full bg-black/50 border border-white/10 rounded-xl px-4 py-3 text-white placeholder-neutral-500 focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all font-medium"
                />
              </div>
              <div className="space-y-1.5">
                <label className="text-sm font-medium text-neutral-300">Git Repository URL</label>
                <input
                  required
                  type="url"
                  placeholder="https://github.com/user/repo"
                  value={repoUrl}
                  onChange={(e) => setRepoUrl(e.target.value)}
                  className="w-full bg-black/50 border border-white/10 rounded-xl px-4 py-3 text-white placeholder-neutral-500 focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all font-medium"
                />
                <p className="text-xs text-neutral-500 mt-1 pl-1">Must be a public repository containing a Dockerfile.</p>
              </div>
              <div className="pt-2">
                <button
                  type="submit"
                  disabled={isLoading || !deploySubdomain || !repoUrl}
                  className="bg-blue-500 text-white hover:bg-blue-400 disabled:opacity-50 disabled:cursor-not-allowed w-full rounded-xl py-3 font-semibold transition-all shadow-[0_0_20px_rgba(59,130,246,0.3)] hover:shadow-[0_0_25px_rgba(59,130,246,0.4)] flex justify-center items-center gap-2"
                >
                  {isLoading ? (
                    <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                  ) : (
                    <>
                      <Rocket className="w-5 h-5" />
                      Start Deployment
                    </>
                  )}
                </button>
              </div>
            </form>
          )}
        </div>
        
        {/* Features/Stats Section at the bottom */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mt-16">
          <div className="p-6 rounded-2xl bg-white/[0.02] border border-white/5 flex flex-col gap-3">
            <div className="w-10 h-10 rounded-lg bg-indigo-500/10 flex items-center justify-center text-indigo-400 mb-2">
              <Server className="w-5 h-5" />
            </div>
            <h3 className="font-semibold text-lg">Instant Infrastructure</h3>
            <p className="text-neutral-400 text-sm leading-relaxed">Containers routed securely via Pingora, load balanced on the fly.</p>
          </div>
          <div className="p-6 rounded-2xl bg-white/[0.02] border border-white/5 flex flex-col gap-3">
            <div className="w-10 h-10 rounded-lg bg-emerald-500/10 flex items-center justify-center text-emerald-400 mb-2">
              <Activity className="w-5 h-5" />
            </div>
            <h3 className="font-semibold text-lg">Continuous Monitoring</h3>
            <p className="text-neutral-400 text-sm leading-relaxed">Deployment lifecycle mapped perfectly to persistent DB state.</p>
          </div>
          <div className="p-6 rounded-2xl bg-white/[0.02] border border-white/5 flex flex-col gap-3">
            <div className="w-10 h-10 rounded-lg bg-blue-500/10 flex items-center justify-center text-blue-400 mb-2">
              <Box className="w-5 h-5" />
            </div>
            <h3 className="font-semibold text-lg">Fully Automated</h3>
            <p className="text-neutral-400 text-sm leading-relaxed">Built from repository URL automatically via Docker build system.</p>
          </div>
        </div>

      </main>
    </div>
  );
}
