import {
  getMe,
  listOrgs,
  listOrgMembers,
} from "@loglens/api-client";
import { buildApiAuthContext, requireSession } from "../../../lib/auth";
import {
  profileUpdateIndividualLicenseAction,
  profileUpdateOrgLicenseAction,
  profileUpdateOrgMemberRoleAction,
  profileCreateOrgAction,
} from "./actions";
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
import {
  User,
  Shield,
  Building2,
  Users,
  Settings,
  Plus,
  Lock,
  Mail,
  CreditCard,
  ArrowLeft,
} from "lucide-react";

export const dynamic = "force-dynamic";

type ProfilePageProps = {
  searchParams?: Promise<Record<string, string | string[] | undefined>>;
};

function readServerApiBaseUrl() {
  return (
    process.env.API_INTERNAL_URL ??
    process.env.NEXT_PUBLIC_API_BASE_URL ??
    "http://localhost:8080"
  );
}

function pickFirst(value: string | string[] | undefined): string | null {
  if (Array.isArray(value)) {
    return value[0] ?? null;
  }
  return value ?? null;
}

function parseOptionalNumber(value: string | null): number | null {
  if (!value) return null;
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : null;
}

function canManageOrg(
  role: "owner" | "admin" | "member" | "viewer"
): boolean {
  return role === "owner" || role === "admin";
}

const ROLE_RANK: Record<string, number> = {
  owner: 4,
  admin: 3,
  member: 2,
  viewer: 1,
};

const ALL_ROLES = ["owner", "admin", "member", "viewer"] as const;

const selectClasses =
  "h-9 rounded-md border border-input bg-background px-3 text-sm text-foreground shadow-xs focus:outline-none focus:ring-2 focus:ring-ring";

