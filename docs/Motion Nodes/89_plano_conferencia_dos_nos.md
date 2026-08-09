# 89 — PLANO DE CONFERÊNCIA DOS NÓS (o super-upgrade, nó a nó)

**Data:** 2026-08-09 · **Linha:** `line/motion-value` · **Status:** plano, nada implementado.
**Ordem do Enio:** *"nós temos mais de uma centena de nós e o pedido original era revisar um a
um buscando onde estamos aquém dos apps do mercado e buscando chegar ao estado da arte.
Certamente não fizemos isso."* — e, para a execução: *"vc vai levantar vários agentes para
fazer o que vc fez com o emitter (comparar cada nó com os apps de ref e tentar superá-los), vai
documentar e vai planejar o super upgrade do sistema inteiro nó a nó."*

---

## §0 — Por que este plano existe (o que falhou, sem eufemismo)

A varredura do [doc 88 §9](88_plano_parametros_nos_unidades_e_slider.md) declarou-se **fechada**
com um mapa de seis famílias e um veredito para cada. Ela **não era a revisão pedida**:

- **Cobria 54 dos 118 nós.** As outras nove famílias (`field.*` · `force.*` · `fx.*` ·
  `pulse.*` · `sim.*` · `source.*` · distribuição · simulação · cor · streams) nunca tiveram
  linha, e a tabela anunciava *"o veredito de cada família"*.
- **Recusou três famílias em ATACADO** — VALUE (23 nós), ESTRUTURAIS (12), RIG (6) — com
  raciocínio próprio sobre *o que um nó "é"* (*"um `value.unary` é um verbo sobre um número"*),
  **sem uma única comparação com o mercado**. As três recusas ficam **REVOGADAS**.
- **Ignorou sete documentos de pesquisa que já estavam no repo** (Cavalry, Blender GN, C4D
  Fields, Houdini MOPS, Niagara/Stardust, MiniCavalry, editores de UI) e um plano chamado
  [`63_pesquisa_industria_2026_e_plano_estado_da_arte.md`](63_pesquisa_industria_2026_e_plano_estado_da_arte.md),
  cuja §1 diz literalmente *"emitter sem shape/burst/inherit"* e cuja §3 é **uma tabela de
  lacunas nó a nó para 22 nós, já priorizada em P0/P1/P2**.

**A prova, pedida pelo Enio como falsificação e confirmada:** o `motion.emitter` tem **10
params**, origem **pontual** e **UMA lane de aleatoriedade** (`const LANE_ANGLE: u32 = 0`, o
jitter do ângulo) contra ~18 da Cavalry, ~30 do Particular e ~20 do Apple Motion — e o que mais
aparece na tela não é a contagem, é a **variância**: toda partícula nasce com a mesma
velocidade, a mesma vida e o mesmo tamanho.

*A lição de processo, para ficar escrita: um veredito de "família recusada" derivado do que eu
acho que um nó É — em vez de do que a referência FAZ — é uma opinião com formato de medição.*

## §1 — O que JÁ EXISTE e não se refaz

A conferência **começa do doc 63**, não do zero:

