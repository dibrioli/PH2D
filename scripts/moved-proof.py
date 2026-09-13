#!/usr/bin/env python3
"""moved-proof.py — PROVA DE MOVIMENTO: cada trecho declarado da revisão ANTES aparece UMA vez no destino DEPOIS
(normalizado como o `rustfmt` reformata um trecho que muda de indentação) e já não está na origem.

É o instrumento com que as linhas que partiram as funções gigantes da shell provaram que MOVERAM código sem o
mudar: a `line/render-loop` (o `run_render_frame`, 12/09), a `line/input-dispatch` (o `on_mouse_input`) e a
`line/render-bodies` (os corpos das fases), as duas últimas em 13/09. ⚠️ **Nasceu em TRÊS cópias fora do repo, e
elas já tinham divergido** quando a integração as comparou: o `mover.py` da `input-dispatch` perdoava 2 das 4
reformatações do `rustfmt`, o `ramo.py` e o `verbatim_norm.py` perdoavam as 4, e só o `ramo.py` tirava a vírgula
final de um trecho que acaba num braço de `match`. *Uma lei escrita em três sítios não é uma lei.* Esta é a cópia
única, versionada na integração de 13/09, com a UNIÃO das réguas e um auto-teste.

A régua — `norm` — tira TODO espaço em branco e perdoa exactamente quatro coisas, cada uma medida na
`line/render-loop` num trecho que o `rustfmt` reformatou ao mudar de indentação:
  1. espaço em branco (quebras e indentação);
  2. a vírgula final antes de um fecho — `(\\n a,\\n b,\\n)` junto numa linha perde a vírgula (P3a);
  3. as chavetas de um FECHO de expressão única — `|id| {\\n f(id)\\n }` passa a `|id| f(id)` (P3t);
  4. as chavetas de um BRAÇO de `match` de expressão única, que ganha a vírgula (P6l).
Aplicada aos DOIS lados. Um bloco com instruções (`;` ao nível dele) fica como está, e ⛔ nenhum token de código ou
de comentário é perdoado: o `rustfmt` não mexe no conteúdo de literais nem de comentários.

Uso:
  python3 scripts/moved-proof.py <spec.json> [--before REV] [--after REV] [--root DIR]
  python3 scripts/moved-proof.py --selftest

  --before REV   de onde o trecho sai (omissão: HEAD) — corra ANTES de commitar o movimento;
  --after REV    onde se procura (omissão: os ficheiros de TRABALHO) — para auditar um commit já feito:
                 `--before <c>~1 --after <c>`;
  --root DIR     a raiz do repositório (omissão: `git rev-parse --show-toplevel` a partir do cwd).

spec (JSON — as linhas vêm da própria spec, para nenhuma passar por aspas de shell):
  {
    "src":  "shells/desktop/src/x.rs",     relativo à raiz
    "dest": "shells/desktop/src/y.rs",     destino por omissão (pode ser o próprio src)
    "blocks": [
      {"a": 120, "b": 180,                 1-based e inclusivo, na revisão ANTES
       "first": "<linha a, exacta>", "last": "<linha b, exacta>",
       "dest": "…",                        opcional: o destino DESTE bloco
       "subst": [["velho", "novo", "porquê"], {"old": "…", "new": "…", "n": 2, "why": "…"}]}
    ]
  }
  As trocas DECLARADAS aplicam-se ao trecho de ANTES antes de o procurar no destino; cada uma tem de casar (com `n`,
  exactamente `n` vezes; sem `n`, pelo menos uma) — uma troca que não casa é um erro da spec, nunca um no-op.

Veredito por bloco: `[OK]` ⇔ o trecho (com as trocas) aparece **1×** no destino e o ORIGINAL **0×** na origem.
Sai 0 com todos OK · 1 com algum `[FALHA]` · 2 com a spec ou a revisão inválidas (nada foi medido).
⚠️ Um bloco de UMA linha mede-se por linha inteira (`strip`), não por substring normalizada — normalizado, um
`}` ou um `return;` casaria em qualquer sítio.
"""
import json
import os
import re
import subprocess
import sys
import tempfile

CLOSURE_BLOCK = re.compile(r"\|[^|{};]*\|\{")
ARM_BLOCK = re.compile(r"=>\{")


class SpecError(Exception):
    """A spec, a revisão ou um caminho não servem — a prova não chegou a medir nada."""