export default async function ProfilePage({
  searchParams,
}: ProfilePageProps) {
  const resolvedSearchParams = searchParams ? await searchParams : {};
  const session = await requireSession();
  const auth = buildApiAuthContext(session);
  const serverApiUrl = readServerApiBaseUrl();

  const [me, organizations] = await Promise.all([
    getMe(auth, serverApiUrl),
    listOrgs(auth, serverApiUrl),
  ]);

  const notice = pickFirst(resolvedSearchParams?.notice);
  const error = pickFirst(resolvedSearchParams?.error);

  const requestedOrgId = parseOptionalNumber(
    pickFirst(resolvedSearchParams?.org)
  );
  const adminOrgs = organizations.orgs.filter((o) => canManageOrg(o.role));
  const selectedAdminOrg =
    adminOrgs.find((o) => o.org_id === requestedOrgId) ??
    adminOrgs[0] ??
    null;

  const members = selectedAdminOrg
    ? await listOrgMembers(auth, selectedAdminOrg.org_id, serverApiUrl)
    : null;

  const isAdmin = adminOrgs.length > 0;

  return (
    <div className="space-y-8">
      {/* Page header */}
      <div className="flex items-center gap-4">
        <Button variant="ghost" size="icon" asChild>
          <Link href="/app">
            <ArrowLeft className="h-4 w-4" />
          </Link>
        </Button>
        <div>
          <h1 className="text-2xl font-semibold text-foreground">Profile</h1>
          <p className="text-sm text-muted-foreground">
            Manage your account settings and preferences
          </p>
        </div>
      </div>

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

      {/* ─── User Profile Section ─── */}
      <section className="space-y-6">
        <h2 className="flex items-center gap-2 text-lg font-semibold text-foreground">
          <User className="h-5 w-5 text-primary" />
          Account
        </h2>

        <div className="grid gap-6 lg:grid-cols-2">
          {/* Personal Information */}
          <Card>
            <CardHeader>
              <CardTitle className="text-base">
                Personal Information
              </CardTitle>
              <CardDescription>
                Your profile details
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-4 sm:grid-cols-2">
                <div className="flex flex-col gap-1.5">
                  <label className="text-xs font-medium text-muted-foreground">
                    First Name
                  </label>
                  <Input
                    placeholder="First name"
                    disabled
                    className="disabled:opacity-60"
                  />
                </div>
                <div className="flex flex-col gap-1.5">
                  <label className="text-xs font-medium text-muted-foreground">
                    Last Name
                  </label>
                  <Input
                    placeholder="Last name"
                    disabled
                    className="disabled:opacity-60"
                  />
                </div>
              </div>
              <div className="flex flex-col gap-1.5">
                <label className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
                  <Mail className="h-3 w-3" />
                  Email
                </label>
                <Input
                  value={me.email}
                  disabled
                  className="disabled:opacity-60"
                />
              </div>
              <p className="text-xs text-muted-foreground">
                Name and email changes will be available in a future
                release.
              </p>
            </CardContent>
          </Card>

          {/* Security */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-base">
                <Lock className="h-4 w-4 text-muted-foreground" />
                Security
              </CardTitle>
              <CardDescription>
                Authentication and password management
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex flex-col gap-1.5">
                <label className="text-xs font-medium text-muted-foreground">
                  Auth Subject
                </label>
                <Input
                  value={me.auth_subject}
                  disabled
                  className="font-mono text-xs disabled:opacity-60"
                />
              </div>
              <div className="flex flex-col gap-1.5">
                <label className="text-xs font-medium text-muted-foreground">
                  Password
                </label>
                <Input
                  type="password"
                  value="••••••••"
                  disabled
                  className="disabled:opacity-60"
                />
              </div>
              <p className="text-xs text-muted-foreground">
                Password management will be available when full
                authentication is implemented.
              </p>
            </CardContent>
          </Card>

          {/* Individual License */}
          <Card className="lg:col-span-2">
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-base">
                <CreditCard className="h-4 w-4 text-muted-foreground" />
                Individual License
              </CardTitle>
              <CardDescription className="flex items-center gap-2">
                <Badge variant="secondary" className="text-xs">
                  {me.individual_license.tier}
                </Badge>
                <Badge
                  variant="outline"
                  className={
                    me.individual_license.status === "active"
                      ? "border-success text-success text-xs"
                      : "text-xs"
                  }
                >
                  {me.individual_license.status}
                </Badge>
                <span className="text-xs">
                  Entitlements:{" "}
                  {me.individual_license.features.join(", ") || "(none)"}
                </span>
              </CardDescription>
            </CardHeader>
            <CardContent>
              <form
                action={profileUpdateIndividualLicenseAction}
                className="flex flex-wrap items-end gap-3"
              >
                <div className="flex flex-col gap-1">
                  <label
                    htmlFor="individual-tier"
                    className="text-xs font-medium text-muted-foreground"
                  >
                    Tier
                  </label>
                  <select
                    id="individual-tier"
                    name="tier"
                    defaultValue={me.individual_license.tier}
                    className={selectClasses}
                  >
                    <option value="free">free</option>
                    <option value="pro">pro</option>
                    <option value="enterprise">enterprise</option>
                  </select>
                </div>
                <div className="flex flex-col gap-1">
                  <label
                    htmlFor="individual-status"
                    className="text-xs font-medium text-muted-foreground"
                  >
                    Status
                  </label>
                  <select
                    id="individual-status"
                    name="status"
                    defaultValue={me.individual_license.status}
                    className={selectClasses}
                  >
                    <option value="active">active</option>
                    <option value="past_due">past_due</option>
                    <option value="canceled">canceled</option>
                  </select>
                </div>
                <Button type="submit" size="sm">
                  Update license
                </Button>
              </form>
            </CardContent>
          </Card>
        </div>
      </section>

      {/* ─── Admin / Organization Section ─── */}
      {isAdmin && (
        <section className="space-y-6">
          <div className="flex items-center gap-2 border-t border-border pt-8">
            <Shield className="h-5 w-5 text-primary" />
            <h2 className="text-lg font-semibold text-foreground">
              Administration
            </h2>
          </div>

          {/* Org switcher for admin panel */}
          {adminOrgs.length > 1 && (
            <div className="flex flex-wrap items-center gap-2">
              <span className="text-sm text-muted-foreground">
                Managing:
              </span>
              {adminOrgs.map((org) => (
                <Button
                  key={org.org_id}
                  variant={
                    selectedAdminOrg?.org_id === org.org_id
                      ? "default"
                      : "outline"
                  }
                  size="sm"
                  asChild
                >
                  <Link href={`/app/profile?org=${org.org_id}`}>
                    {org.name}
                  </Link>
                </Button>
              ))}
            </div>
          )}

          {selectedAdminOrg && (
            <div className="grid gap-6 lg:grid-cols-2">
              {/* Organization Settings */}
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2 text-base">
                    <Settings className="h-4 w-4 text-muted-foreground" />
                    Organization Settings
                  </CardTitle>
                  <CardDescription>
                    {selectedAdminOrg.name} &mdash; License &amp;
                    entitlements
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="flex items-center gap-2">
                    <Badge variant="secondary" className="text-xs">
                      {selectedAdminOrg.license.tier}
                    </Badge>
                    <Badge
                      variant="outline"
                      className={
                        selectedAdminOrg.license.status === "active"
                          ? "border-success text-success text-xs"
                          : "text-xs"
                      }
                    >
                      {selectedAdminOrg.license.status}
                    </Badge>
                  </div>
                  <p className="text-sm text-muted-foreground">
                    Entitlements:{" "}
                    {selectedAdminOrg.license.features.join(", ") ||
                      "(none)"}
                  </p>
                  <form
                    action={profileUpdateOrgLicenseAction}
                    className="flex flex-wrap items-end gap-3"
                  >
                    <input
                      type="hidden"
                      name="orgId"
                      value={selectedAdminOrg.org_id}
                    />
                    <div className="flex flex-col gap-1">
                      <label
                        htmlFor="org-tier"
                        className="text-xs font-medium text-muted-foreground"
                      >
                        Tier
                      </label>
                      <select
                        id="org-tier"
                        name="tier"
                        defaultValue={selectedAdminOrg.license.tier}
                        className={selectClasses}
                      >
                        <option value="free">free</option>
                        <option value="pro">pro</option>
                        <option value="enterprise">enterprise</option>
                      </select>
                    </div>
                    <div className="flex flex-col gap-1">
                      <label
                        htmlFor="org-status"
                        className="text-xs font-medium text-muted-foreground"
                      >
                        Status
                      </label>
                      <select
                        id="org-status"
                        name="status"
                        defaultValue={selectedAdminOrg.license.status}
                        className={selectClasses}
                      >
                        <option value="active">active</option>
                        <option value="past_due">past_due</option>
                        <option value="canceled">canceled</option>
                      </select>
                    </div>
                    <Button type="submit" size="sm">
                      Update
                    </Button>
                  </form>
                </CardContent>
              </Card>

              {/* Members & Roles */}
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2 text-base">
                    <Users className="h-4 w-4 text-muted-foreground" />
                    Members &amp; Roles
                  </CardTitle>
                  <CardDescription>
                    {(members?.members ?? []).length} member
                    {(members?.members ?? []).length !== 1 ? "s" : ""} in{" "}
                    {selectedAdminOrg.name}
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <ul className="space-y-1.5">
                    {(members?.members ?? []).map((member) => {
                      const isSelf = member.user_id === me.user_id;
                      return (
                        <li
                          key={member.user_id}
                          className="flex items-center gap-2 text-sm"
                        >
                          <span className="font-mono text-xs text-muted-foreground">
                            #{member.user_id}
                          </span>
                          <span className="flex-1 truncate text-foreground">
                            {member.email}
                            {isSelf && (
                              <span className="ml-1 text-xs text-muted-foreground">
                                (you)
                              </span>
                            )}
                          </span>
                          <Badge variant="outline" className="text-xs">
                            {member.role}
                          </Badge>
                        </li>
                      );
                    })}
                  </ul>

                  <form
                    action={profileUpdateOrgMemberRoleAction}
                    className="flex flex-wrap items-end gap-3 border-t border-border pt-4"
                  >
                    <input
                      type="hidden"
                      name="orgId"
                      value={selectedAdminOrg.org_id}
                    />
                    <div className="flex flex-col gap-1">
                      <label
                        htmlFor="member-user-id"
                        className="text-xs font-medium text-muted-foreground"
                      >
                        Member
                      </label>
                      <select
                        id="member-user-id"
                        name="memberUserId"
                        className={selectClasses}
                      >
                        {(members?.members ?? []).map((member) => (
                          <option
                            key={member.user_id}
                            value={member.user_id}
                          >
                            #{member.user_id} {member.email}
                            {member.user_id === me.user_id ? " (you)" : ""}
                          </option>
                        ))}
                      </select>
                    </div>
                    <div className="flex flex-col gap-1">
                      <label
                        htmlFor="member-role"
                        className="text-xs font-medium text-muted-foreground"
                      >
                        Role
                      </label>
                      <select
                        id="member-role"
                        name="role"
                        defaultValue="member"
                        className={selectClasses}
                      >
                        {ALL_ROLES.map((r) => (
                          <option key={r} value={r}>
                            {r}
                          </option>
                        ))}
                      </select>
                    </div>
                    <Button type="submit" size="sm">
                      Update role
                    </Button>
                  </form>
                  <p className="text-xs text-muted-foreground">
                    You cannot lower your own role. Ask another owner or
                    admin to change it.
                  </p>
                </CardContent>
              </Card>
            </div>
          )}

          {/* Create new organization */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-base">
                <Plus className="h-4 w-4 text-muted-foreground" />
                Create Organization
              </CardTitle>
            </CardHeader>
            <CardContent>
              <form
                action={profileCreateOrgAction}
                className="flex items-end gap-3"
              >
                <div className="flex flex-1 flex-col gap-1">
                  <label
                    htmlFor="new-org-name"
                    className="text-xs font-medium text-muted-foreground"
                  >
                    Organization name
                  </label>
                  <Input
                    id="new-org-name"
                    name="name"
                    type="text"
                    required
                    placeholder="My Organization"
                  />
                </div>
                <Button type="submit" size="sm">
                  Create
                </Button>
              </form>
            </CardContent>
          </Card>
        </section>
      )}

      {/* Organizations list for non-admins (they can still see their orgs) */}
      {!isAdmin && organizations.orgs.length > 0 && (
        <section className="space-y-4">
          <h2 className="flex items-center gap-2 text-lg font-semibold text-foreground">
            <Building2 className="h-5 w-5 text-primary" />
            Organizations
          </h2>
          <Card>
            <CardContent className="pt-6">
              <ul className="space-y-2">
                {organizations.orgs.map((org) => (
                  <li
                    key={org.org_id}
                    className="flex items-center gap-2 text-sm"
                  >
                    <span className="flex-1 text-foreground">
                      {org.name}
                    </span>
                    <Badge variant="outline" className="text-xs">
                      {org.role}
                    </Badge>
                    <Badge variant="secondary" className="text-xs">
                      {org.license.tier}
                    </Badge>
                  </li>
                ))}
              </ul>
            </CardContent>
          </Card>
        </section>
      )}

      {/* Create org for non-admins without any orgs */}
      {!isAdmin && organizations.orgs.length === 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base">
              <Plus className="h-4 w-4 text-muted-foreground" />
              Create Organization
            </CardTitle>
            <CardDescription>
              Create your first organization to get started.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form
              action={profileCreateOrgAction}
              className="flex items-end gap-3"
            >
              <div className="flex flex-1 flex-col gap-1">
                <label
                  htmlFor="new-org-name-first"
                  className="text-xs font-medium text-muted-foreground"
                >
                  Organization name
                </label>
                <Input
                  id="new-org-name-first"
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
