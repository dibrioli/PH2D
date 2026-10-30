#!/usr/bin/env bash
# Verificações BARATAS no commit (item F da comissão de velocidade, 29/10) — sem compilar, alvo < 5 s.
#
#     bash scripts/commit-barato.sh                 # os ficheiros do índice (o que `git commit` leva)
#     bash scripts/commit-barato.sh <paths…>        # estes ficheiros, como estão na árvore (mudados ou não)
#     bash scripts/commit-barato.sh --comando '<o comando git commit inteiro>'   # o hook usa este
#
# ⚠️ É um ESPELHO dos gates Rust, nunca uma 2.ª lei: os tectos (`*_LOC_CAP`) e as listas congeladas
# (`FILE_OVERAGE_OK`, `FN_OVERAGE_OK`, `BASELINE`) LEEM-SE do fonte do gate a cada corrida — mudou o
# gate, mudou o espelho. Se o gate mudar de forma e o espelho não o ler, ele reprova ALTO («espelho
# partido»). A lei continua a ser o gate, na suíte.
#   tecto de ficheiro: architecture_workspace_file_loc_cap · architecture_panel_loc_cap ·
#                      architecture_widget_loc_cap · shells/desktop file_loc_caps (+ marcador morto)
#   tecto de função:   architecture_panel_loc_cap · shells/desktop fn_loc_caps (o mesmo parser)
#   HR-15:             no_literal_color · no_magic_numeric · hr15_no_hardcoded_ui_strings
#   formato:           `rustfmt --check` por stdin (não segue `mod`), com o `rustfmt.toml` mais próximo
#   nextest:           cada `test(…)`/`package(…)` NOVO em `.config/nextest.toml` existe no fonte
# Exit: 0 verde · 1 ✗ (lista `ficheiro:linha — porquê`) · outro = erro do próprio espelho (o hook deixa passar).
# Desligar num commit (só move código entre ficheiros, por exemplo): `PH2D_COMMIT_BARATO=0`.
set -uo pipefail
[ "${PH2D_COMMIT_BARATO:-1}" = "0" ] && exit 0
raiz="$(git rev-parse --show-toplevel 2>/dev/null)" || { echo "commit-barato: fora de um repo git" >&2; exit 2; }
exec python3 -I - "$raiz" "$@" <<'PY'
import os, re, shlex, subprocess, sys, time
from concurrent.futures import ThreadPoolExecutor

t0 = time.monotonic()
raiz = sys.argv[1]
args = sys.argv[2:]
erros = []  # (ficheiro, linha, porquê)


def git(*a, cwd=None, ok=(0,)):
    p = subprocess.run(["git", *a], cwd=cwd or raiz, capture_output=True)
    if p.returncode not in ok:
        return None
    return p.stdout


# ── que ficheiros, e de onde se lê o conteúdo ─────────────────────────────────────────────────────
def do_comando(cmd):
    """(cwd, pathspecs, modo) do `git commit` dentro de um comando de shell; None se não há commit."""
    try:
        lex = shlex.shlex(cmd, posix=True, punctuation_chars=";&|\n")
        lex.whitespace_split = True
        toks = list(lex)
    except ValueError:
        return None
    segs, cur = [], []
    for t in toks:
        if t and set(t) <= set(";&|\n"):
            segs.append(cur); cur = []
        else:
            cur.append(t)
    segs.append(cur)
    cwd = os.environ.get("PH2D_COMMIT_BARATO_CWD") or os.getcwd()
    for s in segs:
        while s and re.match(r"^[A-Za-z_][A-Za-z0-9_]*=", s[0]):
            s = s[1:]
        if len(s) >= 2 and s[0] == "cd":
            cwd = os.path.join(cwd, os.path.expanduser(s[1]))
            continue
        if not s or os.path.basename(s[0]) != "git":
            continue
        i, gcwd = 1, cwd
        while i < len(s) and s[i].startswith("-"):
            if s[i] == "-C" and i + 1 < len(s):
                gcwd = os.path.join(gcwd, s[i + 1]); i += 2
            elif s[i] == "-c" and i + 1 < len(s):
                i += 2
            else:
                i += 1
        if i >= len(s) or s[i] != "commit":
            continue
        rest = s[i + 1:]
        if "--" in rest:
            return gcwd, rest[rest.index("--") + 1:], "arvore"
        if "-a" in rest or "--all" in rest or any(re.match(r"^-[a-zA-Z]*a[a-zA-Z]*$", r) and not r.startswith("--") for r in rest):
            return gcwd, [], "todos"
        return gcwd, [], "indice"
    return None


