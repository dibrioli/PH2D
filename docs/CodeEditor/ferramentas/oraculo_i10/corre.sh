#!/usr/bin/env bash
# Oráculo do I10 (CodeEditor, Onda 2): como o VS Code RENOMEIA (F2) e como DESFAZ um renomear que tocou
# em dois ficheiros — a pergunta do marcador do undo global (um passo que mora em duas histórias).
# O produto é CORRIDO e FOTOGRAFADO, nunca lido (CLAUDE.md §0.9). Medição: ../medicoes_onda2/I10_rename_e_undo.md
#
# Licença: VS Code = binário Microsoft (núcleo Code-OSS MIT), só o corremos. O Godot 4.7.2 não tem rename
# nem referências no editor de scripts (a lista de atalhos `script_text_editor/*` do binário: só find/replace).
#
# Uso: bash corre.sh [pasta_de_saída]      (por omissão: uma pasta nova em /tmp)
#
# ⛔ As regras do oraculo_i8b/corre.sh valem iguais: Xvfb PRIVADO (nunca :0, nunca `spectacle`), `env -i` com
#   HOME / --user-data-dir / --extensions-dir próprios DESDE O INÍCIO (sem isso ele escreve no
#   ~/.vscode-shared do dono, medido 18/10), XDG_RUNTIME_DIR curto, tudo sob `timeout`, pára-se pelo PID.
#   Só teclado (Ctrl+G linha:coluna) — nenhuma coordenada de clique.
set -euo pipefail

D=  # o ecrã: o Xvfb escolhe-o livre (`xvfb_livre`, abaixo)
OUT="${1:-$(mktemp -d /tmp/oraculo_i10.XXXX)}"
mkdir -p "$OUT/raw"
T=$(mktemp -d /tmp/i10.XXXX)
chmod 700 "$T"
unset WAYLAND_DISPLAY

XVFB_PID=""
CODE_TIMEOUT_PID=""
limpa() {
  [ -n "$CODE_TIMEOUT_PID" ] && kill "$CODE_TIMEOUT_PID" 2>/dev/null || true
  [ -n "$XVFB_PID" ] && kill "$XVFB_PID" 2>/dev/null || true
  rm -rf "$T"
}
trap limpa EXIT

shot() { sleep "${2:-1.5}"; import -window root "$OUT/raw/$1.png"; echo "$1" >>"$OUT/passos.txt"; }
vai() { xdotool key ctrl+g; sleep 0.6; xdotool type --delay 40 "$1"; xdotool key Return; sleep 0.6; }
abre() { xdotool key ctrl+p; sleep 0.8; xdotool type --delay 40 "$1"; sleep 0.8; xdotool key Return; sleep 2; }

{
  echo "# oráculo I10 — $(date -u +%Y-%m-%dT%H:%MZ) — loadavg: $(cat /proc/loadavg)"
  echo "# code --version: $(code --version | paste -sd ' ' -)"
  echo "# display: Xvfb livre (-displayfd) 1600x1000x24"
} >"$OUT/passos.txt"

. "$(dirname "${BASH_SOURCE[0]}")/../xvfb_livre.sh"
xvfb_livre 1600x1000x24 "$T/xvfb.log" 900 || exit 2
D=$DSP; XVFB_PID=$XV

# Cada cena num VS Code NOVO sobre ficheiros novos: um passo que falha não contamina o seguinte.
N=0
sessao() {
  N=$((N + 1))
  V="$T/vs$N"; mkdir -p "$V/home" "$V/ud/User" "$V/ext" "$V/ws"
  printf 'export function helper(x: number) { return x * 2; }\nconst amplitude = 1;\nconsole.log(helper(amplitude));\n' >"$V/ws/a.ts"
  printf 'import { helper } from "./a";\nconsole.log(helper(3));\n' >"$V/ws/b.ts"
  printf '{ "compilerOptions": { "strict": true, "module": "esnext" } }\n' >"$V/ws/tsconfig.json"
  cat >"$V/ud/User/settings.json" <<'JSON'
{ "workbench.startupEditor": "none", "telemetry.telemetryLevel": "off", "update.mode": "none",
  "extensions.autoCheckUpdates": false, "extensions.autoUpdate": false, "workbench.tips.enabled": false,
  "security.workspace.trust.enabled": false, "window.restoreWindows": "none",
  "chat.disableAIFeatures": true, "workbench.enableExperiments": false, "files.autoSave": "off" }
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
  { echo "-- ficheiros no disco no fim da sessão $N:"; head -3 "$V/ws/a.ts" "$V/ws/b.ts"; } >>"$OUT/passos.txt"
}
# F2 sobre (linha:coluna), escreve, e termina com a tecla dada (Return · ctrl+Return · Escape)
f2() { vai "$1"; xdotool key F2; sleep 1.2; xdotool key ctrl+a; xdotool type --delay 60 "$2"; sleep 0.4; xdotool key "$3"; }

