# Pesquisa — composição de objetos, prefab/instância e o inspetor (Fases B1–B3)

> **Pesquisa externa: 2026-08-21.** Cada afirmação traz a fonte. Distingo três níveis:
> **[DOC]** documentação oficial do fornecedor · **[COM]** post/discussão/artigo da comunidade ·
> **[INF]** inferência minha a partir das duas anteriores (marcada sempre).
>
> **O que este doc NÃO refaz:** os [7 dossiês de 2026-08-20](pesquisa/) já cobriram o *catálogo de
> componentes de gameplay* de 10 engines. Aqui o assunto é outro e não se sobrepõe: **o modelo de
> composição, o modelo de reuso com override, e a UX do painel**. Onde um dossiê já respondeu, eu
> aponto em vez de repetir (ex.: Required Components do Bevy —
> [`dossie_bevy_rust.md §0.1`](pesquisa/dossie_bevy_rust.md)).
>
> **Estado do PH2D:** [`01_auditoria_modelo_de_objeto.md`](01_auditoria_modelo_de_objeto.md). Este doc
> **não decide nada** — a decisão é a [Fase D](04_decisao_arquitetura.md).

---

## 📍 Índice

| § | assunto |
|---:|---|
| **§0** | Higiene de licença e proveniência |
| **§1** | **B1** — os seis modelos de composição, e o que cada decisão custou |
| **§1.7** | ⭐ A pergunta central: componente ECS, nó de DAG e item do inspetor são a mesma abstração? |
| **§2** | **B2** — prefab: **quatro** modelos de override, e como cada um sobrevive (ou não) ao mestre mudar |
| **§2.8** | ⭐ A tabela que decide: como cada modelo ENDEREÇA um override |
| **§3** | **B3** — UX do inspetor: Add Component, disclosure, data-driven, e a caneta |
| **§4** | O que disto é aplicável ao PH2D (ponteiros, sem escolher) |
| **§5** | Fontes |

---

## §0 — Higiene de licença e proveniência

| Fonte | Licença | O que eu li | O que é permitido |
|---|---|---|---|
| Unity, Unreal, Figma, Houdini, Rive | proprietárias | **só documentação pública** | estudar comportamento, terminologia, modelo conceitual |
| Godot | MIT | documentação + classes públicas | comportamento **e** código, se necessário — MIT é compatível |
| Blender | **GPL** | ⚠️ **só o manual do utilizador e notas de release** | comportamento e terminologia apenas. ⛔ **Nenhum código GPL foi lido; nenhuma implementação derivada de fonte Blender** |
| flecs | MIT | manual público (`flecs.dev`) | comportamento **e** código — MIT compatível |
| Bevy | MIT/Apache-2.0 | docs, migration guides, discussões do GitHub | tudo — é a licença do próprio `bevy_ecs` que o PH2D já usa |
| OpenUSD | Apache-2.0 (Tomorrow/modified) | docs oficiais + guia comunitário | modelo conceitual e, se um dia interessar, código |

⚠️ **Nota de método:** `docs.blender.org` **recusa fetch automatizado (HTTP 403)**. As afirmações sobre
Blender abaixo vêm de **resumos de busca sobre as páginas oficiais** (manual e release notes), não de
leitura direta da página. Estão marcadas **[DOC-indireto]** e devem ser reconferidas à mão antes de
qualquer decisão que dependa só delas.

**Preferência declarada:** onde houver equivalente permissivo, ele vence — **flecs (MIT)** para prefab
em ECS, **Godot (MIT)** para inspetor derivado, **OpenUSD (Apache-2.0)** para o modelo formal de
composição. Blender e Figma entram como **referência de comportamento e de UX**, nunca de implementação.

---

## §1 — B1: os seis modelos de composição

### §1.0 — A tabela de uma olhada

| Engine | Unidade do usuário | Como comportamento se acrescenta | Herança? | Custo do container vazio | Dor documentada |
|---|---|---|---|---|---|
| **Unity** | `GameObject` | `AddComponent` | não (composição pura) | `Transform` **obrigatório** | ordem de execução, `RequireComponent` não cobre múltiplas instâncias |
| **Godot** | `Node` | **criar um node filho** — a posição na árvore É a semântica | **sim**, hierarquia de classes | node cru é barato | explosão de tipos; comportamento amarrado à posição |
| **Unreal** | `Actor` | `AddComponent`, em 3 camadas (`UActorComponent`→`USceneComponent`→`UPrimitiveComponent`) | **sim**, e além disso o *Gameplay Framework* | Actor traz maquinaria de rede/tick | mistura herança + composição = duas perguntas para "onde ponho isto?" |
| **Bevy** | `Entity` | `insert(Component)` + **Required Components** (0.15+) | não | `Entity` vazia ≈ um índice | **discoverability**: não dá para ver o que vem junto (§1.4) |
| **flecs** | entidade | `add(Component)` + **`IsA`** (herança prototípica) | **sim, por relação** | idem | comportamento de instanciação depende de um *trait* por componente |
| **Construct/GDevelop** | objeto | **behavior** num diálogo | não | — | behavior é fechado; o event sheet é a válvula |

### §1.1 — Unity: a composição pura, e os dois atributos que a tornam usável

**[DOC]** `[RequireComponent(typeof(Rigidbody))]` faz a engine **inserir automaticamente** o componente
dependente quando o script é adicionado — *"This is useful to avoid setup errors… you are unlikely to
get the setup wrong."*
**[DOC]** `[DisallowMultipleComponent]` impede que o **mesmo tipo (ou subtipo)** seja adicionado duas
vezes ao mesmo GameObject.

**A decisão e o preço:**

- **Transform é obrigatório e não removível.** Isto não é preguiça: torna *"todo objeto tem lugar no
  mundo"* um invariante que hierarquia, gizmo, picking e serialização podem assumir sem `Option`. O
  preço é que um objeto puramente lógico (um gestor de estado) paga um Transform que ninguém lê.
- ⚠️ **`RequireComponent` NÃO consegue exigir *duas* instâncias do mesmo componente** — está no issue
  tracker oficial da Unity como comportamento conhecido **[DOC]**, e há pedido de feature aberto
  **[COM]**. É a fronteira exata do modelo: ele expressa *"precisa de um X"*, nunca *"precisa de dois X
  com papéis diferentes"*.
- **Consequência de projeto para o PH2D [INF]:** um mecanismo de dependência automática resolve 90 % do
  *"adicionei X e nada aconteceu"*, e deixa de fora exatamente o caso que o PH2D tem em abundância — a
  família `Area*` da física, os 7 tipos de zona, os joints. Ali a pergunta certa não é *"quantos X?"* e
  sim *"que papel este X ocupa?"*.

