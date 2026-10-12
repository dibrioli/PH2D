#!/usr/bin/env python3
"""W18 — o tubo do CDP sem `unsafe`: o navegador lançado por `sh -c 'exec … 3<&0 4>&1 </dev/null >/dev/null'`
responde no tubo (stdin/stdout do sh)? E quando quem o lançou MORRE (o tubo fecha), o navegador sai sozinho?
Uso: sonda_tubo.py <exe>  — imprime JSON.
"""
import json, os, subprocess, sys, tempfile, time, signal
exe = sys.argv[1]
prof = tempfile.mkdtemp(prefix="ph2d-cdp-")
cmd = f"exec '{exe}' --headless --remote-debugging-pipe --user-data-dir='{prof}' --no-first-run about:blank 3<&0 4>&1 </dev/null >/dev/null 2>/dev/null"
# Um intermediário que segura o tubo e morre de repente (o «PH2D»).
mid = subprocess.Popen([sys.executable, "-c", f"""
import subprocess, sys, time, json
p = subprocess.Popen(["sh", "-c", {cmd!r}], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
p.stdin.write(json.dumps({{"id": 1, "method": "Browser.getVersion"}}).encode() + b"\\0"); p.stdin.flush()
buf = b""
while b"\\0" not in buf:
    buf += p.stdout.read1(65536)
print(json.dumps({{"pid": p.pid, "resposta": json.loads(buf.split(b"\\0")[0])}}), flush=True)
time.sleep(60)
"""], stdout=subprocess.PIPE)
linha = mid.stdout.readline().decode()
r = json.loads(linha)
pid = r["pid"]
vivo = lambda: os.path.exists(f"/proc/{pid}") and open(f"/proc/{pid}/stat").read().split(") ")[1][0] != "Z"
antes = vivo()
mid.kill(); mid.wait()
t0 = time.time()
while vivo() and time.time() - t0 < 10:
    time.sleep(0.05)
depois = vivo()
print(json.dumps({"responde_no_tubo": r["resposta"].get("result", {}).get("product"), "vivo_antes": antes,
                  "sai_sozinho_quando_o_pai_morre": not depois, "segundos_ate_sair": round(time.time() - t0, 2)}))
if depois:
    os.kill(pid, signal.SIGKILL)
