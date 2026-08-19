#!/usr/bin/env bash
# doc-index.sh — regenera o `README.md` (índice) dos diretórios de documentação
# que não tinham nenhum, a partir dos PRÓPRIOS arquivos.
#
# WHY this exists: a auditoria de 2026-08-18 mediu que **45% dos 1.351 markdowns
# do repo são inalcançáveis** a partir do roteador (`CLAUDE.md` §1 e §5) — e que
# **117 deles são lidos assim mesmo, 790 vezes**. Ou seja: o roteador está
# incompleto, não os docs a mais. O pior caso era um diretório com 99 arquivos e
# ZERO índice (`docs/Motion Nodes/`): o agente que precisa da "folha 03" faz `ls`
# e adivinha o número, ou desiste e reconstrói o que já existe.
#
# ⚠️ O índice é DERIVADO, nunca escrito à mão — precedente direto:
# `scripts/adr-index.sh` (160 ADRs) e o placar da conferência do Motion, ambos
# derivados de propósito. Uma lista de 99 itens mantida à mão envelhece na
# primeira semana, e um índice que mente é pior que nenhum.
#
# ⚠️ A FORMA é a do `docs/Physics/handoffs/README.md`, que já funciona e que
# **nove de nove** `docs/*/handoffs/` adotaram. Ela tem três partes, e as três
# são load-bearing:
#   (a) o aviso de que **isto NÃO é o estado atual** — o estado vivo é o
#       `CLAUDE.md §5`; um doc descreve o mundo no dia em que foi escrito;
#   (b) a tabela ordenada (numérica quando o diretório numera, cronológica
#       quando ele data);
#   (c) a marca ◆ nos arquivos que o `CLAUDE.md` cita — o §5 é quem diz o que
#       ainda é caminho quente.
#
# ⚠️ A marca ◆ é DERIVADA lendo o `CLAUDE.md` (só leitura), nunca uma lista
# mantida aqui: uma lista à mão de "quem o §5 cita" descola do §5 no primeiro
# commit dele, e aí passa a mentir na direção mais cara — dizer que um doc é
# canônico quando o roteador já o largou.
#
# ⚠️ O `Papel` sai do NOME do arquivo, e um nome que não classifica aparece como
# `—`. Isso é um ACHADO, não um defeito: é um doc cujo próprio nome não diz o
# que ele é (`avaliacao_e_melhorias.md`, `04_alem_do_blender.md`).
#
# USAGE
#   bash scripts/doc-index.sh            # regenera todos os índices
#   bash scripts/doc-index.sh --check    # não escreve; sai !=0 se algum está desatualizado (p/ gate)
#   bash scripts/doc-index.sh --list     # imprime os diretórios cobertos e some
#
# Só lê os .md dos diretórios cobertos (+ o CLAUDE.md) e escreve o README.md de cada um.

set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 1

MODE="${1:-write}"
case "$MODE" in
  write|--check|--list) ;;
  *) echo "uso: bash scripts/doc-index.sh [--check|--list]"; exit 2 ;;
esac

python3 - "$MODE" <<'PYEOF'
import os, re, sys, subprocess, difflib

MODE = sys.argv[1]

