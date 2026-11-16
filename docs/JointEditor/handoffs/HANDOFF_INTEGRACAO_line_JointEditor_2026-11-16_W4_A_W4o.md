# HANDOFF DE INTEGRAÇÃO — line/JointEditor, W4h–W4o (16/11/2026)

Leitor: o agente integrador. Anterior (já no main): [W1–W3f](HANDOFF_INTEGRACAO_line_JointEditor_2026-11-13_W1_A_W3.md).
Mecanismo de cada wave, com a tabela de commits: [W4j (W4h–W4j)](HANDOFF_CONTINUACAO_W4j_2026-11-15.md) · [W4l](HANDOFF_CONTINUACAO_W4l_2026-11-15.md) ·
[W4m–W4n](HANDOFF_CONTINUACAO_W4m_W4n_2026-11-15.md) · [números medidos](../02_gabaritos_medidos.md) · [plano](../01_plano.md).

## 1. Identidade

Branch `line/JointEditor` · HEAD `e6fa4fc28a` (mais o commit deste handoff) · merge-base com o main `7b3dd983f7` (conferido) · 33 commits · 123 arquivos
(`git diff --shortstat 7b3dd983f7..HEAD`: +4 746 / −673; ~40 são fotos). **Smoke do dono OK (16/11, depois do tecto 128). O dono mandou integrar.** Worktree `Worktrees/line-JointEditor`.

## 2. O que a linha entrega

- **W4h–W4j** — a montagem é a PASTA das peças, cada junta mora no corpo A; olho/cadeado da pasta valem para o conteúdo; ponte da família Pin; «Custom» por família; lista Template em grelha de 3; alças do motor no desenho; «Pin to the scene» abre a montagem.
- **W4k–W4l** — campos do painel = cartão do Inspector (escrever, número vivo, arrasto da letra); seta do motor em espiral sem tecto; «Make the parts» (peças de fábrica reais, montadas, no meio da vista); «Planks».
- **W4m–W4n** — números da junta no painel de criação; «Planks» numa montagem JÁ feita refaz as cópias; emenda de `one_chain` = mola criticamente amortecida (a ponte já não catapulta a caixa).
- **16/11 (smoke do dono reprovado, curado)**:
  - `one_chain` em trechos (`59b4765008`): uma árvore custa ~n³ por passo (64: 5,5 ms · 128: 32 · 256: 302 · 384: 1 366; 1 024 = 29 s no 1.º passo; 256 numa árvore dá NaN). Com `SEGMENT_LINKS=32` (conta ELOS por árvore desde `6bf7c27b4a`) o passo do app é 64: 3,6 ms · 128: 7,5 · 192: 11,8 · 224: 13,0 · 256: 15,8 · 512: 32,4. `MAX_ROW_PARTS=128` (`e6fa4fc28a`; era 256; o 2.º smoke do dono 16/11 travou em Play com 256: tique 15,8 ms + desenho > 16,7 ms ⇒ o `FixedStep` paga até 8 tiques por quadro — quadro de 105 ms, ~10 fps, sim com pico 96 ms; com 128 o quadro fica na base, 34–37 ms no Xvfb). Tabela no doc 02. ≤64 tábuas bit-idêntico ao anterior (hash do gate). K=8/16 recusados (ponte de 64 tábuas colapsa). Guarda contra ângulo infinito no laço de `seed_chains`. Tabelas em `02_gabaritos_medidos.md` (inclui a correção «1,288» do W4m–W4n §3.3).
  - Caixas de número usam o motor do Inspector (`3758c20601`): `WidgetStore` privado com UM campo NumberInput (`ph2d-joint-editor-doc/src/typing.rs`, `FieldBox`; `focus_number_field`, `dispatch_pointer_with_text`, `dispatch_key`, `dispatch_text_input`, `blur_focus`); pintura por `card_field::paint_field_value` (cauda extraída de `paint_field`, que ficou byte-idêntica por auditoria). Fotos `fotos/w4o_texto/`, `fotos/w4o_travou/`.
  - Gates N1/N6 (`b00411b318`): as duas mutações passaram a sangrar (N1 esq. 10 / dir. 7; N6 asserta a posição).
  - **W4o** (`00c6b37a0d`): o cadeado de pasta COMUM (Empty) tranca o que está dentro. `ObjectType::is_folder` (Empty+Assembly); `ph2d_app_components::folder_lock::sync` corre a cada quadro a partir de `fase_timeline_drain.rs` (UMA linha na shell); marcador de não-guardado `LockCoversChildren`; a montagem fica FORA do sync (`6bf7c27b4a`). Gate `locking_a_common_folder_locks_what_is_inside` (mutação vermelha).