# I10_SO=6 corre só a 6.ª cena (o atraso do realce).
if [ "${I10_SO:-}" != 6 ]; then
# 1. A caixinha (vem com o nome? seleccionado?), aplicar, e desfazer/refazer pelos DOIS ficheiros
sessao
vai 3:15; xdotool key F2; shot r1_f2_aberto
xdotool type --delay 60 ajuda; shot r2_escrito 0.8
xdotool key Return; shot r3_aplicado 2.5                    # a.ts e b.ts mudam? b.ts fica aberto/sujo?
xdotool key ctrl+z; shot r4_ctrlz_a 1.5                     # pergunta? desfaz os dois?
xdotool key ctrl+shift+z; shot r6_refazer 1.5
abre b.ts; shot r7_b_aberto 1
xdotool key ctrl+z; shot r8_ctrlz_b 1.5                     # desfazer pelo OUTRO ficheiro
fecha
# 2. Conflito: escrever no b.ts DEPOIS do renomear e desfazer no a.ts
sessao
f2 3:15 ajuda Return; sleep 2.5
abre b.ts; xdotool key ctrl+End; xdotool type --delay 60 '// x'; sleep 0.5; shot r9a_b_escrito 1
abre a.ts; xdotool key ctrl+z; shot r9_conflito 2
xdotool key Return; shot r9b_enter_no_dialogo 2
abre b.ts; shot r9c_b_depois 1
fecha
# 3. Nome inválido, palavra reservada, nome que já existe no âmbito
sessao
f2 2:9 1x Return; shot r10_invalido 2.5
xdotool key Escape; xdotool key ctrl+z; sleep 1
f2 2:9 class Return; shot r11_reservada 2.5
xdotool key Escape; sleep 0.5
f2 2:9 helper Return; shot r12_existente 2.5
fecha
# 4. Prévia (Ctrl+Enter, como diz a caixinha)
sessao
f2 3:15 ajuda ctrl+Return; shot r13_previa 4
fecha
# 5. O realce das ocorrências com o cursor parado, e as referências (Shift+F12)
sessao
vai 3:15; shot r14_realce 2
xdotool key shift+F12; shot r15_referencias 3
fecha
fi
# 6. O ATRASO do realce: clique em `helper` (linha 1) e rajada de fotos (cada foto leva o seu relógio, porque
#    `import` não é instantâneo). O realce acha-se pela COR, nunca por um pixel fixo (o `p{552,135}` de 18/10
#    caía num glifo no 1.145.2): a cor do realce = a mais frequente da área do texto na ÚLTIMA foto que não
#    existe na foto ANTES; cada foto conta os seus pixels dessa cor; chegou = a 1.ª com ≥ metade da última.
AREA=1200x300+400+100   # a área do texto inteira (à direita do Explorer), larga de propósito
hist() { magick "$1" -crop "$AREA" +repage -format %c histogram:info:; }
sessao
vai 2:1; sleep 2.5
import -window root "$T/antes.miff"
t0=$(date +%s%N); xdotool mousemove 544 96 click 1
for i in $(seq 0 39); do
  import -window root "$T/burst$i.miff"; echo "$(( ($(date +%s%N) - t0) / 1000000 ))" >"$T/burst$i.t"
done
hist "$T/antes.miff" >"$T/antes.h"
for i in $(seq 0 39); do hist "$T/burst$i.miff" >"$T/burst$i.h"; done
python3 -I - "$T" >>"$OUT/passos.txt" <<'PY'
import re, sys
t = sys.argv[1]
def h(p):
    return {m[1].upper(): int(m[0]) for m in re.findall(r"^\s*(\d+):.*?#([0-9A-Fa-f]{6,12})", open(p).read(), re.M)}
antes, ult = h(f"{t}/antes.h"), h(f"{t}/burst39.h")
novas = sorted(((n, c) for c, n in ult.items() if c not in antes), reverse=True)
if not novas:
    print("realce: NENHUMA cor nova na área do texto (o clique não realçou)"); sys.exit()
n_ult, cor = novas[0]
curto = lambda c: "#" + "".join(c[i:i + 2] for i in range(0, len(c), len(c) // 3))[:7] if len(c) == 12 else "#" + c
print(f"realce: cor {curto(cor)} ({n_ult} px na última foto; antes 0 px); 2.ª cor nova {curto(novas[1][1]) if len(novas) > 1 else '-'}")
cheg = None
for i in range(40):
    n = h(f"{t}/burst{i}.h").get(cor, 0); ms = open(f"{t}/burst{i}.t").read().strip()
    if cheg is None and n * 2 >= n_ult: cheg = (i, ms)
    print(f"realce {i} t={ms}ms px_realce={n}")
print(f"realce CHEGOU na foto {cheg[0]} a {cheg[1]} ms" if cheg else "realce: nunca chegou a metade")
PY
fecha

echo "gravado em $OUT"; cat "$OUT/passos.txt"