def _unbrace(s, rx, arm):
    """Tira as chavetas de um bloco de expressão ÚNICA aberto por `rx` (fecho ou braço), recursivamente.

    Um bloco com `;` ao nível dele fica intacto. Num braço, o bloco vazio também fica (`=> {}` não é uma expressão
    que o `rustfmt` desembrulhe), e as chavetas tiradas dão lugar à vírgula — sem a duplicar se já vinha uma."""
    out = []
    i = 0
    while True:
        m = rx.search(s, i)
        if not m:
            out.append(s[i:])
            break
        open_ = m.end() - 1
        depth, j, stmt = 0, open_, False
        while j < len(s):
            ch = s[j]
            if ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
                if depth == 0:
                    break
            elif ch == ";" and depth == 1:
                stmt = True
            j += 1
        inner = s[open_ + 1 : j] if j < len(s) else ""
        if j >= len(s) or stmt or (arm and not inner):
            out.append(s[i : m.end()])
            i = m.end()
            continue
        out.append(s[i:open_])
        out.append(_unbrace(inner, rx, arm))
        if arm:
            out.append("," if s[j + 1 : j + 2] != "," else "")
        i = j + 1
    return "".join(out)


def norm(s):
    """A régua (ver o cabeçalho): espaço em branco · braço de expressão única · vírgula antes de fecho · fecho de
    expressão única — nesta ordem, porque o braço desembrulhado GANHA uma vírgula que a regra seguinte decide."""
    s = re.sub(r"\s+", "", s)
    s = _unbrace(s, ARM_BLOCK, True)
    s = re.sub(r",(?=[)\]}])", "", s)
    return _unbrace(s, CLOSURE_BLOCK, False)


def _git(root, *args):
    r = subprocess.run(["git", "-C", root, *args], capture_output=True, text=True)
    if r.returncode != 0:
        raise SpecError(f"git {' '.join(args)}: {r.stderr.strip()}")
    return r.stdout


def _read(root, rev, path):
    if rev is None:
        full = os.path.join(root, path)
        if not os.path.isfile(full):
            raise SpecError(f"o ficheiro de trabalho não existe: {path}")
        with open(full, encoding="utf-8") as f:
            return f.read()
    return _git(root, "show", f"{rev}:{path}")


def _apply_subst(text, substs, where):
    for s in substs or []:
        if isinstance(s, dict):
            old, new, n = s["old"], s["new"], s.get("n")
        else:
            old, new, n = s[0], s[1], None
        c = text.count(old)
        if (n is not None and c != n) or (n is None and c == 0):
            want = f"{n}" if n is not None else "≥ 1"
            raise SpecError(f"{where}: a troca {old!r} casou {c}× (a spec pede {want})")
        text = text.replace(old, new)
    return text


def prove(spec, root, before="HEAD", after=None, out=print):
    """Corre a prova e devolve (ok, falhas). Levanta `SpecError` se a spec ou as revisões não servirem."""
    for key in ("src", "blocks"):
        if key not in spec:
            raise SpecError(f"a spec não tem `{key}`")
    if not spec["blocks"]:
        raise SpecError("a spec não tem blocos — uma prova sem sujeito leria-se como aprovada")
    src = spec["src"]
    old = _read(root, before, src).split("\n")
    cache = {}

    def after_text(path):
        if path not in cache:
            cache[path] = _read(root, after, path)
        return cache[path]

    ok_n, fails = 0, 0
    for idx, blk in enumerate(spec["blocks"], 1):
        where = f"bloco {idx} (L{blk.get('a')}-L{blk.get('b')})"
        dest = blk.get("dest") or spec.get("dest")
        if not dest:
            raise SpecError(f"{where}: sem `dest` (nem no bloco nem na spec)")
        a, b = blk["a"], blk["b"]
        if not (1 <= a <= b <= len(old)):
            raise SpecError(f"{where}: fora do ficheiro ANTES ({len(old)} linhas)")
        if old[a - 1] != blk["first"]:
            raise SpecError(f"{where}: a linha {a} ANTES não é `first`: {old[a - 1]!r}")
        if old[b - 1] != blk["last"]:
            raise SpecError(f"{where}: a linha {b} ANTES não é `last`: {old[b - 1]!r}")
        body = old[a - 1 : b]
        orig = "\n".join(body)
        moved = _apply_subst(orig, blk.get("subst"), where)
        dest_text = after_text(dest)
        src_text = after_text(src)
        if len(body) == 1:
            line_moved = moved.strip()
            k = sum(1 for ln in dest_text.split("\n") if ln.strip() == line_moved)
            still = sum(1 for ln in src_text.split("\n") if ln.strip() == body[0].strip())
            if dest == src and line_moved == body[0].strip():
                still -= k
        else:
            # Um trecho que ACABA num braço (`_ => x(..),`) leva a vírgula solta; no destino vem a seguir o `}` do
            # `match` e a régua tira-a lá (vírgula antes de fecho) — sem este corte o trecho nunca casaria.
            n_moved = norm(moved).rstrip(",")
            n_orig = norm(orig).rstrip(",")
            if not n_moved:
                raise SpecError(f"{where}: o trecho é vazio depois de normalizado")
            k = norm(dest_text).count(n_moved)
            still = norm(src_text).count(n_orig)
            if dest == src and n_orig == n_moved:
                still -= k
        good = k == 1 and still == 0
        ok_n += good
        fails += not good
        n_sub = len(blk.get("subst") or [])
        out(
            f"[{'OK' if good else 'FALHA'}] L{a}-L{b} ({len(body)} L) -> {dest}: {k}x no destino"
            + (f"; ainda na origem {still}x" if still else "")
            + (f" ({n_sub} troca(s) declarada(s))" if n_sub else "")
        )
    out(f"== {ok_n + fails} bloco(s): {ok_n} OK, {fails} FALHA")
    return fails == 0, fails


