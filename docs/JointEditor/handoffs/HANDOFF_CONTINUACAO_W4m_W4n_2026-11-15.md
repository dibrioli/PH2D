# HANDOFF de continuação — line/JointEditor, W4m–W4n feitas (15/11/2026)

Leitor: a próxima LLM da linha. Ramo `line/JointEditor`, worktree `Worktrees/line-JointEditor`, base = main `7b3dd983f7`, sem rebase,
sem integrar. Anterior: [HANDOFF_CONTINUACAO_W4l_2026-11-15.md](HANDOFF_CONTINUACAO_W4l_2026-11-15.md) (W4k–W4l). Plano: [01_plano.md](../01_plano.md).

## 1. Commits desta jornada (`git log --oneline 17ceb580e9..HEAD`)

| sha | o quê |
|---|---|
| `a462a0482b` | W4m: os números da junta no painel de CRIAÇÃO (pedido do dono 15/11). O rascunho mostra os MESMOS cartões da montagem feita (Limits/Motor/Breaks, Drive do carro) com números de fábrica e o mesmo «Applies to» («Plank» = todas as juntas da fila). Edições guardadas por regra, em ordem (`draft_edits`); `Edit::Make{edits}` escreve-as depois de `assemble` via `joint_with_edit`. `targets()` devolve linhas próprias (`rows()`: juntas da montagem, ou uma por regra do rascunho, bits 0); `write()` decide mundo × rascunho. «New» volta à fábrica. Testes `fazer::o_painel_de_criacao_muda_as_juntas_das_tabuas_antes_de_fazer`, `o_carro_por_fazer_mostra_a_suspensao_de_fabrica_e_o_drive`; unit `o_rascunho_mostra_a_de_fabrica_e_a_edicao_so_na_regra`. Texto i18n `joint_editor.empty_panel` mudou (3 linhas) |
| `7b2e729716` | W4n: «Planks» numa montagem JÁ feita refaz as cópias (plano §W4, 2.º ponto). `ph2d_joint_assembly::joint_rows::set_row_count`: copia a ÚLTIMA peça da fila (`deep_copy_subtree` do `ph2d_ecs` com o `ComponentRegistry` do APP — `frame()`/`apply()` agora levam `&ComponentRegistry`; a shell passa `component_registry` em `fase_snapshots_publish` `document_families`), remove as juntas filhas copiadas, põe cada cópia um passo à frente (passo da cena; do gabarito se há uma só peça), nome pelo sufixo da última («Plank 9 (1)»), `SiblingOrder` logo após a última; menos = remove do fim e despawn; o seguidor `ToLast` (poste direito) anda; juntas novas (e a religada no fim) herdam os números da irmã da mesma regra. `Edit::Rows`; toast «Made N copies of the last one in the row.». Teste `fazer::planks_numa_ponte_feita_refaz_as_copias` (1→5→12→3) |
| `0de2ff8783` | refactor: crate nova `ph2d-joint-editor-doc` (doc, doc_values, draft, input, grab, model, stage, doc_tests, geo; `format` + `MOTOR_PREVIEW_S`) sai de `ph2d-app-joint-editor`; o app reexporta doc/model/stage. Contagem: app 4 042 (tecto baixado a 4 142), doc 2 153 (tecto 2 253, entrada nova em `TETOS_DAS_CRATES_DA_FAMILIA`). Crates inteiras (código movido): 15 + 26 = 41 testes verdes |
| `8965ea181c` | test: «Applies to: Rear wheel» no carro por fazer muda só a de trás (a mutação M2 passou a sangrar, medido) |
| `51051dde4f` | fix(physics): a EMENDA de `one_chain` (única junta de impulso entre as duas árvores reduzidas, sob a caixa entre as tábuas 4 e 5; `crates/ph2d-physics/src/world/chains.rs`) é mola criticamente amortecida a 15 Hz (`SEAM_SOFTNESS`). Mecanismo medido pelo investigador: a ponte nunca cedeu; a emenda com rigidez padrão do rapier devolvia o erro de impacto de 8–15 cm num substep → caixa atirada a 2,0 m, 14/18 seguravam por sorte. Tabela: 240 Hz 1,04 m · 60 Hz 0,30 m · 15 Hz amortec. 1: **0,028 m, 18/18** · emenda num poste 1,92 m 11/18 · todas as juntas a 60 Hz moveram a porta (REJEITADO). Gate novo `the_bridge_catches_a_dropped_crate_without_throwing_it` (`ph2d-app-physics`, 9 quedas ±4 mm): vermelho antes, verde depois. `joint_factory::centre_at`: comentário atualizado (a ordem já não importa; código igual) — fecha o §2.3 do handoff W4l |