# ---------------------------------------------------------------------------
# CONFIG — um diretório por entrada. Acrescentar um índice é UMA linha aqui.
#
#   dir          : caminho relativo à raiz do repo
#   titulo       : o `# ` do README gerado
#   o_que_e      : o parágrafo "o que é esta pasta" (pode ser multi-linha)
#   ordem        : 'num' (prefixo numérico) | 'data' (data no nome) | 'nome'
# ---------------------------------------------------------------------------
DIRS = [
    dict(
        dir="docs/Motion Nodes",
        titulo="`Motion Nodes` — índice do módulo",
        ordem="num",
        o_que_e=(
            "O **pensamento** do módulo Motion Nodes: o plano, as pesquisas de referência, "
            "e uma *nota-ADR* por família de nós — o registro de por que cada nó tem os "
            "parâmetros que tem. O registro de **como** foi construído (um arquivo por sessão "
            "de linha) fica em [`handoffs/`](handoffs/README.md); a conferência nó-a-nó, em "
            "[`89_conferencia/`](89_conferencia/README.md)."
        ),
    ),
    dict(
        dir="docs/Motion Nodes/89_conferencia",
        titulo="`89_conferencia` — as folhas da conferência dos nós",
        ordem="num",
        papel_fixo="folha",
        o_que_e=(
            "As **17 folhas** da conferência nó-a-nó do catálogo, abertas pelo "
            "[plano 89](../89_plano_conferencia_dos_nos.md). Uma folha por família: cada uma "
            "lista os nós daquela família, o que o referencial entrega, o que a PH2D entrega, "
            "e os P0/P1/P2 abertos.\n>\n"
            "> ⚠️ **O placar é DERIVADO**, nunca escrito à mão — "
            "[`ferramentas/placar_conferencia.py`](../ferramentas/placar_conferencia.py). "
            "Uma folha é a fonte; o placar é a soma."
        ),
    ),
    dict(
        dir="docs/Vector Module/Estudos",
        titulo="`Vector Module / Estudos` — pesquisa e pensamento vivo",
        ordem="nome",
        o_que_e=(
            "**Estudos** — levantamentos, auditorias de widget, manuais de referência "
            "(Figma / Rive) e os planos de UI/UX que deles saíram. É **pensamento vivo**: "
            "um estudo propõe, não decide.\n>\n"
            "> ⚠️ Esta pasta **continha 13 handoffs de integração** (registro morto misturado "
            "com pensamento vivo). Eles foram para "
            "[`../handoffs/`](../handoffs/README.md) na arrumação de 2026-08-18 — é lá que se "
            "procura *\"o que foi integrado no dia X\"*."
        ),
    ),
    dict(
        dir="docs/audits",
        titulo="`audits` — auditorias por lente",
        ordem="nome",
        o_que_e=(
            "Auditorias **por lente independente** (`lens-alpha`, `lens-beta`, …): cada arquivo "
            "é uma lente que olhou o mesmo diff sem ver o veredito das outras, e o "
            "`-CONSOLIDATED` é a fusão. O protocolo está na "
            "[DIRETRIZ](../IntegracaoMultiAgente/DIRETRIZ.md).\n>\n"
            "> ⚠️ Estas auditorias são de **2026-05** (a era do CTT / W1). Elas registram o que "
            "foi encontrado **naquele diff** — não são um checklist do estado de hoje."
        ),
    ),
    dict(
        dir="docs/Timeline",
        titulo="`Timeline` — índice do módulo",
        ordem="num",
        o_que_e=(
            "O **pensamento** do módulo Timeline: planos de wave, pesquisas de referência, a "
            "auditoria das expressões e o registro da feature que foi **retirada**. O registro "
            "de **como** foi construído fica em [`handoffs/`](handoffs/README.md)."
        ),
    ),
    dict(
        dir="docs/Flip",
        titulo="`Flip` — índice do módulo",
        ordem="num",
        o_que_e=(
            "O **pensamento** do módulo Flip (animação 2D no idioma do Grease Pencil): o plano "
            "de waves, a referência dos algoritmos do Blender 5.2, e a pesquisa do **motor novo "
            "de traço** (o traço percorrido, não rasterizado). O registro de **como** foi "
            "construído fica em [`handoffs/`](handoffs/README.md)."
        ),
    ),
    dict(
        dir="docs/Physics",
        titulo="`Physics` — índice do módulo",
        ordem="num",
        o_que_e=(
            "O **pensamento** do módulo de Física: o plano de waves (que é o *tracker* das "
            "waves), a visão, os planos por família (joints, IK, polia, player de plataforma) e "
            "a auditoria contra as engines de referência. O registro de **como** foi construído "
            "fica em [`handoffs/`](handoffs/README.md)."
        ),
    ),
    dict(
        dir="docs/plans",
        titulo="`plans` — planos de fan-out (multi-módulo)",
        ordem="nome",
        o_que_e=(
            "Planos que **atravessam módulos** — as waves de nós, de imageio, de compressão de "
            "textura, de coerência cromática. Planos de UM módulo moram na pasta do módulo "
            "(`docs/<Módulo>/NN_plano_*.md`), não aqui.\n>\n"
            "> ⚠️ Um plano descreve o que se **pretendia** fazer. O que de facto ficou está no "
            "handoff da wave e no `CLAUDE.md §5` — vários destes têm waves fechadas e fan-out "
            "ainda aberto, e o plano não é atualizado quando isso muda."
        ),
    ),
    dict(
        dir="docs/Deform",
        titulo="`Deform` — índice do módulo",
        ordem="nome",
        o_que_e=(
            "O módulo de **deformação**. ⚠️ O **tracker único** deste módulo é o "
            "[`00_README.md`](00_README.md) (ele diz isso de si mesmo, DIRETIVA §1) — comece por "
            "ele; o resto é pesquisa, arquitetura, spec de painel e plano de implementação.\n>\n"
            "> ⚠️ Esta pasta contém **um handoff** (`HANDOFF_deform_impl.md`) misturado com o "
            "doc vivo. Um handoff é registro **morto** — ele descreve o mundo no dia em que foi "
            "escrito e não é atualizado depois."
        ),
    ),
    dict(
        dir="docs/3D/03-Implementacao",
        titulo="`3D / 03-Implementacao` — a implementação do módulo de escultura",
        ordem="num",
        o_que_e=(
            "A camada de **implementação** do cofre do módulo 3D: onde roda o motor, a "
            "referência SculptGL, o oráculo de fidelidade. O índice do cofre inteiro é o "
            "[`00-INDEX.md`](../00-INDEX.md).\n>\n"
            "> ⚠️ Esta pasta contém **dois handoffs de implementador** (W1 e W4) misturados com "
            "o doc vivo. Um handoff é registro **morto** — ele descreve o mundo no dia em que "
            "foi escrito e não é atualizado depois; os handoffs de sessão do módulo vivem em "
            "[`../handoffs/`](../handoffs/README.md)."
        ),
    ),
    dict(
        dir="docs/DevOps",
        titulo="`DevOps` — máquina, bootstrap e multi-máquina",
        ordem="nome",
        o_que_e=(
            "O runbook de **máquina**: como preparar um clone novo e como as três máquinas "
            "(Mac testes · Linux dev · Windows build) partilham o GitHub como fonte única.\n>\n"
            "> ⚠️ Esta pasta contém **um handoff** (`HANDOFF_linux_bootstrap.md`) misturado com "
            "o doc vivo. Um handoff é registro **morto**; o runbook vivo é o "
            "[`MULTI_MACHINE_SETUP.md`](MULTI_MACHINE_SETUP.md), que o `CLAUDE.md §4` cita."
        ),
    ),
    dict(
        dir="docs/Audio",
        titulo="`Audio` — índice do módulo",
        ordem="num",
        o_que_e=(
            "O **pensamento** do módulo de Áudio (a rack de efeitos, o mixer, a entrega). O que "
            "**falta** vive em [`03_o_que_falta.md`](03_o_que_falta.md) **com o gatilho que "
            "acorda cada item** — é lá que se olha, não no `CLAUDE.md`. O registro de **como** "
            "foi construído fica em [`handoffs/`](handoffs/README.md)."
        ),
    ),
    dict(
        dir="docs/Runtime",
        titulo="`Runtime` — índice do módulo",
        ordem="num",
        o_que_e=(
            "O **pensamento** do módulo Runtime — a saída de sinais (`ph2d-runtime`), onde a "
            "timeline e a física se encontram sem que um produtor chame um consumidor "
            "(ADR-0075). O registro de **como** foi construído fica em "
            "[`handoffs/`](handoffs/README.md)."
        ),
    ),
    # ⛔ `docs/Pixel Art/` e `docs/Tilling/` NÃO entram aqui, e isso é uma CERCA DE
    # CHESTERTON, não um esquecimento: o `.gitignore:100-104` põe as duas fora do
    # git em TODA máquina, por decisão explícita do Enio (2026-07-23 / 2026-07-25),
    # porque são "MVPs paralelos ainda sem associação com o PH2D". Gerar índice ali
    # escreveria num diretório que nenhuma outra máquina tem — e o `--check` de um
    # gate falharia num clone limpo, sobre um arquivo que o repo não deve conter.
    # É também por isso que o `CLAUDE.md §5` não os menciona: não é buraco do
    # roteador, é o produto de uma decisão.
    dict(
        dir="docs/Migracao",
        titulo="`Migracao` — as ondas de migração (2026-05)",
        ordem="nome",
        o_que_e=(
            "O registro das **ondas de migração** de 2026-05 (convention-by-discovery, "
            "eliminação de colisões de nome).\n>\n"
            "> ⚠️ Isto é **histórico**: descreve uma árvore que já mudou de forma.\n"
            "> Os caminhos que apenas **MUDARAM DE ENDEREÇO** (`crates/ph2d-editor/**` →\n"
            "> `ph2d-editor-core/**`, quando a crate virou casca) foram corrigidos em 2026-08-18 —\n"
            "> o arquivo é o mesmo, só o endereço mudou. Os que **MORRERAM** (com o\n"
            "> [ADR-0099](../architecture/decisions/0099-remove-painting-brush-engine-preserve-layers-effects.md),\n"
            "> a remoção da pintura) ficaram como estão, de propósito: ali o texto está certo\n"
            "> sobre o dia em que foi escrito, e reescrever o caminho só trocaria um destino\n"
            "> morto por outro."
        ),
    ),
]

