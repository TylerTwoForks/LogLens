import { NextResponse } from "next/server";
import { apiResetPassword } from "@loglens/api-client";

const API_BASE = process.env.API_INTERNAL_URL ?? "http://localhost:8080";

export async function POST(request: Request) {
  const form = await request.formData();
  const token = String(form.get("token") ?? "");
  const newPassword = String(form.get("new_password") ?? "");

  if (!token || !newPassword) {
    return NextResponse.redirect(
      new URL("/reset-password?error=missing_fields", request.url),
      { status: 303 },
    );
  }

  if (newPassword.length < 8) {
    return NextResponse.redirect(
      new URL("/reset-password?error=weak_password", request.url),
      { status: 303 },
    );
  }

  try {
    await apiResetPassword({ token, new_password: newPassword }, API_BASE);
  } catch {
    return NextResponse.redirect(
      new URL("/reset-password?error=invalid_token", request.url),
      { status: 303 },
    );
  }

  return NextResponse.redirect(
    new URL("/login?reset=success", request.url),
    { status: 303 },
  );
}