def run(argv, out=print):
    args = list(argv)
    if args == ["--selftest"]:
        return selftest(out)

    def opt(name):
        if name in args:
            i = args.index(name)
            if i + 1 >= len(args):
                raise SpecError(f"{name} pede um valor")
            val = args[i + 1]
            del args[i : i + 2]
            return val
        return None

    try:
        before = opt("--before") or "HEAD"
        after = opt("--after")
        root = opt("--root")
        if len(args) != 1:
            out(__doc__)
            return 2
        if root is None:
            root = _git(os.getcwd(), "rev-parse", "--show-toplevel").strip()
        with open(args[0], encoding="utf-8") as f:
            spec = json.load(f)
        ok, _ = prove(spec, root, before, after, out)
        return 0 if ok else 1
    except (SpecError, OSError, json.JSONDecodeError, KeyError) as e:
        out(f"ABORTA (nada foi medido): {e}")
        return 2


def selftest(out=print):
    """As quatro reformatações são perdoadas, um token nunca; e a prova ponta a ponta num repositório temporário
    reprova nas três formas de mentira (destino alterado · origem que não largou · spec que não bate)."""
    checks = []

    def check(name, cond):
        checks.append((name, bool(cond)))

    # 1. a régua, regra a regra, cada uma com a metade que NÃO pode ser perdoada.
    check("espaço em branco", norm("fn a() {\n        x(1);\n    }") == norm("fn a(){x(1);}"))
    check("vírgula final antes de fecho", norm("f(\n    a,\n    b,\n)") == norm("f(a, b)"))
    check(
        "chavetas de fecho de expressão única",
        norm("take(|id| {\n    slot(id).is_some()\n})") == norm("take(|id| slot(id).is_some())"),
    )
    check("fecho com instruções fica", "{" in norm("f(|x| { a(x); b })"))
    com = "match cmd { A => { f(a) } B => g(), C => { h(); } D => {} }"
    sem = "match cmd {\n A => f(a),\n B => g(),\n C => { h(); }\n D => {}\n}"
    check("chavetas de braço de expressão única", norm(com) == norm(sem))
    check("braço já com vírgula não a duplica", norm("x => { y }") == norm("x => y,") == norm("x => { y },"))
    check("token dentro do braço reprova", norm(com.replace("f(a)", "f(b)")) != norm(sem))
    check("token de código reprova", norm("let a = f(1);") != norm("let a = f(2);"))
    check("texto de comentário reprova", norm("// o pick\nx();") != norm("// o clique\nx();"))

    # 2. ponta a ponta, num repositório descartável.
    def scenario(tmp, dest_body, src_keeps_block, spec_override=None, commit_move=False):
        for p in (tmp,):
            subprocess.run(["git", "init", "-q", p], check=True)
            subprocess.run(["git", "-C", p, "config", "user.email", "t@t"], check=True)
            subprocess.run(["git", "-C", p, "config", "user.name", "t"], check=True)
        block = [
            "        let hits = pick(|id| {",
            "            slot(id).is_some()",
            "        });",
            "        // o realce e o clique leem a MESMA fonte",
            "        self.hovered = hits.first().copied();",
        ]
        src_before = ["impl App {", "    fn frame(&mut self) {", *block, "        self.draw();", "    }", "}", ""]
        with open(os.path.join(tmp, "src.rs"), "w") as f:
            f.write("\n".join(src_before))
        subprocess.run(["git", "-C", tmp, "add", "src.rs"], check=True)
        subprocess.run(["git", "-C", tmp, "commit", "-qm", "antes"], check=True)
        src_after = src_before if src_keeps_block else ["impl App {", "    fn frame(&mut self) {", "        self.fase_hover();", "        self.draw();", "    }", "}", ""]
        with open(os.path.join(tmp, "src.rs"), "w") as f:
            f.write("\n".join(src_after))
        with open(os.path.join(tmp, "dest.rs"), "w") as f:
            f.write("\n".join(["impl App {", "    fn fase_hover(&mut self) {", *dest_body, "    }", "}", ""]))
        spec = {
            "src": "src.rs",
            "dest": "dest.rs",
            "blocks": [{"a": 3, "b": 7, "first": block[0], "last": block[-1]}],
        }
        if spec_override:
            spec["blocks"][0].update(spec_override)
        if commit_move:
            subprocess.run(["git", "-C", tmp, "add", "src.rs", "dest.rs"], check=True)
            subprocess.run(["git", "-C", tmp, "commit", "-qm", "depois"], check=True)
        return spec

    # o destino como o `rustfmt` o deixa: 4 espaços a menos e o fecho de expressão única numa linha.
    formatted = [
        "        let hits = pick(|id| slot(id).is_some());",
        "        // o realce e o clique leem a MESMA fonte",
        "        self.hovered = hits.first().copied();",
    ]
    silent = []
    cases = [
        ("E2E: movimento reformatado pelo rustfmt → OK", dict(dest_body=formatted, src_keeps_block=False), None, 0),
        (
            "E2E: um token alterado no destino → FALHA",
            dict(dest_body=[formatted[0], formatted[1], "        self.hovered = hits.last().copied();"], src_keeps_block=False),
            None,
            1,
        ),
        ("E2E: a origem não largou o trecho → FALHA", dict(dest_body=formatted, src_keeps_block=True), None, 1),
        (
            "E2E: `first` que não bate na revisão ANTES → ABORTA",
            dict(dest_body=formatted, src_keeps_block=False, spec_override={"first": "        let hits = nada;"}),
            None,
            2,
        ),
        (
            "E2E: troca declarada que não casa → ABORTA",
            dict(dest_body=formatted, src_keeps_block=False, spec_override={"subst": [["hovered_object", "x", "nunca casa"]]}),
            None,
            2,
        ),
        (
            "E2E: troca declarada que casa → OK",
            dict(
                dest_body=[formatted[0], formatted[1], "        self.hover = hits.first().copied();"],
                src_keeps_block=False,
                spec_override={"subst": [{"old": "self.hovered =", "new": "self.hover =", "n": 1, "why": "renomeação"}]},
            ),
            None,
            0,
        ),
        (
            "E2E: auditoria de um commit feito (--before HEAD~1 --after HEAD) → OK",
            dict(dest_body=formatted, src_keeps_block=False, commit_move=True),
            ("HEAD~1", "HEAD"),
            0,
        ),
    ]
    for name, kw, revs, want in cases:
        with tempfile.TemporaryDirectory() as tmp:
            spec = scenario(tmp, **kw)
            spec_path = os.path.join(tmp, "spec.json")
            with open(spec_path, "w") as f:
                json.dump(spec, f)
            argv = [spec_path, "--root", tmp]
            if revs:
                argv += ["--before", revs[0], "--after", revs[1]]
            got = run(argv, out=silent.append)
            check(f"{name} (saiu {got})", got == want)

    bad = [n for n, ok in checks if not ok]
    for n, ok in checks:
        out(f"  {'✓' if ok else '✗'} {n}")
    out(f"moved-proof selftest: {len(checks) - len(bad)}/{len(checks)}")
    return 0 if not bad else 1


if __name__ == "__main__":
    sys.exit(run(sys.argv[1:]))