# ---------------------------------------------------------------------------
# Papel, derivado do NOME. Ordem importa: a primeira que casa vence.
# ⚠️ `HANDOFF` vem primeiro de propósito — `11_HANDOFF_AUDITORIA_*` é handoff,
# não auditoria, e é a distinção morto/vivo que este índice existe para dar.
# ---------------------------------------------------------------------------
PAPEIS = [
    (re.compile(r'handoff', re.I),                       "⚠️ handoff (morto)"),
    (re.compile(r'\bBUGS_', re.I),                        "bugs"),
    (re.compile(r'plano|plan\b|_plans?_', re.I),          "plano"),
    (re.compile(r'pesquisa|research|estudo|estado_da_arte|levantamento', re.I), "pesquisa"),
    (re.compile(r'auditoria|audit|lens-|_lens_|conferencia', re.I), "auditoria"),
    (re.compile(r'referencia|reference|manual|oraculo', re.I), "referência"),
    (re.compile(r'nota_adr|_adr\b', re.I),                "nota-ADR"),
    (re.compile(r'briefing|README|INDEX|visao', re.I),    "porta de entrada"),
    (re.compile(r'resultado|achados', re.I),              "resultado"),
]

def papel(base, fixo=None):
    # ⚠️ `papel_fixo` é para o diretório em que o papel vem da PASTA, não do nome:
    # em `89_conferencia/` todo arquivo é uma folha de conferência por construção,
    # e uma coluna inteira de `—` ali seria ruído, não achado.
    for rx, nome in PAPEIS:
        if rx.search(base):
            return nome
    return fixo or "—"

