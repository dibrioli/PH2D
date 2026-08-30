#!/usr/bin/env bash
# stack-audit.sh — o inventário de dependências do PH2D, DERIVADO da árvore.
#
# Por que existe: o plano de atualização (docs/Atualizar Stack/) precisa de uma
# tabela «onde estamos × onde dá para chegar». Escrita à mão, ela envelhece na
# semana seguinte — e o §5.0 do CLAUDE.md já cobrou esse preço cinco vezes
# («a fonte de cada número é o código, não a seção»). Então ela se CONTA.
#
# O que ele responde, em UMA corrida:
#   1. toda dependência de terceiros declarada por um MEMBRO do workspace
#      (crates/*, tools/*, shells/desktop, tests/spike) — o resto da árvore
#      (spikes/, docs/Pixel Art/Resources/, vendor/) fica FORA de propósito:
#      não é código que a gente compila nem policia;
#   2. a versão mais nova publicada de cada uma (índice esparso do crates.io);
#   3. a classificação do salto: MAIOR (quebra API) · menor (sobe sozinha) · igual;
#   4. ⚠️ os TETOS — quando uma dependência nossa é segurada por OUTRA
#      dependência. É a parte que uma leitura de `cargo outdated` não dá, e é
#      onde o plano inteiro se decide: «o mais recente possível» ≠ «o mais recente».
#
# Uso:
#   bash scripts/stack-audit.sh            # tabela completa
#   bash scripts/stack-audit.sh --maior    # só os saltos que quebram API
#   bash scripts/stack-audit.sh --tetos    # só as amarras
#   bash scripts/stack-audit.sh --offline  # sem rede: só o que a árvore declara
#
# Sai 0 sempre — é uma sonda, não um portão.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1
exec python3 - "$@" <<'PY'
import os, re, sys, json, collections
import urllib.request, concurrent.futures as cf

ARGS = set(sys.argv[1:])
OFFLINE = '--offline' in ARGS
ONLY_MAJOR = '--maior' in ARGS
ONLY_CEIL = '--tetos' in ARGS

# ── 1. quem é membro do workspace ────────────────────────────────────────────
# Espelha o `members` do Cargo.toml raiz (crates/*, tools/*, shells/desktop,
# tests/spike) e o `exclude` (vendor/deep_filter). Uma crate fora disto não é
# compilada pelo nosso `--workspace` e não entra na conta.
def is_member(path: str) -> bool:
    p = path.replace('\\', '/').lstrip('./')
    if '/vendor/' in p:
        return False
    d = os.path.dirname(p)
    parts = d.split('/')
    if parts[0] in ('crates', 'tools') and len(parts) == 2:
        return True
    return d in ('shells/desktop', 'tests/spike')

SECT = re.compile(r'^\[(?:(?:target\.[^\]]+\.)?)(dependencies|dev-dependencies|build-dependencies)\]\s*$')
SKIP_DIRS = {'target', '.git', 'Worktrees', 'node_modules', 'target-slots', 'backups'}

declared = collections.defaultdict(list)   # crate -> [(dono, req, kind, membro)]
for root, dirs, files in os.walk('.'):
    dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
    if 'Cargo.toml' not in files:
        continue
    path = os.path.join(root, 'Cargo.toml')
    cur = None
    try:
        lines = open(path, encoding='utf-8', errors='replace').read().splitlines()
    except OSError:
        continue
    for line in lines:
        s = line.strip()
        if s.startswith('['):
            m = SECT.match(s)
            cur = m.group(1) if m else None
            continue
        if not cur or not s or s.startswith('#'):
            continue
        m = re.match(r'^([A-Za-z0-9_-]+)\s*=\s*(.+)$', s)
        if not m:
            continue
        name, rhs = m.group(1), m.group(2)
        if name.startswith('ph2d'):
            continue
        if 'path' in rhs and 'version' not in rhs:
            continue          # dependência local: não tem versão publicada
        mv = re.match(r'^"([^"]+)"', rhs) or re.search(r'version\s*=\s*"([^"]+)"', rhs)
        if not mv:
            continue
        declared[name].append((os.path.dirname(path).lstrip('./'), mv.group(1), cur, is_member(path)))

members = {k: v for k, v in declared.items() if any(d[3] for d in v)}
foreign = {k: v for k, v in declared.items() if not any(d[3] for d in v)}

# ── 2. o mais novo publicado, pelo índice esparso ────────────────────────────
def idx_path(n):
    n = n.lower()
    if len(n) == 1: return f"1/{n}"
    if len(n) == 2: return f"2/{n}"
    if len(n) == 3: return f"3/{n[0]}/{n}"
    return f"{n[:2]}/{n[2:4]}/{n}"

