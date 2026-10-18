"""Sonda do I10 (Onda 2 do CodeEditor): rename e referências do `luau-lsp` 1.70.1 sobre os NOSSOS scripts.

Lançado como o app (a API gerada em `<cache>/ph2d/luau-lsp/1.70.1/api/`). Para cada sítio do `bob.luau`
(local, parâmetro, `local function`, campo do `self`, o literal de uma `ph2d.property`, membro da API,
gancho do motor, global do script, sombra): `textDocument/references` (com e sem a declaração),
`textDocument/rename` para um nome válido, para uma palavra reservada, para um nome inválido, para um
nome que já existe no âmbito e para o próprio nome; `prepareRename` e `documentHighlight`; a forma da
resposta (`changes` × `documentChanges`); UTF-16 numa linha com emoji; o tempo a 51 e a 7 500 linhas.
O servidor MORRE em alguns pedidos (medido): cada morte fica registada no sítio e ele reinicia-se.

Uso (da raiz da worktree):  python3 -I docs/CodeEditor/ferramentas/lspprobe_i10.py
Grava docs/CodeEditor/ferramentas/medicoes_onda2/I10_bruto.json e imprime a tabela.
"""
import glob, importlib.util, json, os, statistics

AQUI = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("lspprobe2", os.path.join(AQUI, "lspprobe2.py"))
P = importlib.util.module_from_spec(spec)
spec.loader.exec_module(P)

API = os.path.expanduser("~/.cache/ph2d/luau-lsp/1.70.1/api")
ARGS = []

EMOJI = '  local d = s * self.amplitude * brilho\n'
EMOJI_NOVO = '  local tag = "🎈"; local d = s * self.amplitude * brilho\n'

# (rótulo, agulha, deslocamento) — o cursor fica no início da agulha + deslocamento
SITIOS = [
    ("local_brilho_decl", "local brilho", 6),
    ("local_brilho_uso", "* brilho", 2),
    ("local_s", "local s\n", 6),
    ("param_dt_decl", "update(self, dt)", 13),
    ("param_dt_uso", "+ dt", 2),
    ("param_self", "update(self, dt)", 7),
    ("self_campo_amplitude", "self.amplitude", 5),
    ("self_campo_t", "self.t = 0", 5),
    ("literal_property", 'ph2d.property("amplitude"', 15),
    ("api_membro_set", "ph2d.set(", 5),
    ("api_global_ph2d", "ph2d.set(", 0),
    ("stdlib_math", "math.sin", 0),
    ("stdlib_membro_sin", "math.sin", 5),
    ("gancho_update", "function update", 9),
    ("global_do_script_decl", "function ajuda", 9),
    ("global_do_script_uso", "ajuda(x)", 0),
    ("local_function", "local function com_sombra", 15),
    ("sombra_interna", "local x = 2", 6),
    ("sombra_externa", "local x = 1", 6),
    ("emoji_d", "local d = s", 6),
]

EXTRA = (
    "\nfunction ajuda(v)\n  return v * 2\nend\n"
    "local function com_sombra()\n  local x = 1\n  do\n    local x = 2\n    print(x)\n  end\n  return x + ajuda(x)\nend\n"
)

NOMES = (("valido", "renomeado"), ("reservada", "end"), ("invalido", "1x-y"), ("existente", None), ("igual", None))


def texto(n=None):
    b = P.juntado(n) if n else P.troca(P.bob(), EMOJI, EMOJI_NOVO)
    return b + EXTRA


def edicoes(res):
    if not isinstance(res, dict):
        return None
    out = {"forma": [k for k in ("changes", "documentChanges") if k in res]}
    lst = []
    for uri, es in (res.get("changes") or {}).items():
        lst += [(uri, e) for e in es]
    for dc in res.get("documentChanges") or []:
        lst += [(dc["textDocument"]["uri"], e) for e in dc.get("edits", [])]
    out["n"] = len(lst)
    out["uris"] = sorted({u for u, _ in lst})
    out["edits"] = [(e["range"]["start"]["line"], e["range"]["start"]["character"], e["range"]["end"]["character"],
                     e["newText"]) for _, e in lst]
    return out


def lugares(res):
    if not isinstance(res, list):
        return res
    return [(r["range"]["start"]["line"], r["range"]["start"]["character"], r["range"]["end"]["character"])
            for r in res]


def palavra(linha, c):
    i = c
    while i > 0 and (linha[i - 1].isalnum() or linha[i - 1] == "_"):
        i -= 1
    j = c
    while j < len(linha) and (linha[j].isalnum() or linha[j] == "_"):
        j += 1
    return linha[i:j]


class Vivo:
    """O servidor reinicia-se quando morre; cada morte fica registada no pedido que a provocou."""

    def __init__(self, uri, text):
        self.uri, self.text, self.mortes, self.s = uri, text, [], None
        self.liga()

    def liga(self):
        self.s = P.Lsp(P.BASE, extra_args=ARGS)
        self.s.open(self.uri, self.text)
        P.diags(self.s, self.uri)

    def req(self, m, params, rot):
        try:
            ms, b, n = self.s.req(m, params, timeout=20)
        except BrokenPipeError:
            ms, b, n = None, {"error": "TIMEOUT"}, 0
        if self.s.p.poll() is not None or b.get("error") == "TIMEOUT":
            self.s.p.kill()
            code = self.s.p.wait()
            self.mortes.append({"sitio": rot, "pedido": m, "codigo": code})
            self.liga()
            return None, {"error": {"morreu": code}}, 0
        return ms, b, n