modo, specs = "indice", []
if args and args[0] == "--comando":
    r = do_comando(args[1] if len(args) > 1 else "")
    if r is None:
        sys.exit(0)
    cwd, specs, modo = r
    top = git("rev-parse", "--show-toplevel", cwd=cwd)
    if top is None:
        sys.exit(0)
    raiz = top.decode().strip()
    specs = [os.path.relpath(os.path.join(cwd, p), raiz) for p in specs]
elif args:
    specs = [a for a in args if a != "--"]
    modo = "literal"

if modo == "literal":  # estes ficheiros como estão, mudados ou não (medição, provas)
    saida = "\n".join(os.path.relpath(os.path.abspath(p), raiz) for p in specs).encode()
elif modo == "arvore":
    saida = git("diff", "--name-only", "--diff-filter=ACMR", "HEAD", "--", *specs)
    novos = git("ls-files", "--others", "--exclude-standard", "--", *specs) or b""
    saida = (saida or b"") + novos
elif modo == "todos":
    saida = git("diff", "--name-only", "--diff-filter=ACMR", "HEAD")
else:
    saida = git("diff", "--cached", "--name-only", "--diff-filter=ACMR")
tocados = sorted(set(x for x in (saida or b"").decode().splitlines() if x))


def ler(rel):
    """O conteúdo que o commit VAI levar (índice no modo índice; árvore no resto)."""
    if modo == "indice":
        b = git("show", ":" + rel)
        if b is not None:
            return b
    try:
        with open(os.path.join(raiz, rel), "rb") as f:
            return f.read()
    except OSError:
        return None


def ler_gate(rel):
    """O fonte de um gate: o que vai no commit se ele estiver no commit; senão a árvore."""
    if rel in tocados:
        return ler(rel)
    try:
        with open(os.path.join(raiz, rel), "rb") as f:
            return f.read()
    except OSError:
        return None


def linhas(b):  # = `str::lines().count()` do Rust
    return b.count(b"\n") + (0 if b == b"" or b.endswith(b"\n") else 1)


# ── leitura das leis dos gates (o espelho) ────────────────────────────────────────────────────────
GW = "crates/ph2d-editor-core/tests/it/architecture_workspace_file_loc_cap.rs"
GP = "crates/ph2d-editor-core/tests/it/architecture_panel_loc_cap.rs"
GWI = "crates/ph2d-editor-core/tests/it/architecture_widget_loc_cap.rs"
GSF = "shells/desktop/tests/it/file_loc_caps.rs"
GSN = "shells/desktop/tests/it/fn_loc_caps.rs"
GS15 = "crates/ph2d-editor-core/tests/it/hr15_no_hardcoded_ui_strings.rs"
_cache = {}


class Partido(Exception):
    pass


def gate_texto(g):
    if g not in _cache:
        b = ler_gate(g)
        if b is None:
            raise Partido(f"{g}: o gate não existe")
        _cache[g] = b.decode("utf-8", "replace")
    return _cache[g]


def gate_const(g, nome):
    m = re.search(r"const %s: usize = ([0-9_]+);" % nome, gate_texto(g))
    if not m:
        raise Partido(f"{g}: não li `const {nome}`")
    return int(m.group(1).replace("_", ""))


