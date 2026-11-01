# F3 — Os controlos do Wet Paint: 177 → 31, cinco áreas, um controlo = uma grandeza (E10, desenho)

> Leitor: a LLM que implementa a E10 ([F4_2 §7](F4_2_plano_wet_paint.md) linha E10) e o coordenador. Documento de
> DESENHO — nenhum código mudou. Fontes: o [censo](F1_censo_wet_paint.md) (§1 os 144 knobs, §2 os de fora, §4 as
> famílias D1..D22, §5 os sem grandeza), a [matriz dos presets](F1_presets_de_fabrica.md) (+
> [`matriz.tsv`](fotos/f1_presets/matriz.tsv)), as grandezas da [F2 §2](F2_grandezas_fisicas.md), a proposta da
> [F4_2 §7.2](F4_2_plano_wet_paint.md), [ADR-0197 §1.1](../../architecture/decisions/0197-o-painter-e-uma-lei-de-pigmento-em-volumes-e-os-meios-sao-pontos-dela.md),
> a receita já feita ([E4](E4_receita_e_quantidade.md), `wp/painter/receita.rs`) e o fecho da [E9 §7](E9_portao_mae.md).
> Abreviaturas como no censo: `wp/` = `crates/ph2d-wet-paint/src/`, `tp/` = `crates/ph2d-tool-painter/src/tool/paint/`,
> `defs:N` = `wp/tuning/defs.rs:N`. Árvore: `line/PainterWatercolor` em `4bcf13769e`.

## §0 Decisões

- **N = 31 controlos no painel** (PAINT 7 · BRUSH 9 · PAPER 4 · TIME 4 · LIGHT 7), mais a **linha da ferramenta**
  (1 «Strength» por ferramenta + «Reach» do Blend) e o cartão **Pen** (a mão, partilhado com o Digital, sem mudança
  de lei — fora do N). Os ~177 de hoje: 144 do registo (`KNOB_COUNT`, `wp/tuning.rs:365`) + 33 de fora (censo §2;
  aqui desdobrados em 36 linhas) + 8 linhas da mão fora do registo (matriz (b), `tp/mao.rs:90-125`) = **188 linhas** na §2.
- **Regra de corte (ADR §1.1):** cada controlo escreve UMA grandeza com unidade, e nenhuma grandeza tem dois
  controlos. Uma grandeza pode NÃO ter controlo (fica derivada ou constante) — não é obrigatório expor cada letra da F2.
- **Pigmento + Água → UM controlo, `Dilution`** (φ). Hoje a receita já é `φ = P/(P + K_W·veículo(W, WaterPerDab))`
  (`wp/painter/receita.rs:82-85`): dois sliders com UM grau de liberdade útil (duplicar P e W não muda nada) é a
  definição de «dois controlos para uma grandeza». O rótulo é neutro ao meio (no óleo não há «Water», há médium).
  ⚑ Pergunta ao dono #1 (§6) — a alternativa conservadora (dois sliders, um deles só leitura) está lá.
- **Volume = só `h`** (ex-Opacity). A E4 deixou `h = Volume · carga · H_REF` (`receita.rs:88`) com `carga =
  Knob::Intensity` (`receita.rs:50`) — ainda dois knobs na mesma grandeza. A E10 funde: `Volume' = Opacity · carga`.
- **Medium (m, 0 água … 1 óleo) ganha controlo próprio** na PAINT. Hoje o meio do dab é `Params::oleo` (o Real Oil,
  `wp/trail/transfer.rs:135` `meio: p.oleo`; `deposito_tinta.rs:52-53` «o Real Oil até a E10») e a receita fixa
  `meio: 0.0` (`receita.rs:51`). A caixa **Real Oil morre**: o que ela ligava é o caso `m → 1` + o material da LIGHT.
- **Cobertura não tem controlo** (D21, F2 eq. 3): `Opaque`, `WetCover`, `DryCover`, `Glaze`, Glaze layering,
  `Transmission*` fundem em **Transparency** (o `S` do PIGMENTO, já lido em `wp/grid/material.rs:33`) + Dilution + Volume.
- **Viscosidade é consequência** de `m` e `φ` (F2: `log η_v = (1−m)log η_água + m log η_óleo`, × KD59 em φ). O que
  fica com controlo é a **tensão de cedência τ_y** (`Body`) — prioridade nº 1 do dono (óleo grosso não escorre;
  [RELEVO](RELEVO_oleo_ondas_e_crateras.md)). ⚑ medir se τ_y(m, φ) já basta (então `Body` sai).
- **Paper press fica** (controlo do PINCEL, 0 segue · 1 amassa — memória do dono 28/10, nunca apagar).
- **Por-pincel × por-tela.** Por-pincel (no preset, só vale para os traços seguintes): PAINT, BRUSH, Re-wet, Edge
  darkening, Wet spread, e o material (Metallic/Roughness/Coat). Por-tela (não viaja no preset): Paper, Relief,
  Absorbency, Tilt, Dry speed, a luz da cena. ⚑ Dry speed por-tela é pergunta #2.
- **Ferramentas são ferramentas:** Blend, Smear, Blow, Dry, Water, Erase, Knife têm UMA força cada na linha da
  ferramenta; o que elas «fazem à tinta» é a mesma física dos controlos (Water = pincel com `Dilution 1`, φ = 0).
- **Forma da ponta = asset `Tip`** (escolhido numa lista com miniatura): os ~30 knobs geométricos/estéticos das
  cerdas (Hidden ou Bristles) vivem DENTRO do asset; o painel expõe só o que é grandeza física (Bristle relief).
- **Numéricos de solver** (`MaxVelocity`, `Projection`, `LevelClamp`, Grid/Flow Grid, Smooth Edges) saem do painel
  do artista: constantes medidas (dev Tuning) ou «Quality» nas definições da tela.
- **UI só em inglês, tooltip em Plain English** (memória `feedback_app_ui_english_only`); chaves i18n
  `panel.wet.<area>.<control>` (HR-15). Nenhum portão se justifica por «casa com o Rebelle» (dono 31/10).
- **Mistura de cor:** padrão = Oklab pelo volume («Smooth»); «Pigment» (WGM, azul+amarelo = verde) opcional por
  pincel, desligado por omissão (resposta do dono 31/10, janela 2). É LEI, não grandeza — um seletor.

## §1 O painel

Colunas: **rótulo** (exacto, inglês) · **tooltip** (uma linha) · **grandeza · unidade** · **faixa** · **omissão** ·
**alvo no código** (o que o controlo escreve) · **guarda** (P = por-pincel, no preset · T = por-tela).
⚑ = a faixa ou o próprio controlo depende de uma medição listada na §4/§6.

### §1.1 PAINT (7) — a receita da tinta no pincel

