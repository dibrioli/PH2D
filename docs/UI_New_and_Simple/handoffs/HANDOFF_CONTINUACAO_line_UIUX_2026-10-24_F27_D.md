# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 24/10 — F27 Bloco D: menu da linha de GRUPO, ADR-0191, dívidas do catálogo

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, **rebaseada sobre o main `b48f8b9512`**; ponta anterior
  `965679c088`. **Sem integrar.** Anterior: [F27 B3 + C](HANDOFF_CONTINUACAO_line_UIUX_2026-10-23_F27_B3_C.md);
  integração a fechar: [F18–F26](HANDOFF_INTEGRACAO_line_UIUX_2026-10-22_F18_A_F26.md).
- Desenho e medição: [`../F27_bloco_B_plano.md`](../F27_bloco_B_plano.md). **Não está copiado aqui.**
- Decisão do dono (23/10), mantida: `ph2d-panel-painter-layers` fica FORA.
- Estado: construído e verificado por nome; **smoke do dono (§4) por fazer**; «suíte do diff» por preencher (§5). Próxima LLM: janela nova.

## 1 — Commits (`git log --oneline 965679c088..HEAD`)

| commit | o quê |
|---|---|
| `38b7adc499` | menu do botão direito na linha de GRUPO do dope-sheet (Delete Track + cascata Pre/Post) |
| `c8b1da82eb` | ADR-0191 «o canal de qualquer propriedade é o catálogo de componentes, ligado pelo rótulo» |
| `d9ce636aa6` | `script.rs::controlo`: a caixa do Bool cai na coluna do valor das vizinhas |
| `310b64d0fa` | a camada de ordenação é canal; dívidas do catálogo com a razão MEDIDA |
| `bd1c36c9e0` | cena `PH2D_AUTOKEY_SMOKE=31` — o menu da linha de grupo |
| `f7d7c24ddd` | clippy da onda: imports a mais nos testes do menu de grupo |
| (este) | este handoff + `README.md` da pasta |

## 2 — Como foi construído

### 2.1 Menu de grupo (`38b7adc499`)

- **editor-core:** `TrackMenuKind::Group`; `ContextMenuKind::TimelineTrackGroup { target }` (em `types_menu.rs`;
  reusa `TIMELINE_TRACK_MENU`, linhas em `menu_rows.rs`); `TimelineExtrap` ganhou `grupo: bool`;
  `pointer_down_menus.rs` mapeia Group. `interaction/state/mod.rs` (695/700) **NÃO foi tocado**.
- **Painel:** `linhas.rs::paint_grupo` regista a etiqueta com `ids::timeline_group_row_id(handle)` (escada própria em
  `ids/timeline_grupos.rs`). `timeline_row_id(handle)` continua a ser a do 1.º membro quando o grupo está aberto.
- **`event_track_menu.rs`:** `membros(handle)` resolve por `GrupoChave::do_handle` + `GrupoChave::de` sobre o
  instantâneo — todos os membros, abertos ou não; grupo sumido ⇒ nada. `por_membro` envolve as ações em
  `BeginEdit`/`EndEdit` ⇒ **um** passo de undo. Delete Track = `Unbind` por membro; folha da cascata = `SetTrackExtrap` por membro.
- **Testes** (`grupo_de_canais_seam`): `o_botao_direito_na_etiqueta_do_grupo_abre_o_menu_de_grupo` (clique direito real
  pelo dispatch) · `delete_track_no_grupo_apaga_todos_os_membros_num_passo_de_undo` (doc real: 4 canais da cor saem, a
  rotação fica, um Undo devolve os 4; **mutação: sem `BeginEdit` ⇒ VERMELHO, visto**) ·
  `a_extrapolacao_do_grupo_vale_para_todos_os_membros`.

### 2.2 ADR-0191 (`c8b1da82eb`)

Número **PROVISÓRIO**: o main tem 0189; `line/Macros-Automação` tem 0190 por integrar. **O integrador reconta** (renomear
ficheiro, links, índice). Índice regenerado por `scripts/adr-index.sh`.

