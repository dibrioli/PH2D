# F0 — a varredura do óleo (a régua-mãe da refatoração do Wet Paint)

> Leitor: a próxima LLM. Prompt: [PROMPT_REFATORACAO_WETPAINT_E_AJUDA.md §F0](../49_realshader/PROMPT_REFATORACAO_WETPAINT_E_AJUDA.md).
> Defeito-alvo: [`dono_6.png`](../49_realshader/smoke_31_10_oleo/dono_6.png) — Thick Impasto, Pigment baixo e Water
> alto, amarelo sobre azul assentado, papel creme: (1) manchas de azul MAIS ESCURO que o azul (pigmento que se
> amontoa), (2) pontos ferrugem na orla, (3) verde em degraus por passada.

## 1. A sonda

`crates/ph2d-tool-painter/src/tool/paint/wetpaint_preset/sonda_varredura_oleo.rs` (`#[ignore]`, só motor, sem app,
CPU — nada de `PH2D_WET_GPU`). Por célula: tela 360×240 TRANSPARENTE (o alfa do `canvas_rgba` é a tinta; composto
sobre creme `236,226,206` na régua), relógio fixo, semente 0, o preset de fábrica aplicado a cada passada e depois
`SetValue(PAINTER_WETPAINT_PIGMENT|WATER)` (o mesmo caminho do painel), Size 32, N passadas em vaivém de x 50→310:

1. azul `40,90,220` em y≈120 (±6 por passada), pausa 60 ticks entre passadas;
2. **assenta** `PH2D_SONDA_ASSENTA` ticks (300 = 5 s);
3. amarelo `245,205,25` em y≈150 — meio sobre o azul, meio sobre o papel (dá as duas orlas);
4. 120 ticks, `wet_recomposite_full`, mede no recorte x 30..330, y 70..200 (que é o PNG gravado).

Grelha (as faixas são as do PAINEL, lidas no código, não chutadas):

| eixo | valores | fonte da faixa |
|---|---|---|
| Pigment | 40 · 600 · 2000 | `PIGMENT_MAX = 2000` em `crates/ph2d-panel-painter-layers/src/paint_wetpaint.rs` (mín 0; 0 não pinta) |
| Water | 0 · 0,5 · 1 | `volume_row(.., (0.0, 1.0))` em `paint_wetpaint_presets.rs`; clamp em `wetpaint_settings.rs` `set_wet_knob_value` |
| pressão | 0,2 · 0,6 · 1,0 | — |
| N passadas | 1 · 3 | — |

Pincéis (15 — todos os presets em `assets/brushes/WetPainter/Oil/**` + `ExpressOil/Round/Express Oil`; os de
fábrica estão em Pigment 75–2000, Water 0–0,1).

Métricas por célula (colunas do TSV):

| coluna | o que é |
|---|---|
| `km_max`, `km_p99` | distância (níveis sRGB) à curva K-M azul–amarelo (`fora_da_mistura`, cópia da de `assenta/oleo_sobre_oleo_tests.rs`) nos píxeis de alfa ≥ 0,9. ⚠️ Inclui a LUZ do relevo (o render do Real Oil sombreia), por isso é alta onde há empasto: lê-se por comparação entre células, não em absoluto |
| `manchas_pct` | % dos píxeis cobertos (alfa ≥ 0,9) com luminância < (p50 do azul sozinho − 6) — o defeito (1) |
| `manchas_azul_pct` | o mesmo, medido no azul SOZINHO antes do amarelo (separa «o azul já nasce manchado» de «o amarelo o amontoa») |
| `ferrugem_pct` | % dos píxeis de tinta (alfa ≥ 0,3) com matiz < 40° ou > 250° e saturação > 0,2 (fora do arco azul–verde–amarelo; o creme tem s = 0,13) — o defeito (2) |
| `furos_pct` | % de papel à vista (alfa < 0,9) no miolo do azul (x 100..260, y 118..142) |
| `rug`, `p99_altura` | rugosidade média e p99 da altura fina (`sonda_oleo_grumos::mede`), NaN se o preset não liga o Real Oil |
| `ref_azul` | a luminância de referência (p50 do azul sozinho) |

