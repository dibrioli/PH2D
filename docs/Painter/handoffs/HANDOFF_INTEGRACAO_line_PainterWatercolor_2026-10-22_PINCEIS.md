# HANDOFF DE INTEGRAÇÃO — `line/PainterWatercolor`, missão Pincéis (J1–J5): fábrica 28 → 45 (2026-10-22)

> Leitor: o integrador e a próxima janela da linha. Missão: [`docs/Painter/Pinceis/`](../Pinceis/00_MISSAO.md) ·
> estado final [`ESTADO.md`](../Pinceis/ESTADO.md) · fila de motor [01_CATALOGO §3d](../Pinceis/01_CATALOGO.md) ·
> biblioteca [`exames/biblioteca.md`](../Pinceis/exames/biblioteca.md) · anterior:
> [P53](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-19_P53.md). Smoke do dono: **PENDENTE** (integrar não é aprovar).

## §1 — Coordenadas

- Branch `line/PainterWatercolor`; merge-base `100d34c8e0` (= o `main` quando o fecho correu); 314 commits, **nada
  integrado, nada enviado**. HEAD do fecho `b74b608d0e` (só config de nextest + doc depois de `48be9e05c0`).
- J5 = `git log --oneline 41b958338b..HEAD` (`33c747a11b` … `b74b608d0e`, 33 commits).

## §2 — O que entrega desde o P53

- **18 pincéis APROVADOS** pelo exame de 6 condições (prancha + revisão cega contra o Rebelle); fábrica **28 → 45**
  (Wet 29, Digital 16); 27 «curados, não aprovados» (ficam, com a diferença escrita em `exames/<família>.md`). O alvo
  80–120 da missão **NÃO foi alcançado**: o que trava o resto é motor ([§3d](../Pinceis/01_CATALOGO.md)).
- J5: **Pastel Side** aprovado (`c6c99afab7`); 7 curados, 2 reprovados — `ESTADO.md`.
- **Motor J5:** `Knob::WetOpen`/`WetOpenPush` (`ph2d-wet-paint` `trail/molhado.rs`; molhado × molhado; desligado por
  omissão; portão `o_azul_abre_uma_mancha_redonda_no_vermelho_molhado`, 3 mutações sangram), ligado no Wet Liner e Wet
  Wash; tecto da cauda 1,3 × meia-largura (`eb04c3440d`). Motor J1–J4: [HANDOFF_JANELA_4](../Pinceis/HANDOFF_JANELA_4.md)
  («Motor novo») e `HANDOFF_JANELA_2/3`.
- Biblioteca: nenhum par fora de família < 0,596; custo calmo p95 ≤ 12,5 ms a 200 px. Tutorial do dono:
  [`tutorial/01_pinceis.pdf`](../Pinceis/tutorial/01_pinceis.pdf) (12 páginas).
- **Grain Colors nos 4 meios** (`ae082ffb2e`): o grão indexa a rampa pela lei única `RampPaint::cor` (Aquarela: o mapa de
  granulação; Wet: o grão do dab, que deixa de morder); esmaece sem leitor (`CorDoGrao*`); o preset não grava a rampa.

## §3 — Fecho (1×, `PH2D_SUITE=fecho`)

- fmt ✓ · clippy workspace `--all-targets --features ph2d-spike/bevy_ecs -D warnings` ✓ · censos da árvore combinada
  12/12 e 121/121 ✓.
- nextest-impacted: **19 576 corridos, 19 574 ✓, 0 falharam, 2 TIMEOUT de carga**
  (`a_lei_da_tinta::o_glaze_do_pincel_escurece_o_cruzamento`, `::sem_glaze_cada_preset_de_fabrica_pinta_a_cena_de_antes`;
  sozinhos 8–17 s, 3/3) — promovidos à família de [flakes de carga](../../DevOps/FLAKES_DE_CARGA.md) em `b74b608d0e`
  (tabela + faixa isolada + faixa pesada; `architecture_toda_flake_de_carga_corre_na_faixa_isolada` verde).
- Antes disso o fecho achou 17 falhas reais, curadas (`d9d763b06d` … `002c75267b`, um commit por família: portas
  RGBA tipadas, 8 ficheiros > 700 LOC partidos, citações, `deposit_mask`, controlos mortos, sub-secções recolhidas,
  ordem do navegador) + 4 commits de estilo.

## §4 — Superfície de colisão (`bash <worktree>/scripts/collision-surface.sh`, base `100d34c8e0`)

- Schemas **todos iguais à base** (`PROJECT_SCHEMA` 191, tripla `(191, 13, 23)`, vec 23, flip 13, timeline 20);
  registos de componentes 108/109/109 iguais; contratos §6 intocados. A linha não mexe em schema: **nada a recontar**,
  mas confira no main do dia (`python3 <worktree>/scripts/schema-recount.py <valor_do_main>`).
- ADR: a linha criou **0189** (caneta, P46) — reconte contra o main do dia. `Cargo.lock`: só `ph2d-pen` (P46).
- «Marcadores de conflito» ✗ = **falso positivo**: 2 PNG binários em `fixtures/rebelle_bateria/OIL_AND_ACRYLIC/`.
  «Tecto de LOC» ✗ `app_state.rs` 923 e `main.rs` 1060 (shell): de antes (a linha tocou 3 linhas).
  `painter_bridge.rs` 666/700.
