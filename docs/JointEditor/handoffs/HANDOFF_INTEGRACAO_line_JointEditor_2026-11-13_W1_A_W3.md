# HANDOFF DE INTEGRAÇÃO — line/JointEditor, W1–W3f (13/11/2026)

Leitor: o agente integrador. Mecanismo de cada wave nos handoffs de continuação:
[W1](HANDOFF_CONTINUACAO_W1_2026-11-12.md) · [W2](HANDOFF_CONTINUACAO_W2_2026-11-12.md) · [W3](HANDOFF_CONTINUACAO_W3_2026-11-12.md) ·
[plano](../01_plano.md) · [números](../02_gabaritos_medidos.md) · [ADR-0201](../../architecture/decisions/0201-a-montagem-de-juntas-e-um-objeto-e-o-gabarito-e-dado-puro.md).
⚠️ **A linha NÃO está fechada pelo §1.5.9**: faltam provas de mutação da W3, auditoria, `agent-loop-profile.sh` e o corte de LOC (§5).

## 1. Identidade

Branch `line/JointEditor` · HEAD `0ac47fe92c` (mais o commit deste handoff) · merge-base com o main `4f46a3573c` · 23 commits (+1 deste) · 167 arquivos.
Worktree `Worktrees/line-JointEditor`.

## 2. O que a linha entrega

- **W1** — `ph2d-joint-templates` (gabaritos de junta, dado puro) e a montagem como objeto: componente `JointAssembly` (ADR-0201), cena de física **123**.
- **W2** — a ponte `one_chain` resolvida como UMA corrente; o cartão Joint fala pelo papel; bloco «Joints» no corpo; soltar um objeto em Body A/B. Smoke aprovado pelo dono (12/11).
- **W3** — a aba **Joint Editor** (versão B): crate nova `ph2d-app-joint-editor` (família `joint-editor`).
- **W3b–W3f** (pedidos do dono 13/11) — o painel de ajustes virou coluna vertical à direita no idioma do Inspector: rótulo acima de cada caixa, Joint|Template numa fila, cartões recolhíveis, divisória que muda a largura (do `WidgetStore::DOCK_W_MIN` a metade da aba; nasce com `INSPECTOR_W_PX`), barra arrastável, segmentados que encolhem à 1.ª palavra e quebram em fileiras, reflow a uma coluna abaixo de `FIELD_MIN_W`, limites Min/Max com setas.

## 3. Foundational / compartilhado tocado (fora da pasta do módulo)

- `shells/desktop`: `Cargo.toml` (dep), `init.rs` (+1 linha, install), `render_loop/fase_snapshots_publish.rs` (+1, `frame(sim, hero)`), `project_schema.rs` + `_tests.rs`, `render_loop/fase_physics_edits.rs`, `render_loop/snapshots_inspector.rs`, `asset_drag_wire.rs`, `despacho_metodos_picks_e_arrastos.rs`.
- `ph2d-editor-core`: `interaction/state/joint_editor_ops.rs` (novo) + campo `joint_editor_request` (`mod`, `store_fields`, `store_core`); `hier_types.rs` (`ObjectType::Assembly`); `screens/hero/inspector_model_joint.rs`; `lib.rs`/`screens/mod.rs`.
- `ph2d-component-desc` (`catalog/physics*.rs`, +`JointAssembly`), `ph2d-app-components` (`asset_drop.rs`, `asset_drop_field_tests.rs`, `instance_refs.rs`), `ph2d-app-macros/src/mundo/portas.rs`, `ph2d-app-physics` (montagens, `joint.rs` agora `pub`, cena 123), `ph2d-physics` (`world/{chains,one_chain}`).
- `ph2d-panel-inspector`: `ids/inspector_joint.rs` (`INSP_JOINT_OPEN_EDITOR`), `sections/joint*.rs`, `event_joint.rs`, `event_physics.rs`, `populate_physics.rs`, `paint.rs`, `lib.rs`.
- `ph2d-i18n`: `joint_editor_tab.rs` (novo: `joint_editor.*`, `panel.inspector.joint.open_in_joint_editor`) + `inspector.rs`, `component_catalog.rs`, `lib.rs`.
- Tokens: `ph2d-tokens` (`color.rs`, `skin_body.rs`), `docs/design/tokens.json` (`obj-assembly`, 3 temas).
- Registry: `ph2d-app-registry-init` (Cargo.toml + lib.rs, **gerado**), `Cargo.lock`.
- Gates: `architecture_the_shell_only_shrinks.rs` (entrada nova da crate), tabela `TETOS_DAS_CRATES_APP`.
- Docs: ADR-0201 + `decisions/README.md`, emenda do ADR-0131, `CLAUDE.md` (Física §1/§5.1 pela forma-filha; entrada nova «Joint Editor» no §5.1), `project-memory/MEMORY.md` + `project_joint_editor_version_b_2026_11_12.md`.

