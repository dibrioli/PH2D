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
