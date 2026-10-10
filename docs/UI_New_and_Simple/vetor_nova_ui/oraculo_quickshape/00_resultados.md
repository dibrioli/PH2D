# O oráculo do QuickShape — o Xournal++ CORRIDO sobre traços nossos (2026-10-08)

> **O que é:** o reconhecedor de formas do Xournal++ 1.3.8 (instalado com autorização do dono, 08/10)
> a correr numa tela invisível, por script, sobre 43 traços NOSSOS — cada corrida é uma fixture, cada
> fixture é um gate que ninguém teve de inventar ([`_ComoInvestigarApps/00`](../../../_ComoInvestigarApps/00_o_metodo.md)).
> ⛔ **Ninguém leu o fonte dele**, e não se lê: é GPL-2 e o método proíbe-o. Só a SAÍDA é gravada
> (GPLv2 §0: a saída sobre entradas nossas não é obra derivada).

## §1 — Triagem de licença por ARTEFACTO (`pacman -Qi`, 08/10)

| artefacto | licença | papel |
|---|---|---|
| `xournalpp` 1.3.8 | **GPL-2.0-or-later** | o app — o reconhecedor vive DENTRO dele (não há biblioteca separada) ⇒ **caixa preta** |
| gtk3 · libsndfile | LGPL-2.1 | toolkit, áudio — irrelevantes |
| poppler-glib · gtksourceview4 | GPL | PDF, editor de texto — irrelevantes |
| libxml2 · portaudio · lua | MIT | — |
| libzip | BSD · qpdf | Apache-2.0/Artistic | — |
| `xorg-server-xvfb` | MIT + GPL-2 | a tela invisível (instrumento, não entra no produto) |
| `xdotool` | BSD-3 | o «dedo» sintético (instrumento) |

⇒ Nenhuma porta permissiva para o reconhecedor: **porta-se a LEI medida, nunca o código.**

## §2 — Como se corre

```
cd docs/UI_New_and_Simple/vetor_nova_ui/oraculo_quickshape
python3 -I gera_entradas.py          # 43 traços nossos em entradas/ (semente fixa — determinístico)
bash corre_oraculo.sh                # Xvfb :99 + Xournal++ (HOME isolado) + xdotool → fixtures/
python3 -I resume_fixtures.py        # a tabela da §3 (geometria, não pontos)
```
~4 min. Cada fixture leva no cabeçalho: a entrada e o que ela varia, a versão do oráculo, a
**calibração tela→página** (medida por uma linha de 2 pontos com o reconhecedor DESLIGADO:
`página = tela × 0,42491 + (−73,51, 169,12)` a 177 % de zoom), os pontos de entrada em unidades de
página e os pontos de saída.

**O que foi MEDIDO para o roteiro funcionar** (cada um custou uma corrida): o reconhecedor liga-se
por `Ctrl+1` e corre ao SOLTAR; deixa a forma seleccionada (`Esc`); `Ctrl+Z` desfaz o
RECONHECIMENTO e só o 2.º desfaz o traço; o 2.º `Ctrl+S` grava sem diálogo (⇒ `Ctrl+Shift+S`);
`xdotool --window` é ignorado pelo GTK; um traço com 4 pontos por lado NÃO é reconhecido (a
amostragem tem de ser a de uma mão, ~4 px); `pkill -f <texto>` mata a shell que o lançou (PID).
⚠️ O diálogo de gravação falha às vezes (na 1.ª passagem, `forma_L` e `forma_V`; na 2.ª, nenhum) — o
roteiro tenta 3× e, se falhar, DESFAZ o traço na mesma (sem isto o seguinte lia o traço errado).

## §3 — As LEIS que o oráculo devolveu (43 fixtures, 27 reconhecidas)

