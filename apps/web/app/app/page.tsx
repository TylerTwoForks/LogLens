import { getMe, listOrgMembers, listOrgs, listJobs } from "@loglens/api-client";
import { buildApiAuthContext, requireSession } from "../../lib/auth";
import {
  createOrgAction,
  updateIndividualLicenseAction,
  updateOrgLicenseAction,
  updateOrgMemberRoleAction,
} from "./actions";
import UploadPanel from "../../components/UploadPanel";

export const dynamic = "force-dynamic";

type AppPageProps = {
  searchParams?: Promise<Record<string, string | string[] | undefined>>;
};

function readServerApiBaseUrl() {
  return process.env.API_INTERNAL_URL ?? process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:8080";
}

function readBrowserApiBaseUrl() {
  return process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:8080";
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

function canManageOrg(role: "owner" | "admin" | "member" | "viewer"): boolean {
  return role === "owner" || role === "admin";
}

function noticeOrError(searchParams?: Record<string, string | string[] | undefined>) {
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

  const [me, organizations] = await Promise.all([getMe(auth, serverApiUrl), listOrgs(auth, serverApiUrl)]);
  const requestedOrgId = parseOptionalNumber(pickFirst(resolvedSearchParams?.org));
  const selectedOrg =
    organizations.orgs.find((org) => org.org_id === requestedOrgId) ?? organizations.orgs[0] ?? null;

  const [members, orgJobs] = await Promise.all([
    selectedOrg ? listOrgMembers(auth, selectedOrg.org_id, serverApiUrl) : Promise.resolve(null),
    selectedOrg ? listJobs(auth, selectedOrg.org_id, {}, serverApiUrl) : Promise.resolve(null),
  ]);
  const canManageSelectedOrg = selectedOrg ? canManageOrg(selectedOrg.role) : false;
  const { notice, error } = noticeOrError(resolvedSearchParams);

  return (
    <section>
      <h2>Phase 2 Dashboard</h2>
      <p>API base URL: {serverApiUrl}</p>
      {notice ? <p>Notice: {notice}</p> : null}
      {error ? <p>Error: {error}</p> : null}

      <section>
        <h3>Authenticated User</h3>
        <ul>
          <li>Auth subject: {me.auth_subject}</li>
          <li>Email: {me.email}</li>
          <li>Individual tier: {me.individual_license.tier}</li>
          <li>Individual status: {me.individual_license.status}</li>
        </ul>
        <p>Entitlements: {me.individual_license.features.join(", ") || "(none)"}</p>
        <form action={updateIndividualLicenseAction}>
          <label htmlFor="individual-tier">Tier</label>
          <select id="individual-tier" name="tier" defaultValue={me.individual_license.tier}>
            <option value="free">free</option>
            <option value="pro">pro</option>
            <option value="enterprise">enterprise</option>
          </select>
          <label htmlFor="individual-status">Status</label>
          <select id="individual-status" name="status" defaultValue={me.individual_license.status}>
            <option value="active">active</option>
            <option value="past_due">past_due</option>
            <option value="canceled">canceled</option>
          </select>
          <button type="submit">Update individual license</button>
        </form>
      </section>

      <section>
        <h3>Organizations</h3>
        <ul>
          {organizations.orgs.map((org) => (
            <li key={org.org_id}>
              <a href={`/app?org=${org.org_id}`}>
                {org.name} (#{org.org_id}) - role {org.role}
              </a>
            </li>
          ))}
        </ul>
        <form action={createOrgAction}>
          <label htmlFor="org-name">Create organization</label>
          <input id="org-name" name="name" type="text" required />
          <button type="submit">Create</button>
        </form>
      </section>

      {selectedOrg ? (
        <section>
          <h3>Selected Organization</h3>
          <ul>
            <li>Name: {selectedOrg.name}</li>
            <li>Role: {selectedOrg.role}</li>
            <li>Tier: {selectedOrg.license.tier}</li>
            <li>Status: {selectedOrg.license.status}</li>
          </ul>
          <p>Entitlements: {selectedOrg.license.features.join(", ") || "(none)"}</p>

          {canManageSelectedOrg ? (
            <form action={updateOrgLicenseAction}>
              <input type="hidden" name="orgId" value={selectedOrg.org_id} />
              <label htmlFor="org-tier">Tier</label>
              <select id="org-tier" name="tier" defaultValue={selectedOrg.license.tier}>
                <option value="free">free</option>
                <option value="pro">pro</option>
                <option value="enterprise">enterprise</option>
              </select>
              <label htmlFor="org-status">Status</label>
              <select id="org-status" name="status" defaultValue={selectedOrg.license.status}>
                <option value="active">active</option>
                <option value="past_due">past_due</option>
                <option value="canceled">canceled</option>
              </select>
              <button type="submit">Update organization license</button>
            </form>
          ) : (
            <p>Billing controls are limited to owner/admin roles.</p>
          )}

          <h4>Members</h4>
          <ul>
            {(members?.members ?? []).map((member) => (
              <li key={member.user_id}>
                #{member.user_id} {member.email} - {member.role}
              </li>
            ))}
          </ul>

          {canManageSelectedOrg ? (
            <form action={updateOrgMemberRoleAction}>
              <input type="hidden" name="orgId" value={selectedOrg.org_id} />
              <label htmlFor="member-user-id">Member</label>
              <select id="member-user-id" name="memberUserId">
                {(members?.members ?? []).map((member) => (
                  <option key={member.user_id} value={member.user_id}>
                    #{member.user_id} {member.email}
                  </option>
                ))}
              </select>
              <label htmlFor="member-role">Role</label>
              <select id="member-role" name="role" defaultValue="member">
                <option value="owner">owner</option>
                <option value="admin">admin</option>
                <option value="member">member</option>
                <option value="viewer">viewer</option>
              </select>
              <button type="submit">Update member role</button>
            </form>
          ) : (
            <p>Member role controls are limited to owner/admin roles.</p>
          )}

          <UploadPanel
            orgId={selectedOrg.org_id}
            auth={auth}
            initialJobs={orgJobs?.jobs ?? []}
            apiBaseUrl={browserApiUrl}
          />
        </section>
      ) : (
        <section>
          <p>No organizations yet. Create one to start organization-scoped testing.</p>
        </section>
      )}
    </section>
  );
}
