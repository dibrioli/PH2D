---
name: reference-flip-fit-cache-ratio-is-a-load-flake
description: "DOIS membros novos da família de flakes de carga do CLAUDE.md §5.0, medidos pela line/UIUX — o cache do Flip (razão de relógios, 09/09) e o `interaction_dispatch_no_alloc` (contador de alocações, 10/09)."
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


---

## Segundo membro — `interaction_dispatch_no_alloc` (contador de ALOCAÇÕES)

`ph2d-editor-core::interaction_no_alloc interaction_dispatch_no_alloc`, medido em 2026-09-10 numa
corrida de **22 203** testes: **único ✗**, e verde **3 de 3** sozinho a `load 2,20`.

⭐ **A assinatura mais forte aqui não é o «sozinho» — é a POPULAÇÃO:** o ficheiro do teste **não
menciona `TextSystem` nem texto**, e o diff acusado (a cache de layouts da `ph2d-text`) não toca
uma linha da `ph2d-editor-core`. *Quando o caminho do teste não alcança o código mudado, a
pergunta «é flake?» já está respondida antes de se correr o teste outra vez.*

⚠️ Ele é da espécie **contador de alocações**, que o `CLAUDE.md` §5.0 já nomeia como espécie
própria (*«um contador de alocações parece imune a carga e não é: sob fan-out o alocador global
reutiliza arenas de outra maneira»*) — os dois membros que a lista tinha eram
`apply_from_doc_is_zero_alloc_steady_state` e `the_trusted_len_collect_allocates_once`. Este é o
**terceiro**.