## 2. Regerar

```bash
# compila o binário de teste (uma vez) e corre UMA célula para confirmar
PH2D_SONDA_OUT=/tmp/f0 PH2D_SONDA_PINCEIS="Oil/Flat/Thick Impasto" PH2D_SONDA_PIGMENTOS=40 PH2D_SONDA_AGUAS=1 \
  PH2D_SONDA_PRESSOES=1 PH2D_SONDA_NS=3 bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke \
  --lib sonda_varredura_oleo -- --ignored --nocapture
# a grelha inteira: 15 pincéis × 54 células ≈ 810 × ~6 s; em 8 processos ≈ 20 min
bash scripts/ph2d-run.sh bash <corre_f0.sh> /tmp/f0full     # ver §2.1
python3 <mosaico_f0.py> /tmp/f0full/varredura.tsv /tmp/f0full/celulas docs/Painter/Arquitetura/fotos/f0_varredura
python3 <resumo_f0.py> /tmp/f0full/varredura.tsv            # a tabela da §3
```

Variáveis: `PH2D_SONDA_OUT` (pasta; `celulas/` dentro), `PH2D_SONDA_TSV`, `PH2D_SONDA_PINCEIS` (relativo a
`WetPainter/`, separados por vírgula), `PH2D_SONDA_PIGMENTOS`, `PH2D_SONDA_AGUAS`, `PH2D_SONDA_PRESSOES`,
`PH2D_SONDA_NS`, `PH2D_SONDA_ASSENTA`.

### 2.1 Os scripts auxiliares

Os três scripts (`corre_f0.sh`, `mosaico_f0.py`, `resumo_f0.py`) foram escritos no scratchpad do agente F0 e
copiados para [`fotos/f0_varredura/`](fotos/f0_varredura/) ao lado das fotos. `corre_f0.sh` parte os 15 pincéis em
8 grupos e corre o binário de teste já compilado (`target/smoke/deps/ph2d_tool_painter-*`) com `--exact`.
Mosaico: linhas = Pigment × Water, colunas = N × pressão; legenda por célula `M<manchas %> F<ferrugem %> K<km_p99>`.

## 3. Resultado

**Corrida do F0 (a 1.ª, `corre_f0.sh`, 8 × `timeout 1500`), fechada:** **564 de 810 células**, juntas em
[`fotos/f0_varredura/varredura.tsv`](fotos/f0_varredura/varredura.tsv). Os mosaicos (um PNG por pincel, `<pincel>.png`,
ex. [`thick_impasto.png`](fotos/f0_varredura/thick_impasto.png)) estão na mesma pasta. A grelha está COMPLETA (54) em 9
pincéis: Thick Impasto, Filbert, Filbert Dry, Flat Bristle, Tilted Shade, Dabber, Dabber 4, Knife Oily e Express Oil (52).
Os outros foram cortados pelo timeout: Knife Smooth 3 27, Knife Textured 14, Knife Grainy 12, Metallic Flat 11, Knife
Stain 8, Knife Dry 8. A pior célula por pincel está na §3.2. Os cortados retomam-se com `PH2D_SONDA_PINCEIS=<pincel>`,
um processo por pincel.

**F0b (31/10):** a grelha antiga voltou a correr (8 processos, `timeout 1500`) e no fecho do F0b tinha **277 células**
(Pigment 40 e 600 quase completos em 8 pincéis; P2000 e os outros 7 pincéis por correr). Os mosaicos desse parcial
(`grelha_antiga/`) saíram no commit seguinte — substituídos pelos da grelha completa (um PNG por pincel na pasta); linhas =
Pigment × Water, colunas = N × pressão, legenda `M<manchas %> F<ferrugem %> K<km_p99>` por célula (só com luz: é a
sonda de ANTES do render plano). Pior célula por pincel nesse parcial:

