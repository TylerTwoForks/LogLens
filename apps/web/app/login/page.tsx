import { redirect } from "next/navigation";
import { getSession } from "../../lib/auth";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";

type LoginPageProps = {
  searchParams?: Record<string, string | string[] | undefined>;
};

function pickFirst(
  value: string | string[] | undefined
): string | null {
  if (Array.isArray(value)) {
    return value[0] ?? null;
  }
  return value ?? null;
}

function errorMessage(
  searchParams?: Record<string, string | string[] | undefined>
): string | null {
  const error = pickFirst(searchParams?.error);
  if (error === "invalid_email") {
    return "Enter a valid email address.";
  }
  return null;
}

export default async function LoginPage({ searchParams }: LoginPageProps) {
  const session = await getSession();
  if (session) {
    redirect("/app");
  }

  const message = errorMessage(searchParams);

  return (
    <main className="flex min-h-screen items-center justify-center bg-background p-4">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle className="text-2xl">LogLens Sign In</CardTitle>
          <CardDescription>
            Use any email for local Phase 2 identity and organization testing.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form
            method="post"
            action="/api/auth/login"
            className="flex flex-col gap-4"
          >
            <div className="flex flex-col gap-2">
              <label
                htmlFor="email"
                className="text-sm font-medium text-foreground"
              >
                Email
              </label>
              <Input
                id="email"
                name="email"
                type="email"
                required
                placeholder="you@example.com"
              />
            </div>
            <Button type="submit" className="w-full">
              Sign in
            </Button>
          </form>
          {message && (
            <p className="mt-4 text-sm text-destructive">{message}</p>
          )}
        </CardContent>
      </Card>
    </main>
  );
}
