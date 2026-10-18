#!/usr/bin/env bash
# Oráculo do lote I9 + I12 + I14 (CodeEditor, Onda 2), numa rodada só (CLAUDE.md §0.10):
#   I9  — as dicas de tipo no meio da linha: onde fica o cursor ao pé de uma dica, o que a seta faz, o que
#         escrever faz à dica, e o modo «só enquanto carrego Ctrl+Alt»;
#   I12 — um modelo com buracos (snippet): o 1.º buraco seleccionado? Tab / Shift+Tab / Esc, o espelho
#         (o mesmo número em dois sítios), o fim da sessão, e quantos Ctrl+Z desfazem o quê;
#   I14 — a paleta com prefixos (Ctrl+P): `@` símbolo, `:` linha, `>` comando; a prévia ao andar na lista
#         e o Esc que volta ao sítio de antes.
# O produto é CORRIDO e FOTOGRAFADO, nunca lido (CLAUDE.md §0.9). Medição: ../medicoes_onda2/I9_I12_I14.md
#
# Licença: VS Code = binário Microsoft (núcleo Code-OSS MIT), só o corremos.
#
# Uso: bash corre.sh [pasta_de_saída]      (por omissão: uma pasta nova em /tmp)
#      ORACULO_SO="2 3" bash corre.sh       (só as cenas dadas)
#
# ⛔ As regras do oraculo_i8b/corre.sh valem iguais: Xvfb PRIVADO (nunca :0, nunca `spectacle`), `env -i` com
#   HOME / --user-data-dir / --extensions-dir próprios DESDE O INÍCIO, XDG_RUNTIME_DIR curto, tudo sob
#   `timeout`, pára-se pelo PID. Só teclado (Ctrl+G linha:coluna) — nenhuma coordenada de clique.
#   A barra de estado («Ln 3, Col 9 (5 selected)») é a régua do cursor e da selecção em cada foto.
set -euo pipefail

D="${ORACULO_DISPLAY:-:119}"
[ "$D" = ":0" ] && { echo "recuso: :0 é o ecrã do dono" >&2; exit 2; }
OUT="${1:-$(mktemp -d /tmp/oraculo_i9_i12_i14.XXXX)}"
SO="${ORACULO_SO:-1 2 3 4 5 6}"
mkdir -p "$OUT/raw"
T=$(mktemp -d /tmp/i9.XXXX)
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

shot() { sleep "${2:-1.2}"; import -window root "$OUT/raw/$1.png"; echo "$1" >>"$OUT/passos.txt"; }
vai() { xdotool key ctrl+g; sleep 0.6; xdotool type --delay 40 "$1"; xdotool key Return; sleep 0.6; }
escreve() { xdotool type --delay 60 "$1"; }
quer() { [[ " $SO " == *" $1 "* ]]; }

{
  echo "# oráculo I9+I12+I14 — $(date -u +%Y-%m-%dT%H:%MZ) — loadavg: $(cat /proc/loadavg)"
  echo "# code --version: $(code --version | paste -sd ' ' -)"
  echo "# display: Xvfb $D 1600x1000x24"
} >"$OUT/passos.txt"

timeout 1200 Xvfb "$D" -screen 0 1600x1000x24 -nolisten tcp >"$T/xvfb.log" 2>&1 &
XVFB_PID=$!
sleep 1.5

