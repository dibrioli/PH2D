#!/usr/bin/env bash
# Oráculo do I13 (CodeEditor, Onda 2): o «formatar ao salvar» do VS Code com o StyLua, numa rodada só
# (CLAUDE.md §0.10). O que se mede, cena a cena (cada uma num VS Code NOVO):
#   1  o cursor a meio de uma palavra numa linha re-recuada (a mais, a menos), dentro do recuo apagado,
#      e na coluna 1 onde o recuo é INSERIDO;
#   2  o cursor a meio do vão de espaços que o StyLua encolhe (os comentários alinhados do bob.luau);
#   3  o cursor antes de `return` em `if not self.active then return end` (o StyLua parte em 3 linhas),
#      e no espaço que vira quebra de linha;
#   4  o cursor dentro de `'abc'` → `"abc"` (entre `a` e `b`) e logo depois da aspa de abertura;
#   5  uma SELECÇÃO que atravessa linhas re-recuadas (para a frente e para trás): âncora e cabeça;
#   6  dois cursores (Add Cursor Below) e duas selecções (Ctrl+D) em linhas que mudam;
#   7  o desfazer: quantos Ctrl+Z voltam ao pré-formatação, se o que se escreveu antes fica, o cursor,
#      o ● do separador, e o Ctrl+Shift+Z;
#   8  erro de sintaxe: grava? formata? notificação? barra? o canal Output do StyLua;
#   9  ficheiro já formatado: o Ctrl+S cria passo de desfazer?;
#   10 escrever logo a seguir ao Ctrl+S: o texto entra antes ou depois da formatação? perde-se algo?
# O produto é CORRIDO e FOTOGRAFADO, nunca lido (CLAUDE.md §0.9). Medição: ../medicoes_onda2/I13.md
#
# Licenças: VS Code = binário Microsoft (núcleo Code-OSS MIT), só o corremos. Extensão StyLua
#   (JohnnyMorganz.stylua 1.7.2, Open VSX): `license` do package.json = MPL-2.0; o .vsix NÃO traz ficheiro
#   de licença (8 ficheiros: manifesto, package.json, readme, changelog, .vscode-test.js, dist/extension.js,
#   ícone). Só a CORREMOS; o código dela não se lê. StyLua 2.5.2 = MPL-2.0 (o binário que o app empacota).
#
# Uso: bash corre.sh [pasta_de_saída]      (por omissão: uma pasta nova em /tmp)
#      ORACULO_SO="1a 7" bash corre.sh     (só as cenas dadas)
#
# ⛔ As regras do oraculo_i9_i12_i14/corre.sh valem iguais: Xvfb PRIVADO (nunca :0, nunca `spectacle` —
#   fotografa o ecrã REAL do dono; `import -window root`), `env -i` com HOME / --user-data-dir /
#   --extensions-dir próprios DESDE O INÍCIO, XDG_RUNTIME_DIR curto, tudo sob `timeout`, pára-se pelo PID.
#   Só teclado (Ctrl+G linha:coluna, a paleta para comandos) — nenhuma coordenada de clique.
#   A régua do cursor é a barra de estado («Ln 3, Col 9 (5 selected)», lida por OCR e conferida na foto),
#   o título da janela (o «●» do sujo) e o ficheiro em DISCO depois de cada passo.
set -euo pipefail

D="${ORACULO_DISPLAY:-:123}"
[ "$D" = ":0" ] && { echo "recuso: :0 é o ecrã do dono" >&2; exit 2; }
R=$(cd "$(dirname "$0")/../../../.." && pwd)
ST="${STYLUA:-$HOME/.cache/ph2d/stylua/2.5.2/stylua}"
CFG="$R/crates/ph2d-code-format/ph2d-format.toml"
VSIX_URL=https://open-vsx.org/api/JohnnyMorganz/stylua/1.7.2/file/JohnnyMorganz.stylua-1.7.2.vsix
VSIX_SHA=1eda514bf96a521be5f04deaa319c86ba3aa7a481f64e34e5b5f7650ac274d8e
LUAU_EXT="$HOME/.vscode/extensions/johnnymorganz.luau-lsp-1.70.1-linux-x64"
OUT="${1:-$(mktemp -d /tmp/oraculo_i13.XXXX)}"
SO="${ORACULO_SO:-1a 1b 1c 1d 2a 2b 3a 3b 4a 4b 5a 5b 6a 6b 7 8 9 10}"
mkdir -p "$OUT/raw" "$OUT/cli"
T=$(mktemp -d /tmp/i13.XXXX)
chmod 700 "$T"
export DISPLAY="$D"
unset WAYLAND_DISPLAY

