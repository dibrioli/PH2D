"""Sonda: o LSP do Godot (editor headless) — que campos traz o publishDiagnostics? Uso: godot_lsp.py <proj> <porta> <saida.json> f1.gd ..."""
import json, socket, sys, time, os, pathlib
proj, porta, saida, *fs = sys.argv[1:]
porta = int(porta)
s = None
for _ in range(60):
    try:
        s = socket.create_connection(("127.0.0.1", porta), timeout=2); break
    except OSError:
        time.sleep(0.5)
if s is None: print("sem ligacao"); sys.exit(2)
s.settimeout(1.0)
buf = b""
def env(m):
    b = json.dumps(m).encode(); s.sendall(b"Content-Length: %d\r\n\r\n" % len(b) + b)
def ler(ate):
    global buf
    msgs = []
    while time.time() < ate:
        try:
            d = s.recv(65536)
            if not d: break
            buf += d
        except socket.timeout:
            pass
        while b"\r\n\r\n" in buf:
            h, r = buf.split(b"\r\n\r\n", 1)
            n = int([l for l in h.split(b"\r\n") if l.lower().startswith(b"content-length")][0].split(b":")[1])
            if len(r) < n: break
            msgs.append(json.loads(r[:n])); buf = r[n:]
    return msgs
root = pathlib.Path(proj).resolve().as_uri()
env({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"processId": os.getpid(), "rootUri": root,
     "capabilities": {"textDocument": {"publishDiagnostics": {"codeDescriptionSupport": True, "relatedInformation": True, "tagSupport": {"valueSet": [1, 2]}}, "hover": {"contentFormat": ["markdown", "plaintext"]}}}}})
todas = ler(time.time() + 5)
env({"jsonrpc": "2.0", "method": "initialized", "params": {}})
for f in fs:
    p = pathlib.Path(proj, f).resolve()
    env({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {"textDocument": {"uri": p.as_uri(), "languageId": "gdscript", "version": 1, "text": p.read_text()}}})
todas += ler(time.time() + 8)
json.dump(todas, open(saida, "w"), indent=1, ensure_ascii=False)
for m in todas:
    if m.get("method") == "textDocument/publishDiagnostics":
        for d in m["params"]["diagnostics"]:
            print(os.path.basename(m["params"]["uri"]), json.dumps(d, ensure_ascii=False))
    elif "id" in m and m.get("id") == 1:
        print("caps.keys:", sorted(m.get("result", {}).get("capabilities", {}).keys()))
