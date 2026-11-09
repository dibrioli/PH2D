# Pastel — o nosso × Clip Studio · Krita · MyPaint (FASE 1: comparar e diagnosticar, 09/11)

> Pedido do dono (09/11): o Pastel (Paint Mode Digital) «parece spray» e «parece ruim»
> ([foto](../material_2026-11-08_pastel/pastel_hoje.png)); comparar «com os três e ver o mais realista». Nada mudou
> no motor, nos presets nem em `.rs`; nenhuma fixture regravada. [M] = medido · [D] = dedução.

**Folha:** [`folha_pastel.png`](folha_pastel.png) — uma linha por pincel; colunas (a) camada VAZIA sobre o papel
branco · (b) branco opaco · (c) blocos `0046BE|F0C800|C81414`, a 50 %; à direita, a reta da coluna (a) a 200 % e as
réguas. Números: [`metricas.json`](metricas.json) (cada linha também a 50 %, a escala da foto do CSP).

## Critério de pastel REAL (escrito antes de olhar a folha)

1. seco e mate; 2. pousa nos PICOS do papel e deixa os vales à mostra — a textura vem do papel, por isso é a MESMA
ao repassar e coerente ao longo do traço; 3. corpo contínuo, sem pontos soltos; 4. borda macia e quebrada;
5. cobre mais com pressão/repasse (⚠️ não medido: ver «o que não correu»).

## Como cada um correu (traço igual em todos)

Cena = a v3 de 08/11 ([`cena_v3.py`](../material_2026-11-08_revisao/cena_v3.py)): 2400×880, reta · S · zigue-zague ·
toque por fundo, verde `00A03C`, pressão 1, 40 px. `prepara.py <scratch>` gera a cena «volta» (os 12 traços R→L) e
a bateria do Krita/MyPaint (verificado: byte-idêntica à usada). Saídas cruas no scratchpad (não versionadas).

| quem | versão · licença por ARTEFACTO (`pacman -Qi`, 09/11) | comando |
|---|---|---|
| PH2D | `line/PainterWatercolor` `8197021`, preset de fábrica «Pastel» | `PH2D_CSP_CENA=<s>/cena PH2D_V3_SO=Pastel bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --test it sonda_csp_v3_revisao -- --ignored --nocapture` (+ `PH2D_V3_VARIANTES=var_pastel.tsv` para as variantes A/B; o mesmo com `<s>/cena_volta`) |
| Clip Studio | CSP 1.13.2 PRO (avaliação) · proprietário — só se corre | **não corrido hoje**: os painéis vêm da [folha de 08/11](../material_2026-11-08_revisao/folha_Pastel.png) (mesma cena, foto a 100 %, que só sobreviveu reduzida a 50 %) |
| Krita | `krita 6.0.4.1-3.1` · **GPL3** — só se corre, nada vem para o repo | `bash scripts/ph2d-run.sh python3 -I docs/Painter/Pinceis/ferramentas/krita_bateria.py --presets "h) Chalk Grainy,h) Chalk Soft,h) Chalk Details" --bateria <s>/bat/bateria_pastel.json --cru <s>/krita_cru --saida <s>/krita_out --sem-fixtures` (~25 s/preset; volta: `bateria_pastel_volta.json`, Grainy e Soft) |
| MyPaint | `libmypaint 1.6.1-2.1` **ISC** · `mypaint-brushes 2.0.2-2` **CC0** | `BAT=bateria_pastel_12.json SUF=_12 python3 -I mp_raw.py <s> ramon/Pastel_1 deevad/chalk` (o C de [`mypaint_bateria.c`](../../Pinceis/ferramentas/mypaint_bateria.c); guarda a camada RGBA) |

Folha e réguas: `python3 -I folha_pastel.py <s> .` (`carrega.py`, `mede.py`, `coerencia_lib.py`, `simula.py`).

## Ranking de realismo (pelo critério acima)

