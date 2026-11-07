---
name: feedback_a_wait_loop_on_pgrep_finds_itself
description: Laço que espera por pgrep -f ou ps|grep acha o próprio shell e nunca acaba; trabalho longo começa direto, nunca atrás de uma espera, e confere-se que começou
metadata:
  type: feedback
---

Em 07/11 a suíte da integração ficou **5 horas** sem começar, com o dono fora e a máquina parada, atrás de
`while pgrep -f 'cargo check -p x'; do sleep 2; done; bash scripts/integrar-rodada.sh suite`. O padrão está na
linha de comando do próprio shell, que se acha a si mesmo. Os transcritos tinham outros 31 laços desses.

**Why:** o arnês só avisa quando o comando de fundo termina, e um comando preso não termina: a falta de aviso
lê-se igual a «ainda a trabalhar». O guarda só recusava `while true`/`until` e deixou passar `while <condição>`.

**How to apply:**
- Trabalho longo começa DIRETO. Nada de encadear o pesado atrás de uma espera por outro processo.
- Na mesma resposta em que lança algo que levará mais de alguns minutos, confira uma vez, logo depois, que ele
  começou (o log existe e cresce, a carga subiu). Esperar o aviso só depois disso.
- Esperar um processo mesmo: `timeout 900 tail --pid=<PID> -f /dev/null`, ou `pgrep -x <programa>` com prazo.
- O guarda `.claude/hooks/tecto-de-recursos.sh` (R10 e R2) agora recusa o laço e a espera sem prazo. Ele não pega
  todo travamento possível: por isso a conferência continua a ser regra.

Família: [[feedback_killing_the_parent_of_an_orphan_kills_the_owners_session]] (outro acidente de gestão de processos).
