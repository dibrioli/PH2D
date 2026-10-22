# HANDOFF DE CONTINUAÇÃO — line/CodeEditor, item 8a: arrastar objectos para o código (21/10/2026)

> Leitor: a próxima LLM da linha (e, depois, o integrador). **Não integrado, não enviado.** O último handoff de
> INTEGRAÇÃO da linha continua válido: [I7 a posse](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-20_ONDA2_I7_A_POSSE.md);
> este vem por cima. **Smoke do dono: PENDENTE** (§6).
> Plano e oráculo: [07_arrastar_para_o_codigo.md](../07_arrastar_para_o_codigo.md) (§3 = divergências do Godot) ·
> fila: [03_onda2.md](../03_onda2.md) item 8 · decisões do dono nº 4 (15/10) e **nº 8 (21/10)**:
> [00_decisoes_do_dono.md](../00_decisoes_do_dono.md).

## 1. Identidade

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor` · branch `line/CodeEditor` · HEAD
  `652730fb20` (antes do commit deste doc) · merge-base com o `main` `100d34c8e0` (= main).
- Oráculo: Godot 4.7.2, arrastos reais numa Xvfb + sonda `sonda.gd` (EditorPlugin) em
  `ferramentas/oraculo_arrastar/`; duas corridas iguais (21/10 22:07Z e 22:13Z).
- Decisão nº 8 (21/10): os scripts aprendem componentes/imagens JÁ — fase **8d antes de 8b**.

## 2. Commits do 8a (`git log 4ef5ea55f4..HEAD`)

- `89433a58a4` oráculo + plano · `62cae46140` decisão 8.
- `c0ffcbc0bb` `ph2d.object(nome) -> Entity` (`api_table`, `api_help`, lane, `ph2d.d.luau`, schema do MCP).
- `9cc483f79d` a lei: `ph2d-app-code/src/dropping.rs` (+ factos em `ph2d-code-syntax`: `object_references`,
  `top_function_body`, `ObjectCall`/`ObjectRef`).
- `7ca1f62ea5` o fio: `DragPayload::Object`, `begin_named_drag`, `hierarchy_drag_in_own_panel`,
  `DocumentSurface::drop_hover/drop`, `surface_input::{drop_hover,drop}`, `asset_drop` (`DropTarget::Code`,
  `DropAction::WriteCode`), `ph2d-app-components/src/object_drag.rs`, `ph2d_ecs::list_order` (saiu da lane de
  script), `ph2d-app-code/drop_doc.rs`, `ph2d-panel-code/drop_mark.rs`, `tests/it/arrastar.rs`.
- `41fbe66ec7` nomes da cena: `scene_names.rs` (lido por quadro, sem `Spawned`); lint `ObjectLint`
  (`unknown_object(_near)`, `object_twice`, `unknown_master(_near)`) em `ph2d-code-explain` + i18n; sugestões
  dentro das aspas de `ph2d.object`/`find_by_name`/`spawn` (só o resto do nome); Ctrl+clique `Dest::Object`
  selecciona; `object_rename.rs` chamado pelo commit de renomear (`hierarchy_rename.rs`) reescreve biblioteca +
  abas abertas/estacionadas com UM `UndoLink` (`undo_links.pending`); `link.rs` ao desfazer percorre TODOS os
  scripts do link.
- `ec4c6d83e4` cena `PH2D_CODE_SMOKE=7` (`code_smoke::monta_arrastar`, `ARRASTAR_LUAU`, `CENAS=7` nos dois
  crates) + cura: Ctrl+soltar em linha EM BRANCO saía sem recuo (agora das vizinhas, `indent_for`).
- `ba0b39d682` corrida conduzida na app real: `ferramentas/conduz_arrastar/corre.sh` (Xvfb, xdotool): Ctrl+soltar
  «Door» → `  local door = ph2d.object("Door")` na linha 10, Hierarquia intacta; fantasma com o nome, cursor fino,
  balão «Drop: writes ph2d.object("…") here. / Hold Ctrl: a local variable on this line.»; Ctrl+Z desfaz a
  soltura num passo.
- `652730fb20` o portão da cena 7 sai dos testes da shell para
  `ph2d-app-components/tests/it/a_cena_7_do_editor_ensina_o_que_diz.rs` (dev-dep `ph2d-app-code`; `Cargo.lock`).

## 3. A lei (divergências declaradas do Godot; mecanismo em 07 §3)

- Referência = `ph2d.object("Nome")` (Entity não-opcional, erro com o nome), por NOME e não por caminho.
- Ctrl dentro de função = `local <snake> = ph2d.object(...)` no INÍCIO da linha da soltura, com o recuo dela.
- Ctrl no topo = `self.<snake> = ...` no início do corpo de `init(self)`, criando `function init(self) … end` se
  faltar (Luau no topo corre ao carregar, fora da cena; `init` é o `@onready` do Godot).
- Vários objectos = a selecção, na ordem da lista. Renomear REESCREVE (Godot parte em silêncio); o editor AVISA
  (Godot não); Ctrl+clique selecciona (Godot não faz nada).
- Curas pelo caminho: (1) largar uma linha da Hierarquia EM QUALQUER SÍTIO da janela reparentava
  (`find_hierarchy_drop` só olhava `cursor_y`) — agora só dentro do painel de origem
  (`WidgetStore::hierarchy_drop_in_own_panel`, também governa o indicador); (2) recuo em linha em branco.

## 4. O que um MERGE pode partir

- `CENAS` 6 → 7 nos dois sítios (`ph2d-app-components/src/code_smoke.rs`, `ph2d-app-code/src/smoke.rs`); CLAUDE.md
  §5.1 passa a `PH2D_CODE_SMOKE=1..7` na integração (a linha não o editou).
- `ph2d_ecs::list_order` mudou de casa; `DocumentSurface` ganhou `drop_hover/drop` (foundational, append-only).
- **Catraca `the_shell_only_shrinks`: shell em 189 032 linhas contra o tecto 189 041 — 9 linhas de folga.** Toda
  linha nova na shell obriga a MOVER código.
- Dep nova `ph2d-app-components` (dev) → `ph2d-app-code`; `Cargo.lock`.
- A recontagem da superfície de colisão no fecho: `bash /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor/scripts/collision-surface.sh`.

## 5. Provas (por nome, todas verdes)

`ph2d-script`: `objecto_pelo_nome`, `a_api_e_a_tabela` (6), `toda_entrada` · `ph2d-code-syntax`: `objecto`,
`corpo_da_funcao` · `ph2d-app-code`: `dropping` (7), `arrastar` (6, integração), `scene_names`, `object_rename`,
`renomear` (16), `explicacoes` (3), `problemas_lista` (8) · `ph2d-code-explain`: `censo` (6) · `ph2d-app-components`:
`um_objecto_so_se_larga`, `object_drag`, `os_scripts_da_cena_saem_limpos_do_formatador`, `ctrl_soltar_a_porta` ·
`ph2d-editor-core`: `the_shell_only_shrinks`. Suíte inteira, clippy, mutação e auditoria: NÃO corridos (fecho).

## 6. Smoke do dono (cena 7) — fica para o handoff de integração

1. No terminal: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && PH2D_CODE_SMOKE=7 cargo run -p ph2d-host-desktop --profile smoke`
2. O script «arrastar.luau» já está aberto. Segure Ctrl e arraste «Door», da Hierarquia, para a linha vazia debaixo do comentário ①. Aparece a linha `local door = ph2d.object("Door")`. Ctrl+S e clique na aba «Cena»: a porta castanha sobe e desce.
3. Volte à aba e siga o ②: escreva `ph2d.set(`, arraste «Gold Coin» SEM Ctrl para logo depois do parêntese, termine com `, "rotation", tempo)` e Ctrl+S: a moeda gira.
4. Na Hierarquia renomeie «Door» para «Gate»: o script muda sozinho. Ctrl+Z: o nome e o script voltam juntos.
5. No texto troque "Door" por "Dor": aparece um aviso amarelo «quis dizer "Door"?».
6. Ctrl+clique no nome "Door" no texto: a porta fica seleccionada.

