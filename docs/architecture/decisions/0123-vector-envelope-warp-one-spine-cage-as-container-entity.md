# ADR-0123 — Envelope / Warp: UMA espinha (`sample + fit`), a gaiola é uma ENTIDADE-container, e os gestos são dois

**Status:** proposto · **Data:** 2026-07-16 · **Linha:** `line/Vector`
**Pesquisa:** [`docs/Vector Module/21_pesquisa_envelope_warp.md`](../../Vector%20Module/21_pesquisa_envelope_warp.md) (fan-out, fontes primárias)
**Precede:** [ADR-0121](0121-vector-live-corners-authored-source-cooked-geometry.md) (fonte≠cozido) · [ADR-0122](0122-vector-blend-object-live-virtual-steps-editable-spine.md) (objeto vivo) · [ADR-0110](0110-vector-nodes-are-ecs-entities-one-hierarchy.md) (toda forma é entidade)

---

## Contexto

O [`20_pesquisa_ferramentas_de_artista.md`](../../Vector%20Module/20_pesquisa_ferramentas_de_artista.md) §4 nomeou a
armadilha do warp, listou 4 famílias (FFD/MLS/ARAP/BBW) e **parou de propósito**: *"a família de
algoritmos vale um estudo próprio. Não fazer isso de improviso."* Este ADR fecha esse estudo.

