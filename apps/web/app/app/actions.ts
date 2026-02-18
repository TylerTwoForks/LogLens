"use server";

import {
  createOrg,
  updateMeLicense,
  updateOrgLicense,
  updateOrgMemberRole,
  type LicenseStatus,
  type LicenseTier,
  type OrgRole,
} from "@loglens/api-client";
import { redirect } from "next/navigation";
import { buildApiAuthContext, requireSession } from "../../lib/auth";

const LICENSE_TIERS = ["free", "pro", "enterprise"] as const;
const LICENSE_STATUSES = ["active", "past_due", "canceled"] as const;
const ORG_ROLES = ["owner", "admin", "member", "viewer"] as const;

function readApiBaseUrl() {
  return process.env.API_INTERNAL_URL ?? process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:8080";
}

function parseIntField(value: FormDataEntryValue | null, fieldName: string): number {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) {
    throw new Error(`invalid ${fieldName}`);
  }
  return parsed;
}

function parseLicenseTier(value: FormDataEntryValue | null): LicenseTier {
  if (value && LICENSE_TIERS.includes(value as (typeof LICENSE_TIERS)[number])) {
    return value as LicenseTier;
  }
  throw new Error("invalid license tier");
}

function parseLicenseStatus(value: FormDataEntryValue | null): LicenseStatus {
  if (value && LICENSE_STATUSES.includes(value as (typeof LICENSE_STATUSES)[number])) {
    return value as LicenseStatus;
  }
  throw new Error("invalid license status");
}

function parseRole(value: FormDataEntryValue | null): OrgRole {
  if (value && ORG_ROLES.includes(value as (typeof ORG_ROLES)[number])) {
    return value as OrgRole;
  }
  throw new Error("invalid role");
}

function isNextRedirectError(error: unknown): boolean {
  return (
    error instanceof Error &&
    "digest" in error &&
    typeof (error as Record<string, unknown>).digest === "string" &&
    ((error as Record<string, unknown>).digest as string).startsWith("NEXT_REDIRECT")
  );
}

function safeRedirect(path: string, error: unknown): never {
  if (isNextRedirectError(error)) {
    throw error;
  }
  const message = error instanceof Error ? error.message : "request_failed";
  redirect(`${path}${path.includes("?") ? "&" : "?"}error=${encodeURIComponent(message)}`);
}

export async function createOrgAction(formData: FormData) {
  const session = await requireSession();
  const auth = buildApiAuthContext(session);
  const apiBaseUrl = readApiBaseUrl();
  const name = String(formData.get("name") ?? "").trim();

  if (!name) {
    redirect("/app?error=organization_name_required");
  }

  let redirectUrl: string;
  try {
    const created = await createOrg(auth, { name }, apiBaseUrl);
    redirectUrl = `/app?org=${created.org_id}&notice=organization_created`;
  } catch (error) {
    safeRedirect("/app", error);
  }

  redirect(redirectUrl!);
}

export async function updateIndividualLicenseAction(formData: FormData) {
  const session = await requireSession();
  const auth = buildApiAuthContext(session);
  const apiBaseUrl = readApiBaseUrl();

  let redirectUrl: string;
  try {
    const tier = parseLicenseTier(formData.get("tier"));
    const status = parseLicenseStatus(formData.get("status"));
    await updateMeLicense(auth, { tier, status }, apiBaseUrl);
    redirectUrl = "/app?notice=individual_license_updated";
  } catch (error) {
    safeRedirect("/app", error);
  }

  redirect(redirectUrl!);
}

export async function updateOrgLicenseAction(formData: FormData) {
  const session = await requireSession();
  const auth = buildApiAuthContext(session);
  const apiBaseUrl = readApiBaseUrl();

  let redirectUrl: string;
  try {
    const orgId = parseIntField(formData.get("orgId"), "orgId");
    const tier = parseLicenseTier(formData.get("tier"));
    const status = parseLicenseStatus(formData.get("status"));
    await updateOrgLicense(auth, orgId, { tier, status }, apiBaseUrl);
    redirectUrl = `/app?org=${orgId}&notice=organization_license_updated`;
  } catch (error) {
    safeRedirect(`/app`, error);
  }

  redirect(redirectUrl!);
}

export async function updateOrgMemberRoleAction(formData: FormData) {
  const session = await requireSession();
  const auth = buildApiAuthContext(session);
  const apiBaseUrl = readApiBaseUrl();

  let redirectUrl: string;
  try {
    const orgId = parseIntField(formData.get("orgId"), "orgId");
    const memberUserId = parseIntField(formData.get("memberUserId"), "memberUserId");
    const role = parseRole(formData.get("role"));
    await updateOrgMemberRole(auth, orgId, memberUserId, { role }, apiBaseUrl);
    redirectUrl = `/app?org=${orgId}&notice=member_role_updated`;
  } catch (error) {
    safeRedirect("/app", error);
  }

  redirect(redirectUrl!);
}