| # | rótulo | tooltip | grandeza · unidade | faixa | omissão | alvo | guarda |
|---|---|---|---|---|---|---|---|
| 1 | **Medium** | What the paint is mixed with: 0 = water (flows, dries light), 1 = oil (stays, keeps its relief). | `m` · adim. | 0..1 | 0 | **novo** `Knob::Medium` → `receita::Controlos::meio` (`receita.rs:41`, hoje `0.0` em `:51`) e `deposito_tinta::Pigmento::meio` (`transfer.rs:135`, hoje `p.oleo`) | P |
| 2 | **Dilution** | How much water or medium is in the mix: 0 = paint straight from the tube, 1 = almost clear. | `1 − φ` · adim. | 0..1 | 0,33 (= φ 0,67 ⚑ do motor: PPD 600, W 1 → φ = 600/950, `receita.rs:17`) | **novo** `Knob::Dilution` → `receita()` φ (`receita.rs:82-85`), substitui `Controlos::{pigmento, agua, agua_por_dab}` | P |
| 3 | **Volume** | How much paint each touch leaves on the paper — thin film or thick paste. | `h` · µm por dab ⚑ (unidade fixa-se com `M_UM`, E9 §7.3 #2) | 0..4 (×`H_REF`) ⚑ | 1 | `Sliders::pigmento` (`receita.rs:49`) com `carga` (`Knob::Intensity`, `:50`) fundida | P |
| 4 | **Transparency** | How much light passes through the pigment: low hides what is under it, high glazes over it. | escala de `S` do pigmento · adim. | 0..1 | 0 | `Knob::Transparency` (`defs:301` → `wp/grid/material.rs:33` → `soma_da_massa(…, transparencia, …)` `deposito_tinta.rs:83`) | P |
| 5 | **Granulation** | How much the pigment settles into the valleys of the paper as it dries. | densidade do pigmento → `γ_ads` (C97 §4.4) · adim. | 0..1 | 0 | `Knob::ExtGranulation` (`defs:203` → `wp/drying.rs:124`, `gpu/seca.rs:58`) | P |
| 6 | **Body** ⚑ | How stiff the paint is: high stays exactly where the brush leaves it, low levels out. | `τ_y` base · Pa (cresce com φ e m por cima) | 0..`τ_MAX` ⚑ | do meio | **novo** → a ladeira única da E7 (`wp/solver/ladeira_do_oleo.rs`, F4_2 §7 E7) | P |
| 7 | **Color mixing** | Smooth: colors blend like light. Pigment: blue and yellow make green, like real paint. | lei de mistura (seletor, não grandeza) | Smooth \| Pigment | Smooth | substitui `km_mixing` (`tp/wetpaint_settings.rs:661`, `session.rs:258`) — sem o efeito colateral `glaze = VELATURA_MEDIDA` (R9) | P |

### §1.2 BRUSH (9) — a ponta, o depósito e o que ela apanha

| # | rótulo | tooltip | grandeza · unidade | faixa | omissão | alvo | guarda |
|---|---|---|---|---|---|---|---|
| 8 | **Tip** | The shape of the brush: round, flat, filbert, fan, knife, felt, spray… | geometria da ponta (asset) | lista | Round | **novo** asset `Tip` que leva `Felt`, `Bristle{Count,Strength,Size}`, os 20 `Bristle*` geométricos, `TuftArc`, `Dust/DustDrop`, `StrokeFilmDome/Ends` (§2) | P |
| 9 | **Spacing** | Distance between touches along the stroke, as a share of the brush width. | fracção do diâmetro | 0,005..0,5 | 0,025 | `Knob::CapsuleSpacing` (`defs:263` → `tp/wetpaint/lote.rs:87`, `wp/trail.rs:483`) absorve `Spacing`, `PigmentSpacing`, `CapsuleMaxPx` | P |
| 10 | **Bristle relief** | How deep each bristle carves its own groove into the paint. | variação de `h` por cerda · µm | 0..8 | 0 | `Knob::BristleHeight` (`defs:260`) ⚑ leitor «nenhum» no censo §1 #83 (via struct `Fios`, censo §6) | P |
| 11 | **Hardness** ⚑ | How deep the tip reaches into the paper's tooth: 0 = only the peaks, 1 = fills the valleys. | profundidade de contacto no dente · fracção de `z_s` | 0..1 | 1 | **novo**; funde a parte POR PINCEL de `PaperGate` (`wp/trail/deposit.rs:124`), `PressureGrain` (`deposit.rs:116,135`), `ExtDryBrush` (`deposit.rs:126`) | P |
| 12 | **Paper press** | How much the paint flattens the paper's tooth under it: 0 = follows the paper, 1 = presses it flat. | deformação do relevo `z_s` sob a tinta · fracção | 0..1 | 1 ⚑ (dono escolhe por foto 0/½/1) | `Knob::PaperPress` (`defs:331` → `wp/material.rs:48`, `wp/grid/material.rs:37`) | P |
| 13 | **Pickup** | How much wet paint the brush lifts from the canvas as it passes. | `k_u` · fracção por contacto (só o ABERTO: `k_u·o·A_c·v`) | 0..0,2 | 0,005 | `Knob::Pickup` (`defs:69` → `wp/trail/bico.rs:31,184`) absorve `WetLift` (o `TipClean` vai ao **Load**, §2.1 #10 — revisão §7 R5) | P |
| 14 | **Load** ⚑ | How much paint the brush holds — how far a stroke carries picked-up color before it runs clean. | `V_cap` · volume do reservatório (unid. de `h·A`) | 0..1 | 0 | `Knob::TipReservoir` (`defs:270` → `bico.rs:32,108,183,211`) | P |
| 15 | **Drag** | How much of the wet paint is pulled along with the stroke. | fracção de `vᶠ` arrastada por px | 0..1 | 0,16 | `Knob::Drag` (`defs:68` → `wp/trail/transfer.rs:115`) absorve `RewetDrag`, `WaterDrag` | P |
| 16 | **Stabilizer** | Smooths a shaky hand: the stroke follows the pen on a string of this length. | atraso · px | 0..400 | 0 | `Knob::PulledString` (`defs:240` → `tp/mao.rs:47,265`) absorve `BrushSpec.stabilizer` (D17) | P |

### §1.3 PAPER (4) — o suporte (por tela)

| # | rótulo | tooltip | grandeza · unidade | faixa | omissão | alvo | guarda |
|---|---|---|---|---|---|---|---|
| 17 | **Paper** | The paper or canvas you paint on. | campo de altura `z_s(x)` (asset) | lista | o papel texturizado padrão do app | cartão Paper existente; absorve o gerador `PaperContrast/Fibres/Grooves` (`wp/painter.rs:233-235`) | T |
| 18 | **Relief** | How strongly the paper's tooth stands out. | amplitude de `z_s` · µm | 0..1 (×`z_MAX`) | o de hoje | existente `PAINTER_SUBSTRATE_RELIEF` (`tp/../ids/painter_substrate.rs:25`) | T |
| 19 | **Absorbency** | How fast the paper drinks water: high = soft edges and quick soaking, low = paint sits on top. | coef. de Washburn (`ℓ = √(γ r cosθ t / 2η)`, W21) · µm/√s | 0..1 ⚑ | 0,4 (= `PaperGate` do produto) | **novo** por tela; funde `PaperGate` (parte do papel), `GateSaturation`, `Capillary`, `CapillaryGate` (`deposit.rs:124-125`, `wp/solver.rs:96-97`) | T |
| 20 | **Tilt** | Tilts the canvas so wet paint runs downhill. | ângulo + direção da tela · ° (2D num widget) | anel 0..8 · raio 0..11 | off | `tilt_on/ring/spoke` (`tp/wetpaint_settings.rs:585-592`, `session.rs:248-254`); `Gravity` vira constante | T |

### §1.4 TIME (4) — secar, reabrir, a orla (+ 3 ações)

| # | rótulo | tooltip | grandeza · unidade | faixa | omissão | alvo | guarda |
|---|---|---|---|---|---|---|---|
| 21 | **Dry speed** ⚑ | How fast water evaporates from the canvas. Oil barely dries while you paint. | `E` · µm/s, efectiva `E·(1−m)` | 0..8 | 1 | `Knob::Evaporation` (`defs:108` → `wp/sim.rs:434,536`, `gpu/passo.rs:692`) absorve `BaseEvaporation`, `Retention` | T ⚑ (#2) |
| 22 | **Re-wet** | How easily dry paint dissolves again when water touches it: 0 = permanent, 1 = lifts like watercolor. | taxa de reabertura `do/dt` com água · s⁻¹ (adim. no painel) | 0..1 | 0,5 (= 1 − `ExtStaining` 0,5) | `Knob::Rewet` (`defs:109` → `wp/sim.rs:435,537,577`) absorve `Dissolve` (`rewetPaint`), `Solubility`, `ExtStaining` | P |
| 23 | **Edge darkening** | How much pigment gathers at the edge of a wet stroke as it dries. | fluxo capilar para a orla (C97) · adim. | 0..200 | 12 | `Knob::EdgeDarkening` (`defs:111` → `wp/drying.rs:122`, `gpu/seca.rs:56`) absorve `EdgeSoftMass` | P |
| 24 | **Wet spread** ⚑ | How far pigment drifts through wet paint on its own. | difusão do pigmento no filme `D` · px²/s | 0..0,25 | 0 | `Knob::ExtDiffusion` (`defs:199` → `wp/sim.rs:459,545`, `wp/solver.rs:558`, `gpu/passo.rs:26`) | P |
| — | ações **Wet canvas** · **Dry canvas** · **Fast dry** | (botões) | ação, não grandeza | — | — | `PAINTER_WETPAINT_{WETCANVAS,DRYCANVAS,FASTDRY}` (`tp/wetpaint_settings.rs:620-631`) | — |

### §1.5 LIGHT (7) — o material da tinta (por pincel → célula) e a luz da cena (por tela)

| # | rótulo | tooltip | grandeza · unidade | faixa | omissão | alvo | guarda |
|---|---|---|---|---|---|---|---|
| 25 | **Roughness** | How matte the paint surface is: 0 = mirror gloss, 1 = chalky matte. | rugosidade · adim. | 0..1 | 0,5 | `Knob::Roughness` (`defs:293` → `wp/grid/material.rs:26`) absorve `OilGloss` e a linha Roughness do cartão (D20, D22) | P |
| 26 | **Metallic** | Makes the paint reflect like metal leaf. | metalicidade · adim. | 0..1 | 0 | `Knob::Metallic` (`defs:292` → `material.rs:20,25`) absorve a linha do cartão | P |
| 27 | **Coat** | A clear varnish layer on top of the paint. | peso do verniz · adim. | 0..1 | 0 | `Knob::Coat` (`defs:295` → `material.rs:28`) absorve a linha do cartão | P |
| 28 | **Light angle** | Where the light comes from. | azimute · ° | 0..360 | o de hoje | `OleoReal::azimute` (`tp/ids/painter_wetpaint.rs:116-160`) | T |
| 29 | **Light height** | How high the light is: low light shows every ridge. | altitude · ° | 5..90 | o de hoje | `OleoReal::altitude` | T |
| 30 | **Environment** | The surroundings reflected in glossy paint. | mapa do céu (enum 4) | lista | o de hoje | `OleoReal::ambiente` (`painter_wetpaint.rs:186`) | T |
| 31 | **Ambient** | How much soft light comes from all around. | força da luz ambiente · adim. | 0..1 | o de hoje | `OleoReal::forca_ambiente` | T |

A luz da cena **liga sempre** (não há caixa Real Oil): a tinta de `m = 0` sem relevo é plana por física (`h_seco(0) ≈ Σv`), não
por um ramo (F2 eq. 5). ⚑ que o render da LIGHT sobre a aquarela plana seja a identidade — `oleo_real_plano_e_a_identidade` (E5).

### §1.6 Linha da ferramenta (fora do N) e cartão Pen (inalterado)

| ferramenta | controlo | grandeza | alvo |
|---|---|---|---|
| Blend | **Strength** · **Reach** | força da mistura · alcance (raios do pincel) | `Knob::BlendForce` (`defs:98`) · `Knob::BlendReach` (`defs:325`) |
| Smear | **Strength** | fracção arrastada | `Knob::Smear` (`defs:183`); a água do Smear = a `Dilution` dele |
| Blow | **Strength** | força do sopro | `Knob::Blow` (`defs:173`) |
| Dry | **Strength** | absorção do secador | `Knob::Dryer` (`defs:163`) |
| Water | — (é um pincel com `Dilution 1`, `Volume`, `Pickup`) | — | preset Water Brush |
| Erase | **Strength** | fracção removida | `erase` × `Knob::Eraser` fundidos: `Strength = Eraser·erase` (`wp/tools.rs:98`, D11) |
| Knife | **Strength** | nivelamento pela lâmina | `Knob::KnifeLevel` (`defs:304` → `wp/lamina.rs:69,85`) |

Cartão **Pen**: as 15 linhas da mão do registo (`PressureSize` … `TiltLag`, `defs:233-247`) + as 8 fora do registo
(`tp/mao.rs:90-125`) ficam como estão; duas mudam só de rótulo: `PressureOpacity` → **Pressure → Volume** (é a curva
`h(p)`, F4_2 §7.1 `tp/mao.rs:255-260`), `tiltOpacity`/`speedOpacity` → **Tilt → Volume** / **Speed → Volume**.

## §2 Migração — cada controlo de hoje, uma linha

Destino: **S** fica (como `<rótulo>`) · **M** funde em `<rótulo>` pela fórmula · **C** vira consequência · **T** vira
ferramenta / asset `Tip` / cartão Pen · **R** sai (razão medida). «pr» = nº de pincéis de fábrica que o tiram do
padrão (matriz (b)). **⚑** = o destino depende de uma medição antes de decidir (§4).

### §2.1 Os 144 do registo

| # | chave (`defs:N`) | → | destino e fórmula | pr | ⚑ |
|---|---|---|---|---|---|
| 0 | `pigmentPerDab` (46) | M | **Dilution** = `1 − P/(P + K_W·veículo(W, WPD))` (`receita.rs:82-85`) | 44 | |
| 1 | `paperGate` (47) | M | parte do papel → **Absorbency** (tela); parte do pincel (lápis/pastel: 0..2,5) → **Hardness** = `clamp(1 − paperGate/2,5)` | 38 | ⚑ |
| 2 | `felt` (51) | T | asset `Tip` | 36 | |
| 3 | `bristleCount` (61) | T | asset `Tip` | 14 | |
| 4 | `drag` (68) | S | **Drag** | 20 | |
| 5 | `pickup` (69) | S | **Pickup** | 9 | |
| 6 | `intensity` (70) | M | **Volume** `= Opacity·carga` (a carga da E4, `receita.rs:50,88`) | 0 (45 após E4) | |
| 7 | `bristleStrength` (74) | T | asset `Tip` | 16 | |
| 8 | `bristleSize` (84) | T | asset `Tip` (2 pincéis gravam 7, fora de 0,2..4 — o asset guarda o valor sem prender) | 10 | ⚑ |
| 9 | `spacing` (93) | M | **Spacing** (o passo sintético em px ÷ diâmetro) | 0 | |
| 10 | `tipClean` (94) | M | **Load** (a ponta volta à cor dela à taxa `apanhado/V_cap`) | 6 | ⚑ |
| 11 | `blendForce` (98) | T | Blend **Strength** | 11 | |
| 12 | `gateSaturation` (105) | C | K48 satura sozinho (F2 §5) | 0 | |
| 13 | `waterPerDab` (106) | M | **Dilution** (no `veículo`) | 3 | |
| 14 | `waterCap` (107) | C | capacidade do papel = **Absorbency** × espessura do suporte | 1 (Wet Liner 30) | ⚑ |
| 15 | `evaporation` (108) | S | **Dry speed** (passa a por-tela) | 4 | ⚑ |
| 16 | `rewet` (109) | M | **Re-wet** | 0 | |
| 17 | `retention` (110) | M | **Dry speed**: `E' = E − ln(retention)/Δt` (a retenção é um `e^{−EΔt}`) | 1 (Gouache Opaque 0,97) | ⚑ |
| 18 | `edgeDarkening` (111) | S | **Edge darkening** | 42 | |
| 19 | `baseEvaporation` (112) | M | **Dry speed** (constante base fundida) | 0 | |
| 20 | `leveling` (116) | C | nivelamento O62 de `η(m,φ)`, `h`, `Body` | 0 | |
| 21 | `capillary` (126) | M | **Absorbency** | 2 | ⚑ |
| 22 | `brake` (136) | C | **Body** (`τ_y` pára a frente) | 0 | |
| 23 | `gravity` (143) | R | constante `g`; o controlo é **Tilt** (0 pincéis, nunca escrito pelo cartão `:574` fora do padrão) | 0 | |
| 24 | `levelClamp` (147) | R | numérico do solver (F2 §5) → constante | 0 | |
| 25 | `viscosity` (154) | C | `η(m, φ)` (F2 tabela) | 0 | |
| 26 | `maxVelocity` (155) | R | CFL, numérico → constante (dev Tuning) | 0 | |
| 27 | `projection` (156) | R | numérico → constante | 0 | |
| 28 | `brakeReach` (157) | C | **Body** | 0 | |
| 29 | `capillaryGate` (158) | M | **Absorbency** | 0 | |
| 30 | `eraser` (159) | M | Erase **Strength** = `Eraser·erase` (D11) | 0 | |
| 31 | `dryer` (163) | T | Dry **Strength** | 0 | |
| 32 | `blow` (173) | T | Blow **Strength** | 0 | |
| 33 | `smear` (183) | T | Smear **Strength** | 6 | |
| 34 | `paperContrast` (190) | T | asset **Paper** (gerador) | 3 (Knife Stain 3, preso) | ⚑ |
| 35 | `paperFibres` (191) | T | asset **Paper** | 0 | |
| 36 | `paperGrooves` (192) | T | asset **Paper** | 0 | |
| 37 | `visualGrain` (193) | R | grão impresso na cor não é física; o papel vê-se pela luz e por `R_g` (F2 §5) | 0 | |
| 38 | `emboss` (194) | C | relevo visto = `h_seco(m)` × **Light** | 15 | ⚑ |
| 39 | `paperVisibility` (197) | R | = `C` derivada (o papel vê-se onde a tinta não cobre) | 0 | |
| 40 | `extDiffusion` (199) | S | **Wet spread** | 1 | ⚑ |
| 41 | `extBackrun` (200) | C | backrun C97 de **Absorbency** × **Dry speed** × água | 0 | ⚑ |
| 42 | `extFingering` (201) | R | instabilidade da frente: 0 pincéis; se o olho a quiser, volta como consequência do escoamento | 0 | ⚑ |
| 43 | `extRivulets` (202) | R | só existe com `extFingering` | 0 | |
| 44 | `extGranulation` (203) | S | **Granulation** | 0 | |
| 45 | `extStaining` (209) | M | **Re-wet** = `1 − staining` (D14: inversos) | 0 | |
| 46 | `extDryBrush` (210) | M | **Hardness** (falha no dente = contacto raso) + consequência de `Volume` baixo | 2 | ⚑ |
| 47 | `extWetSoften` (211) | C | **Wet spread** + filme molhado | 0 | |
| 48 | `extRakeLight` (212) | R | MORTO no produto (só `render_region`, só testes — censo §5) | 0 | |
| 49 | `extValleyGran` (213) | R | MORTO (idem) | 0 | |
| 50 | `extWetSheen` (214) | R | MORTO (idem); o brilho molhado é `Roughness` do filme | 0 | |
| 51 | `extGrainDither` (215) | R | MORTO (idem) | 0 | |
| 52 | `extEdgeTint` (216) | R | MORTO (idem) | 0 | |
| 53 | `wetLift` (221) | M | **Pickup** (`k_u·o`, `o = 1` no molhado) | 1 (Water Brush 0,7) | ⚑ |
| 54 | `glaze` (222) | M | **Transparency** `= max(T, glaze)` (a `vela` filtrava = `S` pequeno) | 7 | ⚑ |
| 55 | `dust` (226) | T | asset `Tip` «Spray» (partículas) | 1 (Airbrush 0,8) | |
| 56 | `pressureSize` (233) | T | cartão Pen | 43 | |
| 57 | `pressureOpacity` (234) | T | cartão Pen «Pressure → Volume» (`h(p)`) | 57 | |
| 58 | `pressureOpacityCurve` (235) | T | cartão Pen | 34 | |
| 59 | `pressureSizeCurve` (236) | T | cartão Pen | 30 | |
| 60 | `pressureFloor` (237) | T | cartão Pen | 2 | |
| 61 | `pressureCeiling` (238) | T | cartão Pen | 1 | |
| 62 | `pressureSmoothing` (239) | T | cartão Pen | 0 | |
| 63 | `pulledString` (240) | S | **Stabilizer** | 0 | |
| 64 | `dabRotation` (241) | T | cartão Pen | 6 | |
| 65 | `tiltStart` (242) | T | cartão Pen | 5 | |
| 66 | `tiltFull` (243) | T | cartão Pen | 5 | |
| 67 | `tiltWidth` (244) | T | cartão Pen | 5 | |
| 68 | `tiltLength` (245) | T | cartão Pen | 5 | |
| 69 | `tiltSmooth` (246) | T | cartão Pen | 0 | |
| 70 | `tiltLag` (247) | T | cartão Pen | 5 | |
| 71 | `bristlePressure` (248) | T | asset `Tip` (leitor «nenhum» no censo §1) | 17 | ⚑ |
| 72 | `bristleNoiseLength` (249) | T | asset `Tip` | 2 | ⚑ |
| 73 | `bristleLaneWidth` (250) | T | asset `Tip` | 16 | ⚑ |
| 74 | `bristleWave` (251) | T | asset `Tip` (0 pincéis; sai se o leitor for morto) | 0 | ⚑ |
| 75 | `bristleWaveLength` (252) | T | asset `Tip` (idem) | 0 | ⚑ |
| 76 | `bristleOscillation` (253) | T | asset `Tip` | 1 | ⚑ |
| 77 | `bristleSpan` (254) | T | asset `Tip` | 1 | ⚑ |
| 78 | `bristleSpanLength` (255) | T | asset `Tip` | 1 | ⚑ |
| 79 | `bristleNoiseHue` (256) | T | asset `Tip` (variação de PIGMENTO por cerda no reservatório, F2 §5 — nunca cor) | 0 | ⚑ |
| 80 | `bristleNoiseSat` (257) | T | asset `Tip` (idem) | 14 | ⚑ |
| 81 | `bristleNoiseLight` (258) | T | asset `Tip` (idem) | 9 | ⚑ |
| 82 | `bristleLift` (259) | T | asset `Tip` | 10 | ⚑ |
| 83 | `bristleHeight` (260) | S | **Bristle relief** | 11 | ⚑ |
| 84 | `bristleRemove` (261) | M | **Bristle relief** (escavar = o mesmo `Δh` com sinal) | 0 | |
| 85 | `pigmentSpacing` (262) | M | **Spacing** | 16 | ⚑ |
| 86 | `capsuleSpacing` (263) | S | **Spacing** (o alvo) | 0 | |
| 87 | `capsuleMaxPx` (264) | M | **Spacing** (o tecto em px vira `min(spacing·d, cap)` constante) | 14 | ⚑ |
| 88 | `capsuleRoundNearest` (265) | R | bool numérico (censo §5) | 15 | ⚑ |
| 89 | `embossSizeExp` (266) | R | D18: em unidades físicas a escala com o diâmetro sai por construção | 6 | ⚑ |
| 90 | `embossSizeRef` (267) | R | D18 | 0 | |
| 91 | `bristleTipRagged` (268) | T | asset `Tip` (5 pincéis gravam 1,6, fora de 0..1) | 7 | ⚑ |
| 92 | `levelingOwnStroke` (269) | C | **Body** (o traço fica isolado porque não escoa) | 15 | ⚑ |
| 93 | `tipReservoir` (270) | S | **Load** | 4 | ⚑ |
| 94 | `pressureGrain` (271) | M | **Hardness** × a curva Pen (pressão → profundidade de contacto) | 3 | ⚑ |
| 95 | `strokeFilm` (272) | C | modo bool (R17): na lei única o filme é a única água — consequência de **Medium** | 2 | ⚑ |
| 96 | `dustDrop` (273) | T | asset `Tip` «Spray» | 0 | |
| 97 | `bristleStreak` (274) | T | asset `Tip` (estético; leitor `wp/bristles_fios.rs:47`) | 0 | ⚑ |
| 98 | `bristleStreakLight` (275) | T | asset `Tip` (leitor «nenhum») | 0 | ⚑ |
| 99 | `bristleStreakLength` (276) | T | asset `Tip` (idem) | 0 | ⚑ |
| 100 | `bristleStreakCut` (277) | T | asset `Tip` (idem) | 0 | ⚑ |
| 101 | `thinStrokeFade` (278) | R | 0 pincéis; correcção do traço fino → `agua_do_traco_fino` (E6a) | 0 | |
| 102 | `thinStrokeRef` (279) | R | idem | 0 | |
| 103 | `waterPush` (280) | C | a água pousada (φ = 0) empurra pelo escoamento comum | 1 (Water Brush 0,35) | ⚑ |
| 104 | `dryerEdge` (281) | C | **Edge darkening** sob a ferramenta Dry | 0 | |
| 105 | `wetOpen` (282) | C | D19: abrir o molhado alheio = `Pickup`·`o` + escoamento | 2 | ⚑ |
| 106 | `wetOpenPush` (283) | C | idem | 2 | ⚑ |
| 107 | `smearWater` (285) | M | a **Dilution** do Smear (o mesmo controlo, no preset do Smear) | 1 (Smear Wet 0,9) | |
| 108 | `knifeRidge` (287) | C | a pasta empurrada amontoa-se: **Body** × **Volume** na faca | 4 | ⚑ |
| 109 | `textureSizeExp` (289) | R | D18 | 1 | |
| 110 | `textureSizeRef` (290) | R | D18 | 0 | |
| 111 | `metallic` (292) | S | **Metallic** | 1 | |
| 112 | `roughness` (293) | S | **Roughness** | 1 | |
| 113 | `iorLevel` (294) | C | `n_v(m)` do **Medium** (F2: `IorLevel` duplica `n_v`) | 0 | |
| 114 | `coat` (295) | S | **Coat** | 1 | |
| 115 | `coatRoughness` (296) | R | constante 0,03 (0 pincéis; uma grandeza sem controlo é legítima) | 0 | ⚑ |
| 116 | `sheen` (297) | R | 0 pincéis; veludo não é grandeza de tinta que pintemos | 0 | ⚑ |
| 117 | `subsurface` (298) | C | o espalhamento K-M (`S`) JÁ é a subsuperfície da camada | 0 | |
| 118 | `subsurfaceRadius` (300) | C | idem | 0 | |
| 119 | `transparency` (301) | S | **Transparency** | 0 | |
| 120 | `lockMaterial` (302) | R | bool; o material da célula é a média pelo volume (como `m`, E9 §7.1) | 0 | |
| 121 | `knifeLevel` (304) | T | Knife **Strength** | 1 | |
| 122 | `textureSizeExpAcross` (306) | R | sentinela −0,01 (R23), D18 | 0 | |
| 123 | `transmission` (308) | M | **Transparency** | 0 | |
| 124 | `paperRelief` (309) | C | **Relief** × (1 − `C`) pela luz | 0 | |
| 125 | `transmissionMode` (311) | R | bool | 0 | |
| 126 | `oilDepth` (314) | C | `h_seco(m)·Volume` (o relevo é tinta que ficou, F2 §5); os 10 pincéis 0,1..0,9 perdem o ganho | 10 | ⚑ |
| 127 | `oilGloss` (315) | M | **Roughness** `= 1 − oilGloss·(1 − roughness)` | 4 | ⚑ |
| 128 | `wetOpenEdge` (317) | C | D19 | 1 (Wet Liner 28) | ⚑ |
| 129 | `strokeFilmDome` (318) | T | asset `Tip` (perfil da carga através da ponta) | 2 | ⚑ |
| 130 | `strokeFilmEnds` (319) | T | asset `Tip` (perfil ao longo) | 2 | ⚑ |
| 131 | `dryerLevel` (321) | R | 0 pincéis; o secador só tira água | 0 | |
| 132 | `blendReach` (325) | T | Blend **Reach** | 0 | |
| 133 | `wetCover` (327) | M | **Transparency** 0 + **Dilution** baixa (cobertura derivada) | 1 (Gouache Opaque) | ⚑ |
| 134 | `bristleLaneSizeExp` (329) | R | D18 | 1 (Fan Brush 0,8) | ⚑ |
| 135 | `paperPress` (331) | S | **Paper press** | 0 | |
| 136 | `rewetPaint` (334) | M | **Re-wet** (`Dissolve`, mesmo rótulo de produto que `rewet`, D8) | 0 | |
| 137 | `solubility` (335) | M | **Re-wet** | 0 | |
| 138 | `rewetDrag` (336) | M | **Drag** | 0 | |
| 139 | `edgeSoftMass` (339) | M | **Edge darkening** (a orla graduada pela massa é C97) | 1 (Wet Wash 4) | ⚑ |
| 140 | `dryCover` (341) | M | **Transparency** | 1 (Gouache Opaque) | ⚑ |
| 141 | `tuftArc` (343) | T | asset `Tip` | 1 (Fan Brush) | |
| 142 | `waterDrag` (345) | M | **Drag** (do preset Water Brush) | 1 | |
| 143 | `opaque` (347) | M | **Transparency** `= 0` onde `opaque = 1` | 3 (Fan Brush, Gouache Flat/Opaque) | ⚑ |

### §2.2 Os de fora do registo (censo §2, desdobrados: 36)

| controlo | id | → | destino |
|---|---|---|---|
| Water | `PAINTER_WETPAINT_WATER` (`wetpaint_settings.rs:571`) | M | **Dilution** (no `veículo`) |
| Opacity | `sliders.pigmento` (`tp/wetpaint_preset.rs:444`) | S | **Volume** (rótulo novo; já é `h` desde a E4) |
| Erase | `PAINTER_WETPAINT_ERASE` (`:581`) | M | Erase **Strength** = `Eraser·erase` |
| Pigment mixing (K-M) | `km_mixing` (`:661`) | S | **Color mixing** (Smooth \| Pigment); a velatura escondida R9 morre |
| Glaze layering (K-M) | `km_glaze` (`:667`) | M | **Transparency** (R2c `vidrado`) |
| Paint Texture | `paper_visual` (`:638`) | R | o papel na tinta vê-se pela luz e `R_g`; caixa sem grandeza (D12) |
| Real Oil | `oleo_real` (`:643`) → `sim.oleo` | M | **Medium** `m = 1` + LIGHT sempre ligada (R3–R6, R12–R13 morrem na E12) |
| Shadow | `OleoReal::sombra` | C | sombra = luz × `h` (render) ⚑ olho do dono |
| Light height | `OleoReal::altitude` | S | **Light height** |
| Light angle | `OleoReal::azimute` | S | **Light angle** |
| Crevice shadow | `OleoReal::fenda` | C | oclusão do relevo (render) ⚑ |
| Crevice reach | `OleoReal::alcance_fenda` | C | idem ⚑ |
| Environment light | `OleoReal::forca_ambiente` | S | **Ambient** |
| Environment | `OleoReal::ambiente` | S | **Environment** |
| Metallic (cartão) | `MaterialCartao` (`wp/oleo_material.rs:24`) | M | **Metallic** por pincel (D20: hoje `max`, `wp/material.rs:146`) |
| Roughness (cartão) | idem | M | **Roughness** (hoje por razão, `material.rs:147`) |
| IOR Level (cartão) | idem | C | `n_v(m)` |
| Coat (cartão) | idem | M | **Coat** |
| Coat Roughness (cartão) | idem | R | constante (como `coatRoughness`) |
| Sheen (cartão) | idem | R | como `sheen` |
| Subsurface (cartão) | idem | C | K-M `S` |
| Subsurface Radius (cartão) | idem | C | K-M `S` |
| Live Water | `agua_viva` (`:653`) | R | agendamento, não grandeza → definição de desempenho; a velatura R10 morre (E12) |
| Heavy Edge | `borda_que_pesa` (`:559`) | C | gravidade (**Tilt**) + sedimentação (**Granulation**) — `wp/solver/sedimenta.rs:78-92` |
| Tilt (toggle) | `tilt_on` (`:615`) | S | **Tilt** (desligado = anel 0) |
| Tilt ring | `tilt_ring` (`:585`) | S | **Tilt** |
| Tilt spoke | `tilt_spoke` (`:590`) | S | **Tilt** |
| Grid Size | `PAINTER_WETPAINT_GRID` (`:544`) | R | resolução numérica → «Quality» da tela |
| Flow Grid | `PAINTER_WETPAINT_FLOW` (`:548`) | R | idem |
| Smooth Edges | `PAINTER_WETPAINT_SMOOTH_EDGES` (`:552`) | R | antisserrilhado → «Quality» |
| Wet canvas | `PAINTER_WETPAINT_WETCANVAS` (`:620`) | T | ação na TIME |
| Dry canvas | `PAINTER_WETPAINT_DRYCANVAS` | T | ação na TIME |
| Fast dry | `PAINTER_WETPAINT_FASTDRY` | T | ação na TIME |
| Show wet | `PAINTER_WETPAINT_SHOWWET` (`:632`) | S | vista (cabeçalho do painel, fora do N) |
| Tuning | `PAINTER_WETPAINT_TUNING` (`:657`) | S | painel dev (lista os 31 + constantes) |
| Reset | `PAINTER_WETPAINT_RESET` (`:611`) | S | «Reset to preset» |

### §2.3 Mão fora do registo (8) e campos do preset fora de `engine`

| campo | → | destino |
|---|---|---|
| `tiltOpacity`, `speedOpacity` | T | cartão Pen, «Tilt → Volume», «Speed → Volume» |
| `speedSize`, `pressureSCurve`, `pressureJitter`, `speedLag`, `pressureHardness`, `speedHardness` | T | cartão Pen, inalterados |
| `volumes.size_px` | S | Size (barra comum a todos os modos) |
| `volumes.opacity` | M | **Volume** (já entrou na carga, `receita.rs:116`) |
| `volumes.water` | M | **Dilution** |
| `volumes.{pressure,length}` | R | sem leitor no Wet Paint (censo §2 rodapé) |
| `paint_mode`, `ferramenta` | T | a ferramenta (linha da ferramenta) |
| `live_water`, `heavy_edge`, `tilt` | R/C/S | como as linhas de cima |
| `brush.*` (42 lidos, `tp/wetpaint_preset/campos.rs:5`) | T | asset `Tip` / Pen; `brush.stabilizer` → **Stabilizer**; `brush.spacing` → **Spacing** |

### §2.4 Contagem (das 188 linhas: 144 + 36 + 8)

| destino | registo (144) | fora (36) | mão (8) | total |
|---|---|---|---|---|
| S fica | 15 | 12 | 0 | **27** |
| M funde | 31 | 7 | 0 | **38** |
| C consequência | 22 | 7 | 0 | **29** |
| T ferramenta / `Tip` / Pen / ação | 49 | 3 | 8 | **60** |
| R sai | 27 | 7 | 0 | **34** |
| ⚑ precisa de medição antes | 57 | 3 | 0 | **60** |

(Contado por script sobre as tabelas §2.1/§2.2. Os 31 = 15 S do registo + Volume + Color mixing + 4 da luz + Tilt
(3 linhas, 1 controlo) = 22, + 5 sem antecessor único (`Medium`, `Dilution`, `Hardness`, `Body`, `Absorbency`) + `Tip`,
`Paper`, `Relief` (assets/cartão Paper existentes) + `Re-wet` (funde 5). Show wet/Tuning/Reset são S fora do N.)

## §3 Presets de fábrica por FUNÇÃO

Uma função `migra_para_controlos(preset_velho) -> Controlos` em `tp/wetpaint_preset/` (nunca tabela à mão), marca
`controlosE10 = 1` no `engine` (como a `receitaE4`, E4 «Migração»). Ordem: a E4 já correu (`migra_da_lei_velha`,
`receita.rs:107-121`) ⇒ a E10 lê `P' = PPD·Op`, `carga`, `W`, `WPD`.

### §3.1 As fórmulas (uma por controlo novo)

| controlo | fórmula a partir do preset velho |
|---|---|
| Medium | tabela por FAMÍLIA (§3.2) — nunca por `realOil` |
| Dilution | `1 − P'/(P' + K_W·veículo(W, WPD))` — exacto: φ por dab igual ao da E4 (`receita.rs:82-85`) |
| Volume | `Op · carga` (carga da E4: 0,011 Airbrush … 2,109 Broad Nib/Dabber — dentro de 0..4) |
| Transparency | `max(transparency, glaze, km_glaze ? 1 : 0, transmission)`, depois `·(1 − max(opaque, wetCover, dryCover))` |
| Granulation | `extGranulation` |
| Body | do meio: `τ_y(m, φ)` sem acréscimo (`Body = 0`), salvo `levelingOwnStroke = 1` ⇒ `Body = B₁` ⚑ (medir B₁ que dá o mesmo traço isolado — 15 Oil/Gouache) |
| Color mixing | `pigmentMixing = 0` (Wet Liner, Wet Wash, Gouache Opaque) ⇒ Smooth; o resto ⇒ Smooth (K-M 3 bandas recusado, F2b) |
| Tip | o asset cujo conteúdo é o par (`felt`, `bristle*`, `tuftArc`, `dust*`, `strokeFilm{Dome,Ends}`, `brush.*` da forma) do preset — um asset por preset de fábrica na 1.ª versão, deduplicado por igualdade |
| Spacing | `capsuleSpacing` (todos no padrão 0,025); `pigmentSpacing ≠ 0,025` (16) e `capsuleMaxPx` (14) ⚑ |
| Bristle relief | `bristleHeight − bristleRemove·0` (bristleRemove nunca fora do padrão) |
| Hardness | dry media (Pencil, Pastel): `1 − clamp(paperGate/2,5)` × (`1 − extDryBrush`); o resto 1 ⚑ |
| Paper press | `paperPress` (todos no padrão 1) |
| Pickup | `pickup + wetLift·[Water Brush]` ⚑ |
| Load | `tipReservoir`; `tipClean` ≠ padrão (6: tintas/marcadores, 3 gravam 1 = fora da faixa) ⇒ `Load = 0` ⚑ |
| Drag | `max(drag, rewetDrag, waterDrag)` |
| Stabilizer | `pulledString` (0 em todos) |
| Edge darkening | `edgeDarkening` (`edgeSoftMass` do Wet Wash ⚑) |
| Re-wet | `max(rewet/8, rewetPaint, solubility, 1 − extStaining)` — o padrão dá 0,5 |
| Wet spread | `extDiffusion` |
| Roughness | `1 − oilGloss·(1 − roughness)` só onde `realOil = 1`; senão `roughness` |
| Metallic · Coat | `metallic` · `coat` |

### §3.2 O meio por família (os 55 Wet Paint; Digital 21 e Watercolor 1 não têm meio)

| família (n) | pincéis | `m` | o resto que muda | olho do dono |
|---|---|---|---|---|
| Oil/* (14) | Dabber ×2, Filbert, Filbert Dry, Flat Bristle, Thick Impasto, Tilted Shade, Knife ×6, Metallic Flat | **1** | `realOil` some; Roughness pela fórmula; relevo = consequência (os 10 `oilDepth` 0,1..0,9 deixam de o fixar) | ⚑ todos: relevo, e E9 §7.3 #2 (a `m = 1` o dab cobre 3,98× menos) |
| ExpressOil (1) | Express Oil | **1** (hoje 0 — errado, E9 §7.1) | — | ⚑ alquídico: 1 ou ~0,85 (seca mais depressa por `τ(m)`) |
| Acrylic (1) | Fan Brush | **0,5** ⚑ | `opaque 1` ⇒ Transparency 0; Re-wet 0 (filme insolúvel) | ⚑ valor de `m`: a acrílica evapora a água MAS o ligante fica (`n_v ≈ 1,48`, não clareia) |
| Watercolor/Fluid (6) | Wet Drip, Wet Dry Brush, Wet Flat, Wet Liner, Wet Round, Wet Wash | **0** | Wet Liner/Wet Wash: `glaze 0,2` → Transparency 0,2; `strokeFilm` → Tip; `edgeSoftMass` ⚑ | Wet Liner (`waterCap 30`, `wetOpenEdge 28`) |
| Watercolor/Gouache (2) | Gouache Flat, Gouache Opaque | **0** | `opaque`/`wetCover`/`dryCover` → Transparency 0; Re-wet 0,5 (a guache re-molha); `retention 0,97` → Dry speed ⚑ | ⚑ ambos (a cobertura passa a sair de φ·h) |
| Ink (4) | Ink Brush, Broad Nib, Dip Pen, Ink Bullet | **0** | `glaze` 0,4/0,75 → Transparency; `tipReservoir` → Load; Re-wet 0 (tinta permanente) ⚑ | ⚑ Re-wet das tintas |
| Marker (2) | Marker Bullet, Marker Chisel | **0** | `glaze` 1/0,5 → Transparency; `tipClean 1` (fora da faixa) → Load 0 | |
| Pastel/Dry (3) | Pastel Side, Soft Pastel, Square Hard | **0** (Dilution 0: sem veículo) | `paperGate` → Hardness | ⚑ Hardness |
| Pastel/Oil + Scrape (3) | Oil Pastel, Wax Crayon, Wax Scrape | **1** ⚑ (ligante de cera não evapora) | `paperGate` → Hardness; Body alto | ⚑ `m` da cera |
| Pencil (8) | Charcoal, Compressed Charcoal, Colored 4B, Graphite 2B/2H/8B/HB, Scratch | **0** (Dilution 0) | `paperGate` → Hardness | ⚑ Hardness |
| Airbrush (1) | Airbrush | **0** | `dust 0,8` → Tip «Spray» | |
| Blend (4) · Smear (5) | Blender Dots/Hard/Soft/Textured · Smear Flat/Shattered/Soft/Spikey/Wet | — (não pousam tinta: o meio vem da célula, E9 §7.1) | Strength da ferramenta; Smear Wet: `smearWater 0,9` → Dilution do Smear | |
| Water (1) | Water Brush | **0**, Dilution **1** (φ = 0) | `wetLift 0,7` → Pickup ⚑; `waterPush`/`waterDrag` → consequência / Drag | ⚑ |

Recontar na E10: a E9 §7.1 diz «15 `Oil/*`»; a matriz (`matriz.tsv`, `realOil` = 1) e `assets/brushes/WetPainter/Oil/`
dão **14**. O número conta-se, não se escolhe.

## §4 Testes por nome

| teste | o que mede | limiar |
|---|---|---|
| `cada_controlo_mexe_uma_grandeza` | para cada um dos 31 controlos `cᵢ`, em 3 pontos (omissão, preset Thick Impasto, preset Wet Round): `cᵢ ± δ` (δ = 5 % da faixa) e lê o vetor de grandezas `Q` no `Engine` direto (sem ferramenta): `Receita{φ,m,h}`, `S` do pigmento, `γ_ads`, `τ_y`, `k_u`, `V_cap`, fracção de arrasto, passo, profundidade de contacto, prensa, `E`, taxa de re-wet, fluxo da orla, `D`, material (3), luz (4), absorção, tilt | jacobiano DIAGONAL: `|∂Q_j/∂c_i| ≤ 1e-9·|Q_j|` para `j ≠ i`; `|∂Q_i/∂c_i| > 0` (vivo). `η` e `τ_y` dependem de `m` e φ por LEI — vão numa linha «derivadas» declarada, não contam como fora da diagonal |
| `cada_preset_de_fabrica_mapeia_por_funcao` | os 55 Wet Paint pela `migra_para_controlos`: (a) φ e `h` por dab iguais à E4; (b) `m` = tabela §3.2; (c) nenhum valor preso pela faixa (a coluna «painel não alcança» da matriz vai a zero); (d) gravar e reler é idempotente (`controlosE10`) | (a) tol. relativa `1e-6` (a da E4); (b)(c)(d) exacto |
| `o_meio_viaja_do_controlo_ao_deposito` | `Medium` = 0 / 0,5 / 1 → `Pigmento::meio` e `Σv·m ÷ Σv` da célula depois de um dab | igual a `m` ± `1e-6`; com `realOil` removido, Express Oil pousa `m = 1` |
| `volume_fixo_varrer_diluicao_nao_muda_o_volume_depositado` · `diluicao_fixa_varrer_volume_nao_muda_a_concentracao` | os dois portões da E4 reescritos sobre `Dilution`/`Volume` (ADR §1.1) | espalhamento `< 1e-7` (o da E4) |
| `o_painel_tem_31_controlos_vivos_em_cada_meio` | censo re-corrido sobre o painel novo: cada controlo tem escritor, leitor de produção e muda o píxel em `m = 0` e `m = 1` (ADR §4.4) | 0 mortos, 0 duplicados, N contado = 31 |
| `mudar_o_painel_do_pincel_nao_muda_a_tinta_pousada` · `um_controlo_do_pincel_nao_recompoe_a_tela` | os gates da memória do Paper press, estendidos a todo controlo P | píxeis ao bit |
| `um_controlo_da_tela_nao_viaja_no_preset` | carregar um preset não muda Paper, Relief, Absorbency, Tilt, Dry speed nem a luz | exacto |
| `miniaturas_de_fabrica_golden` (ADR §4.5) | as 55 miniaturas antes × depois | só mudam as da lista ⚑ da §3.2, cada uma com foto em `fotos/f3/` |
| `toda_string_do_painel_esta_no_i18n_em_ingles` | rótulos e tooltips da §1 por chave `panel.wet.*` (HR-15) | 0 literais, 0 chaves em falta |
| `portao_mae_f0_lei_pigmento` (E9) re-corrido | a varredura com os controlos novos | o limiar da E9 (manchas 0, ferrugem 0, furos 0, degraus 0) |

**Medições ⚑ a fazer ANTES de fixar o destino (um bloco, uma rodada intercalada — §0.10 do CLAUDE.md):**
(1) `Body`: varrer `τ_y(m, φ)` sem acréscimo × os 15 pincéis `levelingOwnStroke = 1` e a régua do RELEVO — se o meio basta,
`Body` sai (N = 30). (2) `Hardness`: os 11 lápis/pastéis com `paperGate` → `Hardness` pela fórmula, foto lado a lado.
(3) Leitores dos 14 `Bristle*` com «nenhum» no censo: grep pela struct `Fios`/`BristleKnobs` + 1 dab a 0 e ao máximo —
mortos saem (R), vivos vão ao `Tip`. (4) Faixa e unidade de `Volume` (µm com `M_UM`) e o `h_MAX` que o Thick Impasto pede
(nunca limite antes da medição, §0.0). (5) `Absorbency`: o valor por tela que reproduz os 38 `paperGate` de pincéis de
tinta (hoje 0..1,2). (6) Os 15 `emboss` + 10 `oilDepth`: relevo antes × depois com `m = 1`. (7) Shadow/Crevice como
consequência: foto do dono.

## §5 Maquete (inglês, colapsáveis)

```
┌ Wet Paint ───────────────────────── [Show wet] [⟲ Reset] ┐
│ Tool: (Paint) Blend Smear Blow Dry Water Erase Knife      │
│       Strength ━━━━━━━○━━ 0.80   Reach ━○━━━━ 0.20 (Blend) │
├ ▾ PAINT ──────────────────────────────────────────────────┤
│  Medium        water ━━━━━━━━━━○ oil            1.00       │
│  Dilution      tube  ━━━○━━━━━━━ clear          0.33       │
│  Volume        thin  ━━━━○━━━━━━ thick          1.00       │
│  Transparency  hides ○━━━━━━━━━━ glazes         0.00       │
│  Granulation   ○━━━━━━━━━━━━━━━                 0.00       │
│  Body          flows ━━━━━━○━━━━ stays          (medium)   │
│  Color mixing  [Smooth|Pigment]                            │
├ ▾ BRUSH ──────────────────────────────────────────────────┤
│  Tip           [▣ Flat Bristle ▾]                          │
│  Spacing  ━○━━━━━ 0.025   Bristle relief ━━○━━━ 0.75       │
│  Hardness ━━━━━━━○ 1.00   Paper press    ━━━━━━○ 1.00      │
│  Pickup   ○━━━━━━━ 0.005  Load           ○━━━━━━ 0.00      │
│  Drag     ━━○━━━━━ 0.16   Stabilizer     ○━━━━━━ 0 px      │
├ ▸ PAPER   [Cold press ▾]  Relief · Absorbency · Tilt (◎)   │
├ ▸ TIME    Dry speed · Re-wet · Edge darkening · Wet spread │
│           [Wet canvas] [Dry canvas] [Fast dry]             │
├ ▸ LIGHT   Roughness · Metallic · Coat │ Light angle ◔      │
│           Light height · Environment ▾ · Ambient           │
└ Pen ▸ (pressure, tilt, speed — shared with Digital)        ┘
```

Cada rótulo abre o cartão de ajuda animado (variante C do dono, [F5](F5_maquete_ajuda.md)), pintado pelo motor sobre o papel
texturizado padrão.

## §6 Perguntas ao dono (só ele responde)

1. **Dilution num slider só, ou Pigment + Water como hoje?** Os dois juntos só mexem numa coisa (a diluição); com dois,
   duplicar os dois não muda nada. Proposta: um slider «Dilution». Alternativa: os dois, com a diluição escrita ao lado.
2. **Dry speed é da sala (da tela) ou do pincel?** Hoje 4 pincéis secam mais depressa/devagar (Ink Brush 3×, Wet Drip
   0,6×). Proposta: da tela; o óleo já seca devagar sozinho pelo meio.
3. **Paper press por omissão: 0, ½ ou 1?** (aberto desde 28/10, por foto.)
4. **O meio da acrílica (Fan Brush), do Express Oil e dos pastéis de óleo/cera** — 0,5 · 1 · 1 propostos, ver por foto.
5. **Shadow e Crevice saem como sliders** (a sombra passa a sair da luz e da espessura). Aceita, ou quer um «Shadow
   strength» artístico?
6. **As riscas e o ruído de cor por cerda** (Bristle streaks / jitter) ficam dentro da ponta (Tip) — sem slider no painel.

### ⛔ Recusas já medidas (não reconstruir)

| recusa | medição | onde |
|---|---|---|
| Mistura K-M de 3 bandas como padrão | azul + amarelo ≈ preto, ruído 20× | F2b, memória `project_rebelle_no_longer_the_reference` |
| Um knob «Opaque» para a cobertura | o azul esconde por absorção a `h` pequeno, o amarelo precisa de 10× — um knob não exprime (fig. b) | F2 §4 |
| Água = receita × pressão invertida | volume variou ×28,9 ao varrer P × W com Volume fixo | E4, `volume_fixo_…` |
| Rebelle como oráculo / portão de paridade | revogado pelo dono 31/10 | ADR-0197 «Alternativas» |
| Digital também por pigmento | o artista digital espera Porter-Duff | ADR-0197 §2 |
| Apagar o «papel amassado» para curar a assimetria | ordem do dono 28/10 | memória `project_paper_press_is_a_brush_control` |
| O meio da célula como `f32` próprio | era sempre 0; o meio é extensivo e viaja nas somas (`Σv·m`) | E9 §7.1 |

## §7 Revisão do coordenador (janela 4, 31/10) — o que fica decidido

O rascunho §0–§6 é ACEITE como desenho da E10, com estas decisões técnicas (não vão ao dono):

| # | ponto | decisão | porquê |
|---|---|---|---|
| R1 | `Body` talvez redundante | fica no desenho PROVISÓRIO; a medição §4 (1) decide. Se `τ_y(m, φ)` sem acréscimo reproduz os 15 traços `levelingOwnStroke = 1` na régua do RELEVO, `Body` sai e N = 30 | um controlo sem grandeza própria é um controlo que mente (ADR-0197 §1.1) — mas só a régua o prova |
| R2 | presets de óleo a `m = 1` cobrem 3,98× menos | a FÍSICA fica: `s(m)` é a lei (o óleo tem índice próximo do pigmento, espalha menos, é mais transparente — é por isso que se faz o vidrado a óleo). A migração preserva `h` (Volume), nunca a cobertura. As miniaturas que mudam vão à lista ⚑ de `miniaturas_de_fabrica_golden`, cada uma com foto | compensar a cobertura no preset seria um 2.º controlo da mesma grandeza escondido na migração |
| R3 | o Thick Impasto não chega ao masstone (3,6 µm) | o defeito é de UNIDADE, não de lei: um empaste real tem 100–1000 µm. A medição §4 (4) fixa a unidade do `Volume` em µm de tinta MOLHADA e a faixa pelo que a grade e o `f32` aguentam (§0.0 do CLAUDE.md: o tecto é o do recurso), nunca por `H_REF` herdado | o mesmo número `M_UM` calibrado no branco a `m = 0` não pode decidir a espessura do óleo |
| R4 | `paperGate` partido entre papel e pincel | ACEITE: `Hardness` (P) + `Absorbency` (T), fórmulas §3.1 | são duas grandezas (contacto no dente × velocidade de Washburn) que hoje partilham um número |
| R5 | `TipClean` aparecia em dois destinos (§1.2 #13 Pickup e §2.1 #10 Load) | vai ao **Load** (a ponta volta à cor dela à taxa `apanhado/V_cap`); §1.2 corrigido | uma grandeza, um controlo |
| R6 | 14 knobs de cerda sem leitor | medição §4 (3): grep + 1 dab a 0 e ao máximo; mortos saem (R), vivos vão ao `Tip` | §5.0 «controlo morto» |
| R7 | `Re-wet = max(rewet/8, …)` | ⚑ o divisor 8 é a faixa de hoje do `rewet` (`defs:109`); confirmar na medição §4 que `Re-wet` 0,5 dá a mesma reabertura do padrão | fórmula de migração sem prova é escolha |
| R8 | «Color mixing» Smooth \| Pigment | **substituído por um SLIDER `Pigment mix` 0..1** (omissão 0) que interpola Oklab → WGM 10 bandas (F2b §4.2, como o slider Pigment do MyPaint); o `p` viaja na soma como o meio (`Σv·p`), e a cor da célula é `lerp_Oklab(ō, WGM, p̄)`. Implementa-se na E10b (`E10b_pigment_mix.md`); o slider só aparece no painel quando tiver leitor. As linhas «Color mixing» acima (§1.1 #7, §2.2, §3.1, §5) leem-se como `Pigment mix` (fábrica: 0) | o dono respondeu «Pigment mix por pincel = sim»; um seletor binário esconde que o meio-termo é contínuo e mensurável |
| R9 | ordem | (a) smoke do dono com a maquete e as perguntas; (b) EM PARALELO, as medições §4 (1)(3)(4)(5)(6) num só bloco; (c) implementação do painel depois das duas | §0.10: um bloco |

Perguntas que vão ao dono (num só smoke, com a maquete [`fotos/f3/maquete_painel.png`](fotos/f3/maquete_painel.png) e
[`fotos/e9/borda_fina.png`](fotos/e9/borda_fina.png)): §6 #1–#6 + a camada fina (E9 §7.3 #1: manter a mudança de cor na
borda fina, ou prendê-la à cor escolhida).

## §8 Smoke do dono (01/11) — a FORMA do §1/§5 foi reprovada; a lei fica

Memória: `project-memory/project_painter_panel_e10_owner_decisions_2026_11_01.md` (frases do dono e a captura do painel
de hoje). O que muda no desenho da E10 (vale sobre o §0–§7):

| # | o dono | o que a E10 passa a ser |
|---|---|---|
| D1 | as ferramentas não saem da barra vertical da esquerda | Blend/Smear/Blow/Dry/Water/Erase/Knife ficam onde estão; a força de cada uma continua onde está hoje |
| D2 | os 4 modos partilham o máximo de UI; Paper, Grain, Shape… ficam | nenhuma secção comum aos 4 modos sai nem muda de lugar; o §1 deixa de ser «o painel» e passa a ser **a lista dos controlos de LEI** que entram NAS secções de hoje |
| D3 | a UI segue o padrão do app | campos do Inspector (rótulo + número com setas + ponto de keyframe), secções colapsáveis de hoje; nada de widget próprio. A maquete `fotos/f3/maquete_painel*.png` fica só como registo do que foi recusado |
| D4 | não desfazer vitórias: Spacing «Continuous»; o Tilt e o visor dele | intocados (o §1.2 #9 «Spacing» funde só os knobs de passo POR BAIXO do «Continuous», sem mudar o controlo visível) |
| D5 | a configuração avançada é muito importante | **todo knob de hoje com leitor de produção FICA** (as linhas M/C/R do §2 deixam de apagar controlos vivos): só sai o que é MORTO medido (censo §5: `extRakeLight`, `extValleyGran`, `extWetSheen`, `extGrainDither`, `extEdgeTint` — sem leitor no produto) e o que é a MESMA grandeza que o dono aprovou fundir (Pigment + Water → Dilution). As fusões M que sobram viram «este knob escreve a grandeza X» no doc/tooltip, não remoção |
| Q2 | um só «Dilution» ✓ | Pigment + Water → `Dilution` (migração §3.1, exacta) |
| Q3 | Dry speed da tela inteira | `Evaporation` por tela (T); os 4 pincéis que o mudam migram para o mesmo valor da tela, com foto antes × depois |
| Q4 | escolha técnica | **Paper press = 1 por omissão** (o que os pincéis aprovados já fazem; medido: a diferença é ≤ 8 % e só no óleo fino, `fotos/f3/paper_press_0_meio_1.png`) |
| Q5 | meios ✓ | Fan Brush 0,5 · Express Oil 1 · Oil Pastel/Wax 1 |
| Q6 | quer o controlo da sombra | **Shadow** fica (força da sombra do relevo); Crevice fica no avançado |
| Q7 | riscas/cor por cerda → secção Shape | os `bristleStreak*`/`bristleNoise*` ficam na secção Shape (não num asset escondido) |
| Q8 | o que der menos artefactos | **a camada fina prende o matiz à cor escolhida** (a borda ciano/limão da `fotos/e9/borda_fina.png` some); o vidrado sobre tinta curada continua por banda (amarelo sobre azul seco = verde) |
| Q9 | Body fica | `Body` = tensão de cedência por pincel somada à do meio; substitui a parede binária `levelingOwnStroke` e cura o Bug #56 (a pilha de 12–39 µm) |

Os controlos NOVOS de lei (sem antecessor único) entram nas secções de hoje, no mesmo estilo: `Medium`, `Dilution`,
`Volume` (em µm, F3b §4: faixa 0..1000 µm medida), `Pigment mix` (E10b/E10c, já no motor), `Body`, `Hardness`,
`Absorbency` (tela), `Shadow`.
