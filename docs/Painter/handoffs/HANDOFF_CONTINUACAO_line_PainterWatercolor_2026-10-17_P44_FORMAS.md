# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, prompt 44: as formas (a ponta reta do Flat, a guache, a pressão) (2026-10-17)

## §1 — Coordenadas

Sobre o `main` `ad5f8fe39c` (merge-base, sem rebase). Sem push. [Prompt 44](../NewFeaturesWetPaint/44_PROMPT_as_formas_flat_e_guache.md)
· medição e leis: [44_medicao](../NewFeaturesWetPaint/44_medicao_as_formas.md) · [prancha 44](../NewFeaturesWetPaint/44_prancha_as_formas.png)
· doc [18](../NewFeaturesWetPaint/18_mapa_das_midias.md) §5 bloco C ✅. Anterior: [P43](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-17_P43_PO_FINO.md).
Commits `88559d8cd4` (identidade das 18, ANTES da lei) · `1fbaf9d1d1` (a quina + a pressão + a régua) · `5fe95853dd`
(fábrica 20, o traço mínimo, portões) · `d99c8e3f78` (a janela do rasto) · o deste fecho.

## §2 — Medido primeiro (doc 44 §1–§2)

- **O oráculo das fotos que havia** bastou (não se correu o Rebelle): o Flat acaba RETO deitado (recuo 0,03–0,12) e
  redondo de pé, é mais alto que largo (30 × 34; a guache 31 × 39) e não roda com o traço (onda a 30°: 1,22×).
- **A métrica nova `ponta`** (o recuo da ponta a ±0,7 da meia-largura, as 4 pontas isoladas): Rebelle Flat 0,096 ·
  Gouache Flat 0,083 · Round 0,258 · Liner 0,316; o nosso Wet Round 0,43. ⚠️ O «comprimento além do último
  carimbo» do prompt NÃO se mede: o Rebelle não pousa a ponta onde o gesto acaba (o Round: 9 px × 1 px).
- **A Gouache Round é o nosso Round** (18 §4.3, confirmado); a guache que é outra mídia é a FLAT (cob 0,50 × 0,32).
- **A composição não dava a ponta reta**: a silhueta procedural é sempre `falloff(|f|₂)`; a ponta de imagem não
  viaja no preset. A pressão → opacidade existia no `Dynamics`, mas nada a ligava (`PaintState::dynamics` fixo).

## §3 — As leis

- **A QUINA** (`BrushSpec::dab_quina`): `t = (1−q)|f|₂ + q|f|∞` em `FootprintDeform` (q 0 = o `sqrt` de sempre ao
  bit; sem transcendentes). Contorno exacto, caixa = a menor das duas que contêm. Preset por nome
  (`CAMPOS_DO_PINCEL` 31). **`SCHEMA` fica 1.** Painel: Shape › «Square tip», só no Wet Paint; o gizmo desenha o
  contorno da pegada (`outline_at`). Varrido por consumidor: caches (chave), device (inelegível), blur/clone/inércia
  (`pegada_anisotropica`), anel, alcance e janela do motor molhado.
- ⚠️ **A ponta só se LÊ com borda dura** (≥ 0,85): sobre a queda macia do Wet Round a quina 1 mede 0,44 (= redondo).
- **A pressão do PINCEL**: knobs `pressureSize` (0,9 = os 10 % de sempre, ao bit) e `pressureOpacity` (0 = parada),
  Tuning › Paint; lidos em `PainterTool::dinamica_do_traco` no `Stroke::new`. `KNOB_COUNT` 58.
- **O traço MÍNIMO** (`LeiDaLargura`): quando o raio proporcional cai num raio MEDIDO sem tinta, pinta-se o menor
  traço que se vê (a Gouache Flat: 1,8 → 0 px, 2,1 → 5 px; pedido 4 px pintava 0). Os outros pincéis iguais.
