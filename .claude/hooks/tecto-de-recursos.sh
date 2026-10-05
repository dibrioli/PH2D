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
# (e depois: R3, o fonte de um alvo amuralhado, 18/09; R4, um `kill` que fecha a
#  SESSÃO do dono — o `systemd --user` morto por engano em 05/10.)
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

# ── R3 · o fonte de um alvo AMURALHADO não se lê ──────────────────────────────
# ⛔⛔ O `docs/_ComoInvestigarApps/00_o_metodo.md` §0 afirmava que ninguém *podia*
# ler o fonte de um alvo restrito, «porque o `.claude/settings.local.json` nega os
# caminhos». MEDIDO em 2026-09-18, ao abrir a parede da Unreal: **não existia
# lista `deny` nenhuma — em ficheiro nenhum** (nem no repo, nem em `~/.claude/`).
# A parede era uma promessa a declarar-se propriedade da máquina, que é a família
# que este repo mais paga: *um doc que declara a lei que o código não implementa
# lê-se como auditado.* Esta regra torna-a real para o `Bash`; o `deny` do
# `.claude/settings.json` cobre a ferramenta `Read`. As duas juntas são a frase.
#
# A unidade é o ARTEFACTO INSTALADO e nunca o nome do projecto (CLAUDE.md §0.9):
#   · Unreal 5.8.2 — build promovida sob EULA proprietária, e ela traz
#     `Engine/Source` e `Engine/Shaders` dentro do que se instala;
#   · Blender — GPL, com 810 ficheiros `.py` em `scripts/`.
# ⚠️ O `datafiles/` do Blender fica de FORA **de propósito**: o
# oráculo de cor deste repo lia o `config.ocio` (o `docs/Render3d/ferramentas/oraculo_de_cor.py`,
# que saiu com o 3D — ADR-0179; OpenColorIO, BSD-3) — isso é DADO e não implementação, e é a mesma distinção
# que faz a SAÍDA de um alvo ser livre.
#
# ⚠️ Ela recusa a MENÇÃO e não só a leitura, e isso é deliberado. Separar «este
# caminho é um operando» de «este caminho está dentro de um padrão de busca» não
# se faz com um grep honesto — e o gate irmão `grep que MENCIONA` da prova existe
# exactamente porque a distinção é real. Aqui escolhe-se errar a FECHAR: um
# guarda de parede que erra a favor do alvo não é um guarda. O custo é escrever o
# caminho por outra ferramenta, e a recusa diz qual.
if printf '%s' "$cmd" | grep -qE 'UnrealEngine/Engine/(Source|Shaders)|/usr/share/blender/[^/]*/scripts'; then
  recusa "⛔ Fonte de um alvo AMURALHADO.

A Unreal (EULA proprietária) e o Blender (GPL) são oráculos que se CORREM, nunca
fontes que se leem — CLAUDE.md §0.9. Ler contamina: ~460 notas deste repo já
citaram nome interno de alvo restrito, e a dívida levou uma jornada a zerar.

⚠️ Se você é a janela do PRODUTO, isto não é um obstáculo a contornar: a resposta
que procura tem de vir de CORRER o alvo sobre entradas nossas." \
"Faça assim:

  · precisa do COMPORTAMENTO dele? peça uma corrida a uma janela E — ela monta o
    arnês FORA da árvore, corre sem interface e devolve NÚMEROS;
  · precisa da saída dele? a saída é livre (GPLv2 §0) e entra como fixtura com
    cabeçalho;
  · precisa só de NOMEAR o caminho (um doc, um briefing)? use a ferramenta Write
    ou Edit, que não passa por este guarda.

⛔ Se acha que precisa mesmo de LER, pare e reporte — é decisão do Enio, não sua."
fi

