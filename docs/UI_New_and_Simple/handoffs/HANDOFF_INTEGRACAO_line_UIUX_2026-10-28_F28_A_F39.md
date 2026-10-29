# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-28 — F28 a F39 (vocabulário do discreto · composto do balde · a lista com identidade · camadas do Vetor · filtros · memória da placa · referência por índice)

- `line/UIUX` → `main` por `--ff-only`, **só por ordem EXPLÍCITA do Enio**. Sem push, sem ship.
- Ponta `317205c334`; `main` = merge-base = `c3662cd8e4` (o main NÃO andou desde a F27) ⇒ fast-forward possível, **zero rebase**.
  Se o main andar antes da integração, reconte pela §2. Delta total (`main..HEAD`): **95 commits, 547 ficheiros, +24 050 −2 608**
  (crates 412 f. +20 106 −1 857 · docs 66 f. · `shells/desktop` **67 f. +741 −749: a shell ENCOLHEU**, tecto `the_shell_only_shrinks` respeitado).
- ⚠️ **SMOKES DO DONO PENDENTES: F37, F38 e F39.** O Enio mandou integrar ANTES de os fazer (28/10). **Integrar não é aprovar.**
  Aprovados por ele: F29, F30, F31, F32, F33, F34, F35, F36. F28: o handoff F28 registava o smoke por fazer e nenhum seguinte o repete — conferir com ele.
- A linha já editou a linha do UI/UX do `CLAUDE.md` §5.1 (smokes `6..41`): o integrador reescreve essa linha inteira (§7), não faz merge textual.

## 1 — O que entrou (um handoff de continuação por onda; o mecanismo e os números completos vivem lá)

Pasta: `docs/UI_New_and_Simple/handoffs/` (`HANDOFF_CONTINUACAO_line_UIUX_<data>_<onda>.md`) e planos `../F28_plano.md … F38_plano.md`.

