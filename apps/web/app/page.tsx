import { getHealth, getVersion } from "@loglens/api-client";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import Link from "next/link";

export const dynamic = "force-dynamic";

function readServerApiBaseUrl() {
  return (
    process.env.API_INTERNAL_URL ??
    process.env.NEXT_PUBLIC_API_BASE_URL ??
    "http://localhost:8080"
  );
}

export default async function HomePage() {
  const apiBaseUrl = readServerApiBaseUrl();

  try {
    const [health, version] = await Promise.all([
      getHealth(apiBaseUrl),
      getVersion(apiBaseUrl),
    ]);

    return (
      <main className="flex min-h-screen flex-col items-center justify-center gap-6 bg-background p-4">
        <h1 className="text-4xl font-bold text-foreground">LogLens</h1>
        <div className="flex gap-3">
          <Button asChild>
            <Link href="/login">Sign in</Link>
          </Button>
          <Button asChild variant="outline">
            <Link href="/app">Open app dashboard</Link>
          </Button>
        </div>
        <Card className="w-full max-w-md">
          <CardHeader>
            <CardTitle className="text-base">API Status</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 text-sm">
            <p className="text-muted-foreground">
              Base URL: <code className="text-foreground">{apiBaseUrl}</code>
            </p>
            <div className="flex items-center gap-2">
              <span className="text-muted-foreground">Health:</span>
              <Badge variant="outline">{health.status}</Badge>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-muted-foreground">Database:</span>
              <Badge variant="outline">{health.database}</Badge>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-muted-foreground">Version:</span>
              <Badge variant="secondary">{version.version}</Badge>
            </div>
          </CardContent>
        </Card>
      </main>
    );
  } catch (error) {
    const message =
      error instanceof Error ? error.message : "Unknown API error";

    return (
      <main className="flex min-h-screen flex-col items-center justify-center gap-6 bg-background p-4">
        <h1 className="text-4xl font-bold text-foreground">LogLens</h1>
        <div className="flex gap-3">
          <Button asChild>
            <Link href="/login">Sign in</Link>
          </Button>
          <Button asChild variant="outline">
            <Link href="/app">Open app dashboard</Link>
          </Button>
        </div>
        <Card className="w-full max-w-md border-destructive">
          <CardHeader>
            <CardTitle className="text-base text-destructive">
              API Unreachable
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 text-sm">
            <p className="text-muted-foreground">
              Base URL: <code className="text-foreground">{apiBaseUrl}</code>
            </p>
            <p className="text-destructive">{message}</p>
          </CardContent>
        </Card>
      </main>
    );
  }
}
