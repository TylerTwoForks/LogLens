import { spawnSync } from "node:child_process";
import { writeFileSync } from "node:fs";
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

const formatted = `${JSON.stringify(JSON.parse(result.stdout), null, 2)}\n`;
writeFileSync(contractPath, formatted, "utf8");
console.log(`updated ${contractPath}`);