- Toca fora do Painter: `.config/nextest.toml` · `FLAKES_DE_CARGA.md` · `ph2d-i18n` · `ph2d-panel-painter-layers` ·
  `ph2d-panel-registry-init` · `ph2d-pen`/`ph2d-host` (P46) · `ph2d-editor-core` (P46 + o `header` do §8) · `shells/desktop`.

## §5 — Números a recontar no merge (contar, nunca escolher)

- `PRESETS_DE_FABRICA` agora é **calculado** (`receitas_de_fabrica.rs`: receitas + famílias + ferramentas Wet) = **45**;
  gate `the_factory_presets_are_counted`. Outra linha que some presets: reconte pelos ficheiros de `assets/brushes/`.
  PNG binários de preset em conflito: REGENERAR ([P51 §3](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-19_P51.md)).
- Gates de painel regravados: altura de `painter_layers` **1545** (2091 → 1554 → 1545 no §8; `quantas_entradas_tem_cada_painel.rs`)
  · tecto de nomes que não cabem na coluna 220 px: **15 → 13** (`27179b7792`, gate
  `cada_nome_deste_painel_cabe_na_coluna_da_seccao`) · painel do Painter volta ao tecto de 600 LOC (`265d8ebeb2`).
  Outra linha que some linhas ao cartão do Pen/Brush/Stroke muda 1545: reconte. Chaves `app.painter.presets.folder.*`
  (58) e `group_in_media` SAÍRAM (§8): outra linha que as use troca por `category.*`.
- LOC: 8 ficheiros partidos em `b515ce2549` (`wet-paint` `tools/bordas.rs`, `trail/{bico,medida}.rs`, `solver/gota.rs`,
  `solver/flow_jacobi/backrun.rs`; `tool-painter` `wetpaint/rota_da_lane.rs`; `painter-brush` `blend.rs`): quem
  conflitar em `dab_route.rs`/`tools.rs`/`trail.rs`/`solver.rs` reaplica o MOVIMENTO, não funde à mão.
- Flakes de carga: +2 membros (acima); a lista vive em `FLAKES_DE_CARGA.md` + 2 filtros do `nextest.toml` — duas escritas.
- Grain Colors (§2): `Dependente::TODOS` **41 → 44** (contar) · `ph2d-i18n/src/painter_layers.rs` **700/700** LOC.

## §6 — Smoke do Enio (binário `smoke` compilado)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && PH2D_WETPAINT_SMOKE=2 ./target/smoke/ph2d-host-desktop`
2. Abra o tutorial `docs/Painter/Pinceis/tutorial/01_pinceis.pdf` e siga uma página por categoria: no Painter, escolha o
   pincel da lista (45) e risque um traço; compare com a foto ao lado.
3. Tem de acontecer: cada pincel pinta com a cara do material e diferente dos vizinhos; os lápis vêm do mais duro ao mais macio.
4. Responda as duas perguntas do fim do PDF: **decisão 5** (a cauda do traço afina só no Digital) e **decisão 8**
   (tamanho máximo por pincel).
5. Deu errado se: dois pincéis pintam igual, algum pinta em branco, ou um controle do painel não muda nada.

## §7 — ABERTO (próxima linha)

- **Fila de motor** com ordem e medição em [01_CATALOGO §3d](../Pinceis/01_CATALOGO.md): borda da mancha molhada
  esbatida · pontas cortadas a direito num dab redondo · traço de 200 px do filme · mistura em luz linear (decisão do
  dono, muda todos) · orla do secador · misturador com grão perto da tinta · fio do arrasto · lâmina que gira (C38) ·
  partículas/mip. Destrava os 67 «em estudo».
- **Brecha do gate:** `set_brush_shape_image_rgba` (já no main) recebe `&[u8]` numa linha à parte da assinatura e
  escapa a `arch_color_space_typed` (o gate não lê assinaturas em várias linhas).
- **Custo no app** ([custo_no_app](../Pinceis/exames/custo_no_app.md)): depósito por TEMPO (Filbert 56 → 24 ms a 90 px/qd); falta o pouso do transfer (série).
- C39–C40 e o resto do [P53 §6](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-19_P53.md) continuam.

## §8 — Depois do smoke: o navegador de presets (22/10, `9a8754b976` `86cb0f15ea`)

- O popover do Preset tinha um título apagado por PASTA (lia-se «pincel apagado»). Agora **um título por categoria**
  (`painter_bridge_categorias::CATEGORIAS`): Basic · Pencil · Ink · Marker · Pastel · Oil ·
  Watercolor · Airbrush · Texture · Blend & Erase · Effects · Other (pasta desconhecida). Dentro da categoria o meio de
  agora primeiro.
- Widget partilhado: `DropdownOption::header` (`false` por omissão; consumidores ao bit) — rótulo de secção, sem
  hit/foco, `Role::Heading`. O **Paint Mode** subiu para o cartão do topo, acima do Preset, nos 4 meios.
- Foto: `foto_do_navegador` (`#[ignore]`); tutorial p. 2 novo.
