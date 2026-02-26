import { NextResponse } from "next/server";
import { apiForgotPassword } from "@loglens/api-client";

const API_BASE = process.env.API_INTERNAL_URL ?? "http://localhost:8080";

export async function POST(request: Request) {
  const form = await request.formData();
  const email = String(form.get("email") ?? "").trim().toLowerCase();

  if (!email) {
    return NextResponse.redirect(
      new URL("/forgot-password?error=missing_email", request.url),
      { status: 303 },
    );
  }

  try {
    const result = await apiForgotPassword({ email }, API_BASE);

    // In dev mode, pass the token via query param so the user can test the flow.
    if (result.reset_token) {
      return NextResponse.redirect(
        new URL(
          `/forgot-password?success=true&token=${result.reset_token}`,
          request.url,
        ),
        { status: 303 },
      );
    }
  } catch {
    // Swallow errors to avoid email enumeration
  }

  return NextResponse.redirect(
    new URL("/forgot-password?success=true", request.url),
    { status: 303 },
  );
}
