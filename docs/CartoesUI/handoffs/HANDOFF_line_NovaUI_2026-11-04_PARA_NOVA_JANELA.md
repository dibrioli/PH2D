> **Superado pelo handoff de integração da mesma pasta** ([HANDOFF_INTEGRACAO_NovaUI_2026-11-03.md](HANDOFF_INTEGRACAO_NovaUI_2026-11-03.md)): §3 e §4 abaixo não valem.

# Handoff — `line/NovaUI`: para a janela que fecha a linha (04/11/2026)

Leitor: a LLM que assume a linha sozinha. Worktree `Worktrees/line-NovaUI`, branch `line/NovaUI`, base `c12143dcdf`.
No momento da escrita: HEAD `7c5485f5ad`, 238 commits, 1208 ficheiros, +74 921 −5 912. `main` tem 8 commits a mais (`effe04a1c9`, GEF 22).
⚠️ Estes números envelhecem: a janela anterior acrescenta o fecho das duas tarefas em curso na §3 antes de lhe dar o sinal.

## 1. A missão (ordem do dono, 04/11)

> «A partir de agora vc está sozinho e sua missão é finalizar tudo que está em aberto e fechar a linha. Depois escrever handoff
> de integração e sinalizar a janela `ph2d-44 [c6124b]` que será o agente integrador. Não tire dúvidas, não peça autorizações, não
> peça smoke. Faça tudo até o fim e entregue o handoff. Amanhã faremos o smoke.»

Entregas: (a) os abertos da §4; (b) o fecho da §5; (c) handoff de integração em `docs/CartoesUI/handoffs/`; (d) `SendMessage` a `ph2d-44`.
⛔ Você NÃO integra, NÃO faz push, NÃO faz ship: isso é de `ph2d-44`.

## 2. O que existe (não reconstrua)

Leia nesta ordem, sem ler tudo de uma vez: [10 tracker](../10_TODO_O_APP_TRACKER.md) (registo de cada bloco, decisões sem o dono,
considerações do smoke) · [05 especificação](../05_ESPECIFICACAO_UI_COMPLETA.md) §0 e §4 · [11 receita](../11_RECEITA_PAINEL_EM_CARTOES.md) ·
[19 briefing dos ajudantes](../19_BRIEFING_DOS_AJUDANTES.md). Planos: [09 alfinete](../09_alfinete_nos_cartoes_PLANO.md) · [12 Vector](../12_VECTOR_PLANO.md) ·
[13 Painter](../13_PAINTER_PLANO.md) · [14 Timeline/Motion](../14_CHROME_TIMELINE_MOTION.md) · [15 chrome](../15_CHROME_CENSO.md) · [16 referências](../16_REFERENCIAS_POR_CARTAO.md) ·
[17 varredura](../17_VARREDURA_FINAL.md) · [18 capricho](../18_REVISAO_DE_CAPRICHO.md).

| Área | Estado |
|---|---|
| Inspector, todo tipo de objecto; alfinete em Object, Appearance, Fill, Stroke, Ordering, Material, Transform, componentes; losango do preso; dois presos; girar/espelhar em todo objecto | feito |
| Physics, Tokens, Grid, Wet Paint, Macros, Tags, Assets, Audio Mixer, Padding, Upscale, Equalize, BG Removal, Color Eq, Flip, Pixel, Hierarchy | em cartões |
| Frames, Timeline, Motion Graph, Audio Editor, Pixel Timeline/Preview, Code, Authored, Widget Lab | moldura e barra (a área de trabalho não muda) |
| Vector G1–G8; Painter H1–H12 | fechados; pintor antigo só com `PH2D_CARDS=0` |
| Chrome M1–M9, varredura V1/V2, capricho q1–q4, rodadas R1–R6, mola do Kanban | feitos; abertos só os da §4 |
| Censos: `lei_da_compactacao`, `nenhum_texto_sai_do_botao`, `lei_do_ritmo`, `o_zero_tem_populacao`, `nenhum_layout_antigo_no_cartao` | a zero, só a Galeria pendente (7·1 e ritmo 16) |

