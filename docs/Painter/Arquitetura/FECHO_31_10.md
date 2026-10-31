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
| idem — Blender Dots/Hard/Soft/Textured | **(b) → REGRESSÃO da E4, curada** (§5): 28 886 / 24 337 / 25 129 / 33 382 px | curado: 0 px |
| idem — Oil Dabber, Dabber 4 | **(b) → Bingham** (§5): 11 841 / 8 649 px, cura esperada | regerar (coordenador) |
| `a_agua_nao_sobe_a_ladeira_do_oleo` | (c) vermelho anterior (HANDOFF_AUDITORIA_OLEO_SOBRE_OLEO §4) | nenhuma |
| `mudar_o_painel_do_pincel_nao_muda_a_tinta_pousada` | (c) | nenhuma |
| `o_traco_seguinte_usa_o_depth_o_gloss_e_a_caixa_novos` | (c) | nenhuma |
| `a_aguada_re_molha_o_oleo_soluvel` | (c) | nenhuma |
| `sim_resumable`, `product_tool_doors`, `assenta_bit_tests`, restantes `flow_bit_tests` | verde | — |

## 4. Aquarela / guache

Filtros `aquarela watercolor guache gouache backrun edge_dark`: `ph2d-wet-paint` lib 7 + `tests/it` 8 verdes;
`ph2d-tool-painter` 227 verdes na lib (os 4 vermelhos são os da classe (c)), `tests/it` 8 verdes + a miniatura acima.
Nenhum vermelho de aquarela/guache.

## 5. Bissecção das miniaturas classe (b) (agente MINIATURAS)

Teste temporário sobre `miniatura_do_preset`, interruptores em execução no mesmo processo: `PAPEL_BINGHAM = −1`
(sem τ_y), `trail::sonda::SEM_PISO`, e para a E4 um interruptor provisório com a lei velha exacta de `pressure_dab`
(força × `Intensity`, água pela pressão invertida, `PPD × Opacity`) e sem a migração. Controlo: Fan Brush 1 px → 0 e
Wet Round 0 → 0 com a lei velha (a emulação é exacta). Px diferentes da miniatura de fábrica:

| pincel | produto | sem Bingham | sem piso | lei velha E4 | causa |
|---|---|---|---|---|---|
| Blender Dots | 28 886 | 28 886 | 28 886 | **0** | E4 |
| Blender Hard | 24 337 | 24 337 | 24 337 | **0** | E4 |
| Blender Soft | 25 129 | 25 129 | 25 129 | **0** | E4 |
| Blender Textured | 33 382 | 33 382 | 33 382 | **0** | E4 |
| Oil Dabber | 11 841 | **0** | 11 841 | 11 841 | Bingham |
| Oil Dabber 4 | 8 649 | **0** | 8 649 | 8 649 | Bingham |

**Blender — regressão da migração, curada.** A miniatura de uma ferramenta da água pinta primeiro a FAIXA vermelha
(`preview.rs::faixa_da_ferramenta`) com uma receita da lei velha escrita à mão: `WetKnobs::DEFAULT` + PPD 500 +
Water 0,05, sobre a carga `Intensity 0,4505` do Wet Round (calibrada para PPD 78 · Water 1). Medido por dab: água
`0,00087 → 0,00075` e pigmento `500 → φ·h ≈ 427` — a faixa saía mais clara, e o Blender mistura-a. Os presets
Blender em si migram certo (PPD 78, Water 1, Opacity 1 ⇒ carga 0,4505, água igual à velha). Cura:
`preview::knobs_da_faixa(opacidade)` migra a faixa por `migra_da_lei_velha` (como a Acrylic Round de fábrica).
Teste novo `receita_e4_tests::a_faixa_das_ferramentas_da_agua_pousa_o_mesmo_que_na_lei_velha` (Opacity 1 / 0,6 /
0,25, tolerância 1e-6; sem a cura pousaria 427 de 500). Depois: as quatro Blender **0 px**, Fan Brush 1 px (ruído
f64 da receita, 0 com a lei velha).

**Dabbers — Bingham, cura esperada.** Nem o piso nem a E4 os mexem; τ_y sozinho dá a diferença inteira. A crista e
o S são os mesmos; muda a orla texturada (a pasta deixa de escorrer para as franjas) — máx. 168/195 por canal em
píxeis isolados da orla. Fotos (fábrica · produto · |dif|×4):
[`oil_dabber`](fotos/fecho_31_10/oil_dabber_fabrica_x_bingham_x_dif4.png) ·
[`oil_dabber_4`](fotos/fecho_31_10/oil_dabber_4_fabrica_x_bingham_x_dif4.png). Ação: regerar estas duas (e o Fan
Brush) — só o coordenador.

## 6. `ph2d-wet-paint/src/render.rs` partido (move puro)

| ficheiro | LOC | o que é |
|---|---|---|
| `render.rs` | 696 | o compositor, as linhas de pigmento (era 754, tecto 700) |
| `render/pre_luz.rs` | 64 | `PreLuz` (`fecha_oleo`, `fecha_sem_oleo`), reexportado `pub use pre_luz::PreLuz` |

`architecture_workspace_file_loc_cap` e `architecture_no_orphan_source_file` verdes (`ph2d-editor-core`).

