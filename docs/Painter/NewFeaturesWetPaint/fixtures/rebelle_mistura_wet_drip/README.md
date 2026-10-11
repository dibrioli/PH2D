# Fixtures — a MISTURA com escorrimento no Rebelle 8.3.4 (2026-10-10, prompt 16)

Saída do oráculo (CLAUDE.md §0.9) sobre gestos NOSSOS; nenhum asset do Rebelle. É a barra dos quatro sintomas do smoke
do dono ([`../smoke_dono_mistura_wet_drip/`](../smoke_dono_mistura_wet_drip/): azul escuro com o Wet Drip sobre um
vermelho molhado, as duas caixas K–M ligadas ⇒ pontos e faixas AZUL-CLARO/CIANO).

- **Programa:** Rebelle 8 Pro 8.3.4 (Trial) sob Wine, `~/Apps/rebelle/`, porta WebSocket 8265, tela virtual
  `kwin_wayland --virtual` 1920×1080 (`ferramentas/gotas_inicia.sh`); foto da janela por `import -window`
  (`~/Apps/rebelle/oraculo/foto.sh`), zoom 100 %, recorte `1000x640+480+220` = a tela 1:1.
- **Gerador:** [`ferramentas/mistura_wet_drip.py`](../../ferramentas/mistura_wet_drip.py) +
  [`mistura_wet_drip_tela.sh`](../../ferramentas/mistura_wet_drip_tela.sh). **Régua:**
  [`ferramentas/mistura_wet_drip_regua.py`](../../ferramentas/mistura_wet_drip_regua.py).
- **Cena (tela 1000×640, `Default/EX00 White Simple`, semente 7, gotas de fábrica 5/5 — sem SET_ENGINE_PARAMS):**
  1. VERMELHO (200, 30, 30), Watercolor/Round 60 px, água do preset, tela DIREITA: y = 400, x 120 → 880;
  2. `seco`: SIMULATION 400 + DRY; `molhado`: nada;
  3. `SET_CANVAS_TILT (0, 1)` ligado (o máximo — o «Wet Drip · Rebelle Round água 100» do doc 05 §4b);
  4. 4 passadas de AZUL ESCURO (20, 50, 110), Watercolor/Round 60 px, **água 100**, pressão 1, cada uma com dois
     gestos: **E** x 150 → 450 em y = 190 + 10·(k−1) (acima do vermelho: só os FIOS lá chegam) e **D** x 550 → 850 em
     y = 340 + 10·(k−1) (a orla de baixo do azul cai DENTRO do vermelho).
  Quatro telas: `mol` (molhado, Opacity do preset) · `seco` (DRY entre os dois) · `mol100` · `seco100` (`opacity` 100
  no SET_BRUSH dos dois = o equivalente da Opacity 1 do nosso Wet Drip; sem ela o Round sai diluído, ~0,67 de branco).
- **Comando** (a partir de `docs/Painter/NewFeaturesWetPaint/`):
  ```
  bash ferramentas/gotas_inicia.sh &          # tela virtual + Rebelle; depois as etapas Registration e New Artwork
                                              # do ~/Apps/rebelle/oraculo/arranque.sh (inj.sh, cada uma com prazo)
  bash ferramentas/mistura_wet_drip_tela.sh mol     molhado 4
  bash ferramentas/mistura_wet_drip_tela.sh seco    seco    4
  bash ferramentas/mistura_wet_drip_tela.sh mol100  molhado 4 100
  bash ferramentas/mistura_wet_drip_tela.sh seco100 seco    4 100
  bash ferramentas/gotas_fecha.sh
  python3 -I ferramentas/mistura_wet_drip_regua.py fixtures/rebelle_mistura_wet_drip/{mol,seco,mol100,seco100}_b.png
  python3 -I ferramentas/mistura_wet_drip_regua.py --so-gama --sem-ui fixtures/smoke_dono_mistura_wet_drip/*.png
  ```

## Ficheiros

