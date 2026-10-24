# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 23/10 — F27: fecho da B2, B3, Bloco C

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, base `b9e0132934` (main `972f411a6f`), **sem integrar**.
  Anterior: [F27 B2](HANDOFF_CONTINUACAO_line_UIUX_2026-10-23_F27_B2_TODO_CAMPO.md); integração a fechar:
  [F18–F26](HANDOFF_INTEGRACAO_line_UIUX_2026-10-22_F18_A_F26.md).
- Desenho e medição: [`../F27_bloco_B_plano.md`](../F27_bloco_B_plano.md) §4 (tabela do Bloco C) e §5. **Não está copiado aqui.**
- Decisão do dono (23/10), mantida: `ph2d-panel-painter-layers` fica FORA do Bloco B.
- Estado: construído e verificado por nome; **smoke do dono (§4) por fazer**. Próxima LLM: janela nova.

## 1 — Commits (`git log --oneline b9e0132934..HEAD`)

| commit | o quê |
|---|---|
| `46c9341c8c` | fecho da B2: canal de TEXTO na dope-sheet; tabela `COMPONENTES` da câmera cortada |
| `c7579286ff` | B3: os painéis sem lado escolheram (gate de `SEM_COLUNA`) |
| `18dc5ae68e` | Bloco C: `marcas()` 51,5 ms → 0,23 ms com 462 canais |
| `e96a10c6dc` | cena `PH2D_AUTOKEY_SMOKE=30` |
| `67617ac513` | fecho de onda: `cargo fmt --all`, clippy, agulha do gate de cores |
| (este) | este handoff + `README.md` da pasta |

## 2 — Como foi construído

### 2.1 Fecho da B2 — o canal de texto na dope-sheet

- `ph2d_timeline::TrackValues` deixou de ser tuple: `{valores, textos}`. `publish_com_textos(tracks,
  Option<&TextosDosCanais>, sample)` leva o texto do valor de agora e de cada chave das rows `is_text()`;
  `texto(v)`; `publish` = `publish_com_textos(.., None, ..)`. Teste `a_text_row_carries_its_texts_and_a_number_row_none`.
- Shell: `timeline_bridge_keys.rs::publish_track_values` passa `src.world.get_resource()` (1 linha; mesma contagem de linhas).
- Painel: `tracks_value.rs` escreve o texto; `graph_paint.rs` — `paint_frame` não escreve a escala num canal de
  texto, `paint_anchors` escreve o texto de cada chave à direita até à chave seguinte; `paint_track` ganhou
  `values: &TrackValues` (`expect(too_many_arguments)`). Não há «dica» de valor na timeline.
- `marcas_inspector.rs`: a tabela `COMPONENTES` da câmera saiu; fica `mascara()` (32 bits + cabeçalho,
  `camera_fields.4`, que não pinta rótulo). Teste `a_mascara_da_camera_resolve_no_catalogo` substitui
  `os_campos_derivados_resolvem_no_catalogo`. Isenções POR_FUNCAO `camera_body/follow_body/limits_body` nomeiam os rótulos do catálogo.
- Teste novo `key_mark_tests::a_linha_pintada_regista_o_rotulo_do_catalogo` (editor-core).
- `script.rs::controlo` ponto duplo: **NÃO há** (foto de `PH2D_SCRIPT_SMOKE=1`, secção Script aberta, linha «active» com um ponto).
  Resta cosmético: a caixa de «active» é mais estreita que as vizinhas (a `Checkbox` com `seccao` por omissão recalcula as colunas dentro do `ctrl`).

### 2.2 B3 — gate `architecture_every_editable_field_is_a_channel.rs`

`toda_crate_que_pinta_campos_escolheu_o_seu_lado` conta também os primitivos SEM_COLUNA: tabela
`SEM_COLUNA_FORA: (crate, "ficheiro.rs::fn", Classe, porquê)` com censo de obsolescência; leitor partilhado
`primitivos(s)`; o olho conta também com `paint_icon(` e `_button(` (era `paint_icon_button(`). As 8:

| crate | `ficheiro.rs::fn` | classe |
|---|---|---|
| hierarchy | `filter.rs::paint_chip` | NaoAnimavel (vista) |
| hierarchy | `row.rs::paint_row_eye` | **Canal**: escreve `Visibility.hidden` (a linha «Visible»); o Auto Key compara com a curva do selecionado em `fase_autokey` depois de toda escrita da UI |
| skeleton | `section_tip.rs::tip_row`, `section_smart.rs::smart_action_row` | NaoAnimavel (fiação) |
| timeline | `transport_animation.rs::chip`, `transport_clips.rs::cluster` | NaoAnimavel (vista) |
| timeline | `stack_lane_paint.rs::paint_lane` (mudo da faixa) | NaoAnimavel (parâmetro da animação) |
| flip-frames | `paint_toolbar.rs::dropdown_chip` (ease do tween, ciclo) | NaoAnimavel (a família FLIP não tem ease) |

