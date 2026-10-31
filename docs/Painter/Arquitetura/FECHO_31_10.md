# Fecho técnico 31/10 — Wet Paint (E4, Bingham, piso do espalhar, E2/E3)

Agente FECHO-TÉCNICO, ramo `line/PainterWatercolor` (HEAD `c5e6daa0a9`), sem commit. Testes por nome, `--profile smoke`
no `ph2d-tool-painter`, `dev` no `ph2d-wet-paint`.

## 1. `ph2d-wet-gpu/src/passo.rs` partido (move puro, visibilidade `pub(super)`)

| ficheiro | LOC | o que é |
|---|---|---|
| `passo.rs` | 636 | `placa_sabe`, `PassoPlaca` (era 832) |
| `passo/fatias.rs` | 89 | `Despacho`, `Fatias`, `MAX_CARIMBOS` |
| `passo/hashes.rs` | 89 | `mistura`, `hashes`, `hash_fina`, `hash_fluxo`, `fluxo_de`, `finas_de` |
| `passo/wgsl.rs` | 35 | `ENTRELACA`, `PORTA` |

`cargo-check-narrow ph2d-wet-gpu` verde. ⚠️ Por correr (prazo): `architecture_workspace_file_loc_cap`,
`architecture_no_orphan_source_file`, e os testes de placa do `ph2d-wet-gpu`.

## 2. Teste novo `trail::espalha_tests::o_oleo_grosso_sobre_o_oleo_grosso_soma_massa_e_nao_cava`

`Engine` 96², Real Oil (`sim.oleo`), Water 0, `Intensity` 10 (espessura > 3000, onde o espalhar morde), Drag/Smear 0
(o arrasto é outra lei e cava sozinho, com ou sem piso — medido: 5–11 células com Drag por omissão nos dois ramos),
dois traços paralelos meio sobrepostos (y 48 / 52), pausa 120. Asserções: 0 células do contacto abaixo de
`h₁ − 2 % max`; Σh depois ≥ 0,97 × o prometido pela lei (`max(t₀, t₀·(1−ESPALHA)+v)`).

| variante | crateras no contacto | resultado |
|---|---|---|
| `trail::sonda::SEM_PISO = true` (a lei de antes) | **32 / 340** | VERMELHO |
| produto | **0 / 340** | VERDE |

Achado: o balanço `Σh depois − Σh antes − Σh traço 2` dá −0,18 nesta carga — é a saturação do ESPALHA por desenho
(`t → Σv ÷ ESPALHA`), não perda; por isso a régua do volume compara com o prometido pela lei, não com a soma.

## 3. Portões ao bit e miniaturas

| teste | classe | ação |
|---|---|---|
| `ph2d-wet-paint` `flow::flow_bit_tests::flow_step_bits_are_the_recorded_ones` | (a) E4: a receita recalcula pigmento/água de cada dab por φ·h (`doors.rs`/`deposit.rs`), igual à pressão de referência a < 1e-6 por dab (`receita_tests`); a cena não tem óleo (Bingham e piso fora). Rotas 1 e 2 dão o mesmo par. Diferença em 8-bit não medida (hash de estado f32) | **regravado** `GRAVADO` com o porquê |
| `ph2d-tool-painter` `presets_de_fabrica::portoes::a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` — Fan Brush (Acrylic) | (a) 1 px | PNG: não regravado (ordem) |
| idem — Blender Dots/Hard/Soft/Textured | **(b)** 24 337–33 382 px por miniatura: mudança real. Suspeita: a receita E4 dos pincéis de mistura (pigmento ~0 ⇒ φ/h diferentes da lei velha) ou a migração `migra_da_lei_velha`; não bissecado | por decidir (dono/linha) |
| idem — Oil Dabber, Dabber 4 | **(b)** 11 841 / 8 649 px: mudança real; candidatos: Bingham (`ladeira_do_oleo::bingham`) ou o piso do espalhar (`trail/espalha.rs`), se o preset tem Real Oil; não bissecado | por decidir |
| `a_agua_nao_sobe_a_ladeira_do_oleo` | (c) vermelho anterior (HANDOFF_AUDITORIA_OLEO_SOBRE_OLEO §4) | nenhuma |
| `mudar_o_painel_do_pincel_nao_muda_a_tinta_pousada` | (c) | nenhuma |
| `o_traco_seguinte_usa_o_depth_o_gloss_e_a_caixa_novos` | (c) | nenhuma |
| `a_aguada_re_molha_o_oleo_soluvel` | (c) | nenhuma |
| `sim_resumable`, `product_tool_doors`, `assenta_bit_tests`, restantes `flow_bit_tests` | verde | — |

## 4. Aquarela / guache

Filtros `aquarela watercolor guache gouache backrun edge_dark`: `ph2d-wet-paint` lib 7 + `tests/it` 8 verdes;
`ph2d-tool-painter` 227 verdes na lib (os 4 vermelhos são os da classe (c)), `tests/it` 8 verdes + a miniatura acima.
Nenhum vermelho de aquarela/guache.

## Por fazer

- Bissecar os Blender e os Dabber (classe b) pelos interruptores em execução (`solver::PAPEL_BINGHAM`,
  `trail::sonda::SEM_PISO`, `espalha::sobrepoe(Some(0.0))`) sobre `miniatura_do_preset`, e só então regerar a fábrica.
- Portões de LOC/órfão sobre o corte do `passo.rs`.
