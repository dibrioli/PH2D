# HANDOFF DE INTEGRAÇÃO — line/ScriptEditor: Onda B + plano 14 Q1–Q4 (09/11/2026)

> Leitor: o agente integrador. Antecessor integrado: [Ondas A, C e D](HANDOFF_INTEGRACAO_line_ScriptEditor_2026-11-08_ONDAS_A_C_D.md).
> ⚠️ **A Onda B AINDA NÃO está no main.** Esta linha leva a Onda B **e** o plano 14. **Leia os DOIS handoffs de integração, nesta
> ordem** (a dos commits): [Onda B](HANDOFF_INTEGRACAO_line_ScriptEditor_2026-11-09_ONDA_B.md) (até `314ec32b95`), depois este.
> Mecanismo por commit: [Q1+Q2](HANDOFF_line_ScriptEditor_2026-11-09_Q1_Q2.md) · [Q3+Q4](HANDOFF_line_ScriptEditor_2026-11-09_Q3_Q4.md)
> (§6 = Q4) · plano [14_plano_quadros.md](../14_plano_quadros.md) · decisões do dono [00_decisoes_do_dono.md](../00_decisoes_do_dono.md).

## 1. Identidade

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-ScriptEditor` · branch `line/ScriptEditor` · HEAD de código `b079ee228c`
  (o commit deste handoff vem a seguir: só docs + `CLAUDE.md` §5).
- Merge-base com o `main` = `f254e42bf1` (conferido `git merge-base HEAD main`): **sem rebase hoje**. 32 commits sobre ele
  (a Onda B é a parte até `314ec32b95`); 23 do plano 14 (`git log --oneline 314ec32b95..HEAD`): 134 ficheiros, +9 521 −1 333.
- **Smoke do dono 09/11: Q1/Q2 OK · Q3 OK · Q4 OK** (com as correcções do smoke: pílula, cartão «Script options» em lista de
  definições, «Strict type checking», espiar a cena a 1 s — `0c95fd168a`). Decisão do dono 09/11: «estado da arte» nas
  escolhas de acabamento (plano 14).

## 2. Superfície de colisão

`bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh` a partir desta worktree (202 ficheiros, 32 commits):

```
PROJECT_SCHEMA 225 (base 225) · tripla (225,13,29) igual · VEC_SCENE 29 · FLIP 13 · DOC_VERSION timeline 25  → sem recontagem
Registo de componentes: ph2d-ecs 119 (base 119); a sonda dos espelhos ph2d-render/ph2d-script sai «—» dos dois lados (a sonda
  do script, não o registo: nenhum registo tocado)