def fase(v, text):
    uri = v.uri
    linhas = text.splitlines()
    r = {}
    for rot, agulha, off in SITIOS:
        pos = P.pos_de(text, agulha, off)
        w = palavra(linhas[pos["line"]], pos["character"])
        m = {"pos": pos, "palavra": w, "linha": linhas[pos["line"]].strip()[:60]}
        for decl in (True, False):
            ms, b, _ = v.req("textDocument/references",
                             {**P.at(uri, pos), "context": {"includeDeclaration": decl}}, rot)
            m[f"refs_decl_{decl}"] = {"ms": ms, "lugares": lugares(b.get("result")), "erro": b.get("error")}
        for nome, novo in NOMES:
            alvo = novo or (w if nome == "igual" else ("brilho" if w != "brilho" else "s"))
            ms, b, _ = v.req("textDocument/rename", {**P.at(uri, pos), "newName": alvo}, rot)
            m[f"rename_{nome}"] = {"ms": ms, "novo": alvo, "res": edicoes(b.get("result")), "erro": b.get("error")}
        _, b, _ = v.req("textDocument/prepareRename", P.at(uri, pos), rot)
        m["prepareRename"] = b.get("error") or b.get("result")
        _, b, _ = v.req("textDocument/documentHighlight", P.at(uri, pos), rot)
        m["documentHighlight"] = b.get("error") or b.get("result")
        r[rot] = m
    return r


def tempos(n):
    text = texto(n)
    v = Vivo(f"file:///ph2d-project/i10_{n}.luau", text)
    out = {}
    for rot, req, agulha, extra in (
            ("refs_local", "textDocument/references", "local brilho", {"context": {"includeDeclaration": True}}),
            ("rename_local", "textDocument/rename", "local brilho", {"newName": "renomeado"}),
            ("rename_param_self", "textDocument/rename", "update(self", {"newName": "renomeado"})):
        p = P.pos_de(text, agulha, 6 if agulha == "local brilho" else 7)
        ms = []
        for _ in range(P.REPS):
            t, b, _ = v.req(req, {**P.at(v.uri, p), **extra}, rot)
            ms.append(t)
        res = b.get("result")
        out[rot] = {"p50": statistics.median(ms), "max": max(ms),
                    "n": len(res) if isinstance(res, list) else (edicoes(res) or {}).get("n")}
    out["mortes"] = v.mortes
    v.s.stop()
    return out


def main():
    global ARGS
    d = sorted(glob.glob(os.path.join(API, "ph2d-*.d.luau")))
    j = sorted(glob.glob(os.path.join(API, "ph2d-docs-*.json")))
    assert d and j, "corra uma vez o app/oraculo para gerar a API em " + API
    P.DEFS, ARGS = d[-1], [f"--docs={j[-1]}"]
    out = {"loadavg": open("/proc/loadavg").read().split()[:3]}
    text = texto()
    v = Vivo("file:///ph2d-project/i10_bob.luau", text)
    caps = v.s.init["capabilities"]
    out["caps"] = {k: caps.get(k) for k in ("renameProvider", "referencesProvider", "documentHighlightProvider",
                                            "linkedEditingRangeProvider")}
    out["args"] = v.s.args
    out["diags"] = [(dg.get("code"), dg["message"][:80]) for dg in P.diags(v.s, v.uri)]
    out["bob"] = fase(v, text)
    # Dois documentos abertos: o rename de um parâmetro nunca toca no outro.
    uri2 = "file:///ph2d-project/i10_outro.luau"
    v.s.open(uri2, "function update(self, dt)\n  local brilho = self.amplitude\n  print(brilho)\nend\n")
    _, b, _ = v.req("textDocument/rename", {**P.at(v.uri, P.pos_de(text, "update(self", 7)), "newName": "eu"}, "dois")
    out["dois_documentos_self"] = edicoes(b.get("result"))
    out["mortes"] = v.mortes
    v.s.stop()
    out["tempos"] = {str(n): tempos(n) for n in (None, 7500)}
    out["loadavg_fim"] = open("/proc/loadavg").read().split()[:3]
    dest = os.path.join(AQUI, "medicoes_onda2", "I10_bruto.json")
    json.dump(out, open(dest, "w"), ensure_ascii=False, indent=1)

    print("caps", out["caps"], "loadavg", out["loadavg"], out["loadavg_fim"])
    print("diags", out["diags"])
    for rot, m in out["bob"].items():
        def n(k):
            x = m[k]
            return (x["res"] or {}).get("n") if not x["erro"] else ("ERR " + str(x["erro"].get("message", x["erro"]))[:30])
        refs = m["refs_decl_True"]
        print(f"{rot:<22} {m['palavra']:<10} refs={len(refs['lugares']) if isinstance(refs['lugares'], list) else refs['lugares'] or refs['erro']}"
              f"/{len(m['refs_decl_False']['lugares']) if isinstance(m['refs_decl_False']['lugares'], list) else '-'}"
              f" | ok:{n('rename_valido')} | end:{n('rename_reservada')} | 1x-y:{n('rename_invalido')}"
              f" | existe({m['rename_existente']['novo']}):{n('rename_existente')} | igual:{n('rename_igual')}"
              f" | prep={str(m['prepareRename'])[:30]} | hl={str(m['documentHighlight'])[:30]}")
    print("mortes", out["mortes"])
    print("dois_documentos_self", out["dois_documentos_self"])
    print("tempos", json.dumps(out["tempos"]))


if __name__ == "__main__":
    main()