XVFB_PID=""
CODE_TIMEOUT_PID=""
limpa() {
  [ -n "$CODE_TIMEOUT_PID" ] && kill "$CODE_TIMEOUT_PID" 2>/dev/null || true
  [ -n "$XVFB_PID" ] && kill "$XVFB_PID" 2>/dev/null || true
  rm -rf "$T"
}
trap limpa EXIT

quer() { [[ " $SO " == *" $1 "* ]]; }
vai() { xdotool key ctrl+g; sleep 0.6; xdotool type --delay 40 "$1"; xdotool key Return; sleep 0.6; }
escreve() { xdotool type --delay 60 "$1"; }
paleta() { xdotool key ctrl+shift+p; sleep 0.9; xdotool type --delay 30 "$1"; sleep 0.9; xdotool key Return; sleep 1; }
# A régua: a barra de estado por OCR (conferida à mão na foto), o título (● = sujo) e o disco.
shot() {
  sleep "${2:-1.2}"
  import -window root "$OUT/raw/$1.png"
  magick "$OUT/raw/$1.png" -crop 560x22+1040+978 +repage -resize 300% -negate "$T/barra.png"
  local barra; barra=$(tesseract "$T/barra.png" - --psm 7 2>/dev/null | tr -s ' \n' ' ' || true)
  cp "$V/ws/$FICH" "$OUT/raw/$1.disco.luau"
  printf '%s | título: %s | barra: %s | disco sha: %s\n' "$1" "$(xdotool getwindowname "$W" 2>/dev/null || echo '?')" \
    "$barra" "$(sha256sum <"$V/ws/$FICH" | cut -c1-12)" >>"$OUT/passos.txt"
}

{
  echo "# oráculo I13 — $(date -u +%Y-%m-%dT%H:%MZ) — loadavg: $(cat /proc/loadavg)"
  echo "# code --version: $(code --version | paste -sd ' ' -)"
  echo "# stylua: $("$ST" --version) ($ST) · config: $CFG"
  echo "# display: Xvfb $D 1600x1000x24"
} >"$OUT/passos.txt"

# --- A extensão: o .vsix do Open VSX, conferido por sha256; a triagem de licença por ARTEFACTO.
curl -sS -m 60 -L -o "$T/stylua.vsix" "$VSIX_URL"
echo "$VSIX_SHA  $T/stylua.vsix" | sha256sum -c --quiet
{
  echo "# vsix: JohnnyMorganz.stylua-1.7.2 sha256 $VSIX_SHA"
  echo "# vsix license (package.json): $(unzip -p "$T/stylua.vsix" extension/package.json | python3 -I -c 'import json,sys; print(json.load(sys.stdin).get("license"))')"
  echo "# vsix ficheiros de licença: $(unzip -Z1 "$T/stylua.vsix" | grep -i -c licen || true)"
} >>"$OUT/passos.txt"
mkdir -p "$T/ext0" "$T/cli_ud" "$T/cli_home"
cp -a "$LUAU_EXT" "$T/ext0/"
env -i HOME="$T/cli_home" PATH=/usr/bin:/bin timeout 120 /usr/share/code/bin/code --user-data-dir="$T/cli_ud" \
  --extensions-dir="$T/ext0" --install-extension "$T/stylua.vsix" >"$T/install.log" 2>&1
echo "# extensões: $(env -i HOME="$T/cli_home" PATH=/usr/bin:/bin timeout 60 /usr/share/code/bin/code \
  --user-data-dir="$T/cli_ud" --extensions-dir="$T/ext0" --list-extensions --show-versions 2>/dev/null | paste -sd ' ' -)" >>"$OUT/passos.txt"

# --- As fixtures NOSSAS e o que o StyLua faz a cada uma, pela linha de comandos (a verdade do texto).
FX="$T/fx"; mkdir -p "$FX"
cp "$R/crates/ph2d-code-syntax/tests/it/fixtures/bob.luau" "$FX/bob.luau"
printf 'function f()\n    local amplitude = 1\nreturn amplitude\nend\n' >"$FX/recuo.luau"
printf "local s = 'abc'\nlocal t = 'xyz'\n" >"$FX/aspas.luau"
printf 'function f()\n    local x = 1\n  if x then\n    x = 2\nend\n' >"$FX/erro.luau"
printf 'function f()\n  local amplitude = 1\n  return amplitude\nend\n' >"$FX/ok.luau"
for f in bob recuo aspas erro ok; do
  cp "$FX/$f.luau" "$OUT/cli/$f.luau"
  set +e; "$ST" --config-path "$CFG" - <"$FX/$f.luau" >"$OUT/cli/$f.fmt.luau" 2>"$OUT/cli/$f.err"; echo $? >"$OUT/cli/$f.exit"; set -e
