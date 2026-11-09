# HANDOFF de continuação — line/ScriptEditor, 09/11: a Onda B (a Bancada) está FEITA; smoke do dono PENDENTE

> Leitor: a LLM da próxima janela. Anterior: [`HANDOFF_CONTINUACAO_line_ScriptEditor_2026-11-09_ONDA_B.md`](HANDOFF_CONTINUACAO_line_ScriptEditor_2026-11-09_ONDA_B.md).
> Decisões técnicas da onda (não repetidas aqui): [`13_plano_bancada.md`](../13_plano_bancada.md) «#### Onda B — construída (09/11)».
> Decisão do dono 12 (espiar a cena + mapa, ambos ligados): [`00_decisoes_do_dono.md`](../00_decisoes_do_dono.md).
> Último handoff de integração: [`HANDOFF_INTEGRACAO_line_ScriptEditor_2026-11-08_ONDAS_A_C_D.md`](HANDOFF_INTEGRACAO_line_ScriptEditor_2026-11-08_ONDAS_A_C_D.md).

## 1. Estado

- Branch `line/ScriptEditor`, base `f254e42bf1` (= `main`; nada a rebasear em 09/11).
- Commits da onda, por cima de `81ebfee256` (handoff anterior): `e7778b6336` (código) e `faed3f4e7a` (fotos da cena 13).
- Onda B (itens 6–10) **construída**. **Smoke do dono PENDENTE.** Não integrada, não pushada.

## 2. O que existe agora

- **In this script:** Properties · Functions · Hears com `ln N`; marcas ◆/⚡ na margem, pontos na cor de cada objecto
  que difere do padrão e o balão «em quem» (pairar abre, clicar prende, `Esc` fecha).
- **Who has what:** grelha dobrável. A célula edita pela MESMA `InspectorComponentEdit` do Inspector; com a Bancada em
  foco o `Ctrl+Z` da aba é o do projecto.
- **Where:** mapa com o contorno de cada objecto que corre o script. **Frase do pé** fixa.
- **Espiar a cena:** o rato parado num nome de objecto cede a área à cena com o contorno dele.
- **Menu View** (`Peek at Scene`, `Scene Map`), ambos ligados por omissão (decisão 12).

## 3. Provas (todas verdes em 09/11)

Testes novos:
- `ph2d-panel-code`: `tests/it/bancada.rs` + `bancada/puras.rs` (18).
- `ph2d-code-bancada`: `tests/it` (quem, acto, seguir; 7 com o de `names`).
- `ph2d-code-syntax`: `tests/it/ouvidos.rs` (3).
- `ph2d-app-components`: `script_inspector_tests.rs` `a_celula_da_grelha_e_a_mesma_edicao_que_o_inspector_aplica`.
- `ph2d-app-code`: `tests/it/onda_b.rs` (2).

Crate inteira (código movido): `ph2d-app-code` 254 ✓ (3 ignorados) · `ph2d-panel-code` 57 ✓ · `ph2d-code-bancada` 7 ✓.
Clippy `-D warnings` das 13 crates tocadas ✓.

Portões por nome ✓: `the_shell_only_shrinks`, `no_dependency_climbs_a_layer`, `no_magic_numeric`, `no_literal_color`,
`hr15_no_hardcoded_ui_strings`, `every_member_inherits_the_workspace_lints`, `doc_surface`,
`stack_versions_doc_matches_the_lockfile`, `workspace_src_files_under_loc_cap`,
`the_app_does_not_grow_past_its_numbered_ceiling`, `hover_outline`, `a_frame_request_has_a_listener`, `placa` (8),
`i18n` (10), e os que leem `hero/paint.rs` e `fase_vector_overlays.rs`.

**Impactados e suíte completa ficam para a integração.**

Binário `--profile smoke` compilado 2× na worktree; a 2.ª saída: `Finished \`smoke\` profile [optimized] target(s) in 0.41s` (zero Compiling).

## 4. Lição da onda: uma caixa que une duas coisas rouba o clique de uma

A marca da margem e os pontos do fim da linha eram UMA caixa (`union`): o clique no TEXTO da linha da `ph2d.property`
ia para a marca; 6 testes (renomear ×5, lista ×1) reprovaram. Cura: dois alvos (`marks.rs`); o teste
`a_marca_da_propriedade_vai_da_margem_aos_pontos_de_quem_mudou` passou a exigir que o meio do texto NÃO seja alvo.
Na lista, com a frase de recusa (`notice`) o «In this script» cede a coluna (`surface.rs`).

## 5. Para o integrador (superfície de colisão)

- **Contrato novo, append-only:** `DocumentSurface::yields_to_scene` (padrão `false`) e `Documents::paints_document_ground` (`editor-core`).
- **Crate nova** `ph2d-code-bancada`: linha nova em `TETOS_DAS_CRATES_DA_FAMILIA`, `1_028 // 928`.
- **Código MOVIDO entre crates** (nenhum portão cita os caminhos; verificado):
  - `ph2d-app-code/src/scene_names.rs` → `ph2d-code-bancada/src/names.rs` (+ `object_name`);
  - `ph2d-app-code/src/cards.rs` → `ph2d-panel-code/src/cards.rs`;
  - `FieldKey`/`field_key`/`printable` de `ph2d-app-code/src/edit.rs` → `ph2d-panel-code/src/field.rs`.
- **Shell** (`git diff --stat f254e42bf1 -- shells/desktop`: 5 ficheiros, +51 −4): `+ bancada_scene.rs` (37), `fase_pointer_subjects.rs` (+1 chamada),
  `fase_vector_overlays.rs` (o contorno do espiar), `main.rs` (+1 `mod`), `Cargo.toml` (dep `ph2d-code-bancada`).
  Nenhum campo novo na `App` (o tecto de campos reprovou com um; a cor do espiar lê-se do recurso ao desenhar).
