# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 23/10 — F27 B2: todo campo do Inspector é um canal

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, base `main` `972f411a6f`. Anterior:
  [F27 B1](HANDOFF_CONTINUACAO_line_UIUX_2026-10-23_F27_B1_COMPONENTE.md); integração a fechar:
  [F18–F26](HANDOFF_INTEGRACAO_line_UIUX_2026-10-22_F18_A_F26.md).
- Desenho e medição: [`../F27_bloco_B_plano.md`](../F27_bloco_B_plano.md) §5. **Não está copiado aqui.**
- Decisão do dono (23/10), mantida: `ph2d-panel-painter-layers` fica FORA do Bloco B.
- Estado: B2 construída e verificada por nome; **smoke do dono OK em 23/10** (cena `=29`). Próxima LLM: janela nova.

## 1 — Commits

| commit | o quê |
|---|---|
| `a1b7353b77` | B1: família COMPONENTE + Câmera (cena `=28`) |
| `7eeffe61a8` | handoff da B1 |
| `cc05947a2f` | **B2**: sobre `7eeffe61a8`; `git show --stat` mede 81 ficheiros, +2093/−605 (o plano entra na conta) |
| (este) | este handoff + `README.md` da pasta |

**B1 fechada com prova de mutação, 4/4 sangraram:** colisão nos 32 bits →
`nenhum_componente_partilha_os_32_bits` (+ colateral esperado `toda_chave_resolve_para_o_seu_campo`) · sem
`property_laws::sanitize` → `a_lei_da_camera_vale_para_a_curva` · sem `drive.fields.driven` →
`a_curva_chega_ao_componente_e_nao_ao_documento` · sem a tabela derivada →
`os_campos_derivados_resolvem_no_catalogo`. Smoke do dono de `=27` e `=28` OK em 23/10.

## 2 — Como foi construído

Mecanismo e medição: plano §5. Aqui só o mapa do código.

### 2.1 `ph2d-editor-core` — o rótulo tipado

- `widget/property_box/key_mark.rs`: `Rotulo` (`From<&str>`, `From<&String>`, `From<TextKey>`; `vista`,
  `de_campo`, `de_campos`); registo `ROTULOS`, limpo em `set_key_marks`; `paint_decorator_campo`,
  `rotulos_pintados`, `rotulo_de`.
- Portas de `property_row` (`paint_fields_row`, `paint_field_row`, `paint_field_row_value`,
  `paint_text_row`, `paint_color_row`, `paint_check_row`, `paint_check_rows`) e `paint_choice_row`
  (`escolha.rs`) aceitam `impl Into<Rotulo>`. Chamadores com `&str` (vector, flip) compilam sem mudança.
- A linha de marcar pinta o losango: `Checkbox::coluna_de_fora`, `BooleanMark::pinta_coluna`;
  `decorator_rect` público.

### 2.2 `ph2d-ecs::canal_componente`

- `campo_do_rotulo`, `chaves_do_rotulo` (rótulo partilhado = chaves dos dois campos).
- Canal de TEXTO: `ler_texto`/`escrever_texto`, `info.texto`; `View::Named` de texto é canal;
  `View::Not` lê/escreve o visto (não o guardado).
- Testes: `cada_rotulo_nomeia_um_conceito_so`, `o_visible_le_e_escreve_o_que_se_ve`.

### 2.3 `ph2d-app-components`

- `canais_componente::{ler,escrever}` recebem `TextosDosCanais` (tirado do mundo e reposto em
  `publicar`/`aplicar`). Teste `o_alvo_da_camera_anima_por_nome`.
- `marcas_inspector::{do_rotulo, todos}`; teste `a_linha_pintada_com_o_rotulo_do_catalogo_tem_o_canal_dele`.

### 2.4 `ph2d-panel-inspector`

~50 secções passam o rótulo do catálogo tipado; helpers locais com `Rotulo`; `rows::coluna_da_caixa`;
`slice_nine` com `de_campos`; 15/23 `paint_decorator_dot` → canal.

### 2.5 Gate `architecture_every_editable_field_is_a_channel`

`crates/ph2d-editor-core/tests/it/architecture_every_editable_field_is_a_channel.rs`:
ESCOPO do Inspector = `None` (o crate inteiro); classe «pelo rótulo»; `ficheiro.rs::fn` em `POR_FUNCAO`
vale para todas as chamadas da função; `paint_check_row(s)`/`paint_decorator_campo`/`paint_decorator_dot`
em `PORTAS`; `o_inspector_inteiro_e_populacao` substitui `nenhum_campo_de_junta_fica_fora_do_escopo`;
`DIVIDA_FICHEIRO` = `sections/script.rs`.

### 2.6 Réguas corrigidas (liam a forma antiga)

`every_form_row_reserves_the_animation_column` (nome `fn x<'a>(`) ·
`every_word_this_panel_shows_comes_from_the_string_table` (o rótulo da linha não é letra de célula) ·
`the_ordering_labels_come_from_the_descriptor` (aceita `TextKey::new("component.`) · `anchor_mount_row`
(caixa desligada sem `.visual`).