def titulo_de(path):
    """O primeiro `# ` do arquivo, sem o prefixo redundante. Front-matter YAML é pulado."""
    try:
        linhas = open(path, encoding='utf-8', errors='ignore').read().split('\n')[:80]
    except OSError:
        return "(ilegível)"
    for ln in linhas:
        if ln.startswith('# '):
            t = ln[2:].strip()
            t = re.sub(r'^(ADR-\d+|HANDOFF)\s*[—:-]\s*', '', t)
            t = t.replace('|', r'\|')
            return t[:110] + ('…' if len(t) > 110 else '')
    return "(sem `# ` de título)"

DATA_RX = re.compile(r'(20\d\d)[-_](\d\d)[-_](\d\d)')
NUM_RX  = re.compile(r'^(\d+(?:\.\d+)?)')

def chave(base, ordem):
    if ordem == 'num':
        m = NUM_RX.match(base)
        # sem prefixo numérico vai para o fim, em ordem alfabética
        return (0, float(m.group(1)), base.lower()) if m else (1, 0.0, base.lower())
    if ordem == 'data':
        m = DATA_RX.search(base)
        return (0, m.group(0), base.lower()) if m else (1, '', base.lower())
    return (0, base.lower(), '')

def data_de(base, path):
    """A data do NOME quando existe; senão a do commit que ADICIONOU o arquivo."""
    m = DATA_RX.search(base)
    if m:
        return f"{m.group(1)}-{m.group(2)}-{m.group(3)}"
    try:
        out = subprocess.run(
            ['git', 'log', '--diff-filter=A', '--format=%ad', '--date=short', '-1', '--', path],
            capture_output=True, text=True, timeout=20).stdout.strip()
        return out.split('\n')[0] if out else "—"
    except Exception:
        return "—"

