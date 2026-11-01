# F3b — Medições ⚑ da E10 (F3 §4 (3)(4)(6)(1)), 31/10

Sonda: `crates/ph2d-wet-paint/tests/f3_medicoes.rs` (3 testes `#[ignore]`, um processo, variantes intercaladas por preset).
Presets lidos dos `.ph2d-brush.png` (iTXt `ph2d-brush`) para `PH2D_F3_PRESETS` (`nome|chave=v;…|size_px|water|opacity`):
os 14 `Oil/*` + Wet Liner, Wet Wash, Gouache Opaque (os 15 com `levelingOwnStroke = 1`).

```
PH2D_F3_PRESETS=<presets.txt> bash scripts/ph2d-run.sh cargo test -p ph2d-wet-paint --profile smoke \
  --test f3_medicoes -- --ignored --nocapture --test-threads 1
```

⚠️ Limites da régua (valem para todas as tabelas): é o `Engine` DIRECTO (sem a ferramenta): sem `Fios`/`Relevo` das
cerdas, sem a dinâmica da caneta, raio = `size_px/2` pelo `r = 2 + size·33`, `Sliders.pressure = 1`, lei `Pigmento`,
`sim.oleo = realOil > 0`. Cena: 3 passadas horizontais no MESMO eixo (40 frames, x 30→230), 120 passos entre passadas,
300 no fim, grade 260×120. Alturas = `grid::espessura::em ÷ M_UM` (µm de pigmento, `M_UM = 1374`), sobre as células
com massa > 1.

## (3) Leitores das cerdas — todos VIVOS

Escritor único: `wetpaint_settings.rs:119–147` (`fios_do_pincel` / `relevo_do_pincel`) ← `Knob::Bristle*`. Leitor de
produção: `dab_route.rs:192–195` → `pousa_o_dab.rs:342/425/431/442` e `lote.rs:264/270` → `wp/bristles_fios.rs` /
`wp/bristles_relevo.rs`. O «nenhum» do censo §1 era o grep pelo KNOB: o leitor lê o campo da struct `Fios`.

Prova: `f3_cerdas_cada_knob_decide` — base = Flat Bristle (pressão 1,2, lift 0,4, lane 4, sat 0,3, light 0,1), cada knob
ao `min` × `max` do registo, 3 210 saídas dos fios (ganho, cor, veio, `no_ponto` q/ganho/fio, alvo do relevo) em 5 arcos.

| knob | leitor (`wp/bristles_fios.rs` salvo indicação) | saídas ≠ ao bit | max \|Δ\| | veredicto | pincéis que o gravam |
|---|---|---|---|---|---|
| bristlePressure | :302–306 `ganho_do_fio` | 865 | 1,55 | vivo | 17 |
| bristleLift | :133 `baixa` → :306 | 865 | 0,80 | vivo | 10 |
| bristleNoiseLength | :127, :305, :346 | 956 | 25,4 | vivo | 2 |
| bristleLaneWidth | :213, :265, :288, :314 | 1 200 | 40 | vivo | 16 |
| bristleWave | :374, :389–393 | 96 | 1,0 | vivo | **0** |
| bristleWaveLength | :393 (só com Wave > 0) | 102 | 1,0 | vivo (dependente) | **0** |
| bristleOscillation | :127 | 1 419 | 4,75 | vivo | 1 |
| bristleSpan | :215, :233–236 | 615 | 6,13 | vivo | 1 |
| bristleSpanLength | :238 (só com Span > 0) | 492 | 1,60 | vivo (dependente) | 1 |
| bristleNoiseHue | :341–348 → `bristles_cor` | 261 | 8,71 | vivo | **0** |
| bristleNoiseSat | idem | 261 | 3,24 | vivo | 14 |
| bristleNoiseLight | idem | 375 | 104 | vivo | 9 |
| bristleStreak | :357–366 | 63 | < 1e-4 | **vivo FRACO** (bits mudam, efeito < 1e-4 nas saídas) | **0** |
| bristleStreakLight | :366 | 67 | 38,0 | vivo | **0** |
| bristleStreakLength | :360 (só com Streak > 0) | 52 | < 1e-4 | vivo FRACO (herda o do Streak) | **0** |
| bristleStreakCut | :361 | 204 | 23,2 | vivo | **0** |
| bristleHeight | `bristles_relevo.rs:57` | 125 | 4,21 | vivo | 11 |

**Conclusão → F3:** nenhum `Bristle*` sai por «morto» (R não se aplica); os 16 vão ao asset `Tip` (T) como o F3 §2.1
já diz, e `bristleHeight` → **Bristle relief** (S). Os 7 com **0 pincéis** (Wave, WaveLength, NoiseHue, Streak,
StreakLight, StreakLength, StreakCut) são vivos mas sem uso: a decisão de os manter no `Tip` é de produto, não de
censo. ⚑ `bristleStreak` (escurecer) mexe < 1e-4 no que medi — falta a prova no PÍXEL (1 dab Flat Bristle a 0 × 1 pela
ferramenta) antes de o dar por vivo de verdade.

## (4) Unidade e faixa do Volume

