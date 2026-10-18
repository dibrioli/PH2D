# Os processos da tabela INTEIRA do nvidia-smi (C e G) que não são a área de trabalho nem ESTA medição.
# Esta medição = tudo o que descende do um_processo.sh (o kwin virtual, o Xwayland dele, o app). A área de trabalho = o kwin do wayland-0, o Xwayland :0, plasmashell, portal, VSCode.
import os, re, subprocess
LUZ = "/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D"
def ler(pid, f):
    try:
        return open(f"/proc/{pid}/{f}", "rb").read().decode(errors="replace")
    except OSError:
        return ""
def cwd(pid):
    try: return os.readlink(f"/proc/{pid}/cwd")
    except OSError: return ""
def ppid(pid):
    m = re.search(r"^PPid:\s+(\d+)", ler(pid, "status"), re.M)
    return int(m.group(1)) if m else 0
saida = subprocess.run(["nvidia-smi"], capture_output=True, text=True).stdout
pids = []
dentro = False
for l in saida.splitlines():
    if "Processes:" in l: dentro = True
    c = l.split()
    if dentro and len(c) > 5 and c[1].isdigit() and c[4].isdigit():
        pids.append(int(c[4]))
def desta_medicao(p):
    # Descende do roteiro desta rodada (o kwin virtual nasce antes do app e sai depois dele: 2 falsos sujos).
    while p > 1:
        if "/ferramentas/regua_renasce/um_processo.sh" in ler(p, "cmdline").replace("\0", " "):
            return True
        p = ppid(p)
    return False
for p in pids:
    nome, cmd = ler(p, "comm").strip(), ler(p, "cmdline").replace("\0", " ")
    if desta_medicao(p): continue
    if nome == "kwin_wayland" and "--socket wayland-0" in cmd: continue
    if nome == "Xwayland" and cmd.startswith("/usr/bin/Xwayland :0 "): continue
    if nome in ("plasmashell", "code") or nome.startswith("xdg-desktop-por"): continue
    print(f"{p}:{nome}:{cwd(p).rsplit('/', 1)[-1]}")
