import fs from "fs";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const projectName = process.argv[process.argv.indexOf(__filename) + 1]; // get project name from command, e.g. node ../patch-config.js projectName
if (!projectName) {
  console.error("Please provide a project name.");
  process.exit(1);
}

const contractId = fs
  .readFileSync(`./${projectName}/neardev/dev-account`)
  .toString();

const path = `./test/${projectName}/config.ts`;

fs.readFile(path, "utf-8", function (err, data) {
  if (err) throw err;

  data = data.replace(
    /.*export const contractId.*/gim,
    `export const contractId = "${contractId}";`
  );

  fs.writeFile(path, data, "utf-8", function (err) {
    if (err) throw err;
    console.log("✅ Patched config for", projectName, "contract");
  });
});
