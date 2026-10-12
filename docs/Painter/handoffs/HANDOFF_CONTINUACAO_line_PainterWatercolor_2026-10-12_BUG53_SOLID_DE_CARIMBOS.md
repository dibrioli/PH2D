# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, o Bug #53: o Solid de carimbos e os fios na cor da ponta (2026-10-12)

## As regras (R1–R9)

As mesmas do [handoff do #52](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-12_BUG52_PONTA_CORTADA.md)
(janela curta, `Edit`, teste por nome, bateria 1× no fecho pelo `verificador`, R9 ≤ 10 s, `ph2d-run.sh`).
⚠️ Esta janela passou do R1 (~150 chamadas): o #53 veio do smoke do #52 na mesma sessão, por ordem do dono.

## §1 — Coordenadas

Sobre `e9e749034e` (o #52). Um commit. Sem push. O `main` não andou (merge-base `8f20c60de2`).

## §2 — O pedido (smoke do dono 12/10, duas fotos)

*«Com Solid e jitter, com imagem em Shape, o centro em vez de se encher de dabs com a textura está a ser
preenchido pela cor de Color»* — a foto: um anel de folhas e o miolo BRANCO (o «Use Texture Colors» pôs o
Color a branco). *«E veja se Type está coerente com textura em Shape e jitter.»*

## §3 — O que mudou

- **A lei** (`ph2d-painter-brush/src/solid_carimbos.rs`): `fill_dabs(spec, loops, w, h, seed)` — grade
  ALTERNADA ancorada no CANVAS com o passo do pincel (piso de ½ diâmetro: `PASSO_MINIMO_EM_DIAMETROS`, o Spacing
  é lisura AO LONGO da linha, que uma área não tem); cada ponto com cobertura `≥ ½` vira um `Dab` com o SEU
  sorteio (escala → rotação → cor → posição, a ordem do `dab_from`; o «a cada traço» respeitado), semeado pela
  semente do traço e pelas coordenadas inteiras ⇒ **estável quando a forma cresce**; empilhamento por uma chave
  sorteada. `BrushSpec::solid_fills_with_stamps` = Shape de IMAGEM com pixels (redonda/procedural: a mancha lisa).
- **Tool** (`solid_carimbos.rs`): `solid_fill_stamps()` dos laços ESPELHADOS (`solid_mirrored_loops`, novo; o
  Tiling vem do despacho). Cumulativo: na transação da mancha, carimbados como a CORDA
  (`stamp_corda_em_rascunho`), a região cresce pela pegada deles (`tiled_chord_region`). Re-carimbo
  (Elipse/Linha/…): juntam-se ao lote POR BAIXO do traço no `stamp_drag_preview` (o commit é o preview).
- **Type:** Speed, Rough e o corpo da Ribbon já eram carimbos (a ponta e o jitter valem). Sketchy, Wire e as
  travessas da Ribbon são FIOS e pintavam `brush.color` — branco no modo de cor ⇒ fio invisível, e a 50 % de
  opacidade DESBOTAVA as folhas (6 950 px). Agora `write_threads` pinta `ShapeLayers::tip_mean_colour` (média
  da ponta pesada pela silhueta, `textura × tinta`) quando a cor da textura pinta.

## §4 — Portões

- Tool `solid_carimbos` (4 + sonda): o miolo da mão livre E da Elipse > 90 % folhas (antes 0 %: VERMELHO visto
  com a porta desligada) · Ctrl+Z devolve a tela · sem imagem a mancha continua lisa · `todo_type_pinta_na_cor_da_ponta`
  (6 Types; VERMELHO com o fio no Color: «Sketchy 6 950 de 11 538 px fora da cor»).
- Motor (3): a nuvem estável quando a forma cresce (mutação «sorteio pelo índice» SANGRA) · o Spacing fino não
  multiplica · sem região, nada.
- `sonda_solid_de_folhas` (ponta em folha, PPM em `PH2D_SONDA_OUT`): 87 % do miolo, o aspecto da foto 2 do dono.

## §5 — Fecho

fmt ✓ · clippy `-D warnings --all-targets` (brush, tool) ✓ · `nextest-impacted` **17 132/17 132** (`load` 5–27).

## §6 — Aberto

- **Wet Paint / Watercolor:** o Solid lá é ÁGUA (enche ao soltar / cobertura da aguada) e não lê a cor da
  textura — não tocado.
- **Fios com Randomize Color «a cada carimbo»:** o fio tem UMA cor por lote (a média da ponta, ou o Color);
  uma cor por fio pede o `Thread` com cor (`[f32; 4]` hoje) e um rasterizador de cor — por decidir.
- A Ribbon não se mede sem relógio entre eventos (a mola não anda: 32 px redonda, 34 com folha).
