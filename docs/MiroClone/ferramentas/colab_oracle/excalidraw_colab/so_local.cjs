// Pré-carregado no servidor de salas (node -r): força todo listen() a 127.0.0.1, sem tocar no fonte dele.
const net = require("net");
const orig = net.Server.prototype.listen;
net.Server.prototype.listen = function (...a) {
  if (typeof a[0] === "number" || typeof a[0] === "string" && /^\d+$/.test(a[0])) {
    const port = Number(a[0]); const cb = a.find((x) => typeof x === "function");
    return orig.call(this, { port, host: "127.0.0.1" }, cb);
  }
  if (a[0] && typeof a[0] === "object" && "port" in a[0]) a[0] = { ...a[0], host: "127.0.0.1" };
  return orig.apply(this, a);
};