| onda | o quê (números medidos) |
|---|---|
| F28 (24/10) | Um valor discreto preso ao vocabulário do campo: `ph2d_ecs::canal_palavra::prende` nas 3 fronteiras (funil do mundo, chave da timeline, escritor do vetor). `MISTURA`/`PILHA_MISTURA` 22→19 opções (buracos 4/12/13), `TEXTO_ALINHAMENTO`/`PILHA_DILATA_JUNCAO` 200→2. Mutação 9/9. Cena `AUTOKEY=34`. |
| F29 (25/10) | A tinta do balde nasce FILHA do caminho (`ShowBehindParent`; um z só no composto); booleana veste o operando que não corta (`doador.rs`; muda o estilo de TODO Subtract); o Corte ganha lâmina real (`on_press_blade`) e [Cortar][Descartar] na barra flutuante. Mutação 9/9. Cena `OBJECT_MODE=38`. Tintas antigas NÃO migradas. |
| F30 (25/10) | A lista FORA do Auto Key: `ItemId` em 10 itens (família LISTA=8); a Cena (`StableId::CENA`) e a mesa de som (`MesaDoc`) viram alvos da timeline (famílias MUNDO=9, MESA=10); corpo vivo obedece à curva (`retune_body`); `IkBendSide`, Rope Length; fio do caminho; arrastar várias linhas na Hierarquia. **`PROJECT_SCHEMA` 193→195.** Mutação 8/10→fechadas. Cenas `AUTOKEY=35`, `OBJECT_MODE=39`. |
| F31 (25/10) | Identidade das camadas do Vetor: `FieldKey` ganha `item`; pilha (`PaintEntry.id`/`FxEntry.id`), filtros e contornos por id. **`PROJECT_SCHEMA` 195→196, `DOC_VERSION` 23→24, `VEC_SCENE_SCHEMA` 23→24.** Mutação 13/13. Undo a tocar: 120 passos em ~7 s. Cena `AUTOKEY=36`. |
| F32 (26/10) | Timers/Tweens/máquina de estados por id (`TimerState.item`, `Tween.relogio`, `ObjectPose.filter_ids`…). **`PROJECT_SCHEMA` 196→197.** `.ph2dmacro` antigo migra (`item_id::migra_blob`). Mutação 24/24 (+1 sobrevivente curada). |
| F33 (26/10) | Filtros sobre efeitos: reamostrador partilhado (`fx_resample.rs`) deixou de comer quinas e fazer laços — Nuvem+Flag **0/11 → 11/11 quinas, 6 → 0 segmentos que recuam**. Mutação 4/4. Cena `OBJECT_MODE=40`. |
| F34 (26/10) | Zoom perto numa forma com filtro: o job do FX recorta-se à VISTA (antes `MAX_FX_SIDE` 8192 cortava do canto e a forma SUMIA). Recook **~234 ms → ≤ 1 ms**. Cena `OBJECT_MODE=41`. «Tela preta» do dono = ollama com 13,7/16 GB de VRAM, não defeito. |
| F35 (26/10) | Memória da placa: FX devolve parado 120 quadros (cena 41 **1337→1081 MiB**; pior caso **4169→1344**; posse do FX 2316→12 MiB). Placa cheia por outro app ⇒ aviso no título + toast, o Vello refaz o `Renderer` e volta sozinho (`ph2d_gpu::memoria`). Mutação M1–M7. |
| F36 (27/10) | O que a placa recusou ao NASCER renasce (átlas, texturas individuais, preview do Painter); arranque espera a placa (`GpuContext::new_esperando_memoria`, curou PANIC em `init.rs`); `PH2D_GPU_PEGADA`. Cena 1 com 282 MiB livres: 543 erros + 3 098 desenhos → **0**. Mutação 14/14. |
| F37 (27/10) | Cabe numa placa com ~950 MiB livres: átlas nasce 2048² e cresce no lugar; UM `vello::Renderer` por placa; blocos `MemoryUsage` (`PH2D_GPU_BLOCOS=desempenho` volta). `nvidia-smi`: cena 1 **831→377 MiB**, cena 41 **1343→507**. Mutação M1–M12. |
| F38 (27/10) | Textura recusada vive numa substituta 1×1 sem trabalho de placa. Radeon 610M: cena 1 819→**366** MiB, cena 41 1331→**494**. Smoke da placa cheia medido pelo agente: F37 132 `[gpu-erro]` falsos, F38 0. Mutação M13–M24. |
| **F39 (28/10)** | **Família «referência por índice que envelhece»** — ver §1.1. |

### 1.1 F39 (commits `3aff426743`, `e1c714512f`, `317205c334`)

1. **F32 §6.2** — a pose dos estados de UI misturava o meio da transição por ÍNDICE (`mix_stacks`): reordenar filtros entre Default/Hover trocava o tipo de um degrau a meio.
   Cura: `crates/ph2d-ui-state/src/transition.rs` `mix_filters` alinha pelo `ItemId` do filtro (cai na lei por índice se um lado não tem ids completos e únicos); id só do destino entra
   depois do vizinho de cima; tipo diferente = fade do neutro. Teste `reordenar_os_filtros_entre_estados_nao_troca_o_tipo_de_um_degrau` (`ph2d-ui-state/src/tests.rs:692`).
2. **F32 §6.3** — a chave animada em `from`/`to` de uma seta da StateMachine guardava o NÚMERO do estado; `RemoveState` reescrevia as setas mas não as chaves.
   Cura: `ListaDesc.aponta: &[(field_id, codigo)]` + `alvo()` (`ph2d-component-desc/src/lista.rs`; `catalog/listas.rs`: SETAS aponta os campos 4 e 6 para ESTADOS);
   `ph2d-app-components/src/canais_lista.rs` publica o `ItemId` do estado-alvo como f32 (exacto ≤ 2^24, senão não publica) e escreve de volta a posição de agora; item não achado ⇒ nada escrito.
   Teste `a_chave_de_uma_seta_segue_o_estado_depois_de_apagar_um_anterior` (`canais_lista_tests.rs:342`). A família LISTA não existe no main ⇒ **sem migração**.
