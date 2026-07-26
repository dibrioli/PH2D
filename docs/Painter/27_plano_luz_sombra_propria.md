# 27 — A tinta não faz sombra em si mesma (e é por isso que ela lê como plástico)

> **Origem:** pergunta do Enio, 2026-07-26 — *"que acha de uma guinada radical: em vez de impasto, por
> que não introduzimos na engine o sculpt do Blender? Seriam resultados mais realistas, forma mais
> precisa de trabalhar e benefício da GPU."*
>
> A avaliação daquela proposta está na **§1** e o veredito é **não**. Mas o pedido por trás dela —
> *resultado mais realista* — tem resposta **melhor e mais barata**, e é o resto deste doc.
>
> Estado: **plano, nada construído.** Perf/latência é o doc [26](26_plano_performance_procreate.md);
> a física do relevo é [15](15_impasto_pesquisa_e_design.md)/[16](16_impasto_plano_implementacao.md);
> os verbos são [18](18_plano_sculpt_relevo.md).

---

## 1. Por que **não** trocar o campo de altura por uma malha

Quatro motivos. O segundo é o decisivo, porque a premissa está **invertida**.

### 1.1 Os verbos do Blender já estão aqui, e os que faltam não portam por GEOMETRIA

`PaintMode::Sculpt` (doc 18) carrega **8 verbos portados**: Smooth · Sharpen · Flatten (plano
**inclinado** por mínimos quadrados — um fit horizontal *cava uma cratera na encosta* em vez de
achatá-la) · Scrape · Fill · Chisel · Layer · Inflate.

E os ausentes estão recusados **com motivo escrito**, não esquecidos:

| não temos | por quê |
|---|---|
| **Relax** / Slide Relax | relaxar redistribui **VÉRTICES**; num campo de altura a grade é fixa ⇒ colapsa em Smooth = **knob morto** |
| **Clay** | **é** Flatten com Offset > 0 — os dois knobs já estão na tela |
| **Clay Strips** | Clay + dab quadrado ⇒ buraco de **PINCEL**, não de verbo |
| **Draw Sharp** | colapsa no **Layer** (nosso motor lê o `pre` congelado ⇒ todo verbo aditivo já é "sharp" por construção) |

A lista curta é a lista **honesta**: exatamente os verbos que precisam de **topologia** são os que não
atravessam, e isso é uma propriedade da representação, não uma lacuna de esforço.

### 1.2 ⚠️ A premissa da GPU está INVERTIDA — e este é o motivo decisivo

**Os brushes de sculpt do Blender rodam na CPU** (PBVH + threading TBB). Sculpting na GPU é WIP de anos
lá e nunca virou o caminho principal; o que é GPU é **rasterizar a malha**.

Uma **grade regular** — o nosso campo de altura — é *muito* mais GPU-friendly que uma malha adaptativa:
é literalmente a forma que um compute shader quer, e **a luz já está no device com paridade byte-exata**
(`ImpastoLightPass`, 2026-07-18: `worst delta 0`, 0 de 16384 bytes diferem, 5 materiais).

Trocar grade por malha adaptativa é **andar para longe** da GPU, não para perto.

### 1.3 O preço é o pipeline 2D inteiro

Uma malha **não** compõe com blend mode, **não** recebe adjustment layer, **não** recebe máscara, **não**
passa pelo solver de aquarela, **não** tem undo por pixel (o delta da U1 é sobre 19 planos canvas-shaped).
Seria um segundo app dentro do app.

### 1.4 *"Forma mais precisa de trabalhar"* não transfere

No Blender a precisão vem de **orbitar a vista** + matcap. **Uma pintura tem um ponto de vista por
definição** — é o que a faz uma pintura. (A §5 abaixo tem uma aproximação barata disto, e é a única parte
não-medida deste doc.)

### 1.5 O que o campo de altura de fato NÃO faz — para o pedido não ficar vazio

**Undercut / overhang** (uma altura por texel não expressa dois) **e silhueta real** (a nossa silhueta é
a da COBERTURA). Se é *isto* que "mais realista" significa, é módulo novo do tamanho do Painter, e é 3D.
Fica **nomeado**, não escondido.

