import type { ReactNode } from "react";
import { requireSession, buildApiAuthContext } from "../../lib/auth";
import { getMe, listOrgs } from "@loglens/api-client";
import ThemeToggle from "@/components/ThemeToggle";
import UserProfileMenu from "@/components/UserProfileMenu";
import Link from "next/link";
import { Flame } from "lucide-react";

type AppLayoutProps = {
  children: ReactNode;
};

function readServerApiBaseUrl() {
  return (
    process.env.API_INTERNAL_URL ??
    process.env.NEXT_PUBLIC_API_BASE_URL ??
    "http://localhost:8080"
  );
}

export default async function AppLayout({ children }: AppLayoutProps) {
  const session = await requireSession();
  const auth = buildApiAuthContext(session);
  const serverApiUrl = readServerApiBaseUrl();

  const [me, organizations] = await Promise.all([
    getMe(auth, serverApiUrl),
    listOrgs(auth, serverApiUrl),
  ]);

  return (
    <div className="min-h-screen bg-background">
      <header className="sticky top-0 z-50 border-b border-border bg-card/95 backdrop-blur supports-[backdrop-filter]:bg-card/80">
        <div className="mx-auto flex h-14 max-w-screen-2xl items-center gap-6 px-6">
          {/* Brand */}
          <Link
            href="/app"
            className="flex items-center gap-2 text-lg font-bold text-primary"
          >
            <Flame className="h-5 w-5" />
            LogLens
          </Link>

          {/* Nav links */}
          <nav className="flex items-center gap-1">
            <Link
              href="/app"
              className="rounded-md px-3 py-1.5 text-sm font-medium text-foreground transition-colors hover:bg-accent/10 hover:text-accent"
            >
              Dashboard
            </Link>
          </nav>

          {/* Right side */}
          <div className="ml-auto flex items-center gap-2">
            <ThemeToggle />
            <UserProfileMenu
              email={me.email}
              license={me.individual_license}
              organizations={organizations.orgs}
            />
          </div>
        </div>
      </header>

      <main className="mx-auto max-w-screen-2xl px-6 py-6">
        {children}
      </main>
    </div>
  );
}
