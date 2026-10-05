# 06 — Tipos de objecto e modos de edição por objecto (plano, 2026-10-03)

> **Pedido do dono, 2026-10-03:** *«Precisamos de um sistema de separação dos modos de edição de
> cada objeto como faz o Blender. A primeira coisa é criar os tipos de objetos a partir do + da
> hierarchy […] Já é possível criar Sprites com Ctrl+N. Isso precisa ir para o + da Hierarchy. Assim
> também os objetos Flip, Vector, Sculpt, Model […] E para cada objeto devemos ter como no blender
> modos de edição.»* — e *«antes de implementar, estude apps similares e planeje»*.
>
> Leitor: a janela que implementa. ⚠️ **Isto é um PLANO aprovado por partes**: as escolhas de
> produto estão no §6, com a resposta do dono quando a houver. Não comece a F1 sem o §6 respondido.
>
> ⛔⛔ **05/10 — o 3D SAIU do PH2D** ([ADR-0179](../../architecture/decisions/0179-o-3d-sai-do-ph2d.md);
> código em `b1a6f9b07`). Sculpt, Model e Render já não existem: tudo o que este plano diz deles (§1–§4)
> é HISTÓRIA. Hoje: `ObjectMode` = Object · Paint · Draw · Edit; famílias com modos = Painter (Image ▸
> Paint), Flip (Draw · Edit), Vector (Edit; cada forma é um objecto desde 05/10, e o Edit é do TIPO);
> cenas `PH2D_OBJECT_MODE_SMOKE=1|4|6|7`.

## §0 — Isto já foi DECIDIDO em parte (30/08), e o plano não o re-litiga

[`00_DECISOES_DO_ENIO.md`](../00_DECISOES_DO_ENIO.md) **D3** separa **três eixos**: o **layout**
(o utilizador escolhe, nas abas de cima), o **modo** (que o TIPO do objecto declara, no cabeçalho
da área) e a **ferramenta** (o gesto, dentro do modo). As seis abas de hoje (`Draw · Vector · Flip ·
Model · Animate · Nodes`, [`task_layout.rs`](../../../crates/ph2d-editor-core/src/screens/task_layout.rs))
são **layouts**, e o **modo nunca foi construído**.

A **D6** já tem a tabela de modos por tipo, com duas correcções do dono:
- o Flip tem `Draw` próprio;
- o vetor NÃO tem `Paint` enquanto a feature não existir, porque um modo que não faz nada é um
  controlo morto.

O estudo dos manuais é [`pesquisa/04_modo_layout_e_ferramenta.md`](../pesquisa/04_modo_layout_e_ferramenta.md).
Este plano acrescenta os **oráculos corridos** e o desenho.

## §1 — O que os oráculos dizem (CORRIDOS em 03/10)

O fixture é [`oraculos/modos_e_tipos_2026-10-03.md`](oraculos/modos_e_tipos_2026-10-03.md):
Blender 5.2.2 só corrido (GPL) e Godot 4.7.2 corrido (MIT).

**Blender — o modo é declarado pelo TIPO.** O `mode_set` recusa um modo que o tipo não tem e
devolve a lista dos que tem:

| tipo | modos |
|---|---|
| malha | Object · Edit · Sculpt · Vertex/Weight/Texture Paint |
| Grease Pencil | Object · Edit · Sculpt · **Draw** · Weight · Vertex |
| armadura | Object · Edit · Pose |
| curva, texto, lattice… | Object · Edit |
| câmara, luz, empty… | **só Object** |

- **O menu Add é agrupado por TIPO de dado**, com presets por baixo (Mesh ▸ Cube/Sphere…).
- **O modo pertence ao objecto ACTIVO.**
- *Lock Object Modes* vem **ligado** de fábrica: em Sculpt, Edit ou Weight, clicar noutro objecto
  é RECUSADO.
- **Multi-objecto** só no Edit, e só entre objectos do MESMO tipo do activo; o Sculpt toma só o
  activo.

**Godot — não há modos.** O contexto segue o TIPO do nó seleccionado, sozinho:
- o ecrã 2D/3D muda pela selecção;
- o TileMap, o GridMap e o AnimationPlayer abrem um painel de baixo;
- o Polygon2D, o Line2D e o Path2D acrescentam botões à barra.

O diálogo *Create New Node* é uma árvore de herança pura: 46 tipos sob Node2D e 107 sob Node3D,
sem categorias temáticas.

⇒ **A nossa engine fica com o Blender para o MODO**, porque um modo de criação toma a tela inteira
(D3: *«toda a tela vai mudar»*). Do Godot herda-se o **contexto leve**: seleccionar um objecto em
modo Object mostra no Inspector o que é dele, sem trocar de modo. O menu Add segue o Blender (por
tipo, com presets), e não a árvore da Godot: a nossa lista tem ~10 tipos, não 150.

## §2 — O estado de hoje (medido no código, 03/10)

