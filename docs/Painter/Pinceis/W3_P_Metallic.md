# W3 P-Metallic — Metallic Flat · Round · Splats sobre os motores da W3 (27/10)

> Oráculo: Rebelle `Oil & Acrylic › Metallic › {Flat 3, Round 3, Splats}` (`fixtures/rebelle_bateria/OIL_AND_ACRYLIC/`).
> Método: `04_BANCADA.md` + `05_REVISAO_CEGA.md` + `REGRAS_DE_CUSTO.md`. Bancada sempre com `PH2D_EXAME_OLEO=1`
> (árvore com a K7 por comitar). Partida: os candidatos da W2 (`W2_Metallic_exame.md`). Nada foi para a fábrica.
> O vizinho da prancha cega foi FIXADO no irmão de onde cada preset nasce (Flat → Filbert, Round/Splats → Dabber),
> não o mais perto da matriz: a pergunta desta etapa é «é distinto do irmão?» (P53). Escalas: Flat 1,05 · Round 0,825 · Splats 1.

## Veredito

| preset | veredito | régua (3 ruídos) W2 → W3 | revisor cego |
|---|---|---|---|
| **Metallic Flat** | **APROVADO** (ronda 2) | 306 → **321**/454 | r1: achou o oráculo 6/6 (baixa), o nosso como imitação · r2: chamou o NOSSO de original 6/6 e o Rebelle de imitação; Filbert «claramente outro» 6/6 |
| **Metallic Round** | **curado-não-aprovado** (2 rondas) | 274 → 292/454 (Dabber 248) | r1: o nosso «original» 6/6, Dabber a imitação, o Rebelle «claramente outro» · r2: nosso e Dabber «quase indistinguíveis» (cada um original em 2 e 4 linhas), o Rebelle de novo à parte |
| **Metallic Splats** | **APROVADO** (ronda 1 e confirmado na 2) | 298 → **322**/454 | r1: o nosso «original» 5/6, o Rebelle imitação 5/6 · r2: o nosso original 6/6, o Rebelle imitação 6/6; Dabber «claramente outro» |

Os dois aprovados o são por indistinção de baixa confiança (o revisor pôs os dois no mesmo par e errou o lado), com a
régua acima da W2. As diferenças que ele ainda listou ficam escritas abaixo — são as do nosso lado.

## Valores finais (por cima do candidato da W2; já gravados em `W2_Metallic_candidatos/`)

O `roughnessMul` velho saiu: o material está nas chaves da W3 (`metallic`, `roughness` absoluta, `coat`).

- **Flat**: `metallic` 0,6 → 0,2 · `roughness` 0,4 · `coat` 0,3 · `pigmentPerDab` 300 → 80 · `bristleTipRagged` 1,2 → 0 ·
  `bristleStrength` 0,1 → 0 · `smear` 0,8 → 0,2 · `blendForce` 0,8 → 0,3; pincel `hardness` 1, grão `Noise` `Tiled`
  4 px (Contrast 0,5) com `grain_depth` 0,3.
- **Round**: `metallic` 0,6 → 0,3 · `roughness` 0,5 · `emboss` 1,5 → 0,3 · `textureSizeExp` 0,5 (K3b: os grumos crescem com o pincel).
- **Splats**: `metallic` 1 · `roughness` 0,4 · `pigmentPerDab` 50 → 4; pincel `spray_count` 2, `dab_jitter.particle` 0,02,
  `across` 0,5, `opacity` 0,8, `jitter` 0,3, `spacing` 0,25; o pó da W2 fica (`po` 3, `po_alcance` 0,5, `po_px` 1,5).

## O que se mediu (régua da bancada; tudo em `/tmp/.../p_metallic/`, cegas e chaves em `pr/`)

**Flat** — a chave foi a tinta FINA. A ponta que esfarela da K3b (`Noise Tiled` 60, Contrast 1, `grain_depth` 0,1) dá
a borda de régua e a ponta recta, mas com o Óleo Real a tinta de 300 vira GRUMOS altos com sombra larga (régua 216–222,
largura medida 58 contra 42: a régua lê a sombra). Nem `grain_depth` 0, nem `emboss` 0, nem `leveling` 1, nem grão de
15 px tiraram os grumos (220/221/222/218); `pigmentPerDab` 2000 piora (178). Com 100 → 282 e 40 → 288 os grumos somem;
com o material claro (`metallic` 0,2) e menos arrasto do vermelho → 300; com 80 → **321**.