1. ⭐ **Krita h) Chalk Soft**: o ÚNICO cujo grão é da tela (coerência ida×volta **0,79**), corpo contínuo (cv 0,07,
   zero buracos) e borda quebrada; ressalva: à pressão 1 cobre quase tudo (0,97), quase não mostra vales.
2. **Krita h) Chalk Grainy**: corpo contínuo e borda quebrada, mas o grão fino é sorteado por carimbo (coerência 0,00).
3. **Clip Studio «Pastel»**: névoa clara e contínua (cobertura 0,15, sem buracos), borda macia de 14 px. Lê-se como
   giz muito leve/aerógrafo granulado. Coerência não medida (só a foto).
4. **MyPaint deevad/chalk**: corpo opaco, mas a borda são carimbos redondos que voam; grão sorteado (0,05). Parece
   bolinhas de tinta, não giz.
5. **Krita h) Chalk Details**: linha chapada (cobertura 1,0, cv 0). À pressão 1 é um lápis duro sem textura.
6. **MyPaint ramon/Pastel_1**: manchas redondas translúcidas sobrepostas (borda 23 px). Parece tinta molhada, não
   pastel seco.
— **O NOSSO** fica abaixo de todos: é o único com papel a ZERO dentro do corpo (41 %) e grão de 1 px sorteado.

## Diagnóstico do nosso, MEDIDO [M]

Reta da coluna vazia, x 150–650 (sem as pontas). «miolo» = linhas com perfil ≥ 80 % do pico.

| régua | **nosso Pastel** | **Krita Chalk Soft** | CSP Pastel (50 %) | nosso a 50 % |
|---|---|---|---|---|
| largura a 50 % do perfil | 35 px | 37 px | 32 px | 36 px |
| borda 10→90 % (média dos 2 lados) | 12,5 px | 3,0 px | 14 px | 13 px |
| cobertura média do miolo | 0,26 | 0,97 | 0,15 | 0,26 |
| irregularidade do miolo (desvio/média) | **0,91** | 0,07 | 0,29 | 0,54 |
| miolo com papel a ZERO (k < 10 % da média) | **41 %** | 0 % | 0 % | 5 % |
| píxeis pintados com ≤ 2 vizinhos pintados | 5,2 % | 0 % | 0,2 % | 0,4 % |
| grão (autocorrelação cai a 1/e) | **1 × 1 px** | 3 × 3 px | ≤ 2 px (limite da foto) | 2 px |
| coerência ida×volta (mesmo sítio, 2 corridas) | **0,01** | **0,79** | — | — |

- **Tudo ou nada:** todo píxel pintado do nosso tem k ≥ 0,20 (percentis 1/50/90 = 0,20/0,45/0,60), e 41 % do
  miolo é papel limpo. Um véu fino nunca aparece: cada carimbo pousa PONTOS inteiros. Isto é o spray.
- **O grão não é do papel:** ida e volta no mesmo sítio têm correlação 0,01. O controlo (a ida contra a volta
  deslocada 37 px) também dá ≈ 0. Repassar não preenche os vales do papel: sorteia pontos novos.
- **A receita o diz** [M, [`csp_secos.rs`](../../../../crates/ph2d-tool-painter/tests/it/familias/csp_secos.rs)]: `spray_count 20`, `dab_jitter.particle_px 2`,
  `jitter 0,5`, e o comentário «aqui um spray de grãos de 2 px». O papel da receita (`texture PAPEL_ASPERO`,
  `grain_depth 0,6`) existe, mas o spray afoga-o.
- ⭐ **Variantes no motor REAL, sem regravar nada** (`var_pastel.tsv`, linhas «PH2D variante A/B» da folha). Tirar
  só o spray (`-spray_count -dab_jitter jitter=0`) dá coerência **1,00**, zero buracos e cv 0,11 (grão 0,6) / 0,25
  (grão 1,0): **o papel do motor JÁ é da tela**. Falta-lhe ser pastel: borda de **1 px** (dureza 1, `Sharp`) e o
  papel só MODULA a opacidade, um véu manchado. Não pousa nos picos nem deixa os vales a zero.

