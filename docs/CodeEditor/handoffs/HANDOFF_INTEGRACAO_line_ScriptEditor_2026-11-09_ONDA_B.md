# HANDOFF DE INTEGRAÇÃO — line/ScriptEditor: a Bancada, Onda B (09/11/2026)

> Leitor: o agente integrador. Antecessor integrado: [Ondas A, C e D](HANDOFF_INTEGRACAO_line_ScriptEditor_2026-11-08_ONDAS_A_C_D.md).
> Mecanismo por commit: [continuação ONDA_B_FEITA](HANDOFF_CONTINUACAO_line_ScriptEditor_2026-11-09_ONDA_B_FEITA.md)
> (inclui as curas do smoke do dono). Plano: [13_plano_bancada.md](../13_plano_bancada.md) «Onda B — construída».
> Decisão do dono 12 (espiar a cena + mapa, ambos ligados): [00_decisoes_do_dono.md](../00_decisoes_do_dono.md).

## 1. Identidade

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-ScriptEditor` · branch `line/ScriptEditor` · HEAD `3a5d49589f`
  (o commit deste handoff vem a seguir, só docs + `CLAUDE.md` §5).
- Merge-base = `main` actual = `f254e42bf1` (conferido `git merge-base HEAD main`): **sem rebase hoje**.
- 8 commits sobre `f254e42bf1` (`git log --oneline f254e42bf1..HEAD`): `81ebfee256` (docs) · `e7778b6336` (a Bancada) ·
  `faed3f4e7a`/`95b33bc127` (fotos, handoff) · `7d0fb827f1`/`afa6ebdf9d`/`4dafb508a0` (curas do smoke do dono) ·
  `3a5d49589f` (achados da auditoria do fecho).
- **Smoke do dono: OK em 09/11** (cena 13 passos 1–5 e os valores ao vivo da cena 12). Com a B, o plano 13 está todo feito.

## 2. Superfície de colisão

```
Contratos §6 (nodegraph, tool*, vector, imageio): git diff f254e42bf1..HEAD --name-only nessas pastas = VAZIO; grep do diff
  por NodeOp|OpResolver|NodeManifest|RasterEditTool|CanvasPaintTool|PanelEvent|VectorOp|AnimValue|ColorProfile = 0 linhas
