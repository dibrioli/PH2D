# F4-2 — Plano executável: ligar a lei única (`ph2d-paint-tinta`) ao Wet Paint

> ADR-0197 §1, §1.1, §4, §5 · F2 §3 · F4-1 (`ph2d-paint-tinta`) · F1 censo (R1..R23, D1..D22) · F1 mapa §2–§3.
> 31/10/2026, `line/PainterWatercolor`. Leitor: a próxima LLM. Abreviaturas: `wp/` = `crates/ph2d-wet-paint/src/`,
> `gpu/` = `crates/ph2d-wet-gpu/src/`, `tp/` = `crates/ph2d-tool-painter/src/tool/paint/`, `tt/` = `crates/ph2d-paint-tinta/`.
> Nada aqui foi compilado (máquina a load ~90); todo `ficheiro:linha` foi lido na árvore de `0fce416f88`.

## 0. Decisões deste plano (resumo)

1. **Estado por célula = SOMAS ópticas agregadas, não paleta.** Por fatia (filme, aberto): volume `v` + `Σvᵢ Kᵢ` e
   `Σvᵢ Sᵢ` por banda. Exacto para Duncan (1) enquanto as transferências forem proporcionais e independentes do
   pigmento (são, hoje: §2.3). Sem limite de pigmentos, sem paleta, sem regra de transbordo. Custo §2.4.
2. **O interruptor** é um campo NOVO `Sim::tinta: LeiDaTinta { Velha, Pigmento }` em execução, fora do `Sim::lei`
   (que o preset reescreve em `tp/wetpaint/authored_actions.rs:64`); a sonda F0 varre `PH2D_SONDA_LEI=velha,pigmento`
   no laço mais interno (as duas leis intercaladas, mesmo processo). §5.
3. **A placa recusa a lei nova** até à paridade: `gpu/passo.rs:21 placa_sabe` devolve `false` com `Pigmento` — a CPU
   corre sozinha atrás do interruptor, sem divergência possível. §6.
4. **Receita × quantidade (regra do dono, ADR §1.1):** Pigment + Water → só φ (e `w`); o slider hoje rotulado
   Opacity (`sliders.pigmento`) passa a **Volume** = espessura `h` por dab; a pressão só toca `h`/`A_c`. Etapa E4.
5. **O ecrã sai da camada para o compositor RGBA** pela regra «exacto sobre branco»: §3.2.

## 1. O ESTADO hoje

### 1.1 CPU — `Grid` (`wp/grid.rs:50-181`)