# ── R4 · a SESSÃO do dono não se mata ─────────────────────────────────────────
# Medido em 2026-10-05 18:30:03: um agente quis parar um `ph2d-run.sh` e correu
# `pp=$(ps -o ppid= -p <pid>); kill $pp`. O processo era ÓRFÃO (o shell que o
# lançou já tinha saído) e um órfão é adoptado pelo subreaper da sessão — o
# `systemd --user`. O SIGTERM chegou-lhe, e 26 ms depois o journal regista
# `Activating special unit Exit the Session`: VS Code, Chrome, todas as janelas do
# Claude e o ecrã do Enio acabaram num logout. Ele leu «o PC reiniciou».
# `kill`/`pkill`/`killall` só contam em POSIÇÃO DE COMANDO: um `grep` que os
# MENCIONA passa (o custo de errar a fechar aqui é travar quem documenta).
kill_cmd='(^|[;&|(`]|&&|\|\|)[[:space:]]*(sudo[[:space:]]+)?(kill|pkill|killall)([[:space:]]|$)'
sessao_recusa() { # $1 = o que foi visto
  recusa "⛔ Isto pode FECHAR A SESSÃO do Enio ($1).

Em 05/10 um \`kill\` do «pai» de um processo órfão matou o \`systemd --user\` — o
gestor da sessão — e o ecrã inteiro do dono caiu num logout (VS Code, Chrome,
todas as janelas do Claude). Um órfão é adoptado por ESSE gestor: «matar o pai»
de um processo que não é seu filho directo é matar a sessão." \
"Pare o que é SEU, pelo que é seu:

  · o PID do próprio processo:      kill <pid>        (confira antes:
                                     ps -o pid,comm,args -p <pid>)
  · o grupo inteiro do processo:    kill -- -<pgid>   (ps -o pgid= -p <pid>)
  · a fatia da linha (ph2d-run):    systemctl --user list-units 'ph2d*'
                                     systemctl --user stop <unidade>.scope

⛔ Nunca \`kill\` de um PID CALCULADO (ppid, \$PPID) nem de systemd, kwin, plasma,
code ou claude. Se acha que precisa mesmo, pare e reporte ao Enio."
}
if printf '%s' "$cmd" | grep -qE "$kill_cmd"; then
  # (1) o idioma exacto do incidente: matar um PAI calculado.
  printf '%s' "$cmd" | grep -qiE 'ppid|\$PPID' && sessao_recusa "kill de um PAI calculado"
  # (2) um PID que, AGORA, é da sessão gráfica ou do gestor dela — lido SÓ nos
  #     argumentos do próprio `kill` (um `sleep 1; kill <pid>` não é o PID 1, e o
  #     `%1` é a tarefa 1 do shell).
  protegidos="$(pgrep -u "$(id -u)" -x 'systemd|kwin_wayland|plasmashell|ksmserver|startplasma-way|plasma_session|dbus-broker|code|claude' 2>/dev/null | tr '\n' ' ')"
  alvos="$(printf '%s' "$cmd" | grep -oE "$kill_cmd[^;&|]*" | grep -oE '(^|[[:space:]])[0-9]+([[:space:]]|$)' | tr -d ' \t')"
  for p in 1 $protegidos; do
    printf '%s\n' "$alvos" | grep -qx "$p" && sessao_recusa "o PID $p é da sessão"
  done
  # (3) por NOME, ou TODOS os processos do utilizador.
  printf '%s' "$cmd" | grep -qE '(pkill|killall)[^;&|]*[[:space:]](-u|--user)([[:space:]]|$)' && sessao_recusa "todos os processos do utilizador"
  printf '%s' "$cmd" | grep -qE 'kill[[:space:]]+(-[A-Za-z0-9]+[[:space:]]+)*-1([[:space:]]|$)' && sessao_recusa "kill -1 = todos os processos"
  printf '%s' "$cmd" | grep -qE '(pkill|killall)[^;&|]*(systemd|kwin|plasma|ksmserver|sddm|dbus|code|claude)' && sessao_recusa "um processo da sessão, por nome"
fi
# (4) os comandos de logout, em qualquer posição.
if printf '%s' "$cmd" | grep -qE 'loginctl[[:space:]]+(terminate|kill)-(session|user)|systemctl[[:space:]]+--user[[:space:]]+(exit|kill|halt|poweroff|stop[[:space:]]+(plasma|graphical|default|exit))|org\.kde\.(Shutdown|ksmserver)'; then
  sessao_recusa "um comando de logout"
fi

exit 0
