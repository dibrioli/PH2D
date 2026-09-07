---
name: the-cleanup-runbook-measures-the-wrong-filesystem
description: A DIRETIVA_FIM_DE_DIA mede `df -h /` e o PH2D vive noutro disco — o «número que é a prova» leu 206G→206G sobre 435 GB apagados
metadata:
  type: feedback
---

**Medido em 2026-09-06, ao EXERCITAR o runbook:** o `§4` da
[`DIRETIVA_FIM_DE_DIA.md`](../docs/IntegracaoMultiAgente/DIRETIVA_FIM_DE_DIA.md) imprime
`df -h / | awk 'NR==2{...}'` **antes e depois** da limpeza, e o `§5` item 1 chama esse par de
*«Disco antes → depois (o número é a prova)»*.

Nesta máquina ele mede o **filesystem raiz** (`950 G`), e o PH2D vive num **disco dedicado de 2 TB**
montado em `/home/enio/Documentos/Projetos` ([[project_projects_live_on_a_dedicated_2tb_disk]]).
⇒ A limpeza libertou **435 GB** (`741G → 306G`, lido pelo `btrfs-health.sh`) e as duas linhas do
runbook imprimiram **`206G de 950G (22%)` as duas** — idênticas, ao lado de meio terabyte apagado.

**Why:** é a família favorita deste repo — *uma régua que mede outra coisa não fica vermelha, fica
MUDA*. Pior que ausente: o `§5` promove-a a **prova**, então um agente que corra o runbook e cole a
saída relata honestamente um número que não descreve o trabalho que fez. E o facto que a desmente
já estava na memória — *o conhecimento existia e não estava no CAMINHO de quem executa*
([[feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it]]).

⚠️ **O que salvou o relatório foi o `btrfs-health.sh`**, que o próprio runbook manda correr antes e
depois por outro motivo (metadata/checksum/swap) e que **nomeia o mountpoint certo**. Duas réguas da
mesma grandeza a discordar na mesma página é o achado
([[feedback_a_ruler_placed_after_the_tidying_step_measures_the_tidying]]).

**How to apply:** a cura é uma linha — `df -h "$(git rev-parse --show-toplevel)"` em vez de
`df -h /`, nos **dois** sítios do `§4`. Enquanto não entrar, **cite o `btrfs-health.sh` como o
número do antes/depois**, nunca o `df` do runbook. E a lei geral: *um `df` sem caminho mede o
filesystem de onde se está, que raramente é o que se acabou de apagar.*
