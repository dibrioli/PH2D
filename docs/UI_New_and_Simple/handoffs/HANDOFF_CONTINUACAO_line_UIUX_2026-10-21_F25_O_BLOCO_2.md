# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 21/10 — F25: o bloco 2 (a dívida do gate «todo campo editável é um canal»)

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, base `main` `100d34c8e0` (não andou). Anterior:
  [F24d](HANDOFF_CONTINUACAO_line_UIUX_2026-10-20_F24d_O_LOSANGO_SEM_COLUNA.md) (smoke aprovado 21/10; §6 item 2 = esta onda).
- Plano: [`35_plano_F24_o_auto_key_grava_tudo.md`](../vetor_nova_ui/35_plano_F24_o_auto_key_grava_tudo.md) §10 bloco 2.

## 1 — Commits desta onda

| commit | o quê |
|---|---|
| `62f112d671` | o bloco 2: 114 ficheiros, +9853/−453 |
| (este) | este handoff + fotos/logs `F25_cena_autokey_{11..15}` |

Resultado do gate `architecture_every_editable_field_is_a_channel`: `DIVIDA_FICHEIRO` VAZIA; nenhuma `Divida` do `ph2d-panel-vector` nem `INSP_*`; `DIVIDA_CRATE` só `ph2d-panel-painter-layers` (a linha PainterWatercolor está activa em 48 ficheiros das crates do Painter: colisão, fica para quando ela fechar).

## 2 — Como foi construído

### 2.1 Vocabulário (só APPEND; gates `o_vocabulario_do_*_nao_renumera`)

- **VETOR** (`crates/ph2d-ecs/src/canal_vetor.rs`): `PINCEL_*` 112–116 (membros da tinta do traço) · `TEXTO_QUEBRA_FIXA` 197 (grupo da largura) · `TEXTO_FAMILIA` 198 · `TEXTO_CONTEUDO` 199 · `OFF_PADRAO_DESFASE` 38 (`TINTA_LARGURA` 38→39; nenhum número gravado mudou) · `FILTRO_*` 220–248 (olho = grupo próprio por filtro) · `PERFIL_*` 250–253 · `PADRAO_CAMINHO_*` 260–265 · `TEXTO_CAMINHO_*` 270–271 · `BLEND_PASSOS` 280 · `ENVELOPE_*` 290–301 (um grupo) · `ANCORAS_EXISTE`/`ANCORA_*` 310–318 (discretas, um grupo; frações não arredondam) · `MOLDURA_RECORTA` 320 · `LAYOUT_*` 330–349 · `WIDGET_PELE` 350 · `WIDGET_ICONE` 351.
- O Morph vivo é o `PropKind::Morph` de sempre (não há `MORPH_T`). «Show as Panel» (`VECTOR_FRAME_PANEL_*`) e `VECTOR_LAYOUT_PAD_ALL_MODE` = `NaoAnimavel`.
- **Famílias novas** em `canal.rs`:
  - OBJETO=2 (`canal_objeto.rs`): skew 1–2 · emissão 10 (zero REMOVE o componente) · opacidade da sprite 11 · tint RGB 20–22 (alfa = `PropKind::Opacity`) · self-tint 30–33 · cantos 40–43 (j = canto) · silhueta 50.
  - JUNTA=3 (`canal_junta.rs`): motor 1–4 · quebra 10–12 · limites 20–22 · mola 30–32 · eixos do Custom 40|41|42 + 3·eixo.
  - FLIP=4 (`canal_flip.rs`, i = `LayerId`): visível, opacidade, mistura, profundidade. `LayerId` > 65535 não é endereçável (o recurso é o `u16` do `FieldKey` gravado): a camada fica sem losango, visível.
  - FISICA=5 (`canal_fisica.rs`): 41 campos do §11 «Physics Body» do Inspector (corpo, colisor, zona).
- `CampoInfo::texto` + recurso `TextosDosCanais` (índice estável, 0 = nenhum, só cresce).
- `CanaisLidos::esquecer(entidade, familia)` (antes apagava TODAS as famílias → objeto/junta deixavam valores velhos). Teste `tirar_o_componente_esquece_so_os_campos_desta_familia`.

