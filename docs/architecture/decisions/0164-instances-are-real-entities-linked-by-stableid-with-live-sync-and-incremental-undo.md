# ADR-0164 — Instância = objetos REAIS ligados por id ao mestre, sync vivo no mesmo mundo, e o undo vira INCREMENTAL

- **Status:** Accepted (aprovado pelo Enio em 2026-08-24 ao ordenar a implementação)
- **Data:** 2026-08-24
- **Linha:** `line/components` (a abrir — plano vivo em [`docs/Components/05_plano_de_implementacao.md`](../../Components/05_plano_de_implementacao.md))
- **Toca:** `ph2d-ecs` (foundational: `StableId` · `SiblingOrder` · `WorldSnapshot` v2 · registro) · crate nova `ph2d-component-desc` · `shells/desktop` (undo · sync · verbos · Add Component) · `ph2d-physics-ecs` (filtro `Without<MasterPiece>` + porta `pose_owner`) · `ph2d-timeline` (`WireId` → `StableId`) · o `VecInstance` do vetor (subsumido)
- **Não move:** os três contratos congelados do CLAUDE.md §6 (`NodeOp`/`OpResolver`/`NodeManifest` · `Tool`/`RasterEditTool` · superfície do `ph2d-vector-doc`)
- **Encadeamento:** este ADR governa as fases **F0–F5**; o [ADR-0165](0165-assets-are-born-inside-the-app-three-level-identity-index-before-browser.md) depende deste e governa **F6–F7**.

## Contexto

A auditoria de 2026-08-21 ([doc 01](../../Components/01_auditoria_modelo_de_objeto.md)) mediu cinco
fatos: não há como criar objeto vazio na raiz (o botão `HIERARCHY_ADD` está **morto**); não há UI de
Add Component (falta um `insert_default` na vtable do registro); não existe id estável de objeto (o
`stable_name_id` **quebra ao renomear**, e o doc-comment do `name.rs:73-79` já prescrevia a cura);
o reuso vinculado só existe para desenho vetorial (`VecInstance` = **uma** entidade + geometria
derivada — sem física, sem filhos, e **aninhamento nem renderiza**, `instance_live.rs:149-152`); e a
captura do undo custa **6,3–23,8 ms** a 10 k entidades por quadro-com-input (o 23,8 é o regime real
de edição — o `canonicalize` constrói a chave dentro do comparador do sort).

Uma primeira recomendação (instância *derivada*, um objeto só) foi **vetada pelo Enio**: *"tudo deve
ser possível… encontre um modo da cópia ser física… cópia dentro de cópia… o problema de velocidade
deve ser resolvido"*. A reconsideração ([doc 04 v2](../../Components/04_decisao_arquitetura.md)) fez
pesquisa nova (Unity/Godot/flecs/Bevy **em código**, Houdini/USD/Rive/Blender por doc), **mediu** a
alternativa, e submeteu as três afirmações-chave a refutadores adversariais — **os três refutaram o
enunciado ingénuo e devolveram as condições** que esta decisão incorpora
([pesquisa/instancias_2026-08-21/](../../Components/pesquisa/instancias_2026-08-21/)).

⚠️ O que a pesquisa cravou: **nenhum ECS permissivo faz o que se pede** (flecs *proíbe* re-sync
estrutural; Bevy propaga por despawn+respawn; o único precedente funcional é o editor do Godot — que
**apaga o histórico de undo** ao propagar). O modelo abaixo é a família Unity (materializado + diff
por campo + chave por id) com uma propagação mais fina do que a de qualquer referência.

## Decisão

**Quatro peças, interdependentes de propósito** (os refutadores mostraram que nenhuma fica de pé sem
as outras). A forma longa, com toda a evidência, é o [doc 04 v2 §2](../../Components/04_decisao_arquitetura.md).

### 1. `StableId(u64)` é A identidade de objeto — e a migração é inteira, nunca metade

