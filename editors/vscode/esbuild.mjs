import { build, context } from "esbuild";

const watch = process.argv.includes("--watch");
const minify = process.argv.includes("--minify");

const options = {
  entryPoints: ["src/extension.ts"],
  bundle: true,
  outfile: "out/extension.js",
  platform: "node",
  target: "node18",
  format: "cjs",
  // `vscode` is provided by the extension host at runtime; don't bundle it.
  external: ["vscode"],
  sourcemap: !minify,
  minify,
  logLevel: "info",
};

if (watch) {
  const ctx = await context(options);
  await ctx.watch();
  console.log("esbuild: watching…");
} else {
  await build(options);
}