### 2.2 Timeline (texto como canal discreto)

- `DOC_VERSION` 21→22, `textos` no doc. Na sessão as chaves indexam o recurso e o doc fica com `textos` vazio. Ao gravar, `TimelineDoc::to_bytes_com_textos` compacta numa CÓPIA (só os usados; remapeia chaves, `default` e `rest`). Ao abrir, `adota_textos` no `upkeep` do `ph2d-timeline-persist` ACRESCENTA ao recurso e remapeia (nunca substitui: um índice nunca muda de texto em sessão). `to_bytes()` sem tabela recusa perder texto.
- Discretos forçados a Hold também em `SetInterp`, `SetSelectedInterp`, «Convert to Bezier» e `simplify_recorded`.
- Fixture `crates/ph2d-timeline/tests/fixtures/timeline_doc_v21.bin` gravada ANTES da subida. `PROJECT_SCHEMA` 192 intocado (md5).

### 2.3 Implementação por família (sempre pelo MESMO funil do painel; preview ≠ documento)

- Vetor: `crates/ph2d-app-vec/src/{canais_entidade,canais_filtros,canais_vivos,canais_interface,canais_textos,marcas_vivos}.rs`.
- Objecto: `crates/ph2d-app-components/src/canais_objeto.rs` (+ `Driver::SpriteCor` em `ph2d-preview-drive`).
- Junta: `crates/ph2d-app-physics/src/canais_junta.rs` (`joint_with_edit`). Corpo: `canais_corpo.rs` (`ph2d_panel_inspector::physics_edit_for` → `apply_physics_edit`; `ConducaoFisica`).
- Flip: `crates/ph2d-app-flip/src/{canais_camada,marcas_camada}.rs` (`layers::apply_panel_event`; `ConducaoFlip`). `ph2d-app-flip` ganhou dep `ph2d-timeline`.
- Shell (composição): publicar em `fase_autokey.rs`, aplicar em `fase_timeline_drain.rs`, `undo_app.rs`. `the_shell_only_shrinks` verde.

### 2.4 Curas de passagem (controlos que mentiam ou estavam mortos)

- (a) O texto que a curva muda não se redesenhava (nem o tamanho etc., de antes): `ph2d_app_vec::canais::aplicar` devolve `Aplicado { mudou, textos }` e a shell chama `vec_text_ride::recook`. Teste `vec_text_ride_curva_tests::a_curve_on_a_text_recipe_redraws_the_text`.
- (b) Sliders W Start/Mid/End/Pos do perfil seguem o documento quando o dedo não está neles (`profile_live::store_shows`/`SLIDERS`, `fase_live_offset_and_width.rs`). Teste `os_sliders_mostram_o_perfil_depois_de_escrito_e_so_entao`.
- (c) Números da junta só seguiam na troca de selecção → `sync_joint_fields` a cada quadro, saltando o widget em foco. Teste `seam_joint::the_joint_numbers_follow_the_document_while_the_joint_stays_selected`; apagada a cópia `sync_derived_rope_length`.
- (d) Wrap width mostrava sempre 12 → `seed_text_sliders` leva a largura e a shell publica a cada quadro (removido `text_last_target`). Teste `seam_text_wrap::…`.
- (e) Chip de par convertido mostrava a PISTA (Emissive 0,006 por 0,4; Opacity 1 por 100) → `WidgetStore::linked_chip_display`, porta única (editor-core). Teste `inspector_regression::the_emissive_and_opacity_chips_show_the_value_not_the_track`. ⚠️ Outros painéis com par convertido (ex.: pintor) passam a mostrar o valor real: NÃO fotografados.
- (f) O slider Opacity do Inspector escreve `Sprite::opacity` mas o losango mostrava `PropKind::Opacity` (tint[3]) → agora o campo OBJETO 11.
- (g) Painel de Física: raio/caixa/cápsula são recusados para forma que não os mostra (`every_dimension_field_reaches_the_bus` ajustado).