def gate_lista(g, nome, com_fn=False):
    t = gate_texto(g)
    m = re.search(r"const %s: &\[[^=]*= &\[" % nome, t)
    if not m:
        raise Partido(f"{g}: não li `const {nome}`")
    resto = t[m.end():]
    if resto.lstrip().startswith("]"):
        return {}
    fim = re.search(r"^\)?\];", resto, re.M)
    if not fim:
        raise Partido(f"{g}: não achei o fim de `{nome}`")
    corpo = "\n".join(l for l in resto[:fim.start()].splitlines() if not l.strip().startswith("//"))
    s = r'"((?:[^"\\]|\\.)*)"'
    if com_fn:
        return {(a, f): int(n) for a, f, n in re.findall(r"\(\s*%s\s*,\s*%s\s*,\s*([0-9_]+)" % (s, s), corpo)}
    return {a: int(n) for a, n in re.findall(r"\(\s*%s\s*,\s*([0-9_]+)" % s, corpo)}


# ── parser de funções: porta do `extract_fn_locs` (panel_loc_cap = "painel"; rust_src = "shell") ──
def line_end(b, i):
    j = b.find(b"\n", i)
    return len(b) if j < 0 else j


def skip_string(b, i):
    i += 1
    while i < len(b):
        c = b[i]
        if c == 0x5C:
            i += 2
        elif c == 0x22:
            return i + 1
        else:
            i += 1
    return None


def skip_raw(b, i):
    i += 1
    h = 0
    while i < len(b) and b[i] == 0x23:
        h += 1; i += 1
    if i >= len(b) or b[i] != 0x22:
        return None
    i += 1
    term = b'"' + b"#" * h
    k = b.find(term, i)
    return None if k < 0 else k + len(term)


def skip_char(b, i):
    a = i + 1
    if a < len(b) and b[a] == 0x5C:
        j = a + 1
        while j < len(b) and b[j] != 0x27:
            j += 1
        return j + 1 if j < len(b) else a
    j = a
    while j < len(b) and j < a + 4:
        if b[j] == 0x27:
            return j + 1
        j += 1
    return a


def matching_brace(b, i):
    depth = 0
    n = len(b)
    while i < n:
        c = b[i]
        if c == 0x2F and i + 1 < n and b[i + 1] == 0x2F:
            i = line_end(b, i)
        elif c == 0x2F and i + 1 < n and b[i + 1] == 0x2A:
            k = b.find(b"*/", i + 2)
            if k < 0:
                return None
            i = k + 2
        elif c == 0x72 and i + 1 < n and b[i + 1] in (0x22, 0x23):
            r = skip_raw(b, i)
            i = r if r is not None else i + 1
        elif c == 0x22:
            i = skip_string(b, i)
            if i is None:
                return None
        elif c == 0x27:
            i = skip_char(b, i)
        elif c == 0x7B:
            depth += 1; i += 1
        elif c == 0x7D:
            depth -= 1
            if depth == 0:
                return i
            i += 1
        else:
            i += 1
    return None


_PERM_PAINEL = {"pub", "pub(crate)", "pub(super)", "async", "const", "unsafe", "pub async", "pub const",
                "pub unsafe", "async unsafe", "const unsafe", "pub(crate) async", "pub(crate) const",
                "pub(crate) unsafe"}


def fn_keyword(tr, var):
    i = 0
    while i + 3 <= len(tr):
        if tr[i:i + 3] == b"fn ":
            p = tr[:i].strip().decode("utf-8", "replace")
            if var == "painel":
                okp = p == "" or p in _PERM_PAINEL or p.startswith("pub(")
            else:
                okp = p == "" or p.startswith("pub") or all(w in ("async", "const", "unsafe", "extern") for w in p.split())
            if okp:
                return i
        i += 1
    return None


