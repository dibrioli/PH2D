# HANDOFF de continuação — line/JointEditor, W3 feita (12/11/2026)

Leitor: a próxima LLM da linha. Ramo `line/JointEditor`, worktree `Worktrees/line-JointEditor`. Commits da W3: `508bc60617` (a aba) ·
`e64bb45885` (acabamento pelas fotos, ícone/cor da montagem na Hierarchy) · `f0075889a0` (corpo na cor da montagem, roda como pneu,
`doc.rs` partido, rustfmt) · `d7e592960a` (tecto da crate nova). Não é handoff de integração (a linha não fechou).
Plano: [01_plano.md](../01_plano.md) · números: [02_gabaritos_medidos.md](../02_gabaritos_medidos.md) ·
W1: [HANDOFF_CONTINUACAO_W1_2026-11-12.md](HANDOFF_CONTINUACAO_W1_2026-11-12.md) · W2: [HANDOFF_CONTINUACAO_W2_2026-11-12.md](HANDOFF_CONTINUACAO_W2_2026-11-12.md) ·
memória `project-memory/project_joint_editor_version_b_2026_11_12.md`.

## 1. O que a W3 entregou

### 1.1 Crate nova `crates/ph2d-app-joint-editor` (família `joint-editor`, molde Input Map)

- `lib.rs` (FAMILY, `install`, `frame(sim, hero)`, `is_front`, `gesture_in_progress`) · `surface.rs` (`DocumentSurface`: abas, pintura acima do
  HUD via `chrome_over`, ponteiro, tecla, a11y por alvo pintado, `family_commands` = File ▸ «Joint Editor…» `CMD_OPEN`, `drop_hover`/`drop`
  com `Dropping::Objects`).
- `doc.rs` (estado: Focus Assembly/Draft, Menu, Hit, Edit, ScopeOpt; veredito da queda dito ANTES de soltar; `scope_options`
  «Both wheels · Rear wheel · Front wheel» ligado a `JointAssembly.linked`) · `doc_values.rs` (valor/commit/toggle/Drive) ·
  `input.rs` (press/motion/release, alças `Geo` Linear/Angular, digitar números, drop) · `model.rs` (`Snapshot` lido por quadro: montagens,
  peças, juntas com `build_joint_info` na unidade do cartão, nomes dos corpos).
- `apply.rs` (Edit → mundo SÓ pelas portas: `joint_assembly::{new_assembly,place_part,dissolve}`, `joint::joint_with_edit` (agora `pub`),
  `joint_assembly_card::set_linked`; `set_wheelbase` move a roda da frente + `local_a` + entidade-junta; a montagem só nasce com a 1.ª peça
  aceita, nunca vazia).
- `stage.rs` (`StageMap` metros→px, `MAX_SCALE` 280 px/m, `role_at` com discos à frente das caixas, `rule_ends`) · `paint.rs` (lista
  Assemblies com cor por montagem, seletores Joint/Template, lista suspensa com miniatura tracejada) · `paint_stage.rs` (formas tracejadas
  por papel vazio; peça cheia na cor da montagem; roda como pneu; eixo amarelo; barra laranja do curso; seta verde do motor; arco dos
  batentes do Pin; cota azul do entre-eixos medida na cena; alças LimitMin/LimitMax/Wheelbase; pé com a frase da queda) · `panel.rs`
  (faixa «Car 1 · 3 of 3 parts · 2 joints», «Open in Inspector», alcance, números por tipo com unidade, Drive Rear/Front/Both/None, caixas
  Limits/Motor/Breakable; altura por `wanted`) · `widgets.rs` · `smoke.rs`.
- Desfazer: pelo diff do projeto. Alças e números NÃO escrevem durante o gesto (pré-visualização), só no Up/Enter ⇒ um passo; por isso
  `undo_app_macros` não foi tocado.
- Queda da Hierarchy: pela porta existente `drop_hover`/`drop` da `DocumentSurface` (a do script): a lei `asset_drop` devolve `WriteCode` e
  `object_drag::write_code` chama `surface_input::drop`. Zero linhas novas na shell para o arrasto.

### 1.2 Abrir a aba

- `WidgetStore::request_joint_editor(sid)` / `take_joint_editor_request` (editor-core `interaction/state/joint_editor_ops.rs`; campo em
  `store_fields`/`store_core`, append). Botão «Open in Joint Editor» (`INSP_JOINT_OPEN_EDITOR`) no cartão Joint, só em junta de montagem
  (`sections/joint_assembly_rows.rs::paint_open_editor_row`; `event_joint.rs` pede a aba pela junta; `doc.open_on` aceita a junta e acha a
  montagem dona). Menu File ▸ «Joint Editor…» pelos comandos da família.
- Shell: +1 linha `init.rs` (install), +1 `fase_snapshots_publish.rs` (`frame(sim, hero)`), dep em `shells/desktop/Cargo.toml`;
  registry-init regenerado por `cargo run -p ph2d-app-sync`.

### 1.3 Hierarchy e i18n

