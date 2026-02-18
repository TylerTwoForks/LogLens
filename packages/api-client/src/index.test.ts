import { describe, expect, it } from "vitest";
import { buildUrl } from "./index.js";

describe("buildUrl", () => {
  it("normalizes trailing slash on base url", () => {
    expect(buildUrl("http://localhost:8080/", "/health")).toBe("http://localhost:8080/health");
  });

  it("keeps base url without trailing slash", () => {
    expect(buildUrl("http://localhost:8080", "/version")).toBe("http://localhost:8080/version");
  });
});