Componente registrado em **toda** entidade editável, alocado por hook `on_add` de `Transform` a
partir de **contador monotônico persistido fora do `ProjectState`** (um undo não pode rebobiná-lo).
Único por documento (gate + prova de mutação); **remapeado em toda cópia de blobs**; `restore()`
passa a despawnar `With<StableId>`. As **duas** famílias de `stable_name_id` (timeline `WireId` ·
física `PhysicsJoint.body_a/b`/`PulleyWheel`) migram **na mesma wave** — o `name.rs` proíbe metade.
Junto entram `SiblingOrder(u32)` (a ordem de irmãos vira DADO — hoje reordenar não é desfazível) e o
`WorldSnapshot` **v2**: linhas em ordem de `StableId`, `parent: Option<StableId>` — o que **apaga o
`canonicalize`** (0,088 ms contra 18,7 ms, medido) e exige o bump do `PROJECT_SCHEMA` com **a
primeira migração da história do repo** (corpus de fixtures + round-trip byte-equivalente).

### 2. O undo captura só o que MUDOU — protocolo de seis condições, medido a 0,27 ms @10 k

Cache `BTreeMap<StableId, Row>` com linhas `Arc`; sujo = tick de componente registrado **ou
`ChildOf`** mais novo que o tick da **última captura**, **ou** `ArchetypeId` diferente do cacheado
(⚠️ **remover componente não carimba tick de ninguém** — medido); **tick é pré-filtro, bytes são a
verdade** (o `DerefMut` carimba sem escrever — medido); spawn por ausência, despawn por carimbo;
⚠️ **`clear_trackers()` exatamente uma vez por CAPTURA, nunca nos quadros de retorno precoce** (um
arrasto de 10 quadros perderia 9). O PH2D **nunca avançou o change tick** (zero chamadas no repo) —
este protocolo é quem o liga. Pilha de undo vira **deltas** (~12,5 MB contra ~614 MB a 10 k / 1 %).

### 3. Instância = subárvore REAL, ligada peça a peça, com override por CAMPO e sync por quadro

- **Duplicar** = cópia profunda, ids novos, refs remapeadas, sem vínculo. **Instanciar** = peças
  reais com `InstanceOf { master_root, master_piece }`; física, filhos, scripts funcionam porque
  **são entidades comuns** (a ponte já as vê por query — zero caso especial, lido).
- **Override** esparso na raiz da instância: chave `(path[], peça, type_id, field_id)` — ids no
  **escopo do mestre** (a lei do `VecInstance.sub`), ⚠️ `field_id` **declarado e append-only** no
  descritor, nunca posicional (postcard posicional re-alvejaria campos em silêncio ao mudar a forma).
- **Captura por DIFF, nunca por porta** (todo escritor existente escreve a entidade; *um guard que
  enumera os seus consumidores apodrece*), com **política por (tipo, campo)** no descritor:
  *Propaga* (default) · *Local da instância* (Transform/Name da raiz — os "default overrides" do
  Unity) · *Do runtime* (pose de peça cujo `pose_owner ∈ {Solver, Player}` — a ponte **exporta a
  porta**, ninguém re-deriva).
- **Sync mestre→instâncias por quadro**, dirigido pelos mesmos ticks, escrito com `set_if_neq`, em
  **ordem topológica** com **recusa de ciclo no gesto**; refs internas (`body_a`, `.path`, `.host`,
  bindings) **remapeadas em toda propagação**. ⚠️ **O mestre não simula**: vive na biblioteca
  (*Criar componente* deixa uma instância no lugar), peças marcadas `MasterPiece`, e as **cinco**
  `QueryState`s da ponte ganham `Without<MasterPiece>` — com **lane nova no `physics_ecs_c9`**
  (hoje o hash 3-OS não cobre o cenário).
- **Aninhamento sem limite** (recusa só ciclo) e **duas variantes**: irmã (Figma — o
  `vec_variants.rs` de hoje, zero schema) e derivada (raiz do variant é instância do base), com
  re-key determinístico ao trocar variant↔base. Override sem alvo vira lista **"Overrides sem
  alvo"** — nunca auto-apagado (Unity), volta a pegar se a peça voltar.

### 4. O `VecInstance` é SUBSUMIDO, não mantido ao lado

Duas respostas a *"o que é uma instância?"* é a divergência que o próprio `vec_component.rs` proíbe.
As peças de instância vetorial viram entidades; o produtor `InstanceLive` morre; os verbos e a UI
(Create/Place/Detach/Reset/Update Main/Swap/Variant) ficam, sobre o mecanismo geral; documentos com
`VecInstance` **materializam no load** (degrau de schema).

## Consequências

### O que fica melhor, medido

- ⭐ Captura do undo: **23,8 → 0,27 ms** @10 k parado; 0,95 ms com 10 % sujo; delta 1 % = 24,5 KB
  (97,9× menor que o snapshot). O teto que proibia materializar instâncias **deixa de existir**.