**Round** — `textureSizeExp` 0,5 sobe 274 → 294; `jitter` 0,5/`hardness` 0,4 (borda em lóbulos) desce para 192 e
`jitter_scale` 0,5 para 236; `hardness` 0,8 estreita o traço (27 contra 33). Final r2: `emboss` 0,3 e material mais
claro (292). `bristleStrength` 0,5 desfaz o traço (102).

**Splats** — a nuvem do spray da K5 (`spray` 2, `particle` 0,04, pigmento 15, `across` 1) dá marcas de ~8 px ESCURAS e
cobre o traço a 200 (260–268; largura 50 contra 17, o `across` 1). `across` 0,5 + `opacity` 0,8 + pigmento 5 → 328;
`particle` 0,02 + `spacing` 0,25 → 324; pigmento 8 → 344 (marcas azul-marinho: recusado ao olho); pigmento 4 → **322**,
o tom mais perto. `particle` 0,01 = o mesmo tamanho de marca que 0,02 (324): ver o motor.

## As diferenças que ficaram (o revisor, ORIGINAL × IMITAÇÃO)

- **Flat (aprovado, r2)**: o nosso tem um halo e uma sombra suave à volta e pontas um pouco arredondadas, o Rebelle
  tem borda de régua, ponta quadrada e cor chapada · na espiral e a 200 px o nosso mostra o remoinho e um pouco de
  grumo, o Rebelle é um disco liso de grão fino · no «sobre» o nosso faz relevo e sombra nos cruzamentos, o azul do
  Rebelle cobre o vermelho sem relevo. (r1 tinha: miolo da espiral fechado, 200 px escuro e de borda difusa, mistura com o vermelho.)
- **Round (não aprovado)**: o revisor vê o Rebelle como «contas lisas, densas, com vãos nítidos» e o nosso e o Dabber
  como a mesma família granulosa: o nosso é grumoso e compacto, o Dabber neblina estriada. Diferença escrita: os
  grumos do Rebelle são PEQUENOS, de topo liso e separados por vãos finos e escuros, com a borda a 200 px em lóbulos
  redondos; os nossos são maiores, brilhantes e de topo ondulado, e a borda é salpicada (pontos soltos do `jitter`).
  Nem as duas rondas nem a W2 tiraram o nosso do idioma do Dabber.
- **Splats (aprovado)**: a 200 px os nossos pontos são maiores e mais contrastados que os do Rebelle (miúdos e pálidos,
  com uma 2.ª população rara de pontos médios); o nosso espalha um pouco mais através do traço.

## Fila de motor (não feito — sem código nesta etapa)

1. **Round — vãos finos e fundos entre grumos lisos** (1.º item): nem `textureSizeExp`, nem `emboss`, nem material
   dão as «contas» do Rebelle; com o Óleo Real todo grão vira onda ou cratera (recusa da W2 e da K3b). Pede relevo do
   material por célula (topo liso + vão escuro), o K4 da K3b.
2. **Splats — tamanho mínimo da marca do spray**: `particle` 0,01 e 0,02 dão a MESMA marca (~6–8 px a 200; régua 324 × 324);
   há um piso no raio da marca que não deixa os pontos chegarem aos 2–4 px do Rebelle.
3. **Flat — a sombra do Óleo Real no papel** à volta de um traço liso (o halo que o revisor ainda vê) e os grumos que a
   ponta esfarelada da K3b traz com tinta grossa: a ponta recta da K3b só serve com tinta fina (80).

## Ficheiros

`W2_Metallic_candidatos/Metallic {Flat,Round,Splats}.ph2d-brush.png` (valores finais; miniatura ainda a da W2) ·
pranchas rotuladas da ronda final `pranchas/W3_Metallic_{flat,round,splats}.jpg` · folha W2 | W3 | Rebelle | irmão
(faixa `tamanhos` A) `pranchas/W3_Metallic_folha.jpg`. Rondas: 2 revisores `sonnet` (um por ronda, os 3 presets juntos).

## Smoke do dono 27/10

**Metallic Splats REPROVADO pelo dono** («muito inferior ao do Rebelle, realmente ruim»): saiu da fábrica e voltou a
candidato — `W2_Metallic_candidatos/Metallic Splats.ph2d-brush.png` (o ficheiro que estava na fábrica) e a receita
`METALLIC_SPLATS` em `EM_ESTUDO_W2`. Sem ele saem a isenção do Spacing (`CARIMBOS_ESPACADOS` = só o Dabber 4) e a
cláusula das partículas da `borda_ruidosa()` (só ele a usava no Wet). O Metallic Flat fica.
