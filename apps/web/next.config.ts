import type { NextConfig } from "next";

const apiBackend =
  process.env.API_INTERNAL_URL ??
  process.env.NEXT_PUBLIC_API_BASE_URL ??
  "http://localhost:8080";

const nextConfig: NextConfig = {
  transpilePackages: ["@loglens/api-client"],
  async rewrites() {
    return [
      {
        source: "/api/proxy/:path*",
        destination: `${apiBackend}/:path*`,
      },
    ];
  },
};

export default nextConfig;