- ⭐ Instância com física de verdade: ragdoll instanciado cai e a junta prende **os corpos dele**
  (gate novo) — hoje a junta copiada prenderia os do mestre (nome sufixado + `stable_name_id`).
- ⭐ Renomear deixa de desligar bindings/joints (id opaco, não hash de nome).
- ⭐ Reordenar irmãos passa a ser desfazível e a sobreviver ao restore (bug pré-existente, classe
  BUGS #15).
- ⭐ Override sobrevive a renomear/reordenar/mover-dentro-do-mestre/mudar-forma-de-componente — a
  coluna que nenhuma das sete referências fecha inteira ([doc 04 §2.6, tabela](../../Components/04_decisao_arquitetura.md)).

### O preço, nomeado

- **A maior migração da história do repo** (`StableId` + `SiblingOrder` + snapshot v2 + duas
  famílias de `stable_name_id` + materializar `VecInstance`) — e a wave do item 1 **não é isolável**.
- Editar um mestre com N instâncias suja N×peças linhas do undo (linear no fan-out: 100×10 ≈
  0,95 ms + 247 KB/passo).
- Clipe de timeline autorado no mestre **não propaga** até existir `SequencePlayer` (TOP-20 #19);
  config de física propagada chega ao solver **no Reset** (`bodies.rs:169`) — os dois declarados,
  com degrau nomeado.
- O **restore** do undo continua O(mundo) até a fase F8 (diff por `StableId` — o passo 1 do
  Blender); `VecScene`/`FlipDoc` continuam clone inteiro até lá.
- O mestre não simula — o que simula é a instância que *Criar componente* deixa no lugar. Muda a UX
  atual do vetor (mestre visível no canvas → modo *Editar mestre*).

## Alternativas medidas e recusadas

| alternativa | por que não |
|---|---|
| **Instância derivada** (1 entidade, geometria derivada — a v1 deste doc) | sem física (peça não existe como entidade), aninhamento nem renderiza hoje, e vetada pelo dono. A lei certa dela (chave por id, esparsa, canônica) sobrevive aqui |
| **Diff estrutural** (Unity/Godot clássico) | a chave é a estrutura, que é o que o autor do mestre tem direito de mudar; override bloqueia propagação para sempre e em silêncio; Godot descarta órfãos com WARN e **apaga o undo** ao propagar |
| **Camadas com força** (USD-lite) | segunda máquina de resolução ao lado do ECS ([memória: dois motores, um estado](../../../project-memory/feedback_two_engines_one_state_is_worse_than_a_slow_engine.md)); a única ideia que valia — undo O(edição) — obtém-se por ticks + cache, sem a máquina |
| **`Inherit` do flecs** (storage partilhado, sem cópia) | não há override por campo, e o flecs **aborta** mudança estrutural em prefab instanciado — resolve proibindo exatamente o que o Enio pediu |
| **Sync por reinstanciação** (Godot editor) | funciona e paga com `clear_history` do undo — o anti-exemplo literal |
| **Manter `VecInstance` ao lado do mecanismo novo** | duas respostas a "o que é uma instância?" — a divergência que o `vec_component.rs` existe para impedir |

## Referências

- **A forma longa:** [doc 04 v2 — a arquitetura, §2](../../Components/04_decisao_arquitetura.md) · auditoria [doc 01](../../Components/01_auditoria_modelo_de_objeto.md) · pesquisa [doc 02](../../Components/02_pesquisa_composicao_e_prefab.md)
- **Evidência e refutações:** [`docs/Components/pesquisa/instancias_2026-08-21/`](../../Components/pesquisa/instancias_2026-08-21/) — 6 pesquisas + 1 medição + 3 refutações, com `file:line`
- **Plano vivo:** [`docs/Components/05_plano_de_implementacao.md`](../../Components/05_plano_de_implementacao.md) (fases F0–F5)
- ADRs da casa que este honra: [0025](0025-gameobject-model.md) (GameObject = Entity+Components) · [0074](0074-sprite-component-boundary.md) (regra dos 3 lugares) · [0075](0075-multiagent-parallelism-ecs-decoupling-not-runtime-plugins.md) · [0131](0131-physics-global-runtime-truth-rapier-ecs-bridge.md) (config, nunca estado de solver) · [0037](0037-stable-entity-wire-id-scenedoc.md)