def top_brace(b, i, var):
    pa = br = an = 0
    while i < len(b):
        c = b[i]
        if c == 0x28: pa += 1
        elif c == 0x29: pa -= 1
        elif c == 0x5B: br += 1
        elif c == 0x5D: br -= 1
        elif c == 0x3C: an += 1
        elif c == 0x3E and an > 0 and (var == "painel" or (i > 0 and b[i - 1] != 0x2D)): an -= 1
        elif c == 0x7B and pa == 0 and br == 0 and an == 0: return i
        elif c == 0x3B and pa == 0 and br == 0 and an == 0: return None
        i += 1
    return None


def fn_opener(b, i, var):
    while i < len(b):
        ls = i
        le = line_end(b, i)
        line = b[ls:le]
        tr = line.lstrip()
        k = fn_keyword(tr, var)
        if k is not None:
            after = tr[k + 3:].lstrip()
            m = re.match(rb"[A-Za-z0-9_]*", after)
            ne = m.end()
            if ne == 0:
                i = le + 1; continue
            scan = ls + (len(line) - len(tr)) + k + 3 + (len(tr[k + 3:]) - len(after)) + ne
            bs = top_brace(b, min(scan, len(b)), var)
            if bs is not None:
                return after[:ne].decode(), bs
            if var == "painel":
                i = le + 1; continue
        i = le + 1
    return None


def strip_tests(b, var):
    out, i, tag = [], 0, b"#[cfg(test)]"
    while i < len(b):
        r = b.find(tag, i)
        if r < 0:
            out.append(b[i:]); break
        out.append(b[i:r])
        j = r + len(tag)
        if var == "painel":
            while j < len(b) and b[j] != 0x7B:
                j += 1
            if j >= len(b):
                break
        else:
            while j < len(b) and b[j] not in (0x7B, 0x3B):
                j += 1
            if j >= len(b) or b[j] == 0x3B:
                i = r + len(tag); out.append(tag); continue
        k = matching_brace(b, j)
        if k is None:
            break
        i = k + 1
    return b"".join(out)


def fn_locs(src, var):
    s = strip_tests(src, var)
    out, i = [], 0
    while i < len(s):
        r = fn_opener(s, i, var)
        if r is None:
            break
        nome, bs = r
        be = matching_brace(s, bs)
        if be is None:
            break
        out.append((nome, linhas(s[bs:be + 1])))
        i = be + 1
    return out


def linha_da_fn(src, nome):
    m = re.search(rb"\bfn\s+" + re.escape(nome.encode()) + rb"\b", src)
    return src[:m.start()].count(b"\n") + 1 if m else 1


# ── os tectos ─────────────────────────────────────────────────────────────────────────────────────
def tecto_ficheiro(rel, src, cap, allow, key, gate):
    n = linhas(src)
    if n <= cap:
        return
    t = allow.get(key, cap)
    if n > t:
        congel = f" (congelado em {t}: só desce)" if key in allow else ""
        erros.append((rel, cap + 1, f"{n} linhas, tecto {t}{congel} — parta por RESPONSABILIDADE [{gate}]"))


def tecto_fn(rel, src, cap, allow, key, var, gate):
    for nome, n in fn_locs(src, var):
        if n <= cap:
            continue
        t = allow.get((key, nome), cap)
        if n > t:
            erros.append((rel, linha_da_fn(src, nome), f"fn {nome}: {n} linhas, tecto {t} [{gate}]"))


def ws_excluido(r):
    return ("/tests/" in r or r.endswith("/tests.rs") or r.startswith("ph2d-panel-")
            or r.startswith("ph2d-editor-core/src/widget/") or r.startswith("ph2d-tool-runtime/src/")
            or "registry-init" in r)


def eh_ficheiro_de_teste(rel):
    n = os.path.basename(rel)
    return n == "tests.rs" or n.endswith("_tests.rs")


