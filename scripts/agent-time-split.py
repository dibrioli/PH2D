#!/usr/bin/env python3
"""agent-time-split.py — PARA ONDE VAI o tempo das janelas principais, lido das transcrições.

O irmão do `agent-loop-profile.sh` (que mede HÁBITOS: paralelismo, test:check, contexto). Este mede
o RELÓGIO: o tempo ativo de cada janela partido em gerar (pensar × escrever), ferramentas por família
e a espera de FUNDO (ajudantes e comandos em segundo plano), atribuída ao que estava a correr.

Medido com ele em 17/10/2026 (14/10 16h → 17/10, 7 linhas, 103 h ativas): espera de fundo 61 %
(ajudantes gerais 29 h, verificador 9 h, comandos longos 9 h) · gerar 23 % (56 % disso é PENSAR) ·
testes ~8 %. ⇒ cortar espera de ferramenta mexe pouco; a memória
`project_agent_time_split_2026_10_17` tem o resto e as recusas medidas.

USO
  python3 scripts/agent-time-split.py                      # últimas 72 h
  python3 scripts/agent-time-split.py --desde 2026-10-14T16:00
  python3 scripts/agent-time-split.py --horas 24

Só lê ~/.claude/projects/; não escreve nada. A espera pelo DONO (pedido novo) não conta.
"""
import argparse, collections, datetime as dt, glob, json, os, re, time

ap = argparse.ArgumentParser()
ap.add_argument('--desde', help='ISO local, ex. 2026-10-14T16:00 (fuso -03:00)')
ap.add_argument('--horas', type=float, default=72)
a = ap.parse_args()
A = dt.datetime.fromisoformat(a.desde + '-03:00').timestamp() if a.desde else time.time() - a.horas * 3600

def ts(s): return dt.datetime.fromisoformat(s.replace('Z', '+00:00')).timestamp()

def fam(c):
    if 'nextest-impacted' in c: return 'suíte'
    if 'ship.sh' in c: return 'ship'
    if re.search(r'cargo (\+\S+ )?clippy', c): return 'clippy'
    if '--no-run' in c: return 'compilar testes (--no-run)'
    if re.search(r'cargo (\+\S+ )?(test|nextest)|cargo-test-narrow', c):
        return 'testes' + (' pesados (--ignored/--release)' if re.search(r'ignored|--release', c) else '')
    if re.search(r'cargo-check-narrow|cargo (\+\S+ )?check', c): return 'check'
    if re.search(r'target/[^ ]*/deps/|deps/it-', c): return 'medição (binário à mão)'
    if re.search(r'cargo (\+\S+ )?(build|run)|fotografa|kwin|import -window|xdotool', c): return 'build/smoke/foto'
    if re.search(r'(^|\s)(until|while) |sleep ', c): return 'espera em laço'
    if re.search(r'^\s*(cd \S+ && )?git ', c): return 'git'
    if 'python3' in c: return 'python'
    return 'bash outro'

tot = collections.Counter(); gen = collections.Counter(); fundo = collections.Counter()
por_linha = collections.defaultdict(collections.Counter)
for p in glob.glob(os.path.expanduser('~/.claude/projects/-home-enio-Documentos-Projetos-PH2D*/*.jsonl')):
    if os.path.getmtime(p) < A: continue
    ev = []
    for line in open(p, errors='ignore'):
        try: d = json.loads(line)
        except Exception: continue
        if d.get('timestamp'): ev.append((ts(d['timestamp']), d))
    ev.sort(key=lambda x: x[0])
    linha = next((m.group(1) for _, d in ev if (m := re.search(r'Worktrees/line-([^/]+)', d.get('cwd') or ''))), 'main')
    use = {}; tarefa = {}; prev = None
    for t, d in ev:
        dentro = t >= A
        m = d.get('message') or {}; c = m.get('content'); ty = d.get('type')
        if ty == 'assistant' and isinstance(c, list):
            for b in c:
                if prev and dentro:
                    g = min(t - prev, 1800)
                    k = 'pensar' if b.get('type') == 'thinking' else 'escrever'
                    gen[k] += g; tot['gerar'] += g; por_linha[linha]['gerar'] += g
                if b.get('type') == 'tool_use': use[b['id']] = (t, b.get('name'), b.get('input') or {})
                prev = t
        elif ty == 'user':
            s = json.dumps(c, ensure_ascii=False)
            if isinstance(c, list) and any(isinstance(b, dict) and b.get('type') == 'tool_result' for b in c):
                for b in c:
                    if not (isinstance(b, dict) and b.get('type') == 'tool_result'): continue
                    u = use.pop(b.get('tool_use_id'), None)
                    if not u: continue
                    t0, nm, i = u; txt = json.dumps(b.get('content'), ensure_ascii=False)
                    mm = re.search(r'agentId: ([a-z0-9]+)', txt) or re.search(r'ID: ([a-z0-9]+)', txt)
                    if nm == 'Agent' and mm: tarefa[mm.group(1)] = 'ajudante ' + (i.get('subagent_type') or 'general-purpose')
                    elif nm == 'Bash' and i.get('run_in_background') and mm: tarefa[mm.group(1)] = 'comando: ' + fam(i.get('command', ''))
                    if dentro and nm == 'Bash' and not i.get('run_in_background'):
                        k = fam(i.get('command', '')); tot[k] += t - t0; por_linha[linha]['ferramentas'] += t - t0
                prev = t
            elif 'task-notification' in s:
                mm = re.search(r'<task-id>([a-z0-9]+)</task-id>', s)
                if prev and dentro:
                    g = min(t - prev, 3600); k = tarefa.get(mm.group(1), '?') if mm else '?'
                    fundo[k] += g; tot['espera de fundo'] += g; por_linha[linha]['fundo'] += g
                prev = t
            else:
                prev = t  # pedido do dono: a espera por ele não conta

ativo = sum(tot.values()) or 1
h = lambda v: f'{v / 3600:6.2f} h'
print(f'desde {dt.datetime.fromtimestamp(A):%d/%m %H:%M} · tempo ATIVO das janelas principais: {ativo / 3600:.1f} h\n')
for k, v in tot.most_common(14): print(f'  {k:36s} {h(v)}  {100 * v / ativo:5.1f}%')
g = sum(gen.values()) or 1
print(f'\n  gerar = pensar {100 * gen["pensar"] / g:.0f}% · escrever {100 * gen["escrever"] / g:.0f}%')
print('\n  espera de fundo, pelo que corria:')
for k, v in fundo.most_common(8): print(f'    {k:34s} {h(v)}')
print('\n  por linha (h): gerar · ferramentas · fundo')
for l, x in sorted(por_linha.items(), key=lambda y: -sum(y[1].values())):
    print(f"    {l:22s} {x['gerar'] / 3600:5.1f} · {x['ferramentas'] / 3600:5.1f} · {x['fundo'] / 3600:5.1f}")