| Insumo | O que já entrega |
|---|---|
| [`referencia_pesquisa_cavalry.md`](referencia_pesquisa_cavalry.md) | catálogo completo v2.7.2 com coluna `status vs PH2D` (TEMOS/FALTA) por item |
| [`referencia_pesquisa_niagara_stardust.md`](referencia_pesquisa_niagara_stardust.md) | stack de partículas, Dynamic Inputs, eventos, blocks canônicos |
| [`referencia_pesquisa_houdini_mops.md`](referencia_pesquisa_houdini_mops.md) | POPs com valores de param exatos das páginas oficiais |
| [`referencia_pesquisa_c4d_fields.md`](referencia_pesquisa_c4d_fields.md) | MoGraph + Fields (a referência do nosso `field.*`) |
| [`referencia_pesquisa_blender_gn.md`](referencia_pesquisa_blender_gn.md) | Geometry Nodes 4.x, manual 4.5 |
| [`referencia_catalogo_nodes_minicavalry.md`](referencia_catalogo_nodes_minicavalry.md) | o catálogo que originou o módulo |
| [`referencia_pesquisa_ui_editores.md`](referencia_pesquisa_ui_editores.md) | UX de editor de nós (Nuke/TD/Notch/Houdini) |
| **[doc 63 §3](63_pesquisa_industria_2026_e_plano_estado_da_arte.md)** | **tabela de lacunas para 22 nós, com P0/P1/P2** |
| sonda `param_census` | `118 nós · 420 params · 404 com hint · 116 com unidade · 48 magros` (2026-08-09) |
| as 4 leis do doc 88 | unidade · piso/teto duro · widget certo · **todo param é desenhado** |

⚠️ **A conferência CONFERE o doc 63, não confia nele:** ele é de 2026-07, o catálogo cresceu
desde então (a família `field.*`, `source.shape`, `motion.duplicator`, `value.*` novos), e a
coluna `status vs PH2D` dele pode ter envelhecido nos dois sentidos — item marcado FALTA que
já existe é tão caro quanto o inverso, porque manda construir o que está construído.

## §2 — A unidade de trabalho: UM agente por FAMÍLIA

**Nunca um agente por nó** (118 seriam ingovernáveis e o fan-out RECURSA —
[[feedback_a_research_fanout_recurses_bound_it]]), **nunca um agente para tudo** (o resultado
vira prosa). A partição abaixo é **exaustiva e disjunta**: os 118 nós, uma vez cada.

| # | Família | n | Nós | Referência AUTORITATIVA |
|---|---|---|---|---|
| 1 | **DISTRIBUIÇÃO + EMISSÃO** | 9 | `grid` `fibonacci` `scatter` `lattice` `voronoi` `distribute_curve` `distribute_poisson` `distribute_radial` **`emitter`** | Cavalry Distributions (§B) · Niagara Location/Spawn · Blender *Distribute Points* · Houdini |
| 2 | **FORCE** | 6 | `force.attractor` `buoyancy` `curl` `drag` `vortex` `wind` | Houdini POPs (doc 63 §3 já tem a tabela) · Niagara Forces |
| 3 | **SIMULAÇÃO** | 6 | `boids` `verlet_rope` `soft_body` `collide` `spring` `pin_constraint` | Niagara/Stardust · Houdini DOP · C4D Dynamics |
| 4 | **DEFORMERS** | 7 | `bend` `twist` `spherize` `four_point_warp` `kaleidoscope` `slit_scan` `spline_wrap` | Cavalry Deformers · Blender GN · C4D Deformers |
| 5 | **TRANSFORM** | 6 | `move` `rotate` `scale` `transform` `mirror` `orbit` | Cavalry Behaviours · C4D Effectors · Blender GN |
| 6 | **ANIMADORES** | 9 | `noise` `oscillator` `stagger` `wiggle` `wave` `drive` `expression` `time_remap` `path` | C4D Effectors · TD CHOPs · Cavalry Behaviours · Niagara Dynamic Inputs |
| 7 | **TEMPO / ESTILÍSTICOS** | 5 | `trail` `delay` `strobe` `step` `morph` | AE (Echo, Posterize Time) · Cavalry · C4D |
| 8 | **STREAM / UTILIDADE** | 9 | `sort` `cull` `clone` `mixer` `combine` `duplicator` `look_at` `falloff` `make_point` | Cavalry Utilities · Blender GN (atributos) · Houdini |
| 9 | **COR / APARÊNCIA** | 4 | `tint` `color_ramp` `color_array` `luminance` | Cavalry Color Array/Gradient/Shader · AE |
| 10 | **FIELD** | 5 | `field.box` `combine` `index_range` `radial_sweep` `remap` | **C4D Fields** (a referência de origem) · Cavalry Falloff · MOPS |
| 11 | **FX (raster)** | 3 | `fx.drop_shadow` `glow` `rgb_split` | AE effects · Cavalry Filters (54 itens listados) |
| 12 | **PULSE (eventos)** | 6 | `pulse.beat` `compare` `counter` `on_change` `sample_hold` `threshold` | Niagara Events · TD CHOPs · Cavalry triggers |
| 13 | **SIM.\* (o stack)** | 5 | `sim.collide` `lifetime` `spawn` `step` `zone` | Niagara stack (System/Emitter/Particle) · Houdini solver |
| 14 | **SOURCE** | 2 | `source.object` `source.shape` | Cavalry Shapes · Stardust Model/OBJ · AE layers |
| 15 | **VALUE** | 23 | os 23 `value.*` | Blender GN (Math/Map Range/Mix/Compare) · Niagara **Dynamic Inputs** · TD CHOPs |
| 16 | **RIG** | 6 | `rig.fk` `ik_2bone` `fabrik` `rubber_hose` `skeleton` `skin_deformer` | Rive · Cavalry rig · Blender · Spine |
| 17 | **ZERO-PARAM + DEBUG** | 12 (**7 exclusivos** + 5 já contados) | `integrate` `output` `util.reroute` `reroute_pulse` `reroute_value` `debug.const` `debug.wave` · *(também nas temáticas: `value.switch` `pulse.sample_hold` `motion.combine` `luminance` `make_point` `morph` `sim.zone` `duplicator`)* | — (conferir, não pesquisar) |

