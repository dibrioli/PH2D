---
name: reference-flip-fit-cache-ratio-is-a-load-flake
description: "TRÊS membros novos da família de flakes de carga do CLAUDE.md §5.0, medidos pela line/UIUX — o cache do Flip (razão de relógios, 09/09), o `interaction_dispatch_no_alloc` e o `the_ui_clock_does_not_allocate_per_frame` (contadores de alocações, 10/09; o 2.º passa 3/3 a load 19, logo o discriminador é o FAN-OUT, não o relógio)."
metadata: 
  node_type: memory
  type: reference
  originSessionId: af27d1c2-3a56-4abe-9acd-e2c91caf58f0
  modified: 2026-09-09T22:32:38.227Z
---

`flip_smooth::resample_measurement::precisao::cache::the_cache_makes_a_preview_frame_cost_the_tail_not_the_stroke`
([`shells/desktop/src/flip/fit_cache_tests.rs`](shells/desktop/src/flip/fit_cache_tests.rs)) é um
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

---

## Terceiro membro — `the_ui_clock_does_not_allocate_per_frame` (contador de ALOCAÇÕES, o 4.º da espécie)

`ph2d-editor-core::ui_motion_no_alloc the_ui_clock_does_not_allocate_per_frame`, medido em
2026-09-10 numa corrida de **1 622** testes: **único ✗**, e verde **3 de 3** sozinho a
`load 18,20` · `18,99` · `18,99`.

⭐⭐ **A carga alta na confirmação é o que torna esta leitura FORTE, e não fraca.** A regra do §5.0
manda imprimir o `/proc/loadavg` ao lado da corrida que desmente a flake, porque uma confirmação
sob carga pode ser a própria flake. Aqui é o contrário: ele passa **três vezes a `load ~19`** ⇒ o
que o parte não é a carga da máquina, é o **fan-out dentro do processo** — o alocador global a
reutilizar arenas de outra maneira quando 1 600 testes correm em paralelo. *A espécie está bem
nomeada: o discriminador é o fan-out, não o relógio.*

⛔⛔ **E o doc-comment dele DECLARA-SE IMUNE, com todas as letras:**

> *«⚠️ **CONTADOR e não relógio, de propósito:** um kill de wall-clock nesta workstation mede o
> `load average` tanto quanto o código… Uma contagem de blocos é determinística **e não flaka**.»*

É o **quarto** doc-comment deste repo a dizer-se imune e a ser membro (os outros três estão
nomeados no `CLAUDE.md` §5.0). ⚠️ E a frase está *meia certa*, que é o que a torna perigosa: trocar
o relógio por um contador **cura a deriva da máquina** e **não cura o fan-out**. *Trocar o eixo em
que uma régua é frágil não é o mesmo que a tornar robusta.*

**How to apply:** os **três** membros que esta linha mediu (`the_cache_makes_a_preview_frame_cost_the_tail_not_the_stroke`,
`interaction_dispatch_no_alloc`, `the_ui_clock_does_not_allocate_per_frame`) esperam promoção para
a lista nomeada do `CLAUDE.md` §5.0 — **a linha pede, o integrador escreve** (DIRETRIZ §1.5.9).