## 7. Miniaturas do óleo pelo caminho EXATO do portão (agente MINIATURAS-ÓLEO)

A §5 bissectou por outro caminho e só viu 6 pincéis. Sonda `#[ignore]`
`presets_de_fabrica::sonda_miniatura_oleo::sonda_miniaturas_bingham_x_piso` (`tests/it/presets_de_fabrica/sonda_miniatura_oleo.rs`):
percorre os MESMOS `ficheiros_de_fabrica()` com o MESMO `miniatura_do_preset` do portão, e em cada um que difere pinta
4 variantes no mesmo processo (`PAPEL_BINGHAM = −1` = sem τ_y; `trail::sonda::SEM_PISO`). HEAD `b7f54356d6` (Dabber,
Dabber 4, Fan Brush já regerados). Correr:
`PH2D_SONDA_FOTOS=<pasta> bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --test it sonda_miniaturas_bingham_x_piso -- --ignored --nocapture` (30 s).

**O portão reprova 11 pincéis, não 9** (a saída cortava Knife Textured e Metallic Flat). Px diferentes da fábrica:

| pincel | produto | sem Bingham | sem piso | sem os dois | máx Δ canal | causa | julgamento |
|---|---|---|---|---|---|---|---|
| Oil/Flat/Filbert | 18 013 | **0** | 18 013 | 0 | 69 | Bingham | cura: o sal das franjas some, ficam as estrias das cerdas — **regerar** |
| Oil/Flat/Flat Bristle | 31 046 | **0** | 31 046 | 0 | 63 | Bingham | idem — **regerar** |
| Oil/Flat/Tilted Shade | 25 312 | **0** | 25 312 | 0 | 54 | Bingham | idem — **regerar** |
| Oil/Flat/Thick Impasto | 19 919 | **0** | 19 919 | 0 | 185 | Bingham | ⚠️ o relevo do miolo fica MAIS picado: mais vincos e covas escuras na metade de baixo (o commit `191b3ff` já dizia: Bingham não cura as crateras do Thick Impasto W 0,5) — **NÃO regerar** |
| Oil/Knife/Knife Dry | 16 700 | **0** | 16 700 | 0 | 132 | Bingham | só a orla pontilhada; corpo igual — **regerar** |
| Oil/Knife/Knife Grainy | 13 552 | **0** | 13 552 | 0 | 155 | Bingham | grão mantido, diferença espalhada e fraca — **regerar** |
| Oil/Knife/Knife Oily | 20 184 | **0** | 20 184 | 0 | 161 | Bingham | a crosta pontilhada some, ficam os vincos lisos e o brilho da aresta (cura); ⚠️ as duas pontas ficam moles/esbatidas em vez de cortadas — **regerar, com a ressalva ao dono** |
| Oil/Knife/Knife Smooth 3 | 13 911 | **0** | 13 911 | 0 | 153 | Bingham | quase invisível — **regerar** |
| Oil/Knife/Knife Stain | 4 345 | **0** | 4 345 | 0 | 222 | Bingham | o padrão de furos é o mesmo; Δ em píxeis isolados — **regerar** |
| Oil/Knife/Knife Textured | 6 588 | **0** | 6 588 | 0 | 88 | Bingham | ruído fino — **regerar** |
| Oil/Metallic/Metallic Flat | 16 945 | **0** | 16 945 | 0 | 146 | Bingham | ⚠️ perde TODO o grão (o «brilho» de flocos da orla) e vira traço de aerógrafo liso com pontas esbatidas — o carácter metálico da miniatura some — **NÃO regerar sem o dono** |

**Causa única: Bingham.** O piso do espalhar não move nenhum (sem piso = produto, ao px); a E4 não entra (sem
Bingham = 0 px, logo a receita nova pinta estes pincéis igual à fábrica). Nenhum pincel fica fora de 0 com alguma
variante. Nenhum mostra furo novo, perda de cor do corpo nem borda partida; o que muda é a orla (o pontilhado da tinta
que escorria para as franjas deixa de existir) — e em dois o desaparecimento desse pontilhado leva o carácter junto.

Fotos (fábrica · produto · |dif|×4), em [`fotos/fecho_31_10/oleo/`](fotos/fecho_31_10/oleo/):
`filbert`, `flat_bristle`, `thick_impasto`, `tilted_shade`, `knife_dry`, `knife_grainy`, `knife_oily`,
`knife_smooth_3`, `knife_stain`, `knife_textured`, `metallic_flat` (`<nome>_fabrica_x_bingham_x_dif4.png`).

**Para o coordenador:** `PH2D_REGERA_SO="Filbert,Flat Bristle,Tilted Shade,Knife Dry,Knife Grainy,Knife Oily,Knife Smooth 3,Knife Stain,Knife Textured"`.
**Não regerar** Thick Impasto (relevo mais picado — o defeito de crateras é da E7 por fechar) nem Metallic Flat
(perde o carácter) — com eles por regerar o portão fica vermelho em 2, e isso é verdade: decide o dono / a E7.

## Por fazer

- Regerar as miniaturas Oil Dabber, Dabber 4 (Bingham) e Fan Brush (1 px) — coordenador; depois o portão
  `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` fica verde.
