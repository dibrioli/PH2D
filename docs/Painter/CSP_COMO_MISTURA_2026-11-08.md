# Como o Clip Studio Paint mistura — oráculo corrido (08/11)

> CSP 1.13.2 (avaliação PRO) numa cópia reflink do prefixo, Xvfb `:76`, llvmpipe, xdotool (rato = pressão
> constante, **pressão 1**: todos os «Mix/Alpha P» estão no máximo). Zoom 100 %. Cor do pincel **verde
> (0,160,60)** por Alt+clique; fundo de blocos secos azul `0046BE`, amarelo `F0C800`, vermelho `C81414`, branco.
> Ajustes de fábrica lidos de `EditImageTool.todb` (só leitura, tabela `Variant`). Cópia apagada no fim.
> [M] = medido nos píxeis do ecrã · [D] = dedução.

Folhas: [H1a toques 4×](handoffs/material_2026-11-08/csp_hipoteses_H1a_toques.png) ·
[H1b/c zigue-zague e Pointillism](handoffs/material_2026-11-08/csp_hipoteses_H1bc_ziguezague_pontilhismo.png) ·
[H2 perfis ao longo do traço](handoffs/material_2026-11-08/csp_hipoteses_H2_perfis.png) ·
[H3 aguadas e os 4 pincéis](handoffs/material_2026-11-08/csp_hipoteses_H3_aguadas.png)

## H1 — «o CSP amostra por CERDA, não por carimbo» → **FALSO** (para a cor) [M]

- **Toque único sobre azul|branco, Round mixing (tam. 50):** metade azul `(34,138,136)`, metade branca
  `(98,185,153)`, desvio **0** dentro de cada metade. Resolvendo `pixel = α·P + (1−α)·fundo` nos dois lados dá
  **α = 0,75 e P = (45,161,118)** — a MESMA tinta P nas duas metades. P = **0,65·verde + 0,35·média do que está
  sob o carimbo** (média azul+branco = (128,163,223); R 45/128 = 0,35, B 58/163 = 0,36). Azul|amarelo: a mesma
  conta prevê (31,131,101)/(91,163,54); medido (32,131,102)/(92,163,54).
- **Gouache e Oil paint (pontas de cerda), toque:** cada píxel cai numa reta fundo→P com **dispersão angular
  0,1–0,5°** (300–500 píxeis por lado). Ou seja: **uma cor P por carimbo**; a cerda (imagem da ponta) só modula a
  **cobertura α** píxel a píxel. Gouache azul|branco: α ≈ 0,37, P ≈ (59,157,143) ≈ 0,5·verde + 0,5·média.
- **Zigue-zague Gouache sobre listras de 30 px:** a cor troca listra a listra (sobre azul (12,147,79), sobre
  amarelo (41,162,57); 8 listras alternam sem atraso visível). Não há «manchas grandes» porque o intervalo é
  **3 %** (≈ 1 px a tam. 30): a amostra por carimbo é refeita a cada píxel de avanço.
- **Pointillism:** os pontos sobre o azul saem (19,114,132), sobre o amarelo (147,180,37); dispersão 14–28 por
  canal dentro de cada lado. Coerente com **amostra por carimbo + variação HSV por carimbo 30/20/10** (fábrica);
  separar «por ponto» de «por carimbo» não foi medido (spray 30 % de 80 px = pontos próximos do centro) [D].

**Conclusão H1:** o que parece «por cerda» é (a) a cerda modular só a opacidade e (b) carimbos a 1–3 % de
intervalo. A nossa mancha grande vem de **amostrar raramente** (intervalo largo/reservatório lento), não de
amostrar por carimbo.

## H2 — «o Oil amostra no INÍCIO e decai para a cor do pincel» → **FALSO** [M]

Traço horizontal tam. 30, azul → amarelo → vermelho → branco (perfil RGB em [H2](handoffs/material_2026-11-08/csp_hipoteses_H2_perfis.png)):

| Pincel | sobre azul | sobre amarelo | sobre vermelho | sobre branco | fração do fundo no branco | transição |
|---|---|---|---|---|---|---|
| Oil paint (61/81/87) | (0,126,115) | (90,175,44) | (77,109,50) | (100,198,142) | **0,39** | lenta, ~60–140 px |
| Round mixing (80/50/100) | (0,153,70) | (17,163,56) | (14,150,57) | (18,167,74) | 0,07 | ~40 px |
| Gouache (70/45/15) | (0,143,85) | (46,168,48) | (38,133,52) | (49,179,98) | 0,19 | ≤ 1 largura |
| Thick oil (85/50/60) | (12,150,79) | (29,160,65) | (26,145,65) | (30,163,83) | 0,12 | ≤ 1 largura |
| Paint and apply (80/100/50) | (0,145,82) | (40,167,50) | (34,136,53) | (43,176,93) | 0,17 | ≤ 1 largura |

- Em TODOS a cor **segue o que está por baixo**, região a região, e fica num patamar estável — no branco fica
  MAIS CLARA que o verde e não volta ao verde (Oil: 100,198,142 aos 970 px). Isto é **reamostragem contínua**.
- O patamar ≈ `(1 − Amount)·fundo + Amount·pincel` (Oil 0,39 ≈ 1−0,61; Thick 0,12 ≈ 1−0,85; Paint&apply 0,17 ≈
  1−0,80) [M]; Round mixing (0,07) e Gouache (0,19) desviam-se — a Density (alpha) também entra [D].
