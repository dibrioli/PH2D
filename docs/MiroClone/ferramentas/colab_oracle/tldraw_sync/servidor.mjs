// Servidor mínimo do oráculo de colaboração tldraw, só API PÚBLICA documentada:
//  - 127.0.0.1:5201  HTTP estático (www/: index.html + bundle esbuild + tldraw.css)
//  - 127.0.0.1:5202  WebSocket /connect/<sala>?sessionId=…  → TLSocketRoom.handleSocketConnect (ws 8)
//                    GET /snapshot/<sala> → room.getCurrentSnapshot() (inclui lastChangedClock por registo)
//                    GET /stats/<sala>    → contagem de mensagens recebidas/enviadas pelo servidor
// Uma TLSocketRoom por sala, em memória. Sai sozinho ao fim de PRAZO_MS (padrão 45 min).
// Uso: ORACLE_HOME=~/Apps/tldraw-sync-oracle node servidor.mjs   (imprime «PRONTO» quando escuta)
import { createRequire } from "node:module";
import { createServer } from "node:http";
import fs from "node:fs";
import path from "node:path";

const ORACLE_HOME = process.env.ORACLE_HOME || path.join(process.env.HOME, "Apps/tldraw-sync-oracle");
const require = createRequire(path.join(ORACLE_HOME, "package.json"));
const { TLSocketRoom } = require("@tldraw/sync-core");
const { createTLSchema, defaultShapeSchemas, defaultBindingSchemas } = require("@tldraw/tlschema");
const { WebSocketServer } = require("ws");
const WWW = path.join(ORACLE_HOME, "www");
const PRAZO_MS = Number(process.env.PRAZO_MS || 45 * 60 * 1000);
setTimeout(() => { console.error("servidor: prazo esgotado, a sair"); process.exit(0); }, PRAZO_MS).unref();

const schema = createTLSchema({ shapes: defaultShapeSchemas, bindings: defaultBindingSchemas });
const rooms = new Map();
const stats = new Map();
const room = (id) => {
  if (!rooms.has(id)) {
    const st = { recv: 0, recvBytes: 0, sent: 0, sentBytes: 0 };
    stats.set(id, st);
    rooms.set(id, new TLSocketRoom({
      schema,
      onAfterReceiveMessage: ({ stringified }) => { st.recv++; st.recvBytes += Buffer.byteLength(stringified); },
      onBeforeSendMessage: ({ stringified }) => { st.sent++; st.sentBytes += Buffer.byteLength(stringified); },
    }));
  }
  return rooms.get(id);
};

const MIME = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".woff2": "font/woff2", ".woff": "font/woff", ".ttf": "font/ttf", ".svg": "image/svg+xml", ".png": "image/png", ".json": "application/json" };
const web = createServer((req, res) => {
  const p = path.normalize(decodeURIComponent(new URL(req.url, "http://x").pathname));
  const file = path.join(WWW, p === "/" ? "index.html" : p);
  if (!file.startsWith(WWW) || !fs.existsSync(file) || fs.statSync(file).isDirectory()) { res.writeHead(404).end(); return; }
  res.writeHead(200, { "content-type": MIME[path.extname(file)] || "application/octet-stream" });
  fs.createReadStream(file).pipe(res);
});

const api = createServer((req, res) => {
  const u = new URL(req.url, "http://x");
  const m = u.pathname.match(/^\/(snapshot|stats)\/(.+)$/);
  if (!m || !rooms.has(decodeURIComponent(m[2]))) { res.writeHead(404).end(); return; }
  const id = decodeURIComponent(m[2]);
  const body = m[1] === "snapshot" ? rooms.get(id).getCurrentSnapshot() : stats.get(id);
  res.writeHead(200, { "content-type": "application/json" }).end(JSON.stringify(body));
});
const wss = new WebSocketServer({ noServer: true });
let anon = 0;
api.on("upgrade", (req, socket, head) => {
  const u = new URL(req.url, "http://x");
  const m = u.pathname.match(/^\/connect\/(.+)$/);
  if (!m) { socket.destroy(); return; }
  const sessionId = u.searchParams.get("sessionId") || `anon-${++anon}`;
  wss.handleUpgrade(req, socket, head, (ws) => room(decodeURIComponent(m[1])).handleSocketConnect({ sessionId, socket: ws }));
});

await Promise.all([
  new Promise((r) => web.listen(5201, "127.0.0.1", r)),
  new Promise((r) => api.listen(5202, "127.0.0.1", r)),
]);
console.log(`PRONTO pid=${process.pid} web=5201 ws=5202`);