| família | entrada (varia) | saída do oráculo |
|---|---|---|
| **linha** | 400–600 px, horizontal, vertical, 30°; tremor ±2 … ±24 px | **2 pontos** (recta ajustada; com tremor ±24 o comprimento encolhe 1,6 %: 250,7 de 254,9) |
| linha | 60 px e 30 px (= 25 e 13 unidades de página) | **cru** — abaixo do `strokeRecognizerMinSize = 40` (unidades de página) |
| linha | flecha de 20 px (quase recta) | **2 pontos** — um arco leve é endireitado (2.ª passagem; a 1.ª estava contaminada, §5) |
| **rectângulo** | rodado 10°, 25°, 45°; tremor ±3, ±8, ±16 px; por fechar 15 e 40 px; quadrado | **5 pontos fechados, os 4 ângulos cravados a 90°**, a rotação preservada |
| rectângulo | por fechar 90 px (= 38 un., 7 % do perímetro) | cru |
| rectângulo | 500×60 px (lado de 25 un.) e 70×50 px | cru — um lado abaixo de 40 un. |
| rectângulo | 400×250 alinhado, SEM tremor, fechado | ⚠️ **cru** — a única anomalia: o mesmo rectângulo com tremor ±3 é reconhecido. Uma mão nunca desenha isto; não se copia |
| **triângulo** | regular; tremor ±6; por fechar 40 px | **4 pontos fechados, ângulos PRESERVADOS** (59°/60°/61° com tremor — não regulariza) |
| **polígono ≥ 5** | 5, 6, 8 lados regulares (raio 200 px) | **CÍRCULO** (149–162 pontos; raio = o circunscrito menos ~5 %: 74,5 / 77,6 / 80,7 un. para r = 85) |
| **círculo** | r 150 e 40 px; tremor ±6; por fechar 40 px | **polilinha de círculo PERFEITO** (resíduo rms 0,000), 128 pontos (34 para r 40), fechada; o centro a ±0,2 un. do nosso |
| círculo | uma volta e meia | cru |
| **elipse** | 2:1 alinhada e rodada 30° | **cru** — o oráculo não tem elipse |
| arco | meio e quarto de circunferência | cru |
| rabisco · zigzag · S | — | cru |

### §3b — 2.ª leva (F7, 08/10): 22 traços que a 1.ª não perguntava

Acrescentados ao FIM do `gera_entradas.py` e sem `rng`: as 43 de cima ficaram byte a byte iguais.

| pergunta | entrada | saída do oráculo ⇒ fronteira |
|---|---|---|
| até onde um quadrilátero é rectângulo? | paralelogramos 3°, 6°, 10°, 15°, 20°, 30°; trapézio | 3° e 6° ⇒ **rectângulo**; 10°+ e o trapézio ⇒ cru ⇒ **ângulo de quina 90 ± (6, 10)°** |
| endireita aos eixos? | linha e rectângulo rodados 2°, 4°, 6°, 8° | 2° e 4° saem **a 0°**; 6° e 8° ficam rodados ⇒ **(4, 6)°** (e a linha com gancho, 0,6°, sai horizontal) |
| mão real | cantos arredondados r 20 e 40 px; começado a MEIO de um lado; que PASSA 60 px do início | **todos crus** — o reconhecedor do Xournal++ só aceita quinas vivas a começar numa quina |
| idem | círculo que passa 10 % da volta | círculo, com o centro puxado 5 un. para a sobreposição |
| idem | linha de 500 px com gancho de 30 px | linha (o gancho não a inclina) |
| idem | elipse 3:2 | cru |

## §3c — A nossa lei ([`ph2d-vec-edit/src/quickshape.rs`](../../../../crates/ph2d-vec-edit/src/quickshape.rs))

Protótipo em Python sobre as fixtures, depois Rust; o gate `the_oracle_fixtures_all_agree_with_the_law_or_with_the_owners_product`
lê as **65** fixtures: onde o oráculo reconhece, tipo igual e geometria a ≤ 1 un. (pior caso medido: 0,89
no triângulo com tremor) e o raio a ≤ 1 %; onde é cru, `None`. **22 excepções**, cada uma com a geometria
do gerador como verdade (auto-consistência): 5, 6 e 8 lados ⇒ **polígono** (Procreate; o oráculo faz
círculo) · elipses ⇒ **elipse** · L, V, zigzag, rectângulo aberto 90 ⇒ **polilinha aberta** (o *polyline*
do Procreate) · paralelogramos ≥ 10° e trapézio ⇒ **quadrilátero como desenhado** · rectângulo perfeito,
cantos arredondados, começado a meio, que passa do início ⇒ **rectângulo** (a mão; o oráculo recusa) ·
círculo aberto e que passa ⇒ **círculo** cujo centro desenha o traço MELHOR que o do oráculo (o dele é o
centróide, que foge do buraco/para a sobreposição; o nosso são os momentos de área da região).

Cada constante foi varrida sozinha sobre as 65 + 12 traços sintéticos de mão: a faixa em que tudo
continua certo vai no doc-comment dela (ex.: fecho `0,05..0,07`, recta `0,04..0,075` de flecha).