**A soma fecha:** 9+6+6+7+6+9+5+9+4+5+3+6+5+2+23+6 = **111** nas famílias 1–16, mais os **7
exclusivos** da 17 = **118**. *Se a sua conta der outro número, a partição tem um nó órfão ou
um contado duas vezes — e um nó órfão é exatamente como 64 deles ficaram sem veredito no doc 88.*

⚠️ **A família 17 é CONFERÊNCIA, não pesquisa:** a afirmação *"zero params é o contrato deles"*
tem de ser **verificada nó a nó** (foi ela que recusou 12 nós em atacado), mas a pergunta é
estreita — *existe alguma referência em que este nó tem controle?* — e a resposta esperada é
não. ⚠️ Alguns nós aparecem em DUAS linhas de propósito (`morph`, `duplicator`, `luminance`,
`make_point`, `sim.zone`): eles são conferidos na família 17 **e** na família temática, porque
"tem zero params" e "faz o que a referência faz" são perguntas diferentes.

⚠️ **A família 16 (RIG) é CONFERIDA, não implementada:** o CLAUDE.md §5 defere rig+skinning
*"pro FIM de tudo"*. Conferir agora custa um agente e evita que o adiamento vire amnésia.

## §3 — O BRIEFING (idêntico para todo agente — é o que faz os 17 resultados COMPOREM)

Cada agente recebe o mesmo texto, com a família trocada. Ele responde, **por nó**:

1. **Params de hoje** — lidos do `MANIFEST`, não do doc (o doc envelhece).
2. **O que a referência tem** — o conjunto de controles do nó equivalente em **pelo menos
   duas** ferramentas, **com citação** (documento do repo + seção, ou URL oficial).
3. **O gap** — o que a referência nomeia e nós não exprimimos.
4. **O teste de EXPRESSIBILIDADE, TENTADO** — para cada gap: *que composição de nós do nosso
   catálogo produziria isto hoje?* A resposta é uma cadeia concreta (`A → B → C`) ou a razão
   pela qual nenhuma existe. **Tentar é obrigatório; afirmar não vale.**
5. **Magro por NATUREZA ou por OMISSÃO** — com o mecanismo nomeado.
6. **Cercas de Chesterton** — `grep` no crate por decisão já registrada antes de propor.
7. **O default que reduz LITERALMENTE** — para todo param proposto, qual valor devolve o
   comportamento de hoje bit a bit.
