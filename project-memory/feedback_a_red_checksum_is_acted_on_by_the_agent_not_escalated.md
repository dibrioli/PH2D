---
name: feedback_a_red_checksum_is_acted_on_by_the_agent_not_escalated
description: "Enio (2026-08-22): «checksum» vermelho no fim do dia não se sinaliza e espera — o próprio agente que corre a diretiva acha os arquivos, apaga/restaura, regista o A/B do kernel e devolve ao Enio SÓ o passo que exige reboot ou senha, já com o comando"
metadata:
  type: feedback
---

Ao fechar o dia de 22/08 eu escrevi no relatório «se a linha *checksum* vier vermelha, me
chame». O Enio corrigiu na hora: **quem está a correr a diretriz de fim de dia toma as
atitudes necessárias** — o protocolo está na DIRETIVA_FIM_DE_DIA §6 (kernel? → `--scan` de
targets, sccache, fonte e `.git` → apagar o que regenera / `git checkout` no que é fonte limpa /
listar o resto → sem balance nesse boot → registar no runbook e na memória → relatório com o
único passo dele, com o comando).

**Why:** «destaque no topo e aguardo instruções» transfere para o dono do produto um trabalho
que é inteiramente do agente (encontrar e limpar artefatos corrompidos não exige senha nem
reboot), e deixa no disco, até ao dia seguinte, arquivos que derrubam o `mold` em laço de 10 min.
O que de facto só ele pode fazer — reiniciar, instalar o memtest — cabe numa linha com o comando.

**How to apply:** em qualquer sinal de corrupção (csum no journal, `mold` SIGBUS, `rustc`
SIGSEGV não-determinístico), o gesto é o §6, não a escalada. E a regra generaliza: *o que o
agente PODE fazer sem senha e sem reboot, ele faz; o relatório leva o que fez e o resto em
comando pronto.* Ver [[project_btrfs_metadata_starved_not_disk_full_2026_08_22]] e
[[feedback_decide_dont_ask_gold_standard]].