### 2.7 Cena `PH2D_AUTOKEY_SMOKE=29`

`crates/ph2d-app-vec/src/autokey_smoke_alvo.rs`. q6 «Box A»/visível OK; q18 «Box C»/escondida OK; trilhas
`follow_fields.0` 1×2ch e `visibility.1` 1×2ch. Foto 1930×1040 lida. ⚠️ O resumo do `fotografa_cena.sh`
corta o log: leia `app.log` com `FOTO_GUARDA_TMP=1`.

## 3 — Superfície de colisão (um merge pode partir)

- **`ph2d-ecs/canal_componente.rs`:** mudança de forma da chave/trait → recompila tudo a jusante.
- **`ph2d-editor-core` (partilhado com vector/flip):** `key_mark.rs`, `property_row.rs`, `escolha.rs`,
  `checkbox/{mod,mark,label}.rs`, `toggle.rs`, `widget/mod.rs`, `property_box/{mod,row}.rs`. Linha que mexa
  nas mesmas portas conflita; `&str` continua a compilar.
- **~50 ficheiros de `sections/`** em `ph2d-panel-inspector`: qualquer linha que edite uma secção conflita.
- **O gate** (listas grandes `POR_FUNCAO`/`PORTAS`/`DIVIDA_FICHEIRO`): conflito textual provável.
- **`autokey_smoke.rs`:** despacho `=29` (próximo livre nesta linha; **recontar na integração**).
- **Nada de** `PROJECT_SCHEMA`, `DOC_VERSION` ou contrato §6.
- **Delta de contadores:** cenas de smoke +1 (`=29`); testes novos: `cada_rotulo_nomeia_um_conceito_so`,
  `o_visible_le_e_escreve_o_que_se_ve`, `o_alvo_da_camera_anima_por_nome`,
  `a_linha_pintada_com_o_rotulo_do_catalogo_tem_o_canal_dele`, `o_inspector_inteiro_e_populacao` (troca
  `nenhum_campo_de_junta_fica_fora_do_escopo`); famílias de canal +0.

## 4 — Smoke do dono (em linguagem simples)

1. Rode `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_AUTOKEY_SMOKE=29 cargo run -p ph2d-host-desktop --profile smoke`.
2. Espere uns 3 segundos: a cena grava sozinha e pára no quadro 18 com «Camera» seleccionada.
3. O quadrado do meio (Box B) sumiu. No Inspector, «Camera Follow → Target» diz «Box C», com o losango.
   Na linha do tempo há «Camera · Target» e «Box B · Visible».
4. Arraste a agulha para antes de 0,5 s: o Target volta a «Box A» e a Box B reaparece.
5. «Look Through» não tem ponto ao lado.
6. Deu errado se: o Target não muda, a Box B não some nem volta, ou o terminal diz «PARADA».

Estado: **smoke do dono OK em 23/10**.

## 5 — Verificação

- Gate `architecture_every_editable_field_is_a_channel` 6/6.
- `ph2d-panel-inspector` e `ph2d-editor-core` inteiros (motivo: portas de crate de que muitas dependem):
  4 vermelhos de réguas (§2.6), corrigidos e re-corridos por nome, verdes.
- `ph2d-app-components`: marcas 9/9 + canais 5/5 · `ph2d-ecs canal` 22/22.
- `ph2d-host-desktop --profile smoke`: `check` limpo · `the_shell_only_shrinks` verde.
- Cena `=29` fotografada e lida (§2.7).
- **Não feito:** prova de mutação da B2; clippy e `nextest-impacted` (fecho da linha).

## 6 — ABERTO, por ordem

1. A dope-sheet mostra o valor de um canal de TEXTO como número («Camera · Target 3.00»); devia mostrar o
   texto.
2. Itens de LISTA (Timers, Tweens, StateMachine, CounterWatch, SignalActions, VecAnchors, ShakeEmitter,
   resistências) são Dívida: o endereço por item é desenho próprio.
3. Enum sem opções no catálogo (fonte do HUD `label_fields.1`, forma do colisor `collider.0`) e Enum de nome
   (`sorting_layer.1`); `script_fields.1` sem chave serde; `script.rs` em `DIVIDA_FICHEIRO`; «Emit signals»
   do player fora do catálogo.
4. A tabela `COMPONENTES` da câmera em `marcas_inspector.rs` ficou redundante com o rótulo (só a máscara de
   32 bits precisa dela): cortar junto com o teste.
5. `script.rs::controlo` pode pintar um ponto duplo (a `Checkbox` sem `coluna_de_fora`): conferir no smoke.
6. B3, Bloco C, Bloco D e o item 9 do F18–F26: como no [handoff da B1](HANDOFF_CONTINUACAO_line_UIUX_2026-10-23_F27_B1_COMPONENTE.md) §6.
7. **FORA:** o mesmo da B1.
8. Próxima cena de smoke: `PH2D_AUTOKEY_SMOKE=30`.