- **Os tipos já existem como vocabulário:** `ObjectKind` em
  [`ph2d-component-desc/src/lib.rs:336`](../../../crates/ph2d-component-desc/src/lib.rs) é
  `Empty · Image · Vector · Flip · Painted · Model3D · Sculpt3D`. Cada um é DERIVADO de um
  componente-marcador: `Sprite`, `VecPathRef`, `FlipObjectRef`, `PaintedDoc`, `FieldObject`,
  `BakedForm`. ⛔ A lei dele: o tipo é derivado e **nunca** um campo escrito à mão. O plano mantém-na.
- **O `+` da Hierarchy** cria um Empty: `shells/desktop/src/render_loop/hierarchy_add_root.rs`
  (`spawn_empty_root`). O **Ctrl+N** cria a imagem: `render_loop/fase_new_image_modal.rs`
  (`spawn_blank_canvas`).
- **Os módulos editam um documento do MÓDULO, não «o objecto seleccionado»** — medido na F0
  (03/10), módulo a módulo, no §2.1.
- **O `DrawMode` do vetor mistura os eixos:** 14 variantes, que são 2 modos (`Select` = Object,
  `Node` = Edit) e 12 ferramentas (D3, medido em 30/08). ⚠️ Eram **17** quando a F3 do Vector as
  partiu (04/10): ver §4.
- **Já existe o sítio do selector de modo:** os pulldowns do cabeçalho da área (`AreaMenu`,
  `set_area_commands`). O 3D Model publica ali a vista e o sombreamento, e desde 02/10 o pulldown
  é tão largo quanto a face mais larga que pode mostrar. ⇒ o selector *«Object Mode ▾»* é **um
  pulldown a mais** nessa fila, como no cabeçalho do Blender.

### §2.1 — A F0, MEDIDA (03/10): quem guarda «o que estou a editar»

| módulo | quem guarda | N objectos? | activo de outro tipo | pode ser «o desta entidade»? |
|---|---|---|---|---|
| **Painter** | `PainterTool::bound_doc` = bits da entidade seleccionada (`ph2d-tool-painter/src/tool/mod.rs:313`), `doc_cache` por entidade (`:316`) | sim | recusa: sem `Sprite` não pinta (`painter_canvas_input.rs:358`) | **já é** |
| **Model** | `Smoke::doc` thread-local, cozido da 1.ª raiz `FieldObject` (`ph2d-app-field3d/src/scene.rs:296`) | ⛔ **não**: uma 2.ª raiz nunca é cozida | ignora | não — a ponte tem de cozer a raiz da entidade activa |
| **Sculpt** | `Sculpt3dScene::active` (índice, `cena.rs:37`); peças com `ObjectId` estável e entidade-espelho `Sculpt3dPieceRef` (`entities.rs`) | sim | o raycast só acerta peças | quase: o activo passa a vir da entidade (`world_map`) |
| **Flip** | `FlipDoc` global; o traço cai SEMPRE no 1.º objecto (`autokey.rs:54`) | sim, mas só o 1.º recebe | descarta o traço em silêncio | sim: o `FlipEntityMap` já liga entidade↔objecto |
| **Vector** | `PenTool::selected_paths` (`ph2d-vec-edit/src/lib.rs:142`) | sim | a caneta cria um path novo sem olhar o activo | o mais caro: é a partição do `DrawMode` (D3) |

⚠️ **Duas premissas derrubadas pela medição:**
- o explorador da F0 deu o Model como «por entidade», e a leitura do `sync_scene_and_birth`
  mostrou o contrário (`q.iter(world).next()`). O menu Add oferece **um** Model por cena e di-lo
  no rótulo, até à F3;
- `ObjectKind::Sculpt3D` apontava para o `BakedForm` (a sprite ASSADA, que é uma imagem); a peça
  viva leva o `Sculpt3dPieceRef`, e lia-se como vazia. Corrigido na F1.
- E o `kind_of` (a derivação do tipo, `component_attach.rs`) conhecia 3 dos 6 marcadores.

⇒ **A ordem da F3 pela medição:** Image (Painter já é) → Sculpt → Flip → Model → Vector.

## §3 — O desenho

### 3.1 — O tipo declara-se na crate da FAMÍLIA (drop-crate, ADR-0075)

Cada família (`ph2d-app-flip`, `-vec`, `-sculpt3d`, `-field3d`, `-painter` e as que virão:
pixel art, tilemap) **regista** o seu tipo numa tabela da fundação. A shell não escreve a lista:

```
ObjectTypeDecl {
  kind: ObjectKind,               // o vocabulário que já existe (derivado do marcador)
  label_key, icon,                // i18n + ícone (HR-15)
  add_group: AddGroup,            // 2D · 3D · Jogo · Vazio — o agrupamento do menu Add
  presets: &[AddPreset],          // ex. Sculpt ▸ Esfera · Cubo; Image ▸ abre o diálogo de tamanho
  spawn: fn(&mut World, preset) -> Entity,   // cria a entidade COM o marcador e o documento dela
  modes: &[ObjectMode],           // os modos que este tipo declara (D6)
}
```