- `ObjectType::Assembly` (marcador `ph2d::physics::JointAssembly`, `IconId::Link`, token novo `obj-assembly` / `ColorToken::ObjAssembly`;
  `docs/design/tokens.json` nos 3 temas: escuro `oklch(0.740 0.150 350)`, claros `oklch(0.420 0.150 350)`).
- Gate `every_registered_physics_component_has_a_ui_writer` VERDE: WRITERS 17→19 (`crates/ph2d-app-joint-editor/src/apply.rs`,
  `crates/ph2d-app-physics/src/joint_assembly_card.rs`) — fecha o §4.1 da W2.
- i18n: `crates/ph2d-i18n/src/joint_editor_tab.rs` (chaves `joint_editor.*` e `panel.inspector.joint.open_in_joint_editor`).

### 1.4 Smoke e fotos

- `PH2D_JOINT_EDITOR_SMOKE=1..5` sobre `PH2D_PHYSICS_SMOKE=123` (router próprio, `max_level` 5): 1 Car 1 · 2 carro por fazer + três peças soltas
  «Kart Body/Kart Rear Wheel/Kart Front Wheel» · 3 Bridge 1 · 4 Door 1 · 5 lista Template aberta.
- Fotos: `docs/JointEditor/fotos/w3_aba_1..5.png` e `w3_maquete_lado_a_lado.png` (maquete B3 × aba):
  [cena 2](Worktrees/line-JointEditor/docs/JointEditor/fotos/w3_aba_2.png) ·
  [lado a lado](Worktrees/line-JointEditor/docs/JointEditor/fotos/w3_maquete_lado_a_lado.png).

## 2. Provas (verdes, por nome)

- Crate nova, lib, 13 testes: stage (cabe/volta, roda ganha ao corpo, junta da roda no centro) · doc (Both/Rear/Front, alcance segue
  «Same as», escolher roda desliga «Same as», queda diz e recusa, Drive da frente troca o motor, entre-eixos medido) · apply (corpo+roda
  criam montagem e junta com papel certo, queda recusada não deixa montagem vazia, entre-eixos move roda e âncora) · widgets format.
- Costura `tests/it`, 3, pelas portas reais `surface_input::drop_hover/drop` e o ponteiro da aba:
  `arrastar_as_pecas_para_os_papeis_monta_o_carro`, `a_queda_recusada_diz_porque_e_nao_muda_nada`,
  `o_painel_edita_as_duas_rodas_ou_uma_e_abre_o_inspector`.
- Gates verdes: `every_registered_physics_component_has_a_ui_writer` · `the_shell_only_shrinks` · `the_shell_links_exactly_the_registered_families` ·
  `no_two_families_claim_the_same_router` · `every_registered_family_declares_a_reachable_router` · app-sync sem diff · `design_token_sync` e
  demais de tokens · `cada_tipo_tem_cor_propria_e_legivel` · `cada_tipo_tem_icone_unico` · `o_icone_do_objecto_pinta_a_cor_do_tipo` ·
  `hr15_no_hardcoded_ui_strings` · `a_tabela_inglesa_fala_ingles` e outros de i18n · `architecture_panel_wiring_parity` ·
  `panel_files_under_loc_cap` · `workspace_src_files_under_loc_cap` · `seam_joint_w2` 5/5.
- clippy `-D warnings` da crate nova verde ANTES de partir `doc.rs` (não repetido). `fmt --all` limpo (rustfmt também em
  `ph2d-physics` `world/{chains,one_chain}` e `ph2d-joint-templates`).
- **Mutação NÃO feita na W3** (fica para o fecho: `drop_into` papel, ordem de `role_at`, `scope_index`).

## 3. Contadores como DELTA

- `PROJECT_SCHEMA`: a W3 não soma degrau; o total da linha segue **+1** (238). Registro de componentes **+0**.
- Crates novas **+1** (`ph2d-app-joint-editor`) · famílias de app **+1** · routers de smoke **+1** (`PH2D_JOINT_EDITOR_SMOKE`) ·
  tokens de cor **+1** (`obj-assembly`) · `ObjectType` **+1** (17→18) · WRITERS **+2** · tecto da crate nova **4 049** (tabela
  `TETOS_DAS_CRATES_APP`).
- Superfície de colisão da W3: registry-init (gerado) · `shells/desktop` (`Cargo.toml`, `init.rs`, `fase_snapshots_publish.rs`) ·
  editor-core `interaction/state` (`mod`, `store_fields`, `store_core`, `joint_editor_ops.rs`) · `hier_types.rs` ·
  `architecture_the_shell_only_shrinks.rs` · `ph2d-tokens` (`color.rs`, `skin_body.rs`) · `docs/design/tokens.json` · `ph2d-i18n/lib.rs` ·
  panel-inspector (`ids/inspector_joint.rs`, `sections/joint.rs`, `joint_assembly_rows.rs`, `event_joint.rs`,
  `tests/it/every_physics_component_is_authorable.rs`) · `ph2d-app-physics/src/joint.rs` (`pub`).
