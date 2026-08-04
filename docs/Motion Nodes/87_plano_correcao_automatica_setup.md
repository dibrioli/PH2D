# 87 — Correção automática de setup do grafo (o app conserta quando o artista erra o lugar do nó)

> **Estado:** PLANO (pesquisa + desenho). Nada implementado. Aguarda ordem do Enio para virar wave.
> **Linha:** `line/motion-value` (Modo L). **ADR proposto:** 0155 (PROVISÓRIO — renumera na integração).
> **Pedido do Enio (2026-08-03):** *"se eu tentar colocar algumas forças físicas diretamente na cadeia
> horizontal de nós não irá funcionar. Será necessário usar um Integrate, por exemplo. Mas o artista não
> entende isso. Isso precisa ser automático. Quando o artista colocar um nó no lugar errado o app corrige
> imediatamente o setup."*

---

## §0 — O problema, medido

O grafo de Motion tem uma classe de erro que **não dá erro**. O caso canônico é o que o Enio nomeou:

- Uma `force.*` (attractor/vortex/wind/curl/drag/buoyancy) é um nó **`Effect::Pure`** que **acumula** na coluna transiente `accel` (`crates/ph2d-node-force-attractor/src/accum.rs:28-47` — cria a coluna a zero na primeira força e as forças encadeadas **somam**).
- A coluna `accel` só é **consumida** por DOIS nós, os integradores: `motion.integrate` (`crates/ph2d-node-motion-integrate/src/lib.rs:242-247`, `ColumnAccess::Consume`; Euler semi-implícito em `:343-346`) e `sim.step` (`crates/ph2d-node-sim-step/src/lib.rs:170-172`). Convenção Houdini POP: *microsolvers somam força, UM solver integra* (`accum.rs:26-27`).
- **Sem integrador no caminho, o `accel` é escrito e nunca lido** — o `lower_to_instances_onto` (`crates/ph2d-eval-motion/src/lib.rs:48-63`) só lê `P`/`size`/`rot`/`tint`/`uv_rect`/`texture_id`, **nunca `accel`**. As posições nunca mudam. **A cena fica estática, em silêncio.**
- **Nada detecta isso.** `Graph::validate` (`crates/ph2d-nodegraph/src/graph.rs:559-641`) checa tipo/porta/membrana/param-desconhecido — **zero análise de alcançabilidade**. O cook é pull-based, sem topo-sort; uma entrada não conectada coza para `CookValue::Empty` (`cook.rs:553`), sem falha. O grep por `no.*integrator`/`reachable.*integrate` no substrato veio **vazio**.

O artista arrasta uma força para a cadeia, o fio conecta (as portas batem), o grafo coza, e **nada se move**. Ele não tem como saber por quê.

Este plano faz o app **detectar e consertar** esse setup — e a família inteira de erros irmãos dele — no instante em que o artista coloca o nó.

---

## §1 — Pesquisa do estado da arte

A pergunta tem duas metades: **como detectar** que o grafo está semanticamente inerte, e **o quão agressivo** deve ser o conserto. A segunda é onde todo mundo tropeça.

### 1.1 — Quem resolve por CONSTRUÇÃO (a arquitetura elimina o erro)
- **Houdini POP/DOP.** As forças são *microsolvers* DENTRO de uma rede que **já contém o POP Solver**. Você não consegue ter forças sem o solver — a integração é **estrutural**, mora no contêiner. Não há "força no lugar errado" porque não há lugar errado: o solver é o pai. Preço: rigidez (a rede é uma caixa; a composição livre que temos no grafo horizontal não existe lá).
  - *Aplicável a nós?* É o argumento a favor de um contêiner `sim.zone`/solver-como-pai. **Descartado para v1** — reescreveria a semântica horizontal que já shipou (a família de forças `Pure` + `motion.integrate`, [SKILL §11.13], ADR-0039 congelado). Mantemos a composição livre e **corrigimos**, em vez de proibir.

### 1.2 — Quem CONSERTA numa ação explícita (o modelo a copiar)
- **Blender — Node Wrangler, "Add Texture Setup" (Ctrl+T).** Um Image Texture "quer" um rig canônico (Texture Coordinate + Mapping a montante); o atalho **materializa o rig inteiro** de uma vez. É o precedente mais forte de *"um nó IMPLICA uma tubulação; o app monta a tubulação"*. **É opt-in (um atalho), não silencioso.**
- **Blender/Houdini — splice on-drop + link-drag-search.** Soltar um nó SOBRE um fio o insere na cadeia; arrastar um fio ao vazio abre um menu **filtrado por compatibilidade** que auto-conecta. O app completa a intenção do gesto, não adivinha do nada.
- **Unreal — Blueprint/Material auto-cast.** Arrastar um fio entre pinos incompatíveis **auto-insere um nó de conversão** (int→float, um cast). É exatamente o nosso **Adapter** (`motion_bridge_adapt.rs`, já shipado): a recusa vira cura.
- **Nós NOSSOS já fazem isto:** o **Adapter automático** (arrastar fio entre portas incompatíveis INSERE o conversor — `try_insert_adapter`, `shells/desktop/src/render_loop/motion_bridge_adapt.rs:72`) e o **Splice** (soltar nó sobre fio o insere — `splice_into_wire`, `motion_bridge_rewire.rs:70`). O padrão é sempre: **trial-clone → mutar → `validate` → `push_undo(pre)` → `mark_dirty` → `request_graph_selection` + toast.**