`easing_rows`: razão reescrita na lei do `VECTOR_STATE_` («parâmetro de uma animação; não se anima a si própria»).
`TEXTO_CONTEUDO` (canal_vetor 199): nenhum controlo de painel, edita-se no canvas; nada a fazer no gate.

### 2.3 Bloco C — custo do losango

Tabela completa: plano §4. Resumo: `marcas()` com 462 canais **51,5 ms → 0,23 ms** por quadro (64 canais 6,9 → 0,13 ms).

1. `ph2d-ecs/src/canal_componente.rs::info` fazia blake3 do nome por chamada; o id vem da chave. Gate `o_info_le_o_id_da_chave_sem_hash`.
2. `estado_do_grupo` varria todas as ligações por grupo. Agora `ph2d_timeline::Ligacoes::de(doc, entidades, inst).estado(reps, valor)`
   (índice (objecto, grupo) + relógio por objecto, uma vez por quadro), usado nos 3 pintores: `marcas_inspector::marcas`,
   `ph2d_app_vec::marcas::marcas`, `ph2d_app_flip::marcas_camada::marcas`. `channel_mark_b` recebe a ligação resolvida.
   Gate `as_ligacoes_respondem_o_que_a_varredura_respondia` (mutação «grupo de família por canal» vista vermelha).
3. Sem cura: `linhas::layout` é linear (462 faixas 85 µs; 7 392 faixas 2,1 ms); registo ~0,8 µs por rótulo pintado.
4. Sondas `#[ignore]`: `marcas_inspector_custo_tests.rs`, `linhas_custo_tests.rs` (comando no cabeçalho de cada).

### 2.4 Cena `PH2D_AUTOKEY_SMOKE=30`

`crates/ph2d-app-vec/src/autokey_smoke_texto.rs`; monta pela `=29` (`montar`/`seguir` ficaram `pub(crate)`). Grava o alvo da câmera
Box A (q0) / Box C (q12) com Auto Key, pára no q6 com a câmera escolhida, Auto Key LIGADO. Foto 1930×1040 lida: linha
«Camera · Target» escreve «Box A»; log `q6 … «Box A» OK`, trilha `follow_fields.0 1×2ch`. **A foto não cobre** o gráfico
(nenhuma cena abre o gráfico de uma linha: `TimelinePanelState::expanded` é do painel) **nem o clique no olho** (eventos sintéticos não chegam à Xwayland).

## 3 — Superfície de colisão (um merge pode partir)

- **`ph2d-timeline`:** `track_values.rs` (forma do tipo muda), `channel_group.rs`, `channel_mark.rs`, `lib.rs` (`Ligacoes` exportada).
- **`ph2d-ecs/src/canal_componente.rs`** (`info`) · os três `marcas*.rs` · `graph_paint.rs` / `tracks.rs` / `tracks_value.rs`.
- **Shell:** `timeline_bridge_keys.rs` (1 linha).
- **Gate** `architecture_every_editable_field_is_a_channel` (listas grandes: conflito textual provável) e `quantas_entradas_tem_cada_painel.rs` (agulha do bloco de cores).
- **`cargo fmt --all` em ~30 ficheiros**, a maioria ~19 secções do Inspector da B2: colide com quem as edite; o fmt é mecânico, refaça-o.
- **`autokey_smoke.rs`:** despacho `=30` (**recontar na integração**).
- **Nada de** `PROJECT_SCHEMA`, `DOC_VERSION` ou contrato §6.
- **Delta de contadores:** cenas de smoke +1 (`=30`); testes novos: `a_text_row_carries_its_texts_and_a_number_row_none`,
  `a_mascara_da_camera_resolve_no_catalogo` (troca `os_campos_derivados_resolvem_no_catalogo`),
  `a_linha_pintada_regista_o_rotulo_do_catalogo`, `o_info_le_o_id_da_chave_sem_hash`,
  `as_ligacoes_respondem_o_que_a_varredura_respondia`; sondas `#[ignore]` +2 ficheiros.

