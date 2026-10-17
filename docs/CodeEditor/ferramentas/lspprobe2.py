"""Sonda I0b da Onda 2 do CodeEditor: o que o luau-lsp 1.70.1 RESPONDE sobre as NOSSAS entradas.

Uso (da raiz da worktree):
    python3 -I docs/CodeEditor/ferramentas/lspprobe2.py [saida_dir]
Grava em docs/CodeEditor/ferramentas/medicoes_onda2/:
    caps_luau_lsp_1.70.1.json   capacidades do `initialize` (params iguais aos do cliente Rust)
    I0b_bruto.json              todas as respostas cruas (a fonte do I0b_luau_lsp_respostas.md)
Espelha crates/ph2d-code-lsp: `lsp --stdio --definitions:@ph2d=runtime/luau/ph2d.d.luau`, rootUri
null, configuração `{"platform":{"type":"standard"},"fflags":{"sync":false}}` (+ o que cada fase
acrescenta, dito no bruto). Documentos em memória: file:///ph2d-project/<nome>.luau.
Só stdlib. A parte Godot (§8) corre `godot --headless` com XDG_* temporários (não toca o ~/.config).
"""
import json, os, re, statistics, subprocess, sys, tempfile, threading, time

RAIZ = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
LSP = os.path.expanduser("~/.cache/ph2d/luau-lsp/1.70.1/luau-lsp")
DEFS = os.path.join(RAIZ, "runtime/luau/ph2d.d.luau")
BOB = os.path.join(RAIZ, "crates/ph2d-code-syntax/tests/it/fixtures/bob.luau")
SAIDA = sys.argv[1] if len(sys.argv) > 1 else os.path.join(RAIZ, "docs/CodeEditor/ferramentas/medicoes_onda2")
REPS = 20
BASE = {"platform": {"type": "standard"}, "fflags": {"sync": False}}


def initialize_params(extended):
    """O `initialize_params()` de crates/ph2d-code-lsp/src/client/json.rs; `extended` acrescenta as
    capacidades de cliente das features que a Onda 2 quer (para ver se o servidor muda algo)."""
    td = {
        "synchronization": {"didSave": False},
        "publishDiagnostics": {},
        "completion": {"completionItem": {"snippetSupport": False}},
        "hover": {"contentFormat": ["markdown", "plaintext"]},
    }
    if extended:
        td["completion"] = {"completionItem": {"snippetSupport": False, "labelDetailsSupport": True,
                                               "documentationFormat": ["markdown", "plaintext"],
                                               "resolveSupport": {"properties": ["documentation", "detail"]}}}
        td["signatureHelp"] = {"signatureInformation": {"documentationFormat": ["markdown", "plaintext"],
                                                        "parameterInformation": {"labelOffsetSupport": True},
                                                        "activeParameterSupport": True}}
        td["codeAction"] = {"codeActionLiteralSupport": {"codeActionKind": {"valueSet": [
            "", "quickfix", "refactor", "refactor.extract", "refactor.inline", "refactor.rewrite",
            "source", "source.organizeImports", "source.fixAll"]}}, "isPreferredSupport": True}
        td["documentSymbol"] = {"hierarchicalDocumentSymbolSupport": True}
        td["inlayHint"] = {}
        td["semanticTokens"] = {"requests": {"full": True, "range": True}, "tokenTypes": [], "tokenModifiers": [],
                                "formats": ["relative"]}
        td["diagnostic"] = {}
        td["rename"] = {"prepareSupport": True}
    return {"processId": os.getpid(), "rootUri": None,
            "capabilities": {"general": {"positionEncodings": ["utf-16"]},
                             "workspace": {"configuration": True}, "textDocument": td},
            "initializationOptions": {}}