### 1.3 — O que TODOS recusam a fazer, e por quê
- **Ninguém auto-reescreve uma dependência SEMÂNTICA em silêncio no instante em que você larga um nó.** O mais perto que a indústria chega é (a) o Ctrl+T explícito do Blender e (b) o auto-cast num *drop de fio explícito*. O motivo é universal e a nossa própria memória o grita: *mágica que remodela o grafo por trás do artista é desconfiada.* (GSAP embarcou uma ferramenta de debug PORQUE o `shapeIndex` automático erra; Corel faz o usuário clicar um nó em cada forma.) Cerca de Chesterton do projeto: *"botão que não faz nada é pior que botão que falta"* e *"não remodele estado autorado em silêncio"*.
- **A lição operacional:** o conserto pode ser **automático** (sem diálogo) e **imediato** (no gesto), mas **nunca silencioso** (toast + seleção + badge) e **nunca irreversível** (um Ctrl+Z sempre desfaz). Isto é *precisamente* o que o Adapter já faz, e é a única forma de honrar ao mesmo tempo o pedido do Enio ("imediato") e o DNA do projeto ("não adivinhe, não remodele escondido").

### 1.4 — Veredito da pesquisa
O conserto certo é **heal-on-gesture** (curar no gesto), o padrão do Adapter, generalizado do *tipo de porta* (que o Adapter já cobre) para a **inércia semântica** (que ninguém cobre). Duas coisas separam este trabalho de tudo que existe:
1. **A falha não é uma recusa de conexão** — o fio CONECTA e o grafo COZA. Não há hook de "recusa" natural. Precisamos de um **passe de validação semântica** novo, que roda após cada edição.
2. **O conserto tem graus.** Inserir tubulação que o artista obviamente esqueceu (o integrador) é aditivo e seguro → **auto-cura**. Reordenar fios existentes é invasivo → **oferta** (badge de um clique). Adivinhar uma fonte criativa (um campo sem deformer) é impossível → **aviso**.

---

## §2 — Taxonomia COMPLETA dos casos ("todos os casos possíveis")

Construída do mapa de colunas do catálogo (110 crates-nó). Cada caso tem: o **sintoma**, a **detecção** (via colunas), e o **nível de resposta** (AUTO-CURA / OFERTA / AVISO). As colunas transientes que geram acoplamento são **três** — `accel`, `falloff`, `inv_mass` — mais a aparência.

### Família 1 — CONSUMIDOR transiente faltando (o produtor é inerte)

| # | Sintoma | Detecção | Resposta | Conserto |
|---|---|---|---|---|
| **1a** | Força sem integrador em NENHUM caminho para o sink. **← O caso do Enio.** | `Produces("accel")` sem `Consumes("accel")` alcançável a jusante, e nenhum integrador existe no grafo | **AUTO-CURA** | Inserir `motion.integrate` (ou `sim.step` numa cadeia de partículas) entre o fim da cadeia de forças e o sink |
| **1b** | Força a JUSANTE do integrador (o integrador já consumiu `accel` antes de a força escrever) | `Produces("accel")` sem consumidor a jusante, **mas** existe um integrador no grafo (fora do meu caminho para frente) | **AUTO-CURA** se o integrador alimenta a cabeça da cadeia DIRETAMENTE (W2b, reusa-o) · **OFERTA** se há um nó entre eles | Reordenar: religar a força a MONTANTE do integrador |
| **1c** | `motion.pin_constraint` sem solver que leia `inv_mass` | `Produces("inv_mass")` sem `Consumes("inv_mass")` (integrate/sim.step/spring/collide) a jusante | **OFERTA** | Oferecer inserir um solver (ambíguo qual — pin costuma vir com verlet/cloth) |
| **1d** | `field.*`/`motion.falloff` sem leitor de máscara a jusante | `Produces("falloff")` sem leitor (force/deformer/slit_scan/pin) a jusante | **AVISO** | Não auto-inserir (um campo sem deformer pode ser WIP; não se adivinha uma força) |
| **1e** | `motion.look_at`/`motion.path` escreve `rot` mas nada o lê | `Produces("rot")` sem sink no caminho | **AVISO (baixa prio)** | Normalmente o `motion.output` lê `rot`; só alerta se não há sink algum |

### Família 2 — REQUISITO a montante faltando (o nó não tem o que deformar)

| # | Sintoma | Detecção | Resposta |
|---|---|---|---|
| **2a** | Deformer (bend/twist/spherize/four_point_warp/kaleidoscope/spline_wrap) ou força sem `P` a montante | `Requires("P")` e a entrada não traz `P` | **AVISO** ("precisa de pontos (P) a montante" — não se adivinha a fonte) |
| **2b** | `motion.duplicator` faltando uma das DUAS entradas (shape OU points) | manifesto: 2 entradas obrigatórias, uma sem edge | **AVISO** ("precisa de forma E pontos") |
| **2c** | `force.drag`/`force.buoyancy` sem `vel` (só existe após um integrador rodar) | `Requires("vel")` ausente | **AVISO (baixa)** — no-op inofensivo |

### Família 3 — ORDEM dentro do mesmo acoplamento

| # | Sintoma | Detecção | Resposta |
|---|---|---|---|
| **3a** | `field.*` (falloff) a JUSANTE da força/deformer que deveria mascarar | `Produces("falloff")` cujo único leitor está a MONTANTE dele | **OFERTA** (reordenar o campo a montante) — simétrico ao 1b |
| **3b** | Dois integradores no mesmo caminho (dupla integração) | dois `Consumes("accel")` no mesmo caminho para frente | **AVISO/OFERTA** ("dois integradores num caminho") |

### Família 4 — Incompatibilidade de TIPO de porta (**JÁ RESOLVIDA** pelo Adapter)

| # | Sintoma | Detecção | Resposta |
|---|---|---|---|
| **4a** | Arrastar Stream → porta Value | `connects_directly` falso | **JÁ EXISTE** — `try_insert_adapter` insere `value.attribute`. Este plano **unifica o modelo mental** (mesma família "auto-correção") e **reusa a porta**, sem tocar nada aqui. |

### Família 5 — Estrutural / terminal

