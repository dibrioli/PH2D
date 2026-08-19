# HANDOFF — `line/motion-value` (doc 86 §9.6, item 3): filho vetor/flip de grupo SEM NOME carimba pelo drawing id

**Status:** FECHADO 2026-08-03 · no `main` em `6675733ad` (o commit que trouxe este arquivo).

**Data:** 2026-08-03 · **Linha:** `line/motion-value` (re-aberta pós-integração de 2026-08-02) · **Commit:** `54f2f9ffc` (feat) + handoff · **Estado:** FECHADA, **SMOKE APROVADO pelo Enio (2026-08-03, "parece OK")** — a estrela vetor SEM NOME do centro aparece nas 16 cópias. Pendente só de **ordem de integração (Enio-only)**.

## O que mudou (uma frase)

Um filho **vetor/flip de um grupo NOMEADO** não precisava mais de nome próprio para ser carimbado como instância: a membrana o resolve pelo seu **DRAWING id** (`VecPathRef`/`FlipObjectRef`), não pelo `Name`; e o bake o **tila** porque ele está num grupo nomeado (`entity_is_in_a_named_group`) — a **MESMA** relação de árvore que `group_externals` desce.

Era o único follow-up do doc 86 sem decisão de produto/arquitetura pendente do Enio. O plano (§9.6) o previa: *"o bake é name-keyed; incluí-lo pede chave por-entidade"* — feito, keyed pelo drawing id (undo/rename-stable, ao contrário de `Entity::to_bits`).

## Os arquivos (6 modificados + 1 novo, todos em `shells/desktop/src/`)

- **`render_loop/motion_bridge_objects.rs`** (a membrana): `resolve_leaf` delega a **`resolve_drawing_leaf`** (name-free, headless-testável — separado porque o braço de sprite precisa do atlas, este não). Nova porta **`entity_is_in_a_named_group`** (up-walk bounded `MAX_DEPTH=64`, precedente `container_of`), re-exportada `pub(crate)` por `motion_bridge.rs`.
- **`motion_object_bake.rs`** (vetor) e **`motion_flip_bake.rs`** (flip): cache keyed por **drawing id** (`VecPathId`/`FlipObjectId`), `name: Option<String>` vira **metadata** (não a chave); **`select_present`** tila `named ∪ group-referenced`; `tile_for(name)` → **`tile_for_id`**; `#[cfg(test)] seed_for_test`. `tiles()` (o publish individual) segue devolvendo só os nomeados.
- **`render_loop/motion_bridge.rs`**: +1 linha, `pub(crate) use objects::entity_is_in_a_named_group;`.
- **`motion_object_smoke.rs`**: a estrela vetor do centro do `=4` agora é **SEM NOME** (o caso do item 3) + mensagem atualizada.
- **`render_loop/motion_bridge_objects_tests.rs`** (NOVO): o `mod tests` da membrana foi extraído para este irmão `#[path]` (LOC — o parent foi de 501→446; o precedente `motion_flip_bake_tests.rs`).

## A porta única (o que impede a divergência)

O **up-walk** que o bake usa para decidir *quais desenhos tilar* (`entity_is_in_a_named_group`) é a MESMA relação de árvore que `group_externals` desce (`walk_group_transforms`). Um gate pina que os dois concordam (`the_named_group_predicate_matches_the_group_walk`, asserção `up == down`), então **o conjunto que o bake tila == o conjunto que a membrana carimba**. Onde o up-walk erra, ele degrada com segurança: no pior caso um tile a mais ou a menos, **nunca a tile errada**.

## Gates (5, red-first, cada mutação provada RED→GREEN — 8 corridas de mutação)

| Gate | Local | Mutações provadas (→ RED) |
|---|---|---|
| `an_unnamed_group_child_resolves_by_its_drawing_id` | membrana | name-keyed (o bug pré-fix) |
| `the_named_group_predicate_matches_the_group_walk` | membrana | drop-Name · drop-GroupedChildren · `MAX_DEPTH=1` (3) |
| `select_present_bakes_named_and_group_children_but_not_loose_art` | vetor | drop-group-check · bake-all (2) |
| `select_present_bakes_named_and_group_flip_objects_but_not_loose_art` | flip | drop-group-check · bake-all (2, gêmeos) |
| `select_present_skips_stale_bits` | vetor | *(invariante e2e — ver ⚠️)* |

⚠️ **`select_present_skips_stale_bits` NÃO é independentemente killable, e o doc-comment dele diz isso:** um entity despawned também não tem `Name`, então ele cai no skip *unnamed-AND-no-group* mesmo sem o guard `get_entity(..).is_err()` — dropar o guard **não** falsifica o gate. Ele pina o invariante END-TO-END (desenho despawned não é tilado) e o guard é robustez (espelha o `sync`). Mantido com a nota honesta em vez de over-claim.

**A medição do mecanismo é headless** (os dois primeiros gates usam a fixture EXATA: grupo nomeado + `VecPathRef(7)` sem nome + `FlipObjectRef(9)` sem nome). O smoke `=4` é a confirmação VISUAL para o Enio (precisa de display; não roda headless).

## Close-out (todos VERDES)

- `cargo test --bin` os 5 gates + vizinhos (`a_group_lays_...`, `the_membrane_publishes_...`): **7/7**.
- `cargo clippy -p ph2d-host-desktop --all-targets`: **limpo**.
- `shells/desktop/tests/file_loc_caps`: **2/2** (todos os arquivos tocados ≤ 600; o maior é `motion_flip_bake.rs` em **600 exato**).
- `ph2d-nodegraph/tests/architecture_contract_surface`: **3/3** (`NodeOp=2`/`OpResolver=1`/`NodeManifest=8` intactos — o `texture_id` é CONVENÇÃO de stream, não campo do manifest).
- `ph2d-editor-core/tests/no_tofu_glyphs`: **2/2** (a mensagem do smoke é ASCII puro, como as demais).

**Zero contrato congelado, zero schema (`PROJECT_SCHEMA`/`VEC_SCENE`/`DOC_VERSION` intactos), zero id/token/variant, zero dep, zero `Cargo.toml`.** ⇒ a linha fica **fora** de qualquer disputa de número da janela.

## Colisões de integração previstas

**Nenhuma sensível.** Só 6 arquivos de shell + 1 novo, todos exclusivos do cluster doc-86 (membrana/bakes/smoke). O arquivo novo `motion_bridge_objects_tests.rs` é `#[path]`-montado pelo parent — o gate da árvore combinada (`file_loc_caps`) já foi conferido verde aqui, mas ⚠️ **os gates de `shells/desktop/tests/` só correm na varredura impactada** ⇒ rodar `file_loc_caps` + `no_tofu_glyphs` na árvore combinada é obrigatório (a família do miss que outras linhas documentaram).

## Smoke (Enio, com display)

**`env PH2D_MOTION_OBJ_SMOKE=4 cargo run -p ph2d-host-desktop --release`** — o grupo 'Object' (sprite sem nome + **estrela vetor sem nome** + Flip 'GFlip') carimbado numa grade 4×4 = 16 cópias. **A pergunta de olho:** a estrela do CENTRO (sem nome) tem de aparecer nas 16 cópias; se o centro sair em branco, o item 3 FALHOU. A mensagem `[motion.obj smoke =4]` imprime isso — se a linha não aparecer, pare.