Receita E4: `h = Volume · carga · H_REF`, `H_REF = 950` (massa) ⇒ um dab de referência = `950 / 1374 = 0,69 µm`.

| preset | raio px | µm p50 | µm p99 | µm max | sett max |
|---|---|---|---|---|---|
| Dabber 4 / Dabber | 25,5 | 3,06 | 9,71 | 11,40 | 8 379 |
| Filbert Dry | 17 | 1,74 | 6,42 | 6,61 | 6 118 |
| Filbert | 32 | 6,62 | 17,40 | 19,93 | 8 874 |
| Flat Bristle | 23 | 4,92 | 13,93 | 15,25 | 8 651 |
| **Thick Impasto** | 32 | **6,51** | **17,77** | 19,52 | 9 517 |
| Tilted Shade | 20 | 4,39 | 13,20 | 14,54 | 9 455 |
| Knife Dry | 25,5 | 2,32 | 6,77 | 8,20 | 7 691 |
| Knife Grainy / Smooth 3 | 25,5 | 2,40 | 7,07 | 8,63 | 7 746 |
| Knife Oily | 25,5 | 9,82 | 32,21 | 39,78 | 8 580 |
| Knife Stain | 25,5 | 0,01 | 1,17 | 1,31 | 1 801 |
| Knife Textured | 25,5 | 0,03 | 1,15 | 1,47 | 2 018 |
| Metallic Flat | 20 | 2,93 | 6,18 | 6,88 | 5 489 |

O que satura primeiro (`f3_tecto_da_espessura`: a mesma faixa carregada com `pigmentPerDab` = 2000, o MÁXIMO do
registo, Water 0, Real Oil, sem drag/smear/pickup):

| passadas | µm p50 | µm p99 | µm max | sett max | finito |
|---|---|---|---|---|---|
| 3 | 0,78 | 2,12 | 2,44 | 3 027 | sim |
| 9 | 2,10 | 4,80 | 5,14 | 4 829 | sim |
| 18 | 2,70 | 8,95 | 9,29 | 7 759 | sim |
| 36 | 3,80 | 17,25 | 17,59 | 13 694 | sim |
| 72 | 6,06 | 33,86 | 34,20 | 25 583 | sim |

| recurso | medido | tecto |
|---|---|---|
| o «tecto de 3000» (`trail/transfer.rs:215/337/352`) | `sett` chega a 25 583 e cresce linear | **não satura**: é suave (derrama a média da janela) e a espessura vive à parte (`grid/espessura.rs`) |
| `f32` da massa | ulp a 1000 µm = 0,125 de massa = 9,1·10⁻⁵ µm; a 10 000 µm, 7,3·10⁻⁴ µm | **não satura** antes de 10⁴ µm |
| a carga por dab | `pigmentPerDab` ≤ 2000 (registo) ⇒ ~0,47 µm por passada no p99 | **é ESTE o tecto de hoje** — um limite do registo, não de recurso |

**Conclusão → F3:** R3 confirmado com número: o Thick Impasto pousa **17,8 µm p99** (6,5 p50) de pigmento em 3
passadas — 6× a 56× abaixo de um empaste real (100–1000 µm), e nem o máximo do registo (72 passadas) passa de 34 µm.
A grade e o `f32` aguentam ≥ 10⁴ µm; o que prende é a escala da carga. Proposta: **`Volume` em µm de tinta MOLHADA
por passada**, faixa **0..1000 µm** (o tecto do recurso medido fica 10× acima), com a conversão
`h_massa = Volume_µm · M_UM · (1 − φ_veículo)` (o pigmento é a fracção `φ` da tinta molhada); o `H_REF` deixa de ser
a régua. ⚑ por medir: o relevo/luz (`oleo_campo.rs:40`, `px_por_massa`) a 100–1000 µm (a sombra e a parede foram
afinadas a ≤ 20 µm), e o custo do solver a essas massas.

## (6) Relevo com `oilDepth` / `emboss` no padrão

Relevo do Óleo Real = `em × ganho` com `ganho = Depth × Real Oil` (`grid/oleo_celula.rs:66`, aplicado ANTES do desfoque
em `oleo_campo.rs:40`) — linear: «depois» = «antes» ÷ `oilDepth`. Emboss = `g.relevo` (`trail/transfer.rs:99/174`) × `em`.
Unidade: µm de pigmento.