| pincel | km_p99 | manchas % | manchas azul % | ferrugem % | rug |
|---|---|---|---|---|---|
| Flat/Thick Impasto | 145.0 `P600/W0/pr1/N1` | 12.4 `P600/W0/pr1/N1` | 44.3 `P600/W0/pr1/N3` | 0.5 `P40/W0.5/pr0.2/N3` | 1.1 |
| Flat/Filbert | 67.2 `P600/W0/pr1/N3` | 7.3 `P600/W0/pr1/N1` | 19.0 `P600/W0/pr1/N1` | **5.5** `P600/W0.5/pr0.2/N1` | 0.2 |
| Flat/Filbert Dry | 67.1 `P600/W0/pr1/N3` | 7.8 `P600/W0/pr1/N1` | 21.5 `P600/W0/pr1/N3` | 3.5 `P600/W0.5/pr0.2/N1` | 0.2 |
| Flat/Flat Bristle | 73.5 `P600/W0/pr1/N3` | 8.7 `P600/W0/pr1/N1` | 19.4 `P600/W0/pr1/N3` | 2.1 `P600/W0.5/pr0.2/N1` | 0.3 |
| Flat/Tilted Shade | 65.8 `P600/W0/pr1/N3` | 7.3 `P600/W0.5/pr1/N1` | 19.5 `P600/W0/pr1/N3` | 2.9 `P600/W0.5/pr0.2/N1` | 0.2 |
| Dabber/Dabber | 117.9 `P600/W1/pr1/N3` | 7.7 `P600/W0/pr1/N1` | 23.5 `P600/W0/pr1/N1` | 1.4 `P600/W0.5/pr0.6/N1` | 0.2 |
| Dabber/Dabber 4 | 65.1 `P2000/W0/pr1/N1` | 5.5 `P2000/W0/pr1/N1` | 17.3 `P2000/W0/pr1/N3` | **5.4** `P600/W0.5/pr0.6/N1` | 0.3 |
| Knife/Knife Oily | 172.1 `P600/W0/pr1/N1` | 18.1 `P600/W0/pr0.2/N1` | 41.9 `P600/W0/pr0.6/N3` | 0.1 `P600/W0.5/pr0.6/N3` | 1.9 |

`furos` = 100 % em todos a `P40/pr0.2` (tinta fina, não furo — ver §4). Juntar o resto: `head -1 g0.tsv` + `tail -n +2`
de cada `g*.tsv` em `…/scratchpad/f0full/`, depois `mosaico_f0.py` e `resumo_f0.py`.

### 3.0 F0b — render plano × luz, a ferrugem e os degraus

A sonda grava agora, por célula, as métricas DUAS vezes: com luz (as colunas antigas) e **plano** (`*_plano`: o Real
Oil desligado com `set_wet_oleo_real(None)` e o Emboss a 0 pelo evento do painel, `wet_tuning_slider_id("emboss")`,
seguido de `wet_recomposite_full` — a cor K-M do assentado; a próxima passada reaplica o preset), e um PNG
`…_plano.png` ao lado. Novas variáveis: `PH2D_SONDA_PAPEL=<depth>` (> 0 arma o papel de fábrica `PaperCold` como o
slider Depth; `0` = o do app por omissão: nenhum papel escolhido, o filme do óleo usa `PaperCold` via
`papel_do_documento`), `PH2D_SONDA_ORDEM=aya` (azul → amarelo → azul, a 3.ª mão em y 132). Colunas novas: `degraus_y`,
`degraus_x` (e `_plano`): saltos > 3° do matiz médio do verde (65–190°, s > 0,15) entre linhas vizinhas (x 150..210,
através das passadas) e entre colunas vizinhas (y 130..146, ao longo do traço). Células medidas:
[`f0b_celulas.tsv`](fotos/f0_varredura/f0b_celulas.tsv); script [`corre_f0b.sh`](fotos/f0_varredura/corre_f0b.sh)
`<out> <tsv> <pincéis> <pigs> <águas> <pressões> <ns> <pausa> <papel> <ordem>`.

**Plano × luz** (a mesma célula, as duas leituras):