# Cada cena num VS Code NOVO: um passo que falha não contamina o seguinte. $1 = valor de
# editor.inlayHints.enabled.
N=0
sessao() {
  N=$((N + 1))
  V="$T/vs$N"; mkdir -p "$V/home" "$V/ud/User/snippets" "$V/ext" "$V/ws"
  cat >"$V/ws/a.ts" <<'TS'
const amplitude = helper(2);
function helper(x: number) { return x * 2; }
class Bob {
  speed = 3;
  update(dt: number) { return this.speed * dt; }
  init() { return 0; }
}
const total = new Bob().update(amplitude);
function onSignal(name: string) { return name.length; }
console.log(total, onSignal("go"));

TS
  printf '{ "compilerOptions": { "strict": true, "module": "esnext" } }\n' >"$V/ws/tsconfig.json"
  # Modelos NOSSOS com a forma dos esqueletos do I12: um buraco por parâmetro, o espelho e o $0.
  cat >"$V/ud/User/snippets/typescript.json" <<'JSON'
{ "gancho": { "prefix": "upd", "body": ["function update(${1:self}: object, ${2:dt}: number) {", "\t$0", "}"] },
  "espelho": { "prefix": "mir", "body": "const ${1:x} = ${2:0}; console.log(${1:x});$0" },
  "propriedade": { "prefix": "prop", "body": "property(\"${1:name}\", ${2:0});$0" } }
JSON
  cat >"$V/ud/User/settings.json" <<JSON
{ "workbench.startupEditor": "none", "telemetry.telemetryLevel": "off", "update.mode": "none",
  "extensions.autoCheckUpdates": false, "extensions.autoUpdate": false, "workbench.tips.enabled": false,
  "security.workspace.trust.enabled": false, "window.restoreWindows": "none",
  "chat.disableAIFeatures": true, "workbench.enableExperiments": false, "files.autoSave": "off",
  "editor.inlayHints.enabled": "$1",
  "typescript.inlayHints.variableTypes.enabled": true,
  "typescript.inlayHints.functionLikeReturnTypes.enabled": true,
  "typescript.inlayHints.parameterTypes.enabled": true,
  "typescript.inlayHints.parameterNames.enabled": "none",
  "editor.suggest.snippetsPreventQuickSuggestions": false,
  "editor.quickSuggestions": ${2:-true} }
JSON
  env -i HOME="$V/home" PATH=/usr/bin:/bin USER="$USER" LANG=en_US.UTF-8 DISPLAY="$D" XDG_RUNTIME_DIR="$T" \
    timeout 300 /usr/share/code/code --user-data-dir="$V/ud" --extensions-dir="$V/ext" --new-window \
    --disable-workspace-trust --disable-gpu --password-store=basic --ozone-platform=x11 \
    --skip-welcome --skip-release-notes "$V/ws" "$V/ws/a.ts" >"$T/code$N.log" 2>&1 &
  CODE_TIMEOUT_PID=$!
  W=""
  for _ in $(seq 30); do W=$(xdotool search --name 'Visual Studio Code' 2>/dev/null | head -1 || true); [ -n "$W" ] && break; sleep 1; done
  [ -n "$W" ] || { echo "VS Code não abriu" >&2; cat "$T/code$N.log" >&2; exit 1; }
  xdotool windowmove "$W" 0 0 windowsize "$W" 1600 1000
  sleep 9   # o servidor TS sobe
}
fecha() {
  kill "$CODE_TIMEOUT_PID" 2>/dev/null || true; wait "$CODE_TIMEOUT_PID" 2>/dev/null || true; CODE_TIMEOUT_PID=""
  for _ in $(seq 10); do xdotool search --name 'Visual Studio Code' >/dev/null 2>&1 || break; sleep 1; done
}

# 1. I9 — dicas sempre ligadas: o cursor ao pé de `amplitude: number`, a seta, escrever ao lado.
if quer 1; then
sessao on
shot h1_dicas_ligadas 2
vai 1:16; shot h2_cursor_no_fim_do_nome            # Col 16 = logo depois de `amplitude`: antes ou depois da dica?
xdotool key Right; shot h3_seta_direita            # Col 17: a seta salta a dica inteira?
xdotool key Left; xdotool key shift+Right; shot h4_shift_direita   # a selecção de 1 carácter cobre a dica?
xdotool key Left; vai 1:16; escreve Z; shot h5_escreveu_ao_lado 0.15
shot h6_escreveu_ao_lado_depois 2                  # a dica desaparece até o servidor responder, ou anda?
xdotool key BackSpace; vai 1:1; xdotool key End; shot h7_fim_da_linha
xdotool key Home; xdotool key ctrl+Right ctrl+Right; shot h8_ctrl_direita   # salto de palavra sobre a dica
fecha
fi

# 2. I9 — modo «só com Ctrl+Alt carregado».
if quer 2; then
sessao offUnlessPressed
vai 1:16; shot h9_sem_tecla 2
xdotool keydown ctrl+alt; shot h10_com_ctrl_alt 1.5; xdotool keyup ctrl+alt
shot h11_largou 1
fecha
fi