- **Corte de LOC**: crates novas `ph2d-joint-factory` (`joint_factory`, `joint_rows`, `stage`, testes de máquina) e `ph2d-joint-editor-apply` (`apply.rs`); `ph2d-joint-editor-doc` (estado da aba) saiu na W4n.
- **Auditoria de 2 lentes** (`6bf7c27b4a`): sair da caixa sem escrever regravava o valor arredondado (curado, teste `entrar_e_sair_da_caixa_sem_escrever_nao_grava`); o trecho contava profundidade (agora elos); o sync podia tirar o marcador da montagem (curado).

## 3. Foundational / compartilhado tocado (aditivo)

- `ph2d-physics`: `world/chains.rs` (`pub const SEGMENT_LINKS`, `pub fn set_chain_segment_links`), `lib.rs`, `tests/it/one_chain.rs`.
- `ph2d-physics-ecs`: `assembly.rs`, `bridge/anchors.rs`, `joint_pose.rs` (novo), `ref_remap.rs`, `lib.rs`.
- `ph2d-ecs`: `transform.rs` (SÓ doc-comment), `lib.rs`.
- `ph2d-editor-core`: `widget/card_field.rs` (`paint_field_value`, extração aditiva), `hier_types.rs` (`ObjectType::is_folder`), `doc_surface.rs`, `screens/hero/document_tabs.rs`, gate `architecture_the_shell_only_shrinks.rs` (entradas/tetos das crates novas).
- `ph2d-app-components` (`folder_lock{,_tests}.rs`, `lib.rs`), `ph2d-app-physics` (`joint_*`, overlay, testes da cena), `ph2d-physics-smoke-scenes`, `ph2d-joint-templates` (`catalog/custom.rs`), `ph2d-panel-inspector` (joint/physics rows, ids, 2 gates), `ph2d-i18n` (`joint_editor_tab.rs`), `ph2d-entity-visibility` (`off_canvas.rs`).
- Shell (`shells/desktop`): `fase_timeline_drain.rs` (+1 linha, `folder_lock::sync`), `fase_snapshots_publish.rs`, `fase_source_strategy_and_joint_pivot.rs`, `sim_extract.rs`.
- `Cargo.lock`: +3 internos (`ph2d-joint-editor-apply`, `ph2d-joint-editor-doc`, `ph2d-joint-factory`); nenhum externo.

## 4. Superfície de colisão (conferida por `git diff --name-only 7b3dd983f7..HEAD`, 16/11)

- **Contratos §6 INTOCADOS:** nenhum caminho de `node.rs`, `tool.rs`, `ph2d-nodegraph`, `ph2d-vector*`/`ph2d-vec-*` no diff (`git diff --name-only 7b3dd983f7..HEAD | grep -E "node\.rs|tool\.rs|nodegraph|vector|vec-"` vazio).
- **`PROJECT_SCHEMA` NÃO mudou:** `project_schema.rs` fora do diff; vale 242 no base e no HEAD (`git show 7b3dd983f7:shells/desktop/src/project_schema.rs` linha 490 = `242`). Marcador `LockCoversChildren` é de não-guardado, sem formato gravado. Mesmo assim reconte no ponto combinado: `python3 scripts/schema-recount.py`.
- Registo de componentes: +0 · ADR: +0 · tokens: +0 · cenas de física: +0 · routers de smoke: +0 (`PH2D_JOINT_EDITOR_SMOKE` foi de 1..9 para **1..14**).
- Crates **+3** (internas): `ph2d-joint-editor-doc`, `ph2d-joint-factory`, `ph2d-joint-editor-apply`. `ph2d-app-registry-init` NÃO mudou (crates de domínio, sem família de app).

## 5. Contadores como DELTA (LOC por `conta()` / teto)

`ph2d-joint-assembly` 1 561 / 1 661 (teto era 2 175, baixado) · `ph2d-joint-editor-doc` 2 123 / 2 223 · `ph2d-app-joint-editor` 3 883 / 3 983 · `ph2d-joint-factory` 1 552 / 1 652 · `ph2d-joint-editor-apply` 428 / 528. Tetos de LOC somam entre linhas: se outra linha tocou essas entradas na tabela, reconte, não escolha.

## 6. Um merge pode partir