def tectos(rel, src):
    m = re.match(r"^crates/([^/]+)/src/.*\.rs$", rel)
    if m:
        r = rel[len("crates/"):]
        if not ws_excluido(r):
            tecto_ficheiro(rel, src, gate_const(GW, "FILE_LOC_CAP"), gate_lista(GW, "FILE_OVERAGE_OK"), r, os.path.basename(GW))
        crate = m.group(1)
        if crate.startswith("ph2d-panel-") and crate != "ph2d-panel-registry-init":
            tecto_ficheiro(rel, src, gate_const(GP, "PANEL_FILE_LOC_CAP"), gate_lista(GP, "FILE_OVERAGE_OK"), r, os.path.basename(GP))
            tecto_fn(rel, src, gate_const(GP, "PANEL_FN_LOC_CAP"), gate_lista(GP, "FN_OVERAGE_OK", True), r, "painel", os.path.basename(GP))
        if rel.startswith("crates/ph2d-editor-core/src/widget/") and "/showcase/" not in rel[len("crates/ph2d-editor-core/src/widget"):]:
            cap = gate_const(GWI, "WIDGET_LOC_CAP")
            allow = gate_lista(GWI, "FILE_OVERAGE_OK")
            n = linhas(src)
            if n > cap and not any(rel.endswith(p) and n <= t for p, t in allow.items()):
                erros.append((rel, cap + 1, f"{n} linhas, tecto {cap} — parta em sub-ficheiros [{os.path.basename(GWI)}]"))
    if rel.startswith("shells/desktop/src/") and rel.endswith(".rs"):
        r = rel[len("shells/desktop/src/"):]
        tecto_ficheiro(rel, src, gate_const(GSF, "FILE_LOC_CAP"), gate_lista(GSF, "FILE_OVERAGE_OK"), r, os.path.basename(GSF))
        for k, l in enumerate(src.split(b"\n")[:20]):
            t = l.lstrip()
            if t.startswith(b"//") and t[2:].lstrip().startswith(b"ph2d-loc-cap:"):
                erros.append((rel, k + 1, f"o marcador `// ph2d-loc-cap:` morreu: não isenta nada [{os.path.basename(GSF)}]"))
        if not eh_ficheiro_de_teste(rel):
            tecto_fn(rel, src, gate_const(GSN, "FN_LOC_CAP"), gate_lista(GSN, "FN_OVERAGE_OK", True), r, "shell", os.path.basename(GSN))


# ── HR-15 (porta de no_literal_color · no_magic_numeric · hr15_no_hardcoded_ui_strings) ──────────
def ident(c):
    return c is not None and (chr(c).isalnum() and c < 128 or c == 0x5F)


def at(b, i):
    return b[i] if 0 <= i < len(b) else None


def hex_at(b, s):
    if b[s:s + 2] == b"0x":
        p = 2
    elif b[s] == 0x23:
        p = 1
    else:
        return None
    if s > 0 and ident(b[s - 1]):
        return None
    d, i = 0, s + p
    while i < len(b):
        c = b[i]
        if chr(c) in "0123456789abcdefABCDEF":
            d += 1
        elif c != 0x5F:
            break
        i += 1
    if d in (6, 8):
        if i < len(b) and ident(b[i]):
            return None
        return i - s
    return None


def vello_at(b, s):
    if s > 0 and ident(b[s - 1]):
        return None
    for pre in (b"VelloColor::", b"Color::"):
        if b.startswith(pre, s):
            a = s + len(pre)
            break
    else:
        return None
    for kw in (b"WHITE", b"BLACK", b"TRANSPARENT"):
        if b.startswith(kw, a):
            e = a + len(kw)
            if e < len(b) and ident(b[e]):
                continue
            return e - s
    for c in (b"from_rgba8(", b"from_rgba(", b"from_rgb(", b"rgba8(", b"rgb8("):
        if b.startswith(c, a):
            return a + len(c) - s
    return None