## Simulações do conserto (retoque em Python, `simula.py`; nada no motor)

Papel sintético NOSSO tipo Cold Press (`simula.papel_sintetico`, semente 9: dente de ~2–3 px e ondulação de ~8 px). O
«envelope» é a cobertura do nosso desfocada 4 px: quanta tinta o nosso põe em cada sítio, sem o grão.

| simulação | o que muda | cobertura | cv | buracos | borda | coerência | corr. papel |
|---|---|---|---|---|---|---|---|
| **1** — o papel decide ONDE | a MESMA tinta do nosso, mas só nos picos: pinta onde altura > 1 − envelope | 0,29 | 1,47 | 64 % | 21 px | 0,98 | 0,80 |
| **2** — corpo + grão do papel | envelope levado a ~0,9 no miolo, × limiar pela altura (os picos primeiro, vales enchem com mais tinta) | 0,67 | 0,44 | 8 % | 17 px | 0,92 | 0,79 |
| **3** — CONTROLO | o mesmo spray com grão de 1,2 px (a mesma tinta), sem papel | 0,26 | 0,35 | 0 % | 13,5 px | 0,05 | −0,03 |

- **A mais perto do mais realista (Chalk Soft) é a 2.** Tem corpo contínuo, grão preso à tela e borda quebrada. Cobre
  menos (0,67 contra 0,97) e tem a borda mais larga (17 contra 3 px).
- A 1 mostra a lei mais pura («a mesma tinta, só nos picos»). Com a pouca tinta de hoje (Amount 0,3, afinado contra
  o CSP) os buracos ficam grandes e manchados: a lei pede mais tinta, como a 2.
- O controlo 3 explica o CSP: engrossar o grão vira a névoa do CSP (cv 0,35 contra 0,29, borda 13,5 contra 14), mas
  a coerência fica em 0,05. É menos spray, e o papel continua sem entrar.
- [D] Caminho para a FASE 2 (o dono decide): tirar o spray (variante A já o mede no motor real), manter o papel da
  tela e fazê-lo DECIDIR a cobertura por limiar (simulação 2) em vez de só modular. A borda passa a ser macia e
  quebrada pelo mesmo papel, com dureza < 1.

## O que não correu, e porquê

- **CSP não foi corrido hoje.** Sob Wine custa ≈ 1 h, e as fotos a 100 % de 08/11 perderam-se com o scratchpad
  daquela sessão. Os painéis são a folha de 08/11 a 50 %: o detalhe do CSP está borrado e a coerência dele não se mede.
- **Krita perdeu a coluna (a) inteira** (os 4 primeiros traços do documento) e os toques de um ponto. Os Chalk não
  misturam e o Krita pinta numa camada transparente, por isso a coluna (a) da folha é a (b) do Krita. Nos blocos, a
  camada do Krita e do MyPaint é composta por cima dos blocos (o PH2D pinta direto sobre eles). Chalk Details sem «volta».
- **MyPaint a size 12, não 40.** Estes `.myb` nascem com raio de 2 px e espalham 2–3 raios: a 40 px davam manchas de
  90 px. A size 12 a largura fica em 35–40 px.
- **Só pressão 1** (a regra da cena v3; a sonda fixa a pressão 1 no código). O critério 5 (cobre mais com pressão ou
  repasse) não foi medido. Medi-lo exige uma sonda nova (`.rs`), proibida nesta fase.
- **Nenhum correu sobre papel com grão.** A sonda pinta em tela lisa (o «Textured Paper» do New Image pede o app ou
  uma sonda nova), o grão do Krita e do CSP é a textura do próprio preset, e o MyPaint não tem papel. Só as
  simulações usam papel, e é o sintético acima.

## FASE 2 — rodada 2 (09/11): o papel decide a cobertura pela PRESSÃO (só receita; nada no motor)