### §1.2 — Godot: por que nodes especializados, e onde dói

**[DOC/COM]** No Godot *"adicionar um comportamento" = criar um node filho na árvore* — a posição na
hierarquia **faz parte da semântica** (é literalmente a nota de tradução que o
[dossiê Godot](pesquisa/dossie_godot.md) já registrou: `PathFollow2D` **dentro** de `Path2D`,
`CollisionShape2D` com offset próprio).

**Onde ganha:** descoberta e clareza. O catálogo é uma lista de nomes concretos (`Camera2D`, `Area2D`,
`RayCast2D`) e o utilizador aprende por reconhecimento, não por composição. Um artista consegue nomear
o que quer.

**Onde dói:**
1. **Explosão de tipos** — `Sprite2D`, `AnimatedSprite2D`, `MeshInstance2D`, `MultiMeshInstance2D`,
   `NinePatchRect` são cinco tipos para *"desenhar uma imagem"*.
2. **Herança rígida** — mudar de `Sprite2D` para `AnimatedSprite2D` é trocar de objeto, não acrescentar
   um componente. **[INF]** É exatamente o anti-padrão que o [ADR-0074](../architecture/decisions/0074-sprite-component-boundary.md)
   do PH2D já nomeia (9-Patch como objeto separado no Construct).
3. **O comportamento fica amarrado ao lugar** — mover um node na árvore pode mudar o que ele faz.

### §1.3 — Unreal: três camadas de componente + papéis fora do objeto

**[DOC]** `UActorComponent` (comportamento sem presença) → `USceneComponent` (+ `Transform` e
attachment **dentro** do actor) → `UPrimitiveComponent` (+ geometria que renderiza/colide).

**A decisão que quase ninguém copia, e que é a mais interessante:** além dos componentes, a UE tem o
**Gameplay Framework** (`GameMode`/`GameState`/`PlayerState`/`Controller`/`Pawn`) — *classes-papel* que
respondem *"quem é dono da regra, quem é dono do estado do jogador, quem possui o corpo"*. O
[dossiê Unreal](pesquisa/dossie_unreal.md) §0 já tinha destilado a lição: **componentes sozinhos não
entregam "jogo de graça"; o par (componentes no objeto) + (papéis prontos fora do objeto) entrega.**

**O custo cognitivo, documentado pela prática [COM]:** com herança **e** composição disponíveis, toda
feature nova tem duas respostas plausíveis (*"subclasse de Actor?"* ou *"componente?"*), e a escolha
errada só aparece na terceira reutilização. É o problema que o [ADR-0074](../architecture/decisions/0074-sprite-component-boundary.md)
do PH2D resolve por **regra escrita** — e é a razão de essa regra valer ouro.

### §1.4 — Bevy: de Bundle para Required Components, e a crítica que importa

**[DOC]** Bevy 0.15 **deprecou todos os bundles embutidos** (`SpriteBundle`, `NodeBundle`,
`PbrBundle`…) em favor de **Required Components**: `#[require(Transform, Visibility)]` num componente
faz as dependências serem inseridas em cascata, com inicializador custom
(`#[require(Team(blue_team))]`) e registro em runtime.
**[DOC]** *"The `Bundle` trait will continue to exist and is still the fundamental building block for
insert APIs"* — bundles **não** foram removidos; deixaram de ser o idioma.