### 2.5 Classificados `NaoAnimavel` (com porquê)

`ph2d-panel-physics` (mundo da simulação, ferramentas, debug: configuração) · `ph2d-panel-audio-editor` (edita o asset de som) · `ph2d-panel-audio-mixer` (estado do painel, não gravado no projeto; o motor lê do painel. Automação exigiria: mix como dado do projeto, alvo de cena na timeline ou componente de áudio por objecto, e a fase de áudio a ler por quadro).

### 2.6 Cenas `PH2D_AUTOKEY_SMOKE=11..15` (`autokey_smoke_bloco2.rs`, `autokey_smoke_juntas.rs`; despachante `1..15`)

Mudam no quadro 12, voltam no 24, param no 18.

| cena | o quê | sonda q18 |
|---|---|---|
| 11 | estrela com Drop Shadow + perfil + texto ao lado; painel nos Filtros | sombra raio 0,205, cor [0,47 0,05 0,05 0,90], perfil [0,57·2,00·0,57], texto 0,625 DejaVu Serif quebra 2,2 |
| 12 | a mesma, texto seleccionado; painel em Text/Font/Paragraph | idem 11 |
| 13 | sprite (skew 0,45, tint laranja, emissão 0,8 no q12), Inspector aberto | skew 0,225, tint [0,625 0,600 0,275], emissão 0,4 |
| 14 | dobradiça (limites ±0,4→±1,4, motor liga, força 50→400, quebra liga 500→2000) | limites ±0,9, força 225, quebra 1250 |
| 15 | mola (rigidez 20→200, amortecimento 0,5→5) | rigidez 110, amortecimento 2,75 |

Fotos e logs: `../vetor_nova_ui/fotos/F25_cena_autokey_{11..15}.{png,log}` (conferidas: o número no painel = a sonda).

## 3 — Superfície de colisão (um merge pode partir)

- **Vocabulário/formato:** `canal_vetor.rs` (append; `TINTA_LARGURA` 38→39 — gate de renumeração cobre), `canal.rs` (famílias 2–5, `MaosDasFamilias` de F24d intocado), `ph2d-timeline` `DOC_VERSION` 21→22 (outra linha que suba a versão do doc colide: reconta-se). Fixture v21 nova.
- **Cargo:** `ph2d-app-flip/Cargo.toml` +dep `ph2d-timeline`; `Cargo.lock` +2 linhas (regerar, não mesclar à mão). Sem `PROJECT_SCHEMA` (192 intocado), sem contrato §6 tocado.
- **Shell:** `fase_autokey.rs`, `fase_live_offset_and_width.rs`, `fase_text_panel.rs`, `fase_timeline_drain.rs`, `undo_app.rs`; `the_shell_only_shrinks` verde. Teste `shell_files_respect_hr18_loc_cap` movido para `vec_text_ride_curva_tests.rs`.
- **Comportamento:** `esquecer(entidade, familia)` (assinatura nova); `aplicar` devolve `Aplicado`; chips de par convertido mostram o valor (2.4 e); `linked_chip_display` em `ph2d-editor-core`; a âncora `let grabbed = matches!(` voltou ao ramo do arrasto (gate de fonte de `the_width_sliders_author_the_live_profile`).
- **Painter:** a linha PainterWatercolor mexe em 48 ficheiros das crates do Painter; esta onda NÃO tocou nelas (por isso `ph2d-panel-painter-layers` segue em `DIVIDA_CRATE`).
- Contadores como delta: gates `architecture_every_editable_field_is_a_channel` sem dívida de ficheiro; testes `canal_textos::*` (5) + `timeline_textos_tests`; famílias do `canal.rs` +4.

## 4 — Smoke do dono (em linguagem simples)

Binário: `target/smoke/ph2d-host-desktop` já compilado sobre `62f112d671` — 2.ª build `Finished smoke profile [optimized] target(s) in 0.46s`, zero «Compiling».

Em cada passo: (1) rode o comando, (2) olhe a cena, (3) compare, (4) saiba o que é errado. A cena mexe sozinha entre os quadros 12 e 24 e PÁRA no quadro 18 (a meio da volta): os números do painel têm de ser os da tabela.