3. Varredura da família: só estes dois campos guardam a posição de outro item; `initial` da StateMachine não é canal; F31 §6.2 já curado na F32; F30 §6.5–6.9 são OUTRO mecanismo (ficam abertos).
4. Mutação M1–M6 todas vermelhas (agente `mutacao`), árvore restaurada; o controlo `o_meio_da_transicao_leva_o_id_de_cada_filtro` também sangrou sob M2.
5. **F33 §6.2** (a cena 40 mostrava «No selection»): o produto estava certo (clique na Hierarquia acende o painel). Mecanismo: a selecção da caneta no último quadro da cena era esvaziada por
   `ObjectModes::enter_with` (`ph2d-app-vec/src/vector_mode.rs:231`) porque a última nuvem entra em Edit um quadro depois. Cura SÓ na cena: `vector_mode_filtro_efeito.rs` `seleciona` espera `edit.objects`
   assentar e usa `gizmo.replace_selection`. Fotos `img_F39/cena40_antes.png` · `cena40_depois.png`.

## 2 — Formato e contadores (DELTA — conte no código)

- **`PROJECT_SCHEMA` 193 → 197** (`shells/desktop/src/project_schema.rs:515`; tripla `(197, 13, 24)` em `project_schema_tests.rs:322`). Degraus: 193→194 `a509c21b8a` (mesa no ficheiro) ·
  194→195 `b21477c738` (`ItemId`) · 195→196 `3881e9c48b` (`VecFilter`) · 196→197 `9b8d9bb6ad` (`item_id_v196`, pose). Escada condensada num bloco só (`195..=197`).
  **Reconte com `python3 scripts/schema-recount.py <valor que o degrau passa a ter no main>`** — só se o main andar (corra DENTRO do conflito de rebase; o script não decide o valor: leia o `main` no ficheiro).
  Com o main parado, nada a recontar (verificado: main 193).
- **`TimelineDoc` `DOC_VERSION` 23 → 24** (`ph2d-timeline/src/doc.rs:148`, fixture `timeline_doc_v23.bin`, leitor `canal_chave_v23`). **`VEC_SCENE_SCHEMA_VERSION` 23 → 24** (`ph2d-vec-scene/src/schema.rs:93`).
- Cenas: `PH2D_OBJECT_MODE_SMOKE` `1|2|4|6..41` (próxima **42**) · `PH2D_AUTOKEY_SMOKE` **1..36** (cenas 34–36 vieram da F28/F30/F31; próxima **37**; despachante `autokey_smoke.rs:85-96`).
  Quem integrar outra linha com cena nova RECONTA, não escolhe.
- ADR: **nenhum novo** nesta rodada (`git diff main..HEAD --name-only | grep decisions/` vazio).
- Registo de componentes: o catálogo ganhou a família LISTA (`catalog/listas.rs`, `logic.rs`, `mod.rs`); **não recontei** os registos `108/109/109` — o integrador reconta pelos gates de registo.
- Deps novas: crate **`ph2d-item-id`** (folha) · `ph2d-ecs`/`ph2d-timeline`/`ph2d-timeline-persist`/`ph2d-tween`/`ph2d-ui-state`/`ph2d-vec-scene` → `ph2d-item-id` · `ph2d-timeline → ph2d-board-model` (`undo_tick`) ·
  `ph2d-audio` ganha `serde`+`postcard` · dev-dep `ph2d-vec-scene` na `ph2d-render`. `Cargo.lock`: só o pacote interno novo e arestas; nenhum pacote externo novo no diff.

## 3 — Foundational e contratos congelados (§6)

- Foundational tocado, todo **aditivo** (campos novos com `0`/vazio por omissão, leitura dos formatos antigos por ramo congelado `*_v194/v195/v196/v23`): `ph2d-ecs` (`item_id*`, `canal*`, `cena.rs`, `vec_filter*`, `stable_id` +`CENA`),
  `ph2d-timeline`, `ph2d-editor-core`, `ph2d-render`/`ph2d-gpu` (placa), `ph2d-component-desc`, `ph2d-audio`. Razão: identidade de item e canais (F27 §3 como fundamento).