⭐ **A crítica, e ela é de UX, não de arquitetura** — discussão oficial
[bevyengine/bevy#16570](https://github.com/bevyengine/bevy/discussions/16570), *"Required Components —
There's a huge step backwards in usability that may have been overlooked"* **[COM, no repo oficial]**:

> *"The bundle I can see what I can customize just by mousing over the struct. The `SceneRoot` will
> insert `Transform` and `Visibility` components but how do I know what I can customize without having
> to go look up the component in docs?"*
>
> *"I don't even know what I'm about to insert without looking at docs. Instead of being able to look
> at a bundle and go 'ooh, that's probably what I'm looking for' I have to go back to docs to find the
> component by hand each and every time."*

As mitigações propostas na discussão são **todas de ferramenta** (Ctrl+clique no IDE, renderizar a doc
do componente no tooltip); nenhuma resposta definitiva de mantenedor fechou o assunto.

⭐ **Por que isto é a lição mais transferível de todas [INF]:** a queixa é *"não vejo o que vem junto"*.
Num compilador isso é fricção. **Numa engine com editor, é a especificação de um widget:** o botão
*Add Component* tem de **mostrar a cascata antes de a aplicar**. A dependência automática resolve o erro
de setup; ela **cria** um problema de visibilidade — e a UI é o lugar onde esse problema é barato.

**[DOC]** Outros footguns do idioma required-components estão no próprio tracker: *"Using a raw `Camera`
component is a footgun"* ([issue #19299](https://github.com/bevyengine/bevy/issues/19299), 2025) —
inserir a peça errada da cascata dá um objeto meio-formado que compila e não funciona.

**Composição serializável no ecossistema Rust — o que falhou [DOC/COM]:** `bevy_scene::DynamicScene`
depende de `bevy_reflect` + `TypeRegistry`, e o histórico de issues é uma lista de modos de falha do
mesmo tipo: componente não registrado → *"no registration found for dynamic type"*; handles de asset sem
`ReflectSerialize` não serializam ([#21985](https://github.com/bevyengine/bevy/issues/21985)); structs
aninhados falhando na desserialização ([#10499](https://github.com/bevyengine/bevy/issues/10499)).
⭐ **O PH2D já evitou isto por construção:** o `ComponentRegistry` é **manual**, o formato é **postcard
posicional** com `type_id = blake3(nome)`, e a falha *"tipo não registrado"* é um erro alto no
`spawn_prefab`, não um silêncio. O preço que o PH2D paga em troca está no §2.8.

### §1.5 — ⭐ flecs: o prior art mais sofisticado de "prefab dentro de ECS"

**[DOC — `flecs.dev`, licença MIT]**

- **Prefab é entidade comum com a tag `EcsPrefab`**; a única diferença é que *"queries don't match them
  by default"*. Instanciar = `entity().is_a(Prefab)` — a relação `IsA`.
- ⭐ **O trait `(OnInstantiate, ...)` por COMPONENTE, com três modos** — e este é o achado:
  - **`Inherit`** — o componente fica **só no prefab**; instâncias o leem por herança (**partilhado**;
    mudar no prefab muda em todas as instâncias, ao vivo).
  - **`Override`** — o componente é **copiado** para a instância na instanciação.
  - **`DontInherit`** — não viaja.
  - **[DOC]** *"for a component to be inheritable, it needs to have the `(OnInstantiate, Inherit)`
    trait"* — ou seja, em v4 **copiar é o default e herdar é opt-in por tipo**.
- **Override em runtime:** ao escrever num componente herdado, *"the component on the instance is
  initialized with the value from the prefab component"* e a instância passa a **possuir** a sua cópia.
  A partir daí, mudar o prefab **não** a alcança. `ecs_owns()` responde *"isto é meu ou herdado?"*.
- ⭐ **Prefab slots** — resolvem *"como me refiro a um filho específico da instância?"*. O filho do
  prefab é declarado como slot; a instanciação cria uma **relação na instância** cujo alvo é o filho
  instanciado. **Endereçamento por relação, não por nome nem por índice.**
- **Hierarquia:** *"the entire subtree of a prefab is copied to the instance"*, e ⚠️ *"child entities
  **never** inherit components from prefab children"* — só a raiz herda; os filhos são cópias.
  > ⚠️ **CORREÇÃO por leitura de código (2026-08-21, noite — [flecs_bevy_internals §A](pesquisa/instancias_2026-08-21/flecs_bevy_internals.md)):**
  > a frase acima é verdade **só no storage `ChildOf`**. No storage **`Parent`** (o recomendado para
  > prefabs, `HierarchiesManual` L419-421) *"instance children will **inherit** from the prefab
  > children"* — cada peça instanciada carrega **`(IsA, filho_do_prefab)`** (`tree_spawner.c` L60), e é
  > endereçada por índice posicional + validação desse `IsA`, com erro explícito *"children of '%s'
  > have changed since prefab instantiation"*. **E mudar os filhos de um prefab já instanciado ABORTA**
  > (*"cannot change children of prefab after it has been instantiated"*). Ou seja: o flecs tem o
  > vínculo peça→peça-mestre, mas **proíbe** o re-sync estrutural em vez de o implementar.
- **Variantes:** um prefab faz `IsA` de outro e sobrescreve componentes (`Freighter is-a SpaceShip`).
  **Zero mecanismo novo** — variante é herança de prefab.

⭐ **A lição de projeto, e é grande [INF]:** o flecs põe a escolha *"partilhado ao vivo × copiado"*
**no TIPO do componente**, não na instância nem no gesto do utilizador. Isso torna a pergunta
*"editar o mestre propaga?"* respondível **por componente**, e é exatamente a distinção que o briefing
chama de *"asset vivo × asset congelado"* (§B5 do outro doc). O preço: é uma decisão que o autor da
crate toma, e um utilizador não a vê.

**⚠️ Nota de honestidade sobre a busca:** meu fetch do manual não conseguiu confirmar textualmente o nome
`DontInherit` (a página descreve os três comportamentos; o terceiro apareceu nos resultados de busca).
Reconfira antes de citar o identificador exato.

### §1.6 — Fora dos games — e é aqui que o PH2D tem mais parentes

O briefing pediu peso proporcional. Dou-o: **o PH2D é uma ferramenta de autoria com runtime, não um
runtime com editor.** Os quatro modelos abaixo pesam mais do que Unity/Godot juntos.

#### Blender — objeto + object data + modifier stack + geometry nodes **[DOC-indireto]**

O modelo tem **quatro camadas** onde uma engine tem duas: o **Object** (pose, nome), o **Object Data**
(a malha/curva, partilhável entre objetos — o *"linked duplicate"*), a **pilha de modificadores**
(ordenada, não-destrutiva, cada um com parâmetros), e os **geometry nodes** (um DAG dentro de um
modificador). ⭐ **Consequência:** *"o mesmo desenho em dois lugares"* e *"o mesmo desenho com um efeito
a mais"* são **mecanismos distintos** — data partilhada vs. modificador local. Uma engine que só tem
prefab responde às duas com a mesma ferramenta e força o utilizador a escolher entre propagar tudo ou
nada.

⭐ **E o PH2D já tem a pilha:** os **Live Path Effects** são *"uma pilha por-path, não um grafo de nós"*
([ADR-0132](../architecture/decisions/0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md)) —
a mesma decisão do modifier stack, tomada independentemente.

#### Houdini — tudo é operador, e o HDA é o "componente" **[DOC — sidefx.com]**

⭐ **O mecanismo decisivo, e o PH2D pode copiá-lo hoje:** *"When you **promote** a parameter, Houdini
creates a **copy** of the parameter on the asset, and **replaces the original parameter's value with an
expression that references** the value of the parameter on the asset."*

Ou seja: a interface pública de um asset composto **não é uma lista de campos declarados** — é uma lista
de **referências a parâmetros internos**, autorada por arrastar-e-largar na janela *Type Properties*. O
autor do asset **desenha o painel**; o utilizador nunca vê o interior.

**[INF]** Isto é a resposta de Houdini à mesma pergunta do §1.4 (Bevy): em vez de *"como mostro tudo o
que vem junto?"*, **"mostro exatamente o que o autor decidiu expor"**. É o modelo oposto e resolve o
mesmo problema. ⚠️ E o PH2D **já tem o substrato**: o Vector tem *"a árvore autorada como painel vivo"*
(`VecWidget`/`VecWidgetBind`/`VecWidgetValue` — o app escreve o código do painel), e o Motion tem
`param_source` no nodegraph.

#### Figma — components / variants / instances / overrides **[DOC — help.figma.com]**

Quatro tipos de *component property* declaráveis no mestre: **Variant** (eixos com valores nomeados),
**Boolean** (⚠️ *"currently only available for layer visibility"*), **Instance swap** (que instâncias
podem ser trocadas), **Text**.

⭐ **A frase que decide tudo, e é uma limitação confessa [DOC]:** *"Figma records the changes you make to
an instance and preserves them, even when you swap between instances or select different variants.
**However, Figma only preserves text overrides. To keep any changes you've made to text layers, rename
the layers so they're unique.**"*

**Leia isso duas vezes.** O sistema de design mais usado do mundo **casa override por NOME de camada**, e
a instrução oficial ao utilizador é *renomeie as camadas para serem únicas*. É exatamente:
- o mecanismo que o PH2D já usa (`stable_name_id`, hash do `Name`, com `name_unique.rs` a impor
  unicidade no editor), **e**
- a fragilidade que a auditoria (§1.3 do doc 01) nomeou: **renomear desliga**.

**[INF]** Portanto: *"casar override por nome é frágil"* é verdade **e** é o que a ferramenta líder do
mercado faz, de propósito, porque a alternativa (id opaco por camada) torna *"trocar o mestre por outro
parecido"* impossível. O nome é o que permite o **swap**.

#### Rive — artboards aninhados + data binding **[DOC — rive.app, DeepWiki do runtime]**

O parente mais próximo do PH2D (o motor vetorial é *referenciado* no runtime MIT do Rive,
[ADR-0108](../architecture/decisions/0108-vector-reposition-rive-referenced-native-editor-first.md)).
Duas peças:
1. **Nested artboards** — um artboard dentro de outro, com constraints/animações preservadas.
2. ⭐ **Data binding por ViewModel (MVVM)** — o mestre declara um **ViewModel** com propriedades; a
   instância **liga dados** a elas. Property paths aninhados (`"nested/property/name"`). Em 2025
   acrescentaram **Data Binding Artboards**: *trocar a fonte de um artboard em runtime*, carregando de
   outro ficheiro, preservando constraints e posição.

**[INF]** O Rive fez a mesma escolha do Houdini e do Figma e **contra** a das engines: **a superfície de
override é uma lista de propriedades NOMEADAS que o mestre declara**, não o diff estrutural do que a
instância mudou.

#### After Effects — pre-comp

O modelo mais simples e o mais honesto sobre o seu preço: uma composição vira um objeto dentro de outra.
**Não há override nenhum** — para variar, duplica-se a comp. **[INF]** É o baseline contra o qual medir
se o override vale a complexidade: se 90 % das instâncias forem idênticas, o pre-comp basta.

---

### §1.7 — ⭐ A pergunta central: componente ECS, nó de DAG e item do inspetor são a mesma abstração?

O briefing pede os dois lados antes da posição. Aqui estão.

#### Lado A — SIM, unifique (uma abstração só)

1. **Prova de existência à mão:** o `ParamRow` do PH2D já tem **12 variantes tipadas** e produz painel
   para ~180 tipos de nó. Se um componente ECS publicasse a mesma coisa, **o painel do Motion e o
   Inspector seriam o mesmo código**, e o custo de UI de todo o catálogo de ~150 componentes cairia de
   *artesanal* para *zero*.
2. **Houdini prova que a fusão escala:** lá **tudo é operador**, e um HDA é ao mesmo tempo o nó, o
   componente e a interface. Não é hipótese; é uma ferramenta de 30 anos.
3. **Menos conceitos para o utilizador.** O Enio não conhece as ferramentas por dentro (CLAUDE.md §0.8):
   *"o que se acrescenta a um objeto"* ser **uma** coisa é o melhor produto.
4. **O ADR-0075 não proíbe** — ele proíbe plugin em runtime, e manda desacoplar por ECS. Uma abstração
   comum de *descrição de parâmetro* é dado, não plugin.

#### Lado B — NÃO, mantenha duas camadas com ponte explícita

1. ⚠️ **Elas têm relógios diferentes, e isso não é detalhe.** O DAG do PH2D coza sob `Effect::Pure |
   Temporal`, com memo re-chaveado por `(NodeId, ScopeKey)`, escopos de tempo e **`pre` (feedback)** —
   e recusa *"nó sequencial dentro de escopo remapeado"* (SKILL §11.13). Um componente ECS não tem
   nada disso: ele é config lida por sistemas num tique fixo. Fundir importa a máquina de tempo do DAG
   para dentro de todo componente, ou amputa-a.
2. ⚠️ **Elas têm undos diferentes, e isso é medido.** O undo do ECS é **diff de snapshot do mundo
   inteiro** (6,89 ms a 10 k entidades, §7 do doc 01); o do Motion é o `MotionHistory` próprio. O Enio
   **já separou** esses escopos explicitamente (comentário no `project.rs`: *"enfiar o grafo ali dentro
   faria cada Ctrl+Z do canvas rebobinar o grafo junto"*). Unificar reabre uma decisão de produto já
   tomada.
3. ⚠️ **Elas têm formatos de persistência opostos, de propósito.** O grafo é **texto diffável por
   linha** — requisito multiagente que descartou JSON/RON. O mundo é **postcard posicional**. Fundir
   escolhe um dos dois e perde a razão pela qual o outro foi escolhido.
4. ⚠️ **O contrato de nó está CONGELADO** (`NodeOp=2`/`OpResolver=1`/`NodeManifest=8`,
   [ADR-0039](../architecture/decisions/0039-nodegraph-contract-freeze-w2t4.md), §6 do CLAUDE.md).
   Qualquer fusão mexe nele ⇒ Coord-only + ADR.
5. **A dependência é assimétrica e limpa hoje:** `ph2d-nodegraph` não conhece `bevy_ecs`; 125 crates de
   nó dependem dela. Fundir cria uma dependência de 125 crates no ECS.

#### A leitura honesta [INF]

**Os dois lados discutem coisas diferentes, e é isso que destrava:**

- O **lado A** só precisa de **UMA** coisa: um vocabulário comum de **DESCRIÇÃO DE PARÂMETRO** (o
  `ParamRow`), para que um painel só saiba pintar as duas famílias.
- O **lado B** defende três coisas que **não** estão nesse vocabulário: o **relógio**, o **undo** e o
  **formato**.

Ou seja: **é possível unificar o item do inspetor sem unificar componente e nó.** A abstração comum
seria uma *terceira* coisa — um **descritor**, não um dono de dado — e as duas famílias a produziriam.
Isto **não é a decisão** (ela é da Fase D); é a constatação de que a pergunta do briefing admite uma
resposta que não é nenhum dos dois extremos.

---

## §2 — B2: prefab — quatro modelos de override

### §2.1 — Unity: diff estrutural com apply/revert **[DOC]**

**Três categorias de override:** valor de propriedade modificado · componente **adicionado ou
removido** · GameObject filho **adicionado ou removido**. Sinalizados no Inspector (linha azul, badges
+/−) e geríveis por dropdown (*apply/revert* tudo, seleção, ou individual).

⭐ **A regra que gera todo o comportamento:** *"An overridden property value on a prefab instance always
takes precedence over the value from the prefab asset"* — **override bloqueia a propagação**. Uma vez
que você tocou num campo, mudar o mestre **deixa de te alcançar naquele campo**, para sempre, em
silêncio.

**Nested prefabs + variants (2018.3):** demorou **13 anos** a chegar (pedido nº 1 desde 2005) **[COM]**.
Com aninhamento, um override pode ir para **níveis diferentes** e a UI tem de **perguntar a qual
mestre** aplicar.

**Limitações documentadas [DOC]:** *"If you modify or delete any scripts that declare the value of an
instance override, then the override becomes **unused**"* — e fica lá, morto, a precisar de limpeza.
`Transform` position/rotation **deliberadamente não contam** como override explícito. E mudanças
estruturais no mestre *"risk misalignment with existing instance overrides"*.

**[INF] O preço do modelo, em uma frase:** o diff estrutural é fácil de **produzir** (basta comparar) e
difícil de **manter vivo** — porque a chave do diff é a estrutura, e a estrutura é justamente o que o
autor do mestre tem o direito de mudar.

### §2.2 — Godot: instanciar cena, herdar cena, editable children **[DOC/COM]**

Três mecanismos distintos, e a distinção importa:

1. **Instanciar** uma cena — a instância mostra a raiz; os filhos são invisíveis/não-editáveis por
   default.
2. **Editable children** — abre os filhos da instância para edição **mantendo o vínculo**.
3. **Scene inheritance** — *"traz a ideia de herança de classe para as cenas"*: a cena-filha herda
   estrutura e funcionalidade, e pode sobrescrever propriedades.

⭐ **Mesma lei do Unity, dita de outro jeito [COM]:** *"When a property in an inherited scene is
overwritten, it stays overwritten even if the base scene's value changes later"*.

⚠️ **A patologia concreta, num issue aberto do repo oficial [DOC]**
([godotengine/godot#111807](https://github.com/godotengine/godot/issues/111807)): ao abrir uma cena
herdada no editor, *"its local-to-scene resources get duplicated and saved as overrides"* — a cena
**desliga-se** de edições ao recurso original e *"leaving the user no way to make edits that
propagate"*. **[INF]** É o mesmo mecanismo do Unity a morder por um caminho diferente: **o editor cria
overrides que o utilizador não pediu**, e o vínculo morre em silêncio.

> ⚠️ **PRECISÃO por leitura de código (2026-08-21, noite — [propagacao_unity_godot §(b)](pesquisa/instancias_2026-08-21/propagacao_unity_godot.md), MIT):**
> o Godot **não armazena overrides — deriva-os por diff no `pack()`** contra a pilha de `SceneState`s
> (`_parse_node`, `packed_scene.cpp:865-1208`), chaveados por **`NodePath` + propriedade**; um alvo que
> sumiu é **descartado em silêncio com WARN** (*"…was modified from inside an instance, but it has
> vanished"*). Desde o **4.6** ([PR #106837](https://github.com/godotengine/godot/pull/106837), merged
> 2025-10-06, fecha 7 issues desde 2018) existe um `unique_id` int32 por nó — **só como fallback** do
> caminho (*"By using IDs first, you have the potential of creating new problems"*). E a propagação
> mestre→instância no editor é **reinstanciar a cena inteira** e **apagar o histórico de undo da aba**
> (`EditorUndoRedoManager::clear_history`). O #111807 é um caso à parte (sub-recursos
> `resource_local_to_scene`), não o mecanismo geral de override.

### §2.3 — Unreal: herança como mecanismo de reuso **[DOC/COM]**

- **Blueprint child classes** — reuso por **subclasse**: o filho herda componentes e valores default do
  pai e sobrescreve o que quiser.
- **Child Actor Component** — um Actor **dentro** de outro (o análogo do nested prefab), com o custo de
  ser um actor completo (ciclo de vida, rede).
- **Data Assets** — dados puros, sem comportamento, referenciáveis.

**[INF] O desconforto documentado pela prática:** usar herança como reuso significa que *"acrescentar
uma variação"* e *"acrescentar um tipo"* são o mesmo gesto — e a árvore de classes cresce com cada
variante de conteúdo. É o que o [ADR-0025](../architecture/decisions/0025-gameobject-model.md) do PH2D
já rejeitou (*"Sem `trait Node` polimórfico"*).

### §2.4 — Figma: override por PROPRIEDADE NOMEADA **[DOC]**

O mestre **declara** o que pode variar (as 4 *component properties* do §1.6); a instância só mexe
nisso. Variants são um **espaço de propriedades** (`Size=Small, State=Idle`), não N cópias.

⭐ **A diferença estrutural contra Unity/Godot:** lá o override é *"o que esta cópia mudou"* (descoberto
por diff, aberto, ilimitado). Aqui é *"o que o mestre permitiu variar"* (declarado, fechado, pequeno).
**Consequência:** o autor do mestre pode reestruturar tudo por baixo sem quebrar instância nenhuma —
desde que os **nomes das propriedades** sobrevivam.

⚠️ **E o preço confesso é o do §1.6:** só overrides de **texto** sobrevivem a swap/variant, e a
instrução oficial é *renomeie as camadas para serem únicas*.

### §2.5 — OpenUSD: o modelo formal, e o mais rigoroso que existe **[DOC]**

Composição por **arcos**, com ordem de força canônica **LIVRPS** — hoje **LIVERPS**, porque o arco
`relocates` entrou:

| | Arco | O que faz | Força |
|---|---|---|---|
| **L** | *Local / sublayers* | opiniões diretas na layer stack raiz | mais forte |
| **I** | *Inherits* | *"add overrides to existing (instanceable) prims"* sem perder instanciamento | ↓ |
| **E** | *rElocates* | renomeações/movimentações de namespace | ↓ |
| **V** | *VariantSets* | comuta entre variações **inline** de uma sub-hierarquia | ↓ |
| **R** | *References* | agrega dados de outro ficheiro/hierarquia; **encapsula** | ↓ |
| **P** | *Payloads* | como reference, mas **carregável sob demanda** (geometria pesada) | ↓ |
| **S** | *Specializes* | valores-**template** de base que tudo acima sobrescreve | mais fraco |
| **F** | *Fallback* | default do schema | — |

**As quatro propriedades que nenhum outro modelo tem:**

1. ⭐ **`over` — o "prim mais fraco".** Um `over` **edita sem definir**: assume que o prim existe noutro
   sítio da pilha. É a materialização de *"override é uma camada, não uma mutação"*.
2. ⭐ **`specializes` é o oposto de `inherits`, e é a peça que falta em toda engine.** Ele fornece
   *"template baseline values"* que arcos mais fortes sobrescrevem — *"desirable… without changing
   anything that makes refinements unique"*. Ou seja: **atualizar o template NÃO desfaz o refinamento**
   — exatamente o modo de falha do Unity/Godot (§2.1/§2.2), resolvido por **força**, não por regra.
3. **Arcos são LISTAS list-editáveis** (`prepend`/`append`/`delete`), e reordenar muda a precedência.
4. **Encapsulamento nas fronteiras:** conteúdo referenciado/payload **encapsula** — mudanças estruturais
   no ficheiro referenciado **não propagam automaticamente**; arcos internos ficam *"locked once
   composed"*. **[DOC]** *"This design trades flexibility for predictability at composition boundaries"*.

**Renomear/reordenar [DOC]:** *"Arc paths use path mapping — renaming the prim holding an arc doesn't
break the arc"*, mas ⚠️ *"deleting referenced prims breaks downstream inherits/specializes targeting
those paths"*.

**[INF] O que o PH2D pode tirar disto sem adotar USD:** três ideias, e são independentes umas das outras.
(a) **override é uma camada com FORÇA**, não uma mutação; (b) existe um nível **mais fraco que o
default** (`specializes`) onde um template pode ser atualizado sem apagar refinamentos; (c)
**encapsulamento na fronteira** é uma escolha de previsibilidade, não uma limitação.

### §2.6 — Blender: linked libraries, library overrides, e por que os proxies falharam **[DOC-indireto]**

- **Link** traz o data-block de outro ficheiro, **read-only**. **Library override** cria uma camada local
  editável por cima.
- **Proxies foram DEPRECADOS na 3.0 e REMOVIDOS na 3.2.** A conversão automática existe mas *"results on
  complex characters are not guaranteed and may need manual fixes"*.
- ⭐ **Resync** é o conceito que só o Blender nomeia: *"the relationships between linked data-blocks can
  change, resulting in outdated overrides… overrides need to be resynced to match the new structure of
  their hierarchy"*. E *"resync enforce"* **reseta** o override ao estado *"freshly created"*.
- **Limitações documentadas:** se o dado linkado tem animação, o override tem *"only limited
  possibilities to edit the existing drivers"* — dá para mudar o alvo de um driver, **não** para
  acrescentar drivers.

⭐ **A lição, e é a mais dura da secção [INF]:** o Blender é a única fonte que trata **"o mestre mudou
de forma"** como um **evento de primeira classe com uma operação própria** (*resync*), em vez de um
acidente. Unity e Godot chamam a isso *"risk of misalignment"*; o USD resolve por força de arco; o
Blender **tem um botão**. E ainda assim é a área com mais issues abertos
([#83811 *"Investigate Resync Improvements"*](https://projects.blender.org/blender/blender/issues/83811)).

### §2.7 — flecs: ver §1.5

Resumo da diferença: o flecs é o único que decide *"partilhado ao vivo × copiado"* **por tipo de
componente**, no `(OnInstantiate, ...)`, antes de qualquer instância existir.

---

### §2.8 — ⭐ A tabela que decide: como cada modelo ENDEREÇA um override

**Esta é a pergunta operacional do briefing** (*"como representar override sem que renomear/reordenar
quebre a ligação?"*). A resposta de cada sistema é a sua chave de endereçamento:

| Modelo | Chave do override | Sobrevive a **renomear** | Sobrevive a **reordenar** | Sobrevive a **mestre mudar de estrutura** | Propagação do mestre |
|---|---|---|---|---|---|
| **Unity** | caminho hierárquico + `fileID` do objeto/propriedade | ⚠️ parcial | ⚠️ parcial | ❌ *"risk misalignment"*; override vira *unused* | ❌ bloqueada no campo tocado |
| **Godot (cena)** | `NodePath` + nome da propriedade | ❌ | ✅ | ❌ (e ainda duplica recurso, #111807) | ❌ bloqueada |
| **Unreal (BP child)** | herança de classe + nome do componente | ❌ | ✅ | ⚠️ compila e falha em runtime | ✅ (é herança) |
| **Figma** | **nome da propriedade declarada** (+ nome da camada, para texto) | ❌ *(a doc manda renomear para ser único)* | ✅ | ✅ **se os nomes sobreviverem** | ✅ (o não-declarado propaga) |
| **Rive** | **property path do ViewModel** (`nested/prop/name`) | ❌ | ✅ | ✅ | ✅ |
| **USD** | **path do prim + arco com FORÇA** | ✅ *(path mapping)* | ✅ *(a ordem é a força, e é explícita)* | ⚠️ apagar o prim quebra inherits/specializes | ✅ **e com `specializes` sobrevive ao update do template** |
| **Blender** | hierarquia de override + **resync** explícito | ⚠️ | ⚠️ | ⚠️ **tratado como evento, com botão** | ✅ |
| **flecs** | ownership por **componente** (`ecs_owns`) + **slots** por relação | ✅ *(relação, não nome)* | ✅ | ✅ *(o componente é a unidade)* | ✅ para `Inherit`, ❌ depois de override |
| **PH2D `VecInstance` (hoje)** | `(sub = VecPathId do MESTRE, slot = espécie)` | ✅ **(id, não nome)** | ✅ *(lista canônica por `(sub, kind)`)* | ✅ *(o comentário do código diz: "a peça continua a mesma peça mesmo que a geometria mude por inteiro")* | ✅ (geometria derivada por frame) |

⭐ **Duas leituras que saltam desta tabela [INF]:**

1. **O PH2D já implementou, no vetor, o modelo que a tabela indica como o mais robusto** — chave por
   **id do mestre** + **espécie fechada** + **lista canônica** + **derivação por frame**. Ele bate o
   Unity e o Godot nas quatro colunas, e o que lhe falta contra Figma/Rive é vocabulário
   (**2 espécies**) e contra o USD é **força** (não há camada mais fraca que o default).
2. **Ninguém sobrevive a renomear**, exceto quem endereça por **id opaco** (USD, flecs, PH2D-vetor). E o
   PH2D **tem os dois esquemas ao mesmo tempo**: `VecInstance` usa id; timeline, joints e rótulos usam
   `stable_name_id`, que **quebra ao renomear**. É uma divergência interna real, e ela é do §8 do doc 01.

---

## §3 — B3: UX do inspetor

### §3.1 — O padrão *Add Component*

| Engine | Gesto | Descoberta | Dependências |
|---|---|---|---|
| Unity | botão + **janela de busca com categorias** e criação inline de script novo | busca fuzzy por nome | `RequireComponent` insere em cascata **[DOC]** |
| Unreal | botão `+ Add` no painel de componentes, busca por classe | busca + árvore de classes | herança |
| Godot | *"Add Child Node"* — **catálogo em árvore com busca e descrição** | por reconhecimento de nome | nenhuma (a árvore é a semântica) |
| Construct 3 | diálogo *"Add behavior"* com ícones e categorias **[dossiê]** | ícones + categoria | nenhuma |
| Bevy | ❌ não há editor; é código | ⚠️ **a queixa do §1.4** | required components |

**[INF] O que a comparação diz:** todas convergiram em **busca sobre um catálogo categorizado**, e a
diferença real está em **o que a UI mostra ANTES de aplicar**. Nenhuma delas mostra a cascata — e é
por isso que a queixa do Bevy existe mesmo em engines com editor (`RequireComponent` do Unity insere
sem avisar).

### §3.2 — Disclosure progressivo — os quatro padrões

1. **Blender** — **tabs de propriedades** (uma coluna de ícones: objeto, modificadores, física,
   material…) **+ painéis colapsáveis** dentro de cada tab. **[DOC-indireto]** O painel nunca mostra dois
   domínios ao mesmo tempo.
2. **Unreal Details** — **[DOC]** três mecanismos combinados: **busca que filtra ao vivo**
   (`SFilterableDetail` esconde tudo o que não casa), **Favorites** (clicar na estrela põe a propriedade
   numa secção *Favorites* no topo — ⚠️ e a doc avisa que *"some properties may not offer this ability
   due to customization complexity"*), e **secções Advanced** colapsadas.
3. **Houdini** — **[DOC]** a interface é **autorada por asset** (§1.6): o autor decide as abas, a ordem,
   os rótulos e o que sequer aparece.
4. **Figma / Substance** — painel **contextual ao que está selecionado**, com as propriedades declaradas
   pelo componente e nada mais.

⭐ **A ordem de eficácia [INF]:** *autorado* (Houdini) > *declarado pelo mestre* (Figma) > *busca +
favoritos* (Unreal) > *tabs + fold* (Blender) > *lista plana*. E note que os **dois melhores não são
recursos de UI — são decisões de MODELO DE DADOS**: só se pode autorar/declarar a interface se o dado
souber descrever-se.

### §3.3 — Inspetor data-driven × hard-coded

⭐ **Godot é o exemplo canônico de "schema em dados", e é MIT [DOC]:**

- O inspetor é construído a partir de `Object.get_property_list()` — um **array de dicionários**, cada
  um com `name`, `type` e, opcionalmente, `hint` (`PropertyHint`), `hint_string` e `usage`.
- *"EditorInspector will show properties in the same order as the array returned by
  `get_property_list()`"* — **a ordem do painel é dado**.
- ⭐ *"If a property's name is **path-like** (contains forward slashes), EditorInspector will create
  **nested sections** for 'directories' along the path"* — `highlighting/gdscript/node_path_color` vira
  *Node Path Color* dentro de *GDScript* dentro de *Highlighting*. **O agrupamento também é dado, e vem
  de graça no nome.**
- `EditorInspectorPlugin` é o ponto de extensão para substituir o editor de uma propriedade específica —
  **o escape para o caso que o schema não exprime**.

**Unreal** faz o contrário e chega perto: o painel é gerado por reflexão sobre `UPROPERTY`, e
**Details Customization** (Slate) é o escape — mais poderoso e muito mais caro.

⭐ **A lição de qualidade visual [INF]:** *"data-driven fica genérico e feio"* é falso como lei — é
verdade quando o **vocabulário** é pobre. Godot tem `PropertyHint` (range, enum, file, layers, curve,
color-no-alpha…); Unreal tem meta-specifiers; o PH2D **já tem 12 variantes de `ParamRow`** com unidade,
teto duro, faixa dupla, seções e *modified*. **A beleza vem do vocabulário + do escape**, não de escrever
cada painel à mão.

### §3.4 — A restrição do PH2D: caneta e tablet

Nenhuma das cinco referências acima foi desenhada para stylus. As implicações são do PH2D, e o repo já
tem material:

- **Alvos de toque** — o design system tem tokens de densidade e `ph2d-a11y` (HR-12); o `NumberInput` já
  tem *drag-slider* estilo Blender com axis-lock a 4 px e *continuous-hold* de 250/30 ms (SKILL §11.9).
- ⚠️ **Drag-and-drop com caneta é o gesto mais caro** — e é exatamente o que um Asset Browser exige
  (arrastar do painel para o canvas). ⚠️ **E o PH2D tem um dado adverso já medido**: o `winit` escreve
  `force: None` nos três backends e o Wayland não expõe `zwp_tablet_v2`
  (§5 do CLAUDE.md, `vec_pencil_input.rs`) — *o tablet está mal precificado*.
- **Menus radiais / marking menus** já são pesquisa aberta do módulo Vector (**E4**, §5 do CLAUDE.md) e
  são o candidato natural para *Add Component* sob caneta.

### §3.5 — O caso concreto: fatiar a Sprite sem tornar "criar uma sprite" uma tarefa de 6 cliques

O briefing diz que presets/arquétipos são "a resposta óbvia" e pede validação. **Validação: sim, mas é
a resposta de SEGUNDA ordem — a de primeira é a dependência automática.** Prova por comparação:

| Mecanismo | Quem usa | O que resolve | O que NÃO resolve |
|---|---|---|---|
| **Dependência automática** | Unity `RequireComponent`, Bevy required components, flecs `IsA` | *"adicionei X e faltou Y"* | descoberta (§1.4) |
| **Preset / arquétipo** | Unity prefabs de projeto, Godot cenas-modelo, Construct *"objeto pronto"* | *"quero o pacote completo num clique"* | ⚠️ **divergência**: preset é cópia, e mudar o preset não alcança o que já foi criado |
| **Componente-fachada** | Godot (`Sprite2D` = um node), Rive (artboard) | manter o gesto simples **e** o dado fatiado | precisa de um conceito a mais |

⭐ **[INF] E o PH2D tem um dado que decide isto:** a Sprite **já está fatiada** (19 componentes opcionais
fora do struct) e **criar uma sprite continua sendo um gesto só** — porque os opcionais são *ausentes por
default* e o `Sprite::atlas()` preenche os 20 campos. Ou seja, **a fatiação já provou não custar cliques
nesta engine.** O risco do briefing é real para *fatiar mais* (folha inline, região, gradiente de cantos),
e a mitigação medida é a mesma: **ausência = default benigno** (o teste decisivo do
[ADR-0074 §2.2](../architecture/decisions/0074-sprite-component-boundary.md)).

---

## §4 — O que disto é aplicável ao PH2D (ponteiros, sem escolher)

Sem recomendação — a Fase D decide. Apenas o que a pesquisa **põe na mesa**:

1. **Um vocabulário de descrição de parâmetro já existe na casa** (`ParamRow`, 12 variantes) e é o
   substrato de um inspetor derivado. O Godot prova o padrão com `get_property_list` + `PropertyHint` +
   `EditorInspectorPlugin` como escape; a secção aninhada **de graça pelo nome path-like** é um truque
   barato e testado.
2. **O modelo de override mais robusto da tabela §2.8 já está implementado no PH2D**, no vetor. O que lhe
   falta é **vocabulário** (2 espécies) e **força** (não há nível mais fraco que o default, à la
   `specializes`).
3. **A dependência automática resolve o erro de setup e cria o problema de visibilidade.** A queixa do
   Bevy é a especificação do widget: *mostre a cascata antes de aplicar*.
4. **Houdini e Rive dizem a mesma coisa por caminhos diferentes:** a superfície pública de um objeto
   composto deve ser **autorada/declarada**, não descoberta por diff. O PH2D já tem as duas sementes
   (`VecWidget*` e `param_source`).
5. **O Blender é o único que trata *"o mestre mudou de forma"* como operação nomeada (*resync*)** — e
   continua a ser a área com mais bugs abertos. Qualquer plano que prometa override tem de dizer o que
   faz nesse dia.
6. ⚠️ **Nenhum dos modelos sobrevive a renomear, exceto os que endereçam por id opaco.** O PH2D usa os
   **dois** esquemas hoje (§2.8) — e isso é uma divergência interna, não uma escolha.

---

## §5 — Fontes

**Bevy** (MIT/Apache-2.0)
- [Bevy 0.15 — Required Components](https://bevy.org/news/bevy-0-15/) · [Migration 0.14→0.15](https://bevy.org/learn/migration-guides/0-14-to-0-15/) · [0.15→0.16](https://bevy.org/learn/migration-guides/0-15-to-0-16/) · [0.16→0.17](https://bevy.org/learn/migration-guides/0-16-to-0-17/)
- ⭐ [Discussão #16570 — a crítica de usabilidade](https://github.com/bevyengine/bevy/discussions/16570) · [Issue #19299 — raw `Camera` é footgun](https://github.com/bevyengine/bevy/issues/19299) · [Issue #17369 — bundles deprecados geram warning](https://github.com/bevyengine/bevy/issues/17369)
- `DynamicScene`: [#21985](https://github.com/bevyengine/bevy/issues/21985) · [#10499](https://github.com/bevyengine/bevy/issues/10499) · [#6627](https://github.com/bevyengine/bevy/issues/6627) · [docs.rs](https://docs.rs/bevy/latest/bevy/scene/prelude/struct.DynamicScene.html)

**flecs** (MIT)
- ⭐ [Prefabs Manual](https://www.flecs.dev/flecs/md_docs_2PrefabsManual.html) · [Relationships](https://www.flecs.dev/flecs/md_docs_2Relationships.html) · [Designing with Flecs](https://www.flecs.dev/flecs/md_docs_2DesignWithFlecs.html)

**Unity** (proprietária — só documentação)
- [Instance overrides](https://docs.unity3d.com/Manual/PrefabInstanceOverrides.html) · [Nested Prefabs](https://docs.unity3d.com/2023.2/Documentation/Manual/NestedPrefabs.html) · [Prefab Variants](https://docs.unity3d.com/Manual/PrefabVariants.html) · [Overrides at multiple levels](https://docs.unity3d.com/2018.3/Documentation/Manual/PrefabOverridesMultiLevel.html)
- [`RequireComponent`](https://docs.unity3d.com/ScriptReference/RequireComponent.html) · [`DisallowMultipleComponent`](https://docs.unity3d.com/ScriptReference/DisallowMultipleComponent.html) · [issue: RequireComponent não exige múltiplos](https://issuetracker.unity3d.com/issues/requirecomponent-does-not-work-for-adding-multiple-components-of-the-same-type)

**Godot** (MIT)
- [`EditorInspectorPlugin`](https://docs.godotengine.org/en/stable/classes/class_editorinspectorplugin.html) · [Inspector plugins](https://docs.godotengine.org/en/stable/tutorials/plugins/editor/inspector_plugins.html) · [Editor Inspector (DeepWiki)](https://deepwiki.com/godotengine/godot/10.4-editor-inspector)
- ⚠️ [Issue #111807 — cena herdada duplica recursos como override](https://github.com/godotengine/godot/issues/111807)

**Unreal** (proprietária — só documentação)
- [Components in Unreal Engine](https://dev.epicgames.com/documentation/en-us/unreal-engine/components-in-unreal-engine) · [Gameplay Framework](https://dev.epicgames.com/documentation/en-us/unreal-engine/gameplay-framework-in-unreal-engine) · [Details Panel Customization](https://dev.epicgames.com/documentation/unreal-engine/details-panel-customization-in-unreal-engine) · [Level Editor Details Panel](https://dev.epicgames.com/documentation/unreal-engine/level-editor-details-panel-in-unreal-engine)

**OpenUSD** (Apache-2.0 modificada)
- ⭐ [Composition Strength Ordering (LIVRPS) — USD Survival Guide](https://lucascheller.github.io/VFX-UsdSurvivalGuide/pages/core/composition/livrps.html) · [Glossary](https://openusd.org/release/glossary.html) · [What is LIVERPS (NVIDIA)](https://docs.nvidia.com/learn-openusd/latest/creating-composition-arcs/strength-ordering/what-is-liverps.html) · [Payloads](https://docs.nvidia.com/learn-openusd/latest/creating-composition-arcs/references-payloads/what-are-payloads.html)

**Blender** (GPL — ⚠️ **só manual/notas, nenhum código lido**; páginas **não** fetchadas: 403)
- [Library Overrides (manual)](https://docs.blender.org/manual/en/latest/files/linked_libraries/library_overrides.html) · [Core 3.2 release notes — remoção de proxies](https://developer.blender.org/docs/release_notes/3.2/core/) · [#83811 — Resync Improvements](https://projects.blender.org/blender/blender/issues/83811)

**Figma** (proprietária — só documentação)
- ⭐ [Edit instances with component properties](https://help.figma.com/hc/en-us/articles/8883757553943-Edit-instances-with-component-properties) · [Create and use variants](https://help.figma.com/hc/en-us/articles/360056440594-Create-and-use-variants) · [Swap components and instances](https://help.figma.com/hc/en-us/articles/360039150413-Swap-components-and-instances)

**Houdini** (proprietária — só documentação)
- ⭐ [Edit an asset's user interface](https://www.sidefx.com/docs/houdini/assets/asset_ui.html) · [Operator Type Properties](https://www.sidefx.com/docs/houdini/ref/windows/optype.html) · [Edit Parameter Interface](https://www.sidefx.com/docs/houdini/ref/windows/edit_parameter_interface.html)

**Rive** (runtime MIT; editor proprietário)
- [Data Binding Overview](https://rive.app/docs/editor/data-binding/overview) · [Components: Nested Artboards, done right](https://rive.app/blog/components-are-here-nested-artboards-done-right) · [Data Binding supercharged: Lists, Images, Artboards](https://rive.app/blog/data-binding-supercharged-lists-images-and-artboards) · [ViewModel Architecture (DeepWiki)](https://deepwiki.com/rive-app/rive-runtime/7.1-viewmodel-architecture)
