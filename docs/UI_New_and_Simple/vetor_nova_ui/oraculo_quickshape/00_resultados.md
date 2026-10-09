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
