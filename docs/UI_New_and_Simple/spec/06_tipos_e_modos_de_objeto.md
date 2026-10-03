# 06 — Tipos de objecto e modos de edição por objecto (plano, 2026-10-03)

> **Pedido do dono, 2026-10-03:** *«Precisamos de um sistema de separação dos modos de edição de
> cada objeto como faz o Blender. A primeira coisa é criar os tipos de objetos a partir do + da
> hierarchy […] Já é possível criar Sprites com Ctrl+N. Isso precisa ir para o + da Hierarchy. Assim
> também os objetos Flip, Vector, Sculpt, Model […] E para cada objeto devemos ter como no blender
> modos de edição.»* — e *«antes de implementar, estude apps similares e planeje»*.
>
> Leitor: a janela que implementa. ⚠️ **Isto é um PLANO aprovado por partes**: as escolhas de
> produto estão no §6, com a resposta do dono quando a houver. Não comece a F1 sem o §6 respondido.

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
  `Node` = Edit) e 12 ferramentas (D3, medido em 30/08).
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
  - O modelo copia os precedentes: [`ph2d-app-field3d/src/shape_palette.rs`](../../../crates/ph2d-app-field3d/src/shape_palette.rs)
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

O vocabulário fecha-se só com o que algum tipo declara hoje.

- **Um recurso só: `ModoActivo { entidade, modo }`.** O cadeado vem ligado (§6.4), por isso só
  há um objecto em modo de criação de cada vez. Multi-objecto fica para depois, e só no Edit,
  como no Blender.
- **O selector *«Object Mode ▾»*** é o 1.º pulldown do cabeçalho da área. As faces dele são os
  modos que o tipo do activo declara. ⛔ Um modo indisponível NÃO aparece: não fica cinzento
  (D6).
- **Atalhos:**
  - **Tab** alterna Object ↔ o último modo de criação daquele objecto;
  - **Ctrl+Tab** abre a lista;
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
| Sculpt (malha) | Object · **Sculpt** · **Paint** (Edit quando existir) | `ph2d-app-sculpt3d` + Painter na peça |
| Model (SDF) | Object · **Edit** | `ph2d-app-field3d` |
| Vector | Object · **Edit** (nós e alças) | `ph2d-app-vec` (o `DrawMode` partido em modo + ferramentas) |
| Flip | Object · **Draw** · **Edit** | `ph2d-app-flip` |
| Image | Object · **Paint** · **Mask** (⏳ §6.1) | `ph2d-app-painter` |
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
- **F2 — O modo:**
  - `ObjectMode`, `ModoActivo`, o selector no cabeçalho, Tab e o cadeado;
  - ainda sem módulos ligados: só Object, e o Edit de um tipo que já o tenha.
  - **Gates:**
    - as faces do selector = os modos do tipo do activo;
    - um modo indisponível não aparece;
    - o cadeado recusa a troca de activo;
    - Tab ida e volta.
- **F3 — Ligar os módulos, UM de cada vez**, na ordem que a F0 medir. A proposta é **Sculpt**
  primeiro (o exemplo do dono: Object · Sculpt · Paint), depois Flip, Vector (a partição do
  `DrawMode`, o item difícil da D3), Image e Model.
  - Cada módulo passa a abrir sobre a entidade do `ModoActivo`.
  - O pill antigo que fazia o mesmo sai na mesma fase (⛔ dois caminhos para o mesmo módulo
    divergem).
  - **Gate por módulo:** dois objectos do mesmo tipo, entrar em modo num, e o outro fica
    intocado.
- **F4 — Layouts:** o campo opcional *«modo ao abrir»* (§3.3) e a limpeza dos toggles de módulo
  que viraram modos.

⚠️ **Custo e janela:** é uma obra de várias ondas, que cruza a fundação e cinco crates de família.
Recomenda-se uma **linha nova** (`line/ObjectModes`), aberta depois de integrar a `line/UIUX`,
com uma janela por fase.

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