- Um merge pode partir: a lista WRITERS (outra linha acrescenta), o registry-init (regenerar, não mesclar), o tecto das crates app, o
  contador de `ObjectType`/tokens, a ordem das famílias em `init.rs`.

## 4. ABERTO (declarado)

1. ⚠️ **Gate `the_shell_only_shrinks_and_every_app_crate_too` VERMELHO**, só por dívida da W1/W2: `ph2d-app-physics` 39 545 vs 37 798
   (+1 747); `ph2d-app-components` 80 932 vs 80 869 (+63). Plano de corte para o fecho: o código das montagens (`joint_assembly.rs`+testes,
   `joint_assembly_card.rs`, `joint_drop.rs`, ≈1 330 linhas) não sobe simplesmente para `ph2d-app-joint-editor`, porque `joint.rs`
   (`build_joint_info` chama `card_context`) e a cena 123 (`physics_smoke_joint_templates`, despachada por `smoke.rs` do app-physics) o
   chamam por dentro. Opções: (a) crate folha nova abaixo de app-physics com `joint_create`/`joint_world`/`joint_assembly` (exige mover
   `create_joint_at`); (b) inverter: `card_context` recebido pela shell; mover a cena 123 para `ph2d-physics-smoke-scenes` exige a
   montagem abaixo. **Medir antes de escolher.** Components +63 vem de `asset_drop_field_tests.rs` (+118) da W2: mover testes ou cortar.
2. «Custom» no seletor Template (maquete B1) não existe: o catálogo não tem gabarito Custom.
3. A lista Template é vertical; a maquete B1 é grelha de 3 colunas.
4. Seletor de tipos: só os 8 tipos com gabarito (Custom/Generic fora).
5. Nenhuma alça de motor no desenho (velocidade só no painel). A alça do entre-eixos só em Wheel com duas rodas e peças na raiz (`ChildOf`
   recusa).
6. «Open in Joint Editor» só no cartão Joint; o bloco «Joints» do cartão de física do corpo ainda não tem o botão.
7. Mutações da W3 por fazer (§2).
8. Peça de cena (papel `may_be_world`) mostra «Pin to the scene»; não fotografado em clique.
9. O rótulo do papel ainda pode ser cruzado pela linha do eixo na roda (tem fundo próprio).
10. Duas caixas de aceitar numa linha Body durante o arraste (herdado da W2).
11. W4 «Make the parts» por fazer.

## 5. PRÓXIMO

W4 do plano (peças prontas, sem mágica). No fecho da linha: corte de LOC do §4.1 + mutações + README/§5 do CLAUDE.md + handoff §1.5.9.

## 6. Smoke do dono (W3)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-JointEditor && PH2D_PHYSICS_SMOKE=123 PH2D_JOINT_EDITOR_SMOKE=2 ./target/smoke/ph2d-host-desktop`
2. Abre a aba «Joint Editor» com um carro por fazer: três formas tracejadas (Body, Rear wheel, Front wheel). Na Hierarchy (role até o
   fim) há «Kart Body», «Kart Rear Wheel», «Kart Front Wheel».
3. Arraste «Kart Body» para a forma Body: antes de soltar, a forma acende azul e o pé diz o que vai acontecer. Solte: aparece «Car 2» na
   lista da esquerda.
4. Arraste «Kart Rear Wheel» para Rear wheel e «Kart Front Wheel» para Front wheel: as rodas viram pneus, aparecem a seta verde do motor
   e a barra laranja do curso.
5. Embaixo, mude «Suspension stiffness»; clique «Rear wheel» na faixa e mude de novo: só a de trás muda. Arraste a bolinha azul da cota
   «Wheelbase»: a roda da frente anda.
6. Clique «Open in Inspector»: o Inspector mostra a junta, com o botão «Open in Joint Editor». Ctrl+Z desfaz o último passo.
7. Também: «Joint ▾» e «Template ▾» abrem as listas com miniaturas; o menu File tem «Joint Editor…».

Deu errado se: soltar não cria nada, a forma não acende, o carro nasce com uma roda presa no lugar errado, mudar «Both wheels» não muda as
duas, ou Ctrl+Z não desfaz.

## 7. Prompt de continuação (cole na janela nova)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-JointEditor && pwd && git branch --show-current
```
Confirme `line/JointEditor`. Leia, nesta ordem: `docs/JointEditor/01_plano.md`, a memória
`project-memory/project_joint_editor_version_b_2026_11_12.md`, e os handoffs `docs/JointEditor/handoffs/HANDOFF_CONTINUACAO_W2_2026-11-12.md`
e `HANDOFF_CONTINUACAO_W3_2026-11-12.md`. W1–W3 estão feitas (ponte curada, Inspector pelo papel, aba Joint Editor versão B). Comece a
**W4** (plano §W4, «Make the parts»). O fecho da linha = corte de LOC (§4.1 do handoff W3, medir antes de escolher) e as mutações (§2).
Inner loop = `cargo-check-narrow.sh`; testes por nome; sem push.