def fetch(name):
    url = f"https://index.crates.io/{idx_path(name)}"
    try:
        req = urllib.request.Request(url, headers={'User-Agent': 'ph2d-stack-audit'})
        body = urllib.request.urlopen(req, timeout=30).read().decode()
    except Exception as e:
        return name, None, None, str(e)
    rows = []
    for line in body.splitlines():
        if not line.strip():
            continue
        d = json.loads(line)
        if d.get('yanked'):
            continue
        v = d['vers']
        if '-' in v.split('+')[0]:     # pré-lançamento não conta
            continue
        rows.append(d)
    if not rows:
        return name, None, None, 'sem versão estável'
    def _k(v):
        nums = re.findall(r'\d+', v.split('+')[0].split('-')[0])[:3]
        t = tuple(int(x) for x in nums) if nums else (0,)
        return t + (0,) * (3 - len(t))
    newest = max(rows, key=lambda d: _k(d['vers']))
    return name, newest, rows, None

index = {}
if not OFFLINE:
    with cf.ThreadPoolExecutor(max_workers=12) as ex:
        for name, newest, allrows, err in ex.map(fetch, members.keys()):
            index[name] = (newest, allrows, err)

# ── 3. comparação semver ─────────────────────────────────────────────────────
def key(v):
    nums = re.findall(r'\d+', v.split('+')[0].split('-')[0])[:3]
    t = tuple(int(x) for x in nums) if nums else (0,)
    return t + (0,) * (3 - len(t))

def eff_major(v):
    """O componente que o `^` do cargo trava: o primeiro não-zero."""
    t = key(v)
    if t[0]: return ('M', t[0])
    if t[1]: return ('m', t[1])
    return ('p', t[2])

def clean(r):
    return r.lstrip('=^~<>* ') or '0'

rows = []
for name, sites in sorted(members.items()):
    mine = [s for s in sites if s[3]]
    reqs = sorted({s[1] for s in mine})
    lowest = min(reqs, key=lambda r: key(clean(r)))
    newest, allrows, err = index.get(name, (None, None, 'offline'))
    if not newest:
        rows.append((name, reqs, None, 'sem-dados', mine, err))
        continue
    lv = newest['vers']
    if eff_major(clean(lowest)) != eff_major(lv):
        gap = 'MAIOR'
    elif key(clean(lowest)) >= key(lv):
        gap = 'igual'
    else:
        gap = 'menor'
    rows.append((name, reqs, lv, gap, mine, newest))

# ── 4. os TETOS: quem segura quem ────────────────────────────────────────────
# Para cada crate que TAMBÉM é dependência de outra crate nossa-por-tabela,
# vê se o requisito dessa outra impede a versão mais nova.
CEIL_WATCH = {}
if not OFFLINE:
    interesting = [n for n, _, lv, gap, _, _ in rows if gap in ('MAIOR', 'menor') and lv]
    def deps_of(name, ver):
        newest, allrows, err = index.get(name, (None, None, None))
        if not allrows:
            return {}
        for d in allrows:
            if d['vers'] == ver:
                return {x['name']: x['req'] for x in d['deps'] if x['kind'] == 'normal'}
        return {}
    # o requisito que cada candidato-a-topo impõe sobre os outros
    for holder, _, lv, gap, _, _ in rows:
        if not lv:
            continue
        for dep, req in deps_of(holder, lv).items():
            if dep in members:
                CEIL_WATCH.setdefault(dep, []).append((holder, lv, req))

def satisfies(req, ver):
    """Aproximação honesta de `^X.Y.Z`: mesmo major efetivo e >= o pedido."""
    r = clean(req)
    if not r:
        return True
    return eff_major(r) == eff_major(ver) and key(ver) >= key(r)

ceilings = []
for name, reqs, lv, gap, mine, _ in rows:
    if not lv or name not in CEIL_WATCH:
        continue
    blockers = [(h, hv, rq) for h, hv, rq in CEIL_WATCH[name] if not satisfies(rq, lv)]
    if blockers:
        allrows = index[name][1] or []
        ok = [d['vers'] for d in allrows if all(satisfies(rq, d['vers']) for _, _, rq in CEIL_WATCH[name])]
        # Sem versão que sirva a TODOS os donos, o cargo carrega duas cópias.
        # Isso é benigno para uma folha (compressão, derive) e venenoso para
        # quem aparece na NOSSA superfície (um tipo de wgpu/vello/glam atravessa
        # a fronteira e vira erro de tipo). O veredito é por crate, não geral.
        ceilings.append((name, reqs, lv, max(ok, key=key) if ok else 'DUAS CÓPIAS (nenhuma versão serve a todos)', blockers))