| célula | manchas azul luz / plano | manchas luz / plano | km_p99 luz / plano | ferrugem luz / plano | Δ máx PNG luz−plano |
|---|---|---|---|---|---|
| Knife Oily P20/W0/pr1/N3 (pausa 30, papel 0,5) | 21,85 / 21,85 | 7,99 / 7,99 | 57,3 / 57,3 | 0 / 0 | **0** |
| Knife Oily P20/W1/pr1/N3 | 18,23 / 18,34 | 5,39 / 5,38 | 59,2 / 58,9 | 0 / 0 | 2 |
| Knife Oily P600/W0.5/pr0.2/N1 | 16,93 / 17,00 | 11,55 / 12,11 | 79,7 / 78,7 | 0,02 / 0,02 | — |
| Thick Impasto P20/W1/pr1/N3 | 22,39 / 22,39 | 15,93 / 15,80 | 33,7 / 33,3 | 0 / 0 | — |
| Thick Impasto P20/W1/pr0.6/N3 | (nada com alfa ≥ 0,9) | — | — | 0 / 0 | **56** |
| Thick Impasto P40/W0.5/pr0.2/N3 | (nada com alfa ≥ 0,9) | — | — | **0,54 / 0** | — |
| Filbert P600/W0.5/pr0.2/N1 | (nada com alfa ≥ 0,9) | — | — | **5,49 / 5,53** | — |

- ⛔ **Knife Oily: a hipótese da §5.4 está REFUTADA.** Os 18–22 % de manchas no azul sozinho são iguais no plano
  e com luz (o PNG da P20/W0 é idêntico ao bit). Não é sombra da crista: a `Grid::crista` só é lida pela óptica do
  óleo (`oleo_campo.rs:85-107`), e desligá-la não muda nada aqui. As manchas estão na COR do assentado (pigmento
  que a lâmina amontoa), e a lei nova tem de as tratar como pigmento.
- **Thick Impasto aguado (P10–P20, pr 0,6):** as manchas escuras «dentro do azul/verde» que se vêem no PNG com luz
  SOMEM no plano (Δ até 56 níveis) — ali o defeito (1) é óptica do óleo, não pigmento:
  [`f0b_thick_impasto_P10_P20_luz_plano.png`](fotos/f0_varredura/f0b_thick_impasto_P10_P20_luz_plano.png) (esquerda =
  luz, direita = plano; o nome do ficheiro por cima de cada célula).
- **Ferrugem: REPRODUZ** — `Oil/Flat/Filbert` P600 · W0,5 · pressão 0,2 · N 1 (pausa 300, sem papel): **5,49 %** com
  luz e **5,53 %** no plano ⇒ está na lei da COR, não na luz. Pontos salmão/laranja no amarelo fino por cima da orla
  fina do azul. Também na grelha antiga: Dabber 4 5,4 %, Filbert Dry 3,5 %, Tilted Shade 2,9 %, Flat Bristle 2,1 %
  (todos a pressão baixa e W 0,5 — o regime de tinta fina, alfa < 0,9). No Thick Impasto só **0,54 %** (P40/W0.5/pr0.2/N3)
  e é luz (0 no plano); no Knife Oily ≤ 0,02 %. Foto:
  [`f0b_ferrugem.png`](fotos/f0_varredura/f0b_ferrugem.png) — por linha (rótulo na imagem): recorte · ferrugem a
  vermelho · crop 4× do mediano dos píxeis ferrugem; linhas = Filbert luz, Filbert plano, Thick Impasto luz, plano,
  Knife Oily luz, plano. Grelha antiga: [`f0b_ferrugem_grelha_antiga.png`](fotos/f0_varredura/f0b_ferrugem_grelha_antiga.png).
  As variantes pedidas no Thick Impasto/Knife Oily (P20, pr 0,6 e 1, pausa 30, papel de fábrica Depth 0,5, ordem
  aya) deram **0 %**: nem papel, nem pausa, nem a 3.ª mão a acordam; o que a acorda é a tinta FINA dos dois lados.
- ⚠️ **A aguada pálida (luminância ~150) não se mede com a régua de cobertura:** abaixo de P40 ou pr < 1 nada chega a
  alfa 0,9, por isso `ref_azul`, `km` e `manchas` dão NaN/0. Para o regime do dono a régua tem de passar a alfa ≥ 0,3
  sobre o creme (fica para a F1).