| # | Sintoma | Detecção | Resposta |
|---|---|---|---|
| **5a** | Subgrafo sem caminho para nenhum sink (`motion.output`) | nó sem alcance a um sink | **AVISO** na cabeça pendurada ("não conectado ao output") |
| **5b** | Escopo de `sim.zone`/`motion.time_remap` | fora de escopo v1 (complexo) | — |

### Família 6 — Partículas

| # | Sintoma | Detecção | Resposta |
|---|---|---|---|
| **6a** | emitter/spawn + força sem integrador | = 1a (o disambiguador escolhe `sim.step`) | **AUTO-CURA** |
| **6b** | emitter sem lifetime/step para avançar `age` | `Generates("age")` sem quem avance | **AVISO** (pode ser intencional) |

**Resumo da política de resposta:**
- **AUTO-CURA** (insere já, toast, seleciona, 1 undo): só quando a peça faltante é **tubulação inequívoca** que o artista esqueceu → **1a** (e 6a). É o pedido do Enio, ao pé da letra.
- **OFERTA** (badge ⚠ + um clique, o artista decide): remexe fios existentes ou tem >1 conserto razoável → **1b, 1c, 3a, 3b**.
- **AVISO** (só conta, sem auto-fix): a peça faltante é uma **escolha criativa** que não se adivinha → **1d, 1e, 2a-c, 5a, 6b**.

---

## §3 — O desenho, com a PORTA ÚNICA de cada pergunta

Quatro perguntas, quatro portas únicas. (Duas portas para a mesma pergunta divergem em silêncio — a doença que o projeto mais persegue.)

### 3.1 — "O que este nó produz / consome / requer?" → **UMA declaração** (novo canal do registry)
Um canal side-metadata novo em `ph2d-node-registry`, **gêmeo exato** do `ParamGate` que acabei de shipar (`register_param_gates`/`param_gates`, `BTreeMap<NodeTypeId, &'static [T]>`, `.get(&id).copied()`). **NÃO é campo do `NodeManifest`** (congelado, §4).

```rust
// crates/ph2d-node-registry/src/ui.rs — irmão de ParamGate
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Coupling {
    /// Escreve uma coluna transiente INERTE sem um consumidor a jusante.
    /// force.* → Produces("accel");  field.* → Produces("falloff");  pin → Produces("inv_mass").
    Produces(&'static str),
    /// Consome uma coluna transiente (a tubulação que faz o produtor viver).
    /// motion.integrate/sim.step → Consumes("accel").
    Consumes(&'static str),
    /// Precisa de uma coluna na entrada para fazer o seu trabalho.
    /// motion.bend → Requires("P").
    Requires(&'static str),
    /// Gera uma coluna do nada (uma fonte). motion.grid → Generates("P").
    Generates(&'static str),
}
```
Registro: `reg.register_coupling(MANIFEST.id, COUPLINGS)` — **uma linha** no `register()` de cada crate-nó produtora/consumidora (~14 nós anotados: 6 forças + integrate + sim.step + spring + collide + pin + os campos + os deformers que declaram `Requires("P")`). Ausência ⇒ nó neutro (nem produz nem consome transiente). **Zero churn** nos outros ~96 nós, zero mudança no contrato.
**Esta é a fonte única de verdade** — o detector, o pintor de badge e os gates leem DELA.

### 3.2 — "Este grafo está semanticamente inerte? onde? qual o conserto?" → **UM passe** (função pura)
Uma função pura nova, em `ph2d-eval-motion` (ou crate leaf nova `ph2d-motion-diagnose` se LOC apertar):

```rust
pub fn diagnose(graph: &Graph, reg: &NodeRegistry) -> Vec<Diagnostic>;

pub struct Diagnostic { pub node: NodeId, pub kind: Deficit, pub fix: Fix }
pub enum Deficit { InertProducer(&'static str), MissingUpstream(&'static str),
                   Misordered(&'static str), Dangling, DoubleConsumer(&'static str) }
pub enum Fix { InsertBefore { sink_edge: Edge, node_type: &'static str },  // 1a
               Reorder     { node: NodeId, before: NodeId },               // 1b/3a
               Offer       { node_type: &'static str },                    // 1c
               None }                                                       // avisos
```
**Algoritmo (alcançabilidade por arestas para frente, ignorando `delayed`/pre):** para cada nó `N` que `Produces(c)` — BFS pelas arestas não-delayed a partir de `N`; se nenhum nó alcançado `Consumes(c)` → inerte. Distinguir 1a de 1b: se existe um `Consumes(c)` no grafo INTEIRO mas não alcançável de `N` → `Reorder`; se não existe nenhum → `InsertBefore`. O disambiguador integrate×sim.step: se o caminho de `N` passa por um `sim.spawn`/`motion.emitter` com lifetime → `sim.step`; senão `motion.integrate`.
**A MESMA função** alimenta (a) a decisão de auto-cura, (b) o pintor de badge, (c) os gates. Uma porta.

### 3.3 — "Qual o nó canônico que consome a coluna `c`?" → **UM mapa**
```rust
fn canonical_consumer(col: &str, particle_ctx: bool) -> Option<&'static str> {
    match (col, particle_ctx) {
        ("accel", true)  => Some("sim.step"),
        ("accel", false) => Some("motion.integrate"),
        _ => None,  // falloff/inv_mass NÃO têm inserção canônica → OFERTA/AVISO
    }
}
```
Só `accel` tem consumidor canônico inequívoco ⇒ só a família 1a/6a AUTO-CURA. Tudo mais vira OFERTA/AVISO por design. Uma porta.

### 3.4 — "Aplicar o conserto ao documento" → **A porta do Adapter** (reusada, não uma segunda)
O conserto é aplicado pelo padrão do `try_insert_adapter`:
`trial = graph.clone()` → `add_node`/`disconnect`/`connect` → `trial.validate(&reg)` → commit atômico → `push_undo(pre)` → **`reconcile(motion, &pre.graph)`** → `mark_dirty` → `request_graph_selection` → toast.