- **Contratos congelados NÃO tocados** — provado por `git diff --name-only main..HEAD -- <caminho> | wc -l` = **0** em: `crates/ph2d-nodegraph` · `crates/ph2d-editor-core/src/tool.rs` ·
  `crates/ph2d-editor-core/tests/it/architecture_tool_contract_surface.rs` · `crates/ph2d-vector-doc`/`ph2d-vector*` · `crates/ph2d-vec-model`; nenhum ficheiro `*contract_surface*` aparece no diff.
  (`ph2d-editor-core` foi tocado, mas fora do contrato `Tool`: `*_edits.rs`, `undo_link.rs`, `aviso_da_placa.rs`, `screens/hero/inspector_model_*`.)

## 4 — O que um merge pode partir (consolidado)

- **Literais de struct** de outras linhas: `FieldKey { .. }` (+`item`), `PaintEntry`/`FxEntry` (+`id`), `TimerState`, `StateMachineRuntime`, `Tween`, `PathFollow`, `ObjectPose`, `InspectorTweenRow`, `FxImage` (+`texels`), `ListaDesc` (+`aponta`).
  `VecFilter.ops` é PRIVADO (use `ops()`/`ops_mut()`/`push`); `VecFilter::troca_ops` saiu → `troca_pares`; `PosicoesDosItens` é `(BTreeMap, BTreeSet<u64>)`.
- Átlas: todo átlas nasce por `TextureAtlas::with_filter` (branco na origem, `WHITE_TILE_UV`); proibido re-empacotar. Vello: só por `vello_partilhado::da_placa`. `insert_atlas_sprite_with_regrow` mantém fetcher MORTO (6 chamadores/4 famílias).
- Censo `every_recook_tells_vello_the_pixels_changed`: agulha agora `stack.run_from_com_origem(`.
- Estilo de TODO Subtract mudou (F29 §2.2); tintas do balde anteriores à F29 ficam raiz.
- `line/GameEngineFuncional` (F32 §3): toca `fase_sequences_tests.rs`/`inspector_timer_tests.rs` e move `timer_tick.rs` — reconciliar o literal `TimerState`.
- Colisão com `CLAUDE.md` §5.1 (linha UI/UX): ver §7. Rode `collision-surface.sh` pelo caminho absoluto contra as linhas vivas (`git worktree list`); a coluna `base:` é o merge-base.

## 5 — Prova de fecho (por onda; a da árvore combinada é do integrador)

- Cada onda: gates vistos VERMELHOS por mutação (agente `mutacao`; contagens na tabela do §1 e nos handoffs), árvore restaurada.
- Fecho da F39: `cargo fmt` + clippy limpos nas crates tocadas; testes por nome verdes; `nextest-impacted` (`PH2D_SUITE=onda`) **22 423 testes, 2 vermelhos em crates NÃO tocadas, sob load ~75–83**:
  `ph2d-timeline no_expression_allocates_no_link_frame` e `ph2d-tool-painter the_mask_stroke_cost_does_not_follow_the_canvas` — verdes sozinhos por nome.
  Ambos **já estão listados** em `docs/DevOps/FLAKES_DE_CARGA.md:52,54` (família de flakes de carga) ⇒ nada a promover.
- F38: 22 421 testes, 2 vermelhos (HR-15 da `ph2d-app-vec`, curado em `a567ffa870`; `a_cena_3_conduzida_passo_a_passo` da CodeEditor, carga ~47, verde sozinho).
- **A suíte COMPLETA (`PH2D_SUITE=fecho BASE=main`) NÃO correu sobre a ponta final** — é do integrador (ordem do dono 23/10).

## Portões do fecho

Sobre a ponta, 28/10: `fmt --check` ✓ · clippy `--workspace --all-targets -D warnings` ✓ (2m14s) ·
`check-standalone-optional` ✓ (11) · `check-workflow-packages` ✓ (32/410) · `file_loc_caps` ✓ (4, cobre
`the_shell_only_shrinks`) + `the_app_only_sheds_fields` ✓ (3) · `censos-da-arvore-combinada` ✓ (121/121).
`cargo machete` ✗ → curado no commit do fecho: a shell já não chamava `ph2d_panel_audio_mixer` (a ponte
foi para `ph2d-app-audio` na F30 §B2) ⇒ dep removida, a feature só liga o registo; `bevy_ecs` em
`ph2d-app-audio` é do `#[derive(Resource)]` ⇒ `ignored` nomeado (como `ph2d-script`). Suíte completa: do integrador.

