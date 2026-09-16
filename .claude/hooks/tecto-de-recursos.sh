#!/usr/bin/env bash
# Guarda de recursos — corre ANTES de cada Bash de cada agente deste repo.
#
# POR QUE ISTO É UM HOOK E NÃO UMA LINHA NUM DOC (medido no próprio repo,
# CLAUDE.md §2): o `cargo-check-narrow.sh` está escrito no roteador há semanas,
# faz exactamente a coisa certa, e foi invocado **5 vezes em 101 sessões** contra
# **13 791** `cargo check` digitados à mão; o `git-stage-guard.sh` tem 5 docs a
# apontá-lo e **zero** invocações. *Ponteiro não é adoção.* Os quatro scripts
# vivos do repo têm uma coisa só em comum: um passo obrigatório invoca-os pelo
# nome. Aqui o passo obrigatório é este ficheiro.
#
# Ele recusa DUAS formas, as duas medidas em 14–15/09:
#   R1  comando pesado fora da porta  → uma linha podia tomar a máquina inteira
#   R2  vigia de fundo sem prazo      → 9 laços de `sleep` ficaram a girar, e um
#                                        deles escondeu uma sonda pendurada 56 min
#
# ⚠️ **Ele FALHA ABERTO por desenho.** Qualquer coisa inesperada — jq ausente,
# JSON que não lê, estado que não reconhece — devolve 0 e o comando passa. Um
# guarda que se engana a fechar pára seis linhas; um que se engana a abrir volta
# ao que havia antes dele.
set -uo pipefail

entrada="$(cat 2>/dev/null)" || exit 0
command -v jq >/dev/null 2>&1 || exit 0
cmd="$(printf '%s' "$entrada" | jq -r '.tool_input.command // empty' 2>/dev/null)" || exit 0
[ -n "$cmd" ] || exit 0
fundo="$(printf '%s' "$entrada" | jq -r '.tool_input.run_in_background // false' 2>/dev/null)"

recusa() { # $1 = porquê · $2 = a correcção
  jq -n --arg r "$1"$'\n\n'"$2" '{hookSpecificOutput:{hookEventName:"PreToolUse",
     permissionDecision:"deny", permissionDecisionReason:$r}}' 2>/dev/null
  exit 0
}

# ── R1 · comando pesado tem de passar pela porta ──────────────────────────────
# ⚠️ O laço interno NÃO entra aqui: `cargo check/fmt/tree/metadata` custam 1–3 s e
# são a resposta certa a «a minha edição entrou?» (CLAUDE.md §2). Encarecê-los
# empurraria os agentes de volta para o `cargo test`, que é o defeito 4,3:1 que o
# repo já mediu.
if printf '%s' "$cmd" | grep -qE '(^|[;&|[:space:]])cargo[[:space:]]+([^;&|]*[[:space:]])?(test|nextest|build|run|bench|clippy)([[:space:]]|$)'; then
  if ! printf '%s' "$cmd" | grep -q 'ph2d-run\.sh'; then
    recusa "⛔ Comando pesado fora da porta de recursos.

Sem a porta, esta linha pode tomar a máquina inteira — e foi isso que estragou os
smokes do dono em 14/09. A porta põe o comando numa fatia (cgroup) que é da LINHA:
CPU ≤ 50 % dos núcleos, memória ≤ 24G, prazo de 30 min, e mata a árvore INTEIRA no
fim (um binário de teste reparenta-se e sobrevive a um \`timeout\` — medido)." \
"Refaça assim:

  bash scripts/ph2d-run.sh ${cmd}

Se o comando toca na PLACA (gates de GPU, smoke, sondas de device):

  PH2D_GPU=1 bash scripts/ph2d-run.sh ${cmd}

Precisa de mais tecto? MEÇA e diga o número (§0.0):
  PH2D_PRAZO=5400  ·  PH2D_CPU_PCT=75  ·  PH2D_MEM_MAX=48G
Runbook: docs/DevOps/TETOS_DE_RECURSO_POR_LINHA.md"
  fi
fi

# ── R2 · vigia de fundo tem de ter prazo ──────────────────────────────────────
# Um `until …; do sleep 60; done` em segundo plano nunca termina sozinho: se a
# condição não chegar, ele fica a girar para sempre e a sua ausência de saída
# lê-se exactamente como «ainda a trabalhar».
if [ "$fundo" = "true" ]; then
  if printf '%s' "$cmd" | grep -qE '(while[[:space:]]+(true|:)|until[[:space:]]|for[[:space:]]*\(\(;;)'; then
    if ! printf '%s' "$cmd" | grep -qE '(^|[;&|[:space:]])timeout([[:space:]]|$)'; then
      recusa "⛔ Vigia de fundo sem prazo.

Um laço de espera em segundo plano sem tecto de tempo não termina sozinho, e o
silêncio dele lê-se igual a «ainda a trabalhar». Em 14/09 nove destes ficaram a
girar e um deles escondeu uma sonda pendurada a segurar a GPU durante 56 minutos." \
"Ponha um prazo — e prefira um prazo CURTO com uma segunda espera a um prazo longo:

  timeout 900 bash -c '${cmd}'

⚠️ E antes de armar um vigia, pergunte se precisa dele: quando o trabalho é um
comando de fundo desta sessão, o próprio arnês avisa ao terminar. Vigia só para
estado que o arnês não vê (CI, uma fila remota)."
    fi
  fi
fi

exit 0
