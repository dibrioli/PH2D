"""Sonda do I7 (Onda 2 do CodeEditor): as acções de contexto e o «você quis dizer», numa rodada.

1. `luau-lsp` 1.70.1 lançado COMO O APP (a API gerada em `<cache>/ph2d/luau-lsp/1.70.1/api/`): que
   diagnóstico e que `textDocument/codeAction` sobre `pirnt(1)`, `ph2d.proprety`, `self.ampiltude`,
   com e sem `--!strict` — no range do diagnóstico (como o I0b) e no CURSOR (o que o Alt+Enter pede);
   `codeAction/resolve` de cada título; as `inlayHint` de tipo e as suas `textEdits`.
2. A completion do próprio servidor no sítio do erro (fim da palavra e logo depois do `.`): a lista
   tem a palavra certa? quantos itens? (se bastar, não se mantém uma lista de nomes à mão).
3. Os oráculos do «você quis dizer», CORRIDOS sobre os mesmos nomes (nenhum fonte lido): Python
   (`NameError`/`AttributeError`), `rustc` (locais e campos) e Godot (`InputMap`: «Did you mean»), e
   a `String.similarity` do Godot; ao lado, Levenshtein e Damerau (OSA) de cada par.

Uso (da raiz da worktree):  python3 -I docs/CodeEditor/ferramentas/lspprobe_i7.py
Grava docs/CodeEditor/ferramentas/medicoes_onda2/I7_bruto.json e imprime as tabelas.
"""
import glob, importlib.util, json, os, re, subprocess, sys, tempfile

AQUI = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("lspprobe2", os.path.join(AQUI, "lspprobe2.py"))
P = importlib.util.module_from_spec(spec)
spec.loader.exec_module(P)

API = os.path.expanduser("~/.cache/ph2d/luau-lsp/1.70.1/api")
INLAY = {**P.BASE, "inlayHints": {"parameterNames": "all", "variableTypes": True, "parameterTypes": True,
                                  "functionReturnTypes": True}}


def com_typos(b):
    b = P.troca(b, "  self.t = self.t + dt\n", "  self.t = self.t + dt\n  pirnt(1)\n")
    b = P.troca(b, 'ph2d.property("speed"', 'ph2d.proprety("speed"')
    return P.troca(b, "s * self.amplitude * brilho", "s * self.ampiltude * brilho")


# (rótulo, agulha, deslocamento do início da palavra dentro da agulha, comprimento da palavra)
PALAVRAS = [("pirnt", "pirnt(1)", 0, 5), ("proprety", "ph2d.proprety", 5, 8), ("ampiltude", "self.ampiltude", 5, 9)]


def sobrepoe(d, pos):
    s, e = d["range"]["start"], d["range"]["end"]
    k = (pos["line"], pos["character"])
    return (s["line"], s["character"]) <= k <= (e["line"], e["character"])


