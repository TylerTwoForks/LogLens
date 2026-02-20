import {
  listOrgs,
  listJobs,
} from "@loglens/api-client";
import { buildApiAuthContext, requireSession } from "../../lib/auth";
import { createOrgAction } from "./actions";
import DashboardWorkspace from "../../components/DashboardWorkspace";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import Link from "next/link";
import { Building2 } from "lucide-react";

export const dynamic = "force-dynamic";

type AppPageProps = {
  searchParams?: Promise<Record<string, string | string[] | undefined>>;
};

function readServerApiBaseUrl() {
  return (
    process.env.API_INTERNAL_URL ??
    process.env.NEXT_PUBLIC_API_BASE_URL ??
    "http://localhost:8080"
  );
}

function readBrowserApiBaseUrl() {
  return "/api/proxy";
}

function pickFirst(value: string | string[] | undefined): string | null {
  if (Array.isArray(value)) {
    return value[0] ?? null;
  }
  return value ?? null;
}

function parseOptionalNumber(value: string | null): number | null {
  if (!value) {
    return null;
  }
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : null;
}

function noticeOrError(
  searchParams?: Record<string, string | string[] | undefined>
) {
  const notice = pickFirst(searchParams?.notice);
  const error = pickFirst(searchParams?.error);
  return { notice, error };
}

export default async function AppPage({ searchParams }: AppPageProps) {
  const resolvedSearchParams = searchParams ? await searchParams : undefined;
  const session = await requireSession();
  const auth = buildApiAuthContext(session);
  const serverApiUrl = readServerApiBaseUrl();
  const browserApiUrl = readBrowserApiBaseUrl();

  const organizations = await listOrgs(auth, serverApiUrl);
  const requestedOrgId = parseOptionalNumber(
    pickFirst(resolvedSearchParams?.org)
  );
  const selectedOrg =
    organizations.orgs.find((org) => org.org_id === requestedOrgId) ??
    organizations.orgs[0] ??
    null;

  const orgJobs = selectedOrg
    ? await listJobs(auth, selectedOrg.org_id, {}, serverApiUrl)
    : null;

  const { notice, error } = noticeOrError(resolvedSearchParams);

  return (
    <div className="space-y-6">
      {notice && (
        <div className="rounded-md border border-info bg-info/10 px-4 py-3 text-sm text-info">
          {notice.replaceAll("_", " ")}
        </div>
      )}
      {error && (
        <div className="rounded-md border border-destructive bg-destructive/10 px-4 py-3 text-sm text-destructive">
          {error}
        </div>
      )}

      {/* Organization workspace */}
      {selectedOrg ? (
        <>
          {/* Org header */}
          <div className="flex flex-wrap items-center justify-between gap-4">
            <div className="flex items-center gap-3">
              <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-accent/10 text-accent">
                <Building2 className="h-5 w-5" />
              </div>
              <div>
                <h2 className="text-xl font-semibold text-foreground">
                  {selectedOrg.name}
                </h2>
                <div className="flex items-center gap-2 text-sm text-muted-foreground">
                  <Badge variant="secondary" className="text-xs">
                    {selectedOrg.license.tier}
                  </Badge>
                  <Badge
                    variant="outline"
                    className={
                      selectedOrg.license.status === "active"
                        ? "border-success text-success text-xs"
                        : "text-xs"
                    }
                  >
                    {selectedOrg.license.status}
                  </Badge>
                  <span className="text-xs">
                    Your role: {selectedOrg.role}
                  </span>
                </div>
              </div>
            </div>

            {/* Org switcher (if multiple orgs) */}
            {organizations.orgs.length > 1 && (
              <div className="flex gap-1">
                {organizations.orgs.map((org) => (
                  <Button
                    key={org.org_id}
                    variant={
                      org.org_id === selectedOrg.org_id
                        ? "default"
                        : "outline"
                    }
                    size="sm"
                    asChild
                  >
                    <Link href={`/app?org=${org.org_id}`}>{org.name}</Link>
                  </Button>
                ))}
              </div>
            )}
          </div>

          {/* Workspace: upload + jobs + inline detail */}
          <DashboardWorkspace
            orgId={selectedOrg.org_id}
            auth={auth}
            initialJobs={orgJobs?.jobs ?? []}
            apiBaseUrl={browserApiUrl}
          />
        </>
      ) : (
        /* No org selected -- onboarding */
        <Card className="mx-auto max-w-lg">
          <CardHeader className="text-center">
            <div className="mx-auto mb-4 flex h-14 w-14 items-center justify-center rounded-full bg-primary/10 text-primary">
              <Building2 className="h-7 w-7" />
            </div>
            <CardTitle>Get Started</CardTitle>
            <CardDescription>
              Create your first organization to start analyzing Salesforce
              debug logs.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form
              action={createOrgAction}
              className="flex items-end gap-3"
            >
              <div className="flex flex-1 flex-col gap-1">
                <label
                  htmlFor="org-name"
                  className="text-xs font-medium text-muted-foreground"
                >
                  Organization name
                </label>
                <Input
                  id="org-name"
                  name="name"
                  type="text"
                  required
                  placeholder="My Organization"
                />
              </div>
              <Button type="submit">Create</Button>
            </form>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
