import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import openapiTS, { astToString } from "openapi-typescript";

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(__dirname, "..");
const schemaPath = resolve(repoRoot, "contracts/openapi.json");
const outputPath = resolve(repoRoot, "packages/api-client/src/generated.ts");

const schema = JSON.parse(readFileSync(schemaPath, "utf8"));
if (typeof schema.openapi === "string" && schema.openapi.startsWith("3.1")) {
  schema.openapi = "3.0.3";
}

const generated = await openapiTS(schema);
const asString = astToString(generated);
const normalized = asString.endsWith("\n") ? asString : `${asString}\n`;

writeFileSync(outputPath, normalized, "utf8");
console.log(`updated ${outputPath}`);
