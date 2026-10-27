# W3 P-Knife — Smooth · Smooth 2 · Textured · Stain · Soft · Dry 2 sobre os motores da W3 (27/10)

> Oráculo: Rebelle `Oil & Acrylic › Knife › *` (`fixtures/rebelle_bateria/OIL_AND_ACRYLIC/knife_knife_*`). Método:
> `04_BANCADA.md` + `05_REVISAO_CEGA.md` + `REGRAS_DE_CUSTO.md` (2 rondas, 1 revisor `sonnet` por ronda cobrindo as 6).
> Bancada sempre com `PH2D_EXAME_OLEO=1`. Partida: os candidatos da W2 (`W2_Knife_E4.md`) + os valores da K3a/K7.
> O vizinho da prancha cega foi FIXADO no irmão (P53): Smooth 3 para Smooth, Smooth 2 e Textured; Wax Scrape para
> Stain; Dry para Soft e Dry 2 — pintado à MESMA largura do Rebelle. Nada foi para a fábrica.

## Veredito

| preset | veredito | régua nosso×Reb (3 ruídos) | irmão×Reb | nosso×irmão | revisor R1 · R2 |
|---|---|---|---|---|---|
| **Knife Textured** | **APROVADO** (R1) | **306**/432 | 268 | 211 (distinto) | R1: o nosso «original» 6/6, Rebelle a imitação, Smooth 3 «claramente outro» · R2 (controlo): Smooth 3 «original», Rebelle imitação 4/6; Rebelle nunca escolhido como original |
| **Knife Stain** | **APROVADO** (R2) | 316 → **338**/454 | 236 | 246 (distinto) | R1: Wax Scrape «original» 6/6 com o Rebelle de imitação, o nosso «pontos grossos em grumos» à parte ⇒ cura · R2: nosso e Rebelle no MESMO par nas 6 linhas (Rebelle original 4, o nosso 2, confiança baixa — acaso), Wax Scrape «claramente outro» 6/6 |
| **Knife Dry 2** | **APROVADO** (R1) | **364**/454 | 302 | 368 | R1: o nosso «original» 6/6, Dry «claramente outro» 6/6 · R2 (controlo): Dry «original» com o Rebelle de imitação, o nosso «claramente outro» 6/6. Rebelle nunca original nas 12 linhas; os dois revisores separam o nosso do Dry (12/12) |
| **Knife Smooth 2** | **curado-não-aprovado** | **310**/432 | 282 | **366** (> nosso×Reb) | R1: o nosso «original» 6/6, Smooth 3 «claramente outro» · R2: Smooth 3 «original» 6/6 com o NOSSO de imitação 6/6, o Rebelle à parte («liso limpo sem textura») ⇒ lê-se como irmão do Smooth 3 |
| **Knife Smooth** | **curado-não-aprovado** | 258/454 | **264** | **382** | R1: o nosso «original» 6/6 · R2: Smooth 3 «original» 5/6 com o nosso de imitação 5/6, Rebelle à parte. A régua põe o Smooth 3 (à mesma largura) tão perto do Rebelle como o nosso |
| **Knife Soft** | **curado-não-aprovado** | 234 → 248/432 | 224 | 246 | R1: misto (Rebelle original em 3/6, Dry imitação em 4) · R2: o nosso «original» 5/6 com o DRY de imitação 4/6 ⇒ o par nosso≈Dry (P53) |

(«nosso×irmão» = a régua do nosso contra o irmão pintado à mesma largura: alto = parecidos. Para referência, os
irmãos NO Rebelle: Smooth×Smooth 3 240, Smooth 2×Smooth 3 272, Textured×Smooth 3 298, Stain×Wax 180, Soft×Dry
**344**, Dry 2×Dry 294 — no próprio Rebelle o Soft é quase o Dry.)

