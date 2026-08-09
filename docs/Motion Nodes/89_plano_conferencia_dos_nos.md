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
| ~~`motion.emitter`~~ | ~~variância de **VIDA**~~ | ~~Particular `Life Random %`; Cavalry `Override Lifespan`~~ | ⛔ **REFUTADO — é UM nó**: `sim.lifetime(variance, seed)` já existe e lê o `age` que o emissor escreve (`lib.rs:259`). ⚠️ E ele hasheia **por `id`**, não por índice — o doc-comment diz *"stateless, so a scrub reproduces the same deaths"* | — | ⛔ | — |
| `motion.emitter` | 10 (`rate·life·speed·angle·spread·x·y·seed·max·size`), **1 lane de random** (`LANE_ANGLE=0`) | **variância** de **velocidade/tamanho/spin** (Particular `*_Random %`; Apple Motion `+ Random` em cada knob; Niagara `Random Range` em todo Initialize) | **NÃO**, e a razão VERIFICADA é mais forte que a que eu havia escrito: a lane de aleatoriedade que serviria é `value.instance_field(Random)`, que hasheia **`(seed, index)`** — e o conjunto vivo do emissor é uma **janela DESLIZANTE de ids**, então o índice de uma partícula muda por baixo dela e a variância **CINTILA** | **omissão** | **P0** | variância `0` ⇒ o valor único de hoje |
| `motion.emitter` | idem | **forma** do emissor: círculo/retângulo/curva/perímetro (+size, margin, normais) — Cavalry §165; Niagara Location; Apple Motion Shape | **NÃO** — a cadeia que eu mandei medir (`motion.scatter` a jusante) **não existe**: o `scatter` tem `inputs: &[]`, é *Source* e não filtro. A que funciona é `<distribuição> → sim.spawn → motion.combine → sim.step` dentro de uma `sim.zone`: **4 nós, e troca o emissor STATELESS por uma zona com estado** | **omissão** | **P0** | shape `Point` ⇒ a origem `x,y` de hoje |
| `motion.emitter` | idem | direção **inwards/outwards** (Cavalry `Initial Direction Type`; Particular `Disc/Outwards`) | **NÃO** sem forma (é radial À FORMA) | omissão | P1 | modo `Angle` ⇒ o cone de hoje |
| `motion.emitter` | idem | **inherit velocity** (Cavalry `Use Emitter Velocity`; Niagara) | **NÃO** | omissão | P1 | força `0` |
| `motion.emitter` | idem | **burst**: count/time/period/**probability** (Niagara `Spawn Burst`; Cavalry `Duration·Interval·Probability`) | **PARCIAL** — `pulse.*` dirige o `rate` (doc 58); falta a forma declarativa | omissão | P2 | probability `1`, duração `∞` |
| `motion.emitter` | idem | emissão por **DISTÂNCIA** percorrida (Niagara `Spawn Per Unit`; Cavalry `Emitter Type: Distance`) | **NÃO** (o emissor não conhece o próprio deslocamento… mas ele é função do playhead ⇒ **forma fechada**) | omissão | P2 | modo `Time` |

**`SUPERAR:` (da família 1, já derivado)** — o nosso emissor é **stateless**, então tudo o que
nas referências exige um acumulador aqui é **forma fechada e bit-exato sob scrub**: o
`inherit_velocity` é a derivada da posição do emissor no playhead, e o *spawn por distância* é
a inversa do comprimento de arco — as duas coisas que os outros aproximam por integração e que
divergem no scrub deles.

## §10.0 — O ACHADO DA CONFERÊNCIA: o catálogo LÊ qualquer coluna e ESCREVE cinco

> **Este é o resultado que a partição existia para produzir.** Nenhum agente o enunciou inteiro —
> cinco famílias bateram nele por sintomas que pareciam não ter relação, e ele só aparece quando
> os cinco relatos são postos lado a lado. Uma varredura por nó **não podia** tê-lo achado.

**Verificado por grep, nos dois lados:**

| direção | quem faz | alcance |
|---|---|---|
| **LER** | `value.attribute` (modo Custom) | **qualquer coluna, por nome** |
| **ESCREVER** | `motion.drive` | **exatamente cinco**: `X · Y · Rotation · Size · Opacity` — e o comentário do próprio arquivo os chama de *"the shared channel vocabulary"* |

⇒ **o domínio de VALOR é um beco sem saída assimétrico.** Ele enxerga tudo o que o stream tem e
só consegue devolver cinco coisas. Cada "gap inexprimível" P0 abaixo é a MESMA ausência, vista
de uma família diferente:

| a coluna que ninguém escreve | o que fica inexprimível | família |
|---|---|---|
| `falloff` | campo de ruído/textura/fórmula/áudio · densidade por-instância · force-over-life | 2 · 10 |
| `rot` a partir de uma tangente | texto em curva que gira · órbita que vira · *"vire para onde está indo"* | 1 · 4 · 5 · 6 · 15 |
| `vel` | speed-limit suave · a rampa que inclina o chão | 13 |
| `parent` · `len` | comprimento por-osso · ramificação · peso por-osso | 16 |

⚠️ **A leitura ainda RESPONDE ERRADO** (T4): `value.attribute` cai em `_ => vec![0.0; n]` para
Vec4 e para Vec2 em X/Y. Então a assimetria real é pior — *lê quase tudo, e o que não lê devolve
zeros em silêncio; escreve cinco*.

⇒ **A primeira pergunta da consolidação não é "que param falta": é *quem pode escrever uma
coluna, e como?*.** Um escritor genérico (`motion.drive` com canal por NOME, o espelho exato do
`value.attribute` Custom) destrava as quatro linhas da tabela acima **de uma vez**, sem tocar o
contrato congelado — `drive` já é um nó, e o canal é um param dele. Isto **não se constrói antes
das 17 famílias**, mas é a hipótese que a ordenação global tem de testar primeiro.

## §10.1 — ACHADOS TRANSVERSAIS (verificados por mim, §5 — crescem a cada família)

> Um achado que aparece em **duas famílias independentes** não cabe no arquivo de nenhuma delas,
> e é precisamente o que a partição existia para produzir: os agentes não se falam, então a
> convergência é evidência, não coincidência.

### T1 — A coluna `falloff` é FECHADA À ESCRITA pelo domínio de VALOR

**Quem achou:** FORCE (chamou de *"parede 4"*) **e** FIELD (*"o destravador"*), sem se verem.
**Verificado por grep, não por raciocínio:**

```
25 crates tocam a coluna `falloff`  ·  ZERO delas tem uma porta `Domain::Values`
```

⚠️ **A consequência é maior que a soma das duas famílias.** A máscara espacial já é o vocabulário
comum de **cinco** famílias (as `field.*`, as `force.*`, os deformers, os transforms e os
estilísticos leem `falloff`), mas ela só pode ser **DERIVADA DE GEOMETRIA** — nenhuma quantidade
computada pode virar campo. É isso que torna inexprimíveis, de uma vez:

- `field.noise` (C4D *Random Field* · MOPs *Noise Falloff*);
- a **densidade por-instância** do `force.buoyancy` (Cavalry `Buoyancy-FIELD`);
- a *force-over-life* da Stardust e o speed-limit suave;
- textura / fórmula / áudio / atributo como campo — o `motion.luminance` **já devolve luma como
  VALOR** e não tem como virar máscara.

⇒ **uma porta destrava uma classe inteira em cinco famílias**, e a decisão de *onde* ela mora
(porta no `field.remap`? nó `field.attribute`? ambos?) é a primeira pergunta que a consolidação
tem de responder. **Não construir antes das 17.**

### T2 — Existem DUAS lanes de aleatoriedade e só uma sobrevive a um conjunto que desliza

**Quem achou:** DISTRIBUIÇÃO+EMISSÃO. **Verificado no código:**

| lane | hasheia | sobrevive a scrub / janela deslizante? |
|---|---|---|
| `sim.lifetime` | **`hash(seed, id, lane)`** | **sim** — o doc-comment diz *"stateless, so a scrub reproduces the same deaths"* |
| `value.instance_field(Random)` | **`(seed, index)`** | **não** — o conjunto vivo do emissor é uma janela DESLIZANTE de ids, o índice de uma partícula muda por baixo dela, e a variância **CINTILA** |

⚠️ Isto **refutou uma linha do meu próprio gabarito** (a variância de VIDA é UM nó, não um gap) e
**fortaleceu outra**: a variância de velocidade/tamanho/spin não é P0 por *"nada reescreve o
valor"*, é P0 porque *a única lane que a daria é indexada*. A regra da §5 vale para os meus
claims tanto quanto para os dos agentes.

### T4 — O muro tem DOIS lados, e o lado da LEITURA responde ERRADO EM SILÊNCIO

**Quem achou:** TRANSFORM, COR e VALUE — **três** famílias independentes, cada uma batendo nele
por um caminho diferente (o pivô-centroide · a leitura de matiz · o *"vire para onde está indo"*).
**Verificado no código** (`ph2d-node-value-attribute/src/lib.rs:79`):

```rust
(Some(Column::Scalar(v)), m) if m != MODE_LENGTH && v.len() == n => v.clone(),
(Some(Column::Vec2(v)),   MODE_LENGTH)            if v.len() == n => …magnitude…,
_ => vec![0.0; n],                    // ← Vec4, Vec2 em X/Y, escalar-em-Length
```

⚠️ **CORREÇÃO MINHA, aplicando a lei 5 a mim mesmo:** eu escrevi que isto era *"uma resposta
errada apresentada como dado"* — **defeito**. Fui ler o `_` antes de codificar e ele tem **cerca
escrita**: o módulo declara *"A missing column is `0`, not a crash"* com o raciocínio (*"um nó que
desse erro derrubaria o grafo inteiro porque o artista digitou `ag` em vez de `age`"*), e o lado
GPU diz literalmente *"wrong dim for the mode — is zeros, never an error"*. **É política, não
bug**, e eu a teria "consertado" chamando-a de defeito.

**O que sobrevive à cerca — e é o gap de verdade:** o raciocínio dela é sobre **TYPO**, e não se
estende ao caso novo. Pedir o X de uma `Vec2` que EXISTE não é um engano de digitação: é uma
pergunta legítima que o nó **não sabe fazer**, e a política a atende com zeros porque não há
outra resposta a dar. ⇒ a cura não é remover o `_`; é **acrescentar os modos**, deixando o `_`
exatamente onde está para o caso que ele foi escrito para cobrir.

E o `motion.expression` **não é o escape**: o MANIFEST dele é `INST_VEC2 → VALUE`, ou seja ele é
**fonte de valor e nunca transformador** — não tem porta VALUE de entrada, e o `fn attr` dele lê
só `Column::Scalar`. ⚠️ Isto **corrige uma premissa que eu injetei em vários briefings**
(*"tente o `motion.expression` antes de propor um knob"*): ele refuta muito menos do que eu
supunha, e quem de fato refutou 17 gaps na família VALUE foi a **composição de nós de valor**
(`1−x` é `map_range` · `ceil` é `quantize` · AND/OR são `Min`/`Max` · duty cycle é `lfo(Saw)→step`).

⇒ **T1 e T4 são o mesmo muro visto dos dois lados:** o domínio de VALOR não lê as colunas ricas
do stream (T4) e não escreve a máscara que cinco famílias consomem (T1). Enquanto os dois lados
estiverem fechados, *"que param falta neste nó"* é a pergunta pequena.

### T5 — DOIS defeitos com gates VERDES por cima (não são gaps; são bugs)

1. **A alfa por-stop do gradiente nunca chega ao device.** O `g2` e o `ColorRamp::eval` a
   carregam; **`LUTS` tem TRÊS entradas — `r`, `g`, `b`** (verificado) — e o kernel escreve `1.0`.
   Os dois gates de paridade usam presets **opacos**: *a fixture não contém o fenômeno*, então
   eles são verdes por construção.
2. **O `spread` do `motion.collide` é uma porta POR-INSTÂNCIA que o `eval` colapsa em
   `vals.first()`** (claim da família SIMULAÇÃO, ainda **não verificado por mim**) — raio por
   elemento não seria param faltando, seria **entrada descartada**.

⚠️ Os dois entram na tabela mestra como **defeito**, não como gap — e um defeito não espera a
priorização de produto do §7: ele é conserto.

### T6 — Nada no catálogo converte uma DIREÇÃO em ROTAÇÃO (e T4 é o mecanismo)

**Quem achou:** DISTRIBUIÇÃO, DEFORMERS, TRANSFORM, ANIMADORES e VALUE — **cinco** famílias,
cada uma por um sintoma diferente. **Verificado por grep:** exatamente **dois** nós escrevem a
coluna `rot` — `motion.look_at` (que mira um PONTO) e `motion.rotate` (que aplica um ângulo).
**Nenhum toma uma tangente.**

| sintoma | família |
|---|---|
| `distribute_curve` não devolve tangente/normal/rotação — e o `motion.path`, que faz a mesma distribuição, **já tem `align`**: dois nós discordam sobre orientar | 1 |
| `spline_wrap` não gira o que embrulha ⇒ **texto numa curva não vira** | 4 |
| a órbita não carrega a orientação ⇒ o sprite **desliza de lado** | 5 |
| *"vire para onde está indo"* é inexprimível | 6 · 15 |

⚠️ **E T4 explica POR QUÊ:** a direção existe como coluna `Vec2` (`vel`, a tangente), e
`value.attribute` **não lê X nem Y de uma Vec2** — só magnitude. A capacidade não falta por
esquecimento de knob; ela é **bloqueada pelo muro da leitura**. T6 não é um sexto item da lista:
é o que T4 custa, medido em features que o artista vê na primeira cena.

### T7 — A cerca que RECUSA uma família inteira, e ela é ordem do Enio

A família FX achou, por grep no histórico, que a **grade HDR de tela inteira foi construída
(`9a36d4a27`) e REMOVIDA (`f2daa787a`) por ordem direta**: *"vamos abandonar esses efeitos de tela
inteira dentro do motion. Pois teremos um módulo de pós produção no futuro."*

⇒ levels · gamma · posterize · threshold · invert · B&W · grain · sharpen · vinheta **não são
lacunas do grafo**. E a navalha que a família derivou torna a fronteira executável em vez de
opinativa: **o passe do Motion compõe ADITIVAMENTE** (`One/One/Add`, medido) — foi por isso que o
glow pôde ser nó. *Aditivo ⇒ cabe como `fx.*` hoje; subtrativo ou remapeador ⇒ é o módulo de
pós-produção.* ⚠️ Sem essa navalha, ~21 dos ~41 filtros ausentes teriam virado proposta — e
**eles já existem noutro módulo do app** (o Painter tem 24 `AdjustmentKind`, o Vector 15 FX
raster): a pergunta nunca foi *"falta este efeito?"*, era *"onde ele mora?"*.

### T3 — A cura mais barata às vezes é um param num nó que já existe

`motion.falloff` é o **único** campo com forma **Linear** do catálogo e **não tem `rotation`**;
o `field.box`, irmão dele, **tem**. Uma rampa linear em ângulo qualquer — que o C4D expõe como
*Direction* — não pede nó novo, pede **um param**. ⚠️ E o doc 63 §198 chama o `motion.falloff` de
*"alias/compat"*: seguir aquela nota teria **apagado** o único campo linear e radial do catálogo,
do qual 5 das 13 composições da família FORCE dependem.

## §10.2 — A ORDENAÇÃO GLOBAL DAS WAVES (o produto do plano)

**A régua da §7 foi escrita antes dos resultados e não se mexe.** O que os resultados
acrescentam é uma coisa que a régua não previa: **~12 dos ~52 P0 são o MESMO gap**, visto de oito
famílias diferentes. Ordenar por família colocaria a mesma construção em oito waves.

⚠️ **Isto NÃO revoga a §8** (*uma wave por família*). A §8 existe porque uma família partilha
kernel, gates e cena de smoke — e o muro da §10.0 **não pertence a família nenhuma**: ele é o
substrato que todas consomem. Uma wave foundational antes das de família é a consequência da §8,
não a exceção dela.

### W0 — O LEITOR E O ESCRITOR DE COLUNA  ·  *o desbloqueador*

| | |
|---|---|
| **Mata** | ~12 P0 em **8 famílias** (1 · 4 · 5 · 6 · 8 · 9 · 10 · 15) |
| **Custo** | dois nós que **já existem** ganham simetria; zero contrato congelado, zero schema |
| **Metade A (LER)** | `value.attribute` passa a devolver componentes X/Y de `Vec2` e canais de `Vec4` — e **para de devolver zeros em silêncio** (é DEFEITO, não feature) |
| **Metade B (ESCREVER)** | `motion.drive` toma o canal por **NOME**, o espelho exato do `value.attribute` Custom — os cinco de hoje viram os cinco nomes de sempre |
| **Cai de graça** | `field.from_value` (a máscara é uma coluna) · `make_point` para `vel` · hue/sat/lightness · o pivô-centroide · `@P.x` na fórmula |
| **Default que reduz** | os cinco canais atuais, os dois modos atuais — bit a bit |

**Estado: A ✅ · B ✅ (a metade que desbloqueia) · B-genérico ⏳ (wave própria, com o preço medido).**

- **A — o leitor** (`53b54a773`): `value.attribute` lê **pista a pista** (`MODE_COMPONENT_BASE + k`),
  um degrau para Vec2/Vec3/Vec4 ⇒ o caso da COR cai junto. CPU **e** device, porque o
  `encode_project` replica a escada e shipar meia levaria à divergência que o T5 catalogou.
- **B — o escritor, pela porta barata:** a família FIELD nomeou **duas** formas do destravador —
  *"`field.from_value` **ou o canal `Falloff` no `motion.drive`**"* — e a segunda é **binding
  ESTÁTICO**, ou seja GPU-nativa pela máquina que já existe. `motion.drive` ganhou o 6º canal:
  um número computado vira **a máscara que cinco famílias leem**.
  - ⚠️ **Não auto-mascara, e não foi escolha:** todo outro variante binda `falloff` como `Read`
    para misturar, então o variante cujo ALVO é `falloff` o binda uma vez como `ReadWrite` e o
    read comum simplesmente não existe — a auto-máscara é **inexprimível na lista de bindings**,
    que é o tipo de recusa que sobrevive à próxima pessoa.
  - ⚠️ **Não clampa em `[0,1]`, de propósito:** um peso NEGATIVO inverte a força que o consome —
    a capacidade que a família FORCE achou ser nossa e não do C4D/Cavalry (que são `[0,1]` por
    construção). Clampar aqui a apagaria antes de alguém a usar.
  - ⚠️ **Identidade `1.0`, não `0.0`:** coluna ausente significa *efeito cheio* para todo leitor
    da biblioteca, e um escritor que começasse do zero discordaria de todos eles.
- **B-genérico — o que SOBRA e por que é wave própria:** escrever uma coluna **por NOME** (`vel`,
  `parent`, `len` — as famílias 13 e 16) precisa do análogo de ESCRITA do `StreamOp::Project`,
  porque um nome dinâmico não é binding estático. É maquinaria de sequenciador com pipes e
  paridade próprios, **não um apêndice**. O canal `Falloff` já mata a linha mais cara da tabela
  da §10.0; as outras três esperam essa wave.

⚠️ **A metade A vem primeiro e sozinha**, porque é conserto de uma resposta errada: hoje
`_ => vec![0.0; n]` entrega zeros com cara de dado. Um defeito não espera priorização de produto.

### W1..W7 — as famílias, pelo P0 que SOBRA depois do W0

| ordem | wave | P0 restantes | por que aqui |
|---|---|---|---|
| **W1** | **SIMULAÇÃO** (fam. 3) | 8 → **~7** | o maior bloco que o W0 não toca — e o item de cabeça é *os 3 geradores não consomem `accel`*, ou seja **a família `force.*` inteira não alcança simulação nenhuma**. É um segundo desbloqueador, barato (uma leitura de coluna) e com fan-out de 6 nós — **W1-A ✅**, ver abaixo |
| **W2** | **SOURCE** (fam. 14) | 4 | **39 formas já construídas e pagas são inalcançáveis do grafo** — é FIAÇÃO, não geometria: o melhor retorno por linha da conferência inteira |
| **W3** ✅ | **COR** (fam. 9) | 6 → **~3** → **6 FEITOS, FAMÍLIA FECHADA** | **os seis P0 fechados em 2026-08-09** ([§3 do 09_cor](89_conferencia/09_cor.md)): a alfa que não chegava ao device (**defeito**) · a máscara por campo · o canal "Opacity" do picker (que a tabela lia como *duas portas* e a medição mostrou ser **um FANTASMA** — ninguém escreve a coluna `opacity`) · o **canal do `motion.luminance`**, de 1 para 8 · e o **ESPAÇO de interpolação da rampa** (token `g3`, append-only; o motor sempre soube interpolar em HSV/HSL e o formato **não tinha onde guardar a escolha**, e o device herda de graça porque o LUT é assado na CPU). · e a **metade de ESCRITA do laço de cor** (os canais Hue/Saturation/Value do `motion.drive`, o espelho exato do `luminance` — e os modos `Add`/`Multiply` que já existiam JÁ eram o *Master Hue* / *Saturation* da referência). **Nenhum P0 restante nesta família** |
| **W4** ⏳ | **TEMPO / ESTILÍSTICOS** (fam. 7) | 4 → **1 FEITO** | **o DEFEITO caiu em 2026-08-09** ([§6 do 07_tempo](89_conferencia/07_tempo_estilisticos.md)): o `motion.morph` emitia **só `P`** e descartava `size`/`rot`/`tint`/`uv_rect`/`texture_id`/`geometry_id` — morfar dois `source.object` perdia a aparência e caía na tile 0. Agora as quantidades desvanecem e a identidade é carregada pelo **vizinho mais próximo** (lista BRANCA, cujo apodrecimento se VÊ). Seguem P0: a máscara por campo do `trail`, o canal do `delay`, o **reset** do `step` |
| **W5** | **DISTRIBUIÇÃO + EMISSÃO** (fam. 1) | 4 → **~3** | o W0 mata a tangente; sobram a FORMA do emissor e o arco radial |
| **W6** | **PULSE** (fam. 12) | 2 | o *nível* é o gargalo único: com ele, gate/AND/OR/sequenciador/burst caem juntos |
| **W7** | **SIM.\*** (fam. 13) | 3 | o estágio de EVENTOS depende do W6 (um pulso com nível) |

#### W1-A ✅ — os três geradores consomem `accel`  ·  *o segundo desbloqueador*

`motion.verlet_rope` · `motion.soft_body` · `motion.boids` liam **nenhuma** coluna. Agora leem
`accel`, e isso entrega **de uma vez** a família `force.*` inteira a uma corda, a uma gelatina e a
um bando: gravidade com DIREÇÃO, vento, curl (= o *wander* do Reynolds), atrator, vórtice, arrasto.
A fiação já existia, o vocabulário já existia; faltava a leitura.

- **Entra onde a aceleração já entrava**, então não há kernel novo: `p += a·dt²` no Verlet e na
  predição do shape matching, `v += a·dt` como quarto termo de steering no boids — antes do clamp
  de `max_speed`, que é o que faz o `force.curl` ler como *wander* e não como empurrão.
- **Consumido, nunca reemitido** (os três emitem o próprio estado e mais nada), então todo tique
  começa de aceleração zero — a disciplina que o `motion.integrate` já declara.
- **Zeros são a IDENTIDADE** ⇒ uma simulação que nenhuma força alcança é **byte-idêntica** à que
  shipou. ⚠️ E a associação importa: `p.gravity * dt * dt` é `(g·dt)·dt`, então re-agrupá-lo como
  `g·dt²` **move o ulp de toda corda que já pendurou** — o `dt2` só serve ao termo NOVO.
- **A metade do ADR-0155 viaja no mesmo commit**, e não é higiene: sem a `Coupling::Consumes`, o
  diagnose vê um `Produces("accel")` órfão e oferece **inserir um `motion.integrate`** — que é
  `Temporal`, carimba `sim_t = playhead` de passagem, e entregaria `dt = 0` ao gerador,
  **CONGELANDO** a simulação. Declarar o consumo e ensinar o diagnose a vê-lo são a mesma mudança.
- ⚠️ **O boids tem kernel de GPU, então a leitura é DUPLA** (`read_state_accel` no WGSL, binding
  `Consume` na porta de estado) — shipar só a CPU seria a divergência que o T5 catalogou.

**Duas coisas que a MEDIÇÃO corrigiu, e as duas ficam escritas:**

1. **A barra do gate da gelatina era um palpite meu.** O cisalhamento **satura** (o corpo cede até
   a restauração elástica equilibrar o vento): `5 → 0,073` · `20 → 0,282` · **`60 → 0,743`** ·
   `180 → 1,348`. A barra é `0,3` sobre a medição de `60`, com o modo de falha medindo zero.
2. ⚠️ **Os quatro gates de paridade do boids NÃO CONTINHAM O FENÔMENO.** Eles montam
   `boids → output` com o self-loop nu, então `accel` está **ausente nas duas rotas** e elas
   concordam sobre um termo que nenhuma avalia: apagar a linha do WGSL deixa os quatro **VERDES**,
   com a CPU levando o vento e a GPU não. Fixture nova com a força na cadeia (`2,4e-7` no device),
   e a mutação sangra **só** ela.
3. ⚠️ **E uma afirmação MINHA sobre `Consume` era FALSA** — eu escrevi que trocá-lo por `Read`
   seria pego por um gate. Não é: a supressão de ride-through corre pela porta BASE, e a porta 0
   do boids é `VALUE` contra um output `INST_VEC2` ⇒ `rides_base = false`, a saída nasce vazia e
   `Consume` remove uma coluna que nunca esteve lá. Medido, não suposto. A palavra fica (é a certa,
   e é a que o `motion.integrate` usa), **sem gate**, porque um gate que não pode falhar pelo
   motivo que alega é pior que nenhum. Um gate de dois passos foi escrito para pegá-lo e
   **descartado** quando a mutação sobreviveu a ele também.

#### W1-B ⏳ — o `spread` do `motion.collide`: o defeito era MAIOR, e a cura é OUTRA

A conferência classificou isto como *"entrada descartada"* (`spread_amount = vals.first()`
colapsa uma coluna por-instância) e prescreveu *"ler a coluna"*. A verificação achou uma coisa
pior e mudou a cura:

⚠️ **A GPU já lia por elemento** (`read_spread_v(i)`), e a CPU não — logo isto não era entrada
descartada, era **DIVERGÊNCIA CPU×GPU não gateada**. Medido com um `value.instance_field` em modo
Ramp na porta: a CPU devolve a grade **INTOCADA** (o `vals[0]` é `0` ⇒ raio `0` ⇒ a identidade
precoce dispara) enquanto o device empurra — **área 1,5625 contra 2,2039**, pior `|Δpos| = 1,65e-1`.
Não é ε: são dois desenhos. ⚠️ **E as fixtures irmãs não continham o fenômeno** (alimentam `spread`
de comprimento 1 ou nenhum), então os gates estavam todos verdes por cima.

⚠️ **O caminho por-elemento do device era ACIDENTE, não desenho:** a doc das bindings dele declara
*"broadcast (ausente ⇒ 1)"* — a *respiração* animável —, e `ColumnAccess::ReadBroadcast` só faz
broadcast com **um** valor; com N ele devolve `in[i]`. A lei que o acidente produzia é ainda
**assimétrica** (`min_dist = 2·r_i`, o raio de quem olha), então dois discos discordariam sobre
estarem se tocando.

**Fechado agora:** o device honra a intenção que ele mesmo declara (`read_spread_v(0u)`) ⇒ as duas
rotas são **bit-exatas** (`0e0`) e **nenhuma cena se move** (a CPU era a referência e não mudou).

**Aberto, com o desenho escrito — W1-B:** o raio **POR ELEMENTO** é capacidade real e desejada
(o `pscale` do Houdini POP Interact; o tamanho do clone no Push Apart do C4D), e a lei honesta é
**`r_i + r_j`** — simétrica, e **byte-idêntica à de hoje sob spread uniforme** (`r+r = 2r`). O que
a torna wave e não linha é o device: com raios variáveis o alcance da grade tem de ser limitado
pelo raio **MÁXIMO**, ou seja um `Max` reduce sobre a coluna — e **nenhum kernel do repo combina
`register_grid` com `reduces()` hoje**. É máquina nova, com paridade própria.

**O que SOBRA da família 3** (os ~7 P0): o teto de 2 000 do boids (o caminho da CPU definindo o do
device — §0.0) · `max_force` · `pressure` e os clusters do soft body · o `bend stiffness` da corda ·
e o raio por-elemento do `motion.collide` (**W1-B** acima — o defeito de divergência que ele
escondia está FECHADO).
⚠️ E **dois P0 mudaram de natureza com esta wave**: a *gravidade VETOR* da corda e da gelatina
passa a ser **exprimível por composição** (uma `force.wind` em qualquer ângulo, com `gravity = 0`)
— o que era gap virou ergonomia, e é isso que um desbloqueador faz com a tabela que o precede.

**Fora da fila, com motivo escrito:**
- **FX (fam. 11)** — P0 = 0, e a cerca do módulo de pós-produção já recusa a classe (**T7**).
- **RIG (fam. 16)** — deferida por decisão sua; a tabela existe para quando for retomada.
- **DEFORMERS · TRANSFORM · VALUE · STREAM · ANIMADORES · ZERO-PARAM** — o P0 delas **é** o W0.
  Depois dele, o que sobra é P1, e P1 entra por família na segunda volta.

**Os dois DEFEITOS (T5) não entram nesta fila** — eles são conserto e andam junto da wave que
tocar o arquivo: a alfa da rampa no W3, o `spread` do `collide` no W1.

## §11 — Estado da conferência

> ⚠️ **Cada agente ESCREVE o resultado num arquivo próprio** em
> [`89_conferencia/`](89_conferencia/) e devolve só um resumo. Dezassete tabelas cruas num
> contexto só matam a consolidação antes de ela começar — e um arquivo por família é também o
> que faz a §10 ser montada por concatenação em vez de reescrita.

| # | Família | Arquivo (em `89_conferencia/`) | P0 | P1 | P2 | ⛔/refutados | O que ela achou de mais caro |
|---|---|---|---|---|---|---|---|
| 1 | DISTRIBUIÇÃO + EMISSÃO | `01_distribuicao_emissao.md` | 4 | 13 | 9 | 6 / **7** | as duas lanes de random (**T2**); `distribute_curve` sem tangente |
| 2 | FORCE | `02_force.md` | 1 | 5 | 6 | 10 / **13** | a parede `value → falloff` (**T1**); o cluster NOISE não chegou às forças |
| 3 | SIMULAÇÃO | `03_simulacao.md` | 8 | 17 | 6 | — / **8** | os 3 geradores **não consomem `accel`** ⇒ a família `force.*` inteira não alcança sim nenhuma |
| 4 | DEFORMERS | `04_deformers.md` | 2 | 6 | 8 | 6 / **4** | mascarar uma ROTAÇÃO encolhe o layout (o lerp corta pela corda) |
| 5 | TRANSFORM | `05_transform.md` | 1 | 4 | 4 | 6 / **4** | escala em torno da ORIGEM DO MUNDO, sem pivô |
| 6 | ANIMADORES | `06_animadores.md` | 3 | 12 | 15 | — / **9** | a porta `time` opcional (o melhor SUPERAR da conferência) |
| 7 | TEMPO / ESTILÍSTICOS | `07_tempo_estilisticos.md` | 4 | 11 | 8 | 7 / **6** | ⚠️ o escopo de tempo **RECUSA** nó sequencial ⇒ 4 dos 5 não entram nele |
| 8 | STREAM / UTILIDADE | `08_stream_utilidade.md` | 4 | 9 | 10 | 1 / **9** | `look_at` é o único behaviour que **não honra o `falloff`**; duas segundas portas |
| 9 | COR / APARÊNCIA | `09_cor.md` | 6 | 5 | 4 | 6 / **5** | o loop de cor é one-way; a alfa da rampa **não chega ao device** (**T5**) |
| 10 | FIELD | `10_field.md` | 3 | 9 | 5 | 8 / — | a coluna `falloff` é fechada à escrita (**T1**) |
| 11 | FX (raster) | `11_fx_raster.md` | 0 | 6 | 7 | 3 / — | a navalha ADITIVO×SUBTRATIVO e a cerca do módulo de pós (**T7**) |
| 12 | PULSE (eventos) | `12_pulse.md` | 2 | 6 | 7 | 4 / — | um pulso **não tem NÍVEL**; e o gabarito §10 estava errado sobre ele |
| 13 | SIM.\* (o stack) | `13_sim_stack.md` | 3 | 7 | 8 | — / **6** | não há estágio de EVENTOS; substeps **provado** inexprimível |
| 14 | SOURCE | `14_source.md` | 4 | 5 | 5 | — | **47 formas** existem atrás de uma porta única e o nó usa um enum próprio de **8** |
| 15 | VALUE | `15_value.md` | 3 | 16 | 14 | 17 / — | `motion.expression` é **fonte**, não transformador ⇒ refuta muito menos do que eu supunha |
| 16 | RIG *(deferida)* | `16_rig.md` | 3 | 13 | 7 | 4 / — | zero `Strength`/`Mix` (Rive tem em 7 de 7); `parent`/`len` sem escritor |
| 17 | ZERO-PARAM + DEBUG | `17_zero_param_debug.md` | 1 | 1 | — | **9 confirmados** | o `motion.output` é o gap: a ponte crava **10 campos**, entre eles o BLEND |

**Fechamento: 118 nós conferidos · ~52 P0 · ~145 P1 · ~123 P2 · ~65 gaps REFUTADOS por
composição** — e os refutados valem tanto quanto os confirmados, porque cada um impede a próxima
varredura de propor o que já é exprimível.

⚠️ **E o erro mais caro do doc 63 é uma célula só, achada pela família 8:** ele marca
**`Store Named Attribute` como "TEMOS"**, citando o `value.attribute` — que é o **LEITOR**. Um
item marcado *TEMOS* que não existe não manda construir o construído: manda **não construir** o
que a §10.0 acabou de identificar como o gargalo de seis famílias. *É provavelmente por isso que
o escritor de coluna nunca foi escrito.*

⚠️ **O `motion.output` é a mesma assimetria da §10.0 no lado da SAÍDA:** a ponte grafo→render lê
sete colunas e crava dez campos em identidade nas DUAS rotas (CPU e GPU) — entre eles o **blend
mode**, cuja máquina de roteamento por-instância **já existe e é testada**. Um grafo de Motion
hoje não consegue fazer uma faísca aditiva, e o conserto é convenção de stream (o molde do
`texture_id`): **zero ABI, zero contrato congelado**, com `Normal` reduzindo literalmente.

⚠️ **Quatro famílias receberam um briefing MAIS LARGO que "que param falta"**, porque a pergunta
honesta delas é de CATÁLOGO e não de knob — e um agente que só compara params responderia bem à
pergunta errada:

| Família | A pergunta que ela também responde | Por quê |
|---|---|---|
| 10 FIELD | `ESPÉCIES QUE FALTAM:` | o C4D Fields tem um catálogo de campos; nós temos **cinco** |
| 11 FX | `EFEITOS QUE FALTAM:` (+ *já existe noutro módulo?*) | a Cavalry lista **54 filtros**; nós temos **três** — e Painter/Vector já têm parte deles |
| 13 SIM.\* | `ESTÁGIOS QUE FALTAM:` | o stack do Niagara é System/Emitter/Particle com estágios nomeados |
| 14 SOURCE | `ESPÉCIES DE FONTE QUE FALTAM:` | **duas** fontes contra o catálogo de primitivas da Cavalry |
| 16 RIG | `CONSTRAINTS QUE FALTAM:` | Rive e Spine convergiram num conjunto pequeno e nomeado |

---

⚠️ **O doc 88 §9 fica SUPERSEDIDO por este plano** no que diz respeito aos vereditos de
família. O que sobrevive dele são as quatro **leis de param** (unidade · piso/teto duro ·
widget certo · todo param é desenhado), que são executáveis e continuam valendo, e as duas
famílias que de fato foram comparadas contra referência (ECHO e DEFORMERS-magros).