- **Fábrica 18 → 20**: `Watercolor/Fluid/Wet Flat` (pigmento 50) e `Watercolor/Gouache/Gouache Flat` (100), do Wet
  Round com dura 0,9, quina 1, achatado 0,12/0,2 @ 90°, pressão 0,3/1. Contra o Rebelle: cob 0,318/0,315 ·
  0,525/0,504; ponta 0,22/0,10 · 0,18/0,08; largura 34/34 · 39/39; rampa larg 0,76/0,83 · 0,82/0,87. Rótulo
  «Gouache» em `rotulo_da_pasta` + i18n.

## §4 — Portões

Doc 44 §5. Identidade das 18 ao byte; `footprint.rs` (3 portões da quina); `it/as_formas.rs` (o quadrado a 0° e
45°); `it/midias_de_fabrica.rs` 16 mídias (Ponta, RampaLargura, Chato); `regua_midia` Rust = Python;
`stamp_device_tests`; `largura.rs`; contagens 20 · `rows` 32.

## §5 — Mutação

**12/12 sangram** (1.ª rodada 10/12, agente `mutacao`: M6 «o alcance não segue a pegada DESTE dab» e M8 «a pressão
não liga a opacidade» sobreviveram e ganharam portão — `as_formas::a_ponta_reta_que_segue_o_traco_nao_perde_os_cantos`
(0,88× a área com a mutação) e `as_formas::a_pressao_do_pincel_esmaece_a_rampa` (0,862 = 0,862)). Lista: quina no
falloff/caixa/contorno, a quina na pegada do pincel, o `fp_raio`, o device, a pressão (tamanho, opacidade), o JSON do
preset, o traço mínimo, a régua da ponta.

## §6 — Aberto

- **A cobertura da rampa** (lista 26 C27): 0,54/0,62 contra 0,42/0,43 do Rebelle — a rampa linear do `Dynamics`.
- **Smoke do dono (17/10)**: OK, salvo o «Square tip» travado em 1 — o `is_param_field` não reclamava o id (a row
  revertia ao soltar); curado em `bfc868d90f` com gate de costura vermelho sem a cura.
- **O Airbrush** (C28, ordem do dono «como no Rebelle»): tamanho 0,5 + opacidade 1 (`36963d8e1a`, identidade
  re-pinada) → rampa 0,56/0,62 contra 0,63/0,34; os números literais do Spray 2 dão 0,92/0,80 — a opacidade da
  pressão quase não morde no pincel seco de pouca tinta. Mecanismo por achar (C27 + C28) → prompt 45.
- **Custo**: 1,12× por traço com a ponta reta — a área do quadrado (4/π), não a conta.
- **B13 = SIM** (a Shape Color no pincel salvo) → prompt 45. **B14 = a mistura fica da obra inteira** (fechado).

## §7 — Fecho

`cargo fmt --all --check` ✓ · clippy `-D warnings --all-targets` (`PH2D_SUITE=fecho`; painter-brush, wet-paint,
tool-painter, panel-wet-tuning, panel-painter-layers, i18n, app-painter com `panel-painter-layers`) ✓ ·
`nextest-impacted` (base `ad5f8fe39c`) **18 882/18 887** em 1 189 s (`load` 45 → 169): 1 defeito desta linha —
`footprint.rs` 759 LOC > 700 (`workspace_src_files_under_loc_cap`), curado por CORTE (os portões para
`footprint_tests.rs`, 490 + 275; refeito por nome ✓) — e 4 de carga, verdes sozinhos por nome: `g4_o_interior_de_um_
opaco_nao_custa` (light2d), `the_cost_of_a_player_is_linear_in_their_number` (physics-ecs) e os timeouts de 360 s de
`cada_preset_parece_a_sua_midia` (83 s sozinho, `load` 68) e `a_largura_e_a_prometida` (131 s, `load` 129) — estes
dois são desta linha e pesados: 16 mídias e 20 pincéis secos; candidatos à faixa lenta do nextest.
