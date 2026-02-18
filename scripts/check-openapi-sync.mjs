import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(__dirname, "..");
const contractPath = resolve(repoRoot, "contracts/openapi.json");

const result = spawnSync("cargo", ["run", "--quiet", "-p", "loglens-api", "--", "print-openapi"], {
  cwd: repoRoot,
  encoding: "utf8",
});

if (result.error) {
  console.error(`failed to execute cargo: ${result.error.message}`);
  process.exit(1);
}

if (result.status !== 0) {
  console.error((result.stderr ?? "cargo command failed").trim());
  process.exit(result.status ?? 1);
}

const generated = `${JSON.stringify(JSON.parse(result.stdout), null, 2)}\n`;
const existing = readFileSync(contractPath, "utf8");

if (generated !== existing) {
  console.error("contracts/openapi.json is out of sync. Run: pnpm run contract:generate");
  process.exit(1);
}

console.log("openapi contract is in sync");