> Lei do estado da arte (Sousa & Buchanan 1999; Rudolf, Mould & Neufeld 2003/2005): a tinta pousa onde o bastão
> ALCANÇA a altura do papel; pouca pressão = só os picos, muita = os vales também; repassar enche o dente.
> **Folha:** [`folha_variantes.png`](folha_variantes.png) (linhas: nosso hoje · Krita Chalk Soft · 4 variantes · simulação
> do enchimento; colunas: vazio · branco · blocos à pressão 1 · rampa 0,1→1 · 2 passadas na rampa · detalhe 200 %).

**Como correu.** A sonda ganhou `PH2D_V3_PRESSAO=rampa` (0,1→1,0 ao longo de cada traço) e `PH2D_V3_PASSADAS=<n>` (a cena
repete-se por cima); sem elas faz o de antes. `bash corre_lei.sh <scratch>` corre [`var_pastel_lei.tsv`](var_pastel_lei.tsv)
(+ `T=…/var_pastel_lei2.tsv`) em pressão 1 (ida e volta), rampa e rampa ×2 — uma compilação `smoke`, ~25 s por condição.
`python3 -I lei.py <s>` (réguas), `krita_rampa.py <s>` (Krita Chalk Soft na rampa: `prepara_rampa.py` + `krita_bateria.py`),
`folha_lei.py <s> .` (simulação + folha). **A altura do papel** é medida no próprio motor: a variante `papel`
(LinearHeight a d = 1, sem mãos ⇒ cobertura = t) dá o mapa, em ranking. Todas as variantes: `spray_count`, `dab_jitter`
e `jitter` fora; o mesmo `Noise` 48 px do `PAPEL_ASPERO`, `depth_pressure` ligado.

### Pressão 1 (reta da coluna vazia, as réguas da rodada 1) [M]

| linha | modo · d · jitter · dureza · opac. | cobertura | cv | buracos | borda | grão | coerência | corr. altura |
|---|---|---|---|---|---|---|---|---|
| nosso hoje | spray | 0,26 | 0,91 | 41 % | 12,5 | 1×1 | 0,01 | 0,11 |
| **Krita Chalk Soft** | — | **0,97** | **0,07** | 0 % | 3,0 | 3×3 | 0,79 | — |
| ⭐ **H_d08_j07** | Height · 0,8 · 0,7 · 0,85 Smooth · 1 | **0,98** | **0,08** | 0,7 % | 2,0 | 1×2 | **1,00** | 0,13 |
| H_d09_j07 | Height · 0,9 · 0,7 · 0,85 · 1 | 0,99 | 0,02 | 0 % | 2,0 | 1×1 | 0,93 | 0,10 |
| H_j07 / H (d 0,7) | Height · 0,7 · 0,7/1 · 0,85 · 1 | 0,92 | 0,26 | 6,4 % | 2,0 | 2×2 | 1,00 | 0,42 |
| H_o08 | Height · 0,7 · 1 · 0,85 · 0,8 | 0,74 | 0,26 | 6,4 % | 2,0 | 2×2 | 1,00 | 0,42 |
| H_d05 | Height · 0,5 · 1 | 0,49 | 1,01 | 50 % | 9,0 | 3×2 | 1,00 | 0,87 |
| LH (d 0,7) | LinearHeight · 0,7 · 1 · 0,85 · 1 | 0,28 | 0,61 | 8,5 % | 7,5 | 4×3 | 1,00 | 0,99 |
| LH_d09 | LinearHeight · 0,9 · 1 | 0,43 | 0,31 | 0 % | 2,0 | 4×4 | 1,00 | 0,99 |
| SUB | Subtract · 0,7 · 1 | 0,64 | 0,14 | 0 % | 2,0 | 4×4 | 1,00 | 0,99 |

Dureza 0,6 (H_h06, LH_h06) e `pressureOpacity` 0,3 (LH_po03) mudam ≤ 0,01 a cobertura; a dureza 0,6 só alarga a borda
(5,5 px). `LH_g24` (papel de 24 px) não se compara: a altura medida é a do de 48.