- **Color stretch** é o comprimento da memória: Oil (87) leva ~100 px a passar do azul ao amarelo; Gouache (15)
  muda dentro de uma largura de pincel. No início do traço a 1.ª amostra pesa mais (Oil a x=42: (0,104,145),
  mais azul que o patamar (0,126,115)) — o reservatório começa cheio da cor da tela [M].

## H3 — Wet wash, Rough wash, Soft bleed [M]+[D]

- Não são pincéis de mistura por amostra: **Mix 0 / «Use water color» 0** (Soft bleed, Rough wash) ou mistura
  com spray (Wet wash 70/50/10). Os quatro aguados têm **CompositeMode 2** e escurecem como **multiplicar**:
  Wet wash sobre azul (0,47,90), sobre amarelo (75,127,0) — o verde×amarelo; Rough wash (0,31,35)/(59,144,0).
- **Wet wash** (tam. 30, spray 120 dens 4 desvio −50, ponta dupla com spray, Mix P): uma **véu largo e liso**
  muito maior que o tamanho nominal (a tam. 50 cobriu ~250×200 px), sem borda. A dureza 100 é da ponta; a
  maciez vem de **spray denso + ponta dupla + contínuo**, não de dureza baixa → H3 **meio-verdadeiro**: o
  aspeto liso é verdadeiro, a causa é outra.
- **Rough wash:** borda de água r 1 após arrasto; mancha irregular escura (bordas recortadas, spray 20 dens 8).
- **Soft bleed:** borda de água r 2, alpha 25, valor 20, **blur 4**, após arrasto: mancha translúcida com
  orla ligeiramente escura e interior granulado (textura Paper 80).

## Os 4 pincéis pedidos — o que fazem e o que os define (fábrica [M])

| Pincel | Visual (medido) | Ajustes que o definem |
|---|---|---|
| **Watercolor splash** | salpicos dispersos, manchas pequenas com orla, multiplicam (azul (0,66,167)) | tam. 300, contínuo 9,5 %, spray 325 dens 1, **ponta dupla com spray 200**, matiz ±15 por carimbo, textura Sand 15, Opac P·R(40), CompositeMode 2 |
| **Wet wash** | véu largo liso que multiplica | tam. 30, contínuo ~1 %, spray 120 dens 4 desvio −50, **ponta dupla** spray 75 dens 4, Mix 70/50/10 com Mix P, Opac P(10), CompositeMode 2 |
| **Pointillism** | pontos achatados (espessura 30 @45°) com cor do fundo + jitter | tam. 80, intervalo 10, spray 30 desvio −100, Mix 50/50/15, **HSV por carimbo 30/20/10**, Esp T·R |
| **Gouache blender** | só arrasta, não pinta: azul entra no amarelo em leque granulado (amarelo → (225,192,12)) | Mix **0/0**, stretch 20, «Use water color» 1, borda de água r 2, textura Paper 20, intervalo 20, CompositeMode 2 |

## O modelo do CSP (inferido do comportamento)

1. A cada carimbo, amostra a **média da tela sob o carimbo** (uma cor, não por píxel/cerda).
2. Essa amostra entra num **reservatório** que esquece à taxa de *Color stretch* (alto = memória longa).
3. A tinta do carimbo é `P = lerp(reservatório, cor do pincel, Amount)`; a 1.ª amostra enche o reservatório.
4. A ponta (círculo ou cerda) dá só a **cobertura α** por píxel, escalada por *Density*; a cor é única.
5. Intervalo de 1–3 % faz a reamostragem praticamente contínua; aguadas usam spray/ponta dupla + multiplicar.

## Implicações para o nosso «Wet blend»

- **Não** é preciso amostra por texel nem por cerda: amostra **por carimbo** (média sob a ponta) chega, DESDE QUE
  o carimbo ande ~1–3 % do tamanho. Se o nosso blend amostra com passo maior (ou amostra só no início), é aí
  que nascem as manchas.
- **Não** usar «amostra uma vez no início e decai para a cor do pincel»: o CSP nunca volta à cor pura sobre
  fundo colorido; o patamar é `(1−Amount)` de fundo.
- Três controlos, uma grandeza cada: **Amount** (quanto pincel), **Density** (cobertura α), **Stretch**
  (memória do reservatório). A cerda mexe só em α.

## Receita no nosso motor (Digital: `spray_count`, ponta dupla, jitter de cor, grão, taper — §5 do estudo)

| Pincel | Temos | Falta |
|---|---|---|
| Watercolor splash | spray, ponta dupla, jitter de matiz, grão | modo **multiplicar** do carimbo; spray com raio 3× o tamanho; orla de água por mancha |
| Wet wash | spray, ponta dupla, contínuo | multiplicar; espalhamento do spray > tamanho; amostra por carimbo com Amount 70 |
| Pointillism | spray, jitter HSV, espessura/ângulo | **amostra por carimbo** no Digital (cor do fundo no ponto) — hoje o Digital não amostra [D] |
| Gouache blender | — | pincel **sem tinta** (Amount 0): só arrasto do reservatório, stretch 20, grão no α |

⚠️ Limites: rato = pressão 1 (curvas de pressão não exercidas); «por ponto» × «por carimbo» no Pointillism não
separado; Density × Amount não decomposto para Round mixing/Gouache.
