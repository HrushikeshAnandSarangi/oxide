import type { NextConfig } from "next";
import path from "node:path";

const nextConfig: NextConfig = {
  // Pin the workspace root explicitly — without this, Turbopack's
  // auto-detection walks up past the repo root (to the root package.json
  // added for `npm run start` orchestration) and warns about ambiguous
  // lockfiles.
  turbopack: {
    root: path.join(__dirname),
  },
  async rewrites() {
    return [
      {
        source: '/api/:path*',
        destination: 'http://127.0.0.1:3001/:path*',
      },
    ];
  },
};

export default nextConfig;
