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
([`crates/ph2d-app-flip/src/fit_cache_tests.rs`](crates/ph2d-app-flip/src/fit_cache_tests.rs)) é um
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

---

## 2026-09-19 — mais UM membro, e o doc-comment dele narra a PRÓPRIA cura de uma flake

`ph2d-tool-painter tool::paint::tests::measure_input_cost::the_pen_down_is_still_a_canvas_copy_and_this_is_its_number`
— **único ✗ de `24 546`** numa corrida de workspace da `line/UIUX`, com **zero** linhas do diff
naquela crate (a linha mexeu em `ph2d-editor-core` e em dez painéis), e verde **3 de 3** sozinho a
`load 20,72` · `21,58` · `21,58`.

⛔⛔ **É o SÉTIMO deste repo cujo doc-comment se declara imune — e este declara-o contando como já
foi curado de uma flake:**

> *«⚠️ **O oráculo é a RAZÃO contra a CÓPIA DO CANVAS medida no mesmo instante**, e a primeira
> versão errou isso: ela comparava o pen-down a 1024² com o de 4096², que são dois instantes
> diferentes — sob a carga da suíte completa os dois flutuam de forma independente e o gate
> **flakou na 1ª rodada**. … Medidos juntos, os dois números sobem e descem juntos.»*

⭐⭐⭐ **A frase está certa sobre a DERIVA e falsa sobre o FAN-OUT**, que é exactamente a distinção
que esta família existe para guardar: medir os dois no mesmo instante cura o *drift* lento da
máquina (os dois sobem juntos ao longo de segundos) e **não** cura uma interrupção de escalonamento
que caia sobre UM dos dois `ms(&mut || …)`, que duram microssegundos. *Duas medições «no mesmo
instante» são dois instantes diferentes à escala em que o fan-out morde.*

⚠️ E a linha **acredita** na cura ao ponto de escrever *«um gate que flaka é pior que ausente»* logo
a seguir — o que faz dele o caso mais difícil de apanhar por leitura: não é um gate descuidado, é um
gate **já uma vez endurecido contra a grandeza errada**.

**How to apply:** promoção pedida à lista nomeada do `CLAUDE.md` §5.0 — **a linha pede, o integrador
escreve** (DIRETRIZ §1.5.9). Antes de culpar um diff por este ✗, confira as três assinaturas:
gate de razão · zero linhas do diff naquela crate · 3/3 verde sozinho **com o `loadavg` impresso ao
lado**.


---

## ⭐ **Candidato NOVO, medido pela `line/motion-value` em 2026-09-22**

`tool::paint::tests::measure_input_cost::the_pen_down_is_still_a_canvas_copy_and_this_is_its_number`
([`crates/ph2d-tool-painter/src/tool/paint/measure_input_cost.rs`](crates/ph2d-tool-painter/src/tool/paint/measure_input_cost.rs))
— **gate de RAZÃO entre dois relógios, e o doc dele di-lo por escrito**: *«O oráculo é a RAZÃO
contra a CÓPIA DO CANVAS medida no mesmo instante»*.

| assinatura | leitura |
|---|---|
| mede um recurso partilhado | **sim** — divide dois relógios, e é declarado no doc |
| zero linhas do diff acusado naquela crate | **sim** — a rodada era do LOD da forma (`ph2d-app-motion` + `ph2d-eval-motion` + shell) |
| verde sozinho, com a carga impressa | `3 de 3` a `load 5,4` / **`97`–`98 %` de CPU ociosa** |

⚠️ **A 3.ª assinatura é mais FRACA aqui do que nos membros de 10/09**, e isso está declarado: lá a
confirmação correu a `load 17`–`19` (mais carga do que aquela em que tinham reprovado), o que
mostra que o discriminador é o **FAN-OUT** e não o relógio. Aqui a máquina estava **calma**, logo o
que está provado é *«passa com a máquina calma»* — não *«passa sob a mesma carga»*.

⭐⭐ **Mas a QUARTA assinatura fechou-o, e ela é a mais forte de todas:** a **MESMA árvore**, o
**mesmo commit**, corrida outra vez, deu `18 796` de `18 796` — o Painter **não repetiu**. *Um
defeito de lógica reprova o mesmo caso sempre; só um recurso partilhado troca de vítima entre
corridas.*

⇒ **Promoção PEDIDA ao integrador** (a linha pede, o integrador escreve no `CLAUDE.md` §5.0).

**Why:** ele reprovou no `nextest-impacted` (`18 796` testes em paralelo) de uma rodada que não toca
uma linha do Painter, e um agente que leia o vermelho como defeito vai procurar a causa num diff
que não a contém.
