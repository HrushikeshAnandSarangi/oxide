#!/usr/bin/env node
// Regenerates the Criterion-derived tables in benchmarks.md from
// target/criterion/**/base/estimates.json (produced by `cargo bench`),
// replacing only the content between <!-- BENCH:<id>:START/END --> marker
// comments so the surrounding hand-written prose is untouched.
//
// Run after `cargo bench --workspace`:
//   node scripts/update_benchmarks.mjs

import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { join } from "node:path";

const ROOT = process.cwd();
const BENCH_MD = join(ROOT, "benchmarks.md");
const CRITERION_DIR = join(ROOT, "target", "criterion");

function pointEstimateNs(benchPath) {
  const file = join(CRITERION_DIR, benchPath, "base", "estimates.json");
  if (!existsSync(file)) {
    throw new Error(`Missing Criterion output: ${file} (did you run \`cargo bench --workspace\`?)`);
  }
  const estimates = JSON.parse(readFileSync(file, "utf8"));
  // Criterion's printed "time: [lower estimate upper]" uses the slope
  // estimate when available (linear regression over iteration counts);
  // falls back to mean for iter_custom-style measurements, none of which
  // this workspace uses today.
  const stat = estimates.slope ?? estimates.mean;
  return stat.point_estimate;
}

function formatTime(ns) {
  if (ns < 1000) return `${ns.toFixed(2)} ns`;
  return `${(ns / 1000).toFixed(3)} µs`;
}

function formatThroughput(ns) {
  const opsPerSec = 1e9 / ns;
  return `~${(opsPerSec / 1e6).toFixed(2)}M/s`;
}

function replaceBetweenMarkers(content, id, replacement) {
  const start = `<!-- BENCH:${id}:START -->`;
  const end = `<!-- BENCH:${id}:END -->`;
  const startIdx = content.indexOf(start);
  const endIdx = content.indexOf(end);
  if (startIdx === -1 || endIdx === -1) {
    throw new Error(`Markers for "${id}" not found in benchmarks.md`);
  }
  return (
    content.slice(0, startIdx + start.length) +
    "\n" +
    replacement +
    "\n" +
    content.slice(endIdx)
  );
}

let content = readFileSync(BENCH_MD, "utf8");

// --- Proxy routing (resolve_target) ---
const routeSizes = [
  ["1", "resolve_target/1"],
  ["8", "resolve_target/8"],
  ["64", "resolve_target/64"],
  ["1,000", "resolve_target/1000"],
  ["10,000", "resolve_target/10000"],
];
let routingRows = routeSizes.map(([label, path]) => {
  const ns = pointEstimateNs(path);
  return `| ${label} | ${formatTime(ns)} | ${formatThroughput(ns)} |`;
});
const missNs = pointEstimateNs("resolve_target_miss");
routingRows.push(`| miss (unregistered subdomain) | ${formatTime(missNs)} | ${formatThroughput(missNs)} |`);
content = replaceBetweenMarkers(
  content,
  "proxy_routing",
  ["| Registered routes | Time | Throughput |", "|---|---|---|", ...routingRows].join("\n"),
);

// --- Crypto ---
const cryptoRows = [
  ["Encrypt", 'short (`"production"`)', pointEstimateNs("encrypt_short_env_var")],
  ["Encrypt", "long (Postgres connection string, ~120 chars)", pointEstimateNs("encrypt_long_env_var")],
  ["Decrypt", "short", pointEstimateNs("decrypt_short_env_var")],
  ["Decrypt", "long", pointEstimateNs("decrypt_long_env_var")],
].map(([op, size, ns]) => `| ${op} | ${size} | ${formatTime(ns)} |`);
content = replaceBetweenMarkers(
  content,
  "crypto",
  ["| Operation | Value size | Time |", "|---|---|---|", ...cryptoRows].join("\n"),
);

// --- API metrics endpoint ---
const metricsRows = [
  ["`prometheus::gather()` + text-encode (4 registered metrics)", pointEstimateNs("metrics_gather_and_encode")],
  ["Increment a labeled counter (`oxide_deployments_total`)", pointEstimateNs("deployment_counter_increment")],
  ["Observe a histogram value (`oxide_build_duration_seconds`)", pointEstimateNs("build_duration_observe")],
].map(([op, ns]) => `| ${op} | ${formatTime(ns)} |`);
content = replaceBetweenMarkers(
  content,
  "api_metrics",
  ["| Operation | Time |", "|---|---|", ...metricsRows].join("\n"),
);

// --- Last-updated marker ---
const now = new Date().toISOString();
content = replaceBetweenMarkers(
  content,
  "updated",
  `_Last updated: ${now} (automated, via .github/workflows/benchmarks.yml)_`,
);

writeFileSync(BENCH_MD, content);
console.log(`Updated ${BENCH_MD} from Criterion output in ${CRITERION_DIR}`);
