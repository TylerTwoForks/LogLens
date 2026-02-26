import { NextResponse } from "next/server";
import {
  AUTH_COOKIE_NAME,
  createSessionFromEmail,
  serializeSession,
  sessionCookieOptions,
} from "../../../../lib/auth";
import { apiRegister, buildUrl } from "@loglens/api-client";

const API_BASE = process.env.API_INTERNAL_URL ?? "http://localhost:8080";

export async function POST(request: Request) {
  const form = await request.formData();
  const email = String(form.get("email") ?? "").trim().toLowerCase();
  const password = String(form.get("password") ?? "");
  const confirmPassword = String(form.get("confirm_password") ?? "");

  if (!email || !password) {
    return NextResponse.redirect(
      new URL("/register?error=missing_fields", request.url),
      { status: 303 },
    );
  }

  if (password !== confirmPassword) {
    return NextResponse.redirect(
      new URL("/register?error=password_mismatch", request.url),
      { status: 303 },
    );
  }

  if (password.length < 8) {
    return NextResponse.redirect(
      new URL("/register?error=weak_password", request.url),
      { status: 303 },
    );
  }

  try {
    await apiRegister({ email, password }, API_BASE);
  } catch (err) {
    const message = err instanceof Error ? err.message : "";
    if (message.includes("already registered")) {
      return NextResponse.redirect(
        new URL("/register?error=email_taken", request.url),
        { status: 303 },
      );
    }
    return NextResponse.redirect(
      new URL("/register?error=registration_failed", request.url),
      { status: 303 },
    );
  }

  const session = createSessionFromEmail(email);
  const token = serializeSession(session);

  const response = NextResponse.redirect(new URL("/app", request.url), {
    status: 303,
  });
  response.cookies.set(AUTH_COOKIE_NAME, token, sessionCookieOptions());
  return response;
}