- **Tectos de LOC agora** (medidos): `ph2d-app-code` 26 517 · `ph2d-code-viewmodel` 302/304 (⚠️ 2 linhas de folga) ·
  `ph2d-code-scenes` 660 (plano falava em 634/666: confira o tecto na tabela) · `ph2d-code-bancada` 926/1 028.
- **Cena de smoke:** `PH2D_CODE_SMOKE=13`, roteiro `STEPS_13` (`PH2D_CODE_SMOKE_PASSO=1..4`). Scripts em
  `docs/CodeEditor/ferramentas/smoke/bancada_{player,coin,door}.luau`; fotos em `docs/CodeEditor/ferramentas/smoke/cena13/`
  (`passo_0..4.png` e `maquete_B_lado_a_lado.png`).
- **Diferença à maquete que fica:** na maquete o balão empurra as linhas de baixo (vista embutida); aqui é balão por cima,
  encostado por baixo da linha, a partir do texto.

## 6. Aberto

1. **Smoke do dono** da cena 13.
2. A grelha não aceita vírgula decimal num Vec2 (`"1,5, 2"` → nada); num Number aceita.
3. O hook `.claude/hooks/prazo-do-agente.sh:55` recusa toda linha de subagente com a palavra «bancada» (feito para outro
   banco pesado) e colide com o nome da crate nova; os subagentes usaram uma variável. Estreitar o padrão é decisão do processo.
4. O §5.1 do `CLAUDE.md` passa a `PH2D_CODE_SMOKE=1..13` só no fecho da linha.
5. Memória nova `project-memory/project_bancada_peek_and_map_both_on_2026_11_09.md` + linha em
   `reference_topic_owner_product_decisions.md` foram escritas no checkout PRIMÁRIO (o symlink da memória), **não nesta
   worktree**: o integrador leva-as.
6. Os abertos herdados do §6 do [handoff anterior](HANDOFF_CONTINUACAO_line_ScriptEditor_2026-11-09_ONDA_B.md) continuam (não tocados).
7. `backup/line-ScriptEditor-2026-11-08`: apagar quando a Onda B integrar.

## 7. Próxima onda

No [plano 13](../13_plano_bancada.md) as Ondas A, C e D estão no `main` e a B está feita aqui: **não falta nenhuma onda**.
Depois do smoke do dono, a linha está pronta para `/pd-linha-fechar` (handoff de integração, §5.1 do `CLAUDE.md`). Não integre, não pushe.

## Smoke do dono 09/11 — passo 3 (o balão «em quem»): três defeitos, curados

O dono: «o cartão tem pouco contraste com o fundo, fica longe do losango e ao tentar levar o mouse pro cartão o
cartão some». Cura (`ph2d-panel-code/src/bancada/popover.rs`, `mod.rs`):
- **Contraste:** a moldura e o botão passam a ser os PADRÃO da casa (`paint_popover`, `Button::on_elevated` +
  `paint_button` — os do balão do código, `hint.rs`), com tinta `Text1/2/3`; antes era a tinta do cartão
  (`CardField` sobre `CardField`: o fio da borda sumia).
- **Longe:** o balão encosta-se ao canto de baixo à direita da marca que o ABRIU (`Pop::anchor`; a marca ◆ ou os
  pontos), em vez do início do texto; a coluna das marcas das linhas de baixo fica à vista.
- **Some no caminho:** espera `POP_HIDE_NS` = 300 ms (o `editor.hover.hidingDelay` do VS Code, o mesmo do balão do
  código) com o rato fora dos dois; `tick` fecha-o sem o rato se mexer.
- Testes: `o_balao_encosta_na_marca_e_o_caminho_ate_ele_nao_o_fecha` (novo) e os dois de fecho com a espera;
  `ph2d-panel-code bancada` 19 ✓, `ph2d-app-code onda_b` 2 ✓, clippy das duas ✓. Foto nova: `cena13/passo_1.png`.

## Smoke do dono 09/11 — resto OK; ABERTOS: passo 5 (Ctrl+Z) e os valores ao vivo a piscar

- **Passo 5, «Undo não funciona».** Causa 1 (curada, `surface.rs` `fn key`, ramo `undo_key`): com a aba activa o
  Ctrl+Z ia ao texto; com a grelha em foco a aba agora NÃO o consome (teste `onda_b`). Mas o Ctrl+Z real ainda
  não desfez na foto. **Hipótese forte, por provar:** a cena 13 corre em Play, e `motivo_da_supressao`
  (`shells/desktop/src/undo_app_macros.rs:50-64`, «jogo a correr») não regista passos com o jogo a correr — a
  edição da grelha (e a do Inspector) durante o Play não entra na fila. Provar: `PH2D_UNDO_LOG=1` com a cena
  PARADA vs a correr. Rumo provável: a cena 13 nasce parada; se o dono quiser desfazer durante o Play, é
  decisão dele (a casa trata o Play como prévia). ⚠️ Na foto o `xdotool` (agora XTEST, gancho `FOTO_TECLAS`
  em `docs/Components/ferramentas/fotografa_cena.sh`) não mostrou `[undo] Ctrl+Z RECEBIDA` no log: confirmar
  primeiro que a tecla chega à janela virtual.
- **«Os números piscando rápido como ×2 no fim das linhas»** — os valores ao vivo da Onda D (D18) mudam a cada
  quadro: ilegíveis. Cura: amostrar poucas vezes por segundo (medir o ritmo; um token de duração da casa).