Decisão do dono (15/11, §2.4 do W4l): o cadeado de uma pasta COMUM tranca tudo o que está dentro
(memória `project_folder_lock_locks_its_contents_2026_11_15`). **Ainda não implementada.**

Fotos (gesto real, Xvfb): `docs/JointEditor/fotos/w4m/w4m_1_escrevendo.png`, `w4m_2_feita_plank.png`, `w4m_3_doze_tabuas.png`,
`w4m_4_play_inicio.png` (esta última ainda por rastrear: `??` no git). Ferramenta: `docs/JointEditor/ferramentas/fotos_gesto.sh`.

## 2. Provas

- Verdes: `four_named_machines`, `ph2d-physics` `one_chain` (5) + `chain` (12), `ph2d-app-physics` `joint` (181),
  `the_bridge_holds_itself_and_a_load`, `the_bridge_catches_a_dropped_crate_without_throwing_it`; crates inteiras do corte (41).
- Mutações (agente `mutacao`): W4m M1, M3, M4 sangram; M2 sangrou depois de `8965ea181c`. W4n N2–N5 sangram.
  **N1 SOBREVIVEU** (despawn das juntas filhas copiadas: a última peça da fila não tem junta própria nos gabaritos atuais — falta teste com
  junta de usuário sob a última tábua). **N6 SOBREVIVEU** (passo da cena × do gabarito: falta teste com a 2.ª tábua movida à mão).
  W4h/W4i H1–H6 sangram todas (a lista do §4.4 do W4j está cumprida).
- Binário de smoke construído (2.ª construção: `Finished smoke profile in 0.43s`, zero `Compiling`).
  **Smoke do dono: entregue, sem resposta** (W4m painel de criação com números; W4n «Planks» na ponte feita).

## 3. ABERTO (por ordem)

1. **W4o — o cadeado de pasta COMUM tranca o conteúdo.** Generalizar `locked_by_a_folder` (hoje só a montagem). Gate irmão de
   `locking_the_assembly_locks_the_whole_machine`.
2. **Gates para N1 e N6** (acima) — depois rodar de novo as duas mutações com `mutacao`.
3. **Corrigir o doc 02, linha «A ponte sob carga»**: «caixa 1,288» era uma caixa atirada que voltou a cair; trocar pelos números novos
   (0,028 m, 18/18) com a tabela do `51051dde4f` ao lado.
4. **LOC:** `ph2d-joint-assembly` 2 801 contra tecto 2 175. Cura = MOVER `joint_factory.rs` + `joint_rows.rs` (+ testes de máquina
   `joint_assembly_tests.rs`/`joint_assembly_folder_tests.rs` se usarem só API pública) para crate de família nova (ex.: `ph2d-joint-factory`)
   com entrada no tecto. Conferir usuários: `ph2d-app-joint-editor` `apply.rs`, `ph2d-joint-editor-doc` `doc_values`/`draft` (`count_of`),
   `ph2d-physics-smoke-scenes` cena 123, reexports de `ph2d-app-physics`. Código movido ⇒ crates inteiras (motivo imprescindível).
5. **Fecho da linha (DIRETRIZ §1.5.9):** clippy/fmt 1× das crates tocadas (`ph2d-app-joint-editor`, `ph2d-joint-editor-doc`,
   `ph2d-joint-assembly`, `ph2d-physics`, `ph2d-app-physics`, `ph2d-i18n`, shell); censos; gates `the_row_height_is_one_number`,
   `every_physics_component_is_authorable`, `the_shell_only_shrinks` por nome; handoff de integração + linha do §5 do CLAUDE.md (smokes
   `PH2D_JOINT_EDITOR_SMOKE=1..14`, ≤ 700 bytes); `bash scripts/agent-loop-profile.sh`; `rm -rf target/*/incremental`; binário de smoke 2×.

## 4. Prompt de continuação (cole na janela nova)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-JointEditor && pwd && git branch --show-current
```
Confirme `line/JointEditor`. Leia este handoff (`docs/JointEditor/handoffs/HANDOFF_CONTINUACAO_W4m_W4n_2026-11-15.md`) e o anterior.
Comece o §3.1 (W4o, cadeado da pasta comum), depois §3.2 a §3.5. Inner loop = `cargo-check-narrow.sh`; testes por nome; foto de gesto
real com `docs/JointEditor/ferramentas/fotos_gesto.sh`; smoke ao dono em passos numerados; sem push, sem integrar.