## §3d — A 2.ª onda (09/10, depois do smoke do dono: *«sem muita sensibilidade · formas pequenas devem funcionar»*)

As constantes da lei passaram a um `Law` que três sondas varrem NO MESMO PROCESSO
(`cargo test --release -p ph2d-vec-edit --lib measure_ -- --ignored --nocapture --test-threads=1`).

**Formas pequenas** (`measure_small_shapes`; 40 mãos sintéticas por célula, o tremor do `gera_entradas.py`).
Abaixo do tremor a lei FALHA PARA «traço livre» — inofensivo; «outra forma» salta a 8 px e fica no
fundo (1–3 em 40) daí para cima (`measure_what_small_failures_become`) ⇒ **piso 12 px** (era 94):

## tremor ±1 px (média móvel de 5), amostras a 2 px, 40 mãos por célula

| tamanho | rectângulo | triângulo | círculo | elipse 2:1 | linha | zigue-zague | zigue-zague redondo | L | S (não é forma) | meio arco (não é forma) |
|---|---|---|---|---|---|---|---|---|---|---|
| 8 px | 92 % | 98 % | 12 % | 0 % | 100 % | 100 % | 78 % | 100 % | 95 % | 100 % |
| 12 px | 100 % | 100 % | 40 % | 8 % | 100 % | 100 % | 92 % | 100 % | 100 % | 100 % |
| 16 px | 100 % | 100 % | 70 % | 38 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % |
| 20 px | 100 % | 100 % | 95 % | 40 % | 100 % | 100 % | 98 % | 100 % | 100 % | 100 % |
| 24 px | 100 % | 100 % | 100 % | 55 % | 100 % | 100 % | 100 % | 100 % | 98 % | 100 % |
| 32 px | 100 % | 100 % | 100 % | 98 % | 100 % | 100 % | 100 % | 100 % | 95 % | 100 % |
| 40 px | 100 % | 100 % | 100 % | 98 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % |
| 48 px | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % |
| 64 px | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 98 % | 100 % |
| 96 px | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % |
| 160 px | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % |

**O S que virava zigue-zague** (`measure_the_s_curve`): as cristas de um S de mão cabiam nas quinas-faixa
e ele saía polilinha em 8–38 % das mãos, a todos os tamanhos. Recusadas MEDIDAS: a fracção do traço nas
quinas (sobrepõe o chanfro), o termo cúbico do lado (o tremor ondula igual nas formas pequenas), o
cúbico com significância e com piso em px. Separa a PROPORÇÃO quina interior / lado vizinho, só nas
polilinhas abertas:

| quina/lado | fixtures | pior zigue/L ≤48 ±≤1 | pior zigue/L ≥160 | pior zigue/L ≤48 ±2 | pior zigue REDONDO ±≤1 | pior S ≥48 | pior S <48 |
|---|---|---|---|---|---|---|---|
| inf | 0 | 100 % | 100 % | 55 % | 100 % | 62 % | 65 % |
| 1 | 0 | 100 % | 100 % | 55 % | 100 % | 62 % | 65 % |
| 0.8 | 0 | 100 % | 100 % | 55 % | 100 % | 62 % | 68 % |
| 0.6 | 0 | 100 % | 100 % | 55 % | 100 % | 80 % | 85 % |
| 0.5 | 0 | 100 % | 100 % | 55 % | 100 % | 90 % | 90 % |
| 0.4 | 0 | 100 % | 100 % | 55 % | 100 % | 95 % | 95 % |
| 0.3 | 0 | 100 % | 100 % | 55 % | 100 % | 98 % | 95 % |
| 0.2 | 0 | 100 % | 100 % | 52 % | 88 % | 98 % | 95 % |

**A faixa de cada constante** (`measure_the_law_ranges`: as 65 fixtures + os traços de mão a 48–400 px,
uma constante de cada vez). Três estavam na BORDA e foram ao meio: elipse `0,04 → 0,065`, fusão
`25° → 29°`, recorte `0,15 → 0,175`:

| constante | medida | faixa válida contígua |
|---|---|---|
| `corner_px` | 94 | 70.0000 … 200.0000 |
| `corner_of_diag` | 0.2 | 0.1200 … 0.3200 |
| `rdp_of_diag` | 0.04 | 0.0250 … 0.1000 |
| `merge_turn_deg` | 29 | 24.0000 … 44.0000 |
| `bow_max` | 0.055 | 0.0400 … 0.0750 |
| `bow_trim` | 0.1 | 0.0000 … 0.2000 |
| `fit_trim` | 0.175 | 0.1500 … 0.2000 |
| `close_of_length` | 0.06 | 0.0450 … 0.0750 |
| `close_window` | 0.1 | 0.0500 … 0.3000 |
| `right_tol_deg` | 8 | 7.0000 … 10.0000 |
| `axis_snap_deg` | 5 | 4.0000 … 5.5000 |
| `ellipse_rms_max` | 0.065 | 0.0200 … 0.1100 |
| `round_ratio` | 1.1 | 1.0500 … 1.5000 |
| `turn_tol_deg` | 90 | 50.0000 … 300.0000 |
| `open_corner_of_side` | 0.3 | 0.0500 … 0.5500 |
| `min_size_px` | 12 | 0.0000 … 28.0000 |
| test quickshape::measure_tests::measure_the_s_curve ... |

**O parado com folga:** o meio segundo conta desde que a mão ENTROU nos 10 px do toque (`SLOP_PT`),
não desde a última amostra — o tremor de uma mão em repouso passava o passo de 1,5 px do lápis e
reiniciava a contagem.

## §3e — Mais sensível para todas as formas (09/10, 2.º pedido do dono)

As três tolerâncias de «isto é forma» varridas juntas (`measure_sensitivity`) já quase não seguravam:
a folga de fecho não mexe, a da elipse satura em `0,075`. A sonda por constante
(`measure_sensitivity_of_each_constant`) achou quem segura as formas TREMIDAS: a régua do lado recto
(`bow_max`, e o recorte `bow_trim` dela), a da volta inteira do círculo (`turn_tol_deg`). A × B forma a
forma, 80 mãos por célula, média de 24–400 px (`measure_sensitivity_ab`):

| | rectângulo | triângulo | círculo | elipse 2:1 | linha | zigue-zague | L | S lido como forma |
|---|---|---|---|---|---|---|---|---|
| antes, ±2 px | 87,5 % | 93,4 % | 85,6 % | 66,6 % | 97,5 % | 84,4 % | 93,1 % | 0,6 % |
| **depois, ±2 px** | **93,8 %** | **97,5 %** | **87,5 %** | **74,4 %** | **98,8 %** | **90,3 %** | **97,2 %** | 1,9 % |
| antes, ±4 px | 62,2 % | 65,0 % | 60,6 % | 51,2 % | 83,4 % | 58,4 % | 71,9 % | 0,3 % |
| **depois, ±4 px** | **71,2 %** | **71,9 %** | **62,5 %** | **56,2 %** | **90,3 %** | **63,1 %** | **76,6 %** | 1,9 % |

Escolhidos: `bow_max 0,055 → 0,06` (faixa re-medida `0,035–0,085`), `bow_trim 0,10 → 0` (`0–0,175`),
`ellipse_rms_max 0,065 → 0,075` (`0,02–0,11`), `turn_tol_deg 90 → 150` (`50–300`); as 65 fixtures
continuam certas. O preço medido: o S passa por forma em ~2 % das mãos (era 0,6 %) e o zigue-zague de
quinas arredondadas PEQUENO (≤ 48 px) cai a 92 % no pior caso.

## §3f — A linha aberta (09/10, 4.º pedido do dono: *«mais sensível, inclusive com a linha aberta»*)

`measure_open_gaps` (30 mãos por caso, tremor ±1 px, estabilizador): formas fechadas com 4–20 % do
contorno em falta × formas que a mão quer ABERTAS (U fundo 3:1, U, C de ¾, V, L):

| folga | fecha com 8 % aberto | com 12 % | forma aberta fechada à força |
|---|---|---|---|
| 0,06 (antes) | 20 % | 0 % | 0 % |
| 0,10 | 100 % | 20 % | 0 % |
| **0,13** | **100 %** | 20–80 % (o triângulo e a estrela sim; o rectângulo e o círculo não) | **0 %** |
| 0,16 | 100 % | 80 % | **100 % (o U fundo, abertura de 14,3 %)** |

Escolhida **0,13** (e a janela da ponta 0,13). O rectângulo aberto 90 px do oráculo (7,6 %) passa de
«polilinha» a RECTÂNGULO (excepção do dono). E a régua do lado recto `0,06 → 0,065` (A × B, 80 mãos):
rectângulo ±2 px 93,8 → 96,2 %, zigue-zague 90,3 → 94,1 %; o S e o arco passam por forma em 1–2 %
das mãos (a 0,07 o arco chega a 2,5 % — recusado).

