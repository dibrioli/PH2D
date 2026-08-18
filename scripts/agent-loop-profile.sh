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

turnos_tool = chamadas = 0
por_sessao = []
cargo = collections.Counter()
edits = collections.Counter()

for fp in paths:
    nt = 0
    for line in open(fp, encoding='utf-8', errors='replace'):
        try: r = json.loads(line)
        except Exception: continue
        if r.get('type') != 'assistant': continue
        c0 = (r.get('message', {}) or {}).get('content')
        if not isinstance(c0, list): continue
        nt += 1
        usos = [c for c in c0 if isinstance(c, dict) and c.get('type') == 'tool_use']
        if usos:
            turnos_tool += 1
            chamadas += len(usos)
        for c in usos:
            nome = c.get('name'); inp = c.get('input', {}) or {}
            if nome in ('Edit', 'Write', 'NotebookEdit'):
                edits['ferramenta'] += 1
            elif nome == 'Bash':
                cmd = inp.get('command', '')
                if re.search(r"open\([^)]*['\"][wa]['\"]|\.write\(|write_text\(", cmd) \
                   or re.search(r'\bsed -i\b|\bperl -i\b', cmd):
                    edits['script'] += 1
                if re.search(r'cargo (test|nextest)\b', cmd):   cargo['test'] += 1
                elif re.search(r'cargo check\b', cmd):          cargo['check'] += 1
    if nt: por_sessao.append(nt)

def linha(rot, valor, alvo, ok, nota=''):
    marca = '✓' if ok else '✗'
    print(f"  {marca} {rot:<34} {valor:>14}   alvo: {alvo}{nota}")

print(f"\nPERFIL DO LOOP DO AGENTE — {len(paths)} sessao(oes)"
      f"{' (corpus inteiro)' if N=='all' else ' mais recentes'}")
print("─" * 78)

par = chamadas / turnos_tool if turnos_tool else 0
linha("paralelismo de ferramenta", f"{par:.2f}/turno", ">= 1,5", par >= 1.5,
      "  (baseline 2026-08-18: 1,00)")

med = st.median(por_sessao) if por_sessao else 0
linha("turnos do assistente/sessao", f"{med:.0f}", "menor e' melhor", True,
      "  (baseline: 991)")

t, ck = cargo['test'], cargo['check']
raz = t / ck if ck else float('inf')
linha("cargo test : cargo check", f"{t} : {ck}", "<= 1,0", raz <= 1.0,
      f"  razao {raz:.1f}x (baseline: 4,3x)")

ef, es = edits['ferramenta'], edits['script']
pct = 100 * ef / (ef + es) if (ef + es) else 0
linha("edicoes pela ferramenta Edit", f"{pct:.0f}%", ">= 80%", pct >= 80,
      f"  ({es} por script; baseline: 48%)")

print("─" * 78)
print("  As leis moram no CLAUDE.md §2 (sempre carregado); a DIRETIVA_IMPLEMENTACAO aponta pra la'.")
print("  ⚠️ Rode com poucas sessoes para ver o HABITO recente; 'all' e' o baseline historico.\n")
PY
