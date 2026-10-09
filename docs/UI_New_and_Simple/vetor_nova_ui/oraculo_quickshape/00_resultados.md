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