def cfg_ranges(b):  # bug-compatível com o gate: chavetas ingénuas
    rs, i, tag = [], 0, b"#[cfg(test)]"
    while i < len(b):
        r = b.find(tag, i)
        if r < 0:
            break
        j = r + len(tag)
        while j < len(b) and b[j] != 0x7B:
            j += 1
        if j >= len(b):
            break
        d, k = 0, j
        while k < len(b):
            if b[k] == 0x7B:
                d += 1
            elif b[k] == 0x7D:
                d -= 1
                if d == 0:
                    rs.append((j, k + 1)); break
            k += 1
        i = k + 1
    return rs


def float_at(b, s):
    if s > 0 and (ident(b[s - 1]) or b[s - 1] == 0x2E):
        return None
    i = s
    if at(b, i) == 0x2D:
        if s > 0 and (ident(b[s - 1]) or b[s - 1] in (0x29, 0x5D)):
            return None
        i += 1
    i0 = i
    while i < len(b) and (chr(b[i]).isdigit() and b[i] < 128 or b[i] == 0x5F):
        i += 1
    if i == i0 or at(b, i) != 0x2E:
        return None
    i += 1
    f0 = i
    while i < len(b) and (chr(b[i]).isdigit() and b[i] < 128 or b[i] == 0x5F):
        i += 1
    if i == f0:
        return None
    if b[i:i + 3] in (b"f32", b"f64"):
        i += 3
    if i < len(b) and ident(b[i]):
        return None
    return i - s, b[s:i].decode()


_ESTRUT = {x + y for x in ("0.0", "-0.0", "0.5", "-0.5", "1.0", "-1.0", "2.0", "-2.0") for y in ("", "f32", "f64")}


def string_spans(b):
    out, i = [], 0
    while i < len(b):
        if b[i] == 0x27:
            j = i + 1
            if j < len(b) and b[j] == 0x5C:
                j += 1
            j += 1
            i = j + 1 if j < len(b) and b[j] == 0x27 else i + 1
        elif b[i] == 0x22:
            s, j = i, i + 1
            while j < len(b):
                if b[j] == 0x5C:
                    j += 2; continue
                if b[j] == 0x22:
                    break
                j += 1
            e = min(j, len(b))
            out.append((s, e)); i = e + 1
        else:
            i += 1
    return out


def sem_comentario_final(b):
    sp = string_spans(b)
    i = 0
    while i + 1 < len(b):
        e = next((e for a, e in sp if a <= i < e), None)
        if e is not None:
            i = e; continue
        if b[i] == 0x2F and b[i + 1] == 0x2F:
            return b[:i]
        i += 1
    return b


def no_ambito_hr15(rel):
    if not rel.endswith(".rs") or "/blender_color_picker/" in rel or eh_ficheiro_de_teste(rel):
        return False
    if rel.startswith(("crates/ph2d-editor-core/src/widget/", "crates/ph2d-editor-core/src/screens/")):
        return True
    m = re.match(r"^crates/(ph2d-panel-[^/]+)/src/", rel)
    return bool(m) and m.group(1) != "ph2d-panel-registry-init"


def strip_tests_linhas(src):  # porta do strip_test_modules do hr15_no_hardcoded_ui_strings
    ls = src.decode("utf-8", "replace").split("\n")
    if ls and ls[-1] == "":
        ls.pop()
    out, i = [], 0
    while i < len(ls):
        if ls[i].lstrip().startswith("#[cfg(test)]"):
            i += 1
            while i < len(ls) and ls[i].lstrip().startswith("#["):
                i += 1
            if i >= len(ls):
                break
            if ls[i].rstrip().endswith(";") and "{" not in ls[i]:
                i += 1; continue
            d, st = 0, False
            while i < len(ls):
                for c in ls[i]:
                    if c == "{":
                        d += 1; st = True
                    elif c == "}":
                        d -= 1
                i += 1
                if st and d <= 0:
                    break
        else:
            out.append(ls[i]); i += 1
    return out