## §3g — O ARCO (F7c, 10/10, pedido do dono: *«QuickShape com arcos»*)

O Xournal++ deixa os arcos crus (`arco_meio`, `arco_quarto`) ⇒ é lei do PRODUTO do dono (o Procreate
faz arcos): as duas fixtures passam a excepção em `own()` com a geometria do gerador (25 → 27).
[`quickshape_arc.rs`](../../../../crates/ph2d-vec-edit/src/quickshape_arc.rs): um traço ABERTO que a
lei poligonal não explica (ou que ela lê polilinha) é arco se a curvatura tem UM sinal (a fracção da
viragem CONTRA o sentido, entre 6 cordas iguais, ≤ 0,25) e o círculo de **Taubin** (porte do
`CircleFitByTaubin` de Chernov, 2010 — o de Kåsa encolhe o raio nos arcos curtos) o desenha a ≤ 0,05
do menor de raio e diagonal; mais de uma volta não é arco. ⭐ **Polilinha E arco** (um arco de 60° são
dois lados de flecha no limite da régua do recto — achado pelo gate de mão): ganha a forma mais PERTO
do traço, medida sem o `regular` para o `Shift` nunca trocar o tipo. Sondas em
`quickshape_arc_tests.rs` (`measure_arc*`).

**O preço nas formas antigas** (`measure_arc_ab`, 80 mãos, 24–400 px, A × B no mesmo processo):
**zero** — rectângulo, triângulo, círculo, elipse, linha, zigue-zagues, L e S iguais ao décimo com e
sem arco. O meio arco: 0 % → **100 / 99,7 / 74,7 %** a ±1 / ±2 / ±4 px.

**Os arcos de mão** (`measure_arcs`, o estabilizador, 40 mãos, início ao acaso, os dois sentidos):

| varrido | r 20 | r 40 | r 80 | r 160 | r 400 | (±1 px; a ±2 px) |
|---|---|---|---|---|---|---|
| 30° | 0 % | 25 % | 30 % | 55 % | 68 % | a fronteira com a LINHA (flecha 6,5 % ≈ 30°) |
| 45° | 38 % | 75 % | 100 % | 100 % | 100 % | r 80: 78 % |
| 60° | 52 % | 85 % | 100 % | 100 % | 100 % | r 80: 98 % |
| 90° | 88 % | 100 % | 100 % | 100 % | 100 % | r 20: 65 % |
| 120°–315° | 100 % | 100 % | 100 % | 100 % | 100 % | ≥ 98 % (r 20: 85 % a 120°) |
| 330° | 0 % | 0 % | 0 % | 0 % | 0 % | FECHA em círculo (a folga de 13 % fecha a partir de ~318°) |

As falhas: traço livre, LINHA (os arcos rasos) ou arco com o varrido errado > 10 % (os curtos); «outra
forma» só nos arcos MINÚSCULOS (raio 20 px a 45–90°, raio 40 px a 30°: 15–30 px de traço), que já
eram polilinha antes do arco — o tremor faz a polilinha passar mais perto. Os «não é arco» (S de dois
meios círculos, S de seno, rabisco, volta e meia; 48–400 px, ±0,5–2 px): **0 %** lidos como arco. O
C de ¾ da sonda das folgas: arco **30/30** (antes: traço livre), nunca círculo.

**As faixas** (`measure_arc_joint`, as três juntas — o resíduo e a viragem seguram o rabisco a meias):

| constante | medida | faixa | o que cai em cada ponta |
|---|---|---|---|
| `arc_rms_max` | 0,05 | 0,04 … 0,06 | ↓ o meio arco sem estabilizador ±2 px cai de 99 % a 86 % · ↑ rabisco lido como arco (2,5 %; 10 % a 0,1) |
| `arc_bend_back` | 0,25 | 0,15 … 0,35 | ↓ os arcos pequenos tremidos (75 % a 0,1) · ↑ rabisco (5 % a 0,4) |
| `arc_chords` | 6 | 4 … 8 | ↓ rabisco a 3 · ↑ o tremor de cada corda vira para trás (arcos ±2: 90 % a 12, 72 % a 24) |

**Taubin × Kåsa** (`measure_taubin_against_kasa`, 40 mãos): com tremor ±2 px Kåsa encolhe o raio de um
arco de 30° de raio 80 em −9,8 px e de 45° em −3,5 px; Taubin erra −0,1 a +5 (30°) e ~0 de 45° para
cima. Gate `taubin_does_not_shrink_a_short_noisy_arc_as_kasa_does`.