**Leis do dono, vinculativas:** compactação e ritmo (05 §4, 8 leis) · nunca texto cortado ou fora do botão · referência: nossos aprovados e a Widget Gallery → Figma →
Rive → app com UI premiada → Inkscape/Godot/Unity · micro-pesquisa antes de cada cartão, com linha na [16](../16_REFERENCIAS_POR_CARTAO.md) · app só em inglês · `PH2D_CARDS=0` é o único interruptor.

**Smoke do dono já feito (04/11):** Visão geral OK · Painter OK · Gizmo da forma OK · Quadro aceite por agora («o desenho dos cartões poderia ser mais belo»). Foram curados e **não vistos por ele**: Scale travado, Weight preso, pulos no Ctrl+Z, estrela do Preset, capricho.

## 3. As duas últimas tarefas da janela anterior: feitas, sem smoke do dono

- **Risco de 1 px na borda dos sprites** (`a3e57a1306`, `ab5f5aa97a`, [22](../../Sprite_projeto/22_o_recorte_nao_le_o_vizinho.md)). O bilinear lia a célula vizinha; `crates/ph2d-gpu/src/recorte_uv.wgsl`, uma porta só, no sprite e na Luz 2D. Medido em RADV e RTX 5060 Ti. Abertos: `c3_mascara.wgsl` sem portão; pré-visualização com pixel só na foto; bits na NVIDIA não medidos e custo por fragmento não medido; a arte que encosta na borda perde orla macia ao minificar (cabelo a 0,5×: 0,39→0,58, 9 px): registe ao dono; `a_repetida_continua_a_costurar` leu zeros 1× em lote pesado, sem causa, verde em 20+ corridas; o campo Filter Clip deixou de ser preciso: fica, decisão do dono.
- **«Remove Background»** (`3f0c2add91`): nunca saiu; desde a M5 a legenda dizia «Cutout», com ícone de borracha. Agora o nome inteiro vem da dica (Remove Background, Make Square, Color Equalization…; «Reshape» virou Transform), com chip que alarga e `⋯` em lista. Inventário contra o `main` por portão. Abertos: `as_caixas_internas_vestem_a_superficie_sem_mover_alvo` (`NewImageDialog`) VERMELHO, com o teste a esperar os alvos de antes da M8: confira e cure (item 1 da §4); título do painel continua «Background Removal», decisão do dono; o inventário do Vector não teve contraprova própria; na foto do Bones só abriu, não leu.

## 4. O que está aberto, por ordem

**A. Vermelhos conhecidos.**
1. `the_tool_options_can_live_at_the_bottom`: reprova depois do trabalho em `tool_bar.rs` da barra; não isolado.
2. `every_scrub_handle_of_the_pinned_object_cards_survives_the_frames`: só medido em `--profile smoke`; meça no perfil de omissão (a queixa original).
3. Rever os 10 testes alterados na R6, que descreviam o produto antigo: coluna de ferramentas (5), moldura, título «Background Removal», letra de Damping, contador da fixtura, altura da mola. Foi o teste que estava atrasado, ou o produto regrediu?

**B. Defeitos pequenos conhecidos.**
4. `paint_stroke_popover` do Vector fora da camada de listas (`cards_lado::flutua`), e pode ficar por baixo de outro painel (17).
5. «+ Track ▾» com seta em letra (17, item 13).
6. `v2_dica.png`: faixa escura sem causa achada (17).
7. «Keep volume…» cortado em On switch ao lado de Sync tools, no Painter.
8. O «·» no início da 2.ª linha do resumo do Variant (18).
9. Linha arrastada da lista de presets do Wet Paint não voa como o cartão do Kanban (lei do dono: `feedback_every_drag_reorder_flies_like_the_kanban_card`).
10. Grade do visor do Tilt muito apagada na tinta do cartão: E10 manda não redesenhar; ajuste só de tinta, se resolver.
11. `gizmo/transform.rs:194`, `:252`: `.max(0.001)` não recupera objecto a 0 nem mantém o sinal: mesmo problema do Scale travado, no canvas. Decida e cure.
12. Equalize: Upscale + Rasterize um por linha na largura de fábrica (teste só passa no mock); Offset e Side do texto no caminho em duas linhas.
13. Cartões de sprite nascem sem alfinete: estenda se a escrita for reroteada (plano 09); a lei é ícone só com escrita certa.
14. `registo_privado` e `InspectorPrivado` já não são precisos: retire.
15. Filtro por tag mede 22 px contra 28 dos campos. Censo cego à lista solta sem rótulo.