- **Um tipo que não está compilado não aparece**: o módulo é removível por feature. ⇒ Um tipo
  sem crate não deixa uma entrada morta no menu.
- ⭐ **O `+` da Hierarchy abre um MODAL, como o *Add shape…* do Modeling** (ordem do dono, 03/10:
  *«os objetos devem ser selecionados num modal como os objetos de Modeling»*).
  - ⛔ **Não é um modal novo:** é o `ph2d_editor_core::widget::command_palette`, genérico por
    desenho. Ele recebe um `PaletteModel` (título, grupos coloridos, itens com `NodeId` opaco),
    e quem o abriu mapeia o id de volta.
  - Já tem scrim, cascata, **busca** (o mesmo predicado filtra a pintura e o `Enter`),
    sub-grupos, rolagem e 2 colunas.
  - O modelo copia os precedentes: `ph2d-app-field3d/src/shape_palette.rs`
    (`build_from`, `item_id` = hash da chave i18n, `slot_of_pick`) e o `+` do Inspector
    (`component_palette` da shell, ADR-0166/F3).
  - Os **grupos** são *2D · 3D · Jogo · Vazio*, e os **presets** são itens de cada grupo:
    `Sculpt ▸ Esfera · Cubo`.
  - ⚠️ **A lei da paleta de formas vale aqui:** um tipo que existe mas não pode nascer agora
    APARECE num sub-grupo cuja razão viaja no rótulo, e o clique responde. Um tipo NÃO
    compilado não aparece.
- **O mesmo modal no canvas** (Shift+A e botão direito, §6.3) **em modo Object**.
  - Dentro de um modo, o `A`/Shift+A é o *Add* do PRÓPRIO modo: no Edit do Model são as
    formas, como já é hoje. É a mesma divisão do Blender, em que o Shift+A no Edit acrescenta
    dentro do objecto editado.
- **Criar = um passo de undo.** O objecto novo nasce seleccionado e activo, em modo Object, com
  um nome que conta (`Sculpt`, `Sculpt.001`).

### 3.2 — O modo é do objecto ACTIVO, com o cadeado do Blender

```
ObjectMode = Object | Edit | Sculpt | Paint | Mask | Draw | Pose
```

O vocabulário fecha-se só com o que algum tipo declara hoje. ⛔ O `Mask` foi construído e
**RETIRADO pelo dono no smoke (04/10)** — ver F3 ▸ Image ▸ Mask; hoje o enum tem Object · Paint ·
Sculpt · Draw · Edit.

- **Um recurso só: `ModoActivo { entidade, modo }`.** O cadeado vem ligado (§6.4), por isso só
  há um objecto em modo de criação de cada vez. Multi-objecto fica para depois, e só no Edit,
  como no Blender.
- **O selector *«Object Mode ▾»*** é o 1.º pulldown do cabeçalho da área. As faces dele são os
  modos que o tipo do activo declara. ⛔ Um modo indisponível NÃO aparece: não fica cinzento
  (D6).
- **Atalhos:**
  - **Tab** alterna Object ↔ o último modo de criação daquele objecto;
  - **Ctrl+Tab** abre a lista ✅ (04/10);
  - o atalho de cada modo vem depois (Blender: `Tab`, e um menu circular com `Ctrl+Tab`).
- **Entrar num modo = abrir o módulo SOBRE aquela entidade:** as ferramentas, os painéis e a
  vista do módulo. **Sair** volta a Object e o módulo larga a entidade.
- **Seleccionar noutro objecto:**
  - em Object, muda o activo e o Inspector mostra o que é dele (o contexto leve da Godot);
  - num modo de criação, é recusado (o cadeado).
- ⭐ **O tipo continua derivado.** O `ModoActivo` guarda a ENTIDADE, nunca o tipo: o tipo
  pergunta-se ao marcador, e assim não há duas respostas para «que objecto é este».

### 3.3 — Os layouts (as abas de cima) ficam, com o campo opcional do Blender

Um layout pode pedir um modo *«ao abrir»*: o `Mode:` do Workspace do Blender. Aplica-se **só se
o activo tiver esse modo**. Exemplo: abrir *Flip* com um desenho Flip activo entra em `Draw`.

Sem activo compatível, o layout arruma os painéis e fica em Object. ⛔ Não cria um objecto
sozinho (§6.5).

### 3.4 — A tabela de modos (D6, com o que existe HOJE)

