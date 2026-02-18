import { redirect } from "next/navigation";
import { getSession } from "../../lib/auth";

type LoginPageProps = {
  searchParams?: Record<string, string | string[] | undefined>;
};

function pickFirst(value: string | string[] | undefined): string | null {
  if (Array.isArray(value)) {
    return value[0] ?? null;
  }
  return value ?? null;
}

function errorMessage(searchParams?: Record<string, string | string[] | undefined>): string | null {
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
    <main>
      <h1>LogLens Sign In</h1>
      <p>Use any email for local Phase 2 identity and organization testing.</p>
      <form method="post" action="/api/auth/login">
        <label htmlFor="email">Email</label>
        <br />
        <input id="email" name="email" type="email" required placeholder="you@example.com" />
        <button type="submit">Sign in</button>
      </form>
      {message ? <p>{message}</p> : null}
    </main>
  );
}