- **Degraus:** `degraus_y` 7–16 nas células com verde (N 3) e 0–10 com N 1, sem diferença luz/plano; `degraus_x` 0–5
  (pico de 113 numa célula de tinta fina — o perfil é ruidoso abaixo de alfa 0,9). A métrica separa N 1 de N 3 mas
  ainda conta ruído de grão como degrau; o limiar abaixo é provisório.

### 3.1 dono_6 — o que reproduz

Thick Impasto, N = 3, pressão 1 ([TSV](fotos/f0_varredura/thick_impasto_dono6.tsv),
[mosaico](fotos/f0_varredura/thick_impasto_dono6_mosaico.png), [célula P40/W1](fotos/f0_varredura/thick_impasto_P40_W1_pr1_N3.png)):

| Pigment | Water | km_max | km_p99 | manchas % | manchas no azul sozinho % | ferrugem % | rug | p99 altura |
|---|---|---|---|---|---|---|---|---|
| 40 | 1 | 81,8 | 34,7 | 5,85 | **7,81** | 0,00 | 0,134 | 4,05 |
| 40 | 0,1 | 147,5 | 55,4 | 2,97 | 10,59 | 0,06 | 0,119 | 4,61 |
| 600 | 1 | 209,8 | 130,4 | 12,80 | 38,15 | 0,00 | 0,866 | 32,84 |
| 600 | 0,1 (fábrica) | 232,5 | 137,7 | 10,72 | 35,75 | 0,11 | 0,541 | 31,74 |

- **Defeito (1) — manchas de azul mais escuro: REPRODUZ.** Célula `Thick Impasto P40/W1/pr1/N3` (a do dono: Pigment
  baixo, Water alto): 7,8 % do azul SOZINHO fica > 6 níveis mais escuro que o p50 do próprio azul, e as manchas
  vêem-se no PNG como riscos azul-escuro dentro do azul claro. Nasce no azul (antes do amarelo), não no amarelo
  por cima: `manchas_azul` ≥ `manchas` nas células com N = 3 (com N = 1 ambas ≈ 0). Com Pigment 600 a régua lê 36–38 %, mas aí a
  maior parte é SOMBRA do relevo (p99 altura 32 contra 4) — a régua de luminância não separa pigmento amontoado de
  sombra; ver §4.
- **Defeito (2) — ferrugem na orla: REPRODUZ na grelha completa** (corrige a leitura do parcial, que só tinha W 0).
  Aparece no regime de tinta FINA, com Water 0,5 e pressão baixa:
  - Filbert `P600/W0.5/pr0.2/N1`: **5,5 %**, uma tira salmão/laranja entre o verde e o amarelo
    ([foto](fotos/f0_varredura/ferrugem_filbert_P600_W0.5_pr0.2_N1.png));
  - Dabber 4 `P600/W0.5/pr0.6/N1`: 5,4 % ([foto](fotos/f0_varredura/ferrugem_dabber4_P600_W0.5_pr0.6_N1.png));
  - Filbert Dry 3,5 % · Tilted Shade 2,9 % · Flat Bristle 2,1 % · Dabber 1,4 %;
  - no Thick Impasto só 0,5 % (`P40/W0.5/pr0.2/N3`), e o F0b (§3.0) mostra que é luz.
- **Defeito (3) — verde em degraus: visível no mosaico** (as faixas de N = 3 têm 3 verdes distintos na
  sobreposição), mas nenhuma métrica o mede ainda; proposta abaixo.

### 3.2 Pior célula por pincel (grelha do F0, 564 células, só COM LUZ; a leitura plana é a do F0b, §3.0)

Os pincéis cortados (Knife Stain, Knife Dry, Knife Textured, Knife Grainy, Metallic Flat, Knife Smooth 3) só têm Pigment 40 e alguma Pigment 600: o máximo deles é parcial.

