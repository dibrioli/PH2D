---
name: rustfmt-reorders-mods-and-steals-the-doc
description: Um `mod` novo escrito acima de um irmão herda o doc-comment DELE quando o `cargo fmt` reordena — e o irmão fica sem o seu, em silêncio.
metadata:
  type: feedback
---

**Escrever `mod novo;` na linha imediatamente acima de `mod irmao;` faz o `cargo fmt` reordenar os
dois alfabeticamente e deixar o doc-comment do IRMÃO colado ao NOVO.** O irmão fica sem doc nenhum,
o novo fica com um doc que descreve outra coisa, e nada acusa: o `check` é verde, o clippy é verde,
e o diff parece uma inserção limpa.

**Why:** medido em 2026-09-21 no `shells/desktop/src/render_loop/mod.rs` (199 declarações de `mod`,
cada uma com um doc de uma linha). O `mod fase_cataventos;` foi escrito acima do
`mod fase_relight_baked_forms;`; o `fmt` moveu-o para a posição alfabética e levou consigo o doc
*«a re-acendida dos objetos assados — **FORA** da feature `sculpt3d`, de propósito»* — que diz o
**CONTRÁRIO** da fase nova, que está **atrás** daquela feature. ⚠️ O que o torna caro é a
plausibilidade: um doc errado sobre um `mod` lê-se como verdade durante meses, e quem o ler decide
com ele. É a mesma família do corte que sobe por `///` e atravessa um item
([[a-sweep-that-climbs-by-doc-comment-cuts-through-an-item]]) — aqui quem corta é o formatador.

**How to apply:** ao acrescentar um `mod` a um índice, escreva-o **já na posição alfabética** e com
o doc próprio, corra `cargo fmt` e **releia as duas linhas acima e abaixo dele** no diff. O sinal de
que aconteceu é um irmão a perder o doc no mesmo diff em que o novo ganha um — as duas metades
aparecem juntas, e cada uma sozinha lê-se como outra coisa (*«alguém apagou prosa»* · *«alguém
documentou»*).

⚠️ E num ficheiro que é **índice** o tecto de LOC mede o número de entradas, não o autor: ali um
`mod` novo custa sempre 2–3 linhas (doc · `cfg` · `mod`) e a cura prescrita pelo próprio ficheiro é
**cortar narrativa de migração fechada** para os handoffs — nunca subir o número.
