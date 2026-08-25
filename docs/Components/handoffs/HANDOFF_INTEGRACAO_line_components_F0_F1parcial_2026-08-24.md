# Handoff de integração — `line/components`, 2026-08-24 (F0 completa + F1 **PARCIAL**)

> DIRETRIZ §1.5.9. Escrito **a meio da F1, por pedido do Enio** — a linha **não fechou**.
> Leia o §0 antes de decidir integrar: há uma condição que o próprio plano proíbe.

---

## §0 ⛔ LEIA PRIMEIRO — a linha está a meio de uma fase declarada NÃO-ISOLÁVEL

O plano ([05 §F1](../05_plano_de_implementacao.md)) escreve, sobre esta fase:
*«A ordem interna desta fase (**não isolável** — checkpoints obrigatórios)»*, e sobre o passo 5:
*«As duas famílias de `stable_name_id`, **na MESMA wave** (o `name.rs:73-79` proíbe metade)»*.

**Feito:** passos 1, 2, 3, 4 e **metade do 5** (a família da FÍSICA).
**Por fazer:** a outra metade do 5 (a família da **TIMELINE**) e o passo 6 (corte da Sprite).

⇒ **Integrar agora põe o `main` exactamente no estado que o plano proíbe: meia migração.**
Concretamente, o que isso significa no produto:

| | antes desta linha | com esta linha integrada AGORA | com a F1 completa |
|---|---|---|---|
| renomear um corpo com junta | junta **desliga** | ✅ junta aguenta | ✅ |
| renomear um objeto **animado** | binding **desliga** | ⚠️ binding **continua a desligar** | ✅ |

Ou seja: o artista passaria a ver a física a aguentar um rename e a **animação não**, sem nada
que explique a diferença. É uma inconsistência visível, não um risco técnico escondido.

**Não é uma recusa — é o preço, nomeado.** A árvore está verde e o `--ff-only` funde; o que
está em causa é produto. Três saídas, na ordem em que as recomendo:

1. ⭐ **Esperar a F1 fechar** (falta a timeline + o corte da Sprite). É o que o plano desenha.
2. **Integrar assim** e aceitar a assimetria até a próxima integração — defensável se houver
   outra linha à espera de `StableId`/snapshot v2 para não rebasear duas vezes.
3. Integrar só até ao commit `2b210cc4e` (F1 passo 4) — ⛔ **NÃO recomendado**: nesse ponto a
   física ainda resolve por hash de nome, e o passo 5a é o que a converte. Cortar aí não evita
   a assimetria, só a move.

---

## §1 Identidade

| | |
|---|---|
| branch | `line/components` |
| HEAD | `95bfa59676465e97522f212165258fed9e214e43` |
| merge-base com `main` | `5038249c698f74d6d277a10fa42a4d3bd59ad045` |
| commits | **7** |
| arquivos de código tocados | **158** (dos quais ~110 são cenas/testes de smoke da física) |

Ordem dos commits (é **linear e cada um compila**; ⚠️ não reordene — o passo 3 depende do 1):

```
b73349143  docs   ADR-0164 + ADR-0165 + plano entram na linha
791adb8fc  docs   ADR-0166 (composição do Inspector)
eb2a4d1c0  F0     descritor de componente + insert_default + piloto §7
b53272dd4  F1.1-2 StableId + SiblingOrder
d72ed47e7  F1.3   WorldSnapshot v2, `canonicalize` morre
2b210cc4e  F1.4   PRIMEIRA migração de PROJECT_SCHEMA (95→96)
95bfa5967  F1.5a  física aponta por identidade
```

---

## §2 Foundational / compartilhado tocado, e por quê

