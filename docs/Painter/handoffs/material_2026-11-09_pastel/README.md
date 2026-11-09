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
