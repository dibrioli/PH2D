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
| **1b** | Força a JUSANTE do integrador (o integrador já consumiu `accel` antes de a força escrever) | `Produces("accel")` sem consumidor a jusante, **mas** existe um integrador no grafo (fora do meu caminho para frente) | **OFERTA** | Reordenar: religar a força a MONTANTE do integrador |
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
O conserto é aplicado exatamente pelo padrão do `try_insert_adapter` / `splice_into_wire`:
`trial = graph.clone()` → `add_node`/`disconnect`/`connect` → `trial.validate(&reg)` → commit atômico → `push_undo(pre)` → `mark_dirty` → `request_graph_selection([novo])` → toast. **Reusa a porta existente** (`motion_bridge_rewire.rs::splice_into_wire` para o insert-antes-do-sink), não escreve uma segunda.

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
- **W2 — A auto-cura (o gesto).** Ramo em `apply_graph_intents` (slot pós-drain/pré-cook), `canonical_consumer`, reuso do `splice_into_wire`. Gates 3,4,10. O toast.
- **W3 — Badge + quick-fix (OFERTA/AVISO).** Badge painter + `GraphIntent::ApplyFix` + seam gate 9. Reorder (1b/3a).
- **W4 — Smoke + polimento.** A cena, os números medidos, os avisos (5a dangling, 2a/2b requisitos).
- **W5 (só com pedido)** — casos avançados (sim.zone/time_remap escopo, 5b).

Cada wave é fechável e smokável. **W1 é o padrão-ouro isolado:** um detector que o Enio pode ver funcionar (via gate/sonda) antes de qualquer pixel de UI.

---

## §9 — Riscos & decisões em aberto

- **Agressividade (§3.5):** auto-cura só 1a (recomendado) vs. incluir 1c. **Decisão do Enio.**
- **O bracket de undo (§6 gate 10):** conserto no mesmo passo do gesto (recomendado) vs. passo separado. **Recomendado: mesmo passo.**
- **`falloff`/`inv_mass` como AVISO, não auto-cura:** deliberado (não se adivinha uma força/solver). Se o uso mostrar que o artista quer, vira OFERTA com um clique — nunca auto-inserção silenciosa.
- **Reorder (1b/3a)** mexe em fios existentes ⇒ sempre OFERTA, nunca auto. (Insert é aditivo e seguro; reorder é destrutivo.)
- **Custo:** `diagnose` é O(nós+arestas) por edição (não por frame — roda no gesto). Insignificante nos counts de hoje; medir na W1 mesmo assim.