| arquivo | o quê | por quê |
|---|---|---|
| **`crates/ph2d-component-desc/`** (crate NOVA) | descritor de componente, catálogo de 108 tipos | F0. Folha PURA (zero deps) |
| `ph2d-ecs/src/stable_id.rs` · `sibling_order.rs` (NOVOS) | identidade e ordem de irmãos | F1.1-2 |
| `ph2d-ecs/src/scene/save.rs` | `EntitySnapshotRow` ganha `id`, `parent` vira `StableId`, `VERSION` 1→2 | F1.3 |
| `ph2d-ecs/src/scene/save_v1.rs` (NOVO) | a forma v1 CONGELADA + `migrate_v1_to_v2` | F1.4 |
| `ph2d-ecs/src/scene/registry.rs` | `insert_default` + `desc` na vtable, `register_default` | F0 |
| `ph2d-ecs/src/scene/registry_tests.rs` (NOVO) | ⚠️ **o `mod tests` SAIU do `registry.rs`** — ele estourou o teto de 700 LOC (747) | F0 |
| `ph2d-ecs/src/scene/snapshot.rs` · `children_order.rs` | as duas travessias passam a ler `SiblingOrder` | F1.2 |
| `ph2d-physics-ecs/` (bridge/joints, bridge/rope, components/rope, joint_group, name_refs NOVO) | resolução por identidade | F1.5a |
| `ph2d-render/src/registry.rs` · `ph2d-script/src/registry.rs` | **só o contador** (espelhos) | F1.1-2 |
| `ph2d-field-ecs/src/lib.rs` | `register` → `register_default` | F0 (varredura) |
| `ph2d-panel-inspector/` | a §7 lê os rótulos do descritor | F0 (piloto) |
| `shells/desktop/` | migração, load/save, varreduras no passe de quadro, inspector de joints, hierarquia | F0-F1 |

---

## §3 Símbolos que podem COLIDIR

### 3.1 A saída de `collision-surface.sh`, colada (não escrita de memória)

```
SUPERFÍCIE DE COLISÃO — line/components contra main
  merge-base 5038249c6   ·   7 commit(s)   ·   168 arquivo(s)
▸ SCHEMAS
  ⚠ PROJECT_SCHEMA                         96   (base: 95)
  ⚠   └ tripla do gate               (96, 13, 14)   (base: (95, 13, 14))
    VEC_SCENE_SCHEMA                       14   (base: 14)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
▸ REGISTRO DE COMPONENTES
  ⚠ ph2d-ecs                              —   (base: 69)
  ⚠ ph2d-render (espelho)                  71   (base: 70)
  ⚠ ph2d-script (espelho)                  71   (base: 70)
▸ CONTRATO CONGELADO (§6)
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado
▸ ADR
    último no disco: 0166   próximo livre: 0167
  ⚠ esta linha cria ADR: 0164 0165 0166
▸ Cargo.lock
  ⚠ 1 pacote(s) '+name' novo(s):  "ph2d-component-desc"
▸ MARCADORES DE CONFLITO
    nenhum nos arquivos da linha
▸ TETOS DE LOC
    nenhum arquivo da linha passa do teto
```

⚠️ **PRAZO DE VALIDADE:** esta tabela mede a linha contra o `main` de **2026-08-24**. Re-rode a
sonda em cada worktree imediatamente antes de fundir (DIRETRIZ §1.5.3) — a divergência entre as
duas leituras é ela própria um achado.

### 3.2 ⚠️ Duas linhas da sonda que precisam de leitura humana

- **`ph2d-ecs — (base: 69)`** — o traço **não é um erro da linha**: a sonda procura o
  `assert_eq!(reg.len(), N)` dentro do `registry.rs`, e ele **mudou de arquivo** (o `mod tests`
  foi cortado para `registry_tests.rs` pelo teto de 700 LOC). O valor real é **70**, e vale a
  conta: `69 (base) + StableId… ` — **não**: o `StableId` acabou **fora** do registo (§3.4), e o
  `+1` é o **`SiblingOrder`**. ⇒ **ph2d-ecs 69 → 70**, espelhos **70 → 71**.
  ⛔ Se outra linha acrescentar componentes, o valor certo é a **SOMA**, nunca um dos lados.
- **`esta linha cria ADR: 0164 0165 0166`** — o **0164 e o 0165 não são meus**: o Enio escreveu-os
  no primário e eles estavam **não commitados** quando a linha abriu; eu trouxe-os para cá
  (commit `b73349143`, cópia conferida byte-a-byte). Só o **0166** nasceu nesta linha.
  ⚠️ Se o Enio commitar os dois no `main` entretanto, o rebase dá conflito de **arquivo idêntico**
  — resolva mantendo qualquer um dos lados e **regenere o índice** (`bash scripts/adr-index.sh`),
  ⛔ nunca editando o `decisions/README.md` à mão.