**C. Provas que faltaram.**
16. Gizmo da forma: Ctrl+Z/Y, Reset, Timeline pela fila real, cartão preso, gradient handles, parâmetro animado. `the_vector_panel_crosses_the_display_frontier` e irmãos não correram depois de mover os escritores para `ph2d-vec-entities/src/campos_transform.rs`.
17. Weight e Ctrl+Z: cartões de componente um a um, Bones à volta do Ctrl+Z, desfazer que tira cartão de verdade, e a chamada `depois_do_restauro` em `undo_app.rs` (só a medição a prova: tente um teste sem placa).
18. Scale: Ctrl+Z na shell. Vector: mutantes `mark_card_field`, tinta do papel à vista, nome longo cortado, «+» sempre aceso, recado do ícone, largura da lista. Censo: `controlo_curto` sobrevive. Bones: Reach + Handles sem prova por gesto. `the_flow_is_visible…`: teste novo.
19. Custo por quadro dos presos: medido com 5 cartões em 2 objectos (+0,70 ms); meça com 5 objectos (estimados ~0,95 ms, não medidos).
20. Fotos em falta na janela real (a maioria é do arnês): voo, solto, popover, scrub. Pelo menos uma passada em janela real por família: Inspector, Vector, Painter, Hierarchy, Timeline. Mask do Painter, lista aberta de Frames, Audio Editor com clip.

**D. Decisões que o dono ainda não respondeu: não pergunte, mantenha o que está e liste no handoff.** Regra 6 (Per-corner sem código) · «Behind Parent» · legendas sobre os chips da barra · Inspector vazio ao lado do Quadro · Size/Wrap width do texto em metros · proposta da Galeria (17) · e as de [10](../10_TODO_O_APP_TRACKER.md) e [18](../18_REVISAO_DE_CAPRICHO.md). Nomes crípticos em dúvida: `Jetty`, `+M`, `DTCG`, `Offset (1/n)`, `Cross`, `Pick Shapes`, `Smooth wash body (off · on)`.

## 5. Fechar a linha

Protocolo: `/pd-linha-fechar` (DIRETRIZ §1.5.9). ⛔ Sem suíte completa nem `nextest-impacted`: são do integrador.

1. `git rebase main` (`main` andou). Conflito em `Cargo.lock` ou gerado: regenere. Em código fora da linha: é a GEF 22; resolva por índice e compare. `python3 scripts/schema-recount.py` confirma o degrau.
2. A bateria, o barato primeiro: `cargo fmt --all` · `PH2D_SUITE=fecho bash scripts/ph2d-run.sh cargo clippy --workspace --all-targets -- -D warnings` (nunca correu nesta jornada: espere trabalho) · `check-standalone-optional.sh` · `check-workflow-packages.sh` · `cargo machete` (a linha MOVEU código e criou crates: `ph2d-host-presos`, deps de dev de fotos) · `shells/desktop/tests/it/file_loc_caps.rs` · portões por nome da §6 · alarmes de pincel · auditoria com 2 lentes.
3. Limpezas: `project-memory/MEMORY.md` está em **20 286 bytes, alvo ≤ 17 KB**: comprima no `reference_topic_*` da secção, um ponteiro por ficheiro · `docs/CartoesUI/maquete` tem 303 ficheiros e 41 MB: tire as fotos intermédias que nenhum doc cita, sem apagar as citadas · `*.log` soltos · marque a tabela do tracker (várias linhas ainda ⬜, com o registo feito) · `CLAUDE.md §5.1`: entrada CartoesUI (≤ 700 bytes), §5 ≤ 14 000 bytes, tecto de 40 KB.
4. **Handoff de integração** `HANDOFF_INTEGRACAO_NovaUI_<data>.md`, técnico: commits, foundational tocado, crates novas, catracas que subiram (com medida: alturas de `vector` 250→260, `painter_layers` 1618→1670, `onde_comeca_o_valor` piso e declarações, testes movidos), ids e i18n, contratos §6 tocados (nenhum, confirme; `VectorScene::is_recording`/`extend_record` e `Transform::de_autoria` são acrescentos), risco de colisão, portões, «o que só a integração apanha» (clippy e suíte nunca rodaram; 7 testes pesados sob carga; `no_magic_numeric` alternou uma vez sob edições alheias), e o **Smoke do dono**, em passos numerados, com `cd`, sem jargão, em português, do que ele ainda não viu, mais a lista D.
5. Último passo: `bash scripts/agent-loop-profile.sh`, `rm -rf target/*/incremental`, binário `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` duas vezes com a 2.ª saída no handoff.
6. `SendMessage` a `ph2d-44`: caminho do handoff, tip, «linha pronta». E pare.

