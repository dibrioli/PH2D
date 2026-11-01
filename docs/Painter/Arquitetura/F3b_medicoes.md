# F3b — Medições ⚑ da E10 (F3 §4 (3)(4)(6)(1)), 31/10

Sonda: `crates/ph2d-wet-paint/tests/f3_medicoes.rs` (3 testes `#[ignore]`, um processo, variantes intercaladas por preset).
Presets lidos dos `.ph2d-brush.png` (iTXt `ph2d-brush`) para `PH2D_F3_PRESETS` (`nome|chave=v;…|size_px|water|opacity`):
os 14 `Oil/*` + Wet Liner, Wet Wash, Gouache Opaque (os 15 com `levelingOwnStroke = 1`).

```
PH2D_F3_PRESETS=<presets.txt> PH2D_F3_TINTA=<Ink Brush, Wet Round, Gouache Flat, Wet Dry Brush> bash scripts/ph2d-run.sh cargo test -p ph2d-wet-paint --profile smoke \
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

## (1) Body — `levelingOwnStroke` 1 × 0 (2.ª rodada, dois traços cruzados)

⛔ **A 1.ª rodada desta secção era NULA:** o `pointer_down` legado não avança `sim.traco_actual` (quem o faz é a porta do
produto, `painter/doors.rs:79` `begin_direct_stroke`), logo toda célula ficava com o traço `0` e a parede
(`solver::parede_do_traco`) nunca valia — por isso lev 1 = lev 0 ao bit. A sonda agora avança o número a cada traço como
o produto. Re-corrida a 3 passadas com o número certo: os óleos mudam < 1 % (as tabelas (4)/(6) ficam), mas os 3 de água
passam a ter lev 0 ≠ lev 1 (abaixo).

Teste `f3_body_dois_tracos`: traço 1 horizontal (y = 60), traço 2 vertical (x = 130) por cima — **molhado** (10 passos
depois) ou **seco** (`dry_canvas_now`); leitura 300 passos depois. Por variante, o mesmo processo corre «só o 1.º», «só o
2.º» e «os dois». Colunas:
- **cruz**: µm na intersecção (`|dx| ≤ r₂`, `|dy| ≤ r₁`);
- **borda**: µm na faixa de 3 px por DENTRO da borda do 1.º traço, fora da coluna do 2.º;
- **escoa**: (os dois − só 1.º − só 2.º) na coluna do 2.º FORA da faixa do 1.º ÷ massa do 1.º, ou seja o que o 2.º
  levou do 1.º para fora;
- **fuga**: a massa fora das duas faixas ÷ a massa do 1.º.

| preset | estado | cruz p50/p99 lev1 | lev0 | borda p99 lev1 | lev0 | escoa lev1 | lev0 | fuga lev1 | lev0 |
|---|---|---|---|---|---|---|---|---|---|
| Filbert Dry | molhado | 1,99 / 4,30 | = | 0,74 | 0,74 | 0,73 % | 0,73 % | 0 | 0 |
| Filbert | molhado | 6,58 / 10,79 | 6,58 / 10,79 | 0,67 | 0,67 | 0,50 % | 0,50 % | 0 | 0 |
| Filbert | seco | 6,79 / 10,82 | = | 0,68 | 0,68 | 2,20 % | 2,20 % | 0 | 0 |
| Flat Bristle | molhado | 4,74 / 8,16 | 4,75 / 8,16 | 0,98 | 0,98 | 0,64 % | 0,64 % | 0 | 0 |
| Thick Impasto | molhado | 6,68 / 10,82 | 6,68 / 10,82 | 0,68 | 0,68 | 0,62 % | 0,62 % | 0 | 0 |
| Thick Impasto | seco | 6,82 / 11,01 | = | 0,68 | 0,68 | 2,21 % | 2,21 % | 0 | 0 |
| Tilted Shade | molhado | 4,33 / 7,27 | = | 1,24 | 1,23 | 0,68 % | 0,68 % | 0 | 0 |
| Knife Dry | molhado | 2,22 / 4,79 | 2,21 / 4,79 | 0,70 | 0,70 | 0,27 % | 0,27 % | 0 | 0 |
| Knife Grainy / Smooth 3 | molhado | 2,16 / 4,24 | 2,14 / 4,23 | 0,79 | 0,79 | 0,23 % | 0,23 % | 0 | 0 |
| Knife Oily | molhado | 10,08 / 17,32 | = | 2,77 | 2,79 | 0,48 % | 0,47 % | 0 | 0 |
| Knife Stain / Textured | ambos | ≤ 0,39 p99 | = | ≤ 0,01 | ≤ 0,01 | 0 | 0 | 0 | 0 |
| Metallic Flat | molhado | 2,55 / 3,80 | = | 0,67 | 0,67 | 1,46 % | 1,46 % | 0 | 0 |
| **Wet Liner** | molhado | 0,63 / 1,08 | 0,71 / 1,09 | **12,36** | 2,31 | 1,23 % | 1,44 % | **0** | **5,0 %** |
| Wet Liner | seco | 1,24 / 2,53 | 1,23 / 2,51 | 0,84 | 0,84 | 0,54 % | 0,44 % | 0 | 0 |
| **Wet Wash** | molhado | 0,48 / 0,81 | 0,46 / 0,79 | **38,57** | 0,38 | 0,92 % | −13,6 % | **0,02 %** | **39,8 %** |
| Wet Wash | seco | 1,36 / 2,11 | 1,36 / 2,12 | 0,44 | 0,44 | 0,50 % | 0,55 % | 0 | 0,05 % |
| **Gouache Opaque** | molhado | 0,75 / 1,17 | 0,72 / 1,12 | **15,28** | 2,08 | 5,8 % | −6,2 % | **0** | **10,5 %** |
| Gouache Opaque | seco | 1,39 / 1,96 | 1,39 / 1,96 | 0,38 | 0,38 | 2,1 % | 2,1 % | 0 | 0 |

(«=» quer dizer igual nas 3 casas. Os óleos secos que faltam na tabela dão o mesmo padrão: lev 1 = lev 0 a ≤ 0,02 µm.)
Nas 3 passadas no mesmo eixo, com o número certo, os 3 de água dão lev 1 × lev 0: Wet Liner µm p99 **49,3** × 2,7 e
fuga 0 × 43 %; Wet Wash **82,3** (máx. **300**) × 0,7 e fuga 0,35 % × 46 %; Gouache Opaque **37,8** × 2,9 e fuga 0 × 39 %.

**Conclusão → R1:** **o meio basta nos 12 óleos, não basta nos 3 de água.**
- **12 Oil/***: lev 1 × 0 difere ≤ 0,02 µm no cruzamento e na borda, e o escoamento é o mesmo à 4.ª casa. O óleo não
  escoa por si (não há água), logo a parede não tem o que parar. Para eles `levelingOwnStroke` é redundante: a migração
  põe `Body = 0`.
- **Wet Liner, Wet Wash, Gouache Opaque** (`m = 0`, onde `τ_y(m = 0)` ≈ 0): lev 0 deixa escoar **5–40 %** da massa do
  1.º traço para fora das faixas quando o 2.º o apanha molhado. Lev 1 retém tudo, mas EMPILHA na parede: borda p99
  **12–39 µm** contra 0,4–2,3 µm com lev 0, e até 300 µm em 3 passadas. É um degrau, não um corpo: a lei de hoje é uma
  parede binária entre números de traço, e a sua «faixa» só tem dois pontos (0 e 1).
- **Destino:** `Body` como grandeza `τ_y` NÃO reproduz o lev 1 da água (que é uma parede, não uma tensão de cedência).
  Com o meio a cobrir o óleo, o `Body` sai (N = 30) SÓ se o dono aceitar que os 3 de água escoem como os outros pincéis
  de água (com lev 0 ficam como Wet Round). Se o isolamento da água for querido, a forma honesta é `Body` = `τ_y` somado
  ao do meio, com faixa por medir contra a fuga de 5–40 %, e não a parede binária que empilha 12–39 µm na borda. ⚑ A
  pilha na parede (lev 1, água) é por si um defeito a registar em BUGS: um degrau de 38 µm onde a lei pede uma borda.

## (5) Absorbency × `paperGate`

A lei do `paperGate` (`trail/deposit.rs:195`): `deposit = stamp − (1 − tooth) · gate`. Se der ≤ 0, a célula NÃO recebe
nada, nem pigmento nem água (o `wet` só se escreve com depósito). Outros leitores: `tools.rs:106/138` (borracha) e
`trail.rs:603` (Blend). **Não há velocidade de entrada da água**: o knob é um limiar de CONTACTO com o dente do papel.

Teste `f3_absorbency_paper_gate`: um traço horizontal, papel por omissão, `paperGate` ∈ {0 … 1,2} por cima do preset.
Colunas: **cobre** = fracção da faixa do traço com massa; **água t0** = Σ `film` logo depois; **molhadas t0/t30/t120** =
células com `film > 0,01`; **cresce** = (t30 − t0)/30 células por passo; **sett** = fracção assente a t120.

| preset | gate | cobre | água t0 | água ÷ gate 0 | molhadas t0 | t30 | t120 | cresce | sett t120 |
|---|---|---|---|---|---|---|---|---|---|
| Wet Round | 0 | 0,953 | 28 181 | 1,00 | 10 257 | 10 114 | 13 342 | −4,8 | 0,17 |
| Wet Round | 0,2 | 0,877 | 18 817 | 0,67 | 8 842 | 9 159 | 10 844 | 10,6 | 0,16 |
| Wet Round | 0,4 | 0,840 | 13 361 | 0,47 | 8 331 | 8 723 | 9 529 | 13,1 | 0,13 |
| Wet Round | 0,6 | 0,802 | 9 776 | 0,35 | 7 899 | 8 385 | 8 634 | 16,2 | 0,13 |
| Wet Round | 0,8 | 0,750 | 6 854 | 0,24 | 7 334 | 8 011 | 7 922 | 22,6 | 0,15 |
| Wet Round | 1,0 | 0,670 | 4 568 | 0,16 | 6 487 | 7 528 | 7 214 | 34,7 | 0,18 |
| Wet Round | 1,2 | 0,562 | 2 918 | 0,10 | 5 393 | 6 832 | 6 402 | 48,0 | 0,23 |
| Ink Brush | 0 | 0,952 | 2 849 | 1,00 | 9 420 | 8 697 | 7 884 | −24,1 | 0,50 |
| Ink Brush | 0,4 | 0,472 | 1 123 | 0,39 | 4 334 | 5 819 | 3 695 | 49,5 | 0,70 |
| Ink Brush | 0,8 | 0,363 | 579 | 0,20 | 3 319 | 4 138 | 1 665 | 27,3 | 0,87 |
| Ink Brush | 1,2 | 0,226 | 249 | 0,09 | 2 038 | 2 379 | 133 | 11,4 | 0,99 |
| Wet Dry Brush | 0 | 0,957 | 1 011 | 1,00 | 8 863 | 7 719 | 6 574 | −38,1 | 0,58 |
| Wet Dry Brush | 0,4 | 0,742 | 384 | 0,38 | 6 463 | 4 532 | 673 | −64,4 | 0,94 |
| Wet Dry Brush | 1,2 | 0,466 | 88 | 0,09 | 3 266 | 67 | 0 | −106,6 | 1,00 |
| Gouache Flat | 0…1,2 | igual ao Wet Round a ≤ 1 % na cobertura e na água relativa | | | | | | | |

**Conclusão → F3 R4:** o `paperGate` NÃO tem parte de papel a migrar. Medido, ele só decide QUANTO passa no dente:
- a água pousada cai como `≈ exp(−1,9 · gate)` em todos (Wet Round 1,87–1,89; Ink Brush 2,0–2,3);
- a cobertura da faixa cai em paralelo (0,95 → 0,56 / 0,23);
- a «velocidade» de alastrar sobe com o gate só porque há menos água pousada e mais papel seco à volta (Wet Round −4,8 →
  +48 células/passo), e no Wet Dry Brush a mancha SECA mais depressa (−38 → −107) pela mesma razão: menos água.

Nenhuma destas é a absorção de Washburn (velocidade com que o papel bebe a água que JÁ lá está). Proposta:
1. `paperGate` → **Hardness** inteiro (P, contacto no dente), pela fórmula da §3.1, sem divisão.
2. **Absorbency** (T, por tela) entra como grandeza NOVA com o ponto neutro = o motor de hoje ao bit, e nenhum preset a
   migra (não há conversão `Absorbency ↔ paperGate`: a tabela acima mostra que o knob não a tem).
3. A F3 §2.1 #1 corrige-se: «a parte do papel» não existe no código.

⚑ O leitor da absorção de hoje (evaporação/`dryer`/`capillary` por tela) é que dá o ponto neutro do `Absorbency`, e
esse levantamento está por fazer.

## O que ficou por medir

1. (3) Píxel do `bristleStreak` pela ferramenta (1 dab a 0 × 1).
2. (4) Luz/relevo e custo do solver a 100–1000 µm; a fracção `φ` real por preset para a conversão µm molhado → pigmento.
3. (6) Fotos com a luz real; Fan Brush e Express Oil.
4. (1) Decisão do dono: os 3 de água isolados (Body fica, com faixa a medir contra a fuga de 5–40 %) ou como os outros
   (Body sai, N = 30). A pilha de 12–39 µm na parede, lev 1 com água, vai a BUGS.
5. (5) O ponto neutro do `Absorbency`: quem lê hoje a absorção do papel (`capillary`, `dryer`, `baseEvaporation`).
6. Repetir (4)(6)(1) PELA FERRAMENTA (com `Fios`/`Relevo` e dinâmica) — esta régua é o `Engine` directo.