### 3.3 Números que SOMAM entre linhas

| o quê | base | esta linha | quem confere |
|---|---:|---:|---|
| `PROJECT_SCHEMA` | 95 | **96** | escada (`project_schema.rs`) **+ a tripla** (`project_schema_tests.rs`) |
| `WorldSnapshot::VERSION` | 1 | **2** | `save.rs` |
| registro `ph2d-ecs` | 69 | **70** | `registry_tests.rs:147` |
| espelho `ph2d-render` | 70 | **71** | `ph2d-render/src/registry.rs:55` |
| espelho `ph2d-script` | 70 | **71** | `ph2d-script/src/registry.rs:62` |
| registro física | 32 | 32 (intocado) | — |
| registro field | 5 | 5 (intocado) | — |

⚠️ **O degrau do `PROJECT_SCHEMA` tem de ser RECONTADO no dia da integração.** Se outra linha
subir o schema antes, o meu 96 vira o próximo livre — e ⛔ a colisão passa **muda** se as duas
escreverem o mesmo literal.

### 3.4 Componentes novos, e um que deliberadamente NÃO é componente registado

- **`SiblingOrder`** — registado (`ph2d::ecs::SiblingOrder`), `Machinery` no catálogo.
- **`StableId`** — ⛔ **NÃO registado, e a ausência é a decisão.** Ele viaja no campo
  `EntitySnapshotRow::id`, uma fonte só. Registá-lo poria a identidade também num
  `ComponentBlob`, e a cópia profunda da F4 (`extract_component_snapshot` + `insert_from_bytes`,
  que copiam blobs **verbatim**) daria à cópia a identidade do **original** — exactamente o que o
  ADR-0164 §2.7 manda evitar. Mantê-lo fora torna o erro impossível em vez de o deixar por
  lembrar. ⚠️ Quem quiser registá-lo tem de ler isto primeiro.
- **`StableIdCounter`** — `Resource`, não componente. Persistido em `ProjectFile.stable_id_counter`.

### 3.5 Ids / consts / superfície pública nova

| símbolo | onde |
|---|---|
| `ComponentRegistry::register_default::<T>` | `ph2d-ecs/scene/registry.rs` |
| `ComponentTypeEntry::{insert_default, desc}` | idem |
| `ph2d_ecs::{StableId, StableIdCounter, assign_missing_stable_ids, stable_id_of, stable_id_for_name, entity_of_stable_id}` | `stable_id.rs` |
| `ph2d_ecs::{SiblingOrder, assign_missing_sibling_order, set_sibling_order, ordered_children, sibling_key}` | `sibling_order.rs` |
| `ph2d_ecs::scene::{WorldSnapshotV1, EntitySnapshotRowV1, migrate_v1_to_v2, next_free_after_migration}` | `save_v1.rs` |
| `ph2d_physics_ecs::{ResolvedRefs, resolve_body_names}` | `name_refs.rs` |
| `ProjectFile.stable_id_counter` (campo novo, **no fim**) | `shells/desktop/src/project.rs` |
| ⚠️ `ProjectFile` e os 12 campos dele passaram a `pub(crate)` | idem (o `project_migrate` constrói-o) |
| ⚠️ `SavedAsset` passou a `pub(crate)` | idem |
| **nenhum `NodeId`/`IconId`/token novo** | — |

---

## §4 Contratos congelados encostados

**NENHUM.** A sonda confirma `ph2d-nodegraph/src/node.rs` e `ph2d-editor-core/src/tool.rs`
**intocados**. `NodeOp`/`OpResolver`/`NodeManifest`, `Tool`/`RasterEditTool`/`CanvasPaintTool`/
`PanelEvent` e a superfície do `ph2d-vector-doc` não se mexem.

⚠️ **O ADR-0074 é encostado sem ser movido:** o teto de 32 componentes opcionais da Sprite
recebe **+1** (`SiblingOrder` não é da Sprite, mas o corte da Sprite do **passo 6, ainda por
fazer**, gastará +3). Nada a fazer agora; é aviso para quem contar depois.