| tipo | modos | módulo que abre |
|---|---|---|
| qualquer | **Object** (mover · rodar · escalar) | gizmo de transformação |
| ~~Sculpt (malha) · Model (SDF)~~ | ⛔ saíram com o 3D (ADR-0179, 05/10) | — |
| Vector (a forma) | Object · **Edit** ✅ — cada forma é um objecto (F3 ▸ Vector, 2.ª volta, 05/10) | `ph2d-app-vec` (o Edit tem o painel inteiro; criar = *Add ▸ Vector Drawing*, nada nasce até ao 1.º traço) |
| Flip | Object · **Draw** · **Edit** | `ph2d-app-flip` |
| Image | Object · **Paint** (⛔ o Mask foi retirado pelo dono, 04/10) | `ph2d-app-painter` |
| Câmara, corpo de física, áudio, HUD… | **só Object** | Inspector |

⛔ O `Paint` do vetor fica FORA: a feature não existe (D6, correcção 2).

## §4 — As fases (cada uma fecha com gate e foto, na ordem)

- **F0 — Medir, módulo a módulo** (sem código de produto): *o que é o DOCUMENTO de cada módulo, e
  ele consegue ser «o desta entidade»?* Para cada um dos cinco, diga:
  - quem guarda hoje «o que estou a editar»;
  - se há N objectos no projecto;
  - o que faz hoje o módulo com um activo de outro tipo.

  O resultado decide a ordem da F3. ⚠️ É o item caro do plano, e não se adivinha.
- **F1 — Tipos e o menu Add** — ✅ **entregue em 03/10** (`161e4cfd5`): `ph2d_editor_core::object_add`
  + um `object_add.rs` por família + a fase `render_loop/fase_object_add.rs`. O desvio do desenho:
  a `ObjectTypeDecl` não leva `spawn: fn(&mut World, …)` — cada família precisa de recursos
  DIFERENTES (a `VecScene` e a `ShapeTool`, o `FlipDoc`, a cena GPU do Sculpt), e a assinatura
  comum seria uma mentira. Cada família expõe `ENTRIES` e um `add(entry, <os seus recursos>)`
  que devolve `None` para entradas alheias; os `modes` entram na F2, com quem os lê.
  ⏳ **O botão direito do canvas passa para a F2:** ele já tem donos (o vetor cancela a forma, o
  Motion e o Painter abrem menus próprios), e só o modo Object diz quando ele é do menu Add.
  - a tabela `ObjectTypeDecl` e o registo por família;
  - o `+` da Hierarchy e o Shift+A abrem a **paleta** (`command_palette`) com o modelo dos tipos;
  - o Ctrl+N passa a ser atalho da entrada *Image* (§6.2).
  - **Gates:**
    - cada `ObjectKind` compilado tem um item na paleta, e um id de item mapeia de volta a um
      só tipo/preset;
    - a busca acha cada tipo pelo nome;
    - o `spawn` de cada um produz uma entidade cujo tipo DERIVADO é esse;
    - criar e desfazer devolve o mundo ao bit;
    - o clique real (seam `ph2d-ui-testkit`) em cada entrada cria a entidade.
  - **Foto:** a paleta aberta pelo `+`, ao lado da do *Add shape…* (têm de ler-se como a
    mesma janela).
- **F2 — O modo:** ✅ **entregue em 03/10, JUNTO com a F3 da Imagem** (`004a8c683` · `c50f43b3c` ·
  `c5cee174a`; handoff `HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_OS_MODOS.md`). Um modo que aparece
  e não abre nada é controlo morto (D6), logo o 1.º modo veio com o módulo dele: **Image ▸ Paint**.
  - `ph2d_editor_core::object_mode` — `ObjectMode {Object, Paint}` (o vocabulário fecha com o que
    funciona), `ModeState` (a ENTIDADE, nunca o tipo), as leis puras `resolve`/`still_holds`/
    `decide` e o seletor. O quadro é `screens::hero::mode_drive::drive`; cada família declara uma
    `ModeFamily` (tipo→modo + as portas `holds`/`enter`/`leave`) — a do Painter é
    `ph2d_app_painter::paint_mode::FAMILY`; a shell só lista `MODE_FAMILIES`.
  - O seletor *«Object Mode ▾»* é o 1.º pulldown da área (`AreaMenus`: o seletor à frente, os do
    módulo atrás, cada escritor só reescreve a sua parte).
  - **Tab** = Object ↔ o último modo do objecto; o **zen** passou ao `Ctrl+Space` (escolha do dono,
    03/10). ✅ O `Ctrl+Tab` (a lista) entrou em 04/10: é o clique do seletor
    (`mode_drive::mode_key`), o 2.º toque fecha, sem activo não faz nada; o chip transbordado abre
    sob o `⋯`.
  - **O cadeado** já existia para o Painter (3 portas: clique, laço, linha da Hierarquia); passou a
    ser do MODO (`mode_drive::refused`). As outras ~60 portas que trocam a selecção (criar,
    duplicar, apagar, desfazer, largar ficheiro) não se ensinam uma a uma: a rede
    `still_holds` devolve o modo a Object quando a entidade se perde.
  - O botão do Painter saiu da barra IMG (`object_mode::TOOLS_OPENED_BY_A_MODE`); a aba **Draw**
    pede o modo (`CanvasOwner::Mode(Paint)` — o campo do §3.3, puxado da F4).
  - O botão direito no canvas livre, em Object, abre o menu Add (`mode_drive::right_click_on_canvas`).
  - ⚠️ **A pintura da peça 3D continua pelo caminho antigo** (IMG + a aba do Painter ao lado da
    escultura): é o Paint do Sculpt, na F3 dele. `paint_mode::holds_an_image` distingue-a (tela
    do ecrã) do modo da imagem.
  - **Gates:** as faces do selector = os modos do tipo do activo; um modo indisponível não aparece;
    o cadeado recusa a troca de activo; Tab ida e volta; duas imagens, Paint numa, a outra intocada;
    o clique real no seletor com todos os painéis.