### 2.3 Caixa do Bool do script (`d9ce636aa6`)

`sections/script.rs::controlo`: o Bool recebia só o `ctrl` com a `Seccao` por omissão e `paint_boolean_mark` partia-o em
nome+valor (a caixa começava a meio). Agora recebe a fileira inteira `(x, seccao)` com a mesma borda direita. Foto
`PH2D_SCRIPT_SMOKE=1` com a secção Script aberta (edição temporária em `components_scenes.rs::script_smoke` com
`set_collapsed(INSP_LIVE_SCRIPT_SECTION,false)`, **restaurada, não comitada**): «active» alinhada, um ponto.

### 2.4 Camada de ordenação e dívidas do catálogo (`310b64d0fa`)

- `ph2d-ecs/canal_componente::faixas`: `Enum` + `View::Named(Target::Layer)` = 1 faixa discreta (o valor é o ID inteiro da
  camada; lê/escreve pelo `numero`); outros `Named` não-texto continuam 0. Teste `a_camada_de_ordenacao_anima_pelo_id`.
- Gate `architecture_every_editable_field_is_a_channel`: `ordering.rs::layer_row` Divida→Canal
  (`component.field.sorting_layer.1`); `INSP_PLAYER_EMIT` Divida→NaoAnimavel (marcador `PlayerSignals`: estrutura, a lei do
  Mask2D). Razões corrigidas: `hud_corpo::bloco_rotulo` (as variantes de `LabelSource` carregam o nome —
  `Counter(String)`…; `options!` não exprime) e `script_seletor::seletor` (`LuauScript::source` é `ScriptSource`,
  referência por id, `Hidden::Editor`). `collider.0` já estava coberto: canal FISICA `FORMA = 20`
  (`marcas_inspector.rs:149`, `canais_corpo.rs:57`).

### 2.5 Item 9 do F18–F26 (`canal.junta.*` só no locale base) — fechado por PROVA, sem código