1. Cena 11. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_AUTOKEY_SMOKE=11 ./target/smoke/ph2d-host-desktop`. Painel nos Filtros: a estrela tem sombra; o perfil de largura e o texto ao lado também mexem. Tem de ter losango em todo campo novo (sombra, perfil) e linhas novas no Timeline. No quadro 18 a sombra tem raio 0,205 e o perfil mostra 0,57 · 2,00 · 0,57.
2. Cena 12 (`PH2D_AUTOKEY_SMOKE=12`, mesmo comando): o texto está seleccionado, painel em Text/Font/Paragraph. No quadro 18 o texto tem tamanho 0,625, fonte DejaVu Serif e largura de quebra 2,2, e o desenho do texto na tela muda de verdade (fonte e tamanho a meio).
3. Cena 13 (`=13`): sprite com o Inspector aberto. Skew 0,225, cor (tint) a meio entre as duas, emissão 0,4. O slider Opacity mostra o valor (não 100 por 1).
4. Cena 14 (`=14`): dobradiça. Limites ±0,9, força 225, quebra 1250; os números da junta acompanham o relógio sem você clicar.
5. Cena 15 (`=15`): mola. Rigidez 110, amortecimento 2,75.
6. Errado = campo novo sem losango; número do painel diferente do que a tela mostra; texto que não muda de fonte/tamanho a meio; linha que falta no Timeline; objecto da cena 13 que perde o brilho para sempre depois de emissão zero.

## 5 — Verificação

- `cargo fmt` ✓. `clippy --workspace --all-targets -D warnings` ✓ (depois de 5 curas). `cargo check --workspace` ✓.
- `nextest-impacted`: 20410/20414 na 1.ª corrida (968 s, loadavg 32–80). As 4 reprovações curadas e re-corridas por nome verdes: `no_tofu_glyphs`, `shell_files_respect_hr18_loc_cap` (teste movido), e os dois gates de fonte de `the_width_sliders_author_the_live_profile`.
- Gates nomeados verdes: `architecture_every_editable_field_is_a_channel`, `the_shell_only_shrinks`, `architecture_workspace_file_loc_cap`, `architecture_panel_wiring_parity`, `architecture_claude_md_cabe_no_orcamento`. `ph2d-timeline` inteira verde (568).
- Mutação (timeline-textos): 3 sangraram — Hold nos discretos, tabela inteira, sem `adota_textos`.

## 6 — ABERTO, por ordem

1. **Smoke do dono** das cenas 11–15 (§4).
2. **Painter** (camadas, ajustes e o «quadro a quadro» do §9 Q2 do plano) quando a linha PainterWatercolor fechar: `ph2d-panel-painter-layers` é a única `DIVIDA_CRATE`.
3. **Física a meio da simulação:** a curva actualiza o componente mas o corpo vivo só se re-descreve em repouso (como a edição do painel) — estender a ponte como as juntas já fazem.
4. **Decisão do dono:** gravidade do MUNDO animável (precisa de alvo de cena na timeline).
5. **Áudio do mixer:** as 3 mudanças de 2.5.
6. Troca do eixo do motor zera o alvo durante um quadro (funil do painel).
7. Expressão num campo de texto produz índice arbitrário e `SetKeyValue` aceita fracção (nada interpola, mas não está restringido).
8. O losango da camada do Flip só aparece em Draw/Edit.
9. O contentor do Envelope entrar na população do autokey é suposto, NÃO verificado.
10. Ficheiros do Inspector ainda fora do ESCOPO (ex.: `joint.rs` tipo/Active/Collide Connected).
11. De F24d §6, inalterados: perf de `marcas()` · menu do botão direito na linha de grupo · perf de `linhas::layout` · ADR «o canal de qualquer propriedade» · gate de fecho da LINHA · `CLAUDE.md` §5.1 UI/UX ao fechar (link do handoff; `PH2D_AUTOKEY_SMOKE=1..15`; NÃO alterado nesta onda) · undo da timeline separado do do vetor.