| preset | oilDepth | relevo Óleo p50/p99 hoje | com Depth 1 | emboss | relevo Emboss p50/p99 hoje | com Emboss 1 |
|---|---|---|---|---|---|---|
| Dabber 4 | 0,3 | 0,92 / 2,91 | 3,06 / 9,71 | 0,5 | 1,53 / 4,85 | 3,06 / 9,71 |
| Dabber | 1 | 3,06 / 9,71 | = | 0,5 | 1,53 / 4,85 | 3,06 / 9,71 |
| Filbert Dry | 0,3 | 0,52 / 1,93 | 1,74 / 6,42 | 0,4 | 0,70 / 2,57 | 1,74 / 6,42 |
| Filbert | 0,3 | 1,99 / 5,22 | 6,62 / 17,40 | 0,4 | 2,65 / 6,96 | 6,62 / 17,40 |
| Flat Bristle | 0,3 | 1,48 / 4,18 | 4,92 / 13,93 | 0,4 | 1,97 / 5,57 | 4,92 / 13,93 |
| Thick Impasto | 0,9 | 5,85 / 15,99 | 6,51 / 17,77 | 0,3 | 1,95 / 5,33 | 6,51 / 17,77 |
| Tilted Shade | 0,3 | 1,32 / 3,96 | 4,39 / 13,20 | 0,4 | 1,76 / 5,28 | 4,39 / 13,20 |
| Knife Dry | 0,1 | 0,23 / 0,68 | 2,32 / 6,77 | 0,1 | 0,23 / 0,68 | 2,31 / 6,77 |
| Knife Grainy | 1 | 2,40 / 7,07 | = | 0,1 | 0,24 / 0,71 | 2,39 / 7,07 |
| Knife Oily | 1 | 9,82 / 32,21 | = | 0,1 | 0,98 / 3,22 | 9,82 / 32,21 |
| Knife Smooth 3 | 1 | 2,40 / 7,07 | = | 0 | 0 / 0 | 2,39 / 7,07 |
| Knife Stain | 0,1 | 0,001 / 0,12 | 0,01 / 1,17 | 0,1 | 0 / 0,12 | 0 / 1,17 |
| Knife Textured | 0,1 | 0,003 / 0,11 | 0,03 / 1,15 | 0,1 | 0,002 / 0,11 | 0,02 / 1,15 |
| Metallic Flat | 0,5 | 1,46 / 3,09 | 2,93 / 6,18 | 0,2 | 0,59 / 1,24 | 2,93 / 6,18 |
| Wet Liner · Wet Wash · Gouache Opaque | — | 0 (sem Real Oil) | 0 | 0 | 0 | 0,53/2,65 · 0,17/0,51 · 0,54/2,41 |
| Fan Brush · Express Oil | — | não medidos (fora do conjunto) | | 0,05 · 0,2 | | |

**Conclusão → F3 (#38 `emboss` C, #126 `oilDepth` C):** a consequência pura `h_seco(m)` dá relevo 1,1× (Thick
Impasto) a **10×** (Knife Dry/Stain/Textured) mais alto que o de hoje; com Depth e Emboss no padrão os dois relevos
coincidem (o p99 do Emboss com 1 = a espessura). Os 10 `oilDepth` e os 15 `emboss` eram, na prática, um «achatador»
por preset que esconde a espessura real. A migração C está certa como LEI, mas as 7 lâminas e os 4 de cerda ficam
3–10× mais altas: vão TODAS à lista ⚑ de `miniaturas_de_fabrica_golden` com foto. Se o dono quiser as lâminas rasas,
a cura é pousar MENOS tinta (Volume), nunca devolver o ganho. ⚑ por medir: a foto (luz real, `oleo_campo`), e Fan
Brush / Express Oil (emboss 0,05 / 0,2, sem Real Oil).

## (1) Body — `levelingOwnStroke` 1 × 0

| preset | células fora da faixa `r + 2` (lev 1 → lev 0) | massa fora ÷ total | µm p99 lev 0 | massa lev0 ÷ lev1 |
|---|---|---|---|---|
| 12 Oil/* com lev 1 | 0 → 0 | 0 → 0 | = lev 1 (tabela §4) | 1,0000 |
| Wet Liner | 3 683 → 3 683 | 0,432 → 0,432 | 2,725 | 1,0000 |
| Wet Wash | 6 890 → 6 890 | 0,464 → 0,464 | 0,716 | 1,0000 |
| Gouache Opaque | 4 707 → 4 707 | 0,394 → 0,394 | 2,928 | 1,0000 |

**Conclusão (provisória):** nesta cena (três passadas no MESMO eixo) `levelingOwnStroke` não muda nada nos 15 — o
óleo já não escoa para fora da faixa sem ele (0 células), e na água a fuga é a mesma com e sem. O knob só pode decidir
onde um traço encontra OUTRO traço já pousado (é «nivelar só com o próprio traço»). ⚑ **não fecha R1**: falta a cena de
DOIS traços que se cruzam (perpendicular, como `sonda_relevo_oleo` `Perp`) com lev 0 × 1, medida no relevo. Até lá
`Body` fica PROVISÓRIO (N = 31).

## (5) Absorbency — não feito (sem tempo).

## O que ficou por medir

1. (3) Píxel do `bristleStreak` pela ferramenta (1 dab a 0 × 1).
2. (4) Luz/relevo e custo do solver a 100–1000 µm; a fracção `φ` real por preset para a conversão µm molhado → pigmento.
3. (6) Fotos com a luz real; Fan Brush e Express Oil.
4. (1) Cena de dois traços cruzados, lev 0 × 1 — decide `Body`.
5. (5) Absorbency × os 38 `paperGate`.
6. Repetir (4)(6)(1) PELA FERRAMENTA (com `Fios`/`Relevo` e dinâmica) — esta régua é o `Engine` directo.