class Lsp:
    def __init__(self, settings, extra_args=(), extended=True):
        args = [LSP, "lsp", "--stdio", f"--definitions:@ph2d={DEFS}", *extra_args]
        self.args = args
        self.settings = settings
        self.p = subprocess.Popen(args, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.msgs, self.notes, self.config_asks = {}, [], []
        self.cv = threading.Condition()
        self.nid = 0
        self.stderr = []
        threading.Thread(target=self._reader, daemon=True).start()
        threading.Thread(target=lambda: [self.stderr.append(l.decode(errors="replace").rstrip())
                                         for l in self.p.stderr], daemon=True).start()
        _, b, _ = self.req("initialize", initialize_params(extended))
        self.init = b.get("result")
        self.notify("initialized", {})
        self.version = {}

    def _send(self, o):
        b = json.dumps(o).encode()
        self.p.stdin.write(b"Content-Length: %d\r\n\r\n" % len(b) + b)
        self.p.stdin.flush()

    def _reader(self):
        f = self.p.stdout
        while True:
            h = {}
            while True:
                l = f.readline()
                if not l:
                    return
                l = l.decode().strip()
                if not l:
                    break
                k, v = l.split(":", 1)
                h[k.lower()] = v.strip()
            raw = f.read(int(h["content-length"]))
            t = time.perf_counter()
            body = json.loads(raw)
            with self.cv:
                if "id" in body and "method" not in body:
                    self.msgs[body["id"]] = (t, body, len(raw))
                elif "id" in body:
                    res = None
                    if body["method"] == "workspace/configuration":
                        items = body["params"]["items"]
                        self.config_asks.append(items)
                        res = [self.settings if it.get("section") == "luau-lsp" else None for it in items]
                    self._send({"jsonrpc": "2.0", "id": body["id"], "result": res})
                else:
                    self.notes.append(body)
                self.cv.notify_all()

    def notify(self, m, params):
        self._send({"jsonrpc": "2.0", "method": m, "params": params})

    def req(self, m, params, timeout=120):
        self.nid += 1
        i = self.nid
        t = time.perf_counter()
        self._send({"jsonrpc": "2.0", "id": i, "method": m, "params": params})
        with self.cv:
            ok = self.cv.wait_for(lambda: i in self.msgs, timeout=timeout)
        if not ok:
            return None, {"error": "TIMEOUT"}, 0
        t2, b, n = self.msgs.pop(i)
        return (t2 - t) * 1000, b, n

    def open(self, uri, text):
        self.version[uri] = 1
        self.notify("textDocument/didOpen", {"textDocument": {"uri": uri, "languageId": "luau", "version": 1, "text": text}})

    def change(self, uri, text):
        self.version[uri] += 1
        self.notify("textDocument/didChange", {"textDocument": {"uri": uri, "version": self.version[uri]},
                                               "contentChanges": [{"text": text}]})

    def close(self, uri):
        self.notify("textDocument/didClose", {"textDocument": {"uri": uri}})

    def stop(self):
        try:
            self.req("shutdown", None, timeout=10)
            self.notify("exit", None)
            self.p.wait(5)
        except Exception:
            self.p.kill()


# ---------------------------------------------------------------- textos

def bob():
    return "".join(l + "\n" for l in open(BOB, encoding="utf-8").read().splitlines()[2:])


def juntado(n):
    """O `juntado` de crates/ph2d-app-code/tests/it/bancada.rs."""
    ls = bob().splitlines()
    out = []
    for i in range(n):
        l = ls[i % len(ls)]
        out.append(("-- " + l if i >= len(ls) and l.startswith("ph2d.property(") else l) + "\n")
    return "".join(out)


def troca(text, old, new):
    assert text.count(old) == 1, (old, text.count(old))
    return text.replace(old, new)


def marca(text):
    """`|` marca o cursor; devolve (texto sem a marca, posição LSP em UTF-16)."""
    assert text.count("|") == 1
    i = text.index("|")
    pre = text[:i]
    line = pre.count("\n")
    col = pre[pre.rfind("\n") + 1:]
    return text[:i] + text[i + 1:], {"line": line, "character": len(col.encode("utf-16-le")) // 2}


def pos_de(text, needle, offset=0, nth=0):
    i = -1
    for _ in range(nth + 1):
        i = text.index(needle, i + 1)
    i += offset
    pre = text[:i]
    col = pre[pre.rfind("\n") + 1:]
    return {"line": pre.count("\n"), "character": len(col.encode("utf-16-le")) // 2}


def api_members():
    d = open(DEFS, encoding="utf-8").read()
    blk = d[d.index("declare ph2d: {"):]
    blk = blk[:blk.index("\n}")]
    return re.findall(r"^\s{4}(\w+):", blk, re.M)


def td(uri):
    return {"textDocument": {"uri": uri}}


def at(uri, pos):
    return {"textDocument": {"uri": uri}, "position": pos}


def diags(s, uri):
    _, b, _ = s.req("textDocument/diagnostic", td(uri))
    r = b.get("result") or {}
    return r.get("items", []) if isinstance(r, dict) else b


def resumo_acao(a):
    out = {"title": a.get("title"), "kind": a.get("kind"), "isPreferred": a.get("isPreferred")}
    ed = a.get("edit") or {}
    ch = ed.get("changes") or {}
    edits = [e for v in ch.values() for e in v]
    for dc in ed.get("documentChanges") or []:
        edits += dc.get("edits", [])
    out["edits"] = [{"range": e["range"], "newText": e["newText"]} for e in edits]
    if a.get("command"):
        out["command"] = a["command"]
    return out


# ---------------------------------------------------------------- fases

def casos_typo():
    b = bob()
    return {
        "a_global_desconhecida_pirnt": troca(b, "  self.t = self.t + dt\n", "  self.t = self.t + dt\n  pirnt(\"x\")\n"),
        "b1_membro_api_proprety": troca(b, 'ph2d.property("speed"', 'ph2d.proprety("speed"'),
        "b2_membro_api_gte": troca(b, 'self.base_x = ph2d.get(', 'self.base_x = ph2d.gte('),
        "c_self_ampiltude": troca(b, "s * self.amplitude * brilho", "s * self.ampiltude * brilho"),
        "d1_set_valor_string": troca(b, 'ph2d.set(self.id, "x", self.base_x + d * self.direction.x)',
                                     'ph2d.set(self.id, "x", "fast")'),
        "d2_set_campo_fora_da_uniao": troca(b, 'ph2d.set(self.id, "x", self.base_x', 'ph2d.set(self.id, "z", self.base_x'),
        "d3_vec2_com_string": troca(b, "ph2d.vec2(0, 1)", 'ph2d.vec2(0, "1")'),
        "e1_if_igual_simples": troca(b, 'if self.wave == "square" then', 'if self.wave = "square" then'),
        "e2_falta_end": troca(b, "    ph2d.emit(self.top_signal)\n  end\n", "    ph2d.emit(self.top_signal)\n"),
        "e3_if_x_igual_1": troca(b, "  local s\n", "  local s\n  local x = 0\n  if x = 1 then x = 2 end\n"),
        "f_chamar_numero": troca(b, "  local s\n", "  local s\n  local k = 5\n  k()\n"),
    }


def fase_typos(s, out):
    res = {}
    for nome, txt in casos_typo().items():
        res[nome] = {}
        for modo, pref in (("nao_estrito", ""), ("strict", "--!strict\n")):
            uri = f"file:///ph2d-project/typo_{nome}_{modo}.luau"
            s.open(uri, pref + txt)
            ds = diags(s, uri)
            linhas = (pref + txt).splitlines()
            reg = []
            for d in ds:
                _, ca, _ = s.req("textDocument/codeAction", {"textDocument": {"uri": uri}, "range": d["range"],
                                                             "context": {"diagnostics": [d], "triggerKind": 1}})
                acoes = ca.get("result") or []
                reg.append({"code": d.get("code"), "codeDescription": d.get("codeDescription"),
                            "severity": d.get("severity"), "source": d.get("source"), "tags": d.get("tags"),
                            "message": d["message"], "range": d["range"],
                            "texto_da_linha": linhas[d["range"]["start"]["line"]].strip()
                            if d["range"]["start"]["line"] < len(linhas) else None,
                            "codeActions": [resumo_acao(a) for a in acoes] if isinstance(acoes, list) else acoes})
            res[nome][modo] = reg
            s.close(uri)
    out["typos"] = res


def fase_completion(s, out, chave="completion"):
    r = {}
    b = bob()
    t, p = marca(troca(b, "  self.was_up = up\n", "  self.was_up = up\n  ph2d.|\n"))
    uri = "file:///ph2d-project/compl_ponto.luau"
    s.open(uri, t)
    diags(s, uri)
    ms, c, n = s.req("textDocument/completion", at(uri, p))
    res = c.get("result")
    items = res["items"] if isinstance(res, dict) else (res or [])
    r["apos_ph2d_ponto"] = {"ms": ms, "bytes": n, "isIncomplete": res.get("isIncomplete") if isinstance(res, dict) else None,
                            "count": len(items), "items": items}
    decl = api_members()
    labels = [i["label"] for i in items]
    r["comparacao_d_luau"] = {"declarados": decl, "so_no_lsp": sorted(set(labels) - set(decl)),
                              "so_no_d_luau": sorted(set(decl) - set(labels))}
    if s.init.get("capabilities", {}).get("completionProvider", {}).get("resolveProvider") and items:
        it = next((i for i in items if i["label"] == "set"), items[0])
        _, rr, _ = s.req("completionItem/resolve", it)
        r["resolve_set"] = rr.get("result")
    s.close(uri)
    for nome, alvo in (("arg1_apos_parentese", "  ph2d.set(|)\n"),
                       ("arg2_string_poseField", '  ph2d.set(self.id, "|")\n'),
                       ("arg3_numero", '  ph2d.set(self.id, "x", |)\n')):
        t, p = marca(troca(b, "  self.was_up = up\n", "  self.was_up = up\n" + alvo))
        uri = f"file:///ph2d-project/compl_{nome}.luau"
        s.open(uri, t)
        diags(s, uri)
        ms, c, n = s.req("textDocument/completion", at(uri, p))
        res = c.get("result")
        items = res["items"] if isinstance(res, dict) else (res or [])
        ordenados = sorted(items, key=lambda i: (i.get("sortText") or i["label"]))
        r[nome] = {"ms": ms, "bytes": n, "count": len(items),
                   "sortTexts_distintos": sorted({i.get("sortText") for i in items}, key=str),
                   "primeiros_25": [{k: i.get(k) for k in ("label", "kind", "detail", "sortText", "labelDetails", "tags",
                                                            "insertText", "textEdit")} for i in ordenados[:25]]}
        s.close(uri)
    out[chave] = r


def fase_sig_hover_def(s, out, chave="sig_hover"):
    r = {}
    b = bob()
    for nome, alvo in (("ph2d_set_arg2", "  ph2d.set(self.id, |)\n"), ("ph2d_set_arg1", "  ph2d.set(|)\n"),
                       ("ph2d_property_arg3", '  ph2d.property("k", 1, |)\n'), ("ph2d_get_arg2", "  ph2d.get(self.id, |)\n")):
        t, p = marca(troca(b, "  self.was_up = up\n", "  self.was_up = up\n" + alvo))
        uri = f"file:///ph2d-project/sig_{nome}.luau"
        s.open(uri, t)
        ms, c, n = s.req("textDocument/signatureHelp", {**at(uri, p), "context": {"triggerKind": 2, "triggerCharacter": ",", "isRetrigger": False}})
        r["signatureHelp_" + nome] = {"ms": ms, "result": c.get("result"), "error": c.get("error")}
        s.close(uri)
    uri = "file:///ph2d-project/bob.luau"
    s.open(uri, b)
    diags(s, uri)
    for nome, pos in (("ph2d_set", pos_de(b, "ph2d.set(", 5)), ("ph2d_tabela", pos_de(b, "ph2d.set(", 1)),
                      ("local_brilho_uso", pos_de(b, "* brilho", 2)), ("local_d_decl", pos_de(b, "local d =", 6)),
                      ("self_amplitude", pos_de(b, "self.amplitude", 6)), ("funcao_update", pos_de(b, "function update", 10)),
                      ("param_dt", pos_de(b, "self.t + dt", 9))):
        ms, c, n = s.req("textDocument/hover", at(uri, pos))
        r["hover_" + nome] = {"ms": ms, "pos": pos, "result": c.get("result")}
    for nome, pos in (("local_brilho_uso", pos_de(b, "* brilho", 2)), ("ph2d_set", pos_de(b, "ph2d.set(", 5)),
                      ("self_t", pos_de(b, "self.t = self.t + dt", 13))):
        ms, c, _ = s.req("textDocument/definition", at(uri, pos))
        r["definition_" + nome] = {"ms": ms, "pos": pos, "result": c.get("result")}
    s.close(uri)
    out[chave] = r


def fase_inlay_sem_sym(s, out):
    r = {}
    b = bob()
    docs = {"bob51": b, "grande3000": juntado(3000), "grande7500": juntado(7500)}
    uris = {k: f"file:///ph2d-project/{k}.luau" for k in docs}
    linhas = {k: v.count("\n") for k, v in docs.items()}
    for k, v in docs.items():
        s.open(uris[k], v)
        t0 = time.perf_counter()
        diags(s, uris[k])
        r.setdefault("primeiro_check_ms", {})[k] = (time.perf_counter() - t0) * 1000

    def pedidos(uri, n):
        return {
            "inlayHint": ("textDocument/inlayHint", {"textDocument": {"uri": uri},
                                                     "range": {"start": {"line": 0, "character": 0}, "end": {"line": n + 1, "character": 0}}}),
            # só a janela visível (~50 linhas a meio do ficheiro)
            "inlayHint_janela50": ("textDocument/inlayHint", {"textDocument": {"uri": uri},
                                                             "range": {"start": {"line": max(0, n // 2 - 25), "character": 0},
                                                                       "end": {"line": max(0, n // 2 - 25) + 50, "character": 0}}}),
            "semanticTokens/full": ("textDocument/semanticTokens/full", td(uri)),
            "documentSymbol": ("textDocument/documentSymbol", td(uri)),
        }
    # amostra das respostas (bob)
    for m, (meth, par) in pedidos(uris["bob51"], linhas["bob51"]).items():
        _, c, n = s.req(meth, par)
        r["amostra_" + m] = {"bytes": n, "result": c.get("result"), "error": c.get("error")}
    # tempos: QUENTE (sem edição) e APÓS EDIÇÃO (didChange numa linha de comentário, depois o pedido)
    tempos = {modo: {k: {m: [] for m in ("inlayHint", "inlayHint_janela50", "semanticTokens/full", "documentSymbol")} for k in docs}
              for modo in ("quente", "apos_edicao")}
    tamanhos = {k: {} for k in docs}
    alt = {k: [docs[k], docs[k].replace("-- bob.luau — sobe e desce.", "-- bob.luau — sobe e desce!", 1)] for k in docs}
    for k in docs:
        assert alt[k][0] != alt[k][1]
    estado = {k: 0 for k in docs}
    for rep in range(REPS):
        for modo in ("quente", "apos_edicao"):
            for k in docs:
                for m, (meth, par) in pedidos(uris[k], linhas[k]).items():
                    if modo == "apos_edicao":
                        estado[k] ^= 1
                        s.change(uris[k], alt[k][estado[k]])
                    ms, c, n = s.req(meth, par)
                    assert "error" not in c, c
                    tempos[modo][k][m].append(ms)
                    tamanhos[k][m] = n
    r["tempos_ms"] = {modo: {k: {m: {"min": round(min(v), 2), "p50": round(statistics.median(v), 2),
                                     "max": round(max(v), 2), "n": len(v)} for m, v in d2.items()}
                             for k, d2 in d1.items()} for modo, d1 in tempos.items()}
    r["bytes_resposta"] = tamanhos
    r["linhas"] = linhas
    for k in docs:
        s.close(uris[k])
    out["inlay_sem_sym"] = r


def fase_cross_doc(s, out):
    r = {}
    a = "function helper(x)\n  return x * 2\nend\n\nlocal function so_aqui(y)\n  return y + 1\nend\n\nprint(so_aqui(helper(1)))\n"
    b = "function update(self, dt)\n  local v = helper(3)\n  print(v)\nend\n"
    ua, ub = "file:///ph2d-project/a.luau", "file:///ph2d-project/b.luau"
    s.open(ua, a)
    s.open(ub, b)
    r["diag_a"] = [{"code": d.get("code"), "message": d["message"]} for d in diags(s, ua)]
    r["diag_b"] = [{"code": d.get("code"), "message": d["message"]} for d in diags(s, ub)]
    for nome, (uri, pos) in {"helper_def_em_A": (ua, pos_de(a, "helper", 1)),
                             "helper_uso_em_B": (ub, pos_de(b, "helper", 1)),
                             "local_so_aqui_def_em_A": (ua, pos_de(a, "so_aqui", 1))}.items():
        _, c, _ = s.req("textDocument/references", {**at(uri, pos), "context": {"includeDeclaration": True}})
        r["references_" + nome] = c.get("result", c.get("error"))
        _, c, _ = s.req("textDocument/prepareRename", at(uri, pos))
        r["prepareRename_" + nome] = c.get("result", c.get("error"))
        _, c, _ = s.req("textDocument/rename", {**at(uri, pos), "newName": "novo_nome"})
        r["rename_" + nome] = c.get("result", c.get("error"))
    _, c, _ = s.req("textDocument/definition", at(ub, pos_de(b, "helper", 1)))
    r["definition_helper_uso_em_B"] = c.get("result", c.get("error"))
    s.close(ua)
    s.close(ub)
    out["cross_doc"] = r


def docs_json(path):
    """Base de documentação candidata. O símbolo dos membros de uma global declarada é
    `<pacote>/global/<global>.<membro>` (assignPropDocumentationSymbols do Luau); testamos variantes."""
    doc = {
        "@ph2d/global/ph2d": {"documentation": "DOCJSON-TABELA ph2d", "keys": {"set": "@ph2d/global/ph2d.set", "get": "@ph2d/global/ph2d.get"}},
        "@ph2d/global/ph2d.set": {"documentation": "DOCJSON-SET escreve um campo da pose.",
                                  # documentação de parâmetro = SÍMBOLO de outra entrada (forma do api-docs da Roblox)
                                  "params": [{"name": "entity", "documentation": "@ph2d/global/ph2d.set/param/0"},
                                             {"name": "field", "documentation": "@ph2d/global/ph2d.set/param/1"},
                                             {"name": "value", "documentation": "@ph2d/global/ph2d.set/param/2"}],
                                  "returns": [], "learn_more_link": "https://example.invalid/ph2d.set",
                                  "code_sample": "ph2d.set(self.id, \"x\", 1)"},
        "@ph2d/global/ph2d.set/param/0": {"documentation": "DOCJSON-PARAM0 o objeto"},
        "@ph2d/global/ph2d.set/param/1": {"documentation": "DOCJSON-PARAM1 o campo"},
        "@ph2d/global/ph2d.set/param/2": {"documentation": "DOCJSON-PARAM2 o valor"},
        # contraste: documentação de parâmetro como TEXTO directo
        "@ph2d/global/ph2d.get": {"documentation": "DOCJSON-GET",
                                  "params": [{"name": "entity", "documentation": "DOCJSON-PARAM-TEXTO-DIRECTO entity"},
                                             {"name": "field", "documentation": "DOCJSON-PARAM-TEXTO-DIRECTO field"}],
                                  "returns": ["DOCJSON-RET"]},
        "ph2d.set": {"documentation": "DOCJSON-ALT-sem-prefixo"},
    }
    with open(path, "w") as f:
        json.dump(doc, f, indent=1)
    return doc


def fase_docs(out, tmp):
    p = os.path.join(tmp, "ph2d.docs.json")
    out["docs_json_entrada"] = docs_json(p)
    s = Lsp(BASE, extra_args=[f"--docs={p}"])
    sub = {}
    fase_completion(s, sub, "completion")
    fase_sig_hover_def(s, sub, "sig_hover")
    sub["stderr"] = s.stderr[-20:]
    sub["notes_log"] = [n["params"].get("message") for n in s.notes if n.get("method") in ("window/logMessage", "window/showMessage")][:40]
    s.stop()
    # o mesmo pela configuração types.documentationFiles em vez da flag
    s2 = Lsp({**BASE, "types": {"documentationFiles": [p]}})
    sub2 = {}
    fase_sig_hover_def(s2, sub2, "sig_hover")
    sub2["notes_log"] = [n["params"].get("message") for n in s2.notes if n.get("method") in ("window/logMessage", "window/showMessage")][:40]
    s2.stop()
    out["docs_flag"] = sub
    out["docs_config"] = sub2


GODOT_PLUGIN = r'''@tool
extends EditorPlugin

func _enter_tree() -> void:
	var o := []
	o.append("ProjectSettings gui/timers/tooltip_delay_sec = %s" % str(ProjectSettings.get_setting("gui/timers/tooltip_delay_sec")))
	var es := EditorInterface.get_editor_settings()
	for p in es.get_property_list():
		var n: String = p["name"]
		var l := n.to_lower()
		if l.contains("tooltip") or l.contains("hover") or l.contains("delay") or l.contains("text_editor/completion") or l.contains("documentation") or l.contains("timers"):
			o.append("EditorSettings %s = %s" % [n, str(es.get_setting(n))])
	var f := FileAccess.open(OS.get_environment("SONDA_OUT"), FileAccess.WRITE)
	f.store_string("\n".join(o) + "\n")
	f.close()
	get_tree().quit()
'''


GODOT_HOVER = r'''extends SceneTree
var ce: CodeEdit
var t0 := 0
var fired := []
var frame := 0
func _initialize() -> void:
	var args := OS.get_cmdline_user_args()
	if args.size() > 0:
		ProjectSettings.set_setting("gui/timers/tooltip_delay_sec", float(args[0]))
	ce = CodeEdit.new()
	ce.size = Vector2(600, 300)
	ce.text = "local brilho = 1\nprint(brilho)\n"
	ce.symbol_tooltip_on_hover = true
	root.add_child(ce)
	ce.symbol_hovered.connect(func(sym, l, c): fired.append([Time.get_ticks_msec() - t0, sym, l, c]))
	print("props: tooltip_delay_sec=", ProjectSettings.get_setting("gui/timers/tooltip_delay_sec"))
func _process(_d: float) -> bool:
	frame += 1
	if frame == 5:
		var pos: Vector2 = Vector2(ce.get_rect_at_line_column(0, 8).get_center())
		var ev := InputEventMouseMotion.new()
		ev.position = pos
		ev.global_position = pos
		t0 = Time.get_ticks_msec()
		root.push_input(ev)
		print("motion at ", pos)
	if frame > 5 and (fired.size() > 0 or Time.get_ticks_msec() - t0 > 3000):
		print("fired: ", fired)
		return true
	return false
'''


def fase_godot(out):
    r = {"version": subprocess.run(["godot", "--version"], capture_output=True, text=True).stdout.strip()}
    li = subprocess.run(["pacman", "-Qi", "godot"], capture_output=True, text=True, env={**os.environ, "LC_ALL": "C"}).stdout
    r["licenca"] = [l for l in li.splitlines() if l.split(":")[0].strip() in ("Name", "Version", "Licenses")]
    with tempfile.TemporaryDirectory() as d:
        proj = os.path.join(d, "proj")
        os.makedirs(os.path.join(proj, "addons/sonda"))
        open(os.path.join(proj, "project.godot"), "w").write(
            'config_version=5\n\n[editor_plugins]\n\nenabled=PackedStringArray("res://addons/sonda/plugin.cfg")\n')
        open(os.path.join(proj, "addons/sonda/plugin.cfg"), "w").write(
            '[plugin]\n\nname="sonda"\ndescription=""\nauthor=""\nversion="1"\nscript="plugin.gd"\n')
        open(os.path.join(proj, "addons/sonda/plugin.gd"), "w").write(GODOT_PLUGIN)
        env = {**os.environ, "XDG_CONFIG_HOME": os.path.join(d, "cfg"), "XDG_DATA_HOME": os.path.join(d, "data"),
               "XDG_CACHE_HOME": os.path.join(d, "cache"), "SONDA_OUT": os.path.join(d, "out.txt")}
        cmd = ["timeout", "120", "godot", "--headless", "--editor", "--path", proj]
        p = subprocess.run(cmd, capture_output=True, text=True, env=env)
        r["comando"] = "XDG_*=<temporário> " + " ".join(cmd)
        r["rc"] = p.returncode
        try:
            r["valores_de_fabrica"] = open(env["SONDA_OUT"]).read().splitlines()
        except OSError:
            r["valores_de_fabrica"] = None
            r["log"] = (p.stdout + p.stderr)[-3000:]
    dono = os.path.expanduser("~/.config/godot/editor_settings-4.7.tres")
    try:
        r["config_do_dono"] = [l for l in open(dono).read().splitlines()
                               if re.search(r"tooltip|hover|delay|text_editor/completion/code_complete|documentation", l, re.I)]
    except OSError:
        r["config_do_dono"] = None
    # o atraso do tooltip de símbolo de um CodeEdit (symbol_tooltip_on_hover): tempo do movimento do
    # rato até ao sinal `symbol_hovered`, com gui/timers/tooltip_delay_sec de fábrica e forçado.
    r["hover_codeedit"] = {}
    with tempfile.TemporaryDirectory() as d:
        open(os.path.join(d, "project.godot"), "w").write("")
        open(os.path.join(d, "h.gd"), "w").write(GODOT_HOVER)
        for atraso in ("", "1.2", "0.1"):
            p = subprocess.run(["timeout", "30", "godot", "--headless", "--path", d, "-s", "h.gd", "--", *([atraso] if atraso else [])],
                               capture_output=True, text=True)
            r["hover_codeedit"][atraso or "fabrica"] = [l for l in p.stdout.splitlines() if l.startswith(("props:", "fired:", "motion"))]
    out["godot"] = r


def main():
    os.makedirs(SAIDA, exist_ok=True)
    out = {"loadavg_inicio": open("/proc/loadavg").read().strip(), "data": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
           "lsp": LSP, "lsp_version": subprocess.run([LSP, "--version"], capture_output=True, text=True).stdout.strip()}
    # 1. capacidades: params idênticos ao cliente Rust, e com as capacidades estendidas
    s0 = Lsp(BASE, extended=False)
    caps_rust = s0.init
    out["args"] = s0.args
    s0.stop()
    with open(os.path.join(SAIDA, "caps_luau_lsp_1.70.1.json"), "w") as f:
        json.dump(caps_rust, f, indent=1, sort_keys=True)
    s = Lsp({**BASE, "inlayHints": {"parameterNames": "all", "variableTypes": True, "parameterTypes": True,
                                    "functionReturnTypes": True}}, extended=True)
    out["caps_extended_igual_caps_rust"] = (s.init == caps_rust)
    if s.init != caps_rust:
        out["caps_extended"] = s.init
    out["configuracao_pedida_pelo_servidor"] = s.config_asks
    fase_typos(s, out)
    fase_completion(s, out)
    fase_sig_hover_def(s, out)
    fase_cross_doc(s, out)
    fase_inlay_sem_sym(s, out)
    out["stderr_principal"] = s.stderr[-20:]
    s.stop()
    # inlay hints com a configuração do cliente Rust (sem a secção inlayHints): o que aparece por omissão
    s3 = Lsp(BASE)
    u = "file:///ph2d-project/bob_omissao.luau"
    b = bob()
    s3.open(u, b)
    diags(s3, u)
    _, c, n = s3.req("textDocument/inlayHint", {"textDocument": {"uri": u}, "range": {"start": {"line": 0, "character": 0}, "end": {"line": 60, "character": 0}}})
    out["inlay_omissao"] = {"bytes": n, "result": c.get("result")}
    s3.stop()
    with tempfile.TemporaryDirectory() as tmp:
        fase_docs(out, tmp)
    fase_godot(out)
    out["loadavg_fim"] = open("/proc/loadavg").read().strip()
    with open(os.path.join(SAIDA, "I0b_bruto.json"), "w") as f:
        json.dump(out, f, indent=1, ensure_ascii=False)
    print("gravado:", SAIDA)


if __name__ == "__main__":
    main()