---

## 2. 🎯 A lacuna que EXISTE: **a tinta não faz sombra em si mesma**

Lido no produto (`impasto_light.rs` + `impasto_shade.rs`), o modelo hoje é:

- normal por **diferença central** sobre `h` (cruzando `DEPTH_UNIT_PX = 16.0`),
- **4 lâmpadas**, `N·L` com piso ambiente (*"paint in shadow is dark, not black"*),
- termo de **wrap** (o que faz uma orelha ficar vermelha contra o sol),
- **LUT especular 2D** (`ROUGH_LEVELS 65 × SPEC_LUT 256`), que sobe pronta ⇒ `powf` nunca roda no device.

⚠️ **Não há sombra projetada. Não há oclusão de ambiente. Não há horizon mapping.** As menções a
*"shadow"* no código são todas sobre o **piso ambiente** e o wrap — nenhuma é uma sombra.

**A consequência, em uma frase:** uma **crista não faz sombra no vale ao lado dela**, e a **fenda entre
duas pinceladas não escurece**. Um modelo puramente local (`N·L`) desenha relevo como *inclinação*, nunca
como *obstrução* — e é a obstrução que o olho lê como matéria com espessura. É o fator número um de
*"lê como plástico"* contra *"lê como tinta"*, mais que qualquer verbo de sculpt.

### 2.1 Por que esta é a wave certa, e não a troca de representação

| | |
|---|---|
| é operação **pura de campo de altura** | marcha na direção da luz sobre `h`, nada além de `h` |
| é **embaraçosamente paralela** | um texel não depende do vizinho ⇒ o caso ideal de compute shader |
| **o passe já existe** para hospedá-la | `ph2d-render::ImpastoLightPass`, com o LUT já subindo |
| **o template de paridade já existe** | literais exatos no gate CPU-only + épsilon documentado no `#[ignore]` contra o kernel canônico |
| **não toca contrato nenhum** | é óptica; a óptica é o que já portou |

---

## 3. O desenho

### 3.1 O invariante que manda: **o FOLD não porta, só a ÓPTICA**

A lei do `ImpastoLightPass` (2026-07-18) vale aqui **sem emenda**: quais camadas, em que z-order,
`Add`/`Level`, `impasto_depth`, traço vivo e o **teto de vidro** rodam na CPU e chegam como **3 planos
prontos**. Um shader que re-derivasse o fold seria *a segunda resposta a "como camadas de tinta se
empilham"*, divergindo no único lugar onde ninguém lê um número: **uma screenshot**.

⇒ A sombra lê **`relief` e `cover`, os planos que já sobem**. Zero plano novo, zero canal novo.

### 3.2 ⚠️ E a regra que ela NÃO pode violar

*Relevo sob cobertura zero **não acende**.* A luz pesa por `cover`, e o `Supply` do Conserve (histórico) e
o papel (doc 19) se apoiam nisso. Uma sombra é um **fator multiplicativo sobre a luz que já existe**, e a
luz onde `cover == 0` é zero ⇒ **a sombra é automaticamente muda ali**. Não precisa de caso especial —
mas precisa de **gate**, porque é exatamente o tipo de fato que um shader novo contradiz em silêncio.

### 3.3 W1 — **Sombra própria** (horizon marching)

Para cada texel e cada lâmpada acesa, marchar `k` passos na direção da projeção de `L` no plano do canvas
e perguntar se algum vizinho **obstrui** o raio:

```
h_ray(t)  = h(p) + t · (L.z / |L.xy|) · passo          // a reta que a luz percorreria
obstrui   = max_t [ h(p + t·L̂.xy) − h_ray(t) ] > 0
```

Decisões a tomar (cada uma é um número que a MEDIÇÃO dá, não uma preferência):

- **`k` (alcance em texels).** É o único knob de custo: o passe é `O(área · k · lâmpadas)`. Medir a
  1024²/2048²/4096² e escolher o joelho, como o `STRIDE=10` da física e o `EDGE_SAMPLES=2` do form drag.
