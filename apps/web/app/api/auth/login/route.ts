import { NextResponse } from "next/server";
import {
  AUTH_COOKIE_NAME,
  createSessionFromEmail,
  serializeSession,
  sessionCookieOptions,
} from "../../../../lib/auth";

function isValidEmail(value: string): boolean {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value);
}

export async function POST(request: Request) {
  const form = await request.formData();
  const email = String(form.get("email") ?? "").trim().toLowerCase();

  if (!isValidEmail(email)) {
    return NextResponse.redirect(new URL("/login?error=invalid_email", request.url), {
      status: 303,
    });
  }

  const session = createSessionFromEmail(email);
  const token = serializeSession(session);

  const response = NextResponse.redirect(new URL("/app", request.url), {
    status: 303,
  });
  response.cookies.set(AUTH_COOKIE_NAME, token, sessionCookieOptions());
  return response;
}
