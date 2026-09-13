"""MIGRAR O TEXTO PINTADO DE UMA CRATE PARA A TABELA DE STRINGS (HR-15) — com asserções, nunca às cegas.

    # 1. o PLANO: uma linha por literal, com a chave sugerida (edite o TSV antes de aplicar)
    python3 scripts/migrar-texto-pintado.py esqueleto ph2d-panel-painter-layers \
        --prefixo panel.painter_layers \
        --seccoes docs/UI_New_and_Simple/ferramentas/seccoes_painter_layers.tsv > plano.tsv

    # 2. APLICAR: troca cada literal marcado `tr` por `tr("chave")` e escreve os braços da tabela
    python3 scripts/migrar-texto-pintado.py aplicar ph2d-panel-painter-layers plano.tsv \
        --tabela crates/ph2d-i18n/src/painter_layers.rs

⭐ **A régua é a da crate `ph2d-label-census`** (`cargo run -p ph2d-label-census --example censo`), a
mesma que os gates por crate correm — este script NÃO lê Rust. *Uma terceira implementação da mesma
régua seria a terceira resposta à mesma pergunta.* Ele só consome as posições que a régua devolve.

⛔ **Toda substituição é conferida no sítio**: o texto nos índices de carácter que a régua deu tem de
ser EXACTAMENTE `"<texto>"`. Um fonte que mudou entre o esqueleto e o aplicar, uma string crua, um
índice desfasado — aborta tudo antes de escrever um byte (CLAUDE.md §2: um `replace` que não casa é
um no-op SILENCIOSO; aqui ele é um erro ALTO).

⚠️ **Três espécies ficam para a MÃO**, e o esqueleto marca-as `manual`:
- o literal de um `format!` (a macro exige um literal: a frase vai inteira para a tabela com os
  marcadores, e o código usa `ph2d_i18n::tr_with`);
- o valor de um `const`/`static` (o `tr` não é `const fn`);
- uma string CRUA (`r"…"`).

Colunas do plano: `acao · rel · linha · inicio · fim · via · chave · texto`.
"""
import os, re, subprocess, sys, collections

HEADER = ["acao", "rel", "linha", "inicio", "fim", "via", "chave", "texto"]
BEGIN, END = "// ph2d-migrar-texto:begin", "// ph2d-migrar-texto:end"


def unescape_tsv(s):
    out, i = [], 0
    while i < len(s):
        if s[i] == "\\" and i + 1 < len(s):
            out.append({"t": "\t", "n": "\n", "\\": "\\"}.get(s[i + 1], "\\" + s[i + 1]))
            i += 2
        else:
            out.append(s[i]); i += 1
    return "".join(out)


def escape_tsv(s):
    return s.replace("\\", "\\\\").replace("\t", "\\t").replace("\n", "\\n")


def slug(text):
    t = re.sub(r"\{[^}]*\}", " ", text).replace("&", " and ")
    t = re.sub(r"[^A-Za-z0-9]+", "_", t).strip("_").lower()
    return re.sub(r"_+", "_", t) or "text"


def load_sections(path):
    rows = []
    for ln in open(path, encoding="utf8"):
        ln = ln.rstrip("\n")
        if not ln or ln.startswith("#"):
            continue
        prefix, section = ln.split("\t")
        rows.append((prefix, section))
    # o prefixo MAIS LONGO ganha: `paint_stroke/jitter_card.rs` antes de `paint_stroke`
    return sorted(rows, key=lambda r: -len(r[0]))


def esqueleto(crate, prefixo, seccoes_path):
    src_root = f"crates/{crate}/src"
    out = subprocess.run(
        ["cargo", "run", "-q", "-p", "ph2d-label-census", "--example", "censo", "--", src_root],
        check=True, capture_output=True, text=True,
    ).stdout
    sections = load_sections(seccoes_path)
    lines_cache = {}
    print("\t".join(HEADER))
    unmapped = set()
    for row in out.splitlines():
        rel, line, start, end, via, text = row.split("\t", 5)
        sec = next((s for p, s in sections if rel.startswith(p)), None)
        if sec is None:
            unmapped.add(rel)
            sec = "SEM_SECCAO"
        if rel not in lines_cache:
            lines_cache[rel] = open(os.path.join(src_root, rel), encoding="utf8").read().split("\n")
        src_line = lines_cache[rel][int(line) - 1]
        acao = "tr"
        if via == "macro:format":
            acao = "manual"
        elif re.match(r"\s*(pub(\([^)]*\))?\s+)?(const|static)\s", src_line):
            acao = "manual"
        print("\t".join([acao, rel, line, start, end, via, f"{prefixo}.{sec}.{slug(unescape_tsv(text))}", text]))
    if unmapped:
        print("⚠️ ficheiros sem secção no mapa (chave SEM_SECCAO): " + ", ".join(sorted(unmapped)), file=sys.stderr)


