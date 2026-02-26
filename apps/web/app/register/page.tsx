"use client";

import { useEffect, useState } from "react";
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
import { useSearchParams } from "next/navigation";

function errorMessage(error: string | null): string | null {
  switch (error) {
    case "missing_fields":
      return "All fields are required.";
    case "password_mismatch":
      return "Passwords do not match.";
    case "weak_password":
      return "Password must be at least 8 characters.";
    case "email_taken":
      return "An account with this email already exists.";
    case "registration_failed":
      return "Registration failed. Please try again.";
    default:
      return null;
  }
}

export default function RegisterPage() {
  const searchParams = useSearchParams();
  const serverError = errorMessage(searchParams.get("error"));
  const [clientError, setClientError] = useState<string | null>(null);
  const message = clientError ?? serverError;

  function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    setClientError(null);
    const form = e.currentTarget;
    const password = (form.elements.namedItem("password") as HTMLInputElement).value;
    const confirm = (form.elements.namedItem("confirm_password") as HTMLInputElement).value;

    if (password !== confirm) {
      e.preventDefault();
      setClientError("Passwords do not match.");
    }
  }

  return (
    <main className="flex min-h-screen items-center justify-center bg-background p-4">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle className="text-2xl">Create Account</CardTitle>
          <CardDescription>
            Register a new LogLens account.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form
            method="post"
            action="/api/auth/register"
            onSubmit={handleSubmit}
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
            <div className="flex flex-col gap-2">
              <label
                htmlFor="password"
                className="text-sm font-medium text-foreground"
              >
                Password
              </label>
              <Input
                id="password"
                name="password"
                type="password"
                required
                placeholder="At least 8 characters"
                minLength={8}
              />
            </div>
            <div className="flex flex-col gap-2">
              <label
                htmlFor="confirm_password"
                className="text-sm font-medium text-foreground"
              >
                Confirm Password
              </label>
              <Input
                id="confirm_password"
                name="confirm_password"
                type="password"
                required
                placeholder="Repeat your password"
                minLength={8}
              />
            </div>
            <Button type="submit" className="w-full">
              Create Account
            </Button>
          </form>
          {message && (
            <p className="mt-4 text-sm text-destructive">{message}</p>
          )}
          <p className="mt-4 text-center text-sm text-muted-foreground">
            Already have an account?{" "}
            <Link href="/login" className="text-primary underline">
              Sign in
            </Link>
          </p>
        </CardContent>
      </Card>
    </main>
  );
}