## 4. Superfície de colisão (saída de `bash scripts/collision-surface.sh`, 13/11, contra o merge-base `4f46a3573c`)

```
▸ SCHEMAS
  ⚠ PROJECT_SCHEMA                        238   (base: 237)
  ⚠   └ tripla do gate               (238, 13, 29)   (base: (237, 13, 29))
    VEC_SCENE_SCHEMA 29 · FLIP_SCHEMA 13 · DOC_VERSION (timeline) 25   (iguais à base)
▸ REGISTRO DE COMPONENTES: ph2d-ecs 121 (base 121); sonda cega em ph2d-render/ph2d-script (const mudou de ficheiro)
▸ CONTRATO CONGELADO (§6): node.rs e tool.rs intocados
▸ ADR: último no disco 0201 · próximo livre 0202 · esta linha CRIA 0201
▸ Cargo.lock: +ph2d-app-joint-editor, +ph2d-joint-templates (internos; nenhum externo novo)
▸ marcadores de conflito: nenhum · tetos de LOC: nenhum arquivo da linha passa do teto
```

A tabela é REFERÊNCIA (§1.5.9 item 3): leia o valor do main no ficheiro (`git show main:<arq>`) e some o delta.

## 5. Contadores como DELTA

- `PROJECT_SCHEMA` **+1** (237 → 238), tudo na W1 (registo de `JointAssembly`; `PhysicsJoint.assembly/role/one_chain` no FIM, serde posicional). W2 e W3 não somam degrau. ⚠️ **Reconte com `python3 scripts/schema-recount.py`** e escreva o degrau nos três sítios (escada em `project_schema.rs`, tripla do gate, `project_schema_tests.rs`).
- Registo de componentes **+1** (`JointAssembly`; reconte nos três espelhos) · ADR **+1** (0201 provisório) · cenas de física **+1** (122 → 123).
- Crates **+2** (`ph2d-joint-templates`, `ph2d-app-joint-editor`) · famílias de app **+1** · routers de smoke **+1** (`PH2D_JOINT_EDITOR_SMOKE`) · `ObjectType` **+1** (17 → 18) · tokens de cor **+1** (`obj-assembly` / `ObjAssembly`) · WRITERS de `every_registered_physics_component_has_a_ui_writer` **17 → 19**.

## 6. Um merge pode partir

- **Registry-init:** regenerar com `cargo run -p ph2d-app-sync`, nunca mesclar à mão.
- Lista WRITERS (outra linha pode acrescentar), ordem das famílias em `init.rs`, `TETOS_DAS_CRATES_APP`, contador de `ObjectType`/tokens, número do ADR e da cena 123.
- Escada/tripla do `PROJECT_SCHEMA` (arquivos irmãos).
- Ligação com o ADR-0199: a forma é FILHA «Collision Shape»; qualquer cena/teste nova do main com `Collider` no corpo parte nos testes da montagem.

## 7. Prova de fecho (o que foi e o que NÃO foi corrido)

Verde, por nome: os 22 testes da crate `ph2d-app-joint-editor` · `seam_joint_w2` 5/5 · `every_registered_physics_component_has_a_ui_writer` · `a_tabela_inglesa_fala_ingles` · `toda_chave_do_load_tem_texto` · `hr15_no_hardcoded_ui_strings` · gates de tokens/ícone/cor de tipo · `the_shell_links_exactly_the_registered_families` · `no_two_families_claim_the_same_router`. fmt/clippy da crate limpos na W3e.

**VERMELHO conhecido** — `the_shell_only_shrinks_and_every_app_crate_too` (decisão do integrador; a cura pedida pela catraca é CORTAR/mover, não subir o número):
- `ph2d-app-components` 80 932 contra 80 869 (+63; `asset_drop_field_tests.rs` +118 da W2: mover ou cortar testes);
- `ph2d-app-physics` 39 545 contra 37 798 (+1 747; montagens `joint_assembly*.rs`, `joint_assembly_card.rs`, `joint_drop.rs`, cena 123 — não sobem para a crate nova sem inverter `card_context`; opções e ressalva «medir antes» no §4.1 do handoff W3);
- `ph2d-app-joint-editor` (crate nova) está no tecto 4 651 mas **precisa de recontagem após a W3f**.

**NÃO feito (fecho §1.5.9 incompleto):** provas de mutação da W3 (`drop_into` por papel, ordem de `role_at`, `scope_index`) · auditoria · `agent-loop-profile.sh` · `cargo machete`/cruzamento `aarch64-apple-darwin` (a crate não usa `#[cfg(target_os)]`, mas não foi corrido) · `censos-da-arvore-combinada.sh` após rebase · build `smoke` final.

## 8. Smokes (o integrador não os corre; são do Enio)

Binário: `cargo build -p ph2d-host-desktop --profile smoke` (na árvore já integrada).