| pincel | km_p99 | manchas_pct | manchas_azul_pct | ferrugem_pct | furos_pct | rug |
|---|---|---|---|---|---|---|
| Flat/Thick Impasto | 145.0 `P600/W0/pr1/N1` | 22.8 `P2000/W1/pr1/N1` | 44.3 `P600/W0/pr1/N3` | 0.5 `P40/W0.5/pr0.2/N3` | 100.0 `P40/W0/pr0.2/N1` | 5.8 `P2000/W0.5/pr1/N1` |
| Knife/Knife Stain | — | 0.0 `P40/W0/pr0.2/N1` | 0.0 `P40/W0/pr0.2/N1` | 0.0 `P40/W0/pr0.2/N1` | 100.0 `P40/W0/pr0.2/N1` | 0.0 `P40/W0/pr1/N3` |
| Flat/Filbert | 145.8 `P2000/W1/pr1/N3` | 13.1 `P2000/W1/pr1/N1` | 37.2 `P2000/W1/pr1/N3` | 5.5 `P600/W0.5/pr0.2/N1` | 100.0 `P40/W0/pr0.2/N1` | 0.6 `P2000/W0.5/pr1/N3` |
| Knife/Knife Dry | 57.4 `P40/W0/pr1/N1` | 9.6 `P40/W0/pr0.6/N3` | 16.1 `P40/W0/pr1/N1` | 0.0 `P40/W0.5/pr0.2/N3` | 100.0 `P40/W0/pr0.2/N1` | 0.0 `P40/W0/pr1/N3` |
| Flat/Filbert Dry | 129.1 `P2000/W0/pr1/N3` | 12.5 `P2000/W0/pr1/N1` | 37.6 `P2000/W0/pr1/N3` | 3.5 `P600/W0.5/pr0.2/N1` | 100.0 `P40/W0/pr0.2/N1` | 0.6 `P2000/W0.5/pr1/N1` |
| Knife/Knife Textured | — | 0.0 `P40/W0/pr0.2/N1` | 0.0 `P40/W0/pr0.2/N1` | 0.0 `P40/W0/pr0.2/N1` | 100.0 `P40/W0/pr0.2/N1` | 0.0 `P40/W0/pr1/N3` |
| Flat/Flat Bristle | 158.9 `P2000/W1/pr1/N3` | 12.5 `P2000/W0/pr1/N1` | 38.3 `P2000/W1/pr1/N3` | 2.1 `P600/W0.5/pr0.2/N1` | 100.0 `P40/W0/pr0.2/N1` | 0.7 `P2000/W0/pr1/N1` |
| Metallic/Metallic Flat | 36.0 `P40/W0.5/pr0.2/N1` | 0.1 `P40/W0/pr1/N3` | 0.5 `P40/W0.5/pr0.2/N3` | 0.0 `P40/W0/pr0.2/N1` | 97.3 `P40/W0.5/pr0.2/N1` | 0.1 `P40/W0/pr1/N1` |
| Flat/Tilted Shade | 139.8 `P2000/W0/pr1/N3` | 12.5 `P2000/W1/pr1/N1` | 35.4 `P2000/W1/pr1/N3` | 2.9 `P600/W0.5/pr0.2/N1` | 100.0 `P40/W0/pr0.2/N1` | 0.6 `P2000/W0/pr1/N1` |
| Round/Express Oil | 43.5 `P40/W1/pr1/N3` | 25.8 `P40/W0.5/pr1/N3` | 21.7 `P2000/W0/pr1/N3` | 0.0 `P40/W0/pr0.2/N1` | 100.0 `P40/W0/pr0.2/N1` | — |
| Dabber/Dabber | 169.3 `P2000/W0/pr1/N3` | 10.9 `P2000/W0/pr1/N1` | 38.9 `P2000/W0/pr1/N3` | 1.4 `P600/W0.5/pr0.6/N1` | 100.0 `P40/W0/pr0.2/N1` | 0.6 `P2000/W0/pr1/N3` |
| Dabber/Dabber 4 | 65.1 `P2000/W0/pr1/N1` | 5.5 `P2000/W0/pr1/N1` | 17.3 `P2000/W0/pr1/N3` | 5.4 `P600/W0.5/pr0.6/N1` | 100.0 `P40/W0/pr0.2/N1` | 0.3 `P2000/W0/pr1/N3` |
| Knife/Knife Smooth 3 | 132.1 `P600/W0/pr1/N3` | 20.4 `P600/W0/pr0.2/N3` | 34.6 `P600/W0/pr1/N3` | 0.1 `P600/W0.5/pr0.2/N1` | 100.0 `P40/W0/pr0.2/N1` | 0.7 `P600/W0/pr1/N3` |
| Knife/Knife Oily | 193.4 `P2000/W0/pr0.2/N3` | 27.6 `P2000/W0/pr0.2/N1` | 44.5 `P2000/W1/pr1/N1` | 0.2 `P600/W1/pr0.6/N3` | 100.0 `P40/W0/pr0.2/N1` | 18.6 `P2000/W1/pr1/N3` |
| Knife/Knife Grainy | 29.2 `P40/W0.5/pr1/N3` | 0.0 `P40/W0/pr0.2/N1` | 0.0 `P40/W0/pr0.2/N1` | 0.0 `P40/W0/pr0.2/N1` | 100.0 `P40/W0/pr0.2/N1` | 0.1 `P40/W0/pr1/N3` |

