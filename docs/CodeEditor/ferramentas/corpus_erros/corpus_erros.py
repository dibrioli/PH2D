#!/usr/bin/env python3
"""Corpus de erros por mutação (CodeEditor, Onda 2, item I0c).

Mede QUAIS mensagens do Luau o artista mais vê, sem telemetria: aplica operadores de erro de
novato (fontes no doc I0c) a CADA sítio aplicável dos scripts que temos, um erro por mutante, e
recolhe o que as duas vias dizem — a estática (luau-lsp, enquanto se escreve) e a do motor
(`ph2d-script`, ao gravar/correr; sonda `corpus_de_erros` em Rust).

Subcomandos (correr com `python3 -I`, a partir da raiz da worktree):
  gera        escreve os mutantes em target/corpus_erros/mutantes/ e o índice mutantes.tsv
  estatico    corre o luau-lsp (modo LSP, como o app) sobre cada mutante -> estatico.jsonl
  tabelas     junta estatico.jsonl + runtime.jsonl em classes -> classes_*.tsv e tabelas.md
O `corre.sh` ao lado faz os três passos e a sonda Rust, por ordem.
"""
import json
import os
import re
import subprocess
import sys
import threading
import time
from collections import Counter, defaultdict

AQUI = os.path.dirname(os.path.abspath(__file__))
RAIZ = os.path.abspath(os.path.join(AQUI, "..", "..", "..", ".."))
# ⚠️ FORA do repo: o `.luaurc` da raiz põe `languageMode: strict` em tudo o que estiver por baixo
# dela (medido: o bob ganhava 4 TypeError 1030 que o modo do app, não-estrito, não dá).
SAIDA_MUT = os.environ.get("PH2D_CORPUS_DIR", os.path.expanduser("~/.cache/ph2d/corpus_erros"))
DEFS = os.path.join(RAIZ, "runtime", "luau", "ph2d.d.luau")
LSP = os.path.expanduser("~/.cache/ph2d/luau-lsp/1.70.1/luau-lsp")
HOOKS = ("init", "update", "on_signal")

# O corpus: (caminho, entra?, porquê). Os que saem dizem porquê no doc.
CORPUS = [
    ("crates/ph2d-code-syntax/tests/it/fixtures/bob.luau", True, ""),
    ("crates/ph2d-app-components/src/script_boot.luau", True, ""),
    ("runtime/luau/examples/ph2d_script_attach.luau", True, ""),
    ("docs/scripting/examples/spike/lsp/case_2_coroutine.luau", True, ""),
    ("tests/spike/c7_typical_script.luau", True, ""),
    ("tests/spike/coro_timing.luau", True, ""),
    ("tests/spike/gc_stress.luau", True, ""),
    ("tests/spike/hello.luau", True, ""),
    ("docs/scripting/examples/spike/debug-autonomy/bug1_race_condition.luau", False,
     "ph2d.system / ph2d.query (não existem)"),
    ("docs/scripting/examples/spike/debug-autonomy/bug2_determinism.luau", False,
     "ph2d.state_table, tipo ph2d.Entity (não existem)"),
    ("docs/scripting/examples/spike/debug-autonomy/bug3_coroutine_leak.luau", False,
     "já tem 3 SyntaxError + tipo ph2d.Entity"),
    ("docs/scripting/examples/spike/llm-tests/01_fsm.luau", False,
     "ph2d.system / ph2d.query / ph2d.state_table, tipo ph2d.Vec2"),
    ("docs/scripting/examples/spike/llm-tests/02_query_ecs.luau", False,
     "ph2d.system / ph2d.query, tipo ph2d.Vec2"),
    ("docs/scripting/examples/spike/llm-tests/03_coro_anim.luau", False,
     "tipo ph2d.Entity; ph2d.set com campo «Color»"),
    ("docs/scripting/examples/spike/llm-tests/04_message.luau", False,
     "ph2d.message_handler / ph2d.message_send, tipos ph2d.Entity/MessagePayload"),
    ("docs/scripting/examples/spike/llm-tests/05_dialogue.luau", False,
     "ph2d.state_table / ph2d.dialogue, tipo ph2d.Entity"),
    ("docs/scripting/examples/spike/lsp/case_1_query_ecs.luau", False, "ph2d.system / ph2d.query"),
    ("docs/scripting/examples/spike/lsp/case_3_system.luau", False, "ph2d.system / ph2d.query"),
    ("docs/scripting/examples/spike/lsp/case_4_message.luau", False,
     "ph2d.message_handler, tipo ph2d.Entity"),
    ("docs/scripting/examples/spike/lsp/case_5_fsm.luau", False,
     "ph2d.state_table, tipo ph2d.Entity"),
    ("docs/CodeEditor/ferramentas/smoke/script_quebrado.luau", False,
     "quebrado DE PROPÓSITO (linha 3): o erro dele mascara qualquer mutação"),
]

