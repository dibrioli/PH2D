# HANDOFF de continuação — line/UIUX, F42 (01/11) → F43

Worktree `Worktrees/line-UIUX`, branch `line/UIUX`, HEAD de código `c300ff13eb` (F42), merge-base com o
main `422fec2027` (o main não andou no início da sessão). 38 commits sobre o main até `c300ff13eb`
(37 antes da F42). **Nada integrado, nada enviado.** A F42 passou no **smoke do dono em 01/11: OK**.
Antes de ler: `cd` + `pwd` + `git branch --show-current`. Handoff anterior:
[F41](HANDOFF_CONTINUACAO_line_UIUX_2026-11-01_F41.md).

## 1 — O que a F42 fez (commit `c300ff13eb`)

Pedido do dono (01/11): «cada tipo de objeto na hierarchy tenha um ícone único, bem representativo de
sua classe e com cores diferentes». Nota técnica (paleta, contrastes mínimos, ΔE):
[F42_icone_e_cor_por_tipo.md](../F42_icone_e_cor_por_tipo.md) — não repetida aqui.

- **18 tipos CONTADOS no código** (8 `ObjectKind` + componentes definidores do Add/registo):
  `ObjectType` em `crates/ph2d-editor-core/src/hier_types.rs` — glifo exclusivo, token `obj-*`,
  marcador, precedência por `ALL`. Player ganha a todos; Camera/Light/Macro/Particles/Sound/HUD/Joint/
  Body/Collider só num vazio.
- **A cor sai do glifo** (`ObjectType::of_icon`): nenhum campo novo na linha. `ph2d-panel-hierarchy`
  `row.rs` `icon_color`: cor do tipo também seleccionado, `text-disabled` escondido, Cena/conteúdo
  intactos.
- **18 tokens `obj-*`** (`ph2d-tokens`, `docs/design/tokens.json`), cores de DADO;
  `type_colour::legible_on` empurra L até 3:1 só onde a emprestada não lê (Plumber 2,09/1,94 → 3,0).
- **5 glifos originais** novos: body, hud, joint, pixel-sprite, player (`docs/design/icons/*.svg`,
  `icons.rs`).
- **Painted antes de gravar**: `ModeFamily::object_types`; o Painter declara `Painted` para
  `document_entities()` (a lista que o save carimba) ⇒ o ícone não muda ao gravar nem ao sair do Paint.
- De passagem: vazio com `ScrollFactor` deixa de ler-se como câmera (só `GameCamera` faz câmera).
- **Shell cresceu ~6 linhas** (uma chamada de composição em `fase_object_add.rs` + a busca do tipo em
  `snapshots_hierarchy.rs`); `the_shell_only_shrinks` segue verde.

## 2 — Gates e prova de fecho

Gates novos (6): `cada_tipo_tem_icone_unico`, `cada_tipo_tem_cor_propria_e_legivel`,
`o_tipo_vem_do_marcador` (`ph2d-editor-core`, `hier_types_tests.rs`);
`o_icone_do_objecto_pinta_a_cor_do_tipo` (`ph2d-panel-hierarchy`, `row.rs`);
`the_row_type_comes_from_the_registered_markers` (`ph2d-app-components`, `row_components.rs`);
`an_image_with_a_painter_document_is_painted_in_and_out_of_paint` (`ph2d-app-painter`,
`paint_mode_tests.rs`).

- **Mutação 10/10 vermelhas** (9 pelo agente de mutação + a da sonda de mundo, depois de lhe ter sido
  acrescentado o teste).
- Fecho da onda: `fmt --all` limpo · `clippy -D warnings` limpo em 7 crates (`ph2d-tokens`,
  `ph2d-editor-core`, `ph2d-panel-hierarchy`, `ph2d-app-components`, `ph2d-app-painter`,
  `ph2d-app-vec`, `ph2d-host-desktop`) · 17 testes nomeados verdes · binário `smoke` compilado.
  **Sem suíte completa** (é do integrador).

## 3 — Smoke (o dono fez, 01/11: OK)