# ── 5. saída ─────────────────────────────────────────────────────────────────
W, RESET = '\033[1m', '\033[0m'
def hdr(t): print(f"\n{W}{t}{RESET}\n" + '─' * 78)

if not ONLY_CEIL:
    maj = [r for r in rows if r[3] == 'MAIOR']
    mnr = [r for r in rows if r[3] == 'menor']
    eq  = [r for r in rows if r[3] == 'igual']
    nod = [r for r in rows if r[3] == 'sem-dados']
    hdr(f"MAIOR — salto que quebra API ({len(maj)})")
    for n, reqs, lv, _, mine, _ in maj:
        dev = all(s[2] != 'dependencies' for s in mine)
        print("  %-18s %-12s → %-14s %s%s" % (n, '/'.join(reqs), lv,
              ' '.join(sorted({s[0].replace('crates/', '') for s in mine})[:4]),
              '  [só-dev]' if dev else ''))
    if not ONLY_MAJOR:
        hdr(f"menor — compatível, sobe com `cargo update` ({len(mnr)})")
        print('  ' + ', '.join(f"{n} {'/'.join(q)}→{lv}" for n, q, lv, _, _, _ in mnr))
        hdr(f"igual — já no topo ({len(eq)})")
        print('  ' + ', '.join(n for n, *_ in eq))
        if nod:
            hdr(f"sem dados ({len(nod)})")
            for n, q, _, _, _, err in nod:
                print(f"  {n}: {err}")

if not ONLY_MAJOR:
    # ⛔ **Um teto que a rede não respondeu SOME em silêncio, e o cabeçalho conta
    # sem ele.** O `ceilings` só admite quem tem `lv` (a versão mais nova
    # publicada); uma consulta falhada devolve `lv = None` e a crate é saltada
    # pelo `continue` acima — então o número entre parênteses lê-se como *«esta é
    # a resposta»* quando pode ser *«não consegui perguntar»*. É o mesmo byte
    # para duas coisas, e foi apanhado ao ver a MESMA corrida dar 6 e 7.
    # ⇒ Quem está sob vigilância e ficou sem dados é NOMEADO aqui, sempre.
    # ⚠️ **A vigilância NÃO se filtra por `CEIL_WATCH`** — foi a 1.ª tentativa, e a
    # prova de mutação (bloquear a rede) matou-a: o `CEIL_WATCH` é ele próprio
    # DERIVADO das respostas do índice (`deps_of` lê o `allrows`), então sem rede
    # ele nasce **vazio** e o filtro não casa com ninguém — a cura ficava muda
    # exactamente no caso que devia gritar.
    # ⇒ O sinal honesto é toda crate que não pôde ser consultada: sem ela não se
    # sabe sequer *quem vigiar*, logo o inventário inteiro está incompleto.
    mudas = sorted(
        (n, err or 'sem resposta')
        for n, reqs, lv, gap, mine, err in rows
        if not lv
    )
    suf = f" — ⚠️ {len(mudas)} por medir" if mudas else ""
    hdr(f"⚠️  TETOS — o mais novo NÃO é alcançável ({len(ceilings)}){suf}")
    if mudas:
        print(f"  ⛔ {len(mudas)} crate(s) NÃO puderam ser consultadas — o número acima está INCOMPLETO.")
        print("     Sem elas não se sabe sequer QUEM segura quem, logo um teto pode estar a faltar.")
        for n, err in mudas[:8]:
            print(f"      · {n}: {err}")
        if len(mudas) > 8:
            print(f"      · … e mais {len(mudas) - 8}")
        print()
    if not ceilings and not mudas:
        print("  nenhum: toda dependência pode ir ao topo.")
    elif not ceilings:
        print("  nenhum ENTRE AS QUE RESPONDERAM.")
    for n, reqs, lv, ok, blockers in sorted(ceilings):
        print(f"  {n}: temos {'/'.join(reqs)} · topo {lv} · {W}dá para usar {ok}{RESET}")
        for h, hv, rq in blockers:
            print(f"      ↳ segurado por {h} {hv}, que pede {rq}")

if foreign and not ONLY_MAJOR and not ONLY_CEIL:
    hdr(f"fora do workspace — NÃO são nossas ({len(foreign)})")
    byhome = collections.defaultdict(list)
    for n, sites in foreign.items():
        byhome[sites[0][0]].append(n)
    for home, names in sorted(byhome.items()):
        print(f"  {home}\n      {', '.join(sorted(names))}")

print()
PY