PROJECT_SCHEMA: git diff f254e42bf1..HEAD | grep -c PROJECT_SCHEMA = 0 → sem recontagem; nenhum campo serializado novo
Cargo.lock: +1 crate interna (ph2d-code-bancada), nenhum pacote externo → regenera-se
ADR: nenhum
```

**Crate nova:** `ph2d-code-bancada` (quem corre o script, o acto da célula, seguir o objecto; `names.rs` movido de
`ph2d-app-code/src/scene_names.rs`). Depende só de `ph2d-editor-core`, `ph2d-script`, `ph2d-i18n`, `ph2d-ecs`, `ph2d-code-*`
(não sobe camada). Linha em `TETOS_DAS_CRATES_DA_FAMILIA`: `1_028`.

**Código MOVIDO entre crates** (nenhum portão cita os caminhos velhos — verificado): `scene_names.rs` → `ph2d-code-bancada`;
`ph2d-app-code/src/cards.rs` → `ph2d-panel-code/src/cards.rs`; `FieldKey`/`field_key`/`printable` → `ph2d-panel-code/src/field.rs`.

**Foundational tocado — porque é aditivo:**
- `ph2d-editor-core`: `DocumentSurface::yields_to_scene` (método com default `false`; só a aba de código o sobrescreve) e
  `Documents::paints_document_ground` (com `false` dá a condição antiga de `hero/paint.rs`). Nenhum consumidor muda.
- `ph2d-script`: `decls_shown` (pub novo); `debug_end_run` depois de cada `tick`/`hear`/`process`/retoma; `reloaded_ok`.
- `ph2d-script-debug`: **muda de sentido para os dois leitores do workspace** (`ph2d-code-assist/src/debug_view.rs`, o
  `×N`; `ph2d-app-code/src/debug.rs`, os fantasmas): `LineLive.hits` = máximo de UMA corrida (antes: a soma do quadro,
  que dava `×2` nos quadros com dois passos fixos); `end_frame` funde por linha; `forget_all`/`forget_lines` limpam os
  valores. Nenhum outro leitor (grep `live_values|LineLive`).
- `ph2d-vec-render`: `draw_peek_outline`, `outline_rings` (novos). `ph2d-i18n`: chaves `code.bancada.*`, `code.cmd.peek*`,
  `code.cmd.map*`, `code.menu.view` (só inglês).
- `ph2d-app-components`: `script_inspector_tests.rs` (a célula = a edição do Inspector) e a cena 13 em `code_smoke.rs`.
- **Shell** (`git diff f254e42bf1 --stat -- shells/desktop`: 5 ficheiros, +63 −4): `bancada_scene.rs` (novo: o espiar e o
  mapa; `serve_code_smoke_hand` = a mão do roteiro de smoke, inerte sem `PH2D_CODE_SMOKE_PASSO`), `main.rs` (+1 `mod`,
  +1 chamada antes do `post_frame_undo`), `fase_pointer_subjects.rs` (+1), `fase_vector_overlays.rs` (o contorno do
  espiar), `Cargo.toml` (dep `ph2d-code-bancada`). Nenhum campo novo na `App`.

**O que um merge pode partir:** a lista de membros e o `Cargo.lock`; os tectos de LOC que somam entre linhas; o intervalo
`PH2D_CODE_SMOKE=1..13` (`CENAS = 13`); `STEPS_13` (5 passos).

## 3. O que a linha entrega

| Item | O quê |
|---|---|
| In this script | Properties · Functions · Hears com `ln N`; marcas ◆/⚡ na margem com pontos na cor de quem difere do padrão; balão «em quem» (pairar abre, clicar prende, `Esc` fecha, 300 ms de espera no caminho do rato) |
| Who has what | Grelha objectos × propriedades; a célula edita pela MESMA `InspectorComponentEdit` do Inspector; `Ctrl+Z` com a grelha em foco desfaz o projecto |
| Where | Mapa com o contorno de cada objecto que corre o script; frase do pé fixa |
| Espiar a cena | Rato parado num nome de objecto cede a área à cena com o contorno dele; menu View (`Peek at Scene`, `Scene Map`), ambos ligados |
| Valores ao vivo (curas) | `×N` de uma corrida; a linha guarda o último valor; amostra a 2,5 Hz (`Duration::Slow`); rebobinar e recarregar limpam |

## 4. Números medidos

| O quê | Medido (09/11) |
|---|---|
| Gancho de linha, 50 objectos a 60 Hz, mínimo de 9 rodadas intercaladas (`sonda_gancho_de_linha`, smoke, load 5,6–9) | desarmado 0,1329 ms · passo-a-passo + locais 0,3219 ms = **+1,13 %** do quadro (critério ≤ 2 %; antes da mudança dos valores ao vivo: +1,13 %) |
| Ritmo dos valores ao vivo | antes: 60 Hz (cada quadro); agora 3 amostras em 60 quadros (`os_valores_ao_vivo_amostram_duas_vezes_e_meia_por_segundo`) |
| LOC (`wc -l` sobre `*.rs` da crate) contra o tecto | `ph2d-app-code` 26 595 / 26 628 · `ph2d-app-components` 80 756 / 80 869 · `ph2d-code-bancada` 926 / 1 028 · `ph2d-code-viewmodel` 302 / 304 · `ph2d-code-scenes` 660 / 666 · `ph2d-code-smoke` 565 / 614 · shell 188 038 / 189 041 |
| Binário `--profile smoke` 2× | a 2.ª: `Finished … in 0.21s`, zero `Compiling` |

## 5. Gate do fecho (1×, sobre o diff acumulado)

- **Testes por nome ✓:** `ph2d-script depurador_` 23 · `ph2d-app-code depurador` 10 · `onda_b` 2 · `os_valores_ao_vivo` 2 ·
  `ph2d-panel-code bancada` 19 · `ph2d-code-bancada` 7 (Onda B) · `ph2d-code-syntax ouvidos` 3 (Onda B).
- **Portões por nome ✓:** `the_shell_only_shrinks`, `workspace_src_files_under_loc_cap`, `file_loc_caps` (shell, 4), `fn_loc_caps`
  (shell, 2), `architecture` (shell 3, editor-core 120), `arch_safe_clamp_only`, `no_magic_numeric`, `no_literal_color`,
  `hr15_no_hardcoded_ui_strings`, `the_app_does_not_grow_past_its_numbered_ceiling`.
- **Vermelho latente achado e curado:** `arch_safe_clamp_only` (o `.clamp` da largura dos nomes em `bancada/grid.rs`, desde
  `e7778b6336`) → `safe_clamp`.
- **Clippy** `--all-targets -D warnings` das 14 crates tocadas ✓ · `cargo fmt --all --check` ✓.
- **Mutações (sangram):** `+= n` no `end_run` («dois passos não são ×2») · `shown.clear()` no `end_frame` («a linha que não correu
  não some») · sem `shown.clear()` no `forget_all` («rebobinar limpa») · sem `forget_lines` no recarregar («recarregar limpa»).
- **Auditoria, 2 lentes (correcção; contratos/UI).** Curados: a retoma do depurador não fechava a corrida (somaria `×5` em vez de
  `×3`); `forget_all` não limpava os valores; o recarregar do script deixava valores do texto velho. Limpos: contratos, schema,
  HR-15 (hex), camadas, i18n com leitor. Não lidos pela lente de correcção: `popover.rs`, `marks.rs` (provados no smoke do dono).
- ⛔ **`nextest-impacted` e a suíte completa NÃO correram na linha** — regra do dono 23/10 (`CLAUDE.md` §2): são do integrador.

## 6. Smokes (comando exato)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-ScriptEditor
PH2D_CODE_SMOKE=13 cargo run -p ph2d-host-desktop --profile smoke   # a Bancada
PH2D_CODE_SMOKE=12 cargo run -p ph2d-host-desktop --profile smoke   # o depurador e os valores ao vivo
PH2D_CODE_SMOKE=13 PH2D_CODE_SMOKE_PASSO=5 PH2D_UNDO_LOG=1 cargo run -p ph2d-host-desktop --profile smoke   # conduzido até o Ctrl+Z
```

Fotos: `docs/CodeEditor/ferramentas/smoke/cena13/` (`passo_0..5.png`, `maquete_B_lado_a_lado.png`), `…/cena12/`.

## 7. Aberto

1. A grelha não aceita vírgula decimal num Vec2 (`"1,5, 2"` → nada); num Number aceita.
2. Observado na foto do passo 5: depois do `Ctrl+Z` o separador da Timeline passa de «Arrange» a «Keys».
3. `FOTO_TECLAS` (XTEST) não chega à janela do `kwin --virtual` — teclas reais na foto vão pelo roteiro (`Gesture::WindowUndo`).
4. O hook `.claude/hooks/prazo-do-agente.sh` recusa linhas de subagente com «bancada» (também travou a auditoria deste fecho):
   estreitar o padrão é decisão do processo.
5. Memórias `project_bancada_peek_and_map_both_on_2026_11_09.md` + linha em `reference_topic_owner_product_decisions.md` estão no
   checkout PRIMÁRIO (symlink), não nesta worktree: o integrador leva-as.
6. `backup/line-ScriptEditor-2026-11-08`: apagar quando esta integração entrar.
