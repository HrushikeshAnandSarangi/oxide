# Oxide Control Panel

The web dashboard for [Oxide](../README.md) — create projects, trigger deploys, and watch status. Built with Next.js.

`/api/*` requests are rewritten to the Rust API at `http://127.0.0.1:3001` (see [next.config.ts](next.config.ts)), so the API must be running for this to do anything.

## Run

From the repo root, `npm run start` brings this up together with the rest of the stack (see the main [README](../README.md#run-locally)). To run just this piece:

```bash
npm run dev
```

Open [http://localhost:3000](http://localhost:3000).