---

## §5 O que só o `ship.sh` pega (o gate de integração NÃO roda)

Não rodei nenhum destes — são do ship:

- **`cargo fmt --check`** — rodei `cargo fmt` nas crates tocadas, mas **não** na workspace.
- **`clippy --all-targets --features …`** — ⚠️ **não rodado**. A linha tocou 158 arquivos, e há
  código novo com `#[allow]` nenhum: espere achados.
- **`cargo machete`** — ⚠️ **três dependências novas** de path: `ph2d-ecs → ph2d-component-desc`,
  `shells/desktop → ph2d-component-desc`, `ph2d-panel-inspector → ph2d-component-desc`. As três
  são usadas; o machete não deve reclamar, mas é ele quem decide.
- **`cargo deny` / `audit`** — nenhuma dependência EXTERNA nova (a crate nova tem **zero** deps).
- **`typos`** — rodei nos docs; **não** no código novo.
- **`nextest --cargo-profile ci-test`** — corri as suítes por crate em perfil `dev`
  (`ph2d-ecs`, `ph2d-physics-ecs`, `ph2d-host-desktop`, `ph2d-component-desc`,
  `ph2d-panel-inspector`), **não** a workspace inteira em `ci-test`.

⚠️ **O `physics_ecs_c9` (hash 3-OS) NÃO foi re-capturado.** O plano da F1 avisa que o
`deterministic_hash` **muda de valor** com o snapshot v2, e mudou. Não o toquei porque o
scope-creep aqui é grande; **o integrador ou a próxima janela tem de o re-capturar** e ver a
matriz 3-OS verde. É o item mais provável de partir o CI.

---

## §6 Ordem, dependências e o que smokar

### 6.1 Dependências

Linear. O único acoplamento duro: **o passo 3 (snapshot v2) exige o passo 1 (`StableId`)** — a
chave da linha é o id. E o passo 4 (migração) exige o 3.

### 6.2 O que **NÃO** foi smokado (nada foi — não houve smoke do Enio)

⚠️ **Tudo o que segue está provado por gate, não por olho.** Por ordem de risco:

1. ⭐ **Abrir um projeto gravado ANTES de hoje** (formato 95). É a primeira migração da história
   do repo e o caminho com mais superfície nova.
   ```
   cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && cargo run -p ph2d-host-desktop --release
   ```
   Depois: **Open Project…**, escolher um `.ph2dproj` antigo. Tem de abrir e dizer
   *"Project migrated from format 95 to 96"*. ⛔ Se disser *"refused"*, pare.
2. **Reordenar irmãos na Hierarquia** e dar **Ctrl+Z** — tem de voltar. E depois de gravar,
   fechar e reabrir, a ordem tem de estar como ficou.
3. **Renomear um corpo que tem junta** (cena `PH2D_PHYSICS_SMOKE=6` ou `=67`) — a corrente **não**
   pode cair. Era o defeito.
4. **Copiar um ragdoll** e dar Play — cada cópia tem de prender os **seus** corpos.
5. **A §7 Ordering do Inspector** — tem de estar **visualmente idêntica**. Há gate com os oito
   rótulos ao byte, mas a §7 é a seção-piloto e é onde uma regressão de UI apareceria.

---

## §7 `incremental/` — ⛔ NÃO reclamado, de propósito

A DIRETRIZ §1.5.9 item 7 manda `rm -rf target/*/incremental` **ao fechar**. A linha **não
fechou** (F1 a meio), e o `incremental/` do perfil `dev` é o que faz o `cargo check -p` voar
durante a jornada. Reclamo-o quando fechar. Se a integração acontecer antes disso, o comando é:

```
rm -rf /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components/target/*/incremental
```

---

## §8 A linha do `CLAUDE.md §5`, quando isto integrar

O §5 recebe **UMA linha** (a narrativa é este handoff). Módulo novo — sugestão de entrada:

> - **Componentes / instâncias** — identidade de objeto (`StableId`), ordem de irmãos como dado,
>   snapshot v2 e a 1ª migração de `PROJECT_SCHEMA` do repo ([ADR-0164](../../architecture/decisions/0164-instances-are-real-entities-linked-by-stableid-with-live-sync-and-incremental-undo.md) ·
>   [0165](../../architecture/decisions/0165-assets-are-born-inside-the-app-three-level-identity-index-before-browser.md) ·
>   [0166](../../architecture/decisions/0166-the-inspector-shows-what-the-object-has-and-components-attach-through-one-palette-filtered-by-object-type.md)).
>   **Aberto:** a família `stable_name_id` da **timeline** (meia migração — o `name.rs` proíbe
>   metade) · o corte da Sprite (F1.6) · o `physics_ecs_c9` por re-capturar · F2-F8 do
>   [plano vivo](../../Components/05_plano_de_implementacao.md).

---

## §9 ⚠️ Cinco coisas que uma leitura rápida do diff entende ao contrário

1. **Nove gates foram REESCRITOS, não apagados.** Eles pinavam o defeito — *"renomear um corpo
   **tem de** desligar a junta"*, *"dois corpos com o mesmo nome **não podem** ser juntados"* — e
   as razões que davam eram **exactas**; o que mudou foi a premissa (*"the id IS the name"*).
   Cada um ficou com a razão antiga citada e com a metade que continua verdadeira medida ao lado
   (o corpo que **desaparece** ainda solta a junta) — senão o gate não distinguiria *"ficou
   robusto"* de *"deixou de reparar em coisa nenhuma"*.
2. **`canonicalize` foi apagado e a propriedade dele NÃO se perdeu** — ela mudou de dono
   (`world_to_snapshot` ordena por `StableId`, que sobrevive ao respawn por construção). No sítio
   onde a função vivia ficou a nota com a medição (18,7 ms → 0,088 ms) e o ⛔ de não a reintroduzir.
3. **~110 dos 158 arquivos são cenas/testes de smoke da física**, e a mudança neles é **uma
   linha** (`resolve_body_names`). O diff parece muito maior do que a decisão.
4. **`world_to_snapshot` passou a `&mut World`** por desenho, não por acidente: a identidade é
   garantida na **derivação** e não em cada um dos 44 sítios de chamada.
5. **O `StableId` não estar no registo é a decisão, não um esquecimento** (§3.4).

---

## §10 Três premissas do plano que a implementação REFUTOU

Registadas aqui porque a próxima janela vai ler o plano antes do código.

1. ⚠️ **«toda entidade editável tem `Transform`»** — a frase está no `undo.rs` há meses e
   **envelheceu**: desde o módulo de modelagem 3D, os FILHOS de uma peça não o têm. O critério do
   `StableId` é `Transform` **ou** `ChildOf`. Custou um gate vermelho: uma peça de 5 nós voltava
   do arquivo **com 2**, passando em todos os outros gates.
2. ⚠️ **`Entity::to_bits()` não é a ordem de criação no bevy 0.18** — ele a **inverte** (três
   entidades criadas em sequência davam ids `3, 2, 1`). A varredura ordena por `Entity::index()`.
   ⛔ Isto **não** acusa o `assign_missing_root_order`: lá a chave é a mesma que a árvore usa.
3. ⚠️ **O `Attach` do descritor tem TRÊS estados, não dois** — quem o provou foi o compilador:
   **27 dos 109 tipos não implementam `Default`**, e 17 estavam marcados como anexáveis.

---

## §11 Estado das suítes (perfil `dev`, máquina calma)

| crate | passaram | falharam |
|---|---:|---:|
| `ph2d-ecs` | 258 | 0 |
| `ph2d-physics-ecs` | 645 | 0 |
| `ph2d-component-desc` | 6 | 0 |
| `shells/desktop` (bins) | 3516 | 0 |

⚠️ **Uma flake conhecida apareceu numa corrida e não na seguinte:**
`flip_smooth::resample_measurement::precisao::orcamento::…` — é a família documentada no
[`CLAUDE.md` §5.0](../../../CLAUDE.md) (gate de recurso sob fan-out), **não** desta linha. O diff
não toca no Flip.

**Gates novos: 40**, todos com prova de mutação onde a mutação é possível.