> ⚠️ **ACHADO (2026-08-03, ao ler `motion_bridge_plumbing.rs`): a cura de `accel` NÃO é um splice — é um RESTRUTURA.** A porta `forces` do `motion.integrate` (input 1) é a **de FEEDBACK** que o `reconcile` plumba (`out --pre--> chain_head.in0`), e o input 0 é `rest` (as posições base). Uma força **NÃO pode viver na cadeia horizontal `grid → force → output`** — a montagem correta é:
> - `grid.out → integrate.rest(0)` — as posições base (a força **deixa** de ser alimentada pela grid);
> - `last_force.out → integrate.forces(1)` — a saída da cadeia de forças;
> - `integrate.out(0) → output` — no lugar da aresta antiga;
> - o `reconcile` plumba `integrate.out --pre--> chain_head.in0` (a força lê o estado do tick anterior).
>
> Ou seja, curar **reencaminha a aresta `grid → force` do artista** (a grid passa a alimentar o integrador, não a força). Isso é **mais invasivo que "inserir um nó"** — é a única montagem que de fato move os pontos, mas mexe em arestas que o artista desenhou. **Consequência de produto (decisão do Enio, §9):** talvez o `accel` deva ser **OFERTA** (badge de um clique) em vez de AUTO-CURA silenciosa, já que reroteia o grafo autorado. A alternativa "insira integrate entre a força e o sink" foi **descartada por medição do plumbing**: produziria um grafo que valida mas **não cozinha movimento** (o integrador leria `rest` da própria cadeia móvel em vez das posições base) — exatamente a falha silenciosa que esta feature combate.

O restrutura vai numa porta única nova `motion_bridge_heal.rs::heal_setup` (irmã do `motion_bridge_adapt.rs`), gateada por *newly-inert delta* + *batch construtivo*, com um gate que **COZINHA o grafo curado e verifica que P se move** (não só que um nó apareceu).

### 3.5 — "QUANDO o conserto dispara?" (a decisão de produto)
**Gatilho = o gesto que COLOCA o nó** (mirror do Adapter, que dispara no drop de fio):
- **AUTO-CURA** dispara dentro de `apply_graph_intents` **após** um gesto CONSTRUTIVO (`AddNode`/`Connect`/`SpliceNode`/`SmartConnect`/`Paste`), **no mesmo slot pós-drain/pré-cook** onde o `publish` do source.shape roda hoje (`motion_bridge.rs:356`). Roda `diagnose`; para cada produtor inerte NOVO introduzido por ESTA edição, insere a tubulação.
- Gestos **DESTRUTIVOS** (`Delete`/`Disconnect`) **NÃO** disparam auto-cura — só atualizam badges. Assim o artista pode **apagar o integrador para religar à mão** sem o app brigar com ele (re-inserindo). É a diferença exata que impede a mágica de virar armadilha.
- **OFERTA/AVISO** aparecem como **badge** no instante em que o problema existe (qualquer edição), sem modal, sem bloqueio.
- **Tudo é 1 Ctrl+Z.** O undo É a escape hatch (não há toggle "auto-fix on/off" — um toggle seria um botão que promete gerência que o undo já dá).

> **A ÚNICA decisão de produto genuína** é a agressividade da auto-cura. **Recomendação (default):** auto-cura só o caso **1a** (força↔integrador, inequívoco); todo o resto é OFERTA/AVISO. É o mínimo que resolve o pedido do Enio sem nunca adivinhar. Se o Enio quiser 1c (pin→solver) também auto-curando, é uma linha no `canonical_consumer` — mas aí escolhemos um solver por ele, o que a pesquisa desaconselha.

---

## §4 — Contrato congelado (§6) & schema — prova por grep

- **Contrato de nós congelado** (`NodeOp=2`/`OpResolver=1`/`NodeManifest=8`, ADR-0039): **INTACTO.** O gate `crates/ph2d-nodegraph/tests/architecture_contract_surface.rs` conta declarações **textualmente**, por `include_str!`, **só** em `ph2d-nodegraph/src/node.rs` e `.../cook.rs`. O canal `Coupling` novo vive em `ph2d-node-registry/src/{ui.rs,lib.rs}` — um `BTreeMap` a mais no `NodeRegistry`, um value-type novo, um par `register_coupling`/`coupling`. **Nenhum desses arquivos é `node.rs` nem `cook.rs`** ⇒ os contadores 2/1/8 não se movem, por construção (é exatamente como `param_gates`/`reduces`/`luts` entraram). Prova: `grep -n "pub struct NodeManifest" crates/ph2d-nodegraph/src/node.rs` (8 campos, inalterado); o novo canal não toca esse arquivo.
- **`PROJECT_SCHEMA`/`VEC_SCENE`/`DOC_VERSION`:** **INTACTOS.** A auto-cura produz nós e arestas NORMAIS, que o `ProjectState` já serializa; os diagnósticos/badges são **view-state transiente** (não salvos). Nenhuma estrutura nova viaja no arquivo. Grep: o `Graph` já carrega `NodeInstance`/`Edge` no save; nada novo entra.
- **ADR:** proposto **0155 (PROVISÓRIO)** — introduz um conceito arquitetural (validação semântica + heal-on-gesture + canal de acoplamento). Renumera na integração (a memória do repo é explícita: número de ADR escolhido em linha paralela é provisório).

---

## §5 — O que a UI precisa (as 4 condições independentes)

