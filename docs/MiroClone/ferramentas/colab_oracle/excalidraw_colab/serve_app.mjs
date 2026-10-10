// Servidor estático mínimo do build do app (SPA): só 127.0.0.1. Uso: node serve_app.mjs <build_dir> <porta>
import http from "http"; import fs from "fs"; import path from "path";
const [dir, port] = [path.resolve(process.argv[2]), Number(process.argv[3] || 5101)];
const T = { ".html": "text/html", ".js": "text/javascript", ".mjs": "text/javascript", ".css": "text/css", ".json": "application/json",
  ".svg": "image/svg+xml", ".png": "image/png", ".ico": "image/x-icon", ".woff2": "font/woff2", ".wasm": "application/wasm", ".webmanifest": "application/manifest+json", ".map": "application/json", ".txt": "text/plain", ".xml": "text/xml" };
http.createServer((req, res) => {
  let p = decodeURIComponent(new URL(req.url, "http://x").pathname);
  let f = path.join(dir, p); if (!f.startsWith(dir)) { res.writeHead(403).end(); return; }
  if (!fs.existsSync(f) || fs.statSync(f).isDirectory()) f = fs.existsSync(path.join(f, "index.html")) ? path.join(f, "index.html") : path.join(dir, "index.html");
  res.writeHead(200, { "content-type": T[path.extname(f)] || "application/octet-stream", "cache-control": "no-store" });
  fs.createReadStream(f).pipe(res);
}).listen(port, "127.0.0.1", () => console.log(`app em http://127.0.0.1:${port}/`));