### Cobertura × pressão (rampa; núcleo ±5 px da reta, janelas de 50 px) [M]

| linha | p 0,2 | 0,35 | 0,5 | 0,65 | 0,8 | 0,95 | corr. com a altura (0,2 · 0,35 · 0,5) |
|---|---|---|---|---|---|---|---|
| nosso hoje | 0,06 | 0,10 | 0,15 | 0,18 | 0,24 | 0,26 | 0,07 · 0,04 · 0,00 (sobe por QUANTIDADE de pontos) |
| **Krita Chalk Soft** | 0,06 | 0,19 | 0,45 | 0,88 | 0,99 | 1,00 | (sem a nossa altura) |
| ⭐ **H_d08_j07** | 0,05 | 0,14 | 0,30 | 0,58 | 0,81 | 0,97 | **0,57 · 0,82 · 0,82** |
| H_d09_j07 | 0,09 | 0,20 | 0,47 | 0,69 | 0,85 | 0,97 | 0,71 · 0,88 · 0,64 |
| H_j07 | 0,03 | 0,08 | 0,15 | 0,38 | 0,67 | 0,93 | 0,42 · 0,67 · 0,78 |
| H (jitter 1) | 0,00 | 0,02 | 0,03 | 0,19 | 0,59 | 0,92 | — · 0,33 · 0,48 |
| LH_d09 | 0,00 | 0,01 | 0,02 | 0,10 | 0,28 | 0,47 | 0,07 · 0,39 · 0,70 |
| SUB | 0,31 | 0,40 | 0,47 | 0,54 | 0,60 | 0,67 | pinta 100 % dos píxeis em TODA pressão: lei invertida |

- **O Height com `depth_pressure` É a lei** (picos primeiro, vales depois): no H_d08_j07 a fracção pintada sobe
  0,15 → 0,30 → 0,51 → 0,81 → 0,95 → 1,00 e, nas pressões baixas, o que pinta é o ALTO do papel (corr. 0,8). O nosso de
  hoje não tem correlação nenhuma com o papel em pressão nenhuma.
- O LinearHeight é a lei «suave» mas a rampa dele tem a largura da profundidade inteira: nunca enche (0,28 a 0,47 à
  pressão 1) — um véu. O limiar suave de largura curta (`(t − (1 − d))/W`, W ≈ 0,1) **não se exprime por composição**.
- `depth_jitter` 0,7 em vez de 1 adianta a curva para a do Krita (a 0,5: 0,30–0,47 contra 0,45).

### Enchimento: 1 × 2 passadas (H_d08_j07, rampa) [M] e simulação [D]

| | p 0,35 | 0,5 | 0,65 | 0,8 | vales (quartil baixo) a 0,5 · 0,65 · 0,8 |
|---|---|---|---|---|---|
| motor, 1 passada | 0,14 | 0,30 | 0,58 | 0,81 | 0,00 · 0,16 · 0,68 |
| motor, 2 passadas | 0,21 | 0,42 | 0,74 | 0,92 | 0,00 · 0,20 · 0,78 |
| Krita, 1 → 2 passadas | 0,19 → 0,33 | 0,45 → 0,66 | 0,88 → 0,97 | 0,99 → 1,00 | píxeis vazios que a 2.ª pinta: **0 %** |
| sim. `h_ef = h + k·alfa` no PRÓPRIO píxel, k 0,5 e 1 | 0,22 | 0,43 | 0,74 | 0,93 | 0,00 · 0,22 · 0,79 (= o motor) |
| sim. alfa da VIZINHANÇA (1,5 px), k 0,5 | 0,28 | 0,59 | 0,88 | 0,97 | 0,00 · **0,77** · **0,95** |
| sim. alfa da VIZINHANÇA (1,5 px), k 1 | 0,30 | 0,64 | 0,88 | 0,97 | **0,33** · **0,77** · **0,95** |