## 6. Como trabalhar sem exaurir a janela

**Delegue quase tudo:** esta janela anterior chegou a ~450 mil tokens por passo. A sua decide, lê os relatórios, olha uma foto por família, e despacha.

- Ajudantes `general-purpose` para fatias: comece o prompt por «leia `docs/CartoesUI/19_BRIEFING_DOS_AJUDANTES.md` e siga-o», e diga só a fatia. `verificador` para portões e binário. `explorador` para perguntas de código. `documentador` para tracker, memória e rascunho do handoff.
- Paralelo: 3 a 5 ajudantes por vez, em crates disjuntas, numa árvore e um `target` só. O hook dá 45 min a cada um: fatias de ~35 min. Ajudante caído por rede retoma-se por `SendMessage`, com os avisos do que confirmar.
- **Audite os relatórios: recuse atalhos** (`#[ignore]`, catraca subida, isenção, fila que contorna causa, piso baixado, «vermelho alheio» sem ler): nesta jornada cada uma destas escondeu algo real. Foto desmentiu teste verde muitas vezes; censo verde não prova capricho.

**Portões por nome** (crate certa): `ph2d-editor-core`: `no_magic_numeric`, `hr12_widgets_a11y`, `every_widget_file_wires_a11y`, `the_foundation_modules_form_a_dag`, `the_shell_only_shrinks`, `architecture_panel_loc_cap`, `todo_campo_pintado_com_losango`, `node_id_collisions`, `no_slug_is_declared_in_two_places`, `nenhuma_isencao_sobrevive`, `every_frame_goes_through_the_theme_door`, `the_painted_control_reaches_a_consumer`, `architecture_widget_mod_in_sync`. `ph2d-panel-registry-init`: `lei_da_compactacao` (+ ritmo e texto), `o_zero_tem_populacao`, `quantas_entradas_tem_cada_painel`, `nenhum_nome_por_cima_do_controlo`, `onde_comeca_o_valor`, `a_marca_tem_a_altura_da_linha`, `toda_dica_dos_cartoes`, `um_seletor_de_cor`, `todo_painel_com_id_derivado_esta_nomeado`, `nenhum_rotulo_do_app_pinta_nada`, `o_alfinete`. Contratos: `architecture_contract_surface` (`ph2d-nodegraph`), `architecture_vector_contract_surface` (`ph2d-vector-doc`), `architecture_tool_contract_surface` (`ph2d-editor-core`), `is_capped` (`ph2d-imageio`). Pincel, em `ph2d-tool-painter`: `a_largura_de_fabrica_e_a_aprovada`, `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`.

**Armadilhas medidas nesta jornada:**
- Em `zsh`, passe crate e nome como argumentos separados; listas de caminhos em `bash` com `mapfile` e array.
- Commit por caminho leva linhas alheias em ficheiro partilhado: `git diff -- <ficheiro>` antes; ficheiro NOVO exige `git add`.
- Mutação: pasta própria com `mktemp -d`, nunca nome fixo em `/tmp`, nunca sob `timeout` externo, pela `ph2d-run.sh`; confira `git status` no fim.
- Filtro que não casa dá verde falso: confira a contagem. Teste de foto é arnês, não janela real. Grep com a palavra `kwin` num comando dispara o guarda da placa.
- Cópias do dono em `target/smoke-dono/`: nunca sobrescreva, ele pode tê-las abertas («Área de texto ocupada»); use nome novo.
- ⛔ `kill` do pai de um órfão = logout do dono; nunca `spectacle`; nunca `CARGO_TARGET_DIR` partilhado.
