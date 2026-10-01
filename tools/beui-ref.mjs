// Screenshots beui.dev component previews and dumps their computed styles, as the reference for a
// pixel-level port. node tools/beui-ref.mjs <out-dir> <component-path>...
// Example: node tools/beui-ref.mjs /tmp/ref motion/select agents/todo-list
import { mkdirSync, writeFileSync } from "node:fs";
import puppeteer from "puppeteer-core";

const [, , out, ...paths] = process.argv;
mkdirSync(out, { recursive: true });
const browser = await puppeteer.launch({ executablePath: "/usr/bin/chromium", args: ["--no-sandbox"] });
try {
  for (const path of paths) {
    const name = path.replace("/", "-");
    for (const theme of ["dark", "light"]) {
      const page = await browser.newPage();
      await page.setViewport({ width: 1280, height: 900, deviceScaleFactor: 2 });
      await page.emulateMediaFeatures([{ name: "prefers-color-scheme", value: theme }]);
      await page.evaluateOnNewDocument((t) => localStorage.setItem("theme", t), theme);
      await page.goto(`https://beui.dev/components/${path}`, { waitUntil: "networkidle2", timeout: 60000 });
      await new Promise((r) => setTimeout(r, 1500));
      // The first preview frame on the page holds the live demo.
      const preview = (await page.$("[data-preview]")) ?? (await page.$("main [class*=preview]")) ?? (await page.$("main"));
      await preview.screenshot({ path: `${out}/${name}-${theme}.png` });
      if (theme === "dark") {
        const styles = await page.evaluate((root) => {
          const keep = ["width", "height", "padding", "margin", "gap", "border-radius", "background-color", "color",
            "font-size", "line-height", "font-weight", "letter-spacing", "border", "box-shadow", "opacity"];
          return [...root.querySelectorAll("*")].slice(0, 120).map((el) => {
            const cs = getComputedStyle(el);
            const r = el.getBoundingClientRect();
            return {
              tag: el.tagName.toLowerCase(),
              text: (el.childNodes[0]?.nodeType === 3 ? el.childNodes[0].textContent.trim() : "").slice(0, 40),
              box: [Math.round(r.x), Math.round(r.y), Math.round(r.width), Math.round(r.height)],
              ...Object.fromEntries(keep.map((k) => [k, cs.getPropertyValue(k)])),
            };
          }).filter((s) => s.box[2] > 0 && s.box[3] > 0);
        }, preview);
        writeFileSync(`${out}/${name}-styles.json`, JSON.stringify(styles, null, 1));
      }
      await page.close();
    }
  }
} finally {
  await browser.close();
}