- **O motor de hoje NÃO enche o dente:** a 2.ª passada pinta exactamente os mesmos píxeis (fracção pintada 0,805 →
  0,807); só escurece o que já tinha tinta (o composto). **O Krita também não** (0 % dos vazios ganham tinta).
- **O enchimento no próprio píxel não faz nada** (k 0,5 = k 1 = o motor): um vale vazio tem alfa 0 e não sobe. O que
  enche é a tinta pousada À VOLTA (o pó dos picos empurrado para o vale): com a vizinhança de 1,5 px, os vales a meia
  pressão passam de 0,00–0,20 a 0,33–0,77, e a curva de 2 passadas (0,30 · 0,64 · 0,88) cai em cima da do Krita ×2
  (0,33 · 0,66 · 0,97).
- [D] **Vale construir** (é o critério 5 — cobre mais com repasse — e é o que nenhum dos oráculos faz): uma camada de
  «altura pousada» por traço, `h_ef = h + k·desfoque(alfa_pousado, ~1,5 px)`, lida pelo Height no lugar de `t`. É motor
  (uma textura de estado por camada), fora desta rodada.

### A receita (H_d08_j07) para `csp_secos.rs`, se o dono aprovar [M]

Tirar `jitter`, `spray_count`, `dab_jitter`; `hardness` 1,0 → **0,85**; `falloff` Sharp → **Smooth**; `grain_depth` 0,6 →
**0,8**; `opacity` 0,6 → **1,0**; e o papel do Pastel com a lei `{"depth_jitter": 0.7, "depth_min": 0.0,
"depth_pressure": true, "mode": 4}` (o resto do `PAPEL_ASPERO` igual; o Crayon continua com o seu). Mãos, `spacing`,
`color_jitter_sat` e `mistura_opaca` iguais.

### Ressalvas

- À pressão 1 o H_d08_j07 é quase chapado (0,98) como o Chalk Soft (0,97), mas os poucos buracos são MANCHAS do `Noise`
  de 48 px (detalhe 200 %), não o grão fino de 3 px do Krita. O grão fino é o próximo eixo (`size` do papel; a régua de
  altura tem de ser medida no mesmo papel).
- O Height é binário por píxel: a meia pressão a borda dos grãos é dura (dentes de 1–2 px). Um limiar suave curto pede motor.
- Na coluna «branco» as nossas variantes saem mais claras que no vazio: é a `mistura_opaca` (Amount 0,5) a apanhar o
  branco; o Krita não mistura.
- A simulação corre só na reta (onde a altura foi medida); na linha da simulação o «detalhe» é a reta NA RAMPA.
- Nenhum papel com grão de verdade (o «Textured Paper») — o papel é o `Noise` do próprio preset.

## FASE 3 — rodadas 3–5 (09/11): as peças de MOTOR, medidas contra o Krita Chalk Soft

> Motor novo em [`lei.rs`](../../../../crates/ph2d-painter-brush/src/texture/lei.rs) (`GrainLaw`), gravado no JSON do
> pincel (`lei.softness`, `lei.tip_depth`; só quando ligados — os 100 de fábrica não mudam, nenhum usa o Height):
> - **`softness` W** — no Height a tinta sobe de 0 a 1 ao longo de W acima do ponto onde o papel encontra a ponta,
>   `clamp((t − (1 − d))/W)`. W 0 = o degrau de sempre (ao bit); W = d = o Linear Height.
> - **`tip_depth`** — no Height a ponta afunda `d·w` e a cobertura é a do PAPEL alcançado (o factor devolve
>   `alcança / w`): a borda do traço quebra-se pelo papel. Desligado = o Height de sempre (ao bit).
> Testes: `a_maciez_do_height_vai_do_degrau_ao_linear_height`, `a_ponta_na_profundidade_quebra_a_borda_pelo_papel`.
>
> Ferramentas: `gera_var_motor.py <3|4|5>` → `var_pastel_motor{,4,5}.tsv`; `T=<tsv> bash corre_lei.sh <s>`;
> `TSV=<tsv> python3 -I lei_motor.py <s>` (a altura de cada variante é a do SEU papel, `papel_<tex>`);
> `folha_motor.py <s> <png> <var>=<rótulo>…`. Réguas novas: **parciais** (fracção de píxeis com tinta abaixo de 85 %
> do alfa máximo local: o grão de borda macia), **grão a meia pressão** (autocorrelação 1/e na rampa), **quebra** (desvio
> da borda do traço à pressão 1, sem tendência). Folha: [`folha_motor.png`](folha_motor.png).