# ---------------------------------------------------------------------------
# ◆ — DERIVADO do CLAUDE.md, nunca de uma lista mantida aqui.
# ⚠️ O §5 cita muita coisa pela PASTA (`docs/Flip/`), não pelo arquivo. Uma
# marca por prefixo-de-pasta marcaria a pasta inteira e não diria nada; por isso
# só o BASENAME conta.
# ---------------------------------------------------------------------------
try:
    CANON = open('CLAUDE.md', encoding='utf-8', errors='ignore').read()
except OSError:
    CANON = ""

def citado(base):
    return base in CANON

CABECALHO_AVISO = (
    "> ⚠️ **Isto NÃO é o estado atual do módulo.** O estado vivo é o **[`CLAUDE.md §5`]({claude})**;\n"
    "> um doc descreve o mundo **no dia em que foi escrito** e não é atualizado depois. Use-os\n"
    "> para responder *\"por que isto ficou assim?\"* — nunca para decidir a próxima ação."
)

def gera(cfg):
    d = cfg['dir']
    if not os.path.isdir(d):
        return None, f"✗ {d} não existe"
    arquivos = sorted(
        (f for f in os.listdir(d)
         if f.endswith('.md') and f not in ('README.md', '00_INDEX.md', '00-INDEX.md')),
        key=lambda b: chave(b, cfg['ordem']))
    if not arquivos:
        return None, f"✗ {d} não tem .md"

    # profundidade → caminho relativo até o CLAUDE.md da raiz
    claude = '../' * (d.count('/') + 1) + 'CLAUDE.md'

    L = []
    L.append(f"# {cfg['titulo']}")
    L.append("")
    L.append("> **Gerado por `bash scripts/doc-index.sh` — não edite à mão.** Uma lista mantida à")
    L.append("> mão envelhece na primeira semana; esta é derivada do primeiro `# ` de cada arquivo.")
    L.append(">")
    # ⚠️ o bloco inteiro é UMA citação. Uma linha do config que já traga `>` não
    # pode ganhar outro — `> >` é citação ANINHADA, que renderiza como uma caixa
    # dentro da caixa. Normalize antes de prefixar.
    for ln in cfg['o_que_e'].split('\n'):
        ln = re.sub(r'^>\s?', '', ln).rstrip()
        L.append(">" if not ln else f"> {ln}")
    L.append(">")
    L.extend(CABECALHO_AVISO.format(claude=claude).split('\n'))
    L.append("")

    n_cit = sum(1 for f in arquivos if citado(f))
    n_han = sum(1 for f in arquivos if re.search(r'handoff', f, re.I))
    resumo = f"**{len(arquivos)} arquivos** · **{n_cit}** citados pelo `CLAUDE.md` (marcados **◆**)"
    if n_han:
        resumo += f" · **{n_han}** são handoffs (registro **morto**)"
    L.append(resumo + ".")
    L.append("")

    # ⚠️ a 1ª coluna só existe quando o diretório TEM aquela grandeza. Um diretório
    # ordenado por nome não tem número nem data, e uma coluna inteira de `—` seria
    # ruído a fingir informação.
    cab = {'num': "| # ", 'data': "| Data "}.get(cfg['ordem'], "")
    sep = {'num': "|---", 'data': "|---"}.get(cfg['ordem'], "")
    L.append(cab + "| | Arquivo | Papel | Assunto |")
    L.append(sep + "|---|---|---|---|")
    for f in arquivos:
        m = NUM_RX.match(f)
        col1 = ""
        if cfg['ordem'] == 'num':
            col1 = f"| {m.group(1) if m else '—'} "
        elif cfg['ordem'] == 'data':
            col1 = f"| {data_de(f, os.path.join(d, f))} "
        marca = "◆" if citado(f) else " "
        link = f.replace(' ', '%20')
        L.append(f"{col1}| {marca} | [{f}]({link}) | {papel(f, cfg.get('papel_fixo'))} | "
                 f"{titulo_de(os.path.join(d, f))} |")

    # subdiretórios com índice próprio — o índice tem de dizer que eles existem
    subs = sorted(s for s in os.listdir(d)
                  if os.path.isdir(os.path.join(d, s))
                  and os.path.exists(os.path.join(d, s, 'README.md')))
    if subs:
        L.append("")
        L.append("**Subpastas:** " + " · ".join(
            f"[`{s}/`]({s.replace(' ', '%20')}/README.md)" for s in subs))

    L.append("")
    L.append("---")
    L.append("")
    L.append("⚠️ Um `Papel` `—` é um **achado**, não um defeito deste índice: é um doc cujo próprio")
    L.append("nome não diz o que ele é. Um arquivo **sem** ◆ não é lixo — é um doc que o roteador")
    L.append("(`CLAUDE.md`) não alcança, e essa era exactamente a medição que criou este índice.")
    L.append("")
    return "\n".join(L) + "\n", None