## 4 — Smoke do dono (em linguagem simples)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_AUTOKEY_SMOKE=30 cargo run -p ph2d-host-desktop --profile smoke`
2. Na Timeline, na linha «Camera · Target», ao lado do nome tem de estar escrito «Box A» (não um número). Errado: aparece um número como «1.00» ou «3.00».
3. Clique no triângulo ▶ à esquerda de «Camera · Target» para abrir o gráfico: junto de cada ponto tem de aparecer o nome «Box A» e «Box C», e a escala de números à esquerda não aparece. Errado: números na escala, ou nenhum nome junto dos pontos.
4. Arraste o cursor do tempo para depois do quadro 12: a linha passa a dizer «Box C».
5. Na Hierarquia clique na «Box B» (para a escolher), leve o tempo ao quadro 12 e clique no olho dela: a caixa aparece no desenho e na Timeline nasce uma linha «Box B · Visible» com uma chave no 12. Errado: o olho muda mas nenhuma linha/chave aparece.

Os passos 3 e 5 são exactamente o que a foto não cobriu.

## 5 — Verificação (fecho de onda)

- `cargo fmt --all`; clippy `-D warnings` verde em ph2d-ecs, ph2d-timeline, ph2d-panel-timeline, ph2d-editor-core,
  ph2d-app-components, ph2d-app-vec, ph2d-app-flip, ph2d-panel-registry-init, ph2d-panel-inspector, ph2d-host-desktop
  (2 avisos corrigidos: `or_default`, `then_some`).
- Testes por nome verdes: track_values 4 · channel_group 3 · channel_mark 5 · tracks_value 4 · graph 27 · linhas 9 ·
  key_mark 4 · gate every_editable_field 6 · marcas_inspector 9 · canais_componente 5 · canal_componente 8 ·
  ph2d-ecs canal 23 · app-vec marcas 10 · marcas_camada 3 · timeline_bridge 41 · `the_shell_only_shrinks` 1.
- `nextest-impacted` (onda): 19 937 testes, 1 FAIL determinístico `ph2d-panel-registry-init … a_coluna_de_um_bloco_de_cores_sai_de_todos_os_nomes`
  (agulha de fonte lia a forma antiga de `color_tint.rs`; a B2 tipou o rótulo) — **corrigida** (agulha nova), verde por nome.
- 5 TIMEOUT no Painter (`ph2d-panel-painter-layers` seam_estado_escondido, `ph2d-tool-painter` presets_de_fabrica ×2,
  midias_de_fabrica, macro_track tutorial) a loadavg 40–156, crates que esta onda não tocou; **não reexecutados**,
  candidatos a [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md) — reconfirme a calma antes de culpar.
- Prova de mutação da B2 (agente `mutacao`, 2 rondas; controlos verdes, população 1 cada, árvore limpa):
  M1 tirar registo `ROTULOS` → reprova o teste novo E `a_linha_pintada_com_o_rotulo_do_catalogo_tem_o_canal_dele` ·
  M2 `chaves_do_rotulo` vazio → só o de ponta a ponta · M3 sem inversão `View::Not` → só `o_visible_le_e_escreve_o_que_se_ve` ·
  M4 `ler_texto` None → só `o_alvo_da_camera_anima_por_nome` · M5 rótulo duplicado no catálogo (audio `source_fields.0` → `luz_fields.2`) → só `cada_rotulo_nomeia_um_conceito_so`.
- Cena `=30` fotografada e lida (§2.4).

## 6 — ABERTO, por ordem

1. **Bloco D:** menu do botão direito na linha de GRUPO do dope-sheet (Delete Track e a cascata de extrapolação para
   todos os membros). O menu de faixa vive em `event_track_menu.rs`; precisa de uma variante de `ContextMenuKind` no
   `ph2d-editor-core` — `interaction/state/mod.rs` está a ~697/700 do tecto (cure por corte). Depois, o ADR «o canal de
   qualquer propriedade» (número contado no disco). Ficou para janela nova por tamanho de contexto.
2. A caixa da linha de marcar do script (`script.rs::controlo`) mais estreita que as vizinhas (cosmético).
3. Itens 3 do handoff B2 §6 (Enum sem opções, `script_fields.1`, `script.rs` em `DIVIDA_FICHEIRO`, «Emit signals») e o item 9 do F18–F26 (`canal.junta.*` só no locale base).
4. Smoke do dono §4 por fazer (gráfico e olho).
5. **FORA** (o mesmo de sempre): itens de LISTA, `ph2d-panel-painter-layers`, gravidade, física a meio, IkBendSide, Rope Length, áudio do mixer.
6. Próxima cena de smoke: `PH2D_AUTOKEY_SMOKE=31`.