| [M] | larg | borda | quebra | cob p1 | cv | parciais .35/.5 | grão ½p | rampa p .2 · .35 · .5 · .65 · .8 · .95 |
|---|---|---|---|---|---|---|---|---|
| **Krita Chalk Soft** | 37 | 3,0 | 0,39 | 0,97 | 0,07 | 0,86 · 0,86 | 4 px | 0,06 · 0,19 · 0,45 · 0,88 · 0,99 · 1,00 |
| nosso hoje (spray) | 35 | 12,5 | 3,32 | 0,26 | 0,91 | 0,74 · 0,69 | 1 px | 0,06 · 0,10 · 0,15 · 0,18 · 0,24 · 0,26 |
| H_d08_j07 (receita r2) | 38 | 2,0 | 0,05 | 0,98 | 0,08 | 0,01 · 0,01 | 3–4 | 0,05 · 0,14 · 0,30 · 0,58 · 0,81 · 0,97 |
| ⭐ **K1** | 34 | 3,0 | **0,42** | **0,98** | 0,02 | **0,83 · 0,86** | **4 px** | 0,05 · 0,17 · 0,32 · 0,60 · 0,91 · 0,98 |

**K1** = sem spray/jitter/dab_jitter; papel `Noise` 32 px com Contrast 0,4 e Detail 0,3 (3 oitavas: params
`[0.4, 0.5, 0.3, 0.5, 0.5, 0.5]`); lei `{mode 4 Height, depth_jitter 0.8, depth_min 0, depth_pressure, softness 0.3,
tip_depth}`; `grain_depth 1.0`; `hardness 0.75` `Smooth`; `opacity 1.0`; `mao.pressureOpacity 0.3`; o resto da receita igual.

Recusas MEDIDAS (não repetir):
- **Grain (o tipo)** a 32–96 px, mesmo com Detail 0,3: sal de 1 px (o limiar do Height revela as oitavas sub-píxel).
- **Noise 16**: cobertura cai quando a pressão sobe (0,15 → 0,04): papel periódico demais. Noise 24: grão 8 px a ½p.
- **Contraste 0,3–0,35** (K2/K3): a correlação com a altura vai a ≈ 0 e a cobertura a p 0,2 a zero.
- Sem `softness` o grão é binário (parciais 0,01, J_G96s0); sem `tip_depth` a borda é lisa (quebra 0,0, H_G96semT);
  ponta 0,6 quebra demais (0,6–0,8), 0,75 ≈ o Krita, 0,85 quebra pouco (0,17).
- **pressureOpacity 0,85** empalidece o traço inteiro a meia pressão; no pastel quem cobre é o papel (0,3).

[D] **Enchimento do dente (peça 4) NÃO construído:** a 2.ª passada já ganha como a do Krita (K1 p 0,5: +~40 %; Krita
+47 %) e o Krita só ganha tinta nova em vazios a p 0,35. Ir além do Krita pede um plano de «pó pousado» por camada —
o candidato é o `covers` do relevo (já gravado no projeto e no desfazer; um fundo opaco não o escreve), plumbado
por ~20 chamadas do motor até o `DabCtx`. Decisão do dono.