`furos = 100 %` a pressão 0,2 com Pigment 40 não é furo: é tinta fina (alfa < 0,9 em todo o miolo). A régua de furos
tem de se restringir às células onde o MESMO pincel cobre a pressão 1 (ver §4).

## 4. Proposta de limiar (o que a lei nova tem de passar, em TODA a grelha)

> ⛔ **Dono 31/10:** o portão-mãe corre sobre o **papel texturizado PADRÃO** do app (é onde o artista pinta), não só
> «sem papel»: a grelha da lei nova leva as duas colunas (padrão · sem papel) e o padrão é o que decide.

Proposta (a confirmar quando a grelha inteira acabar; os números de partida são os medidos acima):

| métrica | limiar do portão | porquê |
|---|---|---|
| `manchas_azul_pct` (tinta sozinha mais escura que ela própria) | **≤ 1 %** em toda célula com `p99_altura` ≤ 8 (sem empasto) | K-M: concentrar pigmento escurece até à cor de massa, nunca abaixo; a cor pura é o tecto de escuro. Hoje 7,8 % em `Thick Impasto P40/W1/pr1/N3` (dono_6) |
| idem com empasto | medir com a LUZ desligada (`Emboss = 0`/render plano) e aplicar o mesmo ≤ 1 % | a sombra do relevo não é pigmento; a régua de luminância com luz confunde as duas (38 % a P600) |
| `km_p99` (render plano) | **≤ 8 níveis**; `km_max` ≤ 20 | a mistura de duas tintas mora na curva K-M (a lei da `fora_da_mistura`); com luz o número mede sombra |
| `ferrugem_pct` | **0 %** (≤ 0,05 % de ruído de borda) | nenhuma mistura azul+amarelo sai do arco azul–verde–amarelo; hoje não reproduz, mas o portão fica para a lei nova não o abrir |
| `furos_pct` | **0 %** nas células em que o mesmo pincel a pressão 1 cobre (alfa ≥ 0,9 no miolo) | separar furo de tinta fina |
| degraus (`degraus_y_plano`) | **≤ N − 1** saltos > 3° no perfil vertical do verde (as fronteiras entre passadas e nada mais); `degraus_x_plano` ≤ 2 | defeito (3); hoje 7–16 com N 3. Provisório: calibrar o limiar de 3° contra o grão |

**Limiares finais (F0b), lidos sempre no PLANO** (`*_plano`), com a leitura com luz só como diagnóstico:
`manchas_azul_plano ≤ 1 %` · `km_p99_plano ≤ 8`, `km_max_plano ≤ 20` · `ferrugem_plano ≤ 0,05 %` (hoje 5,53 % no
Filbert P600/W0.5/pr0.2/N1 — é o caso vermelho do portão) · `furos = 0` só nas células cujo par a pressão 1 tem
`furos < 1 %` (condicionar em `resumo_f0.py`, não na sonda) · degraus como acima. Falta à sonda: a régua de
cobertura em alfa ≥ 0,3 para o regime aguado (§3.0).