if MODE == '--list':
    for cfg in DIRS:
        print(cfg['dir'])
    sys.exit(0)

n_ok = n_stale = n_err = 0
for cfg in DIRS:
    novo, err = gera(cfg)
    if err:
        print(err); n_err += 1; continue
    out = os.path.join(cfg['dir'], 'README.md')
    velho = open(out, encoding='utf-8').read() if os.path.exists(out) else None
    if MODE == '--check':
        if velho == novo:
            n_ok += 1
        else:
            n_stale += 1
            print(f"✗ {out} desatualizado")
            if velho is not None:
                for ln in list(difflib.unified_diff(
                        velho.split('\n'), novo.split('\n'), lineterm='', n=0))[:12]:
                    print("   " + ln)
    else:
        if velho == novo:
            n_ok += 1
        else:
            with open(out, 'w', encoding='utf-8') as fh:
                fh.write(novo)
            n_stale += 1
            print(f"✓ {out} ({'regenerado' if velho is not None else 'CRIADO'})")

if MODE == '--check':
    if n_stale or n_err:
        print(f"✗ {n_stale} índice(s) desatualizado(s), {n_err} erro(s) — rode: bash scripts/doc-index.sh")
        sys.exit(1)
    print(f"✓ {n_ok} índices em dia")
else:
    print(f"✓ {n_stale} escrito(s) · {n_ok} já em dia · {n_err} erro(s)")
PYEOF