⚠️ Os dois revisores tiveram viés oposto e confiança BAIXA em tudo: o R1 chamou o nosso de original em 4 pranchas, o
R2 chamou o irmão de original em 4. A pista comum é o RENDER: o nosso e o irmão saem com a sombra/relevo do Óleo
Real no papel, o Rebelle não — o revisor agrupa os dois renders nossos. Por isso o desempate foi a régua (o nosso
mais perto do Rebelle que o irmão, e distinto do irmão).

## Larguras × Rebelle (gestos A, px; bancada à largura medida)

| preset | recto | curva | espiral | vertical | rampa 0→1 |
|---|---|---|---|---|---|
| Smooth | 10/9 | 12/**20** | 12/**20** | 10/12 | 8/8 |
| Smooth 2 | 42/42 | 43/42 | 36/36 | 45/42 | 42/**32** |
| Textured | 39/39 | 34/34 | 30/29 | 15/8 | 14/12 |
| Stain | 34/35 | 35/36 | 35/35 | 34/35 | 9/16 |
| Soft | **21/7** | 19/23 | 25/28 | 31/36 | 4/5 |
| Dry 2 | 13/13 | 22/23 | 30/34 | 38/37 | 8/6 |

## Valores finais (por cima do candidato W2; gravados em `W2_Knife_candidatos/`, miniatura ainda a da W2)

Família comum (§0.10): os sulcos paralelos e o «aro escuro» da W2 eram as CERDAS do Dabber de base
(`bristleStrength` 2, `bristleHeight` 0,75 — as faixas de depósito por cerda, visíveis com o Óleo Real e na rampa
a abrir em fios) ⇒ **`bristleStrength` 0 · `bristleHeight` 0 em todas as seis**. A textura `Noise` a Size 48 (K3a §4,
a rede abaixo de 1/8 da largura) entra em Smooth, Soft e Dry 2.

- **Smooth**: `pigmentPerDab` 600 → **200** (a largura vai com a tinta: 600 tinha um piso de ~20 px a qualquer raio;
  200 dá os 9–10 px do Rebelle), `knifeRidge` 3 → 0, textura Size 6 → 48. (Ponta redonda: a lâmina ao comprido —
  `dabRotation` 1, ângulo 0, flatten 0,6/0,8 — deu curvas MAIS finas que os rectos, 77–92 contra 86 na régua.)
- **Smooth 2**: `knifeLevel` **1**, `leveling` 0,5 → 1, `levelClamp` 0,2 → 1, `knifeRidge` 8 → 0, `pressureSize` 0,4
  → 0 (a rampa do Rebelle entra larga e transparente); `pigmentPerDab` 2000 fica (abaixo de ~1200 a tinta abre em
  sulcos). Régua: `knifeLevel` 0,3 146 · 1 + ridge 2 146 · **ridge 0 155** (/216, A).
- **Textured**: `knifeLevel` 1, `pressureSize` 0,6 → **1**, `pressureSizeCurve` 1 → **1,5** (os valores da K3a: a
  ponta triangular). `pigmentPerDab` 200 (mais translúcido) desceu a régua 156 → 144 — ficou 400.
- **Stain**: textura `Noise` Size 5 → **96** com Contrast **1** e Brightness **0,42** (o portão abre o papel em pontos
  pequenos e soltos — com a tonalidade neutra 0,5 o portão nunca abria: `paperGate` 2–4 e contraste 3–8 deixavam a
  camada cheia), `pigmentPerDab` 600 → **1200** (pontos escuros), `paperGate` 1,5 → 2,5, `paperContrast` 2,5 → 3,
  `pressureSize` 0,3 → 1 + curva 1,5 (a rampa afina como a do Rebelle). Size 48/64 davam grumos (o «pontos grossos» do R1).
- **Soft**: `hardness` 0,3 → 0,15, `paperGate` 1 → **0,3** (núcleo opaco que cobre o vermelho como o Rebelle no
  «sobre»), textura Size 48; `pigmentPerDab` 500 (o da W2).
- **Dry 2**: textura Size 6 → 48 (e a família das cerdas). Os ensaios de grão anisótropo (`Noise` 6×48, gate 1,5,
  tinta 200) desceram a régua para 102–110.

**Tabelas de largura MEDIDAS** (já não as do Dabber): cada uma é a tabela da W2 × o factor medido em âncoras —
os rectos 5/40/200 a escala 1 e os rectos da própria bancada (raio do motor → largura que o motor pintou), por
interpolação em log do raio. Raio do motor 16,7 → largura: Smooth 27 · Smooth 2 **32** (a K7 mediu ~33) · Textured 33 ·
Stain 32 · Soft 4,8 · Dry 2 8 (as lâminas fixas pintam o lado fino num recto horizontal). São tabelas de 3–7 pontos
medidos, não a lei de 64 raios: a receita da fábrica regera-as (`tabela_de_largura`) quando um aprovado entrar.

## As diferenças que ficam (ditas pelos revisores; ORIGINAL = Rebelle quando ele o achou)

- **Smooth** (não aprovado): o Rebelle pinta as CURVAS e a espiral com o dobro da largura dos rectos (20 contra 9 px,
  mesma pressão); o nosso tem largura constante e, a 9–12 px, fica à distância de régua do Smooth 3 (382). Do nosso
  lado: halo/sombra do Óleo Real no papel; o Rebelle tracejado a 5 px.
- **Smooth 2** (não aprovado): o nosso é «estriado paralelo» com sombra lateral; o Rebelle é «liso limpo sem textura»
  — as estrias que sobram (depósito por faixas, K3a §1: std do miolo ~12 com `knifeLevel` 1) são o que o aproxima do
  Smooth 3. A rampa 0→1 do Rebelle estreita (32 px) e esbate; a nossa entra larga (42).
- **Soft** (não aprovado): no Rebelle os rectos são um fio de 7 px e a espiral/verticais largos e esfiapados; o nosso
  com tinta que cubra tem um PISO de ~21 px no recto (a tinta espalha: 400–800 de carga dão 21–22 px a qualquer raio;
  com 250 desce a 7 mas fica translúcido e textura 28 contra 10). A borda do nosso é mais nítida; o revisor junta-o ao Dry.
- **Stain** (aprovado): o Rebelle mostra faixas verticais regulares no recto de 200 px e ponta triangular mais longa
  na rampa 0→1 (16 contra 9 px); no «sobre» a borda do nosso é mais lisa.
- **Textured** (aprovado): o nosso tem verticais em lâmina com sombra e textura mais escura; o Rebelle é mais pálido.
- **Dry 2** (aprovado): o nosso é mais granulado/pontilhado; o Rebelle tem bordas plumosas e um véu transparente a 200 px.

## Fila de motor (não feita — sem código nesta etapa)

1. **Largura que depende da curvatura** (Smooth; 1.º item): a lâmina do Rebelle Smooth dá 9 px nos rectos e 20 nas
   curvas e na espiral à mesma pressão — a lâmina ao comprido cuja rotação SEGUE o traço com atraso (o `dabRotation`
   Follow roda sem atraso; a ponta redonda dá largura constante). Medido: a ponta redonda 10/12/12/10, a lâmina ao
   comprido flatten 0,6 17/13/16/17 (recto/curva/espiral/vertical) contra 9/20/20/12.
2. **Estrias residuais do depósito** (Smooth 2): com cerdas a 0, `knifeLevel` 1, `leveling` 1 e carga 2000 ficam
   estrias paralelas finas; abaixo de ~1200 de carga abrem em sulcos.
3. **Piso de largura da lâmina fina com tinta cheia** (Soft): carga ≥ 400 → ~21 px no recto horizontal a qualquer raio
   (raio ÷ 2 deu 4 px: salto, não rampa).
4. **A sombra do Óleo Real no papel** à volta de todo traço (o aberto da K8): é a pista que agrupa os nossos renders
   na revisão cega.

## ⚠️ Para o integrador

- O gate `os_candidatos_da_w2_sao_as_suas_receitas` (tool-painter, `familias/oleo_acrilica.rs`) fica VERMELHO até as
  receitas `EM_ESTUDO_W2` das seis facas receberem os valores acima (este agente não toca em `familias/`).
  Aprovados para a fábrica: Textured, Stain, Dry 2 (+ a regeração da tabela de largura e a miniatura).

## Ficheiros

`W2_Knife_candidatos/Knife {Smooth,Smooth 2,Textured,Stain,Soft,Dry 2}.ph2d-brush.png` (valores finais) · pranchas
rotuladas da ronda final `pranchas/W3_Knife_<slug>.jpg` · cegas e chaves `pranchas/{cegas,chaves}/knife_<slug>_W3r{1,2}.*`
· réguas `exames/knife_<slug>_W3r{1,2}_regua.{txt,json}`. Bancada de trabalho (variantes, âncoras, ferramentas
`pinta.sh`/`mk.py`/`refina.py`): scratchpad `p_knife/`. Rondas: 2 revisores `sonnet` (um por ronda, as 6 juntas;
na R2 Smooth, Smooth 2, Textured e Dry 2 entraram como controlo, sem mudança).

## BUG do smoke 27/10 — Knife Textured sobreposta (artefactos a partir do 2.º traço)

**Sintoma (fotos do dono, papel com grão):** degraus, manchas ciano, placas de sombra, «lâminas» deslocadas e bordas
duplas no 2.º e 3.º traço sobrepostos; pequenos cortes verticais claros já no 1.º.

**Reprodução** — sonda `sonda_knife_textured_sobreposta` (`crates/ph2d-tool-painter/src/tool/paint/wetpaint/lamina_sobreposta_tests.rs`):
o preset de fábrica a raio 45, três diagonais sobrepostas (pressão 0,25→0,9→0,3), papel Cold + Relief 0,6, Óleo Real
ligado; régua da espessura no miolo (degrau entre vizinhas ÷ a mediana, cortes, factor) e da foto. Variantes no MESMO
processo: `knifeLevel` 0, `knifeRidge` 0, papel liso, sem a marca K7 / sem a crista K6 (render), Smooth 3, Oily, Stain.

**Mecanismo medido:** a espessura guardava-se como FACTOR `espessura ÷ massa`. A lâmina que nivela
(`lamina::nivelar`, K3a) leva cada célula com tinta à média sob a lâmina — também o VALE do grão (massa ~10, onde o
portão do papel/da textura quase não pousou): factor ~10² (**`kmax` 1,2·10² e 1 161 células > 4 já no 1.º traço**). A
massa que chega DEPOIS a essa célula (o assentar, a água) multiplicava-se pelo factor; a média do dab seguinte apanhava-a,
subia a altura de toda a lâmina, e o ciclo fechava: **factor ∞ no 2.º traço**, espessura 10¹¹–10²¹ × a mediana, foto em
placas negras (a prancha). O papel com grão é a fonte dos vales (sem grão — «liso», o papel sem desenho ao acaso por
célula — também explode: 5,7·10¹⁶ no 2.º).

**A/B (mesmo processo):** `knifeLevel` 0 (= a faca da W2 sem nível) **não tem** o defeito (factor 1, degrau p99 0,22–0,29);
`knifeRidge` 0 ao bit igual à fábrica (o rebordo não entra); tirar a marca K7 ou a crista K6 só no render não muda as
fotos; Knife Smooth 3 e Oily (as facas antigas, sem nível) sobrepostas 3× não explodem (factor ≤ 2,5 do tecto do
depósito). ⇒ o defeito é K3a × a representação da espessura, não K6/K7/K8.

**Cura (round 2, `grid/espessura.rs`):** abaixo da massa que COBRE (`MASSA_QUE_COBRE` = 499,5, a profundidade óptica da
opacidade do motor `1 − 0,998^m`) o excesso de espessura é ABSOLUTO: `espessura = m + (k − 1)·max(m, 499,5)`. Na tinta
que cobre é o factor de sempre (a massa que sai leva a espessura na mesma proporção; o tecto de 3000 do depósito ao bit);
no vale a tinta que chega SOMA-SE. O planalto da K3a fica (o gate `a_faca_nivela_a_tinta_em_pe_num_planalto_…` verde).

| 3 traços, Knife Textured, papel com grão | degrau p99 / máx (÷ mediana) | espessura máx ÷ mediana | foto: escuros | grad p99 |
|---|---|---|---|---|
| antes | 2,9·10⁹ / 1,3·10¹¹ (2.º: NaN) | ∞ | placa negra | — |
| **depois** | **0,34 / 0,79** | **1,7** | 0,0105 | 50,6 |
| `knifeLevel` 0 (≈ W2) | 0,22 / 0,61 | 1,0 | 0 | 43,0 |
| Knife Smooth 3 (aprovada) | 0,25 / 0,72 | 1,3 | 0,0109 | 52,1 |

**⛔ Recusas MEDIDAS (não reconstruir):**

| variante | o que deu |
|---|---|
| A — tecto `k ≤ max(k antes, 1)` (a lâmina só raspa) | sem explosão, mas sem planalto: a média do dab desce a cada passada (T med 930 → 595) e as colunas da lâmina fixa ficam em degrau (p99 1,7); reprova `a_faca_nivela_…_planalto` |
| A + altura CORRIDA da lâmina ao longo do traço (média dos dabs filtrada pelo caminho, um comprimento de lâmina) | pior: catraca para baixo (T med 478, p99 2,5) |
| C — puxar e pesar a média pela cobertura α(m), factor solto | `kmax` 6 → 9 → 23 por traço: a multiplicação continua, só mais devagar |

**Gates:** `tres_tracos_sobrepostos_da_faca_textured_ficam_no_planalto` (tool-painter, o preset de fábrica, 3 traços,
papel com grão: espessura ≤ 4 × a mediana, degrau ≤ 2 × a mediana) e `o_vale_nivelado_soma_a_tinta_que_lhe_chega`
(wet-paint). Ambos VERMELHOS com a lei de antes — a mutação `max(m, MASSA_QUE_COBRE)` → `m` É a lei de antes e reprova
os dois; verdes com a cura. Neutralidade verde por nome: `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` (nenhuma
miniatura muda — nada a regerar), `a_faca_so_com_o_nivel_nivela`, `a_tinta_que_nao_nivela_apaga_a_marca_pela_massa`,
`a_parede_nivelada_fica_viva`, os testes `lamina`/`espessura`/`nivelado` do wet-paint, o módulo `oleo_acrilica`.

**Folha:** `pranchas/BUG_knife_textured_sobreposta.jpg` (antes | depois | Rebelle; 1/2/3 traços). Rebelle fotografado
pela porta WebSocket (o mesmo gesto ×1,7 no documento de arranque, papel RH00 com grão):
`fixtures/rebelle_bateria/OIL_AND_ACRYLIC/knife_knife_textured_tres_sobrepostos_{1,2,3}.png`.

**O que fica (não é este mecanismo — está igual com `knifeLevel` 0, logo na W2):**
- os **cortes verticais finos** do 1.º traço: é a textura `Noise` Tiled 60×2 do preset (riscas verticais presas à tela)
  a fechar o depósito em 1–2 colunas até ~0 na borda (medido: colunas a 0,6× dos vizinhos, a 0 na borda de baixo); como
  é presa à tela, ficam nos MESMOS sítios em todos os traços (as fotos do dono confirmam). O Rebelle tem riscas mais
  pálidas mas não furos. Cura = afinar a textura do preset (ronda de pincel, com régua contra o Rebelle).
- manchas claras no miolo (o brilho em tinta fina sobre o relevo do papel) e as bordas em terraço das sobreposições (o
  Rebelle também deixa degrau nas pontas, mais macio) — com o «relevo sem pigmento nas bordas» que corre em paralelo
  em `oleo_real.rs`.

**Veredito:** o preset FICA na fábrica — os artefactos terríveis (2.º/3.º traço) eram este mecanismo e somem; o que resta
está ao nível da Knife Smooth 3 aprovada.