| ficheiro | o quê |
|---|---|
| `<tela>_p00.png` | o vermelho, a tela já inclinada |
| `<tela>_p01…p04.png` | ~1,5 s depois da passada k — ⚠️ o Rebelle às vezes NÃO redesenha a tempo e a foto repete a anterior (diferença 0: `mol` p03 = p02; `seco` p04 = p03; `mol100` p04 = p03; `seco100` p01–p03 = p00) |
| `<tela>_a.png` · `_b.png` | +400 · +1 600 passos de SIMULATION — **as medidas são do `_b`** |
| `<tela>_seco.png` | depois de DRY: muda ≤ 5 células do `_b` (o tom ao secar ≈ 0 aqui) |
| `<tela>_8x_juncao.png` | 8×, um fio E a atravessar o vermelho (y 372–427) |
| `<tela>_8x_ponta.png` | 8×, a ponta de um fio E que acaba no branco |
| `<tela>_8x_orla_D.png` | 8×, a orla de baixo do azul D dentro do vermelho (x 640–699, y 376–431) |
| `<tela>_8x_fronteira.png` | 8×, a fronteira lateral esquerda azul D / vermelho |
| `prancha_8x.png` | as 16 de 8× (linhas `mol` · `seco` · `mol100` · `seco100`; colunas junção · ponta · orla D · fronteira) |
| `prancha_1x1_reduzida.png` | os quatro `_b` a 50 % |
| `regua_rebelle.txt` · `regua_controlo_capturas_do_dono.txt` | a saída da régua (Rebelle) e do teste da gama nas capturas do dono (o controlo) |

## Os números (`_b`, régua `mistura_wet_drip_regua.py`)

Luz linear sRGB→linear, lum Rec. 709; tinta = `255 − min(rgb) > 25`.

| | `mol` | `seco` | `mol100` | `seco100` |
|---|---|---|---|---|
| **(a) ponta do fio que acaba no branco**, lum ponta vs corpo (por fio) | .351/.524 · .510/.317 · .678/.685 · .467/.667 | .418/.353 · .694/.753 · .396/.422 · .422/.452 · .691/.749 | .392/.107 · .419/.404 · .540/.619 · .637/.178 | .427/.424 · .378/.427 · .573/.664 · .354/.416 · .673/.768 · .599/.582 · .527/.137 |
| (a) ponta do fio que acaba NO vermelho | [239,188,188] (o vermelho pálido) | [244,208,208] (idem) | — | [179,90,102] lum .179 (vermelho dos lados .174) |
| (a) **junção** fio×vermelho, Δlum vs o vermelho dos lados | −.054 · **+.071** · −.033 · −.047 · −.007 | −.229 · −.173 · −.100 · −.261 · −.064 · −.221 · −.101 | −.038 · −.031 · −.022 · −.019 · −.021 | −.077 · −.078 · −.037 · −.096 · −.090 · −.042 · −.051 · −.073 |
| (a) **orla de baixo** E (azul no branco): escuro 3 de baixo ÷ miolo · lum | 1,50 · .503 vs .648 | 1,93 · .319 vs .594 | 1,62 · .235 vs .454 | 1,72 · .163 vs .410 |
| (a) orla de baixo D (dentro do vermelho): a linha mais escura, mediana | y 423 [111,129,165] lum .218 (miolo .672) | y 394 [64,80,128] lum .083 (miolo .467) | y 423 [97,114,155] lum .169 (miolo .636) | y 395 [40,62,117] lum .052 (miolo .301; vermelho só .17) |
| **(b) fronteira** esq.: pico de lum a ±20 px acima dos DOIS patamares | **+.066** ([212,200,209], lilás) | +.007 | +.019 | +.003 |
| (b) fronteira dir. | −.074 (sem pico) | sem troca R−B | −.128 | sem troca R−B |
| **(c) pontilhado** N = 32, interior (erodido 2 px), todas / só MAIS CLARAS — E azul só | 0,00 / 0,00 % | 0,02 / 0,00 | 0,00 / 0,00 | 0,00 / 0,00 |
| (c) idem, D azul/vermelho | 0,10 / 0,00 | 1,34 / 0,03 | 0,00 / 0,00 | 0,95 / 0,00 |
| (c) N = 16 na máscara INTEIRA, E (a franja de grão de papel da borda) | 3,72 % | 2,85 | 2,76 | 3,05 |
| (c) só mais claras, tela inteira, N 16 · 32 · 48 | 0,19 · 0,04 · 0,00 | 0,30 · 0,13 · 0,01 | 0,03 · 0,00 · 0,00 | 0,12 · 0,01 · 0,00 |
| **(d) fora da gama e mais clara** (sRGB, tetraedro W·Rd·Bl·Rd⊙Bl), tol 0,03 · 0,10 | 0,036 · **0,000 %** | 0,083 · **0,000** | 0,279 · **0,000** | 1,491 · **0,000** |
| (d) das quais CIANO (coord. do vermelho < −0,10) | 0 | 0 | 0 | 0 |