def lsp_fase(out):
    s = P.Lsp(INLAY, extra_args=[f"--docs={J}"])
    r = {"args": s.args}
    for modo, pref in (("nao_estrito", ""), ("strict", "--!strict\n")):
        text = pref + com_typos(P.bob())
        uri = f"file:///ph2d-project/i7_{modo}.luau"
        s.open(uri, text)
        ds = P.diags(s, uri)
        linhas = text.splitlines()
        m = {"diagnosticos": [], "no_cursor": {}, "completion": {}}
        for d in ds:
            _, ca, _ = s.req("textDocument/codeAction", {"textDocument": {"uri": uri}, "range": d["range"],
                                                         "context": {"diagnostics": [d], "triggerKind": 1}})
            acoes = ca.get("result") or []
            m["diagnosticos"].append({"code": d.get("code"), "severity": d.get("severity"),
                                      "message": d["message"][:240], "range": d["range"],
                                      "linha": linhas[d["range"]["start"]["line"]].strip(),
                                      "acoes": [P.resumo_acao(a) for a in acoes]})
        resolvidos = {}
        for rot, agulha, off, n in PALAVRAS:
            for onde, k in (("inicio", off), ("meio", off + n // 2), ("fim", off + n)):
                pos = P.pos_de(text, agulha, k)
                ctx = [d for d in ds if sobrepoe(d, pos)]
                ms, ca, _ = s.req("textDocument/codeAction", {"textDocument": {"uri": uri},
                                                              "range": {"start": pos, "end": pos},
                                                              "context": {"diagnostics": ctx, "triggerKind": 1}})
                acoes = ca.get("result") or []
                m["no_cursor"][f"{rot}_{onde}"] = {"ms": ms, "pos": pos, "codes_no_contexto": [d.get("code") for d in ctx],
                                                  "acoes": [P.resumo_acao(a) for a in acoes]}
                for a in acoes:
                    if a.get("title") not in resolvidos and not a.get("edit"):
                        _, rr, _ = s.req("codeAction/resolve", a)
                        res = rr.get("result") or rr.get("error")
                        resolvidos[a.get("title")] = P.resumo_acao(res) if isinstance(res, dict) and "title" in res else res
            for onde, k in (("depois_do_ponto_ou_inicio", off), ("fim_da_palavra", off + n)):
                pos = P.pos_de(text, agulha, k)
                ms, c, nb = s.req("textDocument/completion", P.at(uri, pos))
                res = c.get("result")
                items = res["items"] if isinstance(res, dict) else (res or [])
                labels = [i["label"] for i in items]
                alvo = {"pirnt": "print", "proprety": "property", "ampiltude": "amplitude"}[rot]
                it = next((i for i in items if i["label"] == alvo), None)
                m["completion"][f"{rot}_{onde}"] = {
                    "ms": ms, "bytes": nb, "count": len(items),
                    "isIncomplete": res.get("isIncomplete") if isinstance(res, dict) else None,
                    "tem_o_alvo": it is not None, "alvo_item": {k2: it.get(k2) for k2 in ("label", "kind", "sortText", "textEdit", "insertText")} if it else None,
                    "labels": labels}
        m["resolve"] = resolvidos
        ms, ih, nb = s.req("textDocument/inlayHint", {"textDocument": {"uri": uri},
                                                      "range": {"start": {"line": 0, "character": 0},
                                                                "end": {"line": len(linhas), "character": 0}}})
        hints = ih.get("result") or []
        m["inlay_tipos"] = {"ms": ms, "bytes": nb, "n_total": len(hints),
                            "tipos": [{"pos": h["position"], "label": h["label"], "textEdits": h.get("textEdits"),
                                       "linha": linhas[h["position"]["line"]].strip()}
                                      for h in hints if h.get("kind") == 1]}
        r[modo] = m
        s.close(uri)
    s.stop()
    out["lsp"] = r
    # O cliente Rust de hoje não declara `codeAction` nem `inlayHint`: o servidor muda a forma? E a
    # quickfix do `FunctionUnused` vem sem o diagnóstico no `context` (a aba esconde-o nos ganchos)?
    cli = {}
    for nome, ext in (("sem_capacidades", False), ("com_capacidades", True)):
        s = P.Lsp(INLAY, extra_args=[f"--docs={J}"], extended=ext)
        text = com_typos(P.bob()) + "local function sobra() end\n"
        uri = "file:///ph2d-project/i7_cliente.luau"
        s.open(uri, text)
        ds = P.diags(s, uri)
        c = {}
        for rot, agulha, off in (("init", "function init", 9), ("sobra", "function sobra", 9)):
            pos = P.pos_de(text, agulha, off)
            ctx = [d for d in ds if sobrepoe(d, pos)]
            for com in (True, False):
                _, ca, _ = s.req("textDocument/codeAction", {"textDocument": {"uri": uri},
                                                             "range": {"start": pos, "end": pos},
                                                             "context": {"diagnostics": ctx if com else [],
                                                                         "triggerKind": 1}})
                res = ca.get("result") or []
                c[f"{rot}_{'com' if com else 'sem'}_diag"] = [
                    {"title": a.get("title"), "kind": a.get("kind"), "tem_edit": bool(a.get("edit")),
                     "diagnostics": [d.get("code") for d in a.get("diagnostics") or []] if "diagnostics" in a else None,
                     "forma": "CodeAction" if "kind" in a or "edit" in a else "Command"} for a in res]
        _, ih, _ = s.req("textDocument/inlayHint", {"textDocument": {"uri": uri},
                                                    "range": {"start": {"line": 0, "character": 0},
                                                              "end": {"line": 99, "character": 0}}})
        c["inlay_n"] = len(ih.get("result") or [])
        cli[nome] = c
        s.stop()
    out["cliente"] = cli


# ---------------------------------------------------------------- oráculos do «você quis dizer»

CAMPOS = ["amplitude", "speed", "active", "top_signal", "wave", "direction", "tint", "base_x", "base_y", "t",
          "was_up", "id"]
GLOBAIS = ["print", "pairs", "ipairs", "math", "string", "table", "type", "tostring", "tonumber", "error",
           "assert", "select", "next", "require", "ph2d", "init", "update"]
MANUAIS = {
    "campo": ["ampiltude", "amplitde", "amplitudee", "amplitode", "aplitde", "ampl", "Amplitude", "AMPLITUDE",
              "sped", "sepeed", "seped", "actve", "atcive", "top_sginal", "topsignal", "wav", "waev", "dirction",
              "direcion", "tnit", "base_z", "baes_x", "tt", "i", "was_upp", "xyz", "banana"],
    "global": ["pirnt", "prnt", "priint", "Print", "prit", "paris", "ipars", "mat", "maht", "tostrnig", "tonumbr",
               "p2hd", "updtae", "inti", "errro"],
    "membro": ["proprety", "propery", "gte", "stt", "emti", "colr", "vec", "spwan", "despwan", "state_gte",
               "find_by_nme", "attach"],
}


def sistematicos(nomes):
    """Para cada nome com ≥ 3 letras: troca das letras 1–2, apaga a 1, substitui a 1 por `q`."""
    out = []
    for w in nomes:
        if len(w) < 3:
            continue
        out += [w[0] + w[2] + w[1] + w[3:], w[0] + w[2:], w[0] + "q" + w[2:]]
    return out


def lev(a, b, osa):
    d = [[0] * (len(b) + 1) for _ in range(len(a) + 1)]
    for i in range(len(a) + 1):
        d[i][0] = i
    for j in range(len(b) + 1):
        d[0][j] = j
    for i in range(1, len(a) + 1):
        for j in range(1, len(b) + 1):
            c = 0 if a[i - 1] == b[j - 1] else 1
            d[i][j] = min(d[i - 1][j] + 1, d[i][j - 1] + 1, d[i - 1][j - 1] + c)
            if osa and i > 1 and j > 1 and a[i - 1] == b[j - 2] and a[i - 2] == b[j - 1]:
                d[i][j] = min(d[i][j], d[i - 2][j - 2] + 1)
    return d[-1][-1]


PY = r'''
import json, sys, traceback
casos = json.loads(sys.argv[1])
out = {}
for familia, nomes, typos in casos:
    for t in typos:
        try:
            if familia == "campo":
                class O: pass
                o = O()
                for n in nomes: setattr(o, n, 0)
                getattr(o, t)
            else:
                ns = {n: 0 for n in nomes}
                ns["__builtins__"] = {}
                exec(t, ns)
            out[familia + ":" + t] = None
        except Exception as e:
            msg = "".join(traceback.TracebackException.from_exception(e).format_exception_only()).strip()
            m = __import__("re").search(r"Did you mean:? '([^']+)'", msg)
            out[familia + ":" + t] = m.group(1) if m else None
print(json.dumps(out))
'''


def python_oraculo(casos):
    p = subprocess.run([sys.executable, "-I", "-c", PY, json.dumps(casos)], capture_output=True, text=True)
    return json.loads(p.stdout)


def rustc_oraculo(casos, tmp):
    out = {}
    for familia, nomes, typos in casos:
        src = []
        linha = {}
        if familia == "campo" or familia == "membro":
            src.append("#[allow(dead_code)] struct S { " + ", ".join(f"r#{n}: i32" for n in nomes) + " }")
            src.append("fn main() { let s = S { " + ", ".join(f"r#{n}: 0" for n in nomes) + " };")
            for t in typos:
                linha[len(src) + 1] = t
                src.append(f"  let _ = s.r#{t};")
        else:
            src.append("#[allow(unused_variables)] fn main() {")
            for n in nomes:
                src.append(f"  let r#{n} = 0;")
            for t in typos:
                linha[len(src) + 1] = t
                src.append(f"  let _ = r#{t};")
        src.append("}")
        f = os.path.join(tmp, f"{familia}.rs")
        open(f, "w").write("\n".join(src) + "\n")
        p = subprocess.run(["rustc", "--edition", "2024", "--emit=metadata", "--error-format=json", "-o",
                            os.path.join(tmp, f"{familia}.rmeta"), f], capture_output=True, text=True)
        for l in p.stderr.splitlines():
            try:
                m = json.loads(l)
            except ValueError:
                continue
            if m.get("level") != "error" or not m.get("spans"):
                continue
            t = linha.get(m["spans"][0]["line_start"])
            if t is None:
                continue
            sug = None
            for c in m.get("children", []):
                for sp in c.get("spans", []):
                    if sp.get("suggested_replacement") is not None and "similar name" in c.get("message", ""):
                        sug = sp["suggested_replacement"].removeprefix("r#")
            out[familia + ":" + t] = sug
        for t in typos:
            out.setdefault(familia + ":" + t, None)
    return out


GD = r'''extends SceneTree
func _init():
	var casos = JSON.parse_string(OS.get_cmdline_user_args()[0])
	for a in InputMap.get_actions():
		InputMap.erase_action(a)
	for c in casos:
		var familia = c[0]
		for n in c[1]:
			InputMap.add_action(n)
		for t in c[2]:
			var melhor = ""
			var sim = -1.0
			for n in c[1]:
				var v = t.similarity(n)
				if v > sim:
					sim = v
					melhor = n
			printerr("@@CASO ", familia, ":", t, " ", melhor, " ", sim)
			Input.is_action_pressed(t)
		for n in c[1]:
			InputMap.erase_action(n)
	quit()
'''


def godot_oraculo(casos, tmp):
    d = os.path.join(tmp, "gd")
    os.makedirs(d)
    open(os.path.join(d, "project.godot"), "w").write("")
    open(os.path.join(d, "s.gd"), "w").write(GD)
    env = {**os.environ, "XDG_CONFIG_HOME": os.path.join(tmp, "x"), "XDG_DATA_HOME": os.path.join(tmp, "x"),
           "XDG_CACHE_HOME": os.path.join(tmp, "x")}
    p = subprocess.run(["timeout", "120", "godot", "--headless", "--path", d, "-s", "s.gd", "--", json.dumps(casos)],
                       capture_output=True, text=True, env=env)
    out, cur = {}, None
    for l in p.stderr.splitlines():
        if l.startswith("@@CASO "):
            k, melhor, sim = l[7:].rsplit(" ", 2)
            cur = k
            out[k] = {"sugere": None, "similarity_melhor": melhor, "similarity": float(sim), "erro": None}
        elif cur and "doesn't exist" in l:
            out[cur]["erro"] = l.strip()[:200]
            m = re.search(r'Did you mean "([^"]+)"', l)
            if m:
                out[cur]["sugere"] = m.group(1)
    return out, p.stderr[-1500:] if not out else None


def oraculos_fase(out, membros):
    casos = [["campo", CAMPOS, MANUAIS["campo"] + sistematicos(CAMPOS)],
             ["global", GLOBAIS, MANUAIS["global"] + sistematicos(GLOBAIS)],
             ["membro", membros, MANUAIS["membro"] + sistematicos(membros)]]
    for c in casos:  # sem duplicados nem nomes que já existem
        c[2] = [t for i, t in enumerate(c[2]) if t not in c[2][:i] and t not in c[1]]
    with tempfile.TemporaryDirectory() as tmp:
        py = python_oraculo(casos)
        rs = rustc_oraculo(casos, tmp)
        gd, gd_log = godot_oraculo(casos, tmp)
    linhas = []
    for familia, nomes, typos in casos:
        for t in typos:
            k = familia + ":" + t
            dist = sorted(((lev(t, n, True), lev(t, n, False), n) for n in nomes))
            g = gd.get(k) or {}
            linhas.append({"familia": familia, "typo": t, "osa": dist[0][0], "lev": dist[0][1],
                           "mais_perto_osa": [n for o, _, n in dist if o == dist[0][0]],
                           "python": py.get(k), "rustc": rs.get(k), "godot": g.get("sugere"),
                           "godot_similarity": g.get("similarity"), "godot_similarity_de": g.get("similarity_melhor"),
                           "godot_erro": g.get("erro")})
    out["oraculos"] = {"versoes": {
        "python": sys.version.split()[0],
        "rustc": subprocess.run(["rustc", "--version"], capture_output=True, text=True).stdout.strip(),
        "godot": subprocess.run(["godot", "--version"], capture_output=True, text=True).stdout.strip()},
        "godot_log_se_vazio": gd_log, "casos": linhas}


def main():
    global J
    d = sorted(glob.glob(os.path.join(API, "ph2d-*.d.luau")))
    j = sorted(glob.glob(os.path.join(API, "ph2d-docs-*.json")))
    assert d and j, "corra uma vez o app/oraculo para gerar a API em " + API
    P.DEFS, J = d[-1], j[-1]
    out = {"loadavg": open("/proc/loadavg").read().split()[:3]}
    lsp_fase(out)
    oraculos_fase(out, P.api_members())
    out["loadavg_fim"] = open("/proc/loadavg").read().split()[:3]
    dest = os.path.join(AQUI, "medicoes_onda2", "I7_bruto.json")
    json.dump(out, open(dest, "w"), ensure_ascii=False, indent=1)

    for modo in ("nao_estrito", "strict"):
        m = out["lsp"][modo]
        print(f"== {modo}")
        for dg in m["diagnosticos"]:
            print(f"  diag {dg['code']!s:<5} {dg['linha'][:40]!r:<44} {dg['message'][:70]!r}")
            for a in dg["acoes"]:
                print(f"      · {a['kind']:<24} {a['title']!r} edits={len(a['edits'])}")
        for k, v in m["no_cursor"].items():
            print(f"  cursor {k:<18} codes={v['codes_no_contexto']} -> {[(a['kind'], a['title'], len(a['edits'])) for a in v['acoes']]}")
        print("  resolve:", json.dumps(m["resolve"], ensure_ascii=False)[:600])
        for k, v in m["completion"].items():
            print(f"  completion {k:<34} n={v['count']:<3} incompleto={v['isIncomplete']} alvo={v['tem_o_alvo']} {v['alvo_item'] and v['alvo_item'].get('textEdit')}")
        print(f"  inlay: {m['inlay_tipos']['n_total']} dicas, {len(m['inlay_tipos']['tipos'])} de tipo; 1.ªs:",
              json.dumps(m["inlay_tipos"]["tipos"][:3], ensure_ascii=False)[:500])
    for nome, c in out["cliente"].items():
        print(f"== cliente {nome}: inlay {c['inlay_n']}")
        for k, v in c.items():
            if k != "inlay_n":
                print(f"  {k:<16} {[(a['kind'], a['title'], a['tem_edit'], a['diagnostics']) for a in v]}")
    o = out["oraculos"]
    print("== oráculos", o["versoes"])
    if o["godot_log_se_vazio"]:
        print("  godot sem saída:", o["godot_log_se_vazio"])
    print(f"  {'familia':<7} {'typo':<14} osa lev {'perto':<22} {'python':<12} {'rustc':<12} {'godot':<12} sim")
    for c in o["casos"]:
        print(f"  {c['familia']:<7} {c['typo']:<14} {c['osa']:>3} {c['lev']:>3} {','.join(c['mais_perto_osa'])[:22]:<22} "
              f"{c['python'] or '-':<12} {c['rustc'] or '-':<12} {c['godot'] or '-':<12} {c['godot_similarity']}")
    print("loadavg", out["loadavg"], "->", out["loadavg_fim"])


if __name__ == "__main__":
    main()
