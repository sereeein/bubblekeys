import { chromium } from "playwright";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const __filename = fileURLToPath(import.meta.url);
const __dirname  = dirname(__filename);
const REPO       = resolve(__dirname, "..");
const ICONS_DIR  = resolve(REPO, "src-tauri/icons");
const MOCKUP_DIR = resolve(REPO, "scripts/icon-mockup");

async function shoot(htmlPath: string, outPath: string) {
  const browser = await chromium.launch();
  const ctx = await browser.newContext({
    viewport: { width: 1024, height: 1024 },
    deviceScaleFactor: 1,
  });
  const page = await ctx.newPage();
  await page.goto("file://" + htmlPath);
  // Force layout settle.
  await page.waitForLoadState("networkidle");
  await page.screenshot({
    path: outPath,
    omitBackground: true,
    clip: { x: 0, y: 0, width: 1024, height: 1024 },
  });
  await browser.close();
  console.log(`wrote ${outPath}`);
}

await shoot(
  resolve(MOCKUP_DIR, "icon-master.html"),
  resolve(ICONS_DIR, "icon-1024.png"),
);
await shoot(
  resolve(MOCKUP_DIR, "icon-master-simplified.html"),
  resolve(ICONS_DIR, "icon-1024-simplified.png"),
);