def hr15(rel, src):
    if not no_ambito_hr15(rel):
        return
    texto = src.replace(b"\r\n", b"\n")
    rs = cfg_ranges(texto)
    off = 0
    ls = texto.split(b"\n")
    if ls and ls[-1] == b"":
        ls.pop()
    for no, l in enumerate(ls):
        base = off
        off += len(l) + 1
        if l.lstrip().startswith(b"//"):
            continue
        dentro = lambda i: any(a <= base + i < z for a, z in rs)
        if b"LITERAL-COLOR-OK" not in l:
            i = 0
            while i < len(l):
                n = hex_at(l, i) or vello_at(l, i)
                if n:
                    if not dentro(i):
                        erros.append((rel, no + 1, f"cor literal `{l[i:i+n].decode('utf-8','replace')}` — `ColorToken::X.resolve(theme)` (HR-15) [no_literal_color.rs]"))
                    i += n
                else:
                    i += 1
        if b"LITERAL-PX-OK" not in l:
            lc = sem_comentario_final(l)
            sp = string_spans(lc)
            i = 0
            while i < len(lc):
                e = next((e for a, e in sp if a <= i < e), None)
                if e is not None:
                    i = e; continue
                r = float_at(lc, i)
                if r:
                    n, lit = r
                    if not dentro(i) and lit.replace("_f32", "f32").replace("_f64", "f64") not in _ESTRUT:
                        erros.append((rel, no + 1, f"número literal de UI `{lit}` — token Spacing/Radius/StrokeToken/TypeToken… (HR-15) [no_magic_numeric.rs]"))
                    i += n
                else:
                    i += 1
    # strings: `.label("` / `.placeholder("` contadas por ficheiro contra o BASELINE
    if rel.endswith(".rs") and not rel.startswith("crates/ph2d-editor-core/src/screens/"):
        if rel.startswith("crates/ph2d-editor-core/src/widget/"):
            key = rel[len("crates/ph2d-editor-core/src/widget/"):]
        else:
            key = rel[len("crates/"):]
        n, primeira = 0, None
        for l in strip_tests_linhas(src):
            if l.lstrip().startswith("//"):
                continue
            k = l.count('.label("') + l.count('.placeholder("')
            if k and primeira is None:
                primeira = l.strip()
            n += k
        t = gate_lista(GS15, "BASELINE").get(key, 0)
        if n > t:
            ln = next((k + 1 for k, l in enumerate(src.decode("utf-8", "replace").split("\n")) if l.strip() == primeira), 1)
            erros.append((rel, ln, f"{n} texto(s) fixo(s) `.label(\"…\")`/`.placeholder(\"…\")`, o baseline deixa {t} — i18n (HR-15) [{os.path.basename(GS15)}]"))


# ── rustfmt por stdin (não segue `mod`, lê o rustfmt.toml mais próximo) ────────────────────────────────
def edicao():
    try:
        m = re.search(r'^edition\s*=\s*"(\d+)"', open(os.path.join(raiz, "Cargo.toml")).read(), re.M)
        return m.group(1) if m else "2024"
    except OSError:
        return "2024"


def fmt(rel, src, ed):
    if b"@generated" in b"\n".join(src.split(b"\n")[:5]):
        return None  # o `cargo fmt` salta-os (`format_generated_files = false`, 5 linhas)
    cmd =["rustfmt", "--check", "--edition", ed]
    # O `rustfmt.toml` MAIS PRÓXIMO, como o `cargo fmt`: um vendorizado traz o seu
    # (`disable_all_formatting`), e o da raiz reprovava 46 ficheiros do wgpu-core (30/10).
    d = os.path.dirname(os.path.join(raiz, rel))
    while True:
        if os.path.exists(os.path.join(d, "rustfmt.toml")):
            cmd += ["--config-path", os.path.join(d, "rustfmt.toml")]
            break
        if os.path.realpath(d) == os.path.realpath(raiz) or os.path.dirname(d) == d:
            break
        d = os.path.dirname(d)
    try:
        p = subprocess.run(cmd, input=src, capture_output=True, cwd=raiz, timeout=8)
    except (OSError, subprocess.TimeoutExpired):
        return None
    m = re.search(rb"Diff in .*?:(\d+):", p.stdout)  # ⚠️ por stdin o `--check` sai 0 COM diff
    if p.returncode == 0 and not m:
        return None
    if m:
        return (rel, int(m.group(1)), "fora do `cargo fmt` — corra `cargo fmt` neste ficheiro")
    err = p.stderr.decode("utf-8", "replace").strip().splitlines()
    return (rel, 1, "o rustfmt não o lê: " + (err[0] if err else f"exit {p.returncode}"))