# ── léxico Luau (o suficiente para achar sítios; comentários e cadeias nunca são sítio) ─────────
KEYWORDS = {
    "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "if", "in",
    "local", "nil", "not", "or", "repeat", "return", "then", "true", "until", "while",
}
OPS = sorted([
    "...", "..=", "//=", "==", "~=", "<=", ">=", "..", "::", "->", "+=", "-=", "*=", "/=", "%=",
    "^=", "//", "+", "-", "*", "/", "%", "^", "#", "<", ">", "=", "(", ")", "{", "}", "[", "]",
    ";", ":", ",", ".", "?", "|", "&",
], key=len, reverse=True)


class Tok:
    __slots__ = ("kind", "text", "start", "end", "line")

    def __init__(self, kind, text, start, end, line):
        self.kind, self.text, self.start, self.end, self.line = kind, text, start, end, line

    def __repr__(self):
        return f"{self.kind}:{self.text!r}@{self.line}"


def _long_bracket(src, i):
    """`[[`, `[==[` em i → (nível) ou None."""
    m = re.match(r"\[(=*)\[", src[i:])
    return len(m.group(1)) if m else None


def lex(src):
    toks, i, line, n = [], 0, 1, len(src)
    while i < n:
        c = src[i]
        if c == "\n":
            line += 1
            i += 1
            continue
        if c in " \t\r":
            i += 1
            continue
        start, sline = i, line
        if src.startswith("--", i):
            lvl = _long_bracket(src, i + 2)
            if lvl is not None:
                close = "]" + "=" * lvl + "]"
                j = src.find(close, i)
                j = n if j < 0 else j + len(close)
            else:
                j = src.find("\n", i)
                j = n if j < 0 else j
            line += src.count("\n", i, j)
            toks.append(Tok("comment", src[i:j], start, j, sline))
            i = j
            continue
        if c == "[" and _long_bracket(src, i) is not None:
            lvl = _long_bracket(src, i)
            close = "]" + "=" * lvl + "]"
            j = src.find(close, i)
            j = n if j < 0 else j + len(close)
            line += src.count("\n", i, j)
            toks.append(Tok("string", src[i:j], start, j, sline))
            i = j
            continue
        if c in "\"'`":
            j = i + 1
            while j < n and src[j] != c:
                if src[j] == "\\":
                    j += 1
                if j < n and src[j] == "\n":
                    line += 1
                j += 1
            j = min(j + 1, n)
            toks.append(Tok("string", src[i:j], start, j, sline))
            i = j
            continue
        m = re.match(r"0[xX][0-9a-fA-F_]+|0[bB][01_]+|(\d[\d_]*\.?[\d_]*|\.\d[\d_]*)([eE][+-]?\d+)?",
                     src[i:])
        if m and m.group(0):
            j = i + len(m.group(0))
            toks.append(Tok("number", src[i:j], start, j, sline))
            i = j
            continue
        m = re.match(r"[A-Za-z_][A-Za-z0-9_]*", src[i:])
        if m:
            w = m.group(0)
            j = i + len(w)
            toks.append(Tok("keyword" if w in KEYWORDS else "name", w, start, j, sline))
            i = j
            continue
        for op in OPS:
            if src.startswith(op, i):
                toks.append(Tok("op", op, i, i + len(op), sline))
                i += len(op)
                break
        else:
            toks.append(Tok("op", c, i, i + 1, sline))
            i += 1
    return [t for t in toks if t.kind != "comment"]


def matching(toks, k):
    """Índice do fecho que casa com a abertura em k (ou None)."""
    pairs = {"(": ")", "{": "}", "[": "]"}
    o, c, depth = toks[k].text, pairs[toks[k].text], 0
    for j in range(k, len(toks)):
        if toks[j].kind == "op" and toks[j].text == o:
            depth += 1
        elif toks[j].kind == "op" and toks[j].text == c:
            depth -= 1
            if depth == 0:
                return j
    return None


def top_commas(toks, a, b):
    """Vírgulas de profundidade 0 entre os índices a e b (exclusivos)."""
    out, depth = [], 0
    for j in range(a + 1, b):
        t = toks[j]
        if t.kind == "op" and t.text in "({[":
            depth += 1
        elif t.kind == "op" and t.text in ")}]":
            depth -= 1
        elif depth == 0 and t.kind == "op" and t.text == ",":
            out.append(j)
    return out


def is_def_paren(toks, k):
    """O `(` em k abre os PARÂMETROS de uma definição de função?"""
    j = k - 1
    if j >= 0 and toks[j].kind == "keyword" and toks[j].text == "function":
        return True
    while j >= 0 and toks[j].kind == "name":
        if j - 1 >= 0 and toks[j - 1].kind == "op" and toks[j - 1].text in (".", ":"):
            j -= 2
            continue
        return j - 1 >= 0 and toks[j - 1].kind == "keyword" and toks[j - 1].text == "function"
    return False


def is_call_paren(toks, k):
    if k == 0 or is_def_paren(toks, k):
        return False
    p = toks[k - 1]
    return p.kind == "name" or (p.kind == "op" and p.text in (")", "]"))