- **F3 — Ligar os módulos, UM de cada vez**, na ordem que a F0 medir. A proposta é **Sculpt**
  primeiro (o exemplo do dono: Object · Sculpt · Paint), depois Flip, Vector (a partição do
  `DrawMode`, o item difícil da D3), Image e Model.
  - Cada módulo passa a abrir sobre a entidade do `ModoActivo`.
  - O pill antigo que fazia o mesmo sai na mesma fase (⛔ dois caminhos para o mesmo módulo
    divergem).
  - **Gate por módulo:** dois objectos do mesmo tipo, entrar em modo num, e o outro fica
    intocado.
  - **Sculpt** — ⛔ **SAIU com o 3D (ADR-0179, 05/10); o que segue é HISTÓRIA** (as leis que ficaram na
    fundação — `ModeFamily` como trait, `follow`, `wants` — continuam a servir Flip, Vector e Painter).
    Foi ✅ entregue em 03/10 (handoff `HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_SCULPT.md`):
    `ObjectMode::Sculpt` e `ph2d_app_sculpt3d::sculpt_mode::Family` — `(Sculpt3D, Sculpt)` e
    `(Sculpt3D, Paint)`. Sculpt = o barro na tela com a peça da entidade PRESA (a mira do pen-down
    só vê ela, `Sculpt3dScene::preso`); Paint = o mesmo com o Painter em mãos (a tela da vista,
    `painter_na_malha`); Object = o barro sai (para a LUZ).
    - **Desvio do desenho:** a `ModeFamily` deixou de ser uma tabela de `fn(…, &mut ToolRegistry)` e
      passou a **trait**, construída por quadro com o que cada família empresta (a escultura precisa
      da cena e do mapa peça↔entidade) — a mesma razão do `spawn` da F1. Ganhou duas portas:
      `follow` (o módulo segue o modo: o barro fora de um modo desta família sai) e `wants` (um
      objecto que NASCE num modo pede-o).
    - **Escolhas do dono (03/10):** o pill SCULPT **saiu** (o modo e o menu Add são as portas); a
      peça nova **nasce em Sculpt** — fora de Sculpt/Paint a peça não se desenha no canvas (o barro
      e a luz são exclusivos por construção, ADR-0150). A marca é da CENA ao nascer
      (`Sculpt3dScene::pede_o_modo`), logo cobre as cinco portas de nascer (Add, load, import, smoke).
    - O `D` deixou de entrar no barro (luz ⇄ desligada fora dele); as abas Sculpt/Painter ao lado
      pedem o modo (`slot_tabs_ferramenta`), sem o IMG.
  - **Flip** — ✅ **entregue em 03/10** (handoff `HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_FLIP.md`):
    `ObjectMode::{Draw, Edit}` e `ph2d_app_flip::flip_mode::Family` — `(Flip, Draw)` e
    `(Flip, Edit)`. Draw = a ferramenta Flip na mão com uma das de pôr/tirar tinta
    (`FlipMode::DRAW_TOOLS`: Draw · Erase · Fill · Colorize · Trace); Edit = com uma das que mexem
    no traço que existe (`EDIT_TOOLS`: Edit · Reshape, lidas «Select · Sculpt»); Object = a
    ferramenta sai. O painel só mostra as do modo em curso.
    - **O ALVO** (`ph2d_flip::FlipTarget`: o desenho em edição + a camada activa nele) substitui o
      `active_layer` solto; as 14 leituras de «o 1.º objecto» e 10 cópias da regra da camada de
      reserva passam por ele. ⛔ Ele não mora no `FlipDoc` (que é `PartialEq` para o diff do undo:
      entrar num modo seria um passo de histórico).
    - **Medido antes (as três perguntas do briefing):** o pill FLIP tinha 4 portas (pill, *Window*,
      a aba Flip, o layout gravado) e criava um objecto ao activar com o documento vazio; o
      `FlipMode` tinha 8 variantes = `Select` (o Object) + 5 de desenhar + 2 de editar (a confusão
      do `DrawMode` do vetor, D3, em pequeno); os `Pairs` são uma sessão do tween, não um modo.
    - **Desvio do desenho:** `ModeFamily::wants` recebe o registo — a ferramenta Flip que chega à
      mão SEM o modo (as ~20 cenas `PH2D_FLIP_*_SMOKE`) pede o modo do grupo dela, e a que troca de
      grupo pede o outro; a ferramenta só sai quando um modo do Flip ACABOU (`releases`). Assim
      nenhuma cena antiga teve de ser editada.
    - **Escolhas do dono (03/10):** o pill FLIP **saiu** (e a linha *Window ▸ Flip*); o desenho
      novo **nasce em Draw** (`FlipState::born`). A aba Flip de cima pede o Draw
      (`CanvasOwner::Mode`), e sem activo compatível fica em Object — ⛔ já não cria um desenho.
  - **Model** — ⛔ **SAIU com o 3D (ADR-0179, 05/10); o que segue é HISTÓRIA** (ficaram na fundação o
    modo de PARTES — `parts`/`owner_of` — e o multi-objecto, que o Vector usa).
    Foi ✅ entregue em 03/10 (handoff `HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_MODEL.md`):
    `ph2d_app_field3d::model_mode::Family` — `(Model3D, Edit)`. Edit = o painel `model3d` aberto
    (é ele que arma o módulo) com a peça da entidade EM MÃOS (`model_mode::target`); Object = o
    painel fecha.
    - **Medido antes (as três perguntas do briefing):** o pill MODEL tinha 5 portas (pill,
      *Window*, a aba Modeling — que abria o painel e, sem peça, PLANTAVA a demo —, o layout
      gravado, `PH2D_FIELD_SMOKE`); o gizmo do módulo (`scene_gizmo.rs`) move as FORMAS
      seleccionadas, não a peça, e o `Transform` 2D da raiz não entra no cozimento; **N peças
      custam pouco** — o documento já é recozido do MUNDO a cada quadro, e as perguntas «que peça?»
      eram quatro `q.iter().next()` (cozer, gizmo, enquadrar, materiais), hoje
      `scene::root_in_hand`. Um documento por raiz (traçar N campos) não foi preciso: fora do Edit a
      peça não se desenha (como a escultura).
    - **Desvio do desenho — o modo de PARTES:** no Model, editar é seleccionar as formas de DENTRO
      da peça, e cada uma é uma linha da Hierarquia. O cadeado exigia a selecção EXACTA, e o Edit
      cairia no 1.º clique. ⇒ `ModeFamily::parts` (o que se edita dentro da entidade) e
      `::owner_of` (uma parte responde pelo dono): em Edit a selecção anda pelas formas e pelas
      luzes da cena (uma, várias, nenhuma), outra peça é recusada; sair devolve a selecção à peça
      inteira (o `Tab` seguinte volta ao Edit). O Paint/Sculpt/Draw não declaram partes e mantêm
      a lei exacta.
    - **N peças:** o menu Add planta a 2.ª, 3.ª… ao lado (`scene::plant`, nome único, ⛔ uma luz por
      CENA); o rótulo «um por cena» saiu. As luzes da cena são parte do Edit de QUALQUER peça.
    - **Escolhas do dono (03/10):** o pill MODEL **saiu** (e *Window ▸ Model 3D*); a peça nova
      **nasce em Edit** (`model_mode::born`). A aba Modeling de cima pede o Edit (`CanvasOwner::Mode`;
      o `CanvasOwner::Model3d` saiu) e já não abre o painel sem peça.
    - ⛔ **Morreu com o 3D (05/10)** — era: em Object a peça não se desenha no canvas 2D (D9: o 3D como
      camada entre camadas é outra obra), e o gizmo do objecto move um `Transform` que o traçado não
      lê. **Medido em 04/10** (o traçado é CPU, 46–57 ms por quadro a 560², ecrã cheio exclusivo; falta
      a ponte para textura na GPU, o afim 2D do `Transform` da raiz e a ordem entre camadas: ~800–1 200
      linhas em 3–5 crates) ⇒ **escolha do dono: LINHA NOVA, depois.**
  - **Vector** — ✅ **entregue em 04/10 como OBJECTO-CONTENTOR** (handoff
    `HANDOFF_INTEGRACAO_line_UIUX_2026-10-04_O_OBJECTO_VETORIAL.md`). Escolha do dono depois do smoke:
    *«apenas uma opção no modal: objeto vetorial. Ao clicar nele cria-se um objeto vazio e entra-se no
    modo edit do vector com o menu exatamente como era antigamente […] as shapes são filhas do objeto
    vetorial vazio»*.
    - **O tipo** é o marcador `ph2d_ecs::VecObject` (uma entidade sem geometria cujas filhas são as
      formas); as formas deixam de ser objectos (`kind_of` = Empty) e passam a PARTES dele.
    - **Add ▸ Vector Object** cria-o vazio no centro da vista e ele **nasce em Edit** (`born`, como o
      Flip e o Model). **Edit** = a ferramenta `vector` na mão com QUALQUER `DrawMode` (o painel
      inteiro) e o objecto como alvo (`VecState::edit`); o que nasce no Edit é filho dele; só as formas
      dele se agarram e mostram nós (`VecViewState::editing`, agora `Option`: `Some(vazio)` = um
      objecto ainda sem formas). **Object** = a ferramenta sai da mão; o clique, o laço e o realce sobem
      ao objecto (`vector_mode::lift_to_objects`) e o gizmo é a caixa-união das formas
      (`vec_gizmo_view::object_view`). `Tab` sobre o objecto ou uma forma dele = Edit; vários = Edit
      de todos (`joins`).
    - **A regra das SOLTAS** (escolha do dono: *«cada uma ganha o seu objecto»*):
      `ph2d_vec_entities::entities::object::adopt_loose`, no passe do desenho e na rede da captura —
      a forma sem objecto entra no do Edit aberto, ou ganha um novo no lugar dela (projecto antigo,
      cenas de teste, colar/duplicar fora de um Edit). A «cabeça» sobe por todo pai só-vetor (moldura,
      envelope, grupo de formas). Um SVG importado de uma vez entra num só objecto (`enclose`).
    - **A fronteira do objecto:** agrupar (logo a booleana viva) e o clique param no objecto
      (`top_within_object`, `selection_root`); desagrupar nunca o dissolve.
    - **Mantém-se da onda anterior:** o pill VECTOR fora; `ModeFamily::joins`/`enter_with`; o
      `in_edit` como porta única do clique e das âncoras. Smoke `PH2D_OBJECT_MODE_SMOKE=6`.
    - ⛔ **RECUSADO pelo dono no smoke (04/10), não reconstruir:** a forma como objecto com a partição
      `DrawMode::EDIT_TOOLS`/`object_mode` (Edit = Node·Fillet·Chamfer·Width·Trim; criar em Object) e
      o painel a filtrar a fileira pelo modo; as entradas Rectangle·Ellipse·Polygon·Star·Pen·Pencil·Text
      no Add. Histórico em `HANDOFF_INTEGRACAO_line_UIUX_2026-10-04_O_VETOR.md`.
  - **Vector, 2.ª volta** — ✅ **entregue em 05/10** (`8d344c9de`). Decisão do dono, que SUBSTITUI o
    contentor acima (⛔ ele saiu: `VecObject`, a regra das soltas, a fronteira do objecto, a caixa-união): *«Não precisaremos mais de um objeto vazio como pai de vetoriais. Quando o usuário criar um
    desenho vetorial, o painel vector abre e nada aparece no canvas ou na hierarquia até que o usuário
    crie alguma forma ou linha. Ao desenhar algo, o objeto aparece na hierarquia no modo edit»*; e,
    perguntado *«a 2.ª forma desenhada no mesmo Edit entra no 1.º objecto ou é outro?»*: **cada forma
    é um objecto** (sem contentor).
    - ⇒ **Add ▸ Vector** (uma entrada só) não cria entidade: abre o painel Vector inteiro (o de
      sempre) com a ferramenta na mão. A 1.ª forma ou linha nasce como OBJECTO na Hierarquia e entra
      em **Edit** sobre ela; a 2.ª é outro objecto, e o Edit passa a ela.
    - Continua ⛔ **recusado** (04/10): as 7 entradas Rectangle·Ellipse·…·Text no Add; o painel a
      filtrar a fileira pelo modo; a partição `DrawMode::EDIT_TOOLS` (Edit = só as de mexer).
    - **Como ficou:** `ObjectKind::Vector` lê `VecPathRef`; um GRUPO de formas é um vazio (como o
      *empty* do Blender) — o Edit é por forma, e várias formas seleccionadas + `Tab` = Edit de todas.
      O Add arma a ferramenta (`EditTarget::arm`; a shell pede Object); a forma que nasce com a
      ferramenta na mão pede o Edit (`vector_mode::newborn`: UMA só, fora de gesto) e, num Edit, o
      modo PASSA a ela sem largar a ferramenta nem repetir o aviso. O quadro do modo deixou de juntar
      a selecção ao pedido de uma família (`wants`). `PROJECT_SCHEMA` 183 → 184: um projecto com
      contentores (v182–183) é recusado, pela decisão de sempre. O duplicar ao lado da original fica
      (`entities::duplicate::place_beside`). Smoke `PH2D_OBJECT_MODE_SMOKE=6`.
  - **Vector, os furos** — ✅ **05/10**. Decisão do dono, perguntado se Shift+clique juntava uma forma
    de fora ao Edit: *«o modo de edição significa que todos os objetos daquele tipo estão em modo de
    edição. Ao clicar num objeto de outro tipo, o objeto deve ser selecionado mas em modo object,
    saindo do modo de edição do objeto de tipo diferente»*. ⇒ **o Edit do vetor é do TIPO**
    (`ModeFamily::holds_the_whole_kind`): toda forma se agarra e é parte dele (o filtro
    `VecViewState::editing` saiu); escolher um objecto de outro tipo (canvas no Select, Hierarquia)
    não é recusado — a selecção muda e o modo volta a Object; sair com `Tab` deixa a selecção como
    estava. ⚠️ O Paint e o Draw continuam com o cadeado (recusam a troca). A forma trancada que
    MORRE passa o Edit à **herdeira** (`ModeFamily::heir`): o *Weld* consome os traços e a rede fica
    em Edit com o gizmo; a caneta continua um caminho aberto pela PONTA (já existia, `reopen_endpoint`)
    e junta outro pela ponta dele. O press do Width não dá paradas a uma forma travada. Smoke
    `PH2D_OBJECT_MODE_SMOKE=7` (duas linhas pela caneta + *Weld*).
  - **Image ▸ Mask** — ⛔ **RETIRADO pelo dono no smoke (04/10):** *«não armou a máscara
    imediatamente. vamos retirar esse modo mask»*. Não reconstruir sem ler o porquê. O que se
    construiu (`1d87241aa`, desfeito por revert com `f796202ff` e `94156f3e4`): `ObjectMode::Mask` =
    o Painter com a MÁSCARA DA CAMADA como alvo (criada branca se faltar), escolhida pelo dono entre
    as duas máscaras medidas (a da camada, gravada; o pincel de protecção, transitório). O alvo
    punha-se no quadro SEGUINTE ao de entrar (o Painter só recebe a imagem mais tarde no quadro) e a
    máscara nascia BRANCA — nada mudava no ecrã até pintar. A causa exacta do «não armou» não foi
    investigada (o dono retirou o modo). Histórico:
    `HANDOFF_INTEGRACAO_line_UIUX_2026-10-04_O_CTRL_TAB_E_O_MASK.md`.
