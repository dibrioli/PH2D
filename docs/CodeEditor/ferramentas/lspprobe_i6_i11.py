"""Sonda do I6/I11 (Onda 2 do CodeEditor): `definition` e `semanticTokens/full` do luau-lsp 1.70.1
sobre o bob + linhas nossas, lançado COMO O APP (a API gerada em `<cache>/ph2d/luau-lsp/1.70.1/api/`,
`--definitions:@ph2d=` + `--docs=`), e também como o I0b (`runtime/luau/ph2d.d.luau`, sem docs).

Uso (da raiz da worktree):  python3 -I docs/CodeEditor/ferramentas/lspprobe_i6_i11.py
Grava docs/CodeEditor/ferramentas/medicoes_onda2/I6_I11_bruto.json e imprime a tabela.
"""
import glob, importlib.util, json, os, sys

AQUI = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("lspprobe2", os.path.join(AQUI, "lspprobe2.py"))
P = importlib.util.module_from_spec(spec)
spec.loader.exec_module(P)

API = os.path.expanduser("~/.cache/ph2d/luau-lsp/1.70.1/api")
EXTRA = """
print(math.sin(1))
pirnt(1)
local function ajuda(x) return x end
local quieto = ajuda(2)
ph2d.proprety("x", 1)
print(quieto, self, ph2d.set)
"""
QUEBRADO = "if self.active then\n  local z = ph2d.get(self.id, \"x\")\n  pirnt(z\n"


def decode(text, data, legend):
    lines = text.split("\n")
    out, line, col = [], 0, 0
    for i in range(0, len(data), 5):
        dl, ds, n, ty, mods = data[i:i + 5]
        line += dl
        col = ds if dl else col + ds
        u16 = lines[line].encode("utf-16-le")
        word = u16[col * 2:(col + n) * 2].decode("utf-16-le")
        ms = [legend["tokenModifiers"][b] for b in range(len(legend["tokenModifiers"])) if mods >> b & 1]
        out.append({"line": line, "col": col, "len": n, "text": word,
                    "type": legend["tokenTypes"][ty], "mods": ms})
    return out


def corre(nome, extra_args):
    s = P.Lsp(P.BASE, extra_args=extra_args)
    legend = s.init["capabilities"]["semanticTokensProvider"]["legend"]
    r = {"args": s.args}
    text = P.bob() + EXTRA
    uri = "file:///ph2d-project/sonda.luau"
    s.open(uri, text)
    td = {"textDocument": {"uri": uri}}
    ms, b, n = s.req("textDocument/semanticTokens/full", td)
    toks = decode(text, b["result"]["data"], legend)
    r["tokens"] = {"ms": ms, "bytes": n, "result_keys": sorted(b["result"].keys()), "tokens": toks}
    defs = {}
    for rotulo, agulha, off, nth in [
        ("local_brilho_uso", "* brilho", 2, 0),
        ("ph2d.set", "ph2d.set(self.id, \"x\"", 5, 0),
        ("ph2d_tabela", "ph2d.set(self.id, \"x\"", 0, 0),
        ("self.t_leitura", "self.t + dt", 5, 0),
        ("self.amplitude", "self.amplitude", 5, 0),
        ("dt_param_uso", "self.t + dt", 9, 0),
        ("print", "print(math", 0, 0),
        ("pirnt", "pirnt(1)", 0, 0),
        ("math.sin", "print(math.sin", 11, 0),
        ("ajuda_uso", "= ajuda(2)", 2, 0),
        ("quieto_uso", "print(quieto", 6, 0),
        ("ph2d.proprety", "ph2d.proprety", 5, 0),
    ]:
        pos = P.pos_de(text, agulha, off, nth)
        ms, c, _ = s.req("textDocument/definition", {"textDocument": {"uri": uri}, "position": pos})
        defs[rotulo] = {"ms": ms, "pos": pos, "result": c.get("result", c.get("error"))}
    r["definition"] = defs
    # Texto partido a meio de uma escrita: o servidor ainda dá tokens?
    uri2 = "file:///ph2d-project/partido.luau"
    s.open(uri2, QUEBRADO)
    ms, b, n = s.req("textDocument/semanticTokens/full", {"textDocument": {"uri": uri2}})
    r["partido"] = {"ms": ms, "raw": b.get("result", b.get("error")),
                    "tokens": decode(QUEBRADO, b["result"]["data"], legend) if "result" in b else None}
    # Versão: a resposta diz a versão? (não há `version` no pedido do full)
    s.change(uri, text.replace("pirnt(1)", "pirnt(2)"))
    ms, b, n = s.req("textDocument/semanticTokens/full", td)
    r["depois_de_editar"] = {"ms": ms, "result_keys": sorted(b["result"].keys()),
                             "n": len(b["result"]["data"]) // 5}
    s.stop()
    return r


def main():
    d = sorted(glob.glob(os.path.join(API, "ph2d-*.d.luau")))
    j = sorted(glob.glob(os.path.join(API, "ph2d-docs-*.json")))
    assert d and j, "corra uma vez o app/oraculo para gerar a API em " + API
    out = {"loadavg": open("/proc/loadavg").read().split()[:3]}
    # O app passa as definições geradas NO LUGAR da runtime/luau; o modo «app» troca o DEFS do módulo.
    P.DEFS = d[-1]
    out["app"] = corre("app", [f"--docs={j[-1]}"])
    P.DEFS = os.path.join(P.RAIZ, "runtime/luau/ph2d.d.luau")
    out["i0b"] = corre("i0b", [])
    out["loadavg_fim"] = open("/proc/loadavg").read().split()[:3]
    dest = os.path.join(AQUI, "medicoes_onda2", "I6_I11_bruto.json")
    json.dump(out, open(dest, "w"), ensure_ascii=False, indent=1)
    for modo in ("app", "i0b"):
        r = out[modo]
        print(f"== {modo}: tokens {len(r['tokens']['tokens'])} em {r['tokens']['ms']:.1f} ms, chaves {r['tokens']['result_keys']}")
        base = P.bob().count("\n")
        for t in r["tokens"]["tokens"]:
            if t["line"] >= base - 30:
                print(f"  L{t['line']:>3}:{t['col']:<3} {t['text']!r:<16} {t['type']:<10} {','.join(t['mods'])}")
        for k, v in r["definition"].items():
            print(f"  def {k:<16} -> {json.dumps(v['result'])[:160]}")
        print("  partido:", [(t["text"], t["type"], t["mods"]) for t in (r["partido"]["tokens"] or [])])
        print("  depois de editar:", r["depois_de_editar"])
    print("loadavg", out["loadavg"], "->", out["loadavg_fim"])


if __name__ == "__main__":
    main()