1. **O componente EXISTE:**
   - **Badge ⚠ no nó** (OFERTA/AVISO): pintado ao lado do `draw_pre_badges`/`paint_socket_glyph` em `crates/ph2d-panel-motion-graph/src/paint_wire.rs`/`paint.rs`. Cor = token DANGER (já existe, usado no ghost-wire ilegal). Não há badge de "inválido" hoje — é território novo, mas o slot de pintura existe.
   - **Toast** (AUTO-CURA): `ToastQueue` já usado pelo Adapter (`motion_bridge_connect.rs:103`) — "Inserted motion.integrate (undo to remove)".
   - **Quick-fix** (OFERTA): o badge é o alvo de clique; um clique aplica o `Fix`.
2. **Pintado e registrado:** o badge entra no passe de paint do nó; o hit do badge usa o sistema de hit existente (como `pre_badge_centers` em `hits.rs:47`).
3. **O clique chega ao barramento:** o quick-fix emite `GraphIntent::ApplyFix { node, fix }` (variante nova no `snapshot_intent.rs`), drenada em `apply_graph_intents`. A AUTO-CURA não precisa de clique (roda na bridge). O verbo passa por `apply_key` panel-side se houver atalho (ex.: uma tecla "conserta o nó selecionado").
4. **A SEQUÊNCIA leva a algum lugar:** aplicar o `Fix` insere/reordena, `push_undo`, `mark_dirty`; o `diagnose` do próximo frame vê o grafo curado e o badge some. Fecha o laço.

---

## §6 — Os gates (red-first) + a fixture que contém o fenômeno

O detector é uma função pura ⇒ a maioria dos gates é headless e barata. **Fixture-chave:** um grafo `motion.grid → force.wind → motion.output` **sem integrador** (contém O fenômeno — a fixture do force→output foi o que faltava no repo inteiro).

1. `a_force_with_no_integrator_is_diagnosed_inert` — `diagnose` do grafo acima → **um** `Diagnostic{ InertProducer("accel"), fix: InsertBefore(motion.integrate) }`. **Mutação:** detector que ignora `accel` → nenhum diagnóstico → RED.
2. `a_healthy_chain_is_diagnosed_clean` — o mesmo grafo COM `motion.integrate` → **zero** diagnósticos (controle: o gate não é vacuamente verde).
3. `constructive_gesture_auto_heals_the_force_chain` (shell) — conectar `force.wind → motion.output` → a bridge **auto-insere** `motion.integrate` entre eles; o grafo passa a cozar com movimento. **Mutação:** remover o ramo de auto-cura → força segue inerte → RED.
4. `a_destructive_gesture_does_not_reinsert` — apagar o integrador (Delete) → **NÃO** re-insere (badge só). **Mutação:** auto-cura em toda edição → re-insere → RED. (É a lei que impede a mágica de brigar com o artista.)
5. `a_force_downstream_of_the_integrator_is_a_reorder_not_a_second_insert` — força DEPOIS do integrate → `diagnose` sugere `Reorder`, e a auto-cura **não** insere um 2º integrador. **Mutação:** tratar como `Insert` → dois integradores → RED. (A lei "UM integrador aplica".)
6. `every_producer_declares_its_coupling` — para cada `force.*`/`field.*`/`pin`, o registry tem `Coupling`. **Mutação:** tirar o coupling de `force.wind` → produtor indiagnosticável → RED. (Impede uma força nova nascer sem diagnóstico.)
7. `the_coupling_channel_leaves_the_frozen_contract_intact` — reafirma os contadores 2/1/8 (o gate `architecture_contract_surface` já cobre; este assere que registrar coupling não os move) + `coupling_round_trip` (None antes, slice depois, None para id desconhecido — gêmeo do `param_gates_round_trip`).
8. `an_ambiguous_case_offers_instead_of_guessing` — pin sem solver → `Fix::Offer`, **não** auto-aplicado. **Mutação:** auto-aplicar Offer → RED.
9. `the_badge_is_painted_and_the_quick_fix_reaches_the_bus` (seam, panel) — o badge do nó inerte é pintado E clicá-lo emite `GraphIntent::ApplyFix`. **Mutação:** tirar o badge do `populate`/paint → clique morto → RED.
10. `auto_heal_is_one_undo_step` — após auto-cura, um Ctrl+Z restaura o grafo pré-edição (força + o integrador inserido somem juntos, OU só o integrador — decidir: a auto-cura é 1 undo SEPARADO do add da força, para o artista poder desfazer só o conserto? **Recomendação:** a auto-cura entra no MESMO bracket do gesto que a disparou — desfazer o gesto desfaz o conserto junto, sem passo órfão). **Mutação:** dois `push_undo` → dois passos → RED.

Rodar **debug E release** (precedente do repo: pânico só-em-debug já mordeu esta linha).

---

## §7 — A cena de smoke (números MEDIDOS)

> ⚠️ **Rodar a sonda headless ANTES de escrever a mensagem** (pd-feature): medir quantos pontos se movem e a deriva px/frame COM vs SEM o integrador, e pôr os números reais na `eprintln`.

`PH2D_AUTOFIX_SMOKE=1`:
- **Frame 3:** monta `motion.grid(4×4) → force.wind → motion.output` (**sem** integrador). No instante em que `force.wind` conecta ao output, o app **auto-insere `motion.integrate`** entre eles. Os 16 pontos, antes congelados, passam a **derivar** com o vento. `eprintln`: *"force.wind ligado ao output sem integrador → o app auto-inseriu motion.integrate; os N pontos agora andam ~X px/frame (mediam 0 antes); um Ctrl+Z remove o conserto."*
- **Frame 90:** o artista **apaga** o `motion.integrate` (gesto destrutivo). Os pontos **congelam** e um badge ⚠ aparece no `force.wind` oferecendo "Insert Integrator" — **sem** re-inserção silenciosa. `eprintln`: *"apaguei o integrador — os pontos param e o badge oferece o conserto, mas o app NÃO re-insere sozinho (gesto destrutivo). Um clique no badge, ou refazer, restaura o movimento."*
- **Frame 150 (opcional):** monta `field.box → force.wind` a jusante (caso 3a) → badge de OFERTA "reorder", demonstrando o nível OFERTA vs AUTO-CURA.