| campo | linha | tipo | o que é | quem ESCREVE (produção) | quem LÊ |
|---|---|---|---|---|---|
| `film` | `:58` | `Vec<f32>` | água livre (= veículo `w`) | `trail.rs:566` (depósito `deposit*law.water_amount`), `trail/deposit.rs:409`, `solver/advect*.rs`, `drying.rs:146` (evapora), `tools/*` | solver, drying, render (não), `gpu/` |
| `susp` | `:60` | `Vec<f32>` | massa de pigmento em suspensão (≈ `vᶠ`) | `trail.rs:565` (`pig += deposit*gain`), `transfer.rs:266,347,352,406,621`, `solver/advect.rs:211`, `sedimenta.rs`, `cobre.rs:20`, `canvas_ops.rs:64`, `drying.rs` | render `:489`, `oleo_campo.rs:67/98/115/137` (altura), ladeira, `gpu/` |
| `susp_rgb` | `:64` | `Vec<[f32;3]>` | **cor** sRGB 0..255 do suspenso | `transfer.rs:338,351,416,619,658`, `solver/advect.rs:212` (`km_weighted_mean_color` `:186`), `advect_commit.rs:94`, `sedimenta.rs:162/197`, `remolha.rs:64`, `tools/smear.rs:234,268` | render `:494`, `drying.rs` (assenta), `bico.rs:99`, `perto.rs:66`, `cobre.rs:14`, `gpu/` |
| `sett` | `:66` | `Vec<f32>` | massa assentada (≈ `vᵃ` + o que hoje seria `R_g`) | `transfer.rs:24,38,222,265,359,408,642`, `canvas_ops.rs:63`, `cobre.rs:51`, `drying.rs` | render `:488`, `oleo_campo.rs`, `ladeira_do_oleo.rs:185`, `gpu/` |
| `sett_rgb` | `:68` | `Vec<[f32;3]>` | **cor** do assentado, resolvida por INVERSA (`assenta.rs:393 resolve`, `:439 assenta`) | `drying.rs` (`assenta_memo_f32`), `canvas_ops.rs:106,132,134`, `transfer.rs:23,36,419,640`, `tools/smear.rs:324` | render `:493`, `bico.rs:98`, `remolha.rs:59`, `oleo_vista_prova.rs` |
| `vela` | `:91` | `Vec<f32>` | Glaze por célula (R2b) | `transfer.rs` | render `:496`, `drying.rs`, `assenta/ponto.rs:121` |
| `aparencia` | `:97` | `Vec<AparenciaCelula>` | papel/grão/textura/**vidrado/opaca** (R2c, R7, R11) | `transfer.rs` | render `:497/499/529`, `drying.rs`, `gpu/passo.rs:37` |
| `oleo` | `:100` | `Vec<OleoCelula>` | fracção de óleo + Depth/Gloss/Solubility (R3, R12) | `grid/oleo_celula.rs:45-57` | render `:498`, `drying.rs`, `solver.rs`, `ladeira_do_oleo.rs`, `oleo_real.rs`, `tp/wetpaint/composite_oleo.rs:161` |
| `oleo_antes`, `oleo_antes_assente` | `:104,:107` | `Vec<f32>` | fotografia da ladeira (R15) | `ladeira_do_oleo.rs:172` | `ladeira_do_oleo.rs:66,160` |
| `relevo` | `:71` | `Vec<f32>` | Emboss pesado pela massa | `transfer.rs` | render `:516` |
| `altura`, `espessura`, `crista`, `nivelado` | `:75,78,81,84` | `Vec<f32>` (vazios até uso) | relevo dos fios, tecto do depósito, faca | `bristles_relevo.rs`, `grid/espessura.rs`, `lamina.rs` | render `:553`, `oleo_campo.rs`, `oleo_real.rs` |
| `material` | `:94` | `Vec<MaterialCelula>` | Principled + Transparency | `transfer.rs` | render `:750`, `oleo_real.rs` |
| `paper`, `wet`, `traco` | `:128,:126,:87` | | dente, molhado, traço | `paper.rs`, solver | render, solver, `gpu/advect.wgsl:281` |

O histórico copia tudo em `wp/grid/snapshot.rs:51 snapshot_grid` / `:87 restore_grid`, os ladrilhos em `wp/grid/patch.rs`,
e o instantâneo do traço em `wp/painter/instantaneo.rs`. Massa ≠ volume: `alpha_of_mass` (`wp/opacity.rs:66`, tabela
saturante até `OPACITY_TABLE_MAX = 3000`) faz massa → cobertura; `PigmentPerDab` de fábrica `600` (`wp/tuning/defs.rs:46`).
⚠️ 1 célula = 1 px no produto: `render.rs:343-345` escreve a linha `cy−1` do canvas para a linha de células `cy`.

### 1.2 GPU (`gpu/`)

| buffer | shader:linha | conteúdo hoje |
|---|---|---|
| `st` (vec2: film, susp), `rgb`, `dst`, `drgb` | `advect.wgsl:24-27` | estado entrelaçado + cor suspensa (só média LINEAR — sem K-M, `gpu/lib.rs:7-9`) |
| `sett` (read) | `advect.wgsl:279` | massa assentada |
| `film`, `susp`, `sett`, `srgb`, `trgb` | `seca.wgsl:16-20` | secagem; `srgb`/`trgb` = cor suspensa/assentada |
| `aux` (vec4: vela, vidrado, óleo, livre) | `seca.wgsl:22-23` | os modificadores R2b/R2c/R3 |
| `alfa` | `seca.wgsl:25` | tabela `alpha_of_mass` |
| CPU ↔ placa | `gpu/lib.rs:134 Estado {film,susp,rgb}`, `:116 pack_state`, `gpu/seca.rs:198 upload`, `:303 readback`, `gpu/passo.rs:429 sobe`, `:584 desce` | |

`seca.wgsl:81` porta `prepara(...).sobre_branco()` — a inversa do assentar também está na placa. A guarda
`gpu/passo.rs:21-41 placa_sabe` já exclui K-M, óleo, `Opaque` e aparência opaca.

## 2. O ESTADO alvo

### 2.1 Campos novos (todos `Vec`, VAZIOS até `Sim::tinta == Pigmento` pintar — quem não liga não paga, como `altura`)

| campo | tipo | grandeza (F2 §3.1) |
|---|---|---|
| `film` (fica) | `f32` | veículo `w` (µm) |
| `susp` (fica, unidade muda) | `f32` | `vᶠ = Σᵢ vᵢᶠ` (µm de pigmento) |
| `sett` (fica, unidade muda) | `f32` | `vᵃ = Σᵢ vᵢᵃ` |
| `filme_ks` | `[f32; 6]` | `Σᵢ vᵢᶠ Kᵢ` e `Σᵢ vᵢᶠ Sᵢ` por banda (3+3) |
| `aberto_ks` | `[f32; 6]` | `Σᵢ vᵢᵃ Kᵢ`, `Σᵢ vᵢᵃ Sᵢ` |
| `fechado` | `[f32; 3]` | `R_g` por banda (init `1,0` = branco de referência; §3.2) |
| `altura_fechada` | `f32` | volume que já curou (o relevo não pode sumir no fecho: `oleo_campo.rs:67` lê hoje `sett+susp`) |
| `meio` | `f32` | `m ∈ [0,1]`, médio pelo volume (substitui `oleo` fracção, `vidrado`, `opaca`, `vela`) |
| `abertura` | `f32` | `o ∈ [0,1]`, decai por `τ(m)`; `o → 0` ⇒ fecho |

Somem (E12): `susp_rgb`, `sett_rgb`, `vela`, `aparencia.{vidrado,opaca}`, `oleo` (fracção; Depth/Gloss vão para Luz),
`oleo_antes*`, `relevo` (o relevo é `h`).

### 2.2 Porquê somas e não «N pigmentos por célula» / paleta indexada

Por banda, (1) é LINEAR nos volumes: `𝒦 = Σ(vᶠᵢ+vᵃᵢ)Kᵢ = Kᶠ_Σ + Kᵃ_Σ`. Se toda transferência move a MESMA fracção α de
cada pigmento (advecção, depósito, pick-up `k_u·o·A_c`, assentar pela fracção de água perdida), mover α do filme é
mover α de `v`, de `Kᶠ_Σ` e de `Sᶠ_Σ`. Logo a soma é estado SUFICIENTE e exacto: o pigmento i não precisa de nome. É
o «pigmento latente» do ADR (Em aberto) levado ao limite.

| esquema | bytes/célula de pigmento | limite | regra extra |
|---|---|---|---|
| hoje (`susp,sett` + 2×rgb) | 32 | — | inversa de Newton |
| paleta global + K=4 slots `(u16 idx, vᶠ, vᵃ)` + `R_g` + `m` + `o` + `altura_fechada` | 8 + 48 + 12 + 12 = 80 | 4 pigmentos | transbordo: (a) slot livre; (b) fechar o aberto da célula; (c) fundir o par mais próximo por Duncan (exacto sob transferências proporcionais) |
| **somas** (escolhido) | 8 + 48 + 12 + 12 = **80** | **nenhum** | nenhuma |

Mesmo custo, sem tecto, sem transbordo, sem tabela a crescer, e o GPU lê vec4 alinhados. **Quando a escolha muda:** se
uma lei futura tornar uma transferência dependente do pigmento (granulação por pigmento, `Staining` por pigmento), a
soma deixa de ser suficiente ⇒ slots K=4 com a regra de transbordo acima. Hoje `ExtGranulation`
(`wp/drying.rs` dentro de `dry_cell`, viés pelo papel) e `ExtStaining` (`drying.rs:26`) são por PINCEL — independentes do
pigmento.

**A regra do fecho** (pedida no briefing) com somas: quando `o < ε` (E8), `R_g ← camada(Optica{Kᵃ_Σ, s·Sᵃ_Σ}, R_g)`,
`altura_fechada += vᵃ`, `aberto_ks ← 0`, `vᵃ ← 0` (= `tt/src/transfere.rs:71 fecha`, que é exacto: o ecrã do aberto
sobre `R_g` é o que passa a `R_g`). Com o filme NÃO vazio o fecho muda a ordem das camadas (filme passa a camada sobre
`R_g` em vez de mistura com o aberto): é o evento físico «o de baixo curou», limitado pelo teorema (o `R_g` novo está
no envelope). Num esquema de slots o transbordo usaria o MESMO fecho como passo (b).

### 2.3 Unidades e calibração

`susp`/`sett` passam de massa (`~600` por dab de fábrica) a µm. Uma função escrita `volume_de_massa(m) = m / M_UM`
com `M_UM` calibrado para que a cobertura derivada (3) a pressão 1 e o pigmento de fábrica dê a mesma cobertura que
`alpha_of_mass(600)` — medido, com a tabela no doc da E4. `Pigmento::de_cor(srgb, transparência)` (`tt/src/pigmento.rs:51`)
com a Transparency do pincel (`render.rs:750 transparencia_da_celula` hoje é composição; passa a S do pigmento).

### 2.4 Memória a 4096² (16,78 M células)

| | B/célula | total |
|---|---|---|
| pigmento hoje (`susp,sett,susp_rgb,sett_rgb`) | 32 | 537 MB |
| alvo (somas, §2.1, sem `film`) | 80 | 1,34 GB |
| líquido depois da E12 | +48 | +805 MB |
| durante o interruptor | só quem liga paga (campos vazios) | |

Tecto: **não se escreve** antes de medir RSS no traço cheio a 4096² nas duas leis (CLAUDE §0). Alavancas medidas
depois, se o RSS morder: `R_g` em `u16` (passo `1,5e-5` ≪ 8 bits; quebra o «fecho ao bit» de `tt` — medir), ou
campos por ladrilho vivo (`grid/patch.rs`).

## 3. O ECRÃ

### 3.1 Onde se calcula hoje

- `wp/render.rs:435 pigment_row` (chamado por `:362 render_pigment_row_visual`, `:378 _oleo`, `:407 _oleo_com_sombra`,
  `:722 _oleo_pre`; o produto: `tp/wetpaint/composite.rs:263,279`). Por camada: `a_s/a_f = alpha_of_mass` (`:488-489`),
  `Ecra` encadeado `com_vela·com_vidrado·com_oleo·com_opaca` (`:495-499`), `assenta::cor_e_alfa` (`:500`),
  `na_caixa_das_tintas` (`:507`, R8), papel+emboss somados à cor (`:521-568`), Transparency de composição (`:572`),
  Porter-Duff pré-multiplicado entre camadas (`:574-578`), Óleo Real (`:593-636`) ou RGBA plano (`:643-646`).
- A lei: `wp/assenta.rs:39 Ecra`, `:217 cor_e_alfa`, `:242 cor_e_alfa_tapando`, `:268 na_caixa_das_tintas`,
  `assenta/ponto.rs` (R6/R7, `mistura_opaca :144`). O assentado precisa da INVERSA: `assenta.rs:393 resolve` (Newton,
  preso a `[0,255]`), chamada por `:439 assenta` ← `drying.rs` (`assenta_memo_f32`), `grid/canvas_ops.rs:106`, e `:617 levanta`
  ← `drying.rs:57`. A orla do óleo: `wp/oleo_borda.rs:20` (`cor_e_alfa`).

### 3.2 Por onde entra a lei (2) — `render.rs:485-578` com `Pigmento`

Por camada `lg`, célula `i`:
```
o   = Optica { k: Kᶠ_Σ+Kᵃ_Σ, s: s_fator(m, molhado)·(Sᶠ_Σ+Sᵃ_Σ) }     // tt lei.rs:31,22 ; (1)
R_w = camada(&o, &R_g)                                                // tt lei.rs:94 ; (2), linear por banda
C   = cobertura(&o)                                                   // tt lei.rs:109 ; (3)
α   = max( max_b(1 − R_w,b), média_b(C_b) )                           // α ≥ 1−R_w ⇒ cor ∈ [0,1]
cor = 1 − (1 − R_w)/α   (α > 0)                                       // «exacto sobre branco»
```
Saída para o compositor RGBA: `(srgb(cor), α)`. Sobre branco `cor·α + 1·(1−α) = R_w` **ao bit** (a régua plana da F0
compõe sobre creme: medir também `R_w` cru da grelha). Sobre outra camada é Porter-Duff (o meio de entre-camadas não é
Kubelka — fora do âmbito: o ADR fala da célula). Somem para `Pigmento`: `:495-514` (Ecra/caixa), `:572` (Transparency de
composição: a transparência é o `S` do pigmento). Ficam: o papel/emboss `:521-568` (área Superfície, F4 seguinte) e a
mistura entre camadas `:574-578`. `molhado` = `film[i] > 0` (o `Ecra::seco()` de `:463` some: o «molhado escurece» é
`s_fator(m, true)`, física, não knob — R22 `TINTA_MOLHADA_ESCURECE`).

Teste da etapa: **`o_ecra_da_celula_e_a_lei_da_tinta_ao_bit`** (célula com somas conhecidas → pixel = `tt::camada`), e
**`sobre_branco_a_composicao_devolve_r_w`**.

### 3.3 Óleo Real

`wp/oleo_luz.rs:155 ilumina(ol, campo, cx, cy, rgb, tinta)` e `:181 ilumina_e_sombra` recebem o albedo `rgb` e a
cobertura `tinta`; a altura vem do `CampoAltura` (`wp/oleo_campo.rs:8 de`, lê `sett+susp` em `:67,98,115,137`). Com
`Pigmento`: `rgb = R_w` (sRGB 0..255), `tinta = α`, e a altura = `h(v) = (vᶠ+vᵃ)/φ_célula + altura_fechada` (volume de
tinta, não de pigmento) — `oleo_campo.rs` muda a função de altura num único ponto. `PreLuz::fecha_oleo`
(`render.rs:677`) e o `w_oleo` (`:483`, R21: dois pipelines de cor) passam a ter `w = 1` (há UM pipeline: a luz aplica-se
a toda a tinta com a rugosidade/material da célula; uma aguada sem relevo tem `h ≈ 0` ⇒ normal plana ⇒ a luz é a
identidade sobre o albedo). Confirmar que a identidade vale ao bit é parte da E5.

## 4. As TRANSFERÊNCIAS (o que move em volumes; onde morre cada ramo)

Regra única: **mover a fracção α de uma fatia move α de `v`, de `K_Σ` e de `S_Σ`** (função `tt` `transfere_soma`, E1).

| transferência | hoje | com `Pigmento` |
|---|---|---|
| depósito (pincel → filme) | `trail.rs:565-566` (`pig += deposit*gain`, `water += deposit*water_amount`), `trail/deposit.rs:123,298,406-409` (`pousa_a_tinta`), `transfer.rs:20 pousa_seco`, `transfer.rs:317-352` (mistura da ponta, `mix.mix`) | `Δvᶠ = φ·h·A`, `Δw = (1−φ)(1−m)·h·A`, `ΔKᶠ_Σ = Δvᶠ·K_pincel` (E4/E6a). Morrem: `mix.mix` (RGB/K-M: R9), `cobre.rs` (R19) |
| pick-up (filme/aberto → pincel) | `trail/bico.rs:21-269` (lê `sett_rgb/susp_rgb` `:98-99`), `pickup_assentado` (sim.rs) | `tt::pick_up` `k_u·o·A_c·v` de cada fatia (`o = 1` no filme); o bico leva uma `Soma` (E6b). D3 colapsa |
| re-molhar | `transfer/remolha.rs:32` (`sett_rgb → susp_rgb`), `drying.rs:41 lift_settled` → `assenta::levanta :617` | aberto → filme pela solubilidade × `o`: `transfere_soma(aberto, filme, f)` — sem inversa (R16 morre) |
| arrasto / smear / blend | `tools/smear.rs:101-324`, `transfer.rs:471 transfer_blend` (`:518-660`), `tools/arrasto_agua.rs`, `tools/secador.rs`, `tools/bordas.rs`, `perto.rs:66` | média de SOMAS (extensivas) — o `km_weighted_mean_color` vira soma (E6b) |
| escoamento | `solver/advect.rs:164-212` (`km_weighted_mean_color` `:186`), `advect_commit.rs:94`, `advect_jacobi.rs`, `flow_jacobi/backrun.rs`, `sedimenta.rs:162,197`, `estreito.rs`, `trail/molhado.rs` | transporte das somas como o de `susp` (E7). `km_mixing`/`ColorMix` (R9) morre |
| ladeira do óleo | `solver/ladeira_do_oleo.rs:14-25` (8 rotas, `AtomicU8`), `:33 altura`, `:172 fotografa`; `solver.rs:104`, `flow_jacobi.rs:126,192`, `painter/doors.rs:85` (R14, R15) | UMA altura `h = w + vᶠ/φ + vᵃ/φ + altura_fechada`; o freio pela tensão de cedência `τ_y(m)` (Orchard). A rota fica só para `Velha` até E12 |
| assentar | `drying.rs:146 dry_cell` → `assenta_memo_f32` (inversa), `canvas_ops.rs:59-134` | `transfere_soma(filme, aberto, lost)` — o ecrã não muda POR CONSTRUÇÃO (R2, R3–R7, R16 morrem) |
| abrir/curar | não existe (o óleo «cura» por `OLEO_PLENO`, R5) | `o ← o·e^{−Δt/τ(m)}`; `o < ε` ⇒ `fecha` (E8) |
| evaporar | `drying.rs` `new_film = f·retention − evap…` | `Δw = −E(1−m)·Δt` (Retention: D5) |

### 4.1 Onde morre cada um dos 23 ramos (F1 censo §3)

| R | morre em | como |
|---|---|---|
| R1 `Lei` | E12 | `Lei`/`Ecra` apagados; `remolha.rs:68`, `grid/traco.rs:77`, `oleo_borda.rs:21` deixam de os pedir em E6b/E5 |
| R2, R2a, R2b, R2c | E5 (ecrã) + E8 (assentar) | (1)–(3); `vela`/`vidrado` = transparência do pigmento |
| R3, R4, R5, R6 | E5 + E8 | `PINCEL_DE_OLEO`, `OLEO_PLENO`, tapar extra: o óleo é `m` alto |
| R7 `com_opaca` | E5 | a opacidade é `S` alto (Pigmento::de_cor com transparência 0) |
| R8 caixa das tintas | E5 | Lema 1 garante o envelope; o clamp some |
| R9 `ColorMix` | E6a/E7 | somas não têm «modo de mistura» |
| R10 velatura do Live Water | E2 | `glaze_filme` ignorado com `Pigmento` (`session.rs:279-283`) |
| R11 caixas → fracções | E4 | `m` por célula (médio pelo volume) |
| R12 `oleo_celula::do_pincel` | E4 (fracção) · Luz (Depth/Gloss) | |
| R13 espalha no 1.º toque | E6a | consequência de `o = 1` + pick-up; o `if p.oleo > 0` (`transfer.rs:174`) some |
| R14, R15 ladeira | E7 | uma altura, `τ_y` |
| R16 `!= Porto` | E8 | sem ramo |
| R17 `StrokeFilm` | E4 | a água do filme é `(1−φ)(1−m)h` — não é modo |
| R18 faca | fica (ferramenta) | a faca move somas (E6b) |
| R19 `WetCover/DryCover` | E6a | cobertura derivada |
| R20 `PorCima` por omissão | E5 | |
| R21 dois pipelines óleo/sem | E5 (§3.3) | |
| R22 constantes Rebelle | E12 | `colorops.rs:101-192` apagadas |
| R23 sentinela `-0.01` | E10 (controlos) | knob → bool explícito ou sai |

## 5. A ESCOLHA EM EXECUÇÃO

- **Onde:** `wp/sim.rs` — `pub enum LeiDaTinta { Velha, Pigmento }` e `Sim::tinta` (ao lado de `lei`, `:139`), copiado
  para `Params::tinta` em `gather_params` (`:325-365`). Lido por: `render.rs:462` (escolhe o corpo da célula), `drying.rs:146`,
  `trail.rs`/`deposit.rs`, `transfer.rs`, `solver/advect.rs`, `ladeira_do_oleo.rs:160`. **Um `match` por porta**, nunca
  espalhado em predicados (lição R1–R23).
- ⛔ NÃO no `Sim::lei`: `tp/wetpaint/authored_actions.rs:64` fixa-o a `Cobre` em cada aplicação de preset — a varredura
  aplica o preset a CADA passada (`sonda_varredura_oleo.rs:308`), e apagaria a escolha.
- **Ferramenta:** `PainterTool::set_wet_lei_da_tinta(LeiDaTinta)` em `tp/wetpaint.rs` (vizinho de `:261
  set_wet_relogio_fixo`), que escreve em `engine.sim.tinta` e sobrevive a `wet_preset_aplicar`.
- **Sonda F0** (`tp/wetpaint_preset/sonda_varredura_oleo.rs`): `celula(...)` (`:287`) ganha `lei: LeiDaTinta` e chama o
  setter logo depois de `set_wet_relogio_fixo` (`:302`); `sonda_varredura_oleo` (`:403`) lê
  `lista("PH2D_SONDA_LEI", &[Velha, Pigmento])` e põe-na no laço MAIS INTERNO (depois de `n`, `:449`) — cada célula
  corre as duas leis seguidas, mesmo processo, mesma compilação `smoke`; o TSV ganha a coluna `lei` e o PNG o sufixo.
  `FromStr` para `LeiDaTinta` (`velha|pigmento`).
- **A placa:** `gpu/passo.rs:21 placa_sabe` ganha `&& p.tinta == LeiDaTinta::Velha` até a E11 — a CPU é a canónica.
- Default do produto: `Velha` até o portão-mãe passar INTEIRO; depois a E12 apaga `Velha` no mesmo bloco (ADR §5).

## 6. GPU

Ordem: CPU primeiro (E3–E9) atrás do interruptor, com `placa_sabe == false` para `Pigmento`; GPU depois (E11), lei
congelada pelo portão-mãe.

| shader / ficheiro | muda |
|---|---|
| `advect.wgsl:24-27` (`st`, `rgb`, `dst`, `drgb`) | `rgb/drgb` (3 f32) → `ks/dks` (vec4×2 por fatia: `K_Σ` 3 + `S_Σ` 3 + `v`… alinhar a 2×vec4); o transporte é o MESMO de `susp` (linear) — mais simples que hoje |
| `advect.wgsl:279` `sett` | + `aberto_ks` só leitura (altura) |
| `seca.wgsl:16-27` | `srgb/trgb` → `filme_ks/aberto_ks`; `aux` (vela, vidrado, óleo) → (`m`, `o`, `altura_fechada`, livre); `fechado` vec4; `alfa` (tabela) sai; a porta da inversa (`seca.wgsl:81`) sai — assentar vira `transfere` (3 mul por banda) |
| ecrã | hoje na CPU (`render.rs`); se migrar à placa, portar `tt/src/lei.rs:71 camada_banda` ordem a ordem, `expm1f` do `libm 0.2.16` linha a linha, sem contracção FMA (F4-1 §1 «Determinismo») |
| `gpu/lib.rs:134 Estado`, `:116 pack_state`, `gpu/seca.rs:198/228/303`, `gpu/passo.rs:429/446/476/584` | sobe/desce os campos novos |
| `gpu/passo.rs:21 placa_sabe` | aceita `Pigmento` só depois de `paridade_da_tinta_por_placa` verde |

Paridade: mesmo bit por placa (ADR §4.3) — o transporte de somas é linear (f32, mesma ordem), o fecho usa `camada_banda`
(a única transcendental). Teste: `gpu/tests` **`paridade_da_tinta_por_placa`** (cena `gpu/cena.rs:8 poca` com somas).

## 7. Etapas E1..E13

`∥` = pode correr em paralelo com as marcadas (ficheiros disjuntos). Cada etapa compila sozinha e não muda `Velha`
(prova: o fingerprint de `Velha` igual antes/depois — `sim_resumable_tests.rs`/`flow_bit_tests.rs` existentes, por nome).

| E | objetivo | ficheiros | teste por NOME que prova | risco | depende |
|---|---|---|---|---|---|
| **E1** ∥E2 | `Soma<N>{v,k,s}`, `transfere_soma`, `optica_de_somas`, `fecha_somas`, `ecra_sobre_branco` (§3.2) na crate tinta | `tt/src/soma.rs` (novo), `tt/src/lib.rs`, `tt/tests/it/soma.rs` | `somas_e_paleta_dao_a_mesma_optica`, `transfere_soma_conserva`, `sobre_branco_a_composicao_devolve_r_w`, `alfa_deixa_a_cor_em_0_1` | baixo | — |
| **E2** ∥E1 | interruptor `LeiDaTinta` + setter + sonda com `PH2D_SONDA_LEI` + `placa_sabe` | `wp/sim.rs`, `tp/wetpaint.rs`, `tp/wetpaint_preset/sonda_varredura_oleo.rs`, `gpu/passo.rs` | `o_preset_nao_apaga_a_lei_da_tinta`, `a_placa_recusa_a_lei_pigmento` | baixo (o preset reescrever) | — |
| **E3** | campos novos vazios + `garante` + histórico/ladrilhos/instantâneo | `wp/grid.rs`, `wp/grid/tinta.rs` (novo), `wp/grid/snapshot.rs:51,87`, `wp/grid/patch.rs`, `wp/painter/instantaneo.rs`, `wp/grid/canvas_ops.rs` (cópias) | `o_historico_leva_a_tinta_nova`, `a_lei_velha_nao_aloca_a_tinta` | médio: snapshot é «formato gravado»? (só em memória — conferir `ph2d-app-painter` persistência do grid) | E1, E2 |
| **E4** | **receita × quantidade** (ADR §1.1, regra do dono) — §7.1 | `wp/painter/receita.rs` (novo), `wp/painter/doors.rs:12-55,171-178`, `wp/painter.rs:287`, `wp/painter/po.rs:28-32`, `wp/painter/mascara.rs:48-51`, `wp/sim.rs:86`, `wp/trail/deposit.rs:121-140`, `wp/trail.rs:523,565-566,614`; `m` por célula (R11/R12) | `volume_fixo_varrer_pigmento_e_agua_nao_muda_o_volume_depositado`, `pigmento_e_agua_fixos_varrer_volume_nao_muda_a_concentracao`, `a_pressao_so_toca_a_quantidade` | ALTO: muda o sentido de 2 sliders; a calibração `M_UM` (§2.3) | E3 |
| **E5** ∥E6b,E7 | ecrã com `Pigmento` (§3.2) + Óleo Real (§3.3) + orla | `wp/render.rs:435-657`, `wp/oleo_borda.rs:20`, `wp/oleo_campo.rs` (altura) | `o_ecra_da_celula_e_a_lei_da_tinta_ao_bit`, `celula_vazia_e_transparente`, `oleo_real_plano_e_a_identidade` | médio: emboss `:521-568` ainda soma cinza à cor | E1, E3 |
| **E6a** | depósito no filme por somas; morrem `mix.mix`, `cobre.rs`, `transfer.rs:174` | `wp/trail/transfer.rs:20-420`, `wp/trail/transfer/cobre.rs`, `wp/trail/deposit.rs:298-410`, `wp/trail/filme.rs` | `o_deposito_conserva_volume_e_somas`, `n_passadas_monotonas_no_motor` | ALTO (o maior ficheiro) | E4 |
| **E6b** ∥E5,E7 | pick-up/bico, re-molhar, smear/blend/arrasto/secador/bordas, faca | `wp/trail/bico.rs`, `wp/trail/transfer/remolha.rs`, `wp/trail/transfer/perto.rs`, `wp/trail/transfer.rs:471-660` (⚠️ mesmo ficheiro que E6a ⇒ E6b depois de E6a, OU E6a mexe só `:20-420`), `wp/tools.rs`, `wp/tools/{smear,arrasto_agua,secador,bordas}.rs`, `wp/lamina.rs` | `o_pick_up_leva_o_aberto_pela_abertura`, `o_smear_conserva_somas` | médio | E3 (E6a se o ficheiro for partilhado) |
| **E7** ∥E5,E6b | escoamento + ladeira única | `wp/solver/advect.rs:150-215`, `advect_commit.rs`, `advect_jacobi.rs`, `advect_fundida.rs`, `flow_jacobi/backrun.rs`, `sedimenta.rs`, `estreito.rs`, `trail/molhado.rs`, `solver/ladeira_do_oleo.rs`, `wp/solver.rs:104` | `a_advecao_conserva_somas`, `a_ladeira_e_uma_altura` + `advect_bit_tests` (Velha ao bit) | médio | E3 |
| **E8** | tempo: evaporar, assentar = `transfere`, abertura `o(t)`, fecho | `wp/drying.rs:41-560`, `wp/grid/canvas_ops.rs:55-135`, `wp/sim.rs` (`τ(m)`) | `assentar_nao_muda_o_ecra_ao_bit_no_motor`, `fechar_e_o_que_se_via`, `a_secagem_conserva` | médio | E5, E6a |
| **E9** | portão-mãe F0 sobre o motor nas duas leis (§4.1 do ADR) + ortogonalidade no motor | `tp/wetpaint_preset/sonda_varredura_oleo.rs` (+ um teste não-ignorado de grelha pequena) | `portao_mae_f0_lei_pigmento` (manchas 0, ferrugem 0, degraus 0, furos 0 condicionados), `portao_ortogonalidade_no_motor` | a varredura inteira é pesada → `scripts/fila-pesada.sh`; UMA corrida intercalada | E4–E8 |
| **E10** | controlos → grandezas (tabela §7.2 → `F3_controlos.md`), preset → lei nova por FUNÇÃO, meios como dados | `tp/wetpaint_settings*.rs`, `tp/wetpaint_preset/campos.rs`, `wp/tuning/defs.rs` (registo: `motivo:` crate inteira), `ph2d-brush-presets` | `cada_controlo_mexe_uma_grandeza` (jacobiano diagonal, ADR §1.1), `cada_preset_de_fabrica_mapeia_por_funcao` | ALTO: miniaturas (golden) | E9 |
| **E11** | GPU com `Pigmento` (§6) | `gpu/*.wgsl`, `gpu/lib.rs`, `gpu/seca.rs`, `gpu/passo.rs` | `paridade_da_tinta_por_placa` | médio (FMA no compilador de shaders) | E9 |
| **E12** | a lei velha SAI no mesmo bloco | `wp/assenta.rs`, `wp/assenta/`, `wp/colorops.rs:101-410`, `wp/opacity.rs` (como cobertura), campos §2.1 «somem», `ladeira` rotas, `session.rs:258-283`, `authored_actions.rs:59-65`, `gpu/` caminhos velhos | toda a bateria por nome do diff + `censo` dos controlos | ALTO (largura) — partir em E12a `wp` / E12b `tp`+`gpu` | E9, E10, E11 |
| **E13** ∥E12b | portões Rebelle reescritos/fora (§8) | os ficheiros de §8 | os nomes novos de §8 | médio | E9 |

Paralelos: {E1, E2} · {E5, E6b, E7} depois de E3 (+E4 para E6a) · {E12b, E13}. Série obrigatória: E3 → E4 → E6a → E8 → E9
(cada uma muda o estado que a seguinte lê).

### 7.1 E4 — cada multiplicação que hoje mistura receita e quantidade

| ficheiro:linha | hoje | erro | com `Pigmento` |
|---|---|---|---|
| `wp/painter/doors.rs:21` | `pressure_base = 0.2 + 0.7·s.pressure` | | pressão → `A_c(p)` e `h(p)` só |
| `wp/painter/doors.rs:22-24` | `intensity = b·0.5·pressure_base`, `.clamp(0,3)`, `*= Knob::Intensity` | Intensity (D1) multiplica pigmento E água (`trail.rs:614` → `:565-566`) | `Intensity` sai (= 1); a quantidade é `Volume` |
| `wp/painter/doors.rs:25-29` | `water_amount = 2·(s.water·0.35)²/(pressure_base+0.01)·WaterPerDab` | água = receita × pressão INVERTIDA (D4) | `Δw = (1−φ)(1−m)·h·A` |
| `wp/sim.rs:86-91` `com_pigmento` ← `painter.rs:287`, `doors.rs:174`, `doors.rs:467`, `po.rs:28`, `mascara.rs:48` | `PigmentPerDab *= sliders.pigmento` (Opacity) | Opacity (quantidade) entra na receita | `sliders.pigmento` → `h` (Volume); `PigmentPerDab` → φ com Water |
| `wp/painter/doors.rs:178`, `po.rs:32` | `water_amount *= agua_do_dab` | | entra em φ se é receita, em `h` se é quantidade — decidir por grandeza na E4 |
| `wp/painter/mascara.rs:51` | `water_amount *= peso.recip()` | a máscara muda a receita | a máscara muda `A` |
| `wp/trail/deposit.rs:123` | `gain = PigmentPerDab` | | `φ` |
| `wp/trail.rs:614` | `stamp = fall·texv·dab.intensity` | | `stamp = fall·texv` (forma) × `h` |
| `wp/trail.rs:565-566`, `deposit.rs:406-409` | `pig += deposit·gain`, `water += deposit·water_amount` | os dois escalam com a mesma `deposit` E com knobs próprios | `Δv = φ·h·stamp`, `Δw = (1−φ)(1−m)·h·stamp` |
| `tp/mao.rs:255-260` | `PressureOpacity` → `strength_min` (força do dab) | | curva `h(p)` |
| `tp/wetpaint_settings.rs:569-570`, `tp/wetpaint/session.rs:228,234` | `water`, `PigmentPerDab`, `sliders.pigmento = opacidade` | | `Water`, `Pigment` → receita; `Opacity` → `Volume` (rótulo/i18n na E10) |

Receita: `φ = P/(P + W·k_w)` com `P` = Pigment (0..1 depois da E10), `W` = Water, `k_w` calibrado; `m` vem do meio do
preset. Portão (b) do dono = os dois primeiros testes da E4, no motor (`Engine` direto, sem ferramenta), e repetidos na
E9 sobre a ferramenta.

### 7.2 As 22 famílias → o controlo que fica e a sua grandeza (proposta; a decisão final vive em `F3_controlos.md`, E10)

| D | fica | grandeza (unidade) | saem / viram consequência |
|---|---|---|---|
| D1 massa por dab | **Volume** (ex-Opacity) | espessura `h` por dab (µm) | `PigmentPerDab`→φ (D4/receita), `Intensity` sai, `PressureOpacity`→curva `h(p)` |
| D4 água por dab | **Pigment** + **Water** | receita φ (adim.) e veículo | `WaterPerDab`, `SmearWater` |
| D2 arrasto | **Drag** | fracção de `vᶠ` arrastada por px | `Smear` (é ferramenta), `RewetDrag`, `WaterDrag` |
| D3 recolha | **Pickup** | `k_u` (fracção por contacto) | `TipClean`, `WetLift` (= `k_u·o`, `o=1`); `TipReservoir` = capacidade do pincel (outra grandeza, área Pincel) |
| D5 secagem | **Dry speed** | `E` (µm/s), × `(1−m)` | `BaseEvaporation`, `Retention`, `DryerLevel/Edge`; Dryer e Fast dry são ferramentas |
| D6 absorção | **Absorbency** | coef. de Washburn do papel | `PaperGate`, `GateSaturation`, `Capillary`, `CapillaryGate` |
| D7 passo | **Spacing** | fracção do diâmetro | `CapsuleSpacing`, `PigmentSpacing`, `CapsuleMaxPx` |
| D8 re-dissolver | **Re-wet** | taxa de reabertura `do/dt` com água | `Rewet` e `Dissolve` (mesmo rótulo), `Solubility` funde |
| D14 aderência | (= D8) | inverso da reabertura | `ExtStaining` |
| D9 orla | **Edge darkening** | fluxo capilar para a orla (Curtis 97) | `EdgeSoftMass`, Heavy Edge (= gravidade + sedimentação) |
| D10 nivelamento | **Viscosity** · **Yield** | `η(m,φ)` · `τ_y(m)` (duas grandezas) | `Leveling`, `LevelClamp`, `Brake`, `BrakeReach`, `LevelingOwnStroke`, `DryerLevel`; `KnifeLevel` = faca |
| D11 borracha | **Erase** | fracção removida | `Eraser` |
| D12 papel visto | **Granulation** (pigmento) · **Paper press** (pincel; memória do dono: nunca apagar) | assentar nos vales · deformação do relevo | `PaperVisibility`, `VisualGrain`, Paint Texture, `PaperRelief` (o papel vê-se pela luz e pelo `R_g`) |
| D13 altura do relevo | (= Volume) | `h` | `Emboss`, `OilDepth`, `BristleRemove`; `BristleHeight` = variação de `h` por cerda (Pincel); `KnifeRidge` = faca |
| D15 pincel seco | — | consequência de `w` e `h` | `ExtDryBrush`, `dry_gate` |
| D16 velatura | **Transparency** (do pigmento) | `S` do pigmento | `Glaze`, Glaze layering, `glaze`, `glaze_filme` |
| D21 cobertura | — (DERIVADA, (3)) | `C` | `esconde`, `Opaque`, `WetCover`, `DryCover`, `PINCEL_DE_OLEO`, `OLEO_PLENO` |
| D17 estabilização | **Stabilizer** | px de atraso | `PulledString` |
| D18 escala | — | leis em unidades físicas | `EmbossSizeExp/Ref`, `TextureSizeExp*`, `BristleLaneSizeExp` |
| D19 abrir molhado alheio | — | consequência de `o` e pick-up | `WetOpen*` |
| D20 material | **8 Principled** por pincel→célula (Luz) | | as 8 linhas globais do cartão Real Oil |
| D22 brilho | **Roughness** · **Coat** | rugosidade · verniz (duas grandezas) | `OilGloss`, `CoatRoughness` (= Roughness do verniz) |

## 8. Portões do Rebelle

Censo (grep `-i rebelle` em ficheiros de teste das crates do Painter, 31/10): **75 ficheiros de teste** mencionam o
Rebelle; **13 testes** têm «rebelle» no NOME (+ 2 ajudantes: `tests/it/as_leis_da_escala.rs:277 rebelle()`,
`wp/tests/it/parallel_rows.rs:376 puddle_do_rebelle`). Classe: **C** = mede cor/cobertura/mistura (a lei muda ⇒
reescrever contra a física: as propriedades do ADR §4.1) · **F** = forma/marca/grão/escala (não toca a lei da tinta;
área Pincel — sai como «semelhança» na E13 ou reescreve-se como propriedade: monotonia largura×pressão, invariância à
escala) · **N** = só o nome (desempenho/cena) ⇒ renomear.

| teste | mede | classe | proposta |
|---|---|---|---|
| `wp/assenta/tests.rs:88 na_lei_do_rebelle_assentar_preserva_o_ecra_seco` | assentar não muda o ecrã | C | → `assentar_nao_muda_o_ecra_ao_bit_no_motor` (E8); sai na E12 com `assenta.rs` |
| `wp/assenta/tests.rs:392 sonda_o_custo_da_lei_do_rebelle` | custo do Newton | N | sai com `resolve` (E12) |
| `wp/tests/it/a_lei_do_glaze.rs:25 a_fraccao_da_camada_e_a_medida_no_rebelle` | velatura calibrada no Rebelle | C | sai (D16): velatura = `S` do pigmento; propriedade: «transparente sobre escuro aproxima `R_g`» (Lema 3) |
| `tp/wetpaint/mistura_tests.rs:286 azul_sobre_vermelho_vela_no_seco_e_sangra_no_molhado_como_no_rebelle` | seco filtra, molhado mistura | C | reescrever: seco = camada sobre `R_g` (fechado/aberto), molhado = Duncan no filme; tolerâncias pelo teorema |
| `tp/wetpaint/mistura_tests.rs:304 com_a_live_water_a_mistura_continua_na_barra_do_rebelle` | Live Water não muda a cor | C | reescrever: «o agendamento não muda a lei» (R10) ⇒ mesmo `R_w` com/sem Live Water |
| `tp/wetpaint/opcoes_tests.rs:85 a_heavy_edge_a_meio_pesa_na_orla_de_baixo_como_no_rebelle` | sedimentação pela gravidade | C/F | reescrever como conservação + sentido da gravidade |
| `tp/efeito_modos_tests.rs:201 o_efeito_a_dez_passagens_e_o_do_rebelle` | 10 passadas | C | → monotonia em N (sem degraus), do portão-mãe |
| `tp/tests/it/familias/mistura.rs:688 cada_preset_da_agua_cai_na_foto_do_seu_rebelle` | semelhança à foto | C | sai; a foto vira golden NOSSA (ADR §4.5) |
| `tp/tests/it/a_caneta_deitada.rs:67 o_lapis_deitado_alarga_e_alisa_como_o_do_rebelle` | largura × tilt | F | propriedade: largura monótona no tilt |
| `tp/tests/it/o_grao_do_lapis.rs:192 o_traco_fino_do_lapis_tem_o_grao_do_rebelle` | grão | F | sai (semelhança) ou propriedade do grão ∝ papel |
| `tp/tests/it/familias/grafite.rs:306 os_grafites_sao_uma_familia_com_a_borda_do_rebelle` | borda | F | propriedade de família (ordem das bordas) |
| `tp/tests/it/familias/grafite.rs:375 o_tecto_do_preset_pinta_o_grafite_ao_tamanho_do_rebelle` | tamanho | F | invariância à escala |
| `wp/tests/it/measure_stage_cost.rs:141 measure_where_the_rebelle_drying_starts_paying_in_parallel` | desempenho | N | renomear |

Ficheiros C com fixtures/réguas do Rebelle no CORPO (não no nome), a reescrever na E13 por propriedade ou sair:
`tp/tests/it/familias/{mistura.rs, mistura/motor.rs, mistura/arrasto_agua.rs, mistura/sonda_agua.rs, aguarela.rs,
aguarela/molhado.rs, aguarela/portoes.rs, aguarela/lamina.rs, guache.rs, oleo_acrilica.rs, tintas.rs}`,
`tp/tests/it/{a_lei_da_tinta.rs, leis_sistemicas.rs, midias_de_fabrica.rs, receitas_de_fabrica.rs, presets_de_fabrica.rs,
presets_de_fabrica/sondas.rs, exame_bancada.rs, regua_midia.rs, os_outros_meios_nao_mudam.rs}`,
`tp/wetpaint/{mistura_camadas_tests, mistura_wet_drip_tests, agua_sobre_oleo_tests, achatamento_tests, gota_tests,
pastel_seco_tests, tampa_tests, lamina_tests, esbate_tests, grid_ratio_tests}.rs`, `wp/tests/it/{o_misturador_so_leva_a_tinta_perto,
a_borda_graduada, a_lei_do_po}.rs`, `wp/{oleo_real_tests, oleo_real_campo_tests, oleo_borda_w3b_tests}.rs`.
Ficheiros F: `tp/tests/it/familias/{pastel_carvao, grafite, lapis, canetas, caligrafia_marcadores, aerografo, basicos,
efeitos, texturas, cap01}.rs`, `tp/tests/it/{a_caneta_deitada, o_grao_do_lapis, as_leis_da_escala, o_relevo_dos_fios,
os_fios, cerda_e_marca, o_dual_brush, o_modo_do_grao, o_po_do_aerografo, textura_do_tamanho, sonda_*}.rs`,
`tp/{traco_plano_tests, mao_velocidade_tests, papel_nasce_tests, cor_do_papel_tests, efeito_vazio_tests}.rs`,
`tp/tests/watercolor_{session,water}.rs`. A classificação por ficheiro é por leitura do grep (contagem de menções e de
fixtures `rebelle*`), NÃO por leitura de cada teste — a E13 confere teste a teste antes de apagar (CLAUDE §5.0: confira
o código antes de acreditar).

## 9. Em aberto (medir, não escolher)

- `M_UM` e `k_w` (§2.3, §7.1): calibrados contra as miniaturas de fábrica (golden), tabela na E4.
- RSS a 4096² nas duas leis (§2.4) antes de qualquer tecto.
- A regra «exacto sobre branco» (§3.2) sobre papel de cor (`cor_do_papel_tests.rs`): medir o erro de composição sobre
  creme na F0 (colunas `*_plano` com `R_w` cru × composto).
- Persistência: o grid do Wet Paint entra no `.ph2dproj`? Se sim, E3 é «formato gravado» (`motivo:` crate inteira) e
  precisa de versão.