Deu errado se: o objecto muda de lugar na Hierarquia ao largá-lo no script · o texto entra sem recuo · Ctrl+Z
precisa de mais de um toque · o aviso não some quando o nome está certo.

## 7. ABERTO (por ordem)

1. **8d** — scripts lêem/escrevem toda propriedade que o Inspector mostra + imagens. Precisa de plano próprio e
   oráculo Godot (`get`/`set`/`get_property_list`). Factos: hoje não há tabela de reflexão; 33 cartões `populate_*`
   à mão; `ph2d-component-desc` tem 107 `ComponentDesc`/`FieldDesc` (ids + `FieldKind`) sem ligação aos valores;
   `Health` tem max/start, a vida actual é `HealthNow` (derivada, não registada); `GameCamera` tem `height_world`,
   não zoom; sprites referem imagens por `SpriteSource{Atlas/Individual/CookedTexture LogicalTextureId [u8;32]}` e
   os assets são endereçados por conteúdo, sem nome único.
2. **8b** — cartões de asset → nome do mestre em string / Ctrl local; lista de scripts → `"x.luau"`; propriedade
   do Inspector → o seu nome em string. Hoje Prefab/Imagem largados no código = `Refuse`.
3. **8c** — Alt → `ScriptValue::Object(u64)` no FIM do enum + campo no Inspector. Hoje Alt = sem modificador.
4. **`/pd-linha-fechar`** — gate batched inteiro, prova de mutação das leis do item 8, auditoria, handoff de
   integração, CLAUDE.md §5.1 com `PH2D_CODE_SMOKE=1..7`.
- Limites conhecidos: sugestões de nome dentro das aspas só com o `luau-lsp` Ready (a lista abre pela resposta do
  assistente); `ph2d.object` só dentro de hooks (como `find_by_name`).

**Smoke do dono da cena 7: OK (22/10).**
