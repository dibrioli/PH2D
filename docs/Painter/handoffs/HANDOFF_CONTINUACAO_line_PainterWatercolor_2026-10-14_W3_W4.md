# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, plano 27 W3 (a lei B, fecha o #47) e W4 (o painel por mídia, o Solid de carimbos na água) (2026-10-14, prompt 30)

## §1 — Coordenadas

Linha reaberta sobre o `main` `0aab2c1cb6` (14/10). Commits desta janela: o prompt 30 (`941137553a`) + o desta onda
(W3+W4). Sem push. Regras de 13–14/10: teste POR NOME, suíte só a cada 10 commits ou no fecho, clippy só no fecho,
edição provada (R5–R7).

## §2 — W3: a lei B ([27 §6.8](../NewFeaturesWetPaint/27_plano_shape_casa_dos_pinceis.md))

- **Medido antes, no produto** (`wetpaint/sonda_shape_casa.rs::sonda_shape_casa_tinta_por_preset`, 12 presets × 7 tipos):
  sobre o GRÃO dos pastéis o Shape procedural pousava 0,000–0,006 do redondo — o portão cortava o padrão.
- **Motor:** `brush.rs::stamp_row_shaped` com o 3.º canal `padrao`; `trail/deposit.rs::DepositLaw::cell(…, sv)` multiplica o
  depósito DEPOIS do portão. Fios: `deposit_shaped`/`shaped_impl`/`accumulate_paint_shaped(_rows)`/`dispatch_pressure_dab_lane`
  ganharam `padrao`; o caminho do motor passa `1.0`; borracha (`for_each_stamp_pixel_shaped`) e máscara passam `None`.
- **Produto:** `dab_route.rs` — Shape procedural (não Image, não borracha) ⇒ silhueta = envolvente, padrão no canal pela porta
  nova `ph2d_painter_brush::dab::shape_value_at` (a `silhouette_at` usa-a; um só sítio da lei do valor do Shape).
- **Depois:** «como vem» e «None» dos 12 presets iguais ao byte de antes; pastéis Noise 0,08–0,12, Clouds 0,22–0,31; ponta
  lisa Noise 0,337 → 0,605. **#47 FECHADO** (linha no índice `BUGS_painter_fechados_41_a_46.md`).

## §3 — W4 ([27 §6.9](../NewFeaturesWetPaint/27_plano_shape_casa_dos_pinceis.md))

- **C1:** `Dependente::CorDaTexturaSemLeitor` (`inercia.rs`, lei `agua || aquarela`), dica `inerte.texture_colors_media`
  (`ph2d-i18n`). Censo: gesto `Gesto::PontaColorida` (arma a ponta de imagem colorida direto na ferramenta, como o «Use as
  Brush Shape» da Hierarchy) + a pré-condição nos três meios.
- **C2:** `mascara.rs::enche_a_mancha_na_agua` — ponta de IMAGEM ⇒ `cobertura_dos_carimbos` (União das silhuetas dos
  `solid_fill_stamps`) pela MÁSCARA. ⛔ Recusado e medido: os carimbos dab a dab (39/400 do miolo).
- **A lisa × as cerdas:** para o smoke do dono (26 B6).

## §4 — Portões

- Ao byte: `presets_de_fabrica` (as 12 miniaturas) · `ph2d-wet-paint` `deposit_rows fingerprint product_doors
  product_tool_doors paper_rows parallel_rows the_accumulate` (33 ✓).
- Novos: `wetpaint/lei_b_tests.rs` (2) · `mascara_tests::com_ponta_de_imagem_a_mancha_da_agua_e_de_carimbos` · a
  pré-condição «ponta de imagem colorida» no `censo_dos_controlos::a_linha_esmaecida_e_a_que_nao_age` (painel 10 ✓).
- **Mutações** (agente `mutacao`): ver §6.
- Sondas: `sonda_shape_casa_tinta_por_preset` (W3), `diag_c2_mancha_de_carimbos` (`PH2D_SONDA_OUT` grava PPM).

## §5 — ABERTO

- **B6** (dono): a ponta lisa (4,7–6,2× as cerdas) e o padrão sobre o grão (4–5× mais fraco que sobre a lisa).
- **C16:** as linhas «Layer N Color» não esmaecem na água (ids de fábrica fora do censo).
- **W5:** `BrushShape::{Flat, Fan}` do motor saem (mortos, §6.4); doc do Shape.
- C13–C15 (26) sem mudança; os tectos de LOC de `stamp_cache.rs`/`stroke.rs` não foram tocados.

## §6 — Fecho

- **Mutações (3/3 sangram, controlo verde antes):** M1 `padrao_a_parte = false` ⇒ os dois `lei_b_tests` (pastel Noise
  0,000; lisa 0,337) · M2 sem o ramo dos carimbos ⇒ o miolo liso (desvio 2,19 × 2,19) · M3 a lei do C1 em `false` ⇒ o
  censo acusa o Wet Paint (inerte e viva); em `true` ⇒ acusa Digital e Impasto (esmaecidos e agem).
- `cargo fmt` (5 crates, 8 ficheiros reformatados) ✓ · clippy `-D warnings --all-targets` nas 5 crates ✓ ·
  `nextest-impacted` (base = merge-base, árvore suja) **18 445/18 445** ✓ em 478 s, com os tectos de LOC, a
  `the_shell_only_shrinks` e o `no_orphan_source_file`.

## §7 — Smoke ao dono

Passos numerados na resposta da janela (Wet Paint: Shape Noise sobre o Soft Pastel; Use Texture Colors esmaecido; Solid
com ponta de imagem; a ponta lisa).