**A armadilha, com precisão:** só transformações **afins** comutam com a avaliação de Bézier. Logo
`for v in verts { v.anchor = warp(v.anchor) }` produz uma curva que **não é a imagem** da original —
e erra *quase* funcionando (forma pouco curva parece certa; o erro aparece quando o envelope curva).
Não é teoria: é um **bug aberto do Inkscape** ([#10547](https://gitlab.com/inkscape/inbox/-/work_items/10547),
2024) contra o LPE de perspectiva deles, e o repro dele **é o nosso gate** (§Aceitação).

**E o caminho ingênuo não é espantalho — gente séria o shipa.** A pesquisa achou, em fonte primária:
o **Blender** achata `BezTriple.vec[0..2]` (alça-esquerda, âncora, alça-direita) num **único array
indistinto** que o lattice deforma — e um core dev está no registro dizendo que *"the little 'Apply
on Spline' button does a terrible job at communicating that"*; o **Rive** deforma âncora e as duas
alças com **3 blends LBS independentes** e **vende o erro como feature** (*"creating a 3D effect"*);
o **Skia** subdivide cúbica em **exatamente 4** (`level = 2` hardcoded, sem tolerância) sob um TODO
de **17 anos** que diz *"let the caller tell us, but that seems like a cop-out"*; e o **Inkscape**
tem o certo e o errado **no mesmo framework, separados por qual virtual a classe sobrescreve**
(`doEffect_pwd2` = composição em S-basis + refit por tolerância; `doEffect` = ingênuo).

**Fazer o ingênuo sabendo é companhia respeitável. O que não se pode é não saber.** E note o que
*todos* eles pagam por não ter tolerância: o "cop-out" do Skia é exatamente o que o Illustrator
(Fidelity), o Cavalry (Quality) e o Inkscape (Add Nodes) **shiparam** — porque é a resposta certa.

---

## Decisão

### 1. A espinha é `sample + fit`, e o kurbo já a tem

Toda deformação não-afim entra por **um** caminho:

```
gesto → warp: R2 → R2 → [ amostrar W(C(t)) + J_W·C'(t) → kurbo::fit_to_bezpath ] → cozido
```

Implementamos `kurbo::ParamCurveFit` (3 métodos: `sample_pt_tangent`, `sample_pt_deriv`,
`break_cusp`; `moment_integrals` tem default) sobre a curva deformada. O ponto é `W(C(t))`; a
derivada é a regra da cadeia `J_W(C(t)) · C'(t)`, fechada para todo mapa que adotamos.

**Por que o kurbo e não código nosso:** a doc do trait diz, textualmente, que ele existe para
*"distortion effects such as perspective transform"*. Está na árvore em `0.13.0`, é a **mesma
instância** que o vello puxa (sem skew), e é Apache-2.0 OR MIT. `accuracy` ≈ distância de **Fréchet**.

**Casa: crate nova `ph2d-vec-envelope`** (`kurbo = "0.13"` + `ph2d-vec-scene`), espelhando o que a
`ph2d-vec-blend` já fez. **A `ph2d-vec-scene` continua pura** (só `serde` + `postcard`) — a convenção
de-facto do módulo, e o motivo de o gate `vello_kurbo_only_in_ph2d_vector` ainda fazer sentido mesmo
nunca tendo sido escrito.

### 2. O deformador é uma **ENTIDADE**, em forma de **CONTAINER**

A gaiola é uma entidade ECS que é **PAI** da arte que deforma — o *warp group* do Affinity. Não é
parâmetro dentro do `VecPath`.

**Placar da pesquisa: 5 de 5 referências modelam o deformador como coisa separada e endereçável;
ZERO usam o modelo de parâmetro.** O Inkscape parece a exceção e não é (o LPE dele é um elemento em
`<defs>` referenciado por id).

**E nós já temos o recibo do modelo de parâmetro:** estado autorado guardado dentro de geometria
derivada **é varrido pelo próximo produtor** — é exatamente por isso que uma Live Shape não tem alça
de raio (ADR-0121 §3). Uma gaiola é ordens de grandeza mais estado autorado que um `f64` por vértice.

**O container faz a pergunta do modo EVAPORAR.** *"Edito a gaiola ou a arte?"* vira **seleção na
Hierarquia**. O Illustrator construiu um modo (`Edit Contents / Edit Envelope`, e é
**mutuamente exclusivo** — a doc deles diz *"you can edit an envelope shape or the enveloped object,
but not both at the same time"*); o Inkscape construiu a tecla `7` com alças que o usuário não acha.
O Affinity construiu um container e não precisou de nenhum dos dois. **Nós já temos o container: a
árvore única do ADR-0110.** Vêm de graça: z, seleção, undo, save, e **nesting = encadeamento**.

**Custo de schema: ZERO.** Componente ECS viaja no `WorldSnapshot`, como `VecBlend`/`VecMorph` — que
não bumparam `VEC_SCENE_SCHEMA_VERSION`. Uma gaiola de tamanho variável dentro do `VecPath` sob
postcard **posicional** seria quebra de schema a cada mudança de topologia.

**Keyframability sai de graça** — a timeline liga por **entidade** (`wire_id` = hash do `Name`). É o
que o lattice do Blender compra ("parenteia num osso"). Gaiola-parâmetro seria inanimável.

### 3. A costura do ADR-0121 **não muda** — o envelope é o 2º consumidor

```
verts autorados → corner_live → deformador₁ → deformador₂ → … → mundo
```

É o `Piecewise<D2<SBasis>> → Piecewise<D2<SBasis>>` do Inkscape: **função pura geometria→geometria**
— é *por isso* que uma pilha de 50 efeitos compõe. Duas cláusulas inegociáveis:

- **`Cow::Borrowed` sobrevive.** Pilha vazia + raio zero = mesmo ponteiro, zero alocação. Foi essa
  propriedade que permitiu ligar o `cooked()` em TODO consumidor sem mudar comportamento.
- **As alças vivem no espaço da FONTE.** O Inkscape diz textual que o knotholder é *"totally
  unaffected by the visible distorted path"*. A gaiola **não é deformada por si mesma**; numa gaiola
  aninhada, a de dentro **é** deformada pela de fora. É [[feedback_derived_coordinate_seed_must_match_sample]]
  de chapéu novo: o afim que leva o dedo do artista até a alça tem de ser o MESMO que o avaliador lê.

### 4. Um mapa, **dois gestos** — e isto responde a pergunta que o handoff mandou fazer ao Enio

**Envelope e puppet warp NÃO são uma feature, e também não são duas features separadas: são UM
pipeline com dois gestos.** O que troca é a função `warp`:

| Gesto | `warp` | Grau da imagem de uma cúbica |
|---|---|---|
| **Preset** (Arc/Flag/…) | mapa fechado dirigido por `Bend %`, **gerando a gaiola** | (via gaiola) |
| **Quad / perspectiva** | homografia (Heckbert, forma fechada: 2 Cramer 2×2, **sem sistema 8×8**) | racional 3 |
| **4 curvas de lado** | **Coons** | **12** |
| **Pinos (puppet)** | **MLS-rigid** (Schaefer 2006) | não-polinomial |

**O puppet NÃO precisa de malha** — e é aqui que a suposição comum (puppet = ARAP = triangulação)
quebra. MLS-rigid é `R2→R2` puro; na forma complexa é multiply-accumulate:

```
S = Σ wᵢ·q̂ᵢ·conj(p̂ᵢ)        f_r(v) = S·(v − p_*) / |S| + q_*     (α = 1)
```

Sem solver, sem fatoração, sem re-malhar a cada edição autorada. **Custo:** o paper mede 2,6–3,8 ms
para **10.000** pontos em hardware de 2006; arte vetorial densificada tem 10²–10³.

> ⚠️ **O contra-sinal, e ele é real — registrado aqui de propósito.** A pesquisa varreu o mercado e o
> resultado é desconfortável: **a pegada inteira do MLS em software criativo é o Warp do Krita.** O
> nicho que o MLS foi *desenhado* para ocupar (warp por handles) está ocupado, em software shipado,
> por **ARAP**: o **OpenToonz Plastic** cita o Igarashi **verbatim no fonte** e o **N-Point do GIMP** é
> ARAP/ASAP (Sýkora). E o rigging 2D inteiro é **LBS** (Spine, Rive, DragonBones). Duas razões
> plausíveis, visíveis na evidência: MLS não tem **acoplamento de rigidez entre regiões distantes**
> (não segura estrutura numa dobra articulada — exatamente o que o ARAP do OpenToonz existe para
> fazer), e não tem **escape por-vértice** — você fica com o que os pinos implicam.
>
> **Isto NÃO derruba a decisão, e a razão é o operando.** OpenToonz e GIMP deformam **malha/pixel**; a
> nossa arte é **path**. O argumento C⁰ (§6) é fatal para nós e inofensivo para eles. Mas a Fatia E
> entra com esta ressalva escrita: **se o smoke mostrar que o gesto quer posar personagem** (membro
> perto do tronco), o MLS **vai** falhar e nenhum parâmetro salva — e aí a decisão é **reaberta**, não
> calibrada ([[feedback_ergonomics_verdict_is_a_design_bug]]).

**O preset só vale primeiro se for GERADOR de gaiola** (é o que o Affinity faz: Arc e Mesh são o
mesmo warp group). Como saco de floats solto não leva a lugar nenhum; como gerador, Quad e 4-curvas
saem quase de graça, e o preset vira **promovível** ("assume a gaiola manualmente").

**A gaiola de 4 lados NÃO limita a complexidade do bordo** — e este é o truque que faz o gesto
funcionar. Um lado da gaiola é uma **sequência de cúbicas**, não uma cúbica: os vértices originais
sobrevivem como **nós invisíveis DENTRO do lado**. É o que a Adobe patenteou (US6271861: *"soft
points (which user can not see or select)"*) e o que o `AIMesh.h` documenta (*"the sequence of
beziers between two vertices is called a segment"*). Um path de 37 âncoras não precisa de 37 cantos:
os 4 cantos **absorvem** o resto. **Sem isto, "gaiola a partir de uma forma desenhada" é impossível.**

**Quando a gaiola vier de uma forma desenhada (Fatia futura), os 4 cantos NÃO saem da bbox alinhada
ao canvas.** A regra da Adobe (bbox + os 4 pontos do bordo mais próximos dos cantos) é **de 1998, e
todo mundo que a usou chama o resultado de inutilizável em forma irregular** — a prática é começar de
um retângulo e deformá-lo. O conserto está publicado e **desimpedido**: Lai, Hu & Martin (SIGGRAPH
2009 §4.1) republicaram a regra da Adobe e acrescentaram **pré-rotação por PCA** (alinhar a caixa aos
eixos principais da FORMA, não aos do canvas) + snap por invariante integral num canto de fato ~90°.
E o escape que os usuários do Illustrator de facto usam: **deixar o artista nomear os 4 cantos.**

### 5. A gaiola do Quad é **convexa por construção** — e isso mata o horizonte

Uma homografia de retângulo para quadrilátero **estritamente convexo** não consegue pôr a linha de
fuga dentro do retângulo. Recusando gaiola não-convexa (o clamp que o LPE do Inkscape já tem), o caso
degenerado fica **inalcançável pelo gesto** — sem clipping, sem epsilon. Gaiola não-convexa é sem
sentido de qualquer forma.

O caso degenerado importa: o CSS Transforms L2 é a única spec que o define, e nomeia o lixo — dividir
por `w` negativo **espelha a geometria pela origem**. *Renderiza*, e por isso é pior que NaN.

### 6. O que **NÃO** fazemos, e por quê

| Rejeitado | Razão |
|---|---|
| **Bézier racional** (o projetivo exato) | Exigiria **peso por ponto de controle**: bump de `VEC_SCENE_SCHEMA_VERSION` + ensinar peso a render, booleana, hit-test, bbox e gradiente. Desproporcional por **um** mapa da família — e o MLS continuaria a exigir fit. |
| **Composição simbólica exata** (DeRose/blossoming/S-basis) | **Medido: é o MESMO problema de aproximação** — compor→reduzir e amostrar→fitar dão erro idêntico a 4 algarismos. Custo não é objeção (896 lerps); **o grau é**: 18 não tem onde morar (nem SVG, nem PostScript, nem `BezPath`). O Inkscape compõe exato e **destrói a exatidão uma chamada depois**, num `LPE_CONVERSION_TOLERANCE = 0.01` marcado `// FIXME`. A única vantagem — limite certificado sem amostragem — nós não consumimos. |
| **ARAP / malha** (Igarashi 2005) | (a) O mapa **É** a malha ⇒ **toda edição autorada invalida a triangulação**, num documento que re-coza sem parar. (b) É **C⁰** por partes-afim ⇒ **quina genuína em cada travessia de aresta**, e a qualidade da curva fica capada pela densidade da malha — degradar exatamente o que arte vetorial vende. (c) Precisa de interior triangulado: path aberto **não tem interior**. |
| **Parâmetro no `VecPath`** | §2. |
| **Slider de α no MLS** | **"α maior = mais local" é FALSO** (medido: α não tem efeito nenhum no campo distante). Um escape que nunca ajuda é enfeite ([[feedback_an_escape_that_never_helps_is_a_design_bug]]). |
| **Handles de segmento do MLS (§3 do paper)** | α travado em 2 (colide com α=1 dos pontos), apêndice de OCR duvidoso, e **nenhuma implementação no mundo shipa**. Somos vetor: pinamos **âncoras**, que é o que a §3 dele foi inventada para fingir. |

---

## Conjunto de aceitação (concreto e CONGELADO — DIRETIVA §5)

1. **Invariância à subdivisão (o gate-mãe).** Partir uma cúbica em duas subcurvas, deformar as duas,
   e o resultado tem de bater com deformar a inteira, dentro da tolerância. **Fixture CURVO** — um
   polígono não exibe o defeito ([[feedback_identical_fixtures_hide_the_tiebreak_you_meant_to_test]];
   a lasca do Build só nascia em curva). Este gate não precisa de implementação de referência nem de
   golden image, e pega o erro de aproximação **e** o lixo do horizonte com uma asserção.

   > ⚠️ **O oráculo mede distância GEOMÉTRICA, nunca no mesmo `t`.** Comparar `A(t)` com `B(t)` no
   > mesmo parâmetro **superestima o erro em ordens de grandeza** — o que domina é deriva de
   > *parametrização*, que é **invisível na tela**. Modelar a aparência, não a regra
   > ([[reference_topic_oracle_discipline]]).
   >
   > ⚠️ **E o controle vermelho/verde é de graça: um mapa AFIM.** Ele tem de voltar em **epsilon de
   > máquina em TODO nível de subdivisão**. Se o gate não ficar verde no afim, ele está medindo a
   > coisa errada — antes de dizer qualquer coisa sobre o não-afim.
2. **Identidade = byte-idêntico.** Gaiola em repouso ⇒ `Cow::Borrowed`, mesmo ponteiro. Com o irmão
   de **presença** ([[feedback_absence_gate_needs_a_presence_sibling]]): "não deforma" fica verde num
   renderer que não desenha nada.
3. **Desvio, NUNCA contagem de pontos.** `assert!(pontos < N)` é gate da *regra do filtro* e fica
   verde pelos motivos errados (foi o `assert!(area > 0.0)` da lasca). Gate = distância máxima
   cozido↔assado, **nos dois sentidos**.
4. **Quina sobrevive ao round-trip.** O `break_cusp` recebe as quinas **autoradas** (nós as
   conhecemos — a doc do kurbo diz que fitar por trechos entre quinas conhecidas é melhor).
5. **MLS: 1 pino = translação pura** (não NaN) e **`v = p_*` não produz NaN** (forma `f⃗_r/μ_r + q_*`,
   nunca a Eq. 8 do paper).
6. **Nenhum contrato congelado encostado.**

## Kill-criterion (declarado ANTES do build)

**Se, depois da 2ª tentativa de otimização, o recook de uma forma de ~200 âncoras sob gaiola curva
passar de 2 ms (p95) na tolerância default, a feature não existe nesta forma.** O escape conhecido é
o padrão do `MorphPlans`: cachear o fit com chave na **geometria + pose + gaiola** (nunca um flag
`dirty` — 2ª fonte de verdade). Se nem com cache fechar, o envelope volta a ser gesto destrutivo
(Apply-only), e o vivo espera GPU.

**Regra two-strikes:** bateu na 2ª reconstrução de topologia, **PARE e prove o modelo** antes da 3ª.

---

## Consequências

**Boas**
- **Batemos o Illustrator no ponto em que os usuários dele reclamam.** A patente do envelope
  (US6919888B1, Perani/Kil, **2001**, e ela *nomeia* o Illustrator) descreve o Fidelity como
  *"introducing additional control points on the original curves **prior to the coordinate
  remapping**, with a **variable frequency of insertion** determined by the user"* — e o `[0..100]`
  representa *"the number of additional anchor points inserted **between the original anchor
  points**"*. Isto é **inserção por CONTAGEM, não fit por erro**, e explica exatamente por que o
  Expand deles cospe pontos. ⚠️ **A patente é de 2001 e patente ≠ código shipado** — o que está
  verificado é a doc da Adobe (*"can add more points to the distorted paths"*, texto inalterado de
  CS6 a 2024) e a ausência total de linguagem de tolerância. Um fit por **Fréchet adaptativo** é
  estritamente melhor que inserção por contagem, e sai da dependência.

  > Curiosidade que confirma a direção: a patente **moderna** da Adobe (US9858701, 2015, o puppet)
  > **é adaptativa, e pelo nosso critério** — *"if the weights ... are constant at each of a
  > triangle's vertices, then the triangle's ideal deformation ... **is itself described by an affine
  > transformation** ... **there is no need to add extra resolution**"*. Refinar onde o mapa **deixa
  > de ser localmente afim** é literalmente a tese deste ADR, escrita pela Adobe 14 anos depois do
  > envelope. **Nós aplicamos isso ao envelope; eles não.**
- **O envelope é o primeiro Live Path Effect** — o item #1 do backlog, e a costura já estava paga.
- Undo, save e keyframes vêm de graça (entidade).
- **Coons custa grau 12, não 18** (achado próprio: a construção restringe o suporte de monômios a
  `max(a+b)=4`) ⇒ menos segmentos que um FFD bicúbico, na mesma tolerância, para sempre.

**Ruins / aceitas**
- **O MLS tem suporte global**: o deslocamento **cresce linearmente com a distância** (medido; e α
  não conserta). Mitigação: **o container É o escopo** — a gaiola contém a arte, e os pinos deformam
  os filhos. O mundo raster sofre aqui porque deforma um *plano de pixels* e tem de inventar a
  fronteira; nós deformamos *paths* e a fronteira já é a seleção.
- **Fold-over ≥ ~90°** de rotação de pino. E **dobra é pior em vetor que em raster**: um fold é um
  contorno **auto-interseccionado**, apontado para a `ph2d-vec-boolean` — é a saga da lasca de novo.
  Registrado como risco; não é bloqueio da Fatia A.
- Uma **Live Shape** não pode hospedar gaiola pelo mesmo motivo do raio (o `recook_into` reescreve
  `verts`). Escape: Convert to Curves. Mesma divisão do ADR-0121.

**As duas armadilhas a resolver NO ADR, não no smoke**
1. **A gaiola é invisível mas está na árvore** ⇒ cai no `RootOrder`/ponto-fixo de z que esta linha já
   pagou (empate em `u32::MAX` desempatado por `Entity::to_bits()`, que o undo TROCA) **e** no
   `settle_origins`-durante-gesto (gaiola sob arrasto é *path em gesto*; tem de entrar na lista de
   ignorados, senão foge do cursor como a caneta fugia). **Decisão:** a gaiola **carrega `RootOrder`
   explícito** e entra no `DERIVED` do gate `settle_skips_every_derived_geometry.rs`.
2. **Regra de um-input-só (Cavalry):** *"an animation curve (keyframes) is considered an input... will
   replace the animation curve meaning any keyframe data will be lost."* A Cavalry é a única
   referência com timeline **e** grafo de nós — a nossa colisão — e a resposta dela é: último a
   escrever ganha, keyframes somem. **Decisão:** na Fatia A a gaiola é **autorada, não dirigida** —
   nenhum nó escreve nela. Quando o Motion quiser dirigi-la, é ADR próprio.

---

## Plano por fatias

| Fatia | Entrega | Estado |
|---|---|---|
| **A** | `ph2d-vec-envelope` + `ParamCurveFit` + o gate de invariância à subdivisão. **Sem UI.** | — |
| **B** | Entidade-gaiola (container) + gesto **Quad** (4 cantos convexos). 1º visível. | — |
| **C** | **Presets** como geradores de gaiola (Arc/Flag/…). | — |
| **D** | **4 curvas de lado** (Coons) — quase de graça depois de B (o Node já edita a gaiola). | — |
| **E** | **Pinos** (MLS-rigid) — mesma espinha, `warp` diferente. | — |

---

## Nota de escopo: o Deform do Painter

Existe um módulo de deformação landado que **não é este**: [`docs/Deform/`](../../Deform/) = Transform +
Liquify do **Painter**, sobre **pixels**, com kernel **inverse-warp** (`out[dst] = sample(dst − D(dst))`)
— Wave 1 e Wave 2 landaram (incl. **grade 4×4 com homografia por célula** e **Distort projetivo**, em
`f32`). O **MLS/puppet do plano dele nunca foi construído** (verificado: `MLS`/`puppet`/`ARAP` não têm
uma linha no repo).

Raster quer o mapa **inverso** (gather de pixel); vetor quer o **direto** (mapear a curva). Problemas
diferentes — mas a **família de campos é a mesma**. Já existem **duas** homografias em `f32` (o nó
`four-point-warp` e o `transform_geom` do Painter); uma terceira em `f64` para vetor é justificável
(precisão e domínio diferentes). **Um segundo MLS não seria** — seriam duas portas para a mesma
pergunta. **Se o puppet raster for construído, o campo deve nascer numa crate isolada compartilhada.**
Isto é **decisão do Enio**: a linha Painter está viva e o `warp/` dela é território dela; **esta linha
não o refatora.**

---

## Alternativas consideradas

- **Deformador como parâmetro do path** (`VecPath.envelope`) — rejeitado: §2 (recook varre o autorado;
  quebra de schema por topologia; inanimável; 2ª representação que Node/marquee/undo não veem).
- **Stack de referências** (Cavalry/Blender/Inkscape: `Vec<Entity>` + setas de reordenar) em vez de
  container — rejeitado: o container reusa hierarquia/z/seleção/undo que já existem, e **nesting já é
  o stack**, sem UI de reordenar.
- **ARAP, MLS com pesos geodésicos/harmônicos, BBW** — rejeitados **por ora**; todos exigem malha. Se
  a Fatia E bater na parede da topologia (membro perto do tronco), a saída **não é** Igarashi: é
  BBW-family, e é ADR próprio. Nota: **"Photoshop Puppet = ARAP" é folclore não-verificado** — a Adobe
  nunca citou o Igarashi, o Igarashi nunca citou a Adobe, e a única família de patentes da Adobe sobre
  deformação 2D por handles (Wampler, 2015/2018) cita **BBW + FAST + CCCP** e trata ARAP como a arte
  anterior *lenta demais* que ela está substituindo.
- **Família meshless boundary-only** (Weber; Chen & Weber 2015/2017) — **localmente injetiva por
  construção**, que é o antídoto do fold e a mais nativa a um documento de paths. **Adiada e sinalizada:
  patenteada** (Technion US8400472 ≈ 2029; Max Planck + Bar-Ilan US11403796 ≈ 2037). Exige avaliação
  jurídica antes de qualquer adoção.