# 3. I12 — o modelo com buracos: abrir, preencher, Tab, Shift+Tab, Tab até ao fim, e o Tab depois.
if quer 3; then
# ⚠️ Sem a lista automática: com ela, escrever `me` no buraco abria «public method» e o Tab ACEITAVA a
#   sugestão em vez de saltar (medido na 1.ª corrida, 18/10 — a lista ganha ao modelo). A lista do
#   modelo abre-se à mão (Ctrl+Espaço).
sessao off false
vai 11:1; escreve upd; xdotool key ctrl+space; shot s1_lista 1.5
xdotool key Tab; shot s2_inserido                  # o 1.º buraco seleccionado (5 selected)?
escreve me; shot s3_escreveu_no_1
xdotool key Tab; shot s4_tab_vai_ao_2              # `dt` seleccionado?
xdotool key shift+Tab; shot s5_shift_tab_volta     # `me` seleccionado inteiro?
xdotool key Tab; escreve t; xdotool key Tab; shot s6_tab_no_0   # o $0 dentro do corpo; a sessão acaba?
xdotool key Tab; shot s7_tab_depois_do_fim         # Tab indenta (sessão acabou) ou salta?
xdotool key ctrl+z; shot s8_z1
xdotool key ctrl+z; shot s9_z2
xdotool key ctrl+z; shot s10_z3
xdotool key ctrl+z; shot s11_z4
xdotool key ctrl+z; shot s12_z5
fecha
fi

# 3b. I12 — escrever logo a seguir ao buraco seleccionado (seta direita, depois letras) e voltar.
if quer 3; then
sessao off false
vai 11:1; escreve upd; xdotool key ctrl+space; sleep 1; xdotool key Tab
xdotool key Right; escreve Q; shot s20_escreveu_no_fim_do_1
xdotool key Tab; xdotool key shift+Tab; shot s21_shift_tab_apanha_o_Q   # «selfQ» seleccionado inteiro?
xdotool key Tab; xdotool key BackSpace; shot s22_buraco_apagado
xdotool key shift+Tab; xdotool key Tab; shot s23_buraco_vazio_volta     # o buraco vazio ainda é paragem?
fecha
fi

# 4. I12 — o espelho, o Esc, e sair do buraco com a seta para baixo.
if quer 4; then
sessao off false
vai 11:1; escreve mir; xdotool key ctrl+space; sleep 1; xdotool key Tab; shot s13_espelho_inserido   # os dois `x` seleccionados (2 cursores)?
escreve abc; shot s14_espelho_escrito
xdotool key Escape; shot s15_esc
xdotool key Tab; shot s16_tab_depois_do_esc        # Tab normal: a sessão acabou no Esc?
xdotool key ctrl+z ctrl+z; vai 11:1; xdotool key ctrl+End; xdotool key Return
escreve prop; xdotool key ctrl+space; sleep 1; xdotool key Tab; shot s17_prop_inserido   # o buraco dentro das aspas
xdotool key Down; shot s18_desceu_para_fora
xdotool key Up; xdotool key Tab; shot s19_tab_depois_de_sair   # ainda salta para o 2.º buraco?
fecha
fi

# 5. I14 — a paleta: `@` símbolo, a prévia ao descer, o Esc que volta, `@:` por tipo.
if quer 5; then
sessao off
vai 10:5
xdotool key ctrl+p; sleep 0.8; escreve @; shot p1_arroba 1.5
escreve upd; shot p2_arroba_upd 1
xdotool key ctrl+a; escreve @; xdotool key Down Down; shot p3_desceu_previa 1    # o editor anda já?
xdotool key Escape; shot p4_esc_volta 1                                          # volta a Ln 10?
xdotool key ctrl+p; sleep 0.8; escreve @:; shot p5_arroba_dois_pontos 1.5
xdotool key Escape
xdotool key ctrl+p; sleep 0.8; escreve @onSig; xdotool key Return; shot p6_enter_salta 1
fecha
fi

# 6. I14 — `:` linha (prévia, Enter, coluna, fora do ficheiro) e `>` comando, e o texto sem prefixo.
if quer 6; then
sessao off
vai 10:5
xdotool key ctrl+p; sleep 0.8; escreve :; shot p7_dois_pontos 1.2
escreve 3; shot p8_dois_pontos_3 1.2                                             # prévia na linha 3?
xdotool key Escape; shot p9_esc_volta 1
xdotool key ctrl+p; sleep 0.8; escreve :5:3; xdotool key Return; shot p10_linha_coluna 1
xdotool key ctrl+p; sleep 0.8; escreve :99; shot p11_fora 1.2
xdotool key Return; shot p12_fora_enter 1
xdotool key ctrl+p; sleep 0.8; escreve '>'; shot p13_maior 1.5
escreve 'inlay'; shot p14_maior_inlay 1.2
xdotool key ctrl+a; escreve 'a.ts'; shot p15_sem_prefixo 1.2
xdotool key Escape
fecha
fi

echo "gravado em $OUT"; cat "$OUT/passos.txt"
