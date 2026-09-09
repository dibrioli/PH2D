---
name: reference-flip-fit-cache-ratio-is-a-load-flake
description: "O gate `the_cache_makes_a_preview_frame_cost_the_tail_not_the_stroke` (shells/desktop/src/flip_fit_cache_tests.rs) é membro da família de flakes de carga do CLAUDE.md §5.0 — medido 2026-09-09."
metadata: 
  node_type: memory
  type: reference
  originSessionId: af27d1c2-3a56-4abe-9acd-e2c91caf58f0
  modified: 2026-09-09T22:32:38.227Z
---

`flip_smooth::resample_measurement::precisao::cache::the_cache_makes_a_preview_frame_cost_the_tail_not_the_stroke`
([`shells/desktop/src/flip_fit_cache_tests.rs`](shells/desktop/src/flip_fit_cache_tests.rs)) é um
**gate de RAZÃO ENTRE DOIS RELÓGIOS** (`ms_sem / ms_com`, barra `> 3.0`) e pertence à família de
flakes de recurso sob fan-out do `CLAUDE.md` §5.0.

Medido em 2026-09-09 pela `line/UIUX`, com as **três** assinaturas da família:

| assinatura | leitura |
|---|---|
| mede um recurso partilhado | sim — divide dois relógios |
| zero linhas do diff acusado naquele módulo | sim (a wave era das abas de painel) |
| verde sozinho **com a carga impressa ao lado** | `3 de 3` a `load 4,69`, razão `4,8×`–`4,9×` |

E a quarta, indirecta: a mesma árvore correu a suíte do shell **outra vez** sob carga ~14 e o
teste **não repetiu** a falha.

**Why:** sob `load 23` ele afunda abaixo de `3,0` e reprova; um agente que leia o vermelho como
defeito vai procurar a causa num diff que não toca aquele módulo. E ⚠️ a régua que desmente a
flake é a própria flake se a confirmação correr sob carga — imprima `/proc/loadavg` **ao lado** de
cada corrida de confirmação (lição já registada em [[feedback_a_flake_red_hides_the_rest_of_the_suite]]).

**How to apply:** ao ver este nome vermelho, re-corra-o **sozinho e com a máquina calma** antes de
olhar para o commit. A promoção dele para a lista nomeada do `CLAUDE.md` §5.0 é trabalho da
**integração** (o precedente é `the_cost_of_a_player_is_linear_in_their_number`, promovido em
2026-09-07 «a pedido da `line/UIUX`») — a linha pede, o integrador escreve.
