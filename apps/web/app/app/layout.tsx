import type { ReactNode } from "react";
import { requireSession } from "../../lib/auth";

type AppLayoutProps = {
  children: ReactNode;
};

export default async function AppLayout({ children }: AppLayoutProps) {
  const session = await requireSession();

  return (
    <main>
      <header>
        <h1>LogLens App</h1>
        <p>Signed in as {session.email}</p>
        <form method="post" action="/api/auth/logout">
          <button type="submit">Sign out</button>
        </form>
      </header>
      {children}
    </main>
  );
}
