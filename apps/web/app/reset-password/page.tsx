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

type ResetPasswordPageProps = {
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

function errorMessage(
  params?: Record<string, string | string[] | undefined>
): string | null {
  const error = pickFirst(params?.error);
  switch (error) {
    case "missing_fields":
      return "Token and new password are required.";
    case "weak_password":
      return "Password must be at least 8 characters.";
    case "invalid_token":
      return "Invalid or expired reset token.";
    default:
      return null;
  }
}

export default async function ResetPasswordPage({
  searchParams,
}: ResetPasswordPageProps) {
  const params = await searchParams;
  const token = pickFirst(params?.token) ?? "";
  const message = errorMessage(params);

  return (
    <main className="flex min-h-screen items-center justify-center bg-background p-4">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle className="text-2xl">Set New Password</CardTitle>
          <CardDescription>
            Enter the reset token and your new password.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form
            method="post"
            action="/api/auth/reset-password"
            className="flex flex-col gap-4"
          >
            <div className="flex flex-col gap-2">
              <label
                htmlFor="token"
                className="text-sm font-medium text-foreground"
              >
                Reset Token
              </label>
              <Input
                id="token"
                name="token"
                type="text"
                required
                defaultValue={token}
                placeholder="Paste your reset token"
              />
            </div>
            <div className="flex flex-col gap-2">
              <label
                htmlFor="new_password"
                className="text-sm font-medium text-foreground"
              >
                New Password
              </label>
              <Input
                id="new_password"
                name="new_password"
                type="password"
                required
                placeholder="At least 8 characters"
                minLength={8}
              />
            </div>
            <Button type="submit" className="w-full">
              Reset Password
            </Button>
          </form>
          {message && (
            <p className="mt-4 text-sm text-destructive">{message}</p>
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
