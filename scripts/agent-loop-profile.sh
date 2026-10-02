#!/usr/bin/env bash
# agent-loop-profile.sh — measure HOW the LLM works, from the session transcripts.
#
# WHY this exists: on 2026-08-18 the question "why does implementing a feature take
# so long?" was answered by measuring 101 sessions instead of guessing. The machine
# was NOT the bottleneck (32 cores, mold, target on tmpfs; `cargo check -p` = 0.2-6.2 s).
# The wall-clock split was ~1.9 h of cargo wait against 2.4-4 h of token GENERATION,
# which is serial and hardware-independent. Four habits drove it:
#
#   1. ZERO tool parallelism  — 279.566 turns, 1,00 call each (8 turns in 279k used two).
#      Every turn is a round trip and the median session has ~991 of them.
#   2. The speed rule INVERTED — `cargo test -p` 61.225 calls vs `cargo check -p` 14.130,
#      i.e. the 2-20x more expensive command ran 4,3x MORE often (CLAUDE.md §2 says the
#      opposite). Measured: check 0,2/6,2/2,0 s vs test 0,4/17,7/38,8 s on three crates.
#   3. Edits by hand-written SCRIPT — 46.772 of them, only 48% of edits going through
#      the Edit tool, costing ~93k tokens of generation per session. The failure mode is
#      the expensive one: `str.replace()` that does not match is a SILENT no-op, and the
#      script prints success; Edit fails LOUD when `old_string` does not match.
#   4. Re-reads caused by compaction — 29 of 71 sessions re-read the same doc >=3x.
#
# A rule without an instrument is a note that ages. This script is the instrument:
# it re-measures the four numbers so the next session can tell whether the discipline
# landed, instead of asserting that it did.
#
# USAGE
#   bash scripts/agent-loop-profile.sh          # the last 20 sessions (recent habit)
#   bash scripts/agent-loop-profile.sh 5        # the last 5
#   bash scripts/agent-loop-profile.sh all      # the whole corpus (the 2026-08-18 baseline)
#
# It reads only ~/.claude/projects/ and writes nothing. Safe to run at any time.

set -uo pipefail
N="${1:-20}"

python3 - "$N" <<'PY'
import json, glob, sys, os, collections, re, statistics as st

N = sys.argv[1]
paths = sorted(glob.glob(os.path.expanduser(
    '~/.claude/projects/-home-enio-Documentos-Projetos-PH2D*/*.jsonl')),
    key=os.path.getmtime, reverse=True)
if N != 'all':
    try: paths = paths[:int(N)]
    except ValueError: paths = paths[:20]
if not paths:
    print("nenhum transcript encontrado em ~/.claude/projects/ — nada a medir"); sys.exit(0)

# ⛔ 2026-10-02: a 1.ª versão contava cada LINHA `assistant` do transcript como um turno — e o
# transcript grava cada BLOCO da resposta (pensamento · texto · cada chamada) numa linha própria.
# Resultado: o paralelismo lia SEMPRE 1,00 (uma chamada por linha, por construção), os turnos
# vinham inflados, e a linha dos turnos marcava ✓ fosse qual fosse o número (4 466 contra o
# baseline de 991 lia-se como aprovado). Hoje a unidade é a RESPOSTA (o `message.id`), e cada
# régua compara com o alvo. ⚠️ Os baselines de 2026-08-18 foram medidos pela régua antiga.
#
# ⭐ E duas réguas novas, porque são o CUSTO: 82 % da conta é RELER o contexto a cada passo.
#   · contexto relido por passo (média de cache-read + cache-write + input por resposta)
#   · contexto no INÍCIO da sessão (o 1.º passo: prompt de sistema + ferramentas + CLAUDE.md + memória)
respostas = {}            # message.id -> conjunto de tool_use ids
ctx_passo = []            # contexto de cada resposta
ctx_inicio = []           # contexto do 1.º passo de cada sessão
por_sessao = []
cargo = collections.Counter()
edits = collections.Counter()