Foto `PH2D_AUTOKEY_SMOKE=14` com `PH2D_LANG=teste` (o roteiro aceita `"VAR=a VAR2=b"` num argumento): as linhas da junta na
dope-sheet e no Inspector saem deformadas («[M…», «[Ɓŕ…») ⇒ vêm da tabela; e o censo `toda_palavra_da_tabela_sai_deformada` cobre `canais.rs`.

### 2.6 Cena `PH2D_AUTOKEY_SMOKE=31` (`bd1c36c9e0`)

`crates/ph2d-app-vec/src/autokey_smoke_grupo.rs`, despachada em `autokey_smoke.rs` como `GRUPO = 29`. Retângulo «Path 0» com
Fill vermelho→azul (4 canais VETOR `TINTA_PREENCH+OFF_SOLIDA+c`, uma linha «Fill») e Opacity 40→100 % em 0..1 s, relógio
parado a 0,5 s, Timeline aberta. Foto 1930×1040 lida: «Path 0 · Fill» e «Path 0 · Opacity 0.70», forma roxa; log «5 faixas
(tem de ser 5)»; `app.log` sem erro. **A foto não cobre o clique direito.**

## 3 — Superfície de colisão (um merge pode partir)

- **`ph2d-editor-core`:** `types_menu.rs` (variante nova + campo novo em `TimelineExtrap` — **quem constrói `TimelineExtrap`
  noutra linha parte**), `menu_rows.rs`, `pointer_down_menus.rs`.
- **`ph2d-panel-timeline`:** `linhas.rs`, `event_track_menu.rs`, `ids/timeline_grupos.rs`.
- **`ph2d-ecs/canal_componente.rs`** (`faixas`).
- **Gate** `architecture_every_editable_field_is_a_channel` (listas grandes: conflito textual provável).
- **`ph2d-app-vec/src/autokey_smoke.rs`:** despacho `=31` (**recontar** o número de cena na integração).
- **ADR 0191:** número provisório (recontar; main 0189, Macros 0190).
- **Nada de** `PROJECT_SCHEMA`, `DOC_VERSION` ou contrato §6.
- **Delta de contadores:** cenas de smoke +1 (`=31`); ADR +1; testes novos: os 3 do menu de grupo,
  `a_camada_de_ordenacao_anima_pelo_id`.

## 4 — Smoke do dono (em linguagem simples)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_AUTOKEY_SMOKE=31 cargo run -p ph2d-host-desktop --profile smoke`
2. Na Timeline, separador Keys, botão direito no NOME «Path 0 · Fill»: aparece o menu Delete Track / Pre / Post.
3. Escolha Post → Loop e carregue em play: depois de 1 s a cor volta a vermelho e repete; a opacidade fica parada em 100 %.
   Ctrl+Z desfaz tudo de uma vez.
4. No mesmo menu, Delete Track: a linha Fill some inteira e a Opacity fica. Ctrl+Z traz a cor de volta inteira.

Deu errado se: o menu não abre no nome do grupo, só um canal da cor muda, ou o Ctrl+Z traz só parte.

Além disso, o smoke §4 do [handoff anterior](HANDOFF_CONTINUACAO_line_UIUX_2026-10-23_F27_B3_C.md) (gráfico de uma linha e o olho) já foi dado OK em 24/10 naquele ficheiro; **este** é o que falta.

## 5 — Verificação

- `cargo fmt --all` (reverteu 4 ficheiros de `ph2d-gpu` que o fmt mexeu e não são desta linha); clippy `-D warnings` limpo em
  editor-core, panel-timeline, panel-inspector, ecs, app-vec.
- Testes por nome verdes: panel-timeline `grupo` 11 · `extrap` 5 · `track_menu` 4; ecs `canal_componente` 9; editor-core
  `architecture_every_editable_field_is_a_channel` 6 · `menu` 106.
- Mutação do menu de grupo: sem `BeginEdit` ⇒ vermelho (visto, §2.1).
- Fotos lidas: `=31` (§2.6), `PH2D_SCRIPT_SMOKE=1` (§2.3), `PH2D_AUTOKEY_SMOKE=14` com `PH2D_LANG=teste` (§2.5).
- **Suíte do diff (`PH2D_SUITE=onda nextest-impacted`, 19 296 testes, `load` 28–48):** 2 vermelhos, nenhum desta
  linha (o diff da onda sobre as duas crates é vazio): `ph2d-panel-painter-layers` `seam_estado_escondido::cada_interruptor_que_neutraliza_pinta_como_o_pincel_limpo`
  TIMEOUT 360 s (a família dos TIMEOUT do Painter sob carga, 23/10) e `ph2d-tool-painter`
  `o_custo_no_caminho_do_app::o_deposito_de_um_pincel_pesado_de_fabrica_vai_pela_rota_que_paga` (régua paralelo × série
  de `a5ef930e34`; reprova também sozinho, a `load` 26–28 — sem núcleos livres o paralelo não ganha). Candidato à
  [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md): confirmar a `load` < 5 antes de culpar alguém; é da linha Painter.

## 6 — ABERTO, por ordem

1. **Smoke do dono `=31`** (§4).
2. Dívidas declaradas que ficam — cada uma é desenho próprio, **só se o Enio pedir**: fonte do rótulo do HUD
   (`LabelSource` com dados); o ficheiro do script (`ScriptSource`); `script.rs` em `DIVIDA_FICHEIRO` (props dinâmicas).
3. A camada de ordenação anima mas nenhuma cena a mostra (projeto novo tem só a camada Default) — candidata a cena.
4. **FORA** (o mesmo de sempre): itens de LISTA, `ph2d-panel-painter-layers`, gravidade, física a meio, IkBendSide, Rope Length, áudio do mixer.
5. Próxima cena de smoke: `PH2D_AUTOKEY_SMOKE=32`.
6. **NÃO integrar nem fazer push sem ordem do Enio.** Na integração: recontar o número do ADR e da cena.
