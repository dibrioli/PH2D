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
- Novo na J5: **Pastel Side** aprovado (`c6c99afab7`, portão `o_pastel_side_e_o_lado_do_bastao_com_po`, 2 mutações
  sangram). Curados: Square Hard, Water Brush, Dry/Lift, Blender Grainy, Wet Liner, Wet Wash, Gouache Wash.
  Reprovados: Mop, Gouache Round. Decisões: Gouache Flat fica; Wet Dry Brush não se funde.
- **Motor J5:** `Knob::WetOpen`/`WetOpenPush` (`ph2d-wet-paint` `trail/molhado.rs`; molhado × molhado; desligado por
  omissão; portão `o_azul_abre_uma_mancha_redonda_no_vermelho_molhado`, 3 mutações sangram), ligado no Wet Liner e Wet
  Wash; tecto da cauda 1,3 × meia-largura (`eb04c3440d`). Motor J1–J4: [HANDOFF_JANELA_4](../Pinceis/HANDOFF_JANELA_4.md)
  («Motor novo») e `HANDOFF_JANELA_2/3`.
- Biblioteca: nenhum par fora de família < 0,596; custo calmo p95 ≤ 12,5 ms a 200 px, RSS ≤ 105 MiB, load 1–3,4.
  Série de lápis no navegador por grau (2H · HB · 2B · 8B, `002c75267b`). Tutorial do dono:
  [`tutorial/01_pinceis.pdf`](../Pinceis/tutorial/01_pinceis.pdf) (12 páginas).

## §3 — Fecho (1×, `PH2D_SUITE=fecho`)

- fmt ✓ · clippy workspace `--all-targets --features ph2d-spike/bevy_ecs -D warnings` ✓ · censos da árvore combinada
  12/12 e 121/121 ✓.
- nextest-impacted: **19 576 corridos, 19 574 ✓, 0 falharam, 2 TIMEOUT de carga**
  (`a_lei_da_tinta::o_glaze_do_pincel_escurece_o_cruzamento`, `::sem_glaze_cada_preset_de_fabrica_pinta_a_cena_de_antes`;
  sozinhos 8–17 s, 3/3) — promovidos à família de [flakes de carga](../../DevOps/FLAKES_DE_CARGA.md) em `b74b608d0e`
  (tabela + faixa isolada + faixa pesada; `architecture_toda_flake_de_carga_corre_na_faixa_isolada` verde).
- Antes disso o fecho achou 17 falhas reais, curadas: `d9d763b06d` (portas RGBA → `&[SrgbRgba]`; `bytemuck` dev-dep →
  dep em `ph2d-tool-painter`) · `b515ce2549` (8 ficheiros > 700 LOC partidos, só movimento) · `b12e750c11` (citações
  13 → 0) · `ddb6a9608e` (defeito real: `deposit_mask` com o peso da lei b) · `f6e2242bac` (3 testes na verdade nova) ·
  `27179b7792` (rótulos curtos, `card_row`, `safe_clamp`) · `9851c5e93b` (controlos mortos: Speed lag sem lei da
  velocidade — regra `AtrasoSemVelocidade`; Grain da água não esmaece no Blend) · `48be9e05c0` (sub-secções recolhidas)
  · `002c75267b` (ordem do navegador; mutação `sort()` vermelha, restaurada). Estilo: `1543fb9f5b` `0f3c03a695`
  `348fb0644c` `cde29e86ed`.

## §4 — Superfície de colisão (`bash <worktree>/scripts/collision-surface.sh`, base `100d34c8e0`)

- Schemas **todos iguais à base** (`PROJECT_SCHEMA` 191, tripla `(191, 13, 23)`, vec 23, flip 13, timeline 20);
  registos de componentes 108/109/109 iguais; contratos §6 intocados. A linha não mexe em schema: **nada a recontar**,
  mas confira no main do dia (`python3 <worktree>/scripts/schema-recount.py <valor_do_main>`).
- ADR: a linha criou **0189** (caneta, P46) — reconte contra o main do dia. `Cargo.lock`: só `ph2d-pen` (crate interna
  da P46; deps externas `wayland-*`, `rustix`, `zstd`, `cursor-icon`, `raw-window-handle`).
- «Marcadores de conflito» ✗ = **falso positivo**: 2 PNG binários em `fixtures/rebelle_bateria/OIL_AND_ACRYLIC/`.
  «Tecto de LOC» ✗ `app_state.rs` 923 e `main.rs` 1060 (shell): a linha só tocou 3 linhas em ambos; são de antes.
  `painter_bridge.rs` 666/700.
