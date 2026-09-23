# Amostrador de pilhas por gdb — o perfilador desta máquina quando o `perf` não está disponível.
#
# ⚠️ `perf_event_paranoid = 2` recusa `perf`/`samply` sem root. Este script interrompe o programa a
# cada ~8 ms (gdb.post_event + "interrupt"), conta as pilhas de TODAS as threads OCUPADAS (as que
# estão paradas num futex/condvar/sleep são descartadas) e escreve três tabelas: inclusivo por
# função, folha, e folha <- pai <- avô.
#
# USO (o binário tem de ter símbolos; o release do repo faz `strip = "symbols"`):
#
#   CARGO_TARGET_DIR=/tmp/alvo-sym CARGO_PROFILE_RELEASE_STRIP=none \
#     CARGO_PROFILE_RELEASE_DEBUG=line-tables-only \
#     bash scripts/ph2d-run.sh cargo build -p ph2d-tool-painter --release --example mede_a_pilha
#   AMOSTRA_SAIDA=/tmp/perfil.txt gdb -batch -x docs/Painter/ferramentas/amostra_gdb.py \
#     --args /tmp/alvo-sym/release/examples/mede_a_pilha perfil
#
# ⛔⛔ DUAS MENTIRAS MEDIDAS (doc 43 §4):
#   1. Sob ptrace, CRIAR uma thread é caríssimo: um programa que abre threads por tarefa aparece com
#      metade do tempo em `pthread_create`, e a cura que isso pede mediu ganho ZERO no build real.
#      ⇒ este script LOCALIZA; quem DECIDE é o A/B alternado de dois binários na mesma ronda.
#   2. Ele mede o binário que lhe dão: um build de `cfg(test)` não é o produto.
import gdb, threading, time, collections, os

SAIDA = os.environ.get("AMOSTRA_SAIDA", "/tmp/perfil.txt")
PERIODO_S = float(os.environ.get("AMOSTRA_PERIODO_S", "0.008"))

gdb.execute("set pagination off")
gdb.execute("set confirm off")
gdb.execute("set print thread-events off")
gdb.execute("set print inferior-events off")

inclus = collections.Counter()
folhas = collections.Counter()
pares = collections.Counter()
amostras = 0


def tick():
    time.sleep(PERIODO_S)
    try:
        gdb.post_event(lambda: gdb.execute("interrupt"))
    except Exception:
        pass


def vivo():
    inf = gdb.selected_inferior()
    return inf is not None and inf.pid != 0


def nome(f):
    n = f.name()
    if not n:
        try:
            txt = gdb.execute(f"info symbol {f.pc()}", to_string=True)
            n = txt.split(" in section")[0].split(" + ")[0].strip()
        except gdb.error:
            n = "?"
    return n


OCIOSO = ("futex", "Condvar::wait", "LockLatch", "wait_until", "sleep", "epoll")

threading.Thread(target=tick, daemon=True).start()
try:
    gdb.execute("run")
except gdb.error:
    pass

while vivo():
    try:
        for th in gdb.selected_inferior().threads():
            th.switch()
            f = gdb.newest_frame()
            nomes = []
            while f is not None and len(nomes) < 50:
                nomes.append(nome(f))
                f = f.older()
            if not nomes or any(any(o in n for o in OCIOSO) for n in nomes[:6]):
                continue
            amostras += 1
            folhas[nomes[0]] += 1
            pares[(nomes[0], nomes[1] if len(nomes) > 1 else "-", nomes[2] if len(nomes) > 2 else "-")] += 1
            for n in set(nomes):
                inclus[n] += 1
    except gdb.error:
        pass
    threading.Thread(target=tick, daemon=True).start()
    try:
        gdb.execute("continue")
    except gdb.error:
        break

with open(SAIDA, "w") as out:
    out.write(f"amostras (threads ocupadas) {amostras}\n\n== INCLUSIVO ==\n")
    for n, c in inclus.most_common(90):
        out.write(f"{100.0 * c / max(amostras, 1):6.1f}%  {n[:120]}\n")
    out.write("\n== FOLHA ==\n")
    for n, c in folhas.most_common(40):
        out.write(f"{100.0 * c / max(amostras, 1):6.1f}%  {n[:120]}\n")
    out.write("\n== FOLHA <- PAI <- AVO ==\n")
    for (a, b, c), k in pares.most_common(50):
        out.write(f"{100.0 * k / max(amostras, 1):6.1f}%  {a[:40]} <- {b[:50]} <- {c[:50]}\n")
