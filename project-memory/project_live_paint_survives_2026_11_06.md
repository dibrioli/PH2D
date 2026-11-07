---
name: project_live_paint_survives_2026_11_06
description: "Dono 06/11: a tinta do Wet fica viva ao trocar de modo/ferramenta sem pesar nos outros modos; opção ao salvar; a Borracha é inteligente (real na viva, imita na morta) e leva juntos os traços por cima"
metadata:
  type: project
---

Dabber + trocar de modo mata a tinta e a Relief Eraser vira corte seco. Decisões do dono (06/11): (1) manter viva ao trocar de modo e ferramenta, sem reduzir a velocidade dos outros modos; (2) traço de outro modo por cima: a Borracha leva os dois; (3) **opção** ao salvar o projeto, não sempre; (4) a Borracha age de verdade na viva e imita o comportamento certo na morta.

**Why:** a borracha viva é a que ele acha realista; arquivos sem a opção continuam a existir, por isso a imitação.

**How to apply:** plano, custos medidos e pesquisa em `docs/Painter/TINTA_VIVA_2026-11-06.md`. Depois disto a fila volta a `docs/Painter/PROXIMA_FASE_2026-11-06.md`. Ver [[project_next_phase_painter_2026_11_06]].

Onda 2 pronta (dentro do Wet, Borracha na morta, compressão zstd): [handoff](../docs/Painter/handoffs/HANDOFF_CONTINUACAO_2026-11-06_tinta_viva_onda2.md); aguarda o veredito do dono.
Lição: `bytes_retidos` conta 0 com o motor no worker (`EngineSlot::Away`): mede-se depois de o trazer (577 MB eram 5 GB).

**Dono 06/11: a Borracha na tinta morta está APROVADA pela foto** (`material_2026-11-06/foto_borracha_morta.png`). As constantes de `borracha_morta.rs` ficam; os passos 2, 3 e 5 do smoke ainda sem veredito.

**Dono 06/11, smoke da rodada da Relief Eraser: OK.** Decisão: com pouca pressão a Borracha TAMBÉM chega ao papel, em mais passadas (hoje o expoente 8 só amassa); mexe na lei aprovada, logo foto nova ao dono. Isto decide `a_borracha_leva_o_traco_e_o_oleo_juntos`. Pergunta aberta do dono: «porque a tinta morre com UNDO?» Lido no código, não reproduzido: `HISTORY_DEPTH = 6` e o orçamento em `undo_vivo.rs` cortam a fotografia, e o undo sem fotografia acaba a sessão (`wet_volta_do_restauro`), como no 1.º traço da sessão.

**Estado 06/11 noite (commits `2ad96106ad` undo · `325d15de9b` mão leve + alfa · `e8444946e7` régua/foto · `648108c5ee` engasgo do óleo · `b798272888` régua):** a pergunta «porque a tinta morre com UNDO?» foi REPRODUZIDA e curada (12 traços morriam no 7.º undo; `HISTORY_DEPTH = 6`, orçamento só das fotos, sessão nascida depois do pen-down; agora `fundo_da_historia`, `teto_da_historia` contando registos + fotos, `foto_em_tiles.rs`). A cena exacta do dono não foi dada: falta o olho dele. A Borracha com pouca pressão chega ao papel: `PISO` 0,30 em `borracha_morta.rs` LIGADO, aguarda veredito pela foto `foto_borracha_pouca_pressao.png`. Os «5 pontos» eram arredondamento do alfa em `escreve_o_sulco` (curado). O engasgo de adormecer após o traço curou-se no Dabber (4K 1721 → 0,5 ms); Wet Round NÃO curado (4K 245/1282 ms; compor por linhas não chega às metas).

**Decisões pendentes do dono:** veredito da foto da mão leve · o que fazer com o resto da foto (`foto_resto_da_borracha.png`: manchas escuras, fio claro no contacto) · a aguada · Wet Round (imagem atrasada × baixar custo da ótica × aceitar) · o passo de troca de modo sem veredito próprio. Provas de mutação: outro agente, ainda a correr. Falta (depois de integrar) a opção de manter a tinta viva ao salvar. Detalhe: seção «Onda do undo, da mão leve e do fecho» do handoff da onda 2.