- **Sombra macia por CONE** em vez de raio duro: um raio duro sobre uma altura quantizada serrilha, e a
  linha já pagou a lição da borda binária do Inflate (*a altura desvanece suave, a cobertura caía de
  uma vez, e a luz acredita na COBERTURA*).
- **`DEPTH_UNIT_PX` é obrigatória na entrada.** `x` é texel, `h` é carga de tinta — [[feedback_geometry_over_mixed_units_needs_the_consumers_conversion]]:
  toda grandeza **geométrica** sobre `h` cruza a conversão do consumidor, e um ângulo cru inclina o
  plano 16× demais.
- **O teto de vidro é da APARÊNCIA** (`soft_ceiling` mora em `height_at`, não no buffer) ⇒ a sombra lê
  `height_at`, **nunca** `heights` cru. Ler o buffer faria a sombra ver uma inclinação que a luz não vê.

### 3.4 W2 — **Oclusão de ambiente** (a fenda escurece)

O termo que falta ao piso ambiente: hoje ele é **constante**, então uma fenda profunda é tão iluminada
pelo ambiente quanto um platô. AO num campo de altura é o **ângulo de céu visível** — e a aproximação
canônica é *horizon mapping* sobre um punhado de direções, ou seja **o mesmo maquinário do W1**, com as
direções sendo as do hemisfério em vez das das lâmpadas.

⇒ **W2 sai quase de graça se o W1 for escrito como uma função de direção**, e é isso que decide a forma
do código do W1. Escrevê-lo grudado nas 4 lâmpadas é o que tornaria o W2 uma segunda implementação.

### 3.5 W3 — os controles (e a lei do knob morto)

Um card **Shadow** dentro de Lighting (que já é gateado por Impasto): **Strength** · **Length** (o `k`) ·
**Softness** · **AO**. ⚠️ Todos com o **neutro em 0 e byte-identidade no neutro** — a arte de hoje não
pode se mover, e é o gate que prova isso que torna a wave segura de integrar.

---

## 4. Custo, ordem e critério de parada

| # | wave | abre com | cancela se | risco |
|---|---|---|---|---|
| 1 | **W0** medir o alcance | sonda no kernel CPU | — | 🟢 nenhum |
| 2 | **W1** sombra própria (CPU, referência) | W0 | render-and-look reprova | 🟡 serrilhado da altura quantizada |
| 3 | **W1g** a mesma óptica no shader | W1 verde | paridade não fecha | 🟡 paridade |
| 4 | **W2** AO | W1 escrito por direção | custo dobra o passe | 🟡 custo |
| 5 | **W3** controles + neutro byte-idêntico | W1/W2 | — | 🟢 seam |

**Critério de parada, explícito:** o oráculo desta wave é **RENDER-AND-LOOK** (a sonda
`push_look_probe`, o método que fechou o bow wave e a borda do Inflate). Um número de sombra não diz se
a tinta lê como tinta. Se o render não convencer, a wave para e o negativo fica escrito.

---

## 5. A ideia não-convencional (e a única não-medida deste doc)

**Um preview de inclinação de ~15°** — parallax/relief mapping no shader que já lê os planos: o artista
*julga* o relevo fora do eixo por um instante, sem a obra virar 3D. É o benefício de **orbitar a vista**
(§1.4) ao preço de um shader, não de uma arquitetura.

⚠️ **Hipótese, não medição** — ao contrário de tudo acima. Precisa de um render antes de virar plano.

---

## 6. O que este plano deliberadamente NÃO propõe

- **Malha, dyntopo, voxel remesh, SDF.** §1, e o motivo decisivo é a §1.2.
- **Re-derivar o fold no shader.** §3.1.
- **Um plano novo no canvas.** A sombra é função de `relief` + `cover`, que já sobem; um 4º plano
  arrastaria o ciclo de vida inteiro (snapshot no MESMO commit — doc 16 §10.4, a lição que o `mats`
  pagou).
- **Sombra entre CAMADAS.** A luz roda **uma vez, pós-composite**, sobre o relevo já dobrado; sombra
  camada-a-camada seria outra pergunta (e o fold já respondeu a esta).