done

timeout 1800 Xvfb "$D" -screen 0 1600x1000x24 -nolisten tcp >"$T/xvfb.log" 2>&1 &
XVFB_PID=$!
sleep 1.5

# Cada cena num VS Code NOVO. $1 = a fixture aberta.
N=0
sessao() {
  N=$((N + 1)); FICH="$1"
  V="$T/vs$N"; mkdir -p "$V/home" "$V/ud/User" "$V/ws"
  cp -a "$T/ext0" "$V/ext"
  cp "$CFG" "$V/ws/stylua.toml"
  cp "$FX/$FICH" "$V/ws/$FICH"
  cat >"$V/ud/User/settings.json" <<JSON
{ "workbench.startupEditor": "none", "telemetry.telemetryLevel": "off", "update.mode": "none",
  "extensions.autoCheckUpdates": false, "extensions.autoUpdate": false, "workbench.tips.enabled": false,
  "security.workspace.trust.enabled": false, "window.restoreWindows": "none",
  "chat.disableAIFeatures": true, "workbench.enableExperiments": false,
  "files.autoSave": "off", "editor.formatOnSave": true, "editor.inlayHints.enabled": "off",
  "editor.quickSuggestions": false, "editor.suggestOnTriggerCharacters": false,
  "[luau]": { "editor.defaultFormatter": "JohnnyMorganz.stylua" },
  "[lua]": { "editor.defaultFormatter": "JohnnyMorganz.stylua" },
  "stylua.styluaPath": "$ST", "stylua.configPath": "$V/ws/stylua.toml",
  "stylua.disableVersionCheck": true, "stylua.searchBinaryInPATH": false,
  "luau-lsp.platform.type": "standard", "luau-lsp.types.roblox": false,
  "luau-lsp.sourcemap.enabled": false, "luau-lsp.fflags.sync": false }
JSON
  env -i HOME="$V/home" PATH=/usr/bin:/bin USER="$USER" LANG=en_US.UTF-8 DISPLAY="$D" XDG_RUNTIME_DIR="$T" \
    timeout 300 /usr/share/code/code --user-data-dir="$V/ud" --extensions-dir="$V/ext" --new-window \
    --disable-workspace-trust --disable-gpu --password-store=basic --ozone-platform=x11 \
    --skip-welcome --skip-release-notes "$V/ws" "$V/ws/$FICH" >"$T/code$N.log" 2>&1 &
  CODE_TIMEOUT_PID=$!
  W=""
  for _ in $(seq 40); do W=$(xdotool search --name 'Visual Studio Code' 2>/dev/null | head -1 || true); [ -n "$W" ] && break; sleep 1; done
  [ -n "$W" ] || { echo "VS Code não abriu" >&2; cat "$T/code$N.log" >&2; exit 1; }
  xdotool windowmove "$W" 0 0 windowsize "$W" 1600 1000
  sleep 10   # as extensões sobem (o StyLua activa-se com a linguagem luau)
}
fecha() {
  kill "$CODE_TIMEOUT_PID" 2>/dev/null || true; wait "$CODE_TIMEOUT_PID" 2>/dev/null || true; CODE_TIMEOUT_PID=""
  for _ in $(seq 10); do xdotool search --name 'Visual Studio Code' >/dev/null 2>&1 || break; sleep 1; done
}
# O sítio do cursor depois da formatação, provado no DISCO: escreve `Q` e grava SEM formatar.
marca() { escreve Q; paleta 'File: Save without Formatting'; shot "$1" 0.8; }
# Uma cena de cursor: abre, vai a L:C, foto, Ctrl+S, foto, a marca.
cursor() {
  local id="$1" f="$2" pos="$3"
  sessao "$f"; vai "$pos"; shot "${id}_antes"
  xdotool key ctrl+s; shot "${id}_depois" 2.5
  marca "${id}_marca"; fecha
}