def table_ranges(toks):
    """Os pares (abre, fecha) de cada construtor de tabela `{...}` (exclui anotações de tipo)."""
    out = []
    for k, t in enumerate(toks):
        if t.kind == "op" and t.text == "{":
            prev = toks[k - 1] if k else None
            # `: {`, `-> {`, `type X = {` são tipos; o corpus não declara tipos por `type`.
            if prev is not None and prev.kind == "op" and prev.text in (":", "->", "::"):
                continue
            e = matching(toks, k)
            if e is not None:
                out.append((k, e))
    return out


def inside_table(ranges, k):
    """k está ao nível ZERO de algum construtor (não dentro de parêntesis aninhados)?"""
    return any(a < k < b for a, b in ranges)


def typo(w):
    """Uma troca de duas letras do meio; se não mudar nada, tira a última."""
    if len(w) == 1:
        return w + w
    i = max(len(w) // 2 - 1, 0)
    t = w[:i] + w[i + 1] + w[i] + w[i + 2:]
    return t if t != w else w[:-1]


# ── os operadores: cada um devolve [(tok_ini, tok_fim_excl, texto_novo, nota)] ─────────────────
def op_tokens(toks, text):
    return [k for k, t in enumerate(toks) if t.kind == "op" and t.text == text]


def kw_tokens(toks, text):
    return [k for k, t in enumerate(toks) if t.kind == "keyword" and t.text == text]


def m_igual_por_atribuicao(toks, src):
    return [(k, k + 1, "=", "") for k in op_tokens(toks, "==")]


def m_atribuicao_por_igual(toks, src):
    tabs = table_ranges(toks)
    out = []
    for k in op_tokens(toks, "="):
        if inside_table_level0(toks, tabs, k):
            continue
        # o `=` do cabeçalho de um `for` numérico não é uma atribuição
        j = k - 1
        while j >= 0 and toks[j].kind == "name":
            j -= 1
        if j >= 0 and toks[j].kind == "keyword" and toks[j].text == "for":
            continue
        out.append((k, k + 1, "==", ""))
    return out


def inside_table_level0(toks, tabs, k):
    for a, b in tabs:
        if a < k < b:
            depth = 0
            for j in range(a + 1, k):
                if toks[j].kind == "op" and toks[j].text in "({[":
                    depth += 1
                elif toks[j].kind == "op" and toks[j].text in ")}]":
                    depth -= 1
            if depth == 0:
                return True
    return False


def m_diferente_estilo_c(toks, src):
    return [(k, k + 1, "!=", "") for k in op_tokens(toks, "~=")]


def m_concat_com_mais(toks, src):
    return [(k, k + 1, "+", "") for k in op_tokens(toks, "..")]


def m_logico_estilo_c(toks, src):
    rep = {"and": "&&", "or": "||", "not": "!"}
    return [(k, k + 1, rep[t.text], t.text) for k, t in enumerate(toks)
            if t.kind == "keyword" and t.text in rep]


def m_comparacao_invertida(toks, src):
    rep = {">=": "=>", "<=": "=<"}
    return [(k, k + 1, rep[t.text], t.text) for k, t in enumerate(toks)
            if t.kind == "op" and t.text in rep]


def m_sem_end(toks, src):
    return [(k, k + 1, "", "") for k in kw_tokens(toks, "end")]


def m_sem_then(toks, src):
    return [(k, k + 1, "", "") for k in kw_tokens(toks, "then")]


def m_sem_do(toks, src):
    return [(k, k + 1, "", "") for k in kw_tokens(toks, "do")]


def m_sem_fecha_parenteses(toks, src):
    return [(k, k + 1, "", "") for k in op_tokens(toks, ")")]


def m_sem_fecha_chave(toks, src):
    return [(k, k + 1, "", t.text) for k, t in enumerate(toks)
            if t.kind == "op" and t.text in ("}", "]")]


def m_sem_fecha_aspas(toks, src):
    out = []
    for k, t in enumerate(toks):
        if t.kind == "string" and t.text[:1] in "\"'" and len(t.text) >= 2 \
                and t.text[-1] == t.text[0]:
            out.append((k, k + 1, t.text[:-1], ""))
    return out


def m_else_if(toks, src):
    return [(k, k + 1, "else if", "") for k in kw_tokens(toks, "elseif")]


def m_palavra_chave_trocada(toks, src):
    return [(k, k + 1, typo(t.text), t.text) for k, t in enumerate(toks)
            if t.kind == "keyword" and t.text not in ("true", "false", "nil")]


def _member(toks, k):
    return k > 0 and toks[k - 1].kind == "op" and toks[k - 1].text in (".", ":")


def m_identificador_trocado(toks, src):
    tabs = table_ranges(toks)
    out = []
    for k, t in enumerate(toks):
        if t.kind != "name" or _member(toks, k):
            continue
        # chave de campo num construtor (`{ min = 0 }`) não é um nome de variável
        nxt = toks[k + 1] if k + 1 < len(toks) else None
        if nxt is not None and nxt.kind == "op" and nxt.text == "=" \
                and inside_table_level0(toks, tabs, k):
            continue
        # anotação de tipo (`x: number`, `-> number`, `:: number?`) não é variável
        prv = toks[k - 1] if k else None
        if prv is not None and prv.kind == "op" and prv.text in ("->", "::"):
            continue
        if prv is not None and prv.kind == "op" and prv.text == ":" and k >= 2 \
                and toks[k - 2].kind == "name" and not is_call_after(toks, k):
            continue
        out.append((k, k + 1, typo(t.text), t.text))
    return out


def is_call_after(toks, k):
    return k + 1 < len(toks) and toks[k + 1].kind == "op" and toks[k + 1].text in ("(", "\"", "{")


def _in_param_list(toks, k):
    """O nome em k está dentro dos parênteses de uma definição de função (ou de um tipo)?"""
    depth = 0
    for j in range(k - 1, -1, -1):
        t = toks[j]
        if t.kind == "op" and t.text == ")":
            depth += 1
        elif t.kind == "op" and t.text == "(":
            if depth == 0:
                return is_def_paren(toks, j)
            depth -= 1
        elif t.kind == "keyword" or (t.kind == "op" and t.text in ("=", "{", "}")):
            if depth == 0:
                return False
    return False


def m_membro_trocado(toks, src):
    out = []
    for k, t in enumerate(toks):
        if t.kind != "name" or not _member(toks, k):
            continue
        # depois de `:` só uma CHAMADA é membro; o resto é anotação (`self: number`, `): boolean`)
        if toks[k - 1].text == ":" and not is_call_after(toks, k):
            continue
        out.append((k, k + 1, typo(t.text), t.text))
    return out


def m_dois_pontos_por_ponto(toks, src):
    out = []
    for k, t in enumerate(toks):
        if t.kind == "op" and t.text == ":" and k + 2 < len(toks) and toks[k + 1].kind == "name" \
                and toks[k + 2].kind == "op" and toks[k + 2].text == "(" \
                and not _in_param_list(toks, k + 1):
            out.append((k, k + 1, ".", ""))
    return out


def m_ponto_por_dois_pontos(toks, src):
    out = []
    for k, t in enumerate(toks):
        if t.kind == "op" and t.text == "." and k + 2 < len(toks) and toks[k + 1].kind == "name" \
                and toks[k + 2].kind == "op" and toks[k + 2].text == "(":
            out.append((k, k + 1, ":", ""))  # chamada OU definição, como o inverso
    return out


def m_sem_self_no_parametro(toks, src):
    out = []
    for k, t in enumerate(toks):
        if t.kind == "op" and t.text == "(" and is_def_paren(toks, k) \
                and toks[k + 1].kind == "name" and toks[k + 1].text == "self":
            e = matching(toks, k)
            commas = top_commas(toks, k, e)
            end_i = commas[0] + 1 if commas else e
            # tira `self` (com a anotação e a vírgula) até ao próximo parâmetro
            out.append((k + 1, end_i, "", "self"))
    return out


def _calls(toks):
    for k, t in enumerate(toks):
        if t.kind == "op" and t.text == "(" and is_call_paren(toks, k):
            e = matching(toks, k)
            if e is not None:
                yield k, e


def m_argumento_a_menos(toks, src):
    out = []
    for k, e in _calls(toks):
        if e == k + 1:
            continue
        commas = top_commas(toks, k, e)
        a = commas[-1] if commas else k + 1
        out.append((a, e, "", ""))
    return out


def m_argumentos_trocados(toks, src):
    out = []
    for k, e in _calls(toks):
        commas = top_commas(toks, k, e)
        if not commas:
            continue
        b = commas[1] if len(commas) > 1 else e
        first = src[toks[k + 1].start:toks[commas[0] - 1].end]
        second = src[toks[commas[0] + 1].start:toks[b - 1].end]
        if first == second:
            continue
        out.append(("raw", toks[k + 1].start, toks[b - 1].end, f"{second}, {first}"))
    return out


def m_chamada_sem_parenteses(toks, src):
    return [(k, e + 1, "", "") for k, e in _calls(toks) if e == k + 1]


def m_indice_zero(toks, src):
    out = []
    for k, t in enumerate(toks):
        if t.kind == "number" and t.text == "1" and k > 0 and k + 1 < len(toks):
            p, n = toks[k - 1], toks[k + 1]
            if p.kind == "op" and p.text == "[" and n.kind == "op" and n.text == "]":
                out.append((k, k + 1, "0", "[1]"))
            elif p.kind == "op" and p.text == "=" and k >= 3 and toks[k - 3].text == "for" \
                    and n.kind == "op" and n.text == ",":
                out.append((k, k + 1, "0", "for 1"))
    return out


def m_virgula_em_falta(toks, src):
    tabs = table_ranges(toks)
    out = []
    for a, b in tabs:
        for c in top_commas(toks, a, b):
            if c + 1 < b:  # a vírgula final (`{ a, }`) é opcional — tirá-la não é erro
                out.append((c, c + 1, "", ""))
    return out


def m_inicializacao_esquecida(toks, src):
    """Apaga uma linha `a.b = expr` inteira (fica a linha vazia: os números das linhas não mudam)."""
    out = []
    lines = src.split("\n")
    pos = 0
    for idx, ln in enumerate(lines):
        code = ln.split("--")[0].strip()
        if re.match(r"^[A-Za-z_]\w*(\.[A-Za-z_]\w*)+\s*=[^=]", code) and code.count("=") >= 1:
            s = pos + ln.index(code)
            out.append(("raw", s, s + len(code), ""))
        pos += len(ln) + 1
    return out


def m_mais_mais(toks, src):
    out = []
    for k, t in enumerate(toks):
        # `x += 1`
        if t.kind == "op" and t.text == "+=" and k + 1 < len(toks) and toks[k + 1].text == "1":
            out.append(("raw", toks[k].start, toks[k + 1].end, "++"))
        # `x = x + 1`
        if t.kind == "op" and t.text == "=" and k >= 1 and k + 3 < len(toks):
            lhs, a, plus, one = toks[k - 1], toks[k + 1], toks[k + 2], toks[k + 3]
            if a.text == lhs.text and plus.text == "+" and one.text == "1" and lhs.kind == "name":
                out.append(("raw", toks[k].start, one.end, "++"))
    return out


OPERADORES = [
    # (id, função, fonte)
    ("igual_por_atribuicao", m_igual_por_atribuicao, "A&B 2015 cat. A (= vs ==)"),
    ("atribuicao_por_igual", m_atribuicao_por_igual, "A&B 2015 cat. A (sentido inverso)"),
    ("diferente_estilo_c", m_diferente_estilo_c, "Lua: ~= (hábito C/Python !=)"),
    ("concat_com_mais", m_concat_com_mais, "Lua: .. (hábito JS/Python +)"),
    ("logico_estilo_c", m_logico_estilo_c, "A&B 2015 cat. D (&&/||) + hábito C"),
    ("comparacao_invertida", m_comparacao_invertida, "A&B 2015 cat. L (=> por >=)"),
    ("sem_end", m_sem_end, "A&B 2015 cat. C (bloco desequilibrado); Becker 2016"),
    ("sem_then", m_sem_then, "Lua: then em falta; Ko & Myers 2005 (omissão)"),
    ("sem_do", m_sem_do, "Lua: do em falta; Ko & Myers 2005 (omissão)"),
    ("sem_fecha_parenteses", m_sem_fecha_parenteses, "A&B 2015 cat. C; Becker 2016 (')' expected)"),
    ("sem_fecha_chave", m_sem_fecha_chave, "A&B 2015 cat. C (} ou ])"),
    ("sem_fecha_aspas", m_sem_fecha_aspas, "A&B 2015 cat. C (aspas)"),
    ("else_if", m_else_if, "Lua: elseif; Becker 2016 ('else' without 'if')"),
    ("palavra_chave_trocada", m_palavra_chave_trocada, "Ko & Myers 2005 (lapso de digitação)"),
    ("identificador_trocado", m_identificador_trocado,
     "Becker 2016 / A&B 2015 (cannot find symbol)"),
    ("membro_trocado", m_membro_trocado, "Becker 2016 (cannot find symbol, membro/API)"),
    ("dois_pontos_por_ponto", m_dois_pontos_por_ponto, "Lua: a:b() vs a.b(); A&B 2015 cat. M"),
    ("ponto_por_dois_pontos", m_ponto_por_dois_pontos, "Lua: a.b() vs a:b()"),
    ("sem_self_no_parametro", m_sem_self_no_parametro, "Lua: self implícito (família de :/.)"),
    ("argumento_a_menos", m_argumento_a_menos, "A&B 2015 cat. I; Becker 2016"),
    ("argumentos_trocados", m_argumentos_trocados, "A&B 2015 cat. I (tipos errados)"),
    ("chamada_sem_parenteses", m_chamada_sem_parenteses, "A&B 2015 cat. J"),
    ("indice_zero", m_indice_zero, "Lua: índices a partir de 1"),
    ("virgula_em_falta", m_virgula_em_falta, "Lua: tabela; A&B 2015 cat. C/F (separadores)"),
    ("inicializacao_esquecida", m_inicializacao_esquecida, "Ko & Myers 2005 (passo omitido)"),
    ("mais_mais", m_mais_mais, "hábito C (x++)"),
]


def nao_estrito(src):
    """O app corre NÃO-estrito por omissão: o `--!strict` vira comentário (as linhas ficam)."""
    return re.sub(r"(?m)^--!strict\s*$", "-- (strict removido para a medição)", src)


def file_id(path):
    return os.path.splitext(os.path.basename(path))[0]


def apply(src, toks, m):
    if m[0] == "raw":
        _, s, e, new = m
        return src[:s] + new + src[e:], s
    a, b, new, _ = m
    s, e = toks[a].start, toks[b - 1].end
    return src[:s] + new + src[e:], s


def cmd_gera():
    out_dir = os.path.join(SAIDA_MUT, "mutantes")
    os.makedirs(out_dir, exist_ok=True)
    for f in os.listdir(out_dir):
        os.remove(os.path.join(out_dir, f))
    idx = []
    for path, entra, _ in CORPUS:
        if not entra:
            continue
        src = nao_estrito(open(os.path.join(RAIZ, path), encoding="utf-8").read())
        fid = file_id(path)
        with open(os.path.join(out_dir, f"{fid}__base__000.luau"), "w", encoding="utf-8") as fh:
            fh.write(src)
        idx.append((f"{fid}__base__000", path, "base", 0, "", ""))
        toks = lex(src)
        for op, fn, _ in OPERADORES:
            n = 0
            seen = set()
            for m in fn(toks, src):
                new, at = apply(src, toks, m)
                if new == src or new in seen:
                    continue
                seen.add(new)
                n += 1
                mid = f"{fid}__{op}__{n:03d}"
                line = src.count("\n", 0, at) + 1
                antes = src.split("\n")[line - 1].strip()
                depois = new.split("\n")[line - 1].strip()
                with open(os.path.join(out_dir, mid + ".luau"), "w", encoding="utf-8") as fh:
                    fh.write(new)
                idx.append((mid, path, op, line, antes, depois))
    with open(os.path.join(AQUI, "mutantes.tsv"), "w", encoding="utf-8") as fh:
        fh.write("id\tficheiro\toperador\tlinha\tantes\tdepois\n")
        for r in idx:
            fh.write("\t".join(str(x).replace("\t", " ") for x in r) + "\n")
    print(f"{len(idx)} ficheiros (incl. {sum(1 for r in idx if r[2] == 'base')} bases) em {out_dir}")


# ── via estática: o luau-lsp em modo LSP, com as definições e as settings do app ────────────────
class Lsp:
    def __init__(self, root):
        self.p = subprocess.Popen(
            [LSP, "lsp", f"--definitions:@ph2d={DEFS}"],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
        self.msgs, self.lock, self.nid = {}, threading.Condition(), 0
        threading.Thread(target=self._reader, daemon=True).start()
        self.req("initialize", {
            "processId": os.getpid(), "rootUri": "file://" + root,
            "capabilities": {"workspace": {"configuration": True},
                             "textDocument": {"diagnostic": {}}},
            "initializationOptions": {}})
        self.send({"jsonrpc": "2.0", "method": "initialized", "params": {}})

    def send(self, o):
        b = json.dumps(o).encode()
        self.p.stdin.write(b"Content-Length: %d\r\n\r\n" % len(b) + b)
        self.p.stdin.flush()

    def _reader(self):
        f = self.p.stdout
        while True:
            h = {}
            while True:
                ln = f.readline()
                if not ln:
                    return
                ln = ln.decode().strip()
                if not ln:
                    break
                k, v = ln.split(":", 1)
                h[k.lower()] = v.strip()
            body = json.loads(f.read(int(h["content-length"])))
            with self.lock:
                if "id" in body and "method" not in body:
                    self.msgs[body["id"]] = body
                elif "id" in body:
                    res = None
                    if body["method"] == "workspace/configuration":
                        # = ph2d_code_lsp::ph2d_settings()
                        res = [{"platform": {"type": "standard"}, "fflags": {"sync": False}}
                               for _ in body["params"]["items"]]
                    self.send({"jsonrpc": "2.0", "id": body["id"], "result": res})
                self.lock.notify_all()

    def req(self, method, params):
        self.nid += 1
        i = self.nid
        self.send({"jsonrpc": "2.0", "id": i, "method": method, "params": params})
        with self.lock:
            ok = self.lock.wait_for(lambda: i in self.msgs, timeout=60)
        if not ok:
            raise SystemExit(f"luau-lsp não respondeu a {method}")
        return self.msgs.pop(i)

    def diagnostics(self, uri, text):
        self.send({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {
            "textDocument": {"uri": uri, "languageId": "luau", "version": 1, "text": text}}})
        b = self.req("textDocument/diagnostic", {"textDocument": {"uri": uri}})
        self.send({"jsonrpc": "2.0", "method": "textDocument/didClose",
                   "params": {"textDocument": {"uri": uri}}})
        r = b.get("result") or {}
        if r.get("kind") != "full":
            raise SystemExit(f"resposta inesperada: {b}")
        return r["items"]

    def close(self):
        try:
            self.req("shutdown", None)
            self.send({"jsonrpc": "2.0", "method": "exit"})
        finally:
            self.p.wait(timeout=10)


def hook_unused(d, text):
    """O filtro do cliente (`is_unused_engine_hook`): FunctionUnused num `function <gancho>` global."""
    if str(d.get("code")) != "8" or not d["message"].startswith("FunctionUnused:"):
        return False
    ln = text.split("\n")[d["range"]["start"]["line"]]
    return any(re.match(rf"^\s*function\s+{h}\s*\(", ln) for h in HOOKS) and \
        any(f"'{h}'" in d["message"] for h in HOOKS)


def cmd_estatico():
    mdir = os.path.join(SAIDA_MUT, "mutantes")
    files = sorted(f for f in os.listdir(mdir) if f.endswith(".luau"))
    lsp = Lsp(mdir)
    t0 = time.time()
    with open(os.path.join(AQUI, "estatico.jsonl"), "w", encoding="utf-8") as out:
        for n, f in enumerate(files):
            text = open(os.path.join(mdir, f), encoding="utf-8").read()
            items = lsp.diagnostics("file://" + os.path.join(mdir, f), text)
            diags = [{"code": str(d.get("code", "")), "severity": d.get("severity"),
                      "line": d["range"]["start"]["line"] + 1,
                      "message": d["message"]} for d in items if not hook_unused(d, text)]
            out.write(json.dumps({"mutant": f[:-5], "diags": diags}, ensure_ascii=False) + "\n")
            if n % 200 == 0:
                print(f"  estático {n}/{len(files)} ({time.time() - t0:.0f} s)", flush=True)
    lsp.close()
    print(f"estático: {len(files)} mutantes em {time.time() - t0:.0f} s")


# ── classes ─────────────────────────────────────────────────────────────────────────────────────
PUNCT_OR_KW = re.compile(r"^([^\w\s]+|" + "|".join(sorted(KEYWORDS)) + r")$")


def _arith(g):
    a, b = sorted((g.group(2), g.group(3)), key=lambda t: (t == "number", t))
    return f"attempt to perform arithmetic (<op>) on {a} and {b}"


def classe(msg):
    """A CLASSE de uma mensagem: tira nomes, números, posições e variantes de gramática."""
    m = " ".join(msg.split())
    m = re.sub(r"^[\w./-]+\.luau:\d+: ", "", m)  # posição de outro chunk
    m = re.sub(r"got ('[^']*'|\"[^\"]*\"|malformed string)", "got <tok>", m)
    m = re.sub(r"to close '\w+' at (line|column) \d+", "to close <kw> at <pos>", m)
    m = re.sub(r"at (line|column) \d+", "at <pos>", m)
    m = re.sub(r"'([^']*)'", lambda g: g.group(0) if PUNCT_OR_KW.match(g.group(1)) else "<x>", m)
    m = re.sub(r"\"[^\"]*\"", "<x>", m)
    # Argument count mismatch: as variantes de número/gramática são UMA mensagem — falta × sobra
    m = re.sub(r"expects \d+( to \d+)? arguments?, but (only \d+|none) (is|are) specified",
               "expects <n> arguments, but only <n> are specified", m)
    m = re.sub(r"expects \d+( to \d+)? arguments?, but \d+ (is|are) specified",
               "expects <n> arguments, but <n> are specified", m)
    # aritmética: a operação e a ordem dos operandos não mudam o erro; os TIPOS mudam
    m = re.sub(r"attempt to perform arithmetic \((\w+)\) on (\w+) and (\w+)", _arith, m)
    # conversão do mlua: o alvo numérico (u64/f64/…) é o mesmo pedido de número
    m = re.sub(r"to [ui](8|16|32|64|size)\b|to f(32|64)\b", "to <número>", m)
    m = re.sub(r"\b\d+(\.\d+)?\b", "<n>", m)
    return m


def load_jsonl(p):
    with open(p, encoding="utf-8") as fh:
        return {r["mutant"]: r["diags"] for r in map(json.loads, fh)}


def subtract_base(res):
    """Tira de cada mutante o que o ficheiro ORIGINAL já dizia (mesma linha e mensagem)."""
    base = {k.split("__")[0]: Counter((d.get("kind", d.get("code")), d["line"], d["message"])
                                      for d in v) for k, v in res.items() if "__base__" in k}
    out = {}
    for k, v in res.items():
        if "__base__" in k:
            continue
        left = Counter(base.get(k.split("__")[0], Counter()))
        keep = []
        for d in v:
            key = (d.get("kind", d.get("code")), d["line"], d["message"])
            if left[key] > 0:
                left[key] -= 1
            else:
                keep.append(d)
        out[k] = keep
    return out, base


def tabela(res, ops, rotulo, code_key):
    """Classes ordenadas por nº de SÍTIOS (mutantes) que as produzem."""
    sites, first, exemplos, por_op = Counter(), Counter(), {}, defaultdict(Counter)
    silencio, n_op, sev = Counter(), Counter(ops[m] for m in res), {}
    for mid, diags in res.items():
        op = ops[mid]
        if not diags:
            silencio[op] += 1
            continue
        keys = []
        for d in sorted(diags, key=lambda d: (d["line"] or 0)):
            k = (str(d[code_key]), classe(d["message"]))
            exemplos.setdefault(k, d["message"])
            sev.setdefault(k, {1: "erro", 2: "aviso", 3: "info", 4: "dica"}.get(d.get("severity"),
                                                                                "erro"))
            if k not in keys:
                keys.append(k)
        first[keys[0]] += 1
        for k in keys:
            sites[k] += 1
            por_op[k][op] += 1
    total = sum(sites.values())
    # ⚖️ a mesma conta com CADA OPERADOR a pesar igual (o viés por sítio, medido ao lado)
    eq = {k: sum(c / n_op[o] for o, c in por_op[k].items()) for k in sites}
    eq_total = sum(eq.values())
    rows, cum = [], 0
    for k, n in sites.most_common():
        cum += n
        rows.append((k[0], k[1], n, 100 * n / total, 100 * cum / total, first[k],
                     100 * eq[k] / eq_total, sev[k], " ".join(exemplos[k].split()),
                     ", ".join(f"{o} {c}" for o, c in por_op[k].most_common())))
    return rows, total, silencio


def cmd_tabelas():
    idx = {}
    with open(os.path.join(AQUI, "mutantes.tsv"), encoding="utf-8") as fh:
        next(fh)
        for ln in fh:
            r = ln.rstrip("\n").split("\t")
            idx[r[0]] = r
    ops = {k: r[2] for k, r in idx.items()}
    est, est_base = subtract_base(load_jsonl(os.path.join(AQUI, "estatico.jsonl")))
    run, run_base = subtract_base(load_jsonl(os.path.join(AQUI, "runtime.jsonl")))
    md = []
    for nome, res, key in (("estatico", est, "code"), ("runtime", run, "kind")):
        rows, total, silencio = tabela(res, ops, nome, key)
        with open(os.path.join(AQUI, f"classes_{nome}.tsv"), "w", encoding="utf-8") as fh:
            fh.write("codigo\tclasse\tsitios\tpct\tpct_acum\tprimeira\tpct_por_operador\t"
                     "gravidade\texemplo\toperadores\n")
            for r in rows:
                fh.write("\t".join(str(round(x, 1)) if isinstance(x, float) else str(x)
                                   for x in r) + "\n")
        md.append(f"\n### {nome}: {len(res)} mutantes, {len(rows)} classes, {total} pares "
                  f"(classe × sítio); calados: {sum(silencio.values())}\n")
        md.append("| # | classe | código | grav. | sítios | % | % acum. | 1.ª | % ⚖️ op. | "
                  "exemplo cru | operadores |")
        md.append("|---|---|---|---|---|---|---|---|---|---|---|")
        for i, r in enumerate(rows, 1):
            ex = r[8].replace("|", "\\|")
            if len(ex) > 140:
                ex = ex[:140] + "…"
            md.append(f"| {i} | `{r[1].replace('|', '\\|')}` | {r[0]} | {r[7]} | {r[2]} | "
                      f"{r[3]:.1f} | {r[4]:.1f} | {r[5]} | {r[6]:.1f} | `{ex}` | {r[9]} |")
        md.append("\nCalados (mutantes SEM nenhuma mensagem nova), por operador: "
                  + ", ".join(f"{o} {c}" for o, c in silencio.most_common()))
    # o mesmo erro, as duas vias: a 1.ª mensagem de cada uma, por operador
    md.append("\n### Mapa entre as vias (1.ª mensagem estática → 1.ª do motor)\n")
    md.append("| operador | mutantes | estática (1.ª) | motor (1.ª) | n |")
    md.append("|---|---|---|---|---|")
    pares = defaultdict(Counter)
    nops = Counter()
    for mid in est:
        op = ops[mid]
        nops[op] += 1

        def prim(ds, key):
            if not ds:
                return "(nada)"
            d = sorted(ds, key=lambda d: d["line"] or 0)[0]
            return f"{d[key]} · {classe(d['message'])}"
        pares[op][(prim(est[mid], "code"), prim(run.get(mid, []), "kind"))] += 1
    for op, _, _ in OPERADORES:
        if op not in pares:
            continue
        for (a, b), n in pares[op].most_common(3):
            md.append(f"| {op} | {nops[op]} | `{a.replace('|', '\\|')}` | "
                      f"`{b.replace('|', '\\|')}` | {n} |")
    # base: o que os originais já diziam
    md.append("\n### O que os ORIGINAIS já dizem (subtraído de cada mutante)\n")
    for nome, base in (("estático", est_base), ("motor", run_base)):
        for fid, c in sorted(base.items()):
            for (k, line, msg), n in c.items():
                md.append(f"- {nome} · `{fid}` linha {line}: {k} `{' '.join(msg.split())[:120]}`")
    # sítios por operador e por ficheiro
    md.append("\n### Sítios por operador × ficheiro\n")
    files = sorted({k.split("__")[0] for k in est})
    md.append("| operador | " + " | ".join(files) + " | total |")
    md.append("|---|" + "---|" * (len(files) + 1))
    for op, _, src in OPERADORES:
        c = Counter(k.split("__")[0] for k in est if ops[k] == op)
        md.append(f"| {op} | " + " | ".join(str(c[f]) for f in files) + f" | {sum(c.values())} |")
    with open(os.path.join(AQUI, "tabelas.md"), "w", encoding="utf-8") as fh:
        fh.write("\n".join(md) + "\n")
    print("\n".join(md[:6]))


if __name__ == "__main__":
    {"gera": cmd_gera, "estatico": cmd_estatico, "tabelas": cmd_tabelas}[sys.argv[1]]()