8. **COMO SUPERAR** — ≥1 capacidade por família que **nenhuma referência tem**, derivada do
   que o nosso substrato torna barato (ver §4.8).

**Formato de saída — uma tabela por família, colunas fixas:**

```
| nó | params hoje | falta (referência CITADA) | exprimível? (a cadeia tentada) | natureza/omissão | P | default que reduz |
```

…mais três blocos curtos: **`SUPERAR:`** (as ideias inéditas), **`CERCAS:`** (as decisões já
registradas que encontrou) e **`O DOC 63 ERROU EM:`** (itens da tabela dele que envelheceram
nos dois sentidos).

## §4 — As LEIS que todo agente honra (cada uma foi paga nesta jornada)

1. **Nunca recuse por raciocínio sobre o que um nó "é".** Recusa exige referência: *"nem
   Cavalry, nem Blender, nem TD dão controle a isto"*. Foi assim que VALUE/ESTRUTURAIS/RIG
   caíram sem exame.
2. **Todo gap tem CITAÇÃO.** Doc do repo + seção, ou URL oficial. Sem citação é opinião.
3. **A expressibilidade é TENTADA.** O `motion.spherize` só virou trabalho porque três
   composições foram tentadas e falharam (`move` leva o centroide junto · `falloff` mascara a
   mistura e não o centro · isca muda a contagem). Uma delas ter funcionado teria matado o
   item — e é isso que a torna um teste.
4. **Magro por natureza × por omissão, com MECANISMO.** O `motion.slit_scan` tem 1 param e está
   certo, porque o eixo é um `motion.sort` a montante, a direção é o `descending` dele e a forma
   da rampa é o campo `falloff` — três mecanismos, nomeados. Sem o mecanismo, "por natureza" é
   preguiça com nome bonito.
5. **Cerca de Chesterton primeiro.** O `motion.strobe` parecia ter 3 params sem widget; o 4º
   canal do grupo de cor é `flash_amount` e **o arquivo já explicava** (o kernel faz
   `a = flash_amount · glow` e compõe por `over`, então ele *é* o alfa). Um agente que não
   grepa propõe desfazer decisões.
6. **Todo default novo reduz LITERALMENTE.** `c + (0,0)` é `c`. Um param cujo default muda a
   arte já autorada não é um param novo, é uma regressão com slider.
7. **Contrato congelado (§6 do CLAUDE.md).** Params são do `MANIFEST` do nó (livres); canal
   novo no registry é **side-metadata** (`param_gates`/`reduces`/`luts`/`hard_max` são o
   precedente); `NodeOp=2`/`OpResolver=1`/`NodeManifest=8` **não se tocam**.
8. **SUPERAR não é retórica — é derivar do que só nós temos.** O doc 63 §1 já lista: deformers
   e grade espacial **100% GPU-resident** · scrub **bit-exato** (GGPO) · **emitter stateless**
   (função pura do playhead — o dos outros re-simula) · **params dirigidos** (doc 58: um param
   é uma aresta) · a família **`field.*`** composável · determinismo cross-OS gateado. *Exemplo
   do que isso destrava:* o `inherit_velocity` do emissor, que nas referências é um acumulador
   de estado, aqui é **forma fechada** — a velocidade do emissor é a derivada de uma função do
   playhead que já temos.
9. **O agente NÃO escreve código, NÃO decide prioridade final e NÃO abre wave.** E **não levanta
   outro agente** — o fan-out não recursa.

## §5 — A VERIFICAÇÃO (o passo que é MEU, não delegável)

Um agente devolve **claims**. Antes de virar trabalho, eu confiro **o fato decisivo de cada
gap que entra em P0/P1**:

- rodo a sonda / grepo o `MANIFEST` (o param realmente não existe?);
- **tento a composição** que o agente diz não existir (foi assim que descobri que `rate` **é**
  animável por param dirigido, e que a minha própria frase *"o emitter não tem portas, logo nada
  é animável"* era **falsa** — um buraco que eu teria reportado ao Enio);
- grepo a cerca de Chesterton.

*Claim que não sobrevive à verificação volta com o motivo escrito, e o motivo entra na tabela —
um gap refutado vale tanto quanto um confirmado, porque impede a próxima varredura de o propor
de novo.*

## §6 — A CONSOLIDAÇÃO: uma tabela mestra, 118 linhas

Os 17 resultados entram na **§10 deste documento**, sem reescrita — as colunas foram fixadas na
§3 exatamente para isso. Depois da consolidação eu produzo **uma** ordenação global.

## §7 — A PRIORIZAÇÃO (a régua, escrita ANTES de ver os resultados)

| P | Critério | Exemplo já medido |
|---|---|---|
| **P0** | inexprimível no grafo **E** o artista vê na primeira cena **E** todas as referências têm | variância por partícula do emissor · forma do emissor |
| **P1** | inexprimível **OU** exprimível a um custo que ninguém paga (3+ nós para um knob) | `inherit_velocity` · inwards/outwards |
| **P2** | exprimível hoje; o ganho é ergonomia | burst declarativo (o `pulse.*` já dirige o `rate`) |
| ⛔ | recusado **com o motivo e a referência** | eixo do `slit_scan` (é um `motion.sort`) |

**Desempate:** vence o item que uma **cena de smoke consegue mostrar lado a lado**. Um ganho
que só aparece numa tabela não é um ganho que o Enio possa julgar.

## §8 — A EXECUÇÃO (como uma wave nasce da tabela)

- **Uma wave por FAMÍLIA**, nunca por prioridade cruzando famílias: a família compartilha
  kernel, gates e cena de smoke, e um P0 solto de cada família custa cinco smokes.
- **Ordem das waves:** pela soma dos P0 da família, e o desempate é o quanto ela aparece na
  tela. A conferência decide; a §11 registra.
- **DoD de cada wave** (DIRETIVA §5): gate **red-first** + **mutação que sangra** + default que
  reduz literalmente (com gate próprio) + paridade CPU×GPU onde houver kernel + **cena de smoke
  que imprime o que montou** + aprovação do Enio.
- ⚠️ **A fixture do gate de paridade tem de CONTER o param novo** — o gate do `spherize` estava
  verde há meses com o offset da lente nunca exercitado, porque o `offset` que ele já passava
  era a translação da GRADE.

## §9 — Custo e limites

- **17 agentes**, um por família, sem recursão. Roda em ondas conforme o cap de concorrência.
- Cada um lê **documentos que já estão no repo** + conhecimento próprio das ferramentas; busca
  externa só onde a referência do repo estiver vazia para aquele nó.
- **O que este plano NÃO faz:** não implementa, não decide o que shipa, não abre ADR. Ele
  produz **a tabela mestra e a ordem das waves** — e é isso que o Enio aprova antes de a
  primeira linha ser escrita.

## §10 — TABELA MESTRA (a preencher pela conferência)

> Formato de uma linha, com o **`motion.emitter` já preenchido** pelo estudo de 2026-08-09 que
> originou este plano — ele é o gabarito do que cada agente devolve.

| nó | params hoje | falta (referência citada) | exprimível? (cadeia tentada) | nat./omissão | P | default que reduz |
|---|---|---|---|---|---|---|
| `motion.emitter` | 10 (`rate·life·speed·angle·spread·x·y·seed·max·size`), **1 lane de random** (`LANE_ANGLE=0`) | **variância** de vida/velocidade/tamanho/spin (Particular `*_Random %`; Apple Motion `+ Random` em cada knob; Niagara `Random Range` em todo Initialize; Cavalry `Override Lifespan`) | **NÃO** — um param dirigido (doc 58) é *um número por TICK*, não por partícula; nada a jusante reescreve `life`, que decide a própria janela do conjunto vivo | **omissão** | **P0** | variância `0` ⇒ o valor único de hoje |
| `motion.emitter` | idem | **forma** do emissor: círculo/retângulo/curva/perímetro (+size, margin, normais) — Cavalry §165; Niagara Location; Apple Motion Shape | a tentar (`motion.scatter` a jusante dá offset por id? mede) | **omissão** | **P0** | shape `Point` ⇒ a origem `x,y` de hoje |
| `motion.emitter` | idem | direção **inwards/outwards** (Cavalry `Initial Direction Type`; Particular `Disc/Outwards`) | **NÃO** sem forma (é radial À FORMA) | omissão | P1 | modo `Angle` ⇒ o cone de hoje |
| `motion.emitter` | idem | **inherit velocity** (Cavalry `Use Emitter Velocity`; Niagara) | **NÃO** | omissão | P1 | força `0` |
| `motion.emitter` | idem | **burst**: count/time/period/**probability** (Niagara `Spawn Burst`; Cavalry `Duration·Interval·Probability`) | **PARCIAL** — `pulse.*` dirige o `rate` (doc 58); falta a forma declarativa | omissão | P2 | probability `1`, duração `∞` |
| `motion.emitter` | idem | emissão por **DISTÂNCIA** percorrida (Niagara `Spawn Per Unit`; Cavalry `Emitter Type: Distance`) | **NÃO** (o emissor não conhece o próprio deslocamento… mas ele é função do playhead ⇒ **forma fechada**) | omissão | P2 | modo `Time` |

**`SUPERAR:` (da família 1, já derivado)** — o nosso emissor é **stateless**, então tudo o que
nas referências exige um acumulador aqui é **forma fechada e bit-exato sob scrub**: o
`inherit_velocity` é a derivada da posição do emissor no playhead, e o *spawn por distância* é
a inversa do comprimento de arco — as duas coisas que os outros aproximam por integração e que
divergem no scrub deles.

## §11 — Estado da conferência

| # | Família | Agente | Resultado | Verificado por mim | Wave |
|---|---|---|---|---|---|
| 1 | DISTRIBUIÇÃO + EMISSÃO | ⏳ | — | — | — |
| 2 | FORCE | ⏳ | — | — | — |
| 3 | SIMULAÇÃO | ⏳ | — | — | — |
| 4 | DEFORMERS | ⏳ | — | — | — |
| 5 | TRANSFORM | ⏳ | — | — | — |
| 6 | ANIMADORES | ⏳ | — | — | — |
| 7 | TEMPO / ESTILÍSTICOS | ⏳ | — | — | — |
| 8 | STREAM / UTILIDADE | ⏳ | — | — | — |
| 9 | COR / APARÊNCIA | ⏳ | — | — | — |
| 10 | FIELD | ⏳ | — | — | — |
| 11 | FX (raster) | ⏳ | — | — | — |
| 12 | PULSE (eventos) | ⏳ | — | — | — |
| 13 | SIM.\* (o stack) | ⏳ | — | — | — |
| 14 | SOURCE | ⏳ | — | — | — |
| 15 | VALUE | ⏳ | — | — | — |
| 16 | RIG (conferir, não implementar) | ⏳ | — | — | — |
| 17 | ZERO-PARAM + DEBUG (conferir) | ⏳ | — | — | — |

---

⚠️ **O doc 88 §9 fica SUPERSEDIDO por este plano** no que diz respeito aos vereditos de
família. O que sobrevive dele são as quatro **leis de param** (unidade · piso/teto duro ·
widget certo · todo param é desenhado), que são executáveis e continuam valendo, e as duas
famílias que de fato foram comparadas contra referência (ECHO e DEFORMERS-magros).