## 5. Estado no prazo do agente F0 e como retomar

**A corrida da grelha.** `corre_f0.sh` lança 8 processos, cada um com `timeout 1500`. Eles terminam sozinhos e nenhum
fica órfão. A ~11 células/min, porém, cada grupo é CORTADO antes de acabar os seus 108 (2 pincéis × 54). O fim do
script junta o que houver em `varredura.tsv`. Para completar, corra de novo só as células que faltam: um grupo por
pincel com `PH2D_SONDA_PINCEIS=<pincel>` e `PH2D_SONDA_PIGMENTOS`/`PH2D_SONDA_AGUAS` restritos ao que falta no TSV,
ou suba o `timeout` do script num momento com a máquina calma.

**F0b fez os itens 1–3 (ver §3.0); o 4 está REFUTADO.** O texto abaixo fica como o pedido original.
1. **Render plano.** Ler `km` e `manchas` sem a luz nem o relevo do Real Oil, com a cor K-M do assentado pura, e manter
   a leitura com luz como colunas à parte. Caminho provável: zerar `Emboss` e a luz do Real Oil no `PainterTool` antes
   de `wet_recomposite_full` (conferir o setter em `wetpaint_settings/tela_e_oleo.rs`). Também se pode ler a cor do
   assentado direto da grade.
2. **Variantes da ferrugem**, numa grelha pequena: Thick Impasto + Knife Oily.
   - (a) Azul em aguada (P40/W1); amarelo pela orla com pausa entre as cores de 10, 30 e 60 ticks. Hoje a pausa entre
     cores é `PH2D_SONDA_ASSENTA`, logo basta `PH2D_SONDA_ASSENTA=10|30|60`, sem código.
     **Medido no prazo** (P40/W1/pr1/N3, pausa entre cores 10 e 60 ticks): a ferrugem fica em **0,00 %** nos dois
     pincéis. As manchas no azul sozinho dão 9,0/6,6 % no Thick Impasto e 25,6/24,5 % no Knife Oily. ⚠️ A aguada da
     sonda NÃO é fina: `ref_azul` = 96, contra a luminância ~150 do azul pálido em dono_6. O próximo passo é descer o
     Pigment abaixo de 40 (10–20) ou a pressão até `ref_azul` ≈ 150, e só depois testar a orla.
   - (b) O papel de fábrica do app.
   - (c) A ordem azul → amarelo → azul.
3. **Degraus.** Histograma do matiz ao longo do eixo do traço no verde: contar os saltos acima do limiar. Também falta
   condicionar os `furos` à cobertura do mesmo pincel a pressão 1.
4. **Knife Oily, mecanismo (só leitura, HIPÓTESE).** O preset liga `knifeRidge = 3` px e `bristleLift = 0,4`, que o
   Thick Impasto não tem. O rebordo da lâmina (`crates/ph2d-wet-paint/src/lamina.rs:11-19`) penteia a altura até uma
   crista nas duas bordas e grava-a no campo `Grid::crista`, que a sombra soma. Os 24,6 % de `manchas` e o km_p99 106
   com Pigment 40/Water 0 seriam então essa SOMBRA da crista lida como luminância, e não pigmento amontoado. Quem
   confirma é o render plano (item 1).

Retomar a variante (a), sem código novo:

```bash
for p in 10 30 60; do PH2D_SONDA_OUT=/tmp/f0_ferrugem PH2D_SONDA_TSV=a_$p.tsv PH2D_SONDA_ASSENTA=$p \
  PH2D_SONDA_PINCEIS="Oil/Flat/Thick Impasto,Oil/Knife/Knife Oily" PH2D_SONDA_PIGMENTOS=40 PH2D_SONDA_AGUAS=1 \
  PH2D_SONDA_PRESSOES=0.6,1 PH2D_SONDA_NS=3 bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke \
  --lib sonda_varredura_oleo -- --ignored --nocapture; done
```