- **F4 — Layouts:** o campo opcional *«modo ao abrir»* (§3.3) e a limpeza dos toggles de módulo
  que viraram modos.

⚠️ **Custo e janela:** foi uma obra de várias ondas, feita na própria `line/UIUX` (F1–F3 integradas
a 04/10; a recomendação de uma `line/ObjectModes` à parte não se seguiu).

## §5 — Recusas e riscos (o que NÃO fazer)

- ⛔ **Guardar o tipo num campo:** cria duas respostas para «que objecto é este» (a lei do
  `ObjectKind`).
- ⛔ **Uma lista global de modos escolhível:** os modos são do tipo (D3).
- ⛔ **Modos cinzentos/indisponíveis no selector:** são controlos mortos (D6; o Blender não os
  mostra).
- ⛔ **A árvore de herança da Godot no menu Add:** é para 150 tipos; temos ~10, e o Blender
  agrupa por tipo de dado.
- ⛔ **Criar um objecto ao abrir um layout:** um clique de navegação que muda o projecto.
- ⚠️ **Risco: o pill antigo e o modo novo a abrir o mesmo módulo** (D3: 2⁹ combinações de
  toggles). A cura é sair na mesma fase.

## §6 — As escolhas do DONO (perguntadas em 03/10)

1. **Pintura:** é um MODO da imagem (Image ▸ Paint, como a D6 e o Texture Paint do Blender), ou
   um tipo próprio no menu Add? — *recomendação: modo da imagem.*