Aberto: a curva de pressão do K1 a 0,5–0,65 fica abaixo da do Krita (0,32/0,60 × 0,45/0,88); no branco e nos blocos
o nosso sai mais pálido (a `mistura_opaca` Amount 0,5 apanha a cor de baixo; o Krita não mistura).

## FASE 4 — o enchimento do dente (10/11, ordem do dono 09/11: «enche o papel paulatinamente em várias repassadas»)

> Motor: `GrainLaw::fill` + [`texture/po.rs`](../../../../crates/ph2d-painter-brush/src/texture/po.rs) (`PoDoPapel`,
> `caixa`) + `tool/paint/po_do_papel.rs`. O Height lê o papel a **`t + fill·pó·(1 − t)`**; o pó é o **alfa da camada no
> pen-down** (o `before` do desfazer) desfocado por três caixas de raio 5 (σ ≈ 5,5 px). Painel: «Tooth fill». Pastel:
> `fill 0.5`. Testes: `o_dente_enche` (2), `o_enchimento_sobe_o_vale_e_e_neutro_sem_po`.
>
> Ferramentas: `var_pastel_fill.tsv` (K1 com fill 0 · 0,5 · 0,75 · 1) pela sonda v3 com `PH2D_V3_PASSADAS=1..5` (pressão 1 e
> rampa); `M=<esta pasta> python3 -I mede_fill.py <scratch> <variantes>`; `folha_fill.py <scratch> <png> <variantes>`.

Rampa, cobertura média da janela por pressão (1.ª → 5.ª passada; a 1.ª é idêntica ao bit em todas):

| fill | p 0,35 | p 0,5 | p 0,65 |
|---|---|---|---|
| 0 (K1) | 0,17 → 0,44 | 0,32 → 0,64 | 0,60 → 0,87 |
| **0,5 (fábrica)** | 0,17 → 0,32 → 0,45 → 0,55 → 0,64 | 0,32 → 0,58 → 0,75 → 0,84 → 0,90 | 0,60 → 0,91 → 0,98 |
| 0,75 | 0,17 → 0,79 | 0,32 → 0,64 → 0,84 → 0,93 → 0,98 | 0,60 → 0,94 → 0,99 |
| 1,0 | 0,17 → 0,91 | 0,32 → 0,69 → 0,89 → 0,98 | 0,60 → 0,95 → 0,99 |

Folha (rampa, linhas = fill 0 · 0,5 · 0,75 · 1; colunas = 1–5 passadas): [`folha_enchimento.png`](folha_enchimento.png).

Recusas MEDIDAS (não repetir):
- **Desfoque de ~1 px e `min(1, t + fill·pó)`** (o plano de 09/11): os números sobem, mas a folha mostra manchas chapadas
  de borda dura — o pó só existe onde já há grão pintado, e somá-lo por igual empurra esses grãos ao opaco; os vales
  (~10 px no Noise 32) ficam vazios. [`folha_enchimento_recusado_1px.png`](folha_enchimento_recusado_1px.png).
- **O `covers` do relevo como plano do pó**: o Digital não o escreve, a borracha do Digital não o limpa e o desfazer
  dele passa pelo diário do relevo do Impasto (guardas de proveniência). O alfa do pen-down já está no desfazer, na
  borracha e no mover da camada. Preço aceite: uma camada OPACA (arte que chegou pintada, não o desenho novo, cuja camada
  nasce vazia sobre o papel) enche o dente já no 1.º traço — a tinta que lá está conta como pó.

Custo: o pó a 4K = 11,1 ms (mín.) · 11,7 ms (mediana) por pen-down, só com o enchimento ligado (`mede_o_po_a_4k`, smoke,
carga 3,7). Sem cache por ladrilho: < 1 quadro.

Rotas: o pó chega à mistura opaca, à por píxel, à de cache e à da rampa. NÃO chega à ponta de píxel (L18), ao Shape de
várias camadas coloridas (`stamp_color`) nem à máscara de stamp em cache (`render_stamp_mask`, Grain View) — com Height +
Tooth fill nessas rotas o controlo não age.