- Toca fora do Painter: `.config/nextest.toml` (2 filtros: faixa isolada e pesada) · `docs/DevOps/FLAKES_DE_CARGA.md`
  (+2 linhas de tabela) · `ph2d-i18n` (`app_painter`, `lib`, `painter_layers`, `painter_marca`, `painter_ponta`,
  teste `a_fronteira_dos_motores`) · `ph2d-panel-painter-layers` (42 ficheiros) · `ph2d-panel-registry-init`
  (`quantas_entradas_tem_cada_painel.rs`) · `ph2d-pen`/`ph2d-host`/`ph2d-editor-core` (P46) · `shells/desktop` (26).

## §5 — Números a recontar no merge (contar, nunca escolher)

- `PRESETS_DE_FABRICA` agora é **calculado** (`receitas_de_fabrica.rs`: receitas + famílias + ferramentas Wet) = **45**;
  gate `the_factory_presets_are_counted`. Outra linha que some presets: reconte pelos ficheiros de `assets/brushes/`.
  PNG binários de preset em conflito: REGENERAR ([P51 §3](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-19_P51.md)).
- Gates de painel regravados: altura de `painter_layers` **1554** (era 2091; `quantas_entradas_tem_cada_painel.rs:2144`)
  · tecto de nomes que não cabem na coluna 220 px: **15 → 13** (`27179b7792`, gate
  `cada_nome_deste_painel_cabe_na_coluna_da_seccao`) · painel do Painter volta ao tecto de 600 LOC (`265d8ebeb2`).
  Outra linha que some linhas ao cartão do Pen/Brush/Stroke muda 1554: reconte.
- LOC: 8 ficheiros partidos em `b515ce2549` (`wet-paint` `tools/bordas.rs`, `trail/{bico,medida}.rs`, `solver/gota.rs`,
  `solver/flow_jacobi/backrun.rs`; `tool-painter` `wetpaint/rota_da_lane.rs`; `painter-brush` `blend.rs`): quem
  conflitar em `dab_route.rs`/`tools.rs`/`trail.rs`/`solver.rs` reaplica o MOVIMENTO, não funde à mão.
- Flakes de carga: +2 membros (acima); a lista vive em `FLAKES_DE_CARGA.md` + 2 filtros do `nextest.toml` — duas escritas.

## §6 — Smoke do Enio (binário `smoke` compilado; 1.ª compilação 59,78 s; 2.ª `Finished smoke profile [optimized] target(s) in 0.43s`)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && PH2D_WETPAINT_SMOKE=2 ./target/smoke/ph2d-host-desktop`
2. Abra o tutorial `docs/Painter/Pinceis/tutorial/01_pinceis.pdf` e siga uma página por categoria: no Painter, escolha o
   pincel da lista (45) e risque um traço; compare com a foto ao lado.
3. Tem de acontecer: cada pincel pinta com a cara do material e diferente dos vizinhos; os lápis vêm do mais duro ao mais macio.
4. Responda as duas perguntas do fim do PDF: **decisão 5** (a cauda do traço afina só no Digital) e **decisão 8**
   (tamanho máximo por pincel).
5. Deu errado se: dois pincéis pintam igual, algum pinta em branco, um controle do painel não muda nada, ou o
   cartão não cabe na coluna estreita.

## §7 — ABERTO (próxima linha)

- **Fila de motor** com ordem e medição em [01_CATALOGO §3d](../Pinceis/01_CATALOGO.md): borda da mancha molhada
  esbatida · pontas cortadas a direito num dab redondo · traço de 200 px do filme · mistura em luz linear (decisão do
  dono, muda todos) · orla do secador · misturador com grão perto da tinta · fio do arrasto · lâmina que gira (C38) ·
  partículas/mip. Destrava os 67 «em estudo».
- **Brecha do gate:** `set_brush_shape_image_rgba` (já no main) ainda recebe `&[u8]` e escapa a `arch_color_space_typed`,
  porque o `&[u8]` está numa linha à parte da assinatura e o gate não lê assinaturas em várias linhas.
- **Ink Brush a vigiar:** p95 12,54 ms a 200 px, 11/475 quadros > 16,7 ms.
- C39–C40 e o resto do [P53 §6](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-19_P53.md) continuam.
