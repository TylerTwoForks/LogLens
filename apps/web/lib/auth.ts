import { createHash, createHmac, timingSafeEqual } from "node:crypto";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";
import type { AuthContext } from "@loglens/api-client";

const AUTH_COOKIE_NAME = "loglens_session";
const SESSION_TTL_SECONDS = 60 * 60 * 24 * 7;

type SessionPayload = {
  authSubject: string;
  email: string;
  issuedAt: number;
};

function getSessionSecret(): string {
  const secret = process.env.AUTH_SESSION_SECRET;
  if (secret) {
    return secret;
  }

  if (process.env.NODE_ENV === "production") {
    throw new Error("AUTH_SESSION_SECRET is required in production");
  }

  // Dev-only fallback so local setup works out of the box.
  return "loglens-dev-session-secret";
}

function toBase64Url(value: string): string {
  return Buffer.from(value, "utf8").toString("base64url");
}

function fromBase64Url(value: string): string {
  return Buffer.from(value, "base64url").toString("utf8");
}

function signPayload(encodedPayload: string): string {
  return createHmac("sha256", getSessionSecret()).update(encodedPayload).digest("base64url");
}

function hashEmailToSubject(email: string): string {
  const digest = createHash("sha256").update(email).digest("hex");
  return `user_${digest.slice(0, 24)}`;
}

function timingSafeMatch(left: string, right: string): boolean {
  const leftBuffer = Buffer.from(left);
  const rightBuffer = Buffer.from(right);
  if (leftBuffer.length !== rightBuffer.length) {
    return false;
  }
  return timingSafeEqual(leftBuffer, rightBuffer);
}

export function createSessionFromEmail(rawEmail: string): SessionPayload {
  const email = rawEmail.trim().toLowerCase();
  return {
    authSubject: hashEmailToSubject(email),
    email,
    issuedAt: Date.now(),
  };
}

export function serializeSession(session: SessionPayload): string {
  const encodedPayload = toBase64Url(JSON.stringify(session));
  const signature = signPayload(encodedPayload);
  return `${encodedPayload}.${signature}`;
}

export function deserializeSession(token: string): SessionPayload | null {
  const [encodedPayload, signature] = token.split(".");
  if (!encodedPayload || !signature) {
    return null;
  }

  const expected = signPayload(encodedPayload);
  if (!timingSafeMatch(signature, expected)) {
    return null;
  }

  try {
    const parsed = JSON.parse(fromBase64Url(encodedPayload)) as SessionPayload;
    if (!parsed.authSubject || !parsed.email || typeof parsed.issuedAt !== "number") {
      return null;
    }
    return parsed;
  } catch {
    return null;
  }
}

export function buildApiAuthContext(session: SessionPayload): AuthContext {
  return {
    authSubject: session.authSubject,
    email: session.email,
  };
}

export async function getSession(): Promise<SessionPayload | null> {
  const cookieStore = await cookies();
  const raw = cookieStore.get(AUTH_COOKIE_NAME)?.value;
  if (!raw) {
    return null;
  }
  return deserializeSession(raw);
}

export async function requireSession(): Promise<SessionPayload> {
  const session = await getSession();
  if (!session) {
    redirect("/login");
  }
  return session;
}

export function sessionCookieOptions() {
  return {
    httpOnly: true,
    path: "/",
    sameSite: "lax" as const,
    secure: process.env.NODE_ENV === "production",
    maxAge: SESSION_TTL_SECONDS,
  };
}

export { AUTH_COOKIE_NAME };