def read_plan(path):
    rows = []
    with open(path, encoding="utf8") as f:
        head = f.readline().rstrip("\n").split("\t")
        assert head == HEADER, f"cabeçalho inesperado: {head}"
        for ln in f:
            ln = ln.rstrip("\n")
            if not ln:
                continue
            cells = ln.split("\t", len(HEADER) - 1)
            assert len(cells) == len(HEADER), f"linha partida: {ln!r}"
            rows.append(dict(zip(HEADER, cells)))
    return rows


def insert_use(src):
    """`use ph2d_i18n::tr;` depois do PRIMEIRO bloco de `use` de topo (o `rustfmt` ordena-o)."""
    if re.search(r"^use ph2d_i18n::tr;$", src, re.M):
        return src, False
    lines = src.split("\n")
    first_use = next((i for i, l in enumerate(lines) if re.match(r"(pub(\([^)]*\))?\s+)?use\s", l)), None)
    if first_use is None:
        # depois dos `//!` e dos atributos internos do topo
        i = 0
        while i < len(lines) and (lines[i].startswith("//!") or lines[i].startswith("#![") or not lines[i].strip()):
            i += 1
        lines.insert(i, "use ph2d_i18n::tr;")
        return "\n".join(lines), True
    lines.insert(first_use, "use ph2d_i18n::tr;")
    return "\n".join(lines), True


def aplicar(crate, plan_path, tabela):
    src_root = f"crates/{crate}/src"
    rows = read_plan(plan_path)
    # 1. a TABELA: uma chave, um texto — nunca dois
    table = collections.OrderedDict()
    for r in rows:
        if r["acao"] not in ("tr", "manual", "saltar"):
            sys.exit(f"acao desconhecida {r['acao']!r} em {r['rel']}:{r['linha']}")
        if r["acao"] == "saltar":
            continue
        prev = table.get(r["chave"])
        if prev is not None and prev != r["texto"]:
            sys.exit(f"⛔ a chave {r['chave']} teria DOIS textos: {prev!r} e {r['texto']!r}")
        table[r["chave"]] = r["texto"]
    # 2. os FICHEIROS, conferidos antes de qualquer escrita
    by_file = collections.defaultdict(list)
    for r in rows:
        if r["acao"] == "tr":
            by_file[r["rel"]].append(r)
    new_sources = {}
    for rel, rs in by_file.items():
        p = os.path.join(src_root, rel)
        src = open(p, encoding="utf8").read()
        for r in sorted(rs, key=lambda r: -int(r["inicio"])):
            a, b = int(r["inicio"]), int(r["fim"])
            want = '"' + unescape_tsv(r["texto"]) + '"'
            got = src[a:b]
            if got != want:
                sys.exit(f"⛔ {rel}:{r['linha']} — nos índices {a}..{b} está {got!r}, esperava {want!r}. NADA foi escrito.")
            src = src[:a] + f'tr("{r["chave"]}")' + src[b:]
        src, _ = insert_use(src)
        new_sources[p] = (src, len(rs))
    # 3. escrever, e reler para contar
    for p, (src, n) in new_sources.items():
        open(p, "w", encoding="utf8").write(src)
        back = open(p, encoding="utf8").read()
        assert back.count('tr("') >= n, f"{p}: esperava ≥{n} chamadas tr, li {back.count('tr(\"')}"
    arms = "\n".join(f'        "{k}" => "{unescape_tsv(v)}",' for k, v in table.items())
    block = f"{BEGIN}\n{arms}\n        {END}"
    if os.path.exists(tabela):
        t = open(tabela, encoding="utf8").read()
        assert t.count(BEGIN) == 1 and t.count(END) == 1, f"{tabela}: marcadores ausentes ou repetidos"
        t = re.sub(re.escape(BEGIN) + r".*?" + re.escape(END), lambda _: block, t, flags=re.S)
    else:
        t = (
            "pub(crate) fn tr(key: &str) -> Option<&'static str> {\n"
            "    Some(match key {\n"
            f"        {block}\n"
            "        _ => return None,\n"
            "    })\n"
            "}\n"
        )
    open(tabela, "w", encoding="utf8").write(t)
    manual = [r for r in rows if r["acao"] == "manual"]
    print(f"✓ {sum(n for _, n in new_sources.values())} literais trocados em {len(new_sources)} ficheiros · "
          f"{len(table)} chaves na tabela · {len(manual)} à MÃO:")
    for r in manual:
        print(f"  {r['rel']}:{r['linha']}\t{r['via']}\t{r['chave']}\t{r['texto']}")


if __name__ == "__main__":
    a = sys.argv[1:]
    if not a or a[0] not in ("esqueleto", "aplicar"):
        sys.exit(__doc__)
    def opt(name):
        return a[a.index(name) + 1]
    if a[0] == "esqueleto":
        esqueleto(a[1], opt("--prefixo"), opt("--seccoes"))
    else:
        aplicar(a[1], a[2], opt("--tabela"))