**O controlo da (d)** (o mesmo teste nas capturas do dono, sem a UI): fora da gama a tol 0,10 = **10,3 · 11,0 · 14,5 ·
19,3 %**, das quais ciano 10,2 · 11,0 · 14,5 · 19,3 % (as quatro capturas pela ordem 1–4; pior célula da 4:
[125,195,202]). ⚠️ As capturas são do ecrã com zoom (células em blocos) e a tinta delas assume-se [20,50,110]: a (c)
NÃO se mede nelas (a mediana 3×3 cai dentro de um bloco: 0,00 %).

⚠️ A (d) por canal contra as tintas PURAS não serve: a aguada do Rebelle é a cor misturada com BRANCO numa fracção
constante nos três canais **em sRGB** (o azul pálido [150,160,188] = 0,553 · 0,537 · 0,538 do caminho do [20,50,110] ao
branco; em luz linear não fecha — com o tetraedro em luz linear o próprio Rebelle sai 99 % «fora»). Por isso o tetraedro
é em sRGB codificado, com o vértice Rd⊙Bl calculado em luz linear.

## O que o Rebelle faz em cada sintoma

1. **Pontas dos fios.** Nenhum ponto claro nem ciano. A ponta que acaba no branco tem a cor do próprio fio (cinza-azul,
   na recta azul–branco); escurece em 11 de 20, fica igual (±0,02 de lum) em 4, e clareia só onde o fio afina no fim (o «corpo» era a
   parte grossa). O fio que acaba NO vermelho desaparece nele: a ponta toma a cor do vermelho (molhado: o vermelho
   pálido; seco: púrpura [179,90,102] com a mesma lum do vermelho dos lados). Ao ATRAVESSAR o vermelho, o fio escurece-o
   em 24 de 25 junções (seco −0,04 a −0,26 de lum: púrpura-escuro, a velatura); a única que clareia (+0,071, `mol`) é
   rosa — a água do fio a diluir o vermelho molhado — e está dentro da gama.
2. **Orla de baixo.** ESCURECE sempre: no branco, 1,50–1,93× o escuro do miolo; dentro do vermelho, uma linha AZUL-MARINHO
   de 1–3 px (lum 0,05–0,22, a coisa mais escura da zona), por cima do vermelho, com células VERMELHAS e brancas (grão do
   papel) soltas na própria linha. Nenhuma faixa clara.
3. **Fios pixelados / blocos.** O ecrã do Rebelle a 100 % não tem degraus de célula: os fios são linhas de 2–4 px com
   cor contínua; o pontilhado no interior da tinta é 0,00–0,02 % (N 32) no azul sozinho, e no azul/vermelho seco os
   0,95–1,34 % são a linha escura da orla e o grão vermelho dentro dela (só 0,00–0,03 % são MAIS CLAROS que a mediana).
   A franja de 3–4 % (N 16) é a borda com grão de papel (células brancas soltas), em todas as tintas.
4. **Fronteira azul/vermelho.** Sobre o vermelho SECO não clareia (+0,003 · +0,007): o azul por cima cobre/vela e a
   orla dele é escura. MOLHADO sobre molhado há uma banda suave ligeiramente mais clara que os dois patamares (+0,066 com
   a Opacity do preset, +0,019 com 100): é LILÁS ([212,200,209] — menos pigmento dos dois, mais papel), dentro da gama,
   nunca ciano nem mais saturada. Nas linhas de baixo a fronteira é escura (vermelho → púrpura → marinho).

**A barra para o motor:** (d) fora da gama a tol 0,10 = **0 %** (ciano 0 %); (c) só mais claras N 32 no interior
≤ **0,13 %** na tela inteira (≤ 0,03 % na zona azul/vermelho); a orla de baixo e a junção ESCURECEM; a fronteira não
clareia acima de +0,07 de lum e nunca para fora da gama.

Notas: as posições dos fios não se repetem com a semente 7 (doc 06); o vermelho molhado com a tela inclinada desce para a
orla de baixo dele e fica pálido no miolo (`mol` p00 [253,245,245]); com a Opacity do preset o azul D molhado LEVA o
vermelho molhado por baixo dele (as caudas dos fios D saem rosa), com 100 também.