1. `cd <árvore> && PH2D_PHYSICS_SMOKE=123 PH2D_JOINT_EDITOR_SMOKE=<n> ./target/smoke/ph2d-host-desktop`, com `n` = 1 Car · 2 kart por montar (três peças soltas; arrastar da Hierarchy para as formas tracejadas) · 3 Bridge · 4 Door · 5 lista Template aberta · 6 painel largo · 7/8/9 Car/Door/Bridge com o painel no mínimo.
2. Abre a aba «Joint Editor»; na 2, arrastar «Kart Body» para a forma Body acende-a e solta cria «Car 2»; «Open in Inspector» leva ao cartão Joint, que tem «Open in Joint Editor»; Ctrl+Z desfaz um passo. Arrastar a divisória do painel muda a largura; nos níveis 7–9 os campos ficam dois por fila.
3. Deu errado: soltar não cria nada, a forma não acende, a roda nasce no lugar errado, o painel corta rótulos em vez de reflow, Ctrl+Z não desfaz.

Fotos: `docs/JointEditor/fotos/` (`w3f_painel_7..9.png` são as últimas; `w3_maquete_lado_a_lado.png` e `w3d_lado_a_lado.png` comparam com a maquete).

## 9. ABERTO

O dono disse (13/11) que o painel «ainda precisa de ajustes»: acabamento a definir com ele. Resto:
1. Gabarito «Custom» (maquete B1) não existe no catálogo; a lista Template é vertical (a maquete é grelha 3 colunas).
2. Botão «Open in Joint Editor» só no cartão Joint; falta no bloco «Joints» do corpo.
3. A ponte mostra Joint «Rope» (família do gabarito) com cartões de Pin.
4. Sem alça de motor no desenho; alça do entre-eixos só em Wheel de duas rodas com peças na raiz.
5. Peça de cena (`may_be_world`) mostra «Pin to the scene» sem foto de clique.
6. **W4 «Make the parts»** por fazer (peças prontas, criadas pela regra do ADR-0199).
7. Corte de LOC e mutações (§7).

## 10. Limpeza feita

Ficheiros `.log` soltos de `docs/JointEditor/fotos/` (`w3d_painel_7`, `w3f_painel_7..9`) apagados; os `.log` já rastreados das fotos antigas ficam (ruído, não contrato).

## 11. Corte de LOC (13/11, por ordem de integração)

- Crate nova `ph2d-joint-assembly` (domínio, ABAIXO da `ph2d-app-physics` e da aba: a física chama para baixo, `card_context` não se inverte): `joint_assembly{,_tests}.rs`, `joint_assembly_card.rs`, `joint_drop.rs`, `joint_create.rs`, `joint_world.rs` (`pub(crate)` → `pub`) e `set_joint_body` (saiu de `joint.rs`). A física re-exporta os cinco módulos (`pub use`) e `joint::set_joint_body`: nenhum chamador mudou de caminho. 12 testes verdes (crate inteira).
- Cena 123 (`physics_smoke_joint_templates.rs`) → `ph2d-physics-smoke-scenes`; os testes dela ficam na física (`physics_smoke_joint_templates_tests.rs`, editam por `physics_tests::apply`).
- `ph2d-app-components`: o teste `as_linhas_body_da_junta_recebem_um_objecto` compactado (junta por `..Default::default()`, `InspectorJointInfo` ganhou `Default`; a linha inteira fica com o `seam_joint_w2`).
- LOC (`conta()`): physics 39 545 → 37 658 (tecto 37 798) · components 80 932 → 80 869 (= tecto) · joint-editor 4 649 → tecto recontado 4 749 · `ph2d-joint-assembly` 1 714 (tecto 1 814) · smoke-scenes 19 008 → 19 195 (+187 da cena, tecto movido com o código).
- Verde: `the_shell_only_shrinks*` 2/2, física `joint` 180, cena 123, aba 22, clippy `-D warnings` das quatro tocadas, `cargo machete` (tirou `ph2d-joint-templates` da física).
- ⚠️ Continua por fazer: provas de mutação da W3 (§7), `censos-da-arvore-combinada.sh` após o rebase.
- Provas de mutação da W3 (13/11, crate `ph2d-app-joint-editor`, verde 22 antes): 14 mutações (`role_at` ordem disco/caixa e `counts`, `scope_index` ×4, `drop_into` papel/many/veredito/sid/template) — 8 sangraram de início, 6 sobreviveram (`scope_index` fallback, `drop_into` many ×3, sem veredito, `sid`); +5 testes (`a_fileira_responde_em_cada_copia`, `a_queda_na_fileira_leva_todos_…`, `a_queda_recusada_nao_deixa_pedido`, `o_alcance_fora_das_opcoes_fica_na_ultima`, `a_queda_na_montagem_aberta_leva_o_sid_dela`) e as 14 sangram; árvore restaurada.