- Tabela `TETOS_DAS_CRATES_APP` / `architecture_the_shell_only_shrinks.rs` (entradas das crates novas e tetos baixados).
- Código movido: testes por nome de outra linha que apontem para `ph2d-joint-assembly` (`joint_factory`, `joint_rows`, `stage`) ou `ph2d-app-joint-editor` (`apply`, estado da aba) mudaram de crate.
- `fase_timeline_drain.rs` (uma linha) e `ph2d-editor-core` `hier_types.rs`: `ObjectType::is_folder` é novo; um `match` exaustivo de outra linha sobre `ObjectType` não parte (método, não variante).
- `chains.rs`: outra linha em `one_chain`/correntes colide aqui; `SEGMENT_LINKS` e `MAX_ROW_PARTS` mudam o passo de pontes > 32 elos (≤ 64 tábuas bit-idêntico).

## 7. Prova de fecho (verificador, worktree, 16/11)

Verde: `cargo fmt` das 17 crates · clippy `--all-targets -D warnings` · censos 14/14 (138 testes) · gates por nome: `arch_safe_clamp_only`, `the_shell_only_shrinks`, `the_row_height_is_one_number`, `architecture_no_orphan_source_file`, `architecture_no_chip_without_steppers`, `architecture_claude_md_cabe_no_orcamento`, `every_physics_component_is_authorable`, `file_loc_caps`, `architecture_no_downcast_to_concrete_tool_in_shell` · testes de crate inteira (código movido): assembly 3, factory 24 (+1 ignorado), doc 11, apply 3, app 31, physics-smoke-scenes 108 (+31 ignorados) · `a_long_bridge_is_cut_into_trees_and_a_short_one_is_the_old_chain`, `planks_para_no_tecto_medido_da_fila` · binário `smoke`: 2.ª construção sem `Compiling`. Depois do fecho, só `e6fa4fc28a` (uma constante + doc + fotos): `rustfmt --check` e `planks_para_no_tecto_medido_da_fila` verdes; o binário `smoke` desta worktree está com `CARGO_PROFILE_SMOKE_STRIP=none` (do `perf`), sem efeito no código.
**NÃO corrido** (regra do dono, só na integração): `nextest-impacted` e a suíte completa. Mutações: N1/N6 e a do `folder_lock` sangram; H1–H6 e W4m/W4n já constavam.

## 8. Smokes (do Enio; o integrador não os corre)

Binário: `cargo build -p ph2d-host-desktop --profile smoke` na árvore integrada.

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-JointEditor && PH2D_PHYSICS_SMOKE=123 PH2D_JOINT_EDITOR_SMOKE=14 ./target/smoke/ph2d-host-desktop` (14 = «Bridge» por fazer; 1..13 como no handoff anterior e no do W4j).
2. Na aba «Joint Editor»: clicar numa caixa de número seleciona tudo; escrever e Enter grava; clicar fora sem escrever NÃO grava; duplo clique seleciona tudo. Em «Planks» escrever 9999 vira 128. «Make the parts» cria a ponte.
3. Deu errado: a caixa não aceita texto, sair dela muda o valor, a janela trava ao escrever um número grande em Planks.

## 9. ABERTO

1. ~~Smoke do dono desta jornada~~ — **OK em 16/11** (caixas de número, Planks 9999 → 128, Make + Play com a ponte de 128).
2. **Decisão do dono:** «Make» põe a ponte no MEIO da vista, por cima de objetos existentes (p.ex. Elevator); em Play colidem e as tábuas voam. Segue a decisão de 12/11 («no meio da vista»); mudar para «livre de colisão» é do dono.
3. O **congelamento do PC** do dono (16/11 ~15:31) NÃO reproduziu com o gesto exato em Xvfb (mesma GPU NVIDIA): CPU/memória planas, GPU < 26 %. Diário: só travo da thread principal do kwin + LogoutPrompt às 15:31:16, reboot 15:32; nesse minuto três outras linhas compilavam/corriam apps. Caminhos medidos que param o app: Planks enorme (n³, curado pelos trechos) e Planks 256 em Play (a espiral do passo fixo, curada pelo tecto 128). A espiral do `FixedStep` em si (`DEFAULT_MAX_SUBSTEPS = 8`; toda cena cujo tique + desenho passa de 16,7 ms) fica ABERTA, fora desta linha.
6. **Custo por objecto e por quadro do app inteiro, fora desta linha:** em Play com 128 tábuas o `perf` dá física 47 %, `ph2d_app_components::canais_componente` (+ `serde_json`, `BTreeMap<FieldKey>`) 16 %, alocação 10 %, Joint Editor < 1 % (doc 02). Candidato a linha própria se o dono quiser.
4. **Sem foto do gesto do cadeado de pasta** (arrastar na Hierarquia não está roteirizado); só o gate.
5. `agent-loop-profile.sh` e `rm -rf target/*/incremental`: a janela principal corre no fecho.