Os números `N`/`X` saem da sonda `diagnose` + um cook real medindo `P` antes/depois.

---

## §8 — Sequência de implementação (waves)

- **W1 — O canal + o detector (headless, o coração). ✅ LANDOU (2026-08-03, `line/motion-value`).** `Coupling` no registry (`register_couplings`/`couplings` + `couplings_round_trip`, gêmeo do `ParamGate`). Anotados os acoplamentos **`accel` e `inv_mass`** (6 forças `Produces("accel")`; integrate/sim.step `Consumes("accel")`+`Consumes("inv_mass")`; spring/collide `Consumes("inv_mass")`; pin `Produces("inv_mass")`). Crate leaf nova **`ph2d-motion-diagnose`** com `diagnose()` pura (`InertProducer` → `Insert`/`Reorder`/`Offer`) + `canonical_consumer` (porta única). **6 gates** sobre o registry REAL (`register_all_nodes`): força-inerte→Insert · cadeia-saudável limpa · força-a-jusante→Reorder · partícula→sim.step · pin→Offer · every-producer-declares. 3 mutações-chave sangram (canonical_consumer=None · reorder-morto · reachability-curto). Contrato congelado intacto (workspace verde), zero schema, clippy limpo, debug+release. ⚠️ **`falloff` DEFERIDO** — anotar fields sem seus consumidores (força/deformer) geraria falso-positivo; a família viaja junta numa sub-wave.
- **W2 — A auto-cura (o gesto). ✅ LANDOU (2026-08-03, `line/motion-value`).** `motion_bridge_heal.rs::heal_setup` (irmã do `motion_bridge_adapt.rs`), chamada no FIM de `apply_graph_intents` **só após um batch CONSTRUTIVO sem remoção** (`is_constructive && !is_destructive`) — apagar o integrador para religar nunca é combatido. Cura toda cadeia de forças **inerte que ALCANÇA um `motion.output`** (mid-build sem sink fica quieta), **restruturando** (não splice): `grid → integrate.rest`, `last_force → integrate.forces`, `integrate → consumer`, com o `reconcile` plumando o `pre`; dedupe por cabeça de cadeia (**um** integrador por cadeia). **Um passo de undo próprio** (o artista desfaz só o conserto). **6 gates dirigindo o funil REAL** (`apply_graph_intents`): a estrela **COZINHA o grafo curado e prova que os pontos SE MOVEM** (não só que um nó apareceu) · undo remove o integrador · construtivo cura pré-existente × batch-com-remoção não · duas forças = um integrador · cadeia sem output não é curada. **4 mutações sangram** (drop `!is_destructive` no batch misto · `reaches_output` sempre-true na cadeia-sem-output · trocar os ports rest↔forces é pego PELA COZEDURA · drop do dedupe = 2 integradores). Toast + seleção do integrador. clippy limpo, debug+release, suíte `motion_bridge` (142) intacta. Smoke: **`PH2D_AUTOFIX_SMOKE=1`** (a força liga direto ao output → o app auto-insere integrate e os pontos derivam; frame 90 apaga o integrador → congelam, sem re-inserir). ⚠️ **O smoke JULGA se a restrutura (reencaminhar `grid → force`) é agressiva demais** — se for, o fallback é OFERTA (W3).
- **W2b — A auto-cura do REORDER (1b direto). ✅ LANDOU (2026-08-03, `line/motion-value`).** Estende `heal_setup` ao caso **1b** quando a força está spliceada **DIRETAMENTE a jusante** de um integrador (`grid → integrate → force → output`): o app **REUSA** esse integrador em vez de inserir um segundo — a força vira o ramo de `forces`, `integrate → consumer`, e o `reconcile` re-plumba o `pre`. ⚠️ **Decisão de produto (reversão parcial do §9/§185):** o plano recomendava Reorder como OFERTA por ser "invasivo", mas o Enio **aprovou a restrutura do W2** (que já reencaminha `grid → force`, igualmente invasiva) — então o 1b direto, sendo *a mesma "força no lugar errado" com cura canônica* (há UM integrador a reusar, não um a escolher), auto-cura pela mesma porta. `HealPlan` ganhou `HealKind::{Insert, Reuse}`; `plan_heal` decide pela pergunta *"quem alimenta a cabeça da cadeia?"* — um integrador (`consumes_accel(source.0)`) ⇒ `Reuse`; um `grid` ⇒ `Insert`. ⚠️ **O 1b com um NÓ NÃO-INTEGRADOR entre o integrador e a força** (`grid → integrate → transform → force`) **fica de fora** (reusar duplicaria a integração — `rest` seria uma base já transformada e móvel) — é badge do W3. **2 gates novos dirigindo o funil REAL** (a estrela **COZINHA** e prova que os pontos se movem + reuse é 1 passo de undo). **1 mutação-chave sangra** (Reorder→Insert ⇒ dois integradores, `2 ≠ 1`). LOC: `motion_bridge_heal.rs` cruzou 600 ⇒ tests → irmão `motion_bridge_heal_tests.rs` (`#[path]`, FILHO, `use super::*` alcança os privados); e um vermelho-latente do W2 fechou junto — `motion_bridge.rs` estava em **606** (o `mod heal;` o empurrou), split por RESPONSABILIDADE em `motion_bridge_remove.rs` (os handlers destrutivos `apply_disconnect`/`apply_delete_selection`/`output_nodes`, o par de `motion_bridge_connect.rs`; re-export privado ⇒ zero churn de chamador) — e o **doc-comment órfão** do `apply_connect` (que migrou para `connect.rs`) foi removido. clippy limpo, debug+release, `file_loc_caps` verde. Smoke: **`PH2D_AUTOFIX_SMOKE=2`** (sim que já roda; solta-se uma `force.wind` SOBRE o fio de saída; o app reusa o integrador e os pontos derivam para +X — confira que existe **UM só** `motion.integrate`).
- **W3 — Badge + quick-fix. ✅ LANDOU (2026-08-04, `line/motion-value`).** Todo nó que o `diagnose` marca inerte E que ALCANÇA a saída ganha um **pip ⚠** no canto do card (`GraphNodeView.inert`, preenchido pelo shell em `motion_bridge_readout::stamp` a partir de `inert_reaching_output` — exatamente como o `is_sink`; mid-build sem sink NÃO ganha badge, o mesmo filtro `reaches_output` da auto-cura). **Clicá-lo aplica a cura canônica onde ela existe, ou EXPLICA + seleciona onde não existe** — ⚠️ **refino do §8/§185:** o plano dizia "Reorder = OFERTA" (um badge que só oferece); o clique agora é **fix-or-explain**, honrando a lei ADR-0155 de *nunca adivinhar uma escolha criativa*. **Dispara mesmo após um gesto DESTRUTIVO** (ao contrário do `heal_setup` do W2): o clique É o pedido, então re-inserir um integrador que o artista acabou de apagar honra-o em vez de brigar. O intent é **`GraphIntent::FixInert`** (não o `ApplyFix` do plano — o painel só tem o snapshot, então encaminha o id; o shell decide fix-vs-explain em `motion_bridge_heal::heal_one`). ⚠️ **A porta ÚNICA `plan_heal` responde "este fix tem cura canônica?"** — `Some` só para `Fix::Insert` e o Reorder-reuse, `None` para `Offer`/Reorder-indireto; **o `matches!(d.fix, ...)` que `heal_one`/`heal_setup` traziam era uma camada de política REDUNDANTE** (nenhuma mutação single-point a flipava — `plan_heal` sempre pegava o que ela pegaria) e foi REMOVIDA — foi isso que tornou o gate advisory honesto ([[feedback_layered_defenses_need_per_layer_gates]]). Foundational **aditivo**: `GraphHitKind::InertBadge` (variant apendado; enum de interação NÃO-congelado) · `GraphIntent::FixInert` · `GraphNodeView.inert`. **4 gates + 5 mutações single-point RED:** 2 no painel (o hit só existe SOBRE nó inerte — `inert_badge_rect` incondicional ⇒ RED; o clique empurra `FixInert` — arm removido ⇒ RED) · 3 no shell (o badge-set são os setups COMPLETOS, não os mid-builds — filtro `reaches_output` removido ⇒ RED; clicar um badge fixável CURA e os pontos se movem — `plan_heal` recusa Insert ⇒ RED; clicar um badge advisory NÃO muda nada — `plan_heal` planeja Offer ⇒ RED). **LOC — cinco splits** (o painel estava com paint.rs/interact.rs/snapshot.rs colados no cap 600, o miss estrutural de gates em `tests/` que o `cargo check -p` não alcança): `paint_inert_badge.rs` (o badge draw, chamado por `draw_card`) + `paint_grid.rs` (a grade de fundo, abriu espaço) · `interact_zoom.rs` (anchored zoom) · `snapshot_build.rs` (o builder `snapshot_from`, fronteira dados × builder) · `interact_select_tests.rs` (família de seleção, FILHO de `interact_tests`). clippy limpo, debug+release, contrato congelado + panel/workspace/shell LOC + no_tofu verdes; `PROJECT_SCHEMA`/`VEC_SCENE`/`DOC_VERSION` INTOCADOS (badge é view-state transiente). Smoke: **`PH2D_AUTOFIX_SMOKE=3`** (dois setups inertes SEM gesto — nada auto-corrige; a `force.wind` ganha badge fixável, o `pin` ganha badge advisory; clicar o da força AUTO-INSERE integrate e os pontos derivam, clicar o do pin só EXPLICA). **Restam para waves futuras (badges advisory já pintados, o clique explica):** **1c** (pin→solver), **3a** (falloff a montante), **3b** (dupla integração), **1b indireto** (um nó não-integrador entre o integrador e a força).
- **W4 — A família `falloff` (por DERIVAÇÃO) + o toggle "Node Help". ✅ LANDOU (2026-08-04, `line/motion-value`).** ⚠️ **O modelo de ANOTAÇÃO que o W1 deferiu foi TROCADO pela DERIVAÇÃO** (ordem do Enio: *"busque o padrão ouro, sem pensar em custos"*). O `falloff` tem **~40 consumidores** (toda força/deformer/transform) — anotar 48 crates à mão é frágil (um novo consumidor esquecido = falso-positivo). O padrão-ouro que a pesquisa nomeia (Blender geometry nodes / Houdini) **deriva** a dependência de campo da avaliação, nunca de uma tabela paralela — e nós já temos a declaração certa: as **`ColumnBinding` de GPU** que cada nó GPU-resident declara para o cook (`column.rs`) INCLUEM `falloff`. O `diagnose` passou a ler `reg.gpu_kernel(id).bindings` (a MESMA verdade que o cook usa, drift-proof) unida ao `Coupling` só para os CPU-only. **A regra é agnóstica de coluna:** *escreve* ⇒ **Produces**; *lê-mas-não-escreve* ⇒ **Consumes**; *lê-e-escreve* (uma força acumulando `accel`, um `field.combine` compondo `falloff`) é RE-produtor, **NÃO** consumidor — a joia que mantém *duas forças sem integrador* inertes (a 2ª lê `accel` mas re-escreve, não "salva" a 1ª). `accel`/`falloff`/`inv_mass` caem todos na MESMA regra. Cobertura da família: **29 dos 35 nós de graça** (GPU-resident) + **6 CPU-only** anotados com `Consumes("falloff")` (`fx.drop_shadow`/`fx.rgb_split`/`motion.delay`/`slit_scan`/`spline_wrap`/`step`). O único **declarado** é o conjunto minúsculo `TRANSIENT_COLUMNS` (as não-lowered; `accel`/`inv_mass` derivados das bindings `Consume` + `falloff` nomeado) — é ele que impede o falso-positivo de `P`/`size`/`rot` (que o output consome). Um `field.box` ligado a nada vira **`Fix::Offer`** (nada canônico a inserir) → badge que **explica + seleciona** (*"precisa de uma força/deformer"*), nunca adivinha. **O TOGGLE "Node Help"** (Enio, **sobrescreve** o "sem toggle" original): chip `IconId::Help` na barra (`CHROME_NODE_HELP`), **UM flag** (`MotionState::node_help_enabled`, ON default, sessão-only) gateando os **três** pontos (`inert_reaching_output`/`heal_setup`/`heal_one`) — a liberdade do artista e o **release-valve** da família `falloff` (um miss vira um clique). Publicado por thread-local (`set_node_help`, gêmeo do `set_graph_selection`), NUNCA no `GraphViewSnapshot`. **Gates: 4 no diagnose** (campo→nada = falloff Offer *derivado, sem anotação* · campo lido por força = saudável · **duas forças = ambas inertes** [a regra do re-produtor] · CPU-only anotado = saudável · `P`→output nunca inerte · `TRANSIENT` cobre todo Consume) — **4 mutações RED** (re-produtor conta como consumidor · só-Coupling sem GPU · `P` em TRANSIENT · tirar `accel` do conjunto). **3 no shell heal** (help off ⇒ sem badge / sem auto-cura / clique no-op — um guard por camada, **3 mutações RED**). **2 no painel** (o chip é oferecido em `chip_specs` e veste o estado vivo · o clique emite `SetNodeHelp(!node_help())` — **2 mutações RED**). Contrato congelado INTACTO (`NodeOp=2`/`OpResolver=1`/`NodeManifest=8`, gate verde), zero schema, no_tofu/LOC/workspace verdes. Smoke: **`PH2D_AUTOFIX_SMOKE=4`** (`grid → field.box → output`: 1 campo de falloff inerte — o field.box NÃO tem Coupling, o aviso vem PURO da binding de GPU; clicar explica; o chip 'Node Help' liga/desliga o sistema inteiro).
- **W4b — Requisito-a-montante (2a). ✅ LANDOU (2026-08-04, `line/motion-value`).** Um deformer/força que **lê** `P` mas tem **nada** ligado na entrada → `Deficit::MissingSource("P")`, `Fix::Offer` → badge advisory (explica + seleciona; WHICH source é escolha criativa). **Pela MESMA derivação da W4:** fonte de P = binding `Write` puro (não `reads()`), leitor de P = qualquer `reads()` — a regra do re-produtor no eixo de montante, **zero anotação**. ⚠️ **Regra "SEM aresta de entrada", não reachability** (medido: `sim.spawn` gathera de um template, então reachability quebraria o gate de partículas; "sem aresta" o evita por construção) — zero falso-positivo, sub-avisa na direção segura, o toggle respalda. `REQUIRED_UPSTREAM=["P"]` disjunto do transiente (gate). **4 gates comportamentais + 1 de unidade + 1 de shell, 3 mutações RED** (`has_input`-sempre-true · drop do `continue` · `Write`-conta-como-leitura). Contrato congelado intacto, zero schema, clippy limpo, workspace verde. Smoke: **`PH2D_AUTOFIX_SMOKE=5`** (`motion.bend -> output` sem grid; o badge explica; ligar um grid cura). ⛔ **2b (`duplicator` sem uma das 2 entradas) FICA FORA:** é requisito de PORTA (não de coluna) e o `PortSpec` não distingue obrigatória de opcional ⇒ "≥2 inputs, um solto" dispararia em qualquer 2º input opcional (a fragilidade de enumeração que a derivação removeu). Pede flag `required` no manifesto OU caso-especial por-nó — **decisão do Enio, não escopo de wave de derivação**.
- **W4c — Smoke + polimento (restante).** Os avisos AVISO remanescentes (5a dangling; 2c `vel`; 1e `rot`; 6b lifetime) — cada um só com pedido/decisão de UX (proliferação de badge advisory).
- **W5 (só com pedido)** — casos avançados (sim.zone/time_remap escopo, 5b).