# ── nextest: cada filtro NOVO nomeia algo que existe no fonte ─────────────────────────────────────
def nextest(rel, src):
    velho = git("show", "HEAD:" + rel) or b""
    atomo = re.compile(r"\b(test|package)\(([^()]*)\)")
    antes = set(atomo.findall(velho.decode("utf-8", "replace")))
    pedidos = []
    for no, l in enumerate(src.decode("utf-8", "replace").split("\n")):
        if not re.match(r"\s*(default-)?filter\s*=", l):
            continue
        for k, v in atomo.findall(l):
            if (k, v) not in antes:
                pedidos.append((no + 1, k, v.strip()))
    if not pedidos:
        return
    nomes_pkg = None
    for no, k, v in pedidos:
        if k == "package":
            if nomes_pkg is None:
                o = git("grep", "-h", "-E", r"^name\s*=", "--", "*Cargo.toml", ok=(0, 1)) or b""
                nomes_pkg = set(re.findall(r'"([^"]+)"', o.decode()))
            if v not in nomes_pkg:
                erros.append((rel, no, f"package({v}) — não há crate com esse nome"))
            continue
        if v.startswith("/") or v.startswith("#"):
            continue  # regex/glob: não se confere por texto
        exacto = v.startswith("=")
        alvo = v.lstrip("=~")
        ult = alvo.split("::")[-1]
        if not re.fullmatch(r"[A-Za-z0-9_]+", ult):
            continue
        if exacto:
            achou = subprocess.run(["git", "grep", "-q", "-w", "-E", r"fn\s+" + ult, "--", "*.rs"], cwd=raiz).returncode == 0
        else:
            achou = subprocess.run(["git", "grep", "-q", "-F", ult, "--", "*.rs"], cwd=raiz).returncode == 0
        if not achou:
            erros.append((rel, no, f"test({v}) — nenhuma `fn {ult}` no fonte: o filtro não casa nada"))


# ── corrida ───────────────────────────────────────────────────────────────────────────────────────
try:
    conteudos = {}
    for rel in tocados:
        b = ler(rel)
        if b is not None:
            conteudos[rel] = b
    ed = edicao()
    rs = [r for r in conteudos if r.endswith(".rs")]
    with ThreadPoolExecutor(max_workers=min(8, max(1, len(rs)))) as ex:
        fut = [ex.submit(fmt, r, conteudos[r], ed) for r in rs]
        for rel in rs:
            tectos(rel, conteudos[rel])
            hr15(rel, conteudos[rel])
        if ".config/nextest.toml" in conteudos:
            nextest(".config/nextest.toml", conteudos[".config/nextest.toml"])
        for f in fut:
            r = f.result()
            if r:
                erros.append(r)
except Partido as e:
    print(f"✗ commit-barato: espelho partido — {e}. O gate mudou de forma: actualize scripts/commit-barato.sh "
          f"(ou PH2D_COMMIT_BARATO=0 neste commit).")
    sys.exit(1)

dt = time.monotonic() - t0
if erros:
    for f, l, p in sorted(set(erros)):
        print(f"✗ {f}:{l} — {p}")
    print(f"✗ commit-barato: {len(set(erros))} problema(s) em {len(conteudos)} ficheiro(s), {dt:.2f} s")
    sys.exit(1)
print(f"✓ commit-barato: {len(conteudos)} ficheiro(s), {dt:.2f} s")
PY