**Arcos de ELIPSE** (fora do escopo; `measure_elliptic_arcs`, raio maior 160 px, ±1 px, 20 mãos): até
2:1 eles viram arco de CÍRCULO — 90° com desvio máximo de 3 px (1,25:1) a 8 px (2:1); o meio arco pelo
eixo MAIOR com 6–12 px; pelo eixo MENOR (1,5:1 e mais) fica traço livre; a 3:1, 90°, metade sai
polilinha. Também viram arco as espirais de Euler (60–120°) e o «taco» (recta + curva de 90°). ⇒ o
círculo «chega» no sentido de não estragar nada que antes funcionava; se o desvio incomodar o dono, o
próximo degrau é o arco de elipse (a medir) ou apertar o resíduo com um piso em px para o tremor.

**O `regular`** (`Shift` / o 2.º dedo) — decisão técnica, sem medição: o Procreate não corre aqui (é
iPad) e o manual só descreve o 2.º dedo para linha (15°), elipse (círculo) e polígonos (regulares). A
lei escolhida é a da LINHA aplicada ao arco: o varrido em múltiplos de 15° (90° e 180° exactos) e a
corda início → fim a 15°; o início fica onde a mão o pôs e o raio é o ajustado.

## §4 — O que isto decide para o PH2D (e o que NÃO decide)

- **Copia-se a lei:** linha ⇒ 2 pontos; rectângulo ⇒ 4 cantos a 90° com a rotação do traço;
  triângulo ⇒ 3 cantos como desenhados; círculo ⇒ centro e raio; o mínimo de 40 un. (no PH2D: em px
  de tela, a medir); a folga de fecho até ~4 % do perímetro aceita, 7 % recusa; tremor até ±16 px
  num lado de 250 aceita (6 %).
- **A barra sai do VALE medido** (§3 passo 6 do método): o gate compara a NOSSA saída com a do
  oráculo por fixture — tipo igual e geometria a ≤ 1 un. de página (linhas, cantos) ou ≤ 1 % do raio
  (círculos); os «cru» têm de dar `None` no nosso reconhecedor, excepto onde o PRODUTO do dono manda
  mais (abaixo).
- ⛔ **O produto de referência do DONO manda mais que o oráculo** ([memória](../../../../project-memory/feedback_the_owners_reference_product_decides_the_law.md)):
  o QuickShape do Procreate (R4 L165–169) faz **elipses** e **polígonos** de 5+ lados, regulariza com
  um **2.º dedo**, e o gatilho é **desenhar e SEGURAR**, não soltar. Onde o oráculo diz «cru» e o
  Procreate diz «forma» (elipse, pentágono), a lei é a do Procreate e o gate é de auto-consistência
  (a elipse ajustada reproduz os nossos pontos a ≤ 1 %) — sem oráculo, dito no cabeçalho.
- **Repetibilidade:** ver §5.

## §5 — Repetibilidade (duas passagens completas)

(preenchido pelo diff das duas passagens — ver o fim deste ficheiro)

**Medido (duas passagens completas, 08/10):** **40 de 41 fixtures byte a byte iguais** entre as
duas corridas — o oráculo REPETE (a barra pode ser um máximo, não precisa de p95). A excepção foi
`linha_arco_leve` na 1.ª passagem (201 pontos crus): contaminação pela gravação falhada do traço
anterior (`forma_V`), não variância — na 2.ª passagem, com o diálogo a abrir, `forma_L` e `forma_V`
gravaram (**cru**, polilinhas abertas) e a flecha de 20 px virou **LINHA** (2 pontos): o oráculo
endireita um arco leve. As fixtures destes três são as da 2.ª passagem; as outras 40 são idênticas
nas duas. ⇒ 43 fixtures, **27 reconhecidas**.

**2.ª leva (22):** 2.ª passagem completa — **19 de 20** comparáveis iguais byte a byte (uma gravação
falhou). A diferente, `rect_rodado_08`, veio logo a seguir a essa gravação falhada (a contaminação de
cima); uma **3.ª leitura isolada** deu-a igual à 1.ª, e o `trapezio` igual nas duas em que gravou. ⇒ **65
fixtures, 39 reconhecidas.** Duas lições do instrumento, já no roteiro: o caminho temporário tem de ser
CURTO (é digitado no diálogo do GTK) e um `timeout` à volta do roteiro deixava o Xvfb `:99` órfão.