Cada wave é fechável e smokável. **W1 é o padrão-ouro isolado:** um detector que o Enio pode ver funcionar (via gate/sonda) antes de qualquer pixel de UI.

---

## §9 — Riscos & decisões em aberto

- **Agressividade (§3.5):** auto-cura só 1a (recomendado) vs. incluir 1c. **Decisão do Enio.**
- **O bracket de undo (§6 gate 10):** conserto no mesmo passo do gesto (recomendado) vs. passo separado. **Recomendado: mesmo passo.**
- **`falloff`/`inv_mass` como AVISO, não auto-cura:** deliberado (não se adivinha uma força/solver). Se o uso mostrar que o artista quer, vira OFERTA com um clique — nunca auto-inserção silenciosa.
- **Reorder (1b/3a):** o plano recomendava sempre OFERTA por mexer em fios existentes — **revisto no W2b** (decisão do Enio): o **1b DIRETO** (força spliceada logo a jusante de um integrador) auto-cura *reusando* aquele integrador, porque é a mesma "força no lugar errado" que o W2 já reencaminha e há UM integrador inequívoco a reusar. O **1b indireto** (um nó entre o integrador e a força) e o **3a** seguem OFERTA (reusar duplicaria a integração / o campo tem outro dono).
- **Custo:** `diagnose` é O(nós+arestas) por edição (não por frame — roda no gesto). Insignificante nos counts de hoje; medir na W1 mesmo assim.