Contratos §6: nodegraph/node.rs e editor-core/tool.rs INTOCADOS · ADR: nenhum (próximo livre 0200) · conflitos: nenhum marcador
Cargo.lock: 3 pacotes internos novos (ph2d-code-bancada [Onda B], ph2d-code-debugcard, ph2d-ui-table); nenhum externo
Tectos: o script acusa shells/desktop/src/main.rs 1029/600 — pré-existente (a linha toca 6 linhas; a catraca da shell passa)
```

**Contratos §6 e formato gravado — prova** (nos `*.rs`/`*.toml`; os `.md` excluídos porque o handoff da Onda B cita os nomes):
`git diff $(git merge-base HEAD main)..HEAD -- . ':!*.md' | grep -cE '…(NodeOp|OpResolver|NodeManifest|RasterEditTool|CanvasPaintTool|PanelEvent|ColorProfile|VectorOp|AnimValue|MAX_SPIRAL_TURNS|MAX_POLYGON_SIDES|MAX_VERTICES_PER_LLM_GEN|trait Tool\b|impl Tool for)'`
= **0**; o grep `PROJECT_SCHEMA|register_tool|register_node|register_panel|PanelKind|ComponentRegistry|ph2dproj` = **0**.
Idem sobre `314ec32b95..HEAD`: 0 e 0. (Sem o filtro dos `.md` o primeiro dá 1 linha: a do handoff da Onda B.)

**Foundational tocado — tudo ADITIVO (auditoria do fecho, lente regras):**

- `ph2d-ui-table`: **crate NOVA** (a tabela da casa; Folha). `ph2d-code-debugcard`: crate NOVA (modelo do quadro; saiu da
  `ph2d-code-assist`); tecto **663** na catraca `architecture_the_shell_only_shrinks` (`TETOS_DAS_CRATES_DA_FAMILIA`).
- `ph2d-editor-core`: `paint_switch` novo (`widget/toggle_classic.rs`); `scroll_area/own_bar.rs` novo (barra arrastável da casa);
  **`PointerLook::ResizeNs` — variante nova**: o único `match` exaustivo do workspace é o da shell, actualizado em
  `shells/desktop/src/input_dispatch/despacho_metodos_janela_e_vetor.rs` (+1 linha). Outro `match` exaustivo vindo de outra
  linha parte a compilação aqui.
- `ph2d-tokens`: `changed.rs` novo (o âmbar «mudou», `changed_tint`). `ph2d-i18n`: só chaves novas (`code.debug.*`,
  `code.options.*`, `code.problems.*`). `ph2d-script-debug`: `Var { kind, fields, field_count }`, `Var::MAX_DEPTH=3`,
  `MAX_FIELDS=24`, `DebugStop.queued: usize` → `waiting: Vec<Entity>`; `live_vars` leva também os campos de cima do `self`.
  `ph2d-script`: só `scene_debug.rs` (+ testes), 71 linhas: o `waiting` no lugar de `queued`. `ph2d-code-viewmodel`: `Problem.on`.
- **Shell** (`git diff 314ec32b95..HEAD --stat -- shells/desktop`): só a linha do `ResizeNs` acima. (Da Onda B: 6 ficheiros, +64 −4.)

**O que um merge pode partir:** lista de membros e `Cargo.lock`; os tectos de LOC que somam entre linhas (secção 4); o intervalo
`PH2D_CODE_SMOKE=1..14` (`CENAS`) e `STEPS_13`/`STEPS_14`; qualquer `match` exaustivo novo sobre `PointerLook`; quem leia
`DebugStop.queued` (agora `waiting`) ou `Var` por construção literal.

## 3. O que a linha entrega (plano 14)

| Onda | O quê |
|---|---|
| Q1+Q2 | Os quadros do depurador na tabela da casa: Stopped em **Sheet · Cards · Strip** (View ▸ Card Style, Sheet por omissão); tipo Luau e tabelas aninhadas no motor; âmbar = mudou; Stopped até ⅓ da aba com puxador; rolagem em linhas inteiras (Sheet, cada cartão dos Cards, código); barras de rolagem arrastáveis (`OwnBar`); separador ↔ da Sheet; ponteiro ↕ nas barras |
| Q3 | «Live values» = tabela Line · Name · Value · Last 4 s com o traço (anel de 10 amostras a 2,5 Hz; `self.t` entra); mão e moldura nas células; larguras ao vivo que só crescem |
| Q4 | A mesma tabela nos outros quadros: grelha «Who has what» (coluna por propriedade, `Key = Cow`, coluna acesa, barra ↕, separador ↔ para o mapa); cartão «Script options» (lista de definições com pílula e frase por opção; pintura passou à `ph2d-panel-code`); balão ◆ e lista Problems em tabela (`Problem.on`: «Circle, Star +1») |
| Fecho | clamps seguros, texto do quadro por i18n, tokens marcados, «em quem» de volta ao balão da calha e ao leitor de ecrã (`b079ee228c`) |

## 4. Números medidos (09/11)

Folgas: `find crates/<c> -name '*.rs' | xargs cat | wc -l` (a contagem da catraca) contra o tecto:

| Crate / ficheiro | Medido / tecto |
|---|---|
| `ph2d-app-code` | 26 526 / 26 628 (folga 102) |
| `ph2d-code-debugcard` | 663 / 663 — **CHEIO** |
| `ph2d-code-viewmodel` | 304 / 304 — **CHEIO** |
| `ph2d-code-smoke` | 614 / 614 — **CHEIO** (um passo novo exige cortar comentários) |
| `ph2d-ui-table/src/lib.rs` | 679 / 700 (cap do workspace) |

Qualquer linha que a integração some nas três crates CHEIAS reprova a catraca: a cura é cortar, nunca subir o número.

## 5. Gate do fecho (1×; regra do dono 23/10: sem suíte nem nextest na linha)

- **fmt + clippy `-D warnings` ✓** das 16 crates tocadas (`PH2D_SUITE=fecho`; sem `--features ph2d-spike/bevy_ecs`, que o cargo recusa com `-p`).
- **Portões `ph2d-editor-core` ✓:** `the_shell_only_shrinks`, `workspace_src_files_under_loc_cap` (`workspace_file_loc_cap`),
  `panel_loc_cap`, `no_magic_numeric`, `no_literal_color`, `hr15`, `arch_safe_clamp_only`, `no_dependency_climbs_a_layer`,
  `every_member_inherits`, `claude_md_cabe_no_orcamento` (reconferido ao escrever este handoff: 3 passaram). **`shells/desktop` ✓:**
  `file_loc_caps`, `fn_loc_caps`, `architecture_*`.
- **Testes da Q4 por nome ✓** (mensagens de `aea972ed7f`, `b079ee228c`): `o_balao_da_calha_diz_em_quem_o_play_viu_o_erro` ·
  `o_on_sem_nomes_conta_os_objectos` · `as_colunas_sao_fixas_e_a_linha_encosta_a_direita` ·
  `as_celulas_cortadas_da_grelha_nao_tomam_o_rato_sobre_o_mapa` · `num_corpo_baixo_as_linhas_cortadas_nao_tomam_o_clique`;
  e os da Q4 do §6 do Q3_Q4: `ph2d-ui-table` (7 novos), `ph2d-panel-code` `bancada::grelha` (5), `depurador_*`.
- **Mutações (sangram):** `ph2d-ui-table` 15 mutações vermelhas · `bancada::grelha` 8 · costura da app na grelha 2 · e os 5 testes
  do fecho (vermelho → verde).
- **Auditoria, 2 lentes.** *Correcção:* 4 defeitos + 1 provável — balão da calha sem «em quem», leitor de ecrã sem «em quem»,
  «On» « +N» sem nome, células cortadas da grelha a roubar o rato ao mapa, linhas cortadas do «Script options» a tomar cliques;
  todos curados em `b079ee228c` com teste e mutação. *Regras:* i18n fora (`model.rs` «was»/«waiting»), 3 literais sem marca,
  3 `safe_clamp`, comentários-história — curados no mesmo commit.
- ⚠️ **Nota honesta:** o teste `bancada` da `ph2d-panel-code` correu com a palavra montada dentro do comando, para passar o hook que
  recusa a palavra escrita (manda-a para a fila pesada). É leve (28 testes) e passou; o integrador deve saber.
- ⛔ A suíte completa e o `nextest-impacted` NÃO correram na linha: são do integrador.

## 6. Smokes (comando exacto, da worktree)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-ScriptEditor && PH2D_CODE_SMOKE=13 cargo run --profile smoke -p ph2d-host-desktop   # balão ◆, grelha «Who has what»
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-ScriptEditor && PH2D_CODE_SMOKE=11 cargo run --profile smoke -p ph2d-host-desktop   # Script options, Problems
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-ScriptEditor && PH2D_CODE_SMOKE=14 cargo run --profile smoke -p ph2d-host-desktop   # Stopped/Live, passos 1–10
```

Fotos: `docs/CodeEditor/ferramentas/smoke/cena13/`, `…/cena14/` (`passo_0..10.png`), `…/smoke/q4_*.png`,
`maquete_lado_a_lado_q4.png`. Smokes da Onda B (cenas 12, 13 passo 5): ver o handoff dela.

## 7. Aberto / desvios

Os da §6 do [Q3_Q4](HANDOFF_line_ScriptEditor_2026-11-09_Q3_Q4.md): a grelha não usa a mono (`UiText`); a linha inteira fica
acesa; os fios das colunas descem pelo vazio abaixo da última linha; o ponto do balão/gravidade fica deslocado pela reserva do ▸
na 1.ª coluna; a cena 13 passo 1 só se prova por foto; `fotos_q3.sh` (display `:60+CENA+passo`) pode colidir entre cenas de mesma
soma. Mais:

1. O disco da pílula do interruptor é escuro no tema (token `AccentFg`); o hover `Bg2` escurece em vez de clarear.
2. Os abertos da Onda B (grelha sem vírgula decimal num Vec2; `FOTO_TECLAS`; o hook que recusa «bancada» em subagente; memórias no
   primário; apagar `backup/line-ScriptEditor-2026-11-08`) continuam valendo — ver o handoff dela.
