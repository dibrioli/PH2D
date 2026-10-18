#!/usr/bin/env bash
# Medição I8: os oráculos dão EXPLICAÇÃO além da mensagem? Corre todos sobre entradas NOSSAS (scratchpad).
S=$1; B=~/.cache/ph2d/luau-lsp/1.70.1/luau-lsp
echo "# I8 — saída CRUA dos oráculos de explicação de erro. data: $(date -u +%FT%TZ)"
echo "# godot: $(godot --version) | $(LC_ALL=C pacman -Qi godot | grep -E '^Licenses' )"
echo "# luau-lsp: $($B --version) (MIT, binário fixado em ~/.cache/ph2d/luau-lsp/1.70.1)"
echo "# rustc: $(cd $S && rustc --version) (rustup; MIT OR Apache-2.0) | python: $(python3 --version) PSF-2.0 | lua: $(lua -v 2>&1 | cut -c1-9) MIT | luajit MIT"
echo; echo "############ 1. GODOT --check-only (entradas em $S/gd)"
cd $S/gd
for f in a_global b_membro c_prop_tipo d_nargs e_tipo_arg f_sintaxe g_local_nao_usada; do echo "=== $f.gd"; cat -n $f.gd; echo "--- saída"; timeout 30 godot --headless --path . --check-only -s $f.gd 2>&1 | grep -v '^Godot Engine'; done
echo "=== h_nil.gd (CORRIDO, erro de execução)"; cat -n h_nil.gd; timeout 10 godot --headless --path . -s h_nil.gd 2>&1 | grep -v '^Godot Engine'
echo "=== b2_membro_run.gd (CORRIDO)"; timeout 10 godot --headless --path . -s b2_membro_run.gd 2>&1 | grep -v '^Godot Engine'
echo; echo "############ 1b. GODOT com avisos promovidos a erro (projeto $S/gdw)"; cd $S/gdw; cat project.godot
for f in g_local_nao_usada b_membro; do echo "=== $f.gd"; timeout 30 godot --headless --path . --check-only -s $f.gd 2>&1 | grep -v '^Godot Engine'; done
echo; echo "############ 1c. GODOT LSP (editor headless --lsp-port; cliente declara codeDescriptionSupport) — cru em $S/godot_lsp_bruto.json"
python3 -I - "$S/godot_lsp_bruto.json" <<'PY'
import json,sys,os
for m in json.load(open(sys.argv[1])):
    if m.get("method")=="textDocument/publishDiagnostics":
        for d in m["params"]["diagnostics"]: print(os.path.basename(m["params"]["uri"]), json.dumps(d,ensure_ascii=False))
PY
echo "# --doctool: as descrições dos 49 avisos debug/gdscript/warnings/* saem VAZIAS (o texto de ajuda não sai do binário por script)."
echo; echo "############ 2. luau-lsp analyze (entradas em $S/lua)"; cd $S/lua; cat -n casos.luau; cat -n sintaxe.luau
for fm in default plain gnu; do echo "=== --formatter=$fm"; timeout 30 $B analyze --platform=standard --formatter=$fm casos.luau sintaxe.luau 2>&1; done
echo "=== 2b. publishDiagnostics do luau-lsp (sondas I0b/I6/I7 já gravadas): campos por código"
python3 -I - <<'PY'
import json,glob,collections
R="/home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor/docs/CodeEditor/ferramentas/medicoes_onda2/"
c={}
def walk(o):
    if isinstance(o,dict):
        if 'severity' in o and 'message' in o and 'range' in o:
            k=(o.get('code'),json.dumps(o.get('codeDescription')),o.get('source')); c.setdefault(k,o['message'][:150]); return
        for v in o.values(): walk(v)
    elif isinstance(o,list):
        for v in o: walk(v)
for f in sorted(glob.glob(R+"*_bruto.json")): walk(json.load(open(f)))
for k,v in sorted(c.items(),key=str): print(k, "|", v)
PY
echo "=== 2c. o que o href aponta (luau.org/lint, doc pública — não é fonte): secções LocalUnused / FunctionUnused / UnknownGlobal"
for t in "LocalUnused (7)" "FunctionUnused (8)" "UnknownGlobal (1)"; do awk -v t="$t" '$0==t{c++} c==3' $S/web/lint.html.txt | sed -n '1,8p'; echo "..."; done
echo "# luau.org/types (href de TODOS os TypeError/SyntaxError 1000..1030): página genérica 'An introduction to Luau types', sem âncora por código."
echo; echo "############ 3. rustc (entradas em $S/rs)"; cd $S/rs
for f in casos nome sintaxe aviso; do echo "=== rustc $f.rs"; rustc --edition 2021 --color never --emit=metadata -o $S/rs/out.rmeta $f.rs 2>&1; done
for e in E0425 E0609 E0061 E0308; do echo "=== rustc --explain $e  ($(rustc --explain $e | wc -l) linhas, $(rustc --explain $e | wc -c) chars)"; rustc --explain $e; done
echo; echo "############ 3b. Python 3.14 (entradas em $S/py)"; cd $S/py; python3 -I casos.py 2>&1; for f in sintaxe1 sintaxe2; do echo "=== $f.py"; python3 -I $f.py 2>&1; done
echo; echo "############ 3c. Lua 5.5 / LuaJIT — erros de execução (o equivalente do nosso runtime)"; cd $S/lua
for f in global membro nil sintaxe; do echo "=== lua $f.lua"; lua $f.lua 2>&1 | head -1; echo "=== luajit $f.lua"; luajit $f.lua 2>&1 | head -1; done