for fp in paths:
    vistos = {}
    primeiro = None
    for line in open(fp, encoding='utf-8', errors='replace'):
        try: r = json.loads(line)
        except Exception: continue
        if r.get('type') != 'assistant': continue
        m = r.get('message', {}) or {}
        mid = m.get('id') or r.get('uuid')
        u = m.get('usage') or {}
        ctx = sum((u.get(k) or 0) for k in ('input_tokens', 'cache_read_input_tokens',
                                           'cache_creation_input_tokens'))
        if mid not in vistos:
            vistos[mid] = set()
            if ctx: ctx_passo.append(ctx)
            if primeiro is None and ctx: primeiro = ctx
        c0 = m.get('content')
        if not isinstance(c0, list): continue
        for c in c0:
            if not (isinstance(c, dict) and c.get('type') == 'tool_use'): continue
            if c.get('id') in vistos[mid]: continue
            vistos[mid].add(c.get('id'))
            nome = c.get('name'); inp = c.get('input', {}) or {}
            if nome in ('Edit', 'Write', 'NotebookEdit'):
                edits['ferramenta'] += 1
            elif nome == 'Bash':
                cmd = inp.get('command', '')
                if re.search(r"open\([^)]*['\"][wa]['\"]|\.write\(|write_text\(", cmd) \
                   or re.search(r'\bsed -i\b|\bperl -i\b', cmd):
                    edits['script'] += 1
                if re.search(r'cargo (\+\S+ )?(test|nextest)\b|cargo-test-narrow|nextest-impacted', cmd):
                    cargo['test'] += 1
                elif re.search(r'cargo (\+\S+ )?check\b|cargo-check-narrow', cmd):
                    cargo['check'] += 1
    if vistos: por_sessao.append(len(vistos))
    if primeiro: ctx_inicio.append(primeiro)
    respostas.update({(fp, k): v for k, v in vistos.items()})

def linha(rot, valor, alvo, ok, nota=''):
    marca = '✓' if ok else '✗'
    print(f"  {marca} {rot:<34} {valor:>14}   alvo: {alvo}{nota}")

print(f"\nPERFIL DO LOOP DO AGENTE — {len(paths)} sessao(oes)"
      f"{' (corpus inteiro)' if N=='all' else ' mais recentes'}")
print("─" * 78)

com = [len(v) for v in respostas.values() if v]
par = sum(com) / len(com) if com else 0
multi = 100 * sum(1 for n in com if n > 1) / len(com) if com else 0
linha("paralelismo de ferramenta", f"{par:.2f}/passo", ">= 1,5", par >= 1.5,
      f"  ({multi:.0f}% dos passos com 2+ chamadas)")

med = st.median(por_sessao) if por_sessao else 0
linha("respostas por sessao (mediana)", f"{med:.0f}", "<= 800", med <= 800,
      "  (uma janela nova por onda de trabalho)")

t, ck = cargo['test'], cargo['check']
raz = t / ck if ck else float('inf')
linha("cargo test : cargo check", f"{t} : {ck}", "<= 1,0", raz <= 1.0,
      f"  razao {raz:.1f}x (baseline: 4,3x)")

ef, es = edits['ferramenta'], edits['script']
pct = 100 * ef / (ef + es) if (ef + es) else 0
linha("edicoes pela ferramenta Edit", f"{pct:.0f}%", ">= 80%", pct >= 80,
      f"  ({es} por script; baseline: 48%)")

mc = st.mean(ctx_passo) if ctx_passo else 0
linha("contexto relido por passo (media)", f"{mc/1000:.0f} mil", "<= 250 mil", mc <= 250_000,
      "  (set/2026: 606 mil — 82% do custo)")
mi = st.median(ctx_inicio) if ctx_inicio else 0
linha("contexto no inicio da sessao", f"{mi/1000:.0f} mil", "<= 80 mil", mi <= 80_000,
      "  (02/10: 380 mil, CLAUDE.md a 710 KB)")

print("─" * 78)
print("  As leis moram no CLAUDE.md §2 (sempre carregado); a DIRETIVA_IMPLEMENTACAO aponta pra la'.")
print("  ⚠️ Rode com poucas sessoes para ver o HABITO recente; 'all' e' o baseline historico.\n")
PY
