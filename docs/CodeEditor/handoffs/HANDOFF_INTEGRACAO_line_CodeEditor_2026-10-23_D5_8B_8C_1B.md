# HANDOFF DE INTEGRAÇÃO — line/CodeEditor: d5, 8b, 8c e 1b (23/10/2026)

> Leitor: o agente integrador. **Não integrado, não enviado.** Antecessor integrado:
> [Onda 2 até ao 8d](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-22_ONDA2_ATE_8D.md). Continuações desta rodada:
> [d5 e 8b-assets](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-23_D5_E_8B_ASSETS.md) ·
> [8b-2, 8c, cena 9](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-23_8B_8C_CENA9.md). Planos:
> [07](../07_arrastar_para_o_codigo.md) · [08](../08_scripts_tocam_propriedades.md) · [09](../09_propriedade_sem_salvar.md) ·
> [10](../10_objeto_no_topo.md) (reaberta em 24/10 só para ele).

## 1. Identidade

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor` · branch `line/CodeEditor`.
- HEAD: o último commit deste handoff (`git log -1`).
- Rebaseada em 24/10 sobre `b48f8b9512` (o `main` com os 6 commits de velocidade W1–W4; rebase limpo, zero `Solved`).
- 25 commits, todos só desta linha: `git cherry main HEAD` dá só `+`. O último de código é `a3e44525db` (o 10).

## 2. Superfície de colisão (`scripts/collision-surface.sh`, 23/10 — REFERÊNCIA, não evidência)

```
PROJECT_SCHEMA 193 (base 193) · tripla (193, 13, 23) · VEC_SCENE 23 · FLIP 13 · DOC_VERSION 22   — sem mudança
Registo de componentes: ph2d-ecs 108 · ph2d-render 109 · ph2d-script 109                          — sem mudança
Contrato congelado: nodegraph/node.rs intocado · editor-core/tool.rs intocado
ADR: nenhum · Cargo.lock: nenhum pacote externo novo · marcadores de conflito: nenhum
Tectos de LOC nos ficheiros tocados: nenhum passa
```

## 3. O que a linha entrega

| Item | O quê | Onde ler |
|---|---|---|
| d5 | Cena `PH2D_CODE_SMOKE=8`: os scripts tocam as propriedades pelo nome (Hero, Spikes, Coin, Door, Camera). Smoke do dono OK (23/10) | [continuação](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-23_D5_E_8B_ASSETS.md) §2 |
| 8b-1 | Mestres, imagens e scripts da lista largados no código escrevem o NOME; com Ctrl, `local NOME = "nome"` | idem §3 |
| 8b-2 | Uma linha do Inspector largada no código escreve o nome com que o script a lê (`"health.max"`, `self.giro`) | [continuação](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-23_8B_8C_CENA9.md) §2 |
| 8c | Alt ao largar um objecto escreve `ph2d.property("porta", ph2d.object("Porta"))`; o cartão Script ganha um campo que aceita um objecto largado; renomear o objecto reescreve os valores | idem |
| cena 9 | `PH2D_CODE_SMOKE=9`: arrastar do Inspector e Alt. Smoke do dono OK (23/10); estendida no 1b | idem · [09](../09_propriedade_sem_salvar.md) |
| mutação | 79 mutações nas leis do 8d/8b/8c: 53 sangraram, os sobreviventes ganharam asserção (`1ed7ceba82`) e 2 são equivalentes | idem §2 |
| **1b** | **A propriedade aparece no cartão Script SEM salvar** (o `@export` do Godot, oráculo corrido 2×). Com o texto salvo partido, o cartão mostra a versão que corre, onde antes ficava vazio | [09](../09_propriedade_sem_salvar.md) |
| **10** | **Um `ph2d.object` no topo fora de `ph2d.property` é um aviso** (classe `object_at_top`; pausa da escrita, lista de problemas, sublinhado da chamada). Oráculo Godot 4.7.2 corrido (`GET_NODE_DEFAULT_WITHOUT_ONREADY`), sem o ponto cego dele | [10](../10_objeto_no_topo.md) |
| ferramenta | `scripts/cargo-test-narrow.sh`: um alvo que não compila sai com 2 (antes saía «0 falharam · 0 passaram» com 1); um filtro que não casa nenhum teste sai com 4; um binário que morre sai com 1, com aviso | §6 |

## 4. Fora da família (foundational / partilhado) — todo ADITIVO

- **`ph2d-script`**
  - Público novo: `ScriptHost::draft_decls`, `SceneScripts::decls_in_use` (em `scene_reload.rs`) e o recurso `ScriptDrafts` (`component.rs`, re-exportado no `lib.rs`).
  - Interno novo: `StateTable::untouched` (`pub(crate)`).
  - `ScriptValue::Object(String)` e `ScriptValueKind::Object` estão no FIM dos enums. O fio é posicional e uma variante no fim só acrescenta.
  - `ph2d.object` no topo, mais o lane, o `scene_self` e o `valores`, vieram do 8c.
  - **O `ScriptDrafts` não entra no documento:** não deriva `Serialize`, e nenhum código de undo, save ou captura o lê (grep feito na auditoria).
- **`ph2d-editor-core`**
  - Variantes novas: `DragPayload::Property { script }`, `Dropping::Names`, `Dropping::Property { names, script }`, `InspectorScriptValue::Object` e `ScriptFieldEdit::SetObject`.
  - Módulo novo `widget::property_box::arrasto`, com a gravação das áreas dos rótulos.
  - Os ganchos em `paint_property_label` e `paint_section_header`.
  - `WidgetStore::end_named_drag`.
- **`ph2d-panel-inspector`**
  - `script_object_slot_pick` e `script_prop_name` vivem em `state_components.rs`. Ficaram fora do `state.rs` pelo tecto de 600.
  - O campo de objecto no cartão Script.
  - O órfão de objecto pela chave i18n.
- **`ph2d-i18n`**: `panel.inspector.script.object_drop`, `panel.inspector.script.object_value`, `code.drop.*`, `chrome.interaction.property` e (o 10) `code.explain.object_at_top[.why]`.
- **Família, o 10**: `Syntax::stray_top_objects` (`ph2d-code-syntax`, público novo) e `Kind::ObjectAtTop` no FIM da lista de `ph2d-code-explain`.
- **Shell** (+50 / −13, a catraca `the_shell_only_shrinks` está verde):
  - `asset_drag_wire.rs`: o arrasto de propriedade e o slot de objecto.
  - `components_scenes.rs`: a física da cena 8.
  - `render_loop/fase_sprite_inspector_smokes.rs`: as portas da cena 8.
- **`hr12_widgets_a11y.rs`**: `property_box/arrasto.rs` entra no `A11Y_OPT_OUT`, porque grava áreas e não pinta.
- **`scripts/cargo-test-narrow.sh`** e a linha dele no `CLAUDE.md` §2: o exit `4` é novo.
- **`docs/Components/ferramentas/fotografa_cena.sh`**: compara o `WAYLAND_DISPLAY` de fora (d5).

## 5. O que um merge pode partir

- **`CENAS` 7 → 9 nos dois roteadores**: `ph2d-app-components/src/code_smoke.rs` e `ph2d-app-code/src/smoke.rs`. Outra linha não cria cenas do CodeEditor; o teste `smoke::tests` afirma 9 = `Some`.
- **CLAUDE.md §5.1**: a entrada do Editor de scripts diz `PH2D_CODE_SMOKE=1..9` e aponta para este handoff.
- **Variantes no fim de enums partilhados** (§4). Se outra linha acrescentou ao mesmo enum, ordene-as por chegada, no fim.
- **`state_components.rs` do Inspector**: outra linha que mexa no `state.rs` ou nesse ficheiro encontra lá as duas portas novas.

## 6. Contratos congelados

Nenhum: a superfície de colisão diz «intocado» e o grep do diff não acha `NodeOp`, `Tool`, `PanelEvent` nem `VectorOp` em linha nova.

## 7. Portões do fecho (23/10, MEDIDOS)

| Portão | Resultado |
|---|---|
| `nextest-impacted.sh` (fecho, 1.ª corrida sobre `1090d128a1`) | 19 067 corridos: 19 059 verdes, **7 FAIL + 1 TIMEOUT**. Os 6 da linha foram curados em `9c152d9613`: cena 3 passo 7, tecto do `scene.rs` (703), tecto do `state.rs` (623), hr12 `arrasto.rs`, HR-15 `object(…)` e `smoke::tests` CENAS. Os 2 de fora passam sozinhos com load a 40–46: `ph2d-app-board opening_the_room_…` e `painter-layers seam_estado_escondido::cada_interruptor_…` (143 s; já na faixa pesada do `nextest.toml`) |
| `nextest-impacted.sh` (fecho, 2.ª corrida sobre `4dd85f630f`) | 19 067 corridos: **19 057 verdes, 1 FAIL e 9 TIMEOUT, com load 134–172** (outras linhas a correr). Nenhum é de crate ou teste tocado pela linha: `ph2d-tool-painter` ×3, `ph2d-app-flip` ×2, `ph2d-app-light2d` ×2, `painter-layers` e `app-components::as_duas_barras_do_igual_…` (tecto, fora do código da linha). **Os 9 corridos SOZINHOS: os 9 verdes**; dois tiveram de ser repetidos porque a 1.ª corrida esperou a vez da placa até ao prazo. É a família do `FLAKES_DE_CARGA.md` |
| `clippy --workspace --all-targets --features bevy_ecs` | 3 avisos da linha curados em `4dd85f630f`, depois **0** |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | verde |
| `cargo fmt --all --check` | verde, depois de formatar 4 ficheiros da linha |
| `check-standalone-optional.sh` · `check-workflow-packages.sh` | verdes (11 crates · 32 nomes) |
| `censos-da-arvore-combinada.sh` | 121/121, com os 12 censos corridos |
| `file_loc_caps` · `the_shell_only_shrinks` · `arch_safe_clamp_only` | verdes (na suíte e de novo depois do fmt) |
| **Reabertura do 10 (24/10), só sobre `4afc3fd320..a3e44525db`** | clippy `-D warnings` nas 4 crates tocadas (`ph2d-code-syntax`, `-code-explain`, `app-code`, `i18n`) `--all-targets`: **0**. `nextest-impacted.sh BASE=4afc3fd320` (fecho): **16 651/16 655**, 4 TIMEOUT de pintura com load 75–180 (`painter-layers` ×3, `tool-painter` ×1), **os 4 verdes sozinhos** (load 15–40). `cargo fmt --all --check`: os 5 ficheiros do 10 limpos; 6 diffs em `crates/ph2d-gpu`, vindos do `main` (`9e7a70cd61`, velocidade W2) — dívida do `main`, não desta linha. Testes por nome: os 2 novos verdes, o de ponta a ponta **RED medido** sem a ligação; censo do `ph2d-code-explain` (`classe`, 3) verde |
| `cfg(target_os` novo | nenhum ⇒ não houve cruzamento para macOS |
| `Cargo.toml` tocado | nenhum ⇒ `machete` sem objecto |
| Auditoria, 2 lentes (correção · costura/portões) | **Costura**: nada. **Correção**: 4 pontos baixos. Curados (`1090d128a1`): `state_set` no topo do rascunho escrevia o `StateTable` real (asserção vermelha sem a cura, medida) e dois casos do `cargo-test-narrow.sh` (`--color`/`-p` perdiam o valor; um binário que morria era lido como «não compilou»). **Aceite:** uma aba recarregada por fora com texto já partido mantém a última lista boa do texto anterior — a lei do Godot (§2.3 do doc 09) |

## 8. Binário do smoke

`rm -rf target/*/incremental`, depois `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` duas vezes; a 2.ª saída:

```
▸ linha line_codeeditor · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.24s
```

(Refeito em 24/10 sobre `a3e44525db`, o 10.)

## 9. Smoke para o DONO — **OK em 24/10** (cena 9 com o passo ③ sem salvar, binário da linha)

Cena 9, o passo novo ③. As cenas 8 e 9 já estão aprovadas.

1. Depois de integrado: `cd /home/enio/Documentos/Projetos/PH2D && PH2D_CODE_SMOKE=9 cargo run -p ph2d-host-desktop --profile smoke`.
   **Antes** de integrar, a pasta principal NÃO tem a cena 9 (`CENAS = 7` no main: o app abre vazio — mordeu o dono
   em 24/10); o binário pronto é o da linha:
   `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && PH2D_CODE_SMOKE=9 ./target/smoke/ph2d-host-desktop`
2. Faça o ① e o ② como no script, com Ctrl+S. Depois segure **Alt** e arraste **«Door»** da Hierarquia para a linha vazia logo abaixo do comentário ③.
3. **Sem apertar Ctrl+S**, a aba continua com ●. Em cerca de 1,5 s aparece no cartão **Script** do Inspector a linha **«door  Door»**. Agora Ctrl+S, e na aba **Cena** a porta sobe e desce.
4. **Deu errado se** o campo «door» só aparece depois do Ctrl+S, ou se o cartão Script fica sem «giro».

Fotos da corrida conduzida: `docs/CodeEditor/ferramentas/conduz_inspector/foto_09_campo_door_sem_salvar.png` e as vizinhas.

### 9.1 O aviso do objeto no topo (o 10, 24/10) — por fazer pelo dono

1. O mesmo comando da cena 9 (antes de integrar, o binário da linha):
   `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && PH2D_CODE_SMOKE=9 ./target/smoke/ph2d-host-desktop`
2. Clique na **linha 7** do script (a linha vazia logo abaixo do comentário ③) e escreva
   `local porta = ph2d.object("Door")` (o editor fecha as aspas e o parêntese sozinho). Pare de escrever.
3. Em cerca de 1,5 s a chamada `ph2d.object("Door")` fica **sublinhada a amarelo**, a barra de baixo passa
   a dizer **⚠ 3**, e em **Problems (3)** aparece **«ph2d.object here runs before the scene exists — Line 7»**.
   Apague a linha (Ctrl+Z): o aviso some. Depois, o passo ③ normal (Alt + arrastar «Door») não tem aviso.
4. **Deu errado se** nada fica sublinhado, ou se o passo ③ (o `ph2d.property("door", ph2d.object("Door"))`) também acusa.

Fotos da corrida conduzida (binário da linha, `a3e44525db`): `docs/CodeEditor/ferramentas/conduz_objeto_no_topo/foto_01_sublinhado.png`, `foto_02_lista_de_problemas.png`, `foto_03_desfeito.png`.

## 10. ABERTO

- ~~Um `ph2d.object("X")` no topo, fora de `ph2d.property`~~ — **feito em 24/10** ([10](../10_objeto_no_topo.md)).
- ~~Aba recarregada por fora com texto partido~~ — **decisão do dono 24/10: fica assim** (a última lista boa, a lei
  do Godot). Não «cure».
