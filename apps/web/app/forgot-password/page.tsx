import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import Link from "next/link";

type ForgotPasswordPageProps = {
  searchParams?: Promise<Record<string, string | string[] | undefined>>;
};

function pickFirst(
  value: string | string[] | undefined
): string | null {
  if (Array.isArray(value)) {
    return value[0] ?? null;
  }
  return value ?? null;
}

export default async function ForgotPasswordPage({
  searchParams,
}: ForgotPasswordPageProps) {
  const params = await searchParams;
  const success = pickFirst(params?.success) === "true";
  const token = pickFirst(params?.token);
  const error = pickFirst(params?.error);

  return (
    <main className="flex min-h-screen items-center justify-center bg-background p-4">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle className="text-2xl">Reset Password</CardTitle>
          <CardDescription>
            Enter your email to receive a password reset token.
          </CardDescription>
        </CardHeader>
        <CardContent>
          {success ? (
            <div className="flex flex-col gap-4">
              <p className="text-sm text-muted-foreground">
                If that email is registered, a reset token has been generated.
              </p>
              {token && (
                <div className="rounded-md bg-muted p-3">
                  <p className="text-xs text-muted-foreground mb-1">
                    Dev mode — your reset token:
                  </p>
                  <code className="text-xs break-all">{token}</code>
                </div>
              )}
              <Link href={`/reset-password${token ? `?token=${token}` : ""}`}>
                <Button variant="outline" className="w-full">
                  Reset Password
                </Button>
              </Link>
            </div>
          ) : (
            <form
              method="post"
              action="/api/auth/forgot-password"
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
                Send Reset Token
              </Button>
            </form>
          )}
          {error === "missing_email" && (
            <p className="mt-4 text-sm text-destructive">
              Please enter your email.
            </p>
          )}
          <p className="mt-4 text-center text-sm text-muted-foreground">
            <Link href="/login" className="text-primary underline">
              Back to sign in
            </Link>
          </p>
        </CardContent>
      </Card>
    </main>
  );
}