```
✗ paralelismo 1.16/passo (≥1,5) · ✓ respostas/sessão 128 · ✗ test:check 266:117 = 2.3x (≤1,0)
✓ edições sem conferir 1.4% · ✓ contexto/passo 245 mil · ✓ contexto inicial 66 mil
```

## 6 — Smokes do dono — PENDENTES (F37, F38, F39)

Build (1×, uma `smoke`): `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` e o exemplo `ocupa_a_placa` (`-p ph2d-gpu --example ocupa_a_placa --profile smoke`).
Depois da integração, os mesmos comandos a partir do checkout principal (`cd /home/enio/Documentos/Projetos/PH2D`).

- **F37.** 1) Terminal 1: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && ./target/smoke/examples/ocupa_a_placa 950 120` — espere «seguro 120 s».
  2) Terminal 2: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=41 ./target/smoke/ph2d-host-desktop`.
  3) Certo: a forma verde aparece e a Hierarchy mostra «Bevel». 4) Errado: aviso de memória, «Path 0» ou forma ausente.
- **F38.** 1) `… ocupa_a_placa 400 60` — «seguro 60 s». 2) o mesmo comando da cena 41. 3) Certo: forma presente e NENHUMA linha `[gpu-erro] … recusou trabalho` no terminal, nem depois dos 60 s. 4) Errado: essa linha a repetir.
- **F39.** 1) `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=40 ./target/smoke/ph2d-host-desktop`.
  3) Certo: seis nuvens verdes, o Flag seleccionado, o Inspector com a pilha dele (foto `img_F39/cena40_depois.png`). 4) Errado: «No selection» (foto `cena40_antes.png`).

## 7 — Para o integrador: `CLAUDE.md` §5.1 (a linha NÃO edita o CLAUDE.md na entrega)

- UI/UX: trocar o link para `Último: [F28–F39, no main 28/10](docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-28_F28_A_F39.md)` e as faixas para
  `PH2D_OBJECT_MODE_SMOKE=1|2|4|6..41` (já está na ponta da linha) · **`PH2D_AUTOKEY_SMOKE=1..36`** (a ponta ainda diz `1..33`; máximo no código = 36). Entrada ≤ 700 bytes (gate `architecture_claude_md_cabe_no_orcamento`).

## 8 — ABERTO, por ordem

1. **Smokes F37, F38, F39 do dono** (§6) — nenhuma das três está aprovada.
2. **F33 §6.3** — Zig Zag/Sketch no vale côncavo: decisão do DONO, com o oráculo a correr.
3. **F30 §6.5–6.9** (decisões e dívidas): `View::Toggle` lido `(off,on)` em `canal_componente.rs` (inverte chaves gravadas — decisão do dono) · sem losango na mesa (fader/pan/tom/corte/M/S) e no painel do esqueleto,
   matriz de camadas da física não animável · scrub re-simula sem reler a curva · mundo e mesa fora do undo · dívida de cor crua nos overlays vetoriais (`cor_crua_tests.rs`: `connector` 5, `marquee` 4, `cut_line` 2, `gradient` 2, `guides` 2, `symmetry_overlay` 1).
4. Do F38 §5: `readback_mip` do átlas recusado lê a substituta (só testes) — medir numa AMD dedicada. De outras famílias (não fazer aqui): tela 4K com rabisco automático e pool de buffers do lote no `StampPass` (Painter);
   fetcher morto de `insert_atlas_sprite_with_regrow`.
5. Herdados de F18–F26/F27 (não re-verificados): F25 §6 itens 2–9 e 11.
6. Registos de componentes não recontados (§2); suíte completa e gates da árvore combinada por correr (§5).
7. Integrar e push só por ordem EXPLÍCITA do Enio.
