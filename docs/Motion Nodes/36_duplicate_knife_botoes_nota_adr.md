# 36 — Editor F2: botões (pan/seleção), Ctrl+D e a faca — nota-ADR

**Data:** 2026-07-12 · **Linha:** `line/motion-value` (Modo L) · **Fase:** editor **F2**
**Status:** implementado, testado, **pendente smoke do Enio**
**Contrato congelado encostado:** **nenhum** · **Foundational tocado:** `ph2d-tokens` (1 token novo, aditivo)

---

## 1. Os botões (correção do smoke)

O botão **esquerdo panava** a tela — o que deixava a multi-seleção **sem onde morar**. A convenção de
editor de nós (Blender · Nuke · Houdini) é **meio = pan / esquerdo = seleciona**, e agora é isso:

- **MEIO = pan, de QUALQUER lugar** da superfície (sobre um card, um fio, um backdrop): o grafo desliza
  sob o cursor. O dispatch **já entregava todos os botões** ao grafo — o comentário do `pointer_down`
  literalmente dizia *"a middle-drag reaches the graph pan"*. O painel é que tratava Primary e Middle
  igual. Zero foundational.
- **ESQUERDO no vazio = RUBBER BAND**, que **nunca existiu** no painel (era M1.E5 e nunca landou). Pega
  todo card que a banda **TOCA** (interseção — Blender e Nuke idem; exigir o card inteiro dentro
  perderia justamente os que a banda cruzou). **Shift** = aditivo; sem Shift, substitui.

**Token novo `graph-marquee`** (o accent de cada tema a 18% de alpha; nos 3 temas que carregam a
família `graph-*`). Translúcido **por contrato**: a banda é desenhada POR CIMA dos cards que está
selecionando, e um fill opaco os esconderia. Cor literal em painel é barrada pelo gate
`no_literal_color` — o rubber-band do canvas usa literal *baked* (dívida documentada lá), o painel não
pode. Borda = `ColorToken::Accent`.

## 2. Ctrl+D — duplicate

Copia os nós selecionados com **params**, **text params** e os **fios ENTRE eles**, deslocados.

- **Text params importam:** copiar só os `f32` devolveria um `motion.expression` **sem fórmula** (o
  canal de texto é um segundo mapa — doc 32). Gate: `duplicate_carries_the_text_param_too`.
- **Fio de FORA não é copiado.** Um duplicado é uma coisa nova pra posicionar, **não** um segundo
  consumidor emendado no upstream alheio. Blender (Ctrl+D) e Nuke (copy/paste) também só levam os links
  internos. Gate: a cópia do `move` não alimenta o `output` original.
- Nó sequencial duplicado tem o **self-loop `pre` re-plumbado** por `reconcile_after`, igual a um solto
  do add-menu. **1 passo de undo** pra tudo.

**As CÓPIAS viram a seleção** — via um canal novo `request_graph_selection` (**shell → painel**), o
único que corre **contra** a direção usual, e tem de correr: só o shell conhece os ids que acabou de
mintar, e se os originais seguissem selecionados, o arrasto que naturalmente segue um Ctrl+D moveria os
**ORIGINAIS**, em silêncio. Gate: `the_shells_new_ids_become_the_selection`.

## 3. Knife

**K arma** a lâmina; o próximo arrasto esquerdo corta todo fio que o traço **cruza** (teste de
orientação/straddle contra a polilinha do fio, amostrada mais denso que o desenho — uma polilinha
grossa deixa passar um traço que visivelmente cruza a curva entre duas amostras).

- **UM `CutWires` pro traço inteiro = 1 passo de undo.** Uma faca que corta 5 fios e exige 5 Ctrl+Z é
  uma armadilha.
- **O traço desarma a lâmina** (e Esc, e um segundo K): um modo do qual não se sai é uma armadilha, então
  ele tem três saídas.
- **Fios `pre` são pulados** no painel (não têm spline na tela — desenham como badges de portal) **e
  recusados no shell** com o mesmo toast do alt-click. Duas barreiras: o state loop é plumbing do editor,
  não fio que o artista desenhou.
- Traço em `ColorToken::Danger` — o corte é destrutivo, e a cor diz isso.
- Faca armada **suprime o rubber band** (os dois dividem o mesmo botão e nunca podem disparar juntos).

## 4. Verificação

**12 gates novos** (7 painel + 5 shell). Os que mordem de verdade:
`a_middle_drag_pans_from_anywhere_even_over_a_card` (e não seleciona nem move o card) ·
`a_left_drag_on_empty_canvas_band_selects_what_it_touches` (e o view **não** pana) ·
`shift_makes_the_band_additive` · `duplicate_copies_params_and_the_wires_between_the_copies_only` ·
`duplicate_carries_the_text_param_too` · `duplicate_is_one_undo_step` ·
`the_knife_cuts_every_crossed_wire_in_one_undo_step` · `the_knife_refuses_managed_state_wiring` ·
`an_armed_knife_suppresses_the_rubber_band`.

## 5. Smoke

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && cargo run -p ph2d-host-desktop
```
Tool Motion: **meio-arrasta** (pana de qualquer lugar) · **esquerdo-arrasta no vazio** (banda; Shift
soma) · selecione e **Ctrl+D** (as cópias nascem selecionadas — arraste e são elas que vão) ·
**K** e arraste cruzando fios (corta; um Ctrl+Z devolve o traço inteiro).

## 6. Aberto no F2

**Probe + sparkline** (ring de 60) · **smart-connect popup** (busca fuzzy + auto-inserção de adapter) ·
waypoints/branches · readouts inline no body · template "nó sequencial". A tecla `GraphKey::Probe` já
existe — mesma história de sempre: trilho posto.