2. **Ctrl+N:** fica como atalho de *Add ▸ Image*? — *recomendação: fica.*
3. **O mesmo menu Add no canvas** (botão direito / Shift+A, como no Blender)? — *recomendação: sim,
   na F1.*
4. **O cadeado:** em modo de criação, clicar noutro objecto não troca de objecto (como o Blender de
   fábrica)? — *recomendação: sim.*
5. **Ao abrir uma aba de cima sem objecto compatível:** fica em modo Object (recomendado) ou cria
   um?
6. **Objectos de jogo no menu Add** (Câmara, Corpo de física, Fonte de áudio, HUD = um vazio com o
   componente já posto, só modo Object)? — *recomendação: sim, grupo «Jogo».*

**Respostas do dono (03/10):**

| | escolha |
|---|---|
| 1 | **modo da imagem** |
| 3 | **sim, nos dois sítios** (`+` e Shift+A / botão direito) |
| 4 | **não trocar** (o cadeado ligado) |
| 6 | **sim, grupo «Jogo»** |
| — | ⭐ **o `+` abre um MODAL como o *Add shape…* do Modeling** (pedido a seguir ao plano, §3.1) |

Sem pergunta, ficam na recomendação (o dono pode mudar):
- **2** — o Ctrl+N fica como atalho;
- **5** — ao abrir uma aba sem objecto compatível, fica em Object.

⇒ Na tabela do §3.4 o `Painted` não tem entrada própria no menu Add. O `PaintedDoc` continua a
ser o marcador que a pintura põe na imagem.