quer 1a && cursor 01a_meio_palavra_recuo4 recuo.luau 2:15        # `    local ampl│itude`: 4 espaços → 2
quer 1b && cursor 01b_meio_palavra_recuo0 recuo.luau 3:10        # `return am│plitude`: 0 espaços → 2
quer 1c && cursor 01c_dentro_do_recuo_apagado recuo.luau 2:4     # `   │ local`: o recuo encolhe por baixo
quer 1d && cursor 01d_coluna1_recuo_inserido recuo.luau 3:1      # `│return`: o recuo nasce à esquerda
quer 2a && cursor 02a_meio_do_vao_grande bob.luau 11:42          # 26 espaços → 1
quer 2b && cursor 02b_meio_do_vao_pequeno bob.luau 10:53         # 5 espaços → 1
quer 3a && cursor 03a_antes_do_return bob.luau 32:27             # `then │return end` → 3 linhas
quer 3b && cursor 03b_no_espaco_que_parte bob.luau 32:26         # `then│ return` (o espaço vira quebra)
quer 4a && cursor 04a_dentro_da_string aspas.luau 1:13           # `'a│bc'` → `"abc"`
quer 4b && cursor 04b_depois_da_aspa aspas.luau 2:12             # `'│xyz'` → `"xyz"`

# 5. Selecção que atravessa linhas re-recuadas: para a frente (âncora 2:15 → cabeça 3:15) e para trás.
#    ← colapsa no INÍCIO da selecção, → no FIM: assim a outra ponta também se lê na barra.
if quer 5a; then
  sessao recuo.luau; vai 2:15; xdotool key shift+Down; shot 05a_selec_frente_antes
  xdotool key ctrl+s; shot 05a_selec_frente_depois 2.5
  xdotool key Left; shot 05a_selec_frente_inicio; fecha
fi
if quer 5b; then
  sessao recuo.luau; vai 3:15; xdotool key shift+Up; shot 05b_selec_tras_antes
  xdotool key ctrl+s; shot 05b_selec_tras_depois 2.5
  xdotool key Right; shot 05b_selec_tras_fim; fecha
fi

# 6. Dois cursores (2:15 e 3:15) e duas selecções (`amplitude` ×2 pelo Ctrl+D).
if quer 6a; then
  sessao recuo.luau; vai 2:15; paleta 'Add Cursor Below'; shot 06a_dois_cursores_antes
  xdotool key ctrl+s; shot 06a_dois_cursores_depois 2.5
  marca 06a_dois_cursores_marca; fecha
fi
if quer 6b; then
  sessao recuo.luau; vai 2:15; xdotool key ctrl+d; sleep 0.4; xdotool key ctrl+d; shot 06b_ctrl_d_antes
  xdotool key ctrl+s; shot 06b_ctrl_d_depois 2.5
  xdotool key Escape; shot 06b_ctrl_d_esc; fecha
fi

# 7. O desfazer: escreve ` -- oi` numa linha que o StyLua não muda, cursor numa que muda, Ctrl+S, e Z/⇧Z.
if quer 7; then
  sessao recuo.luau; vai 1:13; escreve ' -- oi'; vai 2:15; shot 07_escrito_antes
  xdotool key ctrl+s; shot 07_salvo 2.5
  xdotool key ctrl+z; shot 07_z1
  xdotool key ctrl+z; shot 07_z2
  xdotool key ctrl+shift+z; shot 07_refaz1
  xdotool key ctrl+shift+z; shot 07_refaz2
  fecha
fi

# 8. Erro de sintaxe (falta um `end`): escreve `9` (para o ficheiro ficar sujo), Ctrl+S, e o que aparece.
if quer 8; then
  sessao erro.luau; vai 2:16; escreve 9; shot 08_erro_antes
  xdotool key ctrl+s; shot 08_erro_salvo_05s 0.5
  shot 08_erro_salvo_4s 3.5
  paleta 'StyLua: Show Output Channel'; shot 08_erro_output 1.5
  fecha
fi

# 9. Já formatado: (a) Ctrl+S no ficheiro limpo e Ctrl+Z; (b) escreve `2` (continua formatado), Ctrl+S, Ctrl+Z.
if quer 9; then
  sessao ok.luau; vai 2:15; xdotool key ctrl+s; shot 09a_limpo_salvo 2
  xdotool key ctrl+z; shot 09a_limpo_z1
  vai 2:22; escreve 2; shot 09b_escrito
  xdotool key ctrl+s; shot 09b_salvo 2
  xdotool key ctrl+z; shot 09b_z1
  fecha
fi

# 10. Escrever logo a seguir ao Ctrl+S (o mesmo `xdotool`, sem pausa): ZZZ no meio de `ampl│itude`.
if quer 10; then
  sessao recuo.luau; vai 2:15; shot 10_antes
  xdotool key ctrl+s type --delay 15 ZZZ; shot 10_logo 0.3
  shot 10_depois 3
  paleta 'File: Save without Formatting'; shot 10_buffer_gravado 0.8
  fecha
fi

echo "gravado em $OUT"; cat "$OUT/passos.txt"