Cena nova `PH2D_OBJECT_MODE_SMOKE=42` (cenas agora `1|2|4|6..42`; implementação em `smoke_tipos.rs`
+ a 42 do vetor = um rectângulo). Foto: `img_F42/cena42_02.png` (+ `cena42.png`, `cena42.log`).

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=42 ./target/smoke/ph2d-host-desktop`
2. Esperar ~2 s: a Hierarchy mostra 18 linhas, cada uma com ícone e cor próprios.
3. Seleccionar uma linha: o ícone mantém a cor do tipo. Esconder (olho): o ícone vai a cinza.
4. O objecto Canvas segue com o pincel rosa ao entrar/sair do Paint (`Tab`) e depois de gravar.
5. Errado se: dois tipos com o mesmo ícone ou cor; o ícone muda de tipo ao gravar ou ao sair do Paint.

Nota: fotos em tela virtual não clicam; os cliques estão provados por testes + o smoke do dono.

## 4 — ABERTO

- **(a) Costura achada, NÃO corrigida (fora do escopo, pré-existente)**: o Painter
  `document_entities()` inclui `bound_doc` mesmo depois do teardown ⇒ uma imagem aberta uma vez em Paint
  e nunca pintada fica carimbada `PaintedDoc` ao gravar. A F42 faz o ícone concordar com isso (Painted
  desde que existe documento do Painter): consistente, mas convém saber ao mexer no save do Painter.
- **(b) Integração da linha**: só por ordem do dono (38 commits sobre o main; o main pode ter andado —
  `collision-surface.sh` pelo caminho absoluto). Degrau do schema: `python3 scripts/schema-recount.py
  <valor no main>`; `PROJECT_SCHEMA` = **211** (`shells/desktop/src/project_schema.rs:556`), a F42 não o
  mexeu.
- **(c) Ficheiros não rastreados** (`.log`/`.png`) em `img_F30/F33/F34/F40/F41`: não são da F42; não
  comitados.
- **(d) Fora de escopo por ordem do dono**: recuo da Luz 2D sob blocos pequenos
  ([BUGS_luz2d.md](../../Luz2D/BUGS_luz2d.md) §8).
- **(e) PRÓXIMO ITEM, F43** (dono, 01/11, verbatim): «próxima: Seções de Effects e Filters de Vector:
  usar layout figma. Nada de todos botões à mostra. Mas sim dropdown, lista de efeitos, Propriedade dos
  efeitos abre na lateral esquerda.»

### F43 — referências (comitadas) e notas

- `img_F43/ref_hoje_botoes_a_mostra.png` — hoje: a secção «Effects» do Inspector do Vector mostra ~20
  botões «Add X» (Trim Path, Zig Zag, Repeater, Pucker & Bloat, Arc / Arc Upper / Lower, Bulge, Flag,
  Wave, Squeeze, Fisheye, Rise, Falloff Radial/Linear/Rect/Sweep, Twist, Knot, Sketch, Hatch) e
  «Filters» com Add Blur / Outer Glow / Drop Shadow…
- `img_F43/ref_figma_dropdown_e_lista.png` — Figma: cabeçalho «Effects» com «+»; o «+» abre um menu
  (dropdown) com os tipos; os efeitos aplicados são LINHAS com ícone + nome, olho (liga/desliga) e «−»
  (remove).
- `img_F43/ref_figma_propriedades_a_esquerda.png` — Figma: clicar numa linha abre um painel flutuante à
  ESQUERDA do Inspector, com o nome do efeito como título em dropdown, os campos e um X para fechar.

Notas:
- **Contar no código** todos os efeitos E todos os filtros (secções de `ph2d-panel-vector`); os ~20 da
  foto são o que se vê, não a lista.
- Mesmo padrão para Effects e Filters.
- HR-15: tokens/i18n, zero hex, UI só em inglês. Manter os losangos de Auto Key nos campos.
- Maquete/foto ANTES de pedir aprovação ao dono (`docs/Components/ferramentas/fotografa_cena.sh`).
- A memória `project_painter_panel_e10_owner_decisions_2026_11_01` («padrão do Inspector») é do Painter:
  **conferir** se aplica, não assumir.
- Ao tocar a shell: a catraca `the_shell_only_shrinks` — compor a partir de crate, não crescer.

## 5 — Números contados

- Tipos de objecto da Hierarchy: **18**; glifos novos: **5**; tokens `obj-*`: **18**; gates novos: **6**.
- `PROJECT_SCHEMA` = 211. Cenas `PH2D_OBJECT_MODE_SMOKE`: `1|2|4|6..42`.
- Commits da linha sobre o main: 38 em `c300ff13eb` (mais o deste handoff).

## 6 — Prompt de continuação (janela seguinte)

```
/pd-linha-assumir line/UIUX — Worktrees/line-UIUX
Próximo item: F43.
Handoff: docs/UI_New_and_Simple/handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-11-01_F42.md

F43 (pedido do dono 01/11): «Seções de Effects e Filters de Vector: usar layout figma. Nada de todos
botões à mostra. Mas sim dropdown, lista de efeitos, Propriedade dos efeitos abre na lateral esquerda.»
Referências: docs/UI_New_and_Simple/handoffs/img_F43/ (hoje × Figma).

- Conte no código TODOS os efeitos e filtros do Vector (não escolha de cabeça).
- Cabeçalho da secção com «+» que abre a lista; efeitos aplicados como linhas (ícone, nome, olho, «−»);
  clicar numa linha abre as propriedades num painel à ESQUERDA do Inspector.
- Mesmo padrão para Effects e Filters; tokens/i18n (zero hex, só inglês); Auto Key nos campos.
- Antes de pedir aprovação, monte cena de smoke e me mostre a foto (fotografa_cena.sh).

Feche com /pd-onda-fechar e PARE no meu smoke, em passos numerados, com a cena fotografada; corra
você mesmo o smoke antes e me diga o resultado. Handoff e prompt da janela seguinte só DEPOIS do smoke.
⛔ Não integre nem faça push: só por ordem minha.
```
