# Auditoria do subsistema de INSTÂNCIAS — 2026-08-27

> **Origem:** report do Enio — *«tudo meio bugado. Master fica invisível, algumas instâncias e
> cópias ficam invisíveis. Mudar uma instância não muda outra instância.»* — e o pedido de
> **auditoria completa multiagêntica**.
>
> **Método:** 6 lentes independentes (visibilidade · duas-fontes-que-discordam · costura de UI ·
> ordem no quadro · propagação/override · undo-persistência-determinismo) sobre a `line/components`
> em `HEAD 5adaf0d7b`, seguidas de **refutação adversarial**: cada achado enfrentou um cético
> instruído a *preferir refutar em caso de dúvida*.
>
> **Saldo: 56 achados brutos → 55 a refutação → 8 CONFIRMADOS · 4 REFUTADOS.**
> ⚠️ **Os 4 refutados valem tanto quanto os confirmados** — são trabalho que não se faz. Estão no
> §3, com o mecanismo que os derrubou.

⚠️ **Nenhum gate estava vermelho.** Vinte e três gates do subsistema correram verdes durante a
auditoria inteira — e é isso que torna esta lista interessante: cada achado abaixo traz **qual gate
estava verde por cima dele e porquê**. O padrão que se repete tem nome desde 26/08: *um gate sobre a
MARCA que eu escolhi fica verde quando a premissa da marca é falsa* — aqui, seis dos oito medem a
marca, ou medem-na numa fixtura onde o fenómeno não pode acontecer.

---

## §1 — Os OITO confirmados

### §1.1 — Tudo o que NASCE ou é ARRASTADO para dentro de um MasterRoot fica invisível — e quatro gestos comuns fazem exatamente isso, sem recusa nem aviso

| | |
|---|---|
| severidade | **alto** · confiança alta |
| explica do report | `instancias-invisiveis` |
| lente | LENTE 1 — VISIBILIDADE |

**Mecanismo.** `assign_master_pieces` (crates/ph2d-ecs/src/master.rs:81-118) marca a raiz e TODA a descendência de todo `MasterRoot`, re-derivado por quadro. `off_canvas::is_off_canvas` (shells/desktop/src/render_loop/off_canvas.rs:43-47) devolve `true` para qualquer `MasterPiece` sem `MasterEditing`, e os DOIS leitores obedecem: `sim_extract.rs:344` não emite `RenderInstance` (logo também não entra em `sort_inputs.push` na :440 nem em `pick_sprites_at_world`, que exige `RenderInstance` — crates/ph2d-render/src/picking.rs:131) e `vec_entities.rs:205` marca o path como hidden. ⇒ *ser descendente de uma receita é invisibilidade, e o único gesto que a produz de propósito é o Make Component*. Quatro portas põem uma entidade nova debaixo de uma peça de receita e NENHUMA recusa:
(1) `instance_verbs.rs:70` `let parent = ChildOf(entity)` + `:75` `instantiate_master(..., parent, ...)` — *Make Component* numa PEÇA da receita cria a instância DENTRO da receita (a recusa em :62 só olha `MasterRoot` na própria entidade; a de :67 só olha instância);
(2) `render_loop/hierarchy.rs:282` → `instantiate.rs:161` `let parent = ChildOf(src)` — *Duplicate* numa peça da receita;
(3) `render_loop/hierarchy.rs:295-307` — *Add Child* numa linha de receita (spawn de `Transform`+`Name`+`ChildOf(parent)`, e como é objeto vazio o anel dele TAMBÉM é suprimido pelo F4 ⇒ zero pixels);
(4) `hero_intents/hierarchy.rs:17-21` — arrastar qualquer linha para a linha da receita, que o próprio doc declara silencioso (*«never pushes a toast (silent reparent matches existing UX)»*).
Os toasts dizem «Made a component — an instance took its place» / «Duplicated entity» / «Added child entity» e o canvas não muda.

**Como reproduzir.** cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_INSTANCE_SMOKE=1 cargo run -p ph2d-host-desktop --release — na Hierarquia, botão direito em `Ragdoll > Arm` → *Duplicate* (repetir com *Make Component* e *Add Child*). Toast verde, canvas idêntico. A prova de que a entidade existe: a linha nova aparece na Hierarquia com o olho ABERTO.

**O gate que faltava.** Um gate que faça a pergunta do FIM sobre a coisa NOVA: «depois deste verbo, `is_off_canvas(a entidade que o artista acabou de criar)` é falso» — para os quatro verbos, com a receita a ter PAI numa das fixturas. Hoje nenhum gate constrói uma receita com pai nem cria nada debaixo de uma.

**O gate que estava VERDE, e porquê.** `instance_verbs::tests::the_whole_recipe_leaves_the_canvas_and_the_instance_stays` (shells/desktop/src/instance_verbs_tests.rs:222, verde agora). Fixtura sem o fenómeno: `plain_rig` (:144) nasce na RAIZ da cena, logo `ChildOf` é `None`, `parent = None` e a instância NUNCA pode cair dentro da receita. O gate ainda mede a MARCA (`get::<MasterPiece>(instance).is_none()`) em vez do fim, mas nem chega a poder falhar.

**Veredito do cético.** O fenómeno ACONTECE e as quatro portas são alcançáveis e sem guarda: `assign_master_pieces` (crates/ph2d-ecs/src/master.rs:81-118) marca raiz+descendência de todo `MasterRoot` e re-deriva por quadro (8/8 gates verdes, `cargo test -p ph2d-ecs --lib master`); `is_off_canvas` (shells/desktop/src/render_loop/off_canvas.rs:43-47) esconde `MasterPiece` sem `MasterEditing`; a tabela do menu de linha é PLANA (crates/ph2d-editor-core/src/screens/hero/menu_rows.rs:188-259 — Duplicate/Add Child/Make Component aparecem em TODA linha, receita incluída); o botão direito NÃO seleciona (pointer_down_menus.rs:164-170 só abre o menu); e nada na Hierarquia marca uma linha como receita (zero `MasterRoot`/`ObjectInstance` na montagem das linhas — grep sem resultado fora de `instance_*`). Recusas existentes: `AlreadyAMaster` só na PRÓPRIA entidade e `InsideAnInstance` (instance_verbs.rs:62-69) — não há `InsideAMaster`; o reparent é mudo por doc (hero_intents/hierarchy.rs:15-16). Não refuto.

⚠️ **Correções do cético ao achado** (leia-as: em três casos elas MUDAM o preço ou o mecanismo).
O achado larga o `MasterEditing` na conclusão, e ele é o que decide. A lei certa não é «ser descendente de uma receita é invisibilidade» — é «o que nasce dentro de uma receita HERDA a regra de visibilidade DELA»: invisível exactamente enquanto nada daquela receita está selecionado. Os gates `the_recipe_comes_back_while_it_is_being_edited` e `changing_the_selection_puts_the_recipe_back_out_of_the_scene` (render_loop/master_editing_tests.rs:37,57) fixam isso e estão verdes porque medem o fim certo.

(a) A REPRO NÃO É PROVA. Com `PH2D_INSTANCE_SMOKE=1` nada está selecionado, então `Hub` e `Arm` — a FONTE do Duplicate — já estão off-canvas antes do gesto. «Toast verde, canvas idêntico» é o que a regra desenhada imprime; não distingue defeito de desenho. A medição que distingue: selecionar a linha da receita PRIMEIRO (a receita entra em cena) e só então correr cada um dos quatro gestos.

(b) Duas mis-citações de árvore: instance_verbs.rs:70 e instantiate.rs:161 são `parent = ChildOf(src).0` — o PAI de src, não `ChildOf(src)`. A cópia/instância aterra IRMÃ, não filha. A conclusão (continua dentro da receita) sobrevive; a árvore descrita não.

(c) Sob a medição discriminante o veredito muda porta a porta:
 • Duplicate numa peça: `duplicate_subtree` chama `assign_master_pieces` (instantiate.rs:180) e o `mark` já correu (mod.rs:2630) muito antes do dreno (mod.rs:9904) ⇒ invisível UM quadro; no quadro N+1 o `mark` re-deriva `subtree(root)` e a cópia entra ⇒ VISÍVEL. Não é silencioso nem permanente.
 • Arrastar para a receita: o Click que selecciona é suprimido no drop (pointer_up.rs:302-326 — `still_hot` é falso porque o ponteiro saiu do `active_rect` da linha arrastada), logo a selecção fica a ANTERIOR. Se ela já apontava para dentro da receita, o objeto continua visível; se não, ele some na hora. A porta é real, mas o resultado é função da selecção prévia — que o achado nunca nomeia.
 • ⭐ Make Component numa peça: a peça ganha `MasterRoot`, e `master_root_of` (master.rs:130-140) pára na raiz MAIS PRÓXIMA ⇒ selecioná-la marca só `subtree(peça)`, e a instância irmã sob a receita EXTERNA fica `MasterPiece` sem `MasterEditing` — invisível MESMO com ela selecionada. Pior que o alegado, e por um mecanismo não nomeado: um `MasterRoot` aninhado ENCURTA a sub-árvore de edição.
 • ⭐⭐ Add Child: a mais forte, e o achado erra a causa («o anel é suprimido pelo F4»). Quem o suprime é um TERCEIRO leitor que não conhece o `MasterEditing`: `group_gizmo_view::is_empty_object` (group_gizmo_view.rs:84-93) testa só `MasterPiece.is_none()`. Ele alimenta `empty_objects` (:121) → a tinta do anel (render_loop/empty_object_overlay.rs:62), o PICK de canvas (`pick_empty_at_world` :148-153) e o realce de hover (hover_highlight.rs:121) ⇒ um objeto vazio ou um GRUPO dentro da receita tem zero pixels E é impegável em TODO estado de selecção, incluindo o modo de edição que existe para tornar a receita alcançável. E master_editing.rs:17-20 afirma que a pergunta tem «dois sítios»: tem TRÊS, e o terceiro responde outra coisa.

O GATE QUE FALTAVA: um que afirme que os TRÊS leitores concordam sobre o mesmo mundo depois de `mark` — `is_off_canvas` × `vec_entities::visible_chain` × `is_empty_object`. O gate hoje verde sobre isto é `what_draws_itself_and_what_is_not_on_the_canvas_get_no_ring` (group_gizmo_view_tests.rs:216-259): fixtura SEM o fenómeno (nunca chama `master_editing::mark`) e controlo positivo que REMOVE o `MasterRoot` em vez de ACRESCENTAR o `MasterEditing` — por construção ele não consegue ver a divergência. Falta ainda a recusa `InsideAMaster` no `make_master` e um aviso no reparent silencioso.

CONFIANÇA: alta na leitura (gates de `ph2d-ecs` corridos verdes; ordem `mark`@2630 → dreno@9904 lida no mesmo ficheiro; `still_hot` lido). Média no comportamento exacto do drag (não corri o app). O que a mudaria: correr `PH2D_INSTANCE_SMOKE=1`, selecionar `Ragdoll`, e comparar as quatro portas — a previsão é Duplicate aparece, Add Child não, a instância do Make Component não.

---

### §1.2 — Duplicate de um MESTRE devolve outro MESTRE — e um mestre não se desenha: a cópia nasce invisível com toast de sucesso

| | |
|---|---|
| severidade | **alto** · confiança alta |
| explica do report | `instancias-invisiveis` |
| lente | LENTE 1 — VISIBILIDADE |

**Mecanismo.** `MasterRoot` é registado (crates/ph2d-ecs/src/scene/registry.rs:342), logo `deep_copy_subtree` copia-o com o resto do blob (crates/ph2d-ecs/src/instantiate.rs:174-196 — só `owned_document` e o `StableId`/`RootOrder`/`SiblingOrder` da raiz ficam de fora). `duplicate_subtree` (shells/desktop/src/instantiate.rs:155-182) não o retira — o doc-comment em :152 declara-o CORRETO (*«Uma cópia de um MESTRE é outro mestre»*) — e chama `assign_master_pieces` na :180. ⇒ a sub-árvore inteira da cópia nasce `MasterPiece` e `is_off_canvas` esconde-a. ⚠️ Aquela linha de doc foi escrita ANTES de a regra de esconder receitas existir (a F4.6/`off_canvas.rs` é de 2026-08-26) e nunca foi reconferida: era verdade quando um mestre desenhava. O menu de linha é PLANO (instance_verbs.rs:265: *«a tabela daquele menu é PLANA — ela não sabe o que a linha é»*), então *Duplicate* aparece na linha da receita como em qualquer outra, e é o item que um artista escolhe quando quer «mais uma».

**Como reproduzir.** cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_INSTANCE_SMOKE=2 cargo run -p ph2d-host-desktop --release — botão direito na linha `Badge` (a receita) → *Duplicate*. Toast «Duplicated entity», zero pixels novos, e a Hierarquia ganha `Badge (1)` com o olho aberto. Comparar com *Instantiate* na mesma linha, que aparece.

**O gate que faltava.** «Uma cópia produzida pela row *Duplicate* está na cena» — isto é, `is_off_canvas(copy) == false` — com a fonte a ser um `MasterRoot`. Alternativamente o gate que declara o veredito de produto: ou *Duplicate* numa receita é RECUSADO com frase própria (como as três recusas de `VerbRefusal`), ou ele tira o `MasterRoot` da cópia.

**O gate que estava VERDE, e porquê.** Nenhum gate cobre isto. Os de duplicação (`render_loop/hierarchy_duplicate_routing_tests.rs`, `instance_docs_tests::duplicating_a_group_gives_its_vector_children_their_own_paths`) medem a ROTA e a GEOMETRIA da cópia, nunca se ela desenha.

**Veredito do cético.** Tentei refutar por cinco vias e todas confirmaram. (1) `MasterRoot` É registado — `crates/ph2d-ecs/src/scene/registry.rs:342`, com o comentário da `:340` a dizer que só o `MasterPiece` fica de fora. (2) `deep_copy_subtree` NÃO o salta: os únicos saltos são `owned_document` (`crates/ph2d-ecs/src/instantiate.rs:191`) e `RootOrder`/`SiblingOrder` da raiz (`:217-218`); o `StableId` sai por não ser registado. (3) `duplicate_subtree` (`shells/desktop/src/instantiate.rs:155-182`) não o retira e chama `assign_master_pieces` na `:180` — enquanto o irmão `instantiate_master` faz `root.remove::<MasterRoot>()` na `:112` com o comentário «A instância NÃO é um mestre: com o marcador ela nasceria inerte». A assimetria é literal, no mesmo ficheiro. (4) O gesto é alcançável: o menu de linha é uma tabela ESTÁTICA sem filtro por tipo de linha (`crates/ph2d-editor-core/src/screens/hero/menu_rows.rs:190` — "Duplicate" é o 2.º item, logo abaixo do Rename), o guarda do despacho aceita-o (`crates/ph2d-panel-hierarchy/src/event.rs:46`) e o intake em `render_loop/mod.rs:4069` só faz `get_or_insert`. (5) A via de refutação mais promissora era o ROTEAMENTO: se a raiz `Badge` tivesse `VecPathRef`, o ramo vetorial (`hierarchy.rs:248`) duplicaria um PATH e a entidade cunhada pelo `vec_entities::sync` desenharia. Mas em `shells/desktop/src/instance_smoke.rs:162-168` o `Badge` é uma raiz SEM `VecPathRef` (ele está nos filhos `Box`/`Label`) ⇒ `duplicate_kind` devolve `Entity` ⇒ ramo da cópia profunda. O repro está certo tal como escrito. Os dois leitores escondem-na de facto: `sim_extract.rs:344` e `vec_entities.rs:205`. E o gate que sanciona o mecanismo está VERDE — rodei `cargo test -p ph2d-host-desktop --bins duplicating_a_master`: `instantiate::tests::duplicating_a_master_gives_a_master_whose_pieces_are_already_inert` (`shells/desktop/src/instantiate_tests.rs:393`) passa, e afirma `MasterRoot` na cópia E `MasterPiece` nas peças. Ele é verde porque mede a MARCA que escolheu (inércia de física — «um ragdoll da biblioteca a cair meio metro e a parar») e não o FIM que a marca passou a decidir também: o `MasterPiece` ganhou um SEGUNDO consumidor em 2026-08-26 (`render_loop/off_canvas.rs`, os pixels) e nenhum gate re-perguntou o que aquele gate ainda media.

⚠️ **Correções do cético ao achado** (leia-as: em três casos elas MUDAM o preço ou o mecanismo).
O mecanismo alegado está correcto linha a linha. Duas correcções, e a segunda AGRAVA o achado. (A) A cópia não é permanentemente invisível — ela PISCA COM A SELECÇÃO. A cópia é o seu próprio `MasterRoot`, e `master_editing::mark` (chamado em `shells/desktop/src/render_loop/mod.rs:2630` com `hero_screen.gizmo.selection`) carimba `MasterEditing` na sub-árvore de `master_root_of(selection)`. Clicar na linha `Badge (1)` na Hierarquia faz a cópia DESENHAR; mudar de selecção apaga-a outra vez. No instante do gesto a alegação continua exacta: o botão direito NÃO selecciona a linha (`crates/ph2d-editor-core/src/interaction/dispatch/pointer_down_menus.rs:163-169` só abre o menu) e o ramo de entidade do dreno NÃO chama `replace_selection` (só o ramo `Field` o faz, `render_loop/hierarchy.rs:244`) ⇒ o toast «Duplicated entity» cai com zero pixels novos. O sintoma que o artista relata é portanto um objecto que existe na Hierarquia, é inerte à física e só aparece enquanto a própria linha dele está seleccionada — o que casa com «algumas instâncias e cópias ficam invisíveis» melhor do que invisibilidade pura. (B) NÃO HÁ VERBO QUE DESFAÇA. `remove::<MasterRoot>` existe em dois sítios de produto e nenhum é alcançável a partir daqui: `shells/desktop/src/instantiate.rs:112` (nascimento de instância) e `shells/desktop/src/instance_verbs.rs:81` (rollback interno do `make_master` quando ele falha). O enum `Verb` tem só `Make/Place/Detach/Apply`: `Make` recusa com `AlreadyAMaster` (`instance_verbs.rs:63`) e `Detach` exige `InstanceOf` (`instance_verbs.rs:118`). ⇒ a cópia é um fantasma sem cura por gesto — só Ctrl+Z ou Delete. (C) Nota menor: não medi a fonte do ícone do olho na linha da Hierarquia, então a afirmação «com o olho aberto» fica plausível mas não verificada — o que é certo é que o `Visibility` da cópia é o da fonte (o `spawn_vector_master` não põe nenhum, logo ausente) e que a ocultação vem inteiramente do `MasterPiece`, que a Hierarquia não consulta em nenhum sítio que eu tenha encontrado.

---

### §1.3 — Instantiate numa receita ANINHADA larga a cópia na raiz da cena com a pose LOCAL do mestre — ela aparece onde ninguém a procura

| | |
|---|---|
| severidade | **alto** · confiança media |
| explica do report | `instancias-invisiveis` |
| lente | LENTE 1 — VISIBILIDADE |

**Mecanismo.** `Verb::Place` (instance_verbs.rs:304) chama sempre `instantiate_master(sim, registry, entity, None, docs)`. Em `crates/ph2d-ecs/src/instantiate.rs:217-221` a raiz da cópia perde `RootOrder`/`SiblingOrder` e só recebe `ChildOf` `if let Some(p) = parent`; como `ChildOf` **não** é registado (não está em crates/ph2d-ecs/src/scene/registry.rs:315-570), o blob não o traz e a cópia fica mesmo na raiz da cena. Mas o `Transform` copiado é o LOCAL do mestre: a composição com o mundo do antigo pai perde-se. Uma receita que viva dentro de um grupo deslocado devolve a cópia junto à origem — potencialmente fora do enquadramento —, e o `cascade` (instance_verbs.rs:239-246) afasta-a mais a cada clique. Para o artista: «cliquei em Instantiate e não apareceu nada».

**Como reproduzir.** Criar um grupo, movê-lo bem para longe (por exemplo x≈+8 m), pôr uma sprite dentro dele, *Make Component* na sprite (a receita fica DENTRO do grupo, a cópia aterra no sítio certo), depois *Instantiate* na linha da receita: a nova cópia aparece perto da origem do mundo, não ao lado das irmãs.

**O gate que faltava.** «A cópia do *Instantiate* aparece no mesmo sítio de MUNDO em que a receita está, mais o passo da cascata» — com a receita a ter um pai com transform não-identidade. Alternativamente, decidir que `Verb::Place` herda o pai do mestre e gatear isso.

**O gate que estava VERDE, e porquê.** `a_placed_instance_never_lands_on_top_of_what_it_came_from` (instance_verbs_tests.rs:437-447) e todas as outras fixturas (`plain_rig` :144, `ragdoll`, `spawn_master`) montam o mestre na RAIZ da cena ⇒ `parent_world_transform` é a identidade e a diferença entre pose local e pose de mundo é zero. Fixtura sem o fenómeno.

**Veredito do cético.** MEDIDO, não deduzido. Sonda em /tmp/claude-1000/probe-nested (depende por path do `crates/ph2d-ecs` DESTA worktree), grupo em (8,3) com escala 2×, mestre filho com pose local (0.5,0):

  MESTRE  local=(0.500,0.000) scale=(1.000,1.000)
  MESTRE  MUNDO=(9.000,3.000) scale=(2.000,2.000)
  MAKE    MUNDO=(9.000,3.000) scale=(2.000,2.000)  ChildOf=Some(group)
  PLACE   MUNDO=(0.500,0.000) scale=(1.000,1.000)  ChildOf=None
  DELTA   Place vs Make = (-8.500,-3.000) mundo, escala 0.500x

As quatro tentativas de refutação falharam, uma a uma:
(1) «o gesto é impossível — o mestre nunca fica aninhado»: FALSO. `make_master` (shells/desktop/src/instance_verbs.rs:70-75) lê `ChildOf(entity)`, insere `MasterRoot` NA ENTIDADE EM SÍTIO e usa o pai só para a instância aterrar; o mestre fica exactamente onde estava. Um `Make Component` sobre uma sprite que já era filha de um grupo deixa a receita aninhada, que é o caso normal.
(2) «a linha do mestre não é alcançável na Hierarquia»: FALSO. `grep -rln 'MasterPiece|MasterRoot'` não dá um único ficheiro de hierarquia — a árvore é a cena crua, o mestre aninhado aparece como filho do grupo. E o menu é PLANO (crates/ph2d-editor-core/src/screens/hero/menu_rows.rs:251-252, com o doc-comment a dizê-lo): «Instantiate» é oferecido em toda linha.
(3) «há uma guarda a jusante»: FALSO. `instantiate_master` só recusa `NotAMaster` e `WouldNestInItself` — a 2.ª só é avaliada `if let Some(p) = parent`, e o Place passa `None`. Nada depois reparenta: `cascade` (instance_verbs.rs:239-246) apenas SOMA `step*n` à translação.
(4) «o sync corrige a pose no quadro seguinte»: FALSO, e é a metade que fecha o caso. `ROOT_IS_ITS_OWN` (instance_sync.rs:93-95) inclui `"ph2d::ecs::Transform"`, e o laço salta-o quando `is_root` (instance_sync.rs:238-243). A pose da raiz de uma instância é `InstanceLocal` — ninguém a repõe. O erro é permanente.

Mecânica confirmada linha a linha: `ChildOf` NÃO está registado (`grep -n 'ChildOf\b' crates/ph2d-ecs/src/scene/registry.rs` → exit 1; os dois hits de «Children» são `GroupedChildren`/`ClipChildren`), logo o blob não o traz; crates/ph2d-ecs/src/instantiate.rs:215-221 tira `RootOrder`/`SiblingOrder` e só insere `ChildOf` com `Some(p)`; `Transform` ESTÁ registado (registry.rs:315) e vem verbatim — o LOCAL do mestre. `ph2d_ecs::world_transform` (transform_inverse.rs:105-109) compõe por `ChildOf`, então o local do mestre passa a ser o mundo da cópia.

O GATE QUE FALTAVA / os dois que estavam verdes:
- `a_placed_instance_never_lands_on_top_of_what_it_came_from` (shells/desktop/src/instance_verbs_tests.rs:437) é o gate de produto sobre este verbo, e está verde por DUAS cegueiras independentes: (a) fixtura sem o fenómeno — a linha 445 faz `spawn((Transform::from_translation(at), Name::new("Badge")))`, sem `ChildOf`, mestre na raiz da cena; (b) régua no espaço errado — o oráculo consulta `(&InstanceOf, &Transform)` e compara `t.translation`, que é LOCAL. Mesmo com uma fixtura aninhada ele ficaria verde, porque o local da cópia É o local do mestre mais a cascata. Faltava `world_transform`.
- `the_caller_says_where_the_copy_lands` (crates/ph2d-ecs/src/instantiate_tests.rs:168) é verde porque AFIRMA o comportamento: `assert!(get::<ChildOf>(loose.root).is_none())`. Está certo — é o contrato da porta. O gate em falta é uma altitude acima: nada mede que os TRÊS chamadores do produto respondam a mesma pergunta.
- O gate a escrever: com um mestre aninhado sob um pai transformado, `world_transform(cópia do Place)` tem de ficar a um degrau de tela de `world_transform(cópia do Make)` — e a cópia do Place tem de partilhar o pai do mestre. Mutação que o mata: trocar `parent` por `None` no `make_master`.

CONFIANÇA: alta. Sonda executada contra o código desta worktree, com controlo positivo no mesmo binário (o ramo `Some(group)` dá (9,3) e escala 2×, o `None` dá (0.5,0) e escala 1×). O que a mudaria: uma trava de UI que impedisse `Instantiate` numa linha aninhada (procurei e não existe — o menu é plano por decisão documentada), ou um passe pós-dreno que reparenteasse a cópia (não existe: o dreno em render_loop/hierarchy.rs:333-357 só chama `instance_verbs::drain` e marca `title_dirty`).

⚠️ **Correções do cético ao achado** (leia-as: em três casos elas MUDAM o preço ou o mecanismo).
O achado acontece, mas a redacção erra em três pontos e omite a consequência pior.

1) O QUE SE PERDE NÃO É SÓ A TRANSLAÇÃO — é o transform de mundo INTEIRO do pai. A sonda mediu escala `0.500x` (grupo a 2×) além do desvio de (-8.5,-3.0). Rotação idem. Sintoma para o artista: a cópia não está só noutro sítio, está com OUTRO TAMANHO e OUTRO ÂNGULO — e é isso que impede o report de se ler como «apareceu deslocada».

2) A CASCATA NÃO É AGRAVANTE, e nomeá-la baralha o mecanismo. `cascade` (instance_verbs.rs:239-246) soma `PASTE_OFFSET_PX` convertido pela câmara (render_loop/hierarchy.rs:352-357) — 12 px de TELA por cópia, o mesmo degrau do Ctrl+D, deliberado e gateado por `shells/desktop/tests/a_placed_instance_lands_a_screen_step_from_its_main.rs`. Contra os 8,5 unidades de mundo do desvio medido, é ruído. O achado é um só termo: o transform do pai perdido.

3) O DEFEITO NÃO É `deep_copy_subtree` LARGAR NA RAIZ — isso é o contrato dela, e há gate verde que o afirma (`the_caller_says_where_the_copy_lands`, crates/ph2d-ecs/src/instantiate_tests.rs:168-182). O defeito está uma altitude acima: o produto tem TRÊS chamadores da cópia profunda e só um responde diferente a «onde aterra uma cópia?».
   - `make_master` → instance_verbs.rs:70 lê `ChildOf(entity)` e passa-o (linha 75);
   - `duplicate_subtree` → shells/desktop/src/instantiate.rs:161 lê `ChildOf(src)` e passa-o;
   - `Verb::Place` → instance_verbs.rs:304 escreve `None` LITERAL.
   Duas portas derivam o pai da fonte, a terceira tem-no cravado. É uma discordância entre irmãos, não um erro do motor de cópia.

4) A CONSEQUÊNCIA QUE O ACHADO NÃO NOMEIA, e que é pior que o aterrar: a cópia nº1 (a que o *Make Component* deixa no lugar) fica DENTRO do grupo; as cópias nº2..n ficam na RAIZ DA CENA. ⇒ **mover o grupo passa a mover uma instância e não as outras**, e as instâncias do mesmo mestre deixam de se comportar como conjunto. Isto é permanente e sobrevive a o artista arrastar a cópia perdida de volta para o sítio à mão — ela continua a não seguir o grupo, porque o que falta é o `ChildOf`, não a pose. Encaixa directamente na frase do Enio *«mudar uma instância não muda outra»* lida pelo lado do gesto de grupo, e no *«algumas cópias ficam invisíveis»* pelo lado do enquadramento.

5) Correcção de endereço: as linhas do bloco na `deep_copy_subtree` são 215-221 (`let root = entities[&src_root]` na 215), não 217-221.

---

### §1.4 — Duplicate de uma entidade aterra EXACTAMENTE em cima da fonte, enquanto o ramo vetorial da MESMA função desloca

| | |
|---|---|
| severidade | **alto** · confiança alta |
| explica do report | `instancias-invisiveis` |
| lente | LENTE 1 — VISIBILIDADE |

**Mecanismo.** Dentro do mesmo bloco `duplicate_row` de `render_loop/hierarchy.rs`, o ramo `VecPathRef` (:249-262) calcula `screen_offset_world(camera, window_size, PASTE_OFFSET_PX)` e passa `dx, dy` a `duplicate_vec_paths`; o ramo genérico (:277-292) chama `crate::instantiate::duplicate_subtree`, que não toca na translação em passo nenhum (shells/desktop/src/instantiate.rs:155-182 — copia, clona documentos, remapeia, renomeia, e devolve). ⇒ duplicar uma sprite, um grupo ou uma INSTÂNCIA produz um objeto perfeitamente sobreposto ao original: o toast diz «Duplicated entity» e a tela não muda. É a MESMA lei que o `cascade` de ontem (instance_verbs.rs:221-246, com o mesmo `PASTE_OFFSET_PX`) pagou para o *Instantiate*, aplicada a uma porta e não à irmã. `duplicate_subtree` tem um único chamador, este.

**Como reproduzir.** cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_INSTANCE_SMOKE=1 cargo run -p ph2d-host-desktop --release — botão direito na linha de uma das três cópias de baixo → *Duplicate*. A Hierarquia ganha uma linha; o canvas fica igual (a nova instância está debaixo da antiga). Arrastar a linha nova para confirmar que ela existia.

**O gate que faltava.** «Uma duplicação de ENTIDADE não aterra na pose da fonte» — o gêmeo exato de `a_placed_instance_never_lands_on_top_of_what_it_came_from`, para o outro verbo. Nenhum gate mede a POSE de uma cópia feita por `duplicate_subtree`.

**O gate que estava VERDE, e porquê.** Nenhum. `render_loop/hierarchy_duplicate_routing_tests.rs` prova que cada TIPO vai para a porta certa (o assunto declarado do ficheiro) e nunca olha onde a cópia ficou; `instance_docs_tests::the_clone_carries_no_offset_because_the_pose_is_in_the_transform` assere o contrário — que o clone do documento NÃO leva deslocamento — mas essa é a lei do path dentro da peça, não a do objeto na cena.

**Veredito do cético.** Medido na worktree `line/components` (HEAD `5adaf0d7b`), e as cinco vias de refutação falharam. (1) `PASTE_OFFSET_PX` tem exactamente DOIS sítios em `render_loop/hierarchy.rs` — :250 (ramo `VecPathRef`) e :350 (o `place_step` que vai ao `instance_verbs::drain`); o ramo genérico não é nenhum deles. (2) `crates/ph2d-ecs/src/instantiate.rs` não menciona `Transform` fora de um doc-comment (linha 64): o `deep_copy_subtree` serializa e reinsere bytes verbatim, e o `Transform` é `register_default`'d em `crates/ph2d-ecs/src/scene/registry.rs:315`, logo viaja intacto; a cópia herda o mesmo `ChildOf` (`shells/desktop/src/instantiate.rs:161`) ⇒ pose de MUNDO idêntica, por construção. (3) O passe por quadro não a resgata: `"ph2d::ecs::Transform"` está em `ROOT_IS_ITS_OWN` (`instance_sync.rs:94`, consumido em :242), então a raiz de uma cópia de instância nunca recebe a pose do mestre — fica onde nasceu, em cima da fonte. (4) O único consumidor de `duplicate_made` (`render_loop/mod.rs:9939-9975`) só bifurca a textura do sprite; não translada nada. (5) A objecção da física não salva o repro: `simulate_physics` é OFF por omissão e em pausa a ponte é read-only sobre `Transform` (`render_loop/mod.rs:2470-2477`) — os dois ragdolls sobrepostos NÃO se empurram no estado de omissão. O gesto é alcançável: o menu da linha é plano (`crates/ph2d-panel-hierarchy/src/event.rs:85-86`, `CTX_MENU_HIER_DUPLICATE` em toda row), `duplicate_kind` (`hierarchy.rs:66-77`) manda sprite/grupo/instância para `DuplicateKind::Entity`, e `duplicate_subtree` tem um chamador de produto só (`hierarchy.rs:282`). ⇒ o mecanismo alegado acontece tal como escrito.

⚠️ **Correções do cético ao achado** (leia-as: em três casos elas MUDAM o preço ou o mecanismo).
Confirmado, com três acrescentos e uma emenda de endereço — nenhum deles refuta, dois AGRAVAM.

**Emenda de endereço.** O ramo vetorial é `hierarchy.rs:249-265` e o `else` genérico abre em **:266** (a cópia é feita em :282), não :277-292.

**Acrescento 1 — a cópia nem seleccionada fica, e a assimetria está no MESMO `if`.** O ramo `Field` faz `hero.gizmo.replace_selection(Some(copy))` em `hierarchy.rs:245` («é o que põe o gizmo em cima dela sem ninguém a ter de procurar»); o ramo genérico (:266-292) não o faz. ⇒ o artista não recebe *nem* deslocamento *nem* gizmo: o retorno inteiro do gesto é o toast «Duplicated entity» e uma linha nova na Hierarquia. São DUAS leis que a mesma função aplica a duas portas e não à terceira.

**Acrescento 2 — a nota «é a mesma lei do cascade» é verdadeira no princípio e FALSA na peça.** O `cascade` (`instance_verbs.rs:239-247`) desloca por `instances_of(master) - 1` (:210-217), que conta raízes com `InstanceOf { master }`. Um *Duplicate* de um sprite ou de um grupo não tem mestre nenhum para contar, e um *Duplicate* de uma instância contaria as irmãs do mestre — que não é a pergunta («quantas cópias DESTA fonte já existem?»). ⇒ a porta irmã não pode reutilizar `cascade` verbatim; ela precisa de um degrau plano de tela ou de um contador próprio. Quem ler «aplica a mesma lei» como copy-paste escreve o defeito de novo com outro sinal.

**Acrescento 3 — o repro é válido, com a fronteira nomeada.** `PH2D_INSTANCE_SMOKE=1` põe o mestre em `y = 3.4` e as três instâncias em `y = 1.2` (`instance_smoke.rs:37-39`), logo «uma das três cópias de baixo» está certo. Em repouso a cópia fica exactamente sobreposta. ⚠️ Se o artista ARMAR a física na barra de transporte e der Play, os dois braços dinâmicos sobrepostos separam-se e a cópia aparece — ou seja, o defeito lê-se como «a cópia não apareceu» precisamente no estado de omissão, e «desaparece» quando ele mexe em algo que não tem nada a ver. É a forma que produz o report do Enio.

**O GATE QUE FALTAVA — e ele existe, verde, com o nome do fim e a asserção da marca.** `the_duplicate_lands_beside_its_source` (`shells/desktop/src/instantiate_tests.rs:409-430`): o NOME promete o canvas («lands beside its source»), o CORPO afirma `ChildOf(copy) == host` (:425-429). Ele mede o **slot na árvore**, não o **lugar na tela**, e fica verde sobre uma sobreposição perfeita. *Um gate cujo nome diz o fim e cuja asserção lê a marca é pior que gate nenhum: ele consome o nome que o gate certo usaria.*

O segundo, por fixtura sem o fenómeno: `duplicating_a_rig_brings_the_whole_subtree_and_its_own_pin` (:328-362) chama `duplicate(...)` e **imediatamente** insere `Transform::from_translation(2.0, 1.2)` na cópia (:338-340) antes de simular; a asserção final `(ax - bx).abs() > 1.0` (:357-361) mede a separação que o TESTE fabricou, não a que a porta produz. Apagar o offset do produto não a mata — apagar as duas linhas 338-340 mataria.

O terceiro, por âncora: o arch-gate `shells/desktop/tests/a_placed_instance_lands_a_screen_step_from_its_main.rs:31-56` ancora em `if let Some((new_id, main)) = arm_instance_of` dentro de `render_loop/mod.rs` — o *Place* do componente VETORIAL antigo. Ele não pode ver a row de *Duplicate* da Hierarquia: está noutro ficheiro e noutro subsistema. O ficheiro chama-se «a placed instance lands a screen step from its main» e o `PASTE_OFFSET_PX` tem outro consumidor sem gate nenhum.

O gate em falta é sobre a PORTA e sobre o FIM: `duplicate_subtree` (ou o dreno da row) devolve uma cópia cuja pose de mundo difere da fonte por um degrau > 0, com o controle negativo a ser o próprio produto de hoje.

**Confiança: alta.** Não corri o app — a prova é estática mas fechada: 28 linhas de `duplicate_subtree` sem uma escrita em `Transform`, `deep_copy_subtree` a copiar bytes verbatim de um componente registado, mesmo `ChildOf`, e nenhum passe a jusante a mexer na raiz. O que a mudaria: um passe de que não sei o nome, a correr entre o dreno da Hierarquia e o `sim_extract`, que translade entidades recém-nascidas — grepei `PASTE_OFFSET_PX`/`screen_offset_world` na árvore inteira e ele não existe.

---

### §1.5 — A raiz de uma receita que seja GRUPO não tem anel, não tem gizmo e não se pega — nem enquanto está a ser EDITADA

| | |
|---|---|
| severidade | **medio** · confiança alta |
| explica do report | `master-invisivel` |
| lente | LENTE 1 — VISIBILIDADE |

**Mecanismo.** `group_gizmo_view::is_empty_object` (shells/desktop/src/group_gizmo_view.rs:91) exige `w.get::<MasterPiece>(e).is_none()` e **não conhece `MasterEditing`** — é a única leitura de `MasterPiece` no repo que ficou com metade da lei da F4.6. Três consumidores morrem com ela: `empty_objects` (:113-138) → o anel (`render_loop/empty_object_overlay.rs:62`); `pick_empty_at_world` (:148-162) → o dedo (`hover_highlight.rs:121`); `view` (:179-181, `if !is_empty_object { return None }`) → o `GizmoView` que `render_loop/snapshots.rs:504` publica. ⇒ com a receita revelada por `MasterEditing`, as PEÇAS com sprite voltam a desenhar e a ser pegáveis, mas a RAIZ não tem anel, não tem caixa nem alças, e não responde ao clique: mover/girar/escalar a receita inteira é inalcançável por gesto de canvas. Toda receita criada por *Make Component* sobre um grupo ou um rig é exatamente esta forma (a raiz do ragdoll é `Transform`+`Name`+`MasterRoot`, sem `Sprite` — instance_smoke.rs:49-57). E fecha-se com o `pick_order::start_on_selection` (shells/desktop/src/pick_order.rs:114-129): a metade `inside_its_gizmo` só entra em jogo quando o press é um Translate no gizmo PRIMÁRIO, e a receita não tem gizmo nenhum ⇒ essa rede também não a apanha.

**Como reproduzir.** cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_INSTANCE_SMOKE=1 cargo run -p ph2d-host-desktop --release — clicar na linha `Ragdoll` na Hierarquia. Os dois sprites da receita aparecem lá em cima; não há anel no centro nem caixa de gizmo à volta, e clicar no centro (entre os sprites) não pega a receita — pega o vazio e a receita desaparece no quadro seguinte.

**O gate que faltava.** A outra metade da lei, com a MESMA forma do gate de `off_canvas`: «com a receita a ser editada, a raiz dela volta ao censo, ao anel e ao gizmo» — três assertivas sobre `empty_objects`, `pick_empty_at_world` e `view`, depois de `master_editing_mark_for_tests`.

**O gate que estava VERDE, e porquê.** `group_gizmo_view::tests::what_draws_itself_and_what_is_not_on_the_canvas_get_no_ring` (shells/desktop/src/group_gizmo_view_tests.rs:216-259, verde agora). A fixtura carimba `MasterRoot` + `assign_master_pieces` e **nunca** carimba `MasterEditing` — nem o tipo aparece no ficheiro. Ela afirma a metade «a receita não ganha anel», que é verdadeira, e o controlo positivo dela (tirar o `MasterRoot`) prova o oposto da metade certa. Fixtura sem o fenómeno: desde a F4.6 a lei tem dois estados e o gate só conhece um.

**Veredito do cético.** CONFIRMADO no essencial, medido na worktree `line/components` (HEAD 5adaf0d7b). (1) `assign_master_pieces` (crates/ph2d-ecs/src/master.rs:83-97) semeia `want` com as PRÓPRIAS raízes (`stack = roots`; `want.insert(e)`) ⇒ a raiz de uma receita carrega `MasterPiece`. (2) `group_gizmo_view::is_empty_object` (shells/desktop/src/group_gizmo_view.rs:84,91) exige `MasterPiece.is_none()` e NÃO consulta `MasterEditing`; grep de `MasterEditing` no repo inteiro dá exactamente dois leitores — `render_loop/master_editing.rs` (quem escreve) e `render_loop/off_canvas.rs:44` — e `group_gizmo_view` não está lá. (3) Os três consumidores caem juntos: `empty_objects` (:121, filtro em :130) → anel (render_loop/empty_object_overlay.rs:62); `pick_empty_at_world` (:148) → dedo (hover_highlight.rs:121, e é a ÚNICA porta pela qual um grupo entra na lista de hits, logo `pick_order::descendants_first` nunca o alcança por ciclo); `view` (:170,179 `!is_empty_object ⇒ None`) → o `GizmoView` de render_loop/snapshots.rs:504. (4) A raiz do ragdoll é `Transform+Name+MasterRoot`, sem `Sprite` (instance_smoke.rs:52-58), portanto cai no ramo final de `build_view` que delega ao `group_gizmo_view` — toda receita criada por *Make Component* sobre grupo/rig tem esta forma. (5) A rede do `pick_order::start_on_selection` está mesmo fechada: `input_dispatch.rs:5392-5397` passa `inside_its_gizmo = matches!(gizmo_kind, Some(Translate))`, e sem `GizmoView` publicado não existe região Translate no hit-index; o fallback `hits.is_empty() && Translate` de input_dispatch.rs:5375-5380 é letra morta pela mesma razão. (6) Corri os dois conjuntos de gates e estão VERDES sobre o buraco: `group_gizmo_view::tests` 9/9 e `render_loop::master_editing::tests` 3/3.

TRÊS AFIRMAÇÕES DA LENTE ESTÃO ERRADAS (detalhe em `correction`): a exclusividade do leitor, a geometria do repro, e o preço.

GATE QUE FALTA — sobre o FIM e não sobre a marca: `MasterRoot` grupo + `assign_master_pieces` + `master_editing::mark(Some(root))` ⇒ `empty_objects()` contém a raiz E `pick_empty_at_world(centro)` devolve-a (controlo negativo: sem `MasterEditing` continua fora). Porque os existentes eram verdes: `what_draws_itself_and_what_is_not_on_the_canvas_get_no_ring` (group_gizmo_view_tests.rs:216-258) é FIXTURA SEM O FENÓMENO — nunca carimba `MasterEditing`, e afirma a ausência do anel, que era a resposta certa antes da F4.6; os três de `master_editing_tests.rs` usam `is_off_canvas` como ORÁCULO ÚNICO (o próprio doc do ficheiro o diz), logo medem que a receita volta a DESENHAR e nunca que ela volta a ser AGARRÁVEL — a mesma metade que faltou ao código faltou ao gate.

CONFIANÇA: alta no mecanismo (é leitura directa de código, com grep exaustivo dos leitores das duas marcas e os gates verdes corridos). Média na severidade sentida: não pude correr o app (só medi headless). O que a mudaria: um smoke real com `PH2D_INSTANCE_SMOKE=1` mostrando se a Hierarquia oferece alguma outra alça sobre a raiz, e a confirmação de que a secção Transform do Inspector é de facto pintada para um grupo seleccionado (li o pintor, não o call site condicional).

⚠️ **Correções do cético ao achado** (leia-as: em três casos elas MUDAM o preço ou o mecanismo).
O mecanismo está certo; a lente erra em três pontos, e o terceiro muda o preço.

1) «é a única leitura de `MasterPiece` no repo que ficou com metade da lei da F4.6» — FALSO. Há mais quatro leitores sem `MasterEditing`: `crates/ph2d-physics-ecs/src/bridge.rs:105` (`type NotAMaster`), `bridge/rope.rs:32`, `bridge/dispatch.rs:40` e `master::count_simulatable`. Esses estão certos DE PROPÓSITO e têm gate (`crates/ph2d-physics-ecs/tests/a_master_is_inert.rs`): uma receita não pode simular nem enquanto é editada. A frase exacta é: *entre os leitores que respondem «esta entidade está na cena?», o `off_canvas.rs` recebeu a 2.ª metade da lei e o `group_gizmo_view.rs` ficou com a 1.ª*.

2) O REPRO está errado na geometria, e a consequência que ele descreve não acontece. Em `PH2D_INSTANCE_SMOKE=1` a raiz está em `MASTER_AT = (0.0, 3.4)` e o `Hub` é filho com `Transform::IDENTITY` (instance_smoke.rs:37,60) ⇒ o sprite 0,3×0,3 do Hub cobre EXACTAMENTE o centro da raiz. Clicar «no centro, entre os sprites» pega o Hub — que é peça da MESMA receita, logo `master_root_of(Hub) == root` (master.rs:127) e `master_editing::mark` mantém a sub-árvore marcada: **a receita NÃO desaparece no quadro seguinte**. Só um clique que não atinja arte nenhuma dá `picked = None` (input_dispatch.rs:5419) e apaga a selecção. O gesto que expõe o buraco é (a) escolher a linha `Ragdoll` na Hierarquia e observar que nenhuma caixa/alça é publicada, ou (b) clicar dentro do disco do marcador (raio `2·HANDLE_SIZE_PX / pixels_per_meter`) mas FORA dos dois sprites — aí o clique atravessa, a selecção cai e a receita some.

3) O PREÇO é menor do que «mover/girar/escalar a receita inteira é inalcançável» sugere. `ROOT_IS_ITS_OWN` (shells/desktop/src/instance_sync.rs:88-92) lista `ph2d::ecs::Transform`: a pose da RAIZ do mestre nunca propaga às instâncias. Logo o gizmo em falta não custa a forma nem a pose das cópias — custa posicionar e identificar a receita no canvas. E existe rota alternativa: a secção *Transform* do Inspector (crates/ph2d-panel-inspector/src/sections/transform.rs) dá X/Y/rot/escala por número para a selecção viva, independentemente de a entidade ter geometria.

4) O QUE A LENTE NÃO DISSE, e é onde o artista de facto se magoa: o *Make Component* deixa a cópia na pose do mestre (a cascata introduzida em 5adaf0d7b é do `Instantiate`, e «a 1.ª cópia fica no ZERO»), portanto mestre e cópia ficam empilhados. Com a receita revelada por `MasterEditing`, ficam duas pilhas idênticas e **só a cópia tem anel e gizmo** (a raiz da cópia é grupo sem `MasterPiece`). O artista vê o anel, agarra a CÓPIA julgando estar a mexer na receita; e a única maneira de afastar a receita da cópia é arrastar as PEÇAS dela — que propagam, porque o `ROOT_IS_ITS_OWN` só cobre a raiz. Separar as duas pilhas move todas as cópias. É esse o sintoma que chega ao report, não «não consigo escalar a receita».

---

### §1.6 — MasterEditing lê só a seleção PRIMÁRIA: a receita fica escondida enquanto a Hierarquia a mostra selecionada

| | |
|---|---|
| severidade | **medio** · confiança alta |
| explica do report | `master-invisivel` |
| lente | LENTE 1 — VISIBILIDADE |

**Mecanismo.** `render_loop/mod.rs:2630` passa `hero_screen.as_ref().and_then(|h| h.gizmo.selection)` — o primário, nunca `extra_selection`. Duas rotas comuns deixam a receita selecionada SEM ser primária: (1) `SelectModifier::Add` / `Toggle` (Shift/Cmd-clique) chamam `add_to_selection` / `toggle_in_selection` (render_loop/hierarchy.rs, ramo `HierarchySelectIntent::Row`) e não mexem no primário; (2) o atalho `preserves_multi` do ramo `Replace` (mesma função) **não** substitui o primário quando já há multi-seleção e a linha clicada já está selecionada. ⇒ a linha da receita fica realçada na Hierarquia e o canvas continua vazio, o que se lê como «cliquei nela e não aconteceu nada».

**Como reproduzir.** Com uma receita e outra coisa na cena: clicar na outra coisa, depois Shift+clicar (ou Cmd+clicar) na linha da receita. As duas linhas ficam realçadas; a receita não desenha.

**O gate que faltava.** «Uma receita que está na seleção — primária ou extra — está na cena»: o `mark` teria de receber o conjunto, e o gate teria de ter um caso com dois selecionados. Hoje `master_editing::mark` só aceita `Option<u64>` e a fixtura de `master_editing_tests.rs` não tem sequer o conceito de extras.

**O gate que estava VERDE, e porquê.** `render_loop::master_editing::tests::the_recipe_comes_back_while_it_is_being_edited` (verde) — a assinatura da função sob teste torna o defeito inexprimível: um gate que só pode passar um bits nunca pode medir o segundo.

**Veredito do cético.** Fui tentar refutar por quatro portas e as quatro fecharam a favor do achado.

(1) `mark` só vê o primário — CONFIRMADO na assinatura, não só na chamada. `render_loop/master_editing.rs:37` é `pub(super) fn mark(sim: &mut SimWorld, selection: Option<u64>)`: um `Option<u64>` não consegue exprimir uma multi-seleção nem que o chamador quisesse. `render_loop/mod.rs:2630` passa `hero_screen.as_ref().and_then(|h| h.gizmo.selection)`. Não há outro escritor de `MasterEditing` em lado nenhum (grep: os únicos sítios são `master_editing.rs:54/61` e a definição em `ph2d-ecs/src/master.rs:70`), e o único leitor é `off_canvas.rs:43`. ⇒ nada compensa a jusante.

(2) O gesto é alcançável — a linha da receita EXISTE e é clicável. `ph2d-ecs/src/scene/snapshot.rs:149` (`build_hierarchy_snapshot`) percorre `(With<Transform>, Without<ChildOf>)` e **não filtra `MasterRoot`/`MasterPiece`**; `hero_bridge.rs:sync_from_snapshot` transcreve tudo. Logo a receita escondida do canvas continua listada na Hierarquia.

(3) A linha FICA REALÇADA enquanto é extra — `render_loop/snapshots.rs:250-256`: `for bits in hero.gizmo.iter_selected() { entry.selected = true }`, e `iter_selected` (ph2d-editor-core/screens/hero/state.rs:239) é primário **encadeado com** `extra_selection`. Ou seja: a Hierarquia diz «está selecionada» e o canvas continua vazio. O cabeçalho, esse, continua a dizer o nome do PRIMÁRIO (`snapshots.rs:275-284`), que é a outra coisa — duas superfícies a discordar.

(4) O primário fica mesmo intacto — `state.rs:274 add_to_selection` (com primário posto e bits novo ⇒ `extra_selection.push`, `return`), `state.rs:290 toggle_in_selection` (cai em `add_to_selection` na linha 303), e `hierarchy.rs:449-453 preserves_multi` (`selected_len() > 1 && is_selected(bits)` ⇒ **não** chama `replace_selection`).

Gate: os três testes de `master_editing_tests.rs` chamam `mark(&mut sim, None)` / `Some(piece)` / `Some(root)` / `Some(loose)` — **fixtura sem o fenómeno, e por construção**: a assinatura de `mark` não aceita um conjunto, então a multi-seleção é inexprimível no arnês. O doc do ficheiro gaba-se, com razão, de o oráculo ser `is_off_canvas` (o FIM, não a marca) — mas a ENTRADA continua a ser a API que o autor escolheu. E não existe nenhum gate de costura que corra `hierarchy::dispatch` e `mark` no mesmo mundo (grep: zero testes que mencionem `extra_selection` e `Master` juntos).

⚠️ **Correções do cético ao achado** (leia-as: em três casos elas MUDAM o preço ou o mecanismo).
O achado ACONTECE, mas três coisas nele estão trocadas ou por dizer:

**(a) O painel da Hierarquia NUNCA emite `SelectModifier::Add`.** `ph2d-panel-hierarchy/src/event.rs:203-218`: `shift && !cmd` ⇒ `HierRangeSelect`; `cmd` ⇒ `HierSelectRow{Toggle}`; senão `Replace`. Logo o Shift+clique **não** entra no ramo `HierarchySelectIntent::Row{Add}` (hierarchy.rs:455) que o achado cita — entra no ramo `Range` (hierarchy.rs:464-513), que chama `add_to_selection` para todas as linhas do intervalo **excepto a âncora** (a âncora é o primário, `hierarchy.rs:487`, e o comentário diz explicitamente *«anchor never demoted by this gesture»*). O ramo `Add` só é alcançável do canvas (`mod.rs:4169`, `input_drop.rs:158`, `vec_selection.rs:139`) — e a receita não é pickable no canvas porque não emite `RenderInstance`. **O efeito líquido é idêntico** (receita vai para `extra_selection`, primário intacto), mas quem for corrigir pelo file:line do achado vai ao ramo errado. O Shift é ainda PIOR do que descrito: ele arrasta para a seleção todas as linhas entre a âncora e a receita.

**(b) O estado é PEGAJOSO, e é isto que produz o «cliquei e não aconteceu nada» repetido.** O achado trata `preserves_multi` como uma segunda rota; ela é na verdade a **falha da auto-reparação da primeira**. Depois do Shift/Ctrl+clique temos `selected_len() > 1` e a receita `is_selected` ⇒ o gesto natural de correcção (clique simples na linha da receita) é engolido em `hierarchy.rs:451` e **não promove nada**. O artista só sai disto quebrando a multi-seleção noutro sítio (clicar noutra linha, ou vazio no canvas). Isso é um beco sem saída sem nenhuma explicação na tela, e a linha continua realçada o tempo todo.

**(c) Há um atraso de UM QUADRO independente de modificadores, e ele não deve ser confundido com o defeito.** `master_editing::mark` corre em `mod.rs:2630`; o dreno do barramento que escreve `gizmo.selection` corre em `mod.rs:4145-4156` e o `hierarchy::dispatch` em ~`mod.rs:9900`. Um clique simples só acende a receita no quadro N+1. A 16,7 ms é invisível ao artista — mas uma sonda headless de um quadro só leria «invisível» e culparia a rota errada.

**(d) Contexto de alcance que muda a leitura do report do Enio:** `instance_verbs.rs` não toca na selecção em verbo nenhum (grep `gizmo` no ficheiro: zero). Como o menu de contexto da Hierarquia **também não selecciona a linha** (`event.rs:189 try_context_menu_row` volta antes do push de selecção), logo a seguir a *Make Component* o primário continua a ser a entidade que ACABOU de virar `MasterRoot` ⇒ a receita fica `MasterEditing`, **desenhada por baixo da cópia** (dois objectos empilhados), e some assim que o artista clica noutra coisa. É o mesmo acoplamento primário↔visibilidade a bater do outro lado, e é provavelmente o que produz a metade «master fica invisível» do report — não a rota da multi-selecção.

---

### §1.7 — As duas cenas do smoke mandam olhar para uma receita que o código já não desenha — o instrumento produz o report que ele existe para prevenir

| | |
|---|---|
| severidade | **medio** · confiança alta |
| explica do report | `master-invisivel` |
| lente | LENTE 1 — VISIBILIDADE |

**Mecanismo.** `instance_smoke.rs:321` imprime *«receita 'Ragdoll' la' em cima (ela NAO se mexe)»*, `:344` *«escolha 'Ragdoll > Arm' (o de CIMA, a receita) e mude a cor»*, e `:272` *«receita 'Badge' (Box + Label) a' ESQUERDA, longe das copias»* — com `VEC_MASTER_AT` / `MASTER_AT` (`:39`, `:145`) escolhidos precisamente para a receita ficar visível e afastada. Estes textos são anteriores à regra de esconder receitas (`off_canvas.rs`, F4.6). Ao correr, o artista vê 3 pêndulos (cena 1) ou 3 crachás (cena 2) e NADA no sítio que o texto nomeia; a instrução seguinte aponta para pixels que não existem, e a única forma de os fazer existir — clicar na linha da Hierarquia — não está escrita em lado nenhum. ⇒ o smoke, que existe para dar o meio-caminho em vez de «não funcionou», entrega exatamente «o mestre ficou invisível».

**Como reproduzir.** cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_INSTANCE_SMOKE=1 cargo run -p ph2d-host-desktop --release — ler as linhas `[instance smoke 1]` no terminal e comparar com o canvas. Repetir com PH2D_INSTANCE_SMOKE=2.

**O gate que faltava.** Não é um gate de código, é a lei do §0.8 aplicada ao instrumento: o texto do smoke tem de dizer o passo que hoje falta (*«a receita só aparece enquanto a linha dela está selecionada na Hierarquia — clique nela»*). O que se poderia gatear é o censo: «toda cena de smoke que nomeia uma posição de canvas para uma entidade `MasterPiece` também nomeia o gesto que a revela» — mas o barato é reescrever as quatro linhas quando as outras curas entrarem.

**O gate que estava VERDE, e porquê.** `instance_docs_tests::the_vector_smoke_scene_builds_three_copies_with_their_own_art` (shells/desktop/src/instance_docs_tests.rs, verde) mede os `VecPathId` que a cena montou e a igualdade de conteúdo — os INGREDIENTES —, nunca `is_off_canvas` de nada que a cena imprime. É a mesma distância entre a marca e o fim que os outros achados desta lente têm.

**Veredito do cético.** O mecanismo acontece e foi medido, mas a redação erra a cronologia e exagera a consequência. CONFIRMADO: `off_canvas.rs:43-47` esconde `MasterPiece && !MasterEditing`; os dois leitores são `sim_extract.rs:344` e `vec_entities.rs:205`; `mod.rs:2630` chama `master_editing::mark` com `hero.gizmo.selection`, que é `None` no arranque; `instantiate.rs:120` já carimba `assign_master_pieces` no quadro 0 das duas cenas (a cena 2 também em `instance_smoke.rs:189`) e `physics_bridge.rs:60` re-carimba por quadro (⚠️ a chamada em `vec_entities.rs:272` é de TESTE — o único carimbo por-quadro no produto é o da ponte); e `group_gizmo_view.rs:91` exclui `MasterPiece` do anel de objeto vazio ⇒ a receita não tem UM pixel no canvas (nem sprite, nem arte vetorial, nem anel) até a linha ser clicada. Corri os 5 gates da família (`master_editing` + `off_canvas`): todos verdes em 0,00 s, o que confirma a semântica lida. ONDE A REDAÇÃO ERRA — (1) cronologia: «estes textos são anteriores à regra» só vale para a cena 1 (texto em `94cd98066` 08-26 16:45, regra dos sprites em `2896d64d7` 08-26 21:05); o texto da cena 2 nasceu em `44d146b56` 08-27 15:11, JÁ DEPOIS dessa regra, e estava CERTO então porque a cadeia do vetor só passou a consultar `is_off_canvas` no commit de HEAD (`5adaf0d7b`, 08-27 16:50) — o próprio `master_editing.rs:7-8` regista-o («foi assim que a cena 2 do smoke passou: com a receita DESENHADA, longe das cópias»). São DOIS eventos de invalidação, e o segundo é o commit de HEAD, que criou o `MasterEditing` e não tocou em `instance_smoke.rs` (git log confirma: o topo do ficheiro continua a ser `44d146b56`). (2) consequência: «a instrução seguinte aponta para pixels que não existem» é falso como escrito — `'Ragdoll > Arm'` e `'Badge > Box'` são notação de CAMINHO DE HIERARQUIA e `Color & Tint` é uma seção do Inspector; `hierarchy.rs:428` faz `hero.gizmo.replace_selection(...)`, que é exatamente a entrada de `master_editing::mark`, logo clicar a linha traz a receita de volta e o smoke ainda CUMPRE O FIM (provar sync e exceção). O `(o de CIMA)` da cena 1 mantém referente válido: as três instâncias nascem depois da receita, logo a linha `Ragdoll` original precede as cópias entre os quatro `Arm` da lista. O QUE DE FACTO QUEBRA são as linhas DESCRITIVAS, não as instrutivas: `:321` («la' em cima»), `:272` («a' ESQUERDA, longe das copias») e o parêntesis de `:297` («a RECEITA, a' esquerda») — elas descrevem coordenadas vazias, e nada (print, selo na Hierarquia, anel) diz que o clique na linha é o que faz a receita aparecer. O GATE QUE FALTAVA: nenhum gate atravessa `spawn_ragdoll_scene`/`spawn_vector_scene` com `selection = None` perguntando quantas peças emitem desenho no quadro 0. Os dois gates verdes sobre isto (`a_recipe_draws_nothing_root_or_piece`, `the_recipe_comes_back_while_it_is_being_edited`) são verdes PORQUE afirmam que esconder está certo: fixtura de duas entidades feita à mão, nunca a cena do smoke, nunca as strings impressas — a mesma forma do defeito que `2896d64d7` documentou (medir a marca em vez do fim), um nível acima. CONFIANÇA: alta no mecanismo (5 gates corridos, caminho lido ponta a ponta, cronologia tirada do git); MÉDIA na atribuição causal ao clause «Master fica invisível» do report — esse clause é mais provavelmente sobre o PRODUTO antes de o `MasterEditing` existir (HEAD é 16:50 de hoje; o binário release está datado 17:04), não sobre o texto do smoke. O que a mudaria: saber o timestamp da corrida do Enio contra `5adaf0d7b`.

⚠️ **Correções do cético ao achado** (leia-as: em três casos elas MUDAM o preço ou o mecanismo).
O mecanismo é o certo (a receita sai da tela por `MasterPiece && !MasterEditing` em `off_canvas.rs:43`), mas a HISTÓRIA e o ALCANCE não. Não é «os textos são anteriores à regra»: são duas invalidações separadas, e a segunda é o próprio commit de HEAD. A cena 1 ficou obsoleta em `2896d64d7` (08-26 21:05, metade de SPRITES); a cena 2 foi escrita DEPOIS disso (08-27 15:11) e estava correta, porque a metade VETORIAL de `is_off_canvas` (`vec_entities.rs:205`) e o `MasterEditing` só nasceram em `5adaf0d7b` (08-27 16:50) — commit que mudou o que a cena 2 mede e não tocou em `instance_smoke.rs`. E o dano não é «a instrução aponta para pixels que não existem»: as instruções usam caminho de Hierarquia (`Ragdoll > Arm`, `Badge > Box`) e uma seção do Inspector, e o clique na linha (`hierarchy.rs:428` → `gizmo.selection` → `master_editing::mark` em `mod.rs:2630`) É o gesto que traz a receita de volta — o smoke ainda prova o sync. O que quebra são as três linhas DESCRITIVAS (`:321` «la' em cima», `:272` «a' ESQUERDA, longe das copias», e o parêntesis de `:297`), que descrevem coordenadas onde o canvas tem literalmente zero pixels — nem sprite, nem arte vetorial, nem o anel de objeto vazio, que `group_gizmo_view.rs:91` exclui de propósito para `MasterPiece`. O buraco de produto por trás disso é de DESCOBERTA, não de desenho: o `MasterEditing` é de hoje e nenhuma superfície que o artista lê (print do smoke, selo na Hierarquia, chrome de canvas) diz que selecionar a linha é o que acende a receita.

---

### §1.8 — Make Component é ACEITE dentro de uma instância viva quando a entidade não tem InstanceOf própria — e o doc diz o contrário

| | |
|---|---|
| severidade | **baixo** · confiança alta |
| explica do report | `instancias-invisiveis` |
| lente | LENTE 1 — VISIBILIDADE |

**Mecanismo.** `instance_verbs.rs:380-382` `belongs_to_an_instance` delega em `instance_root_of`, e a primeira linha desta (`instance_verbs.rs:364`) é `sim.world().get::<InstanceOf>(clicked)?;` — **retorna `None` sem subir a árvore** quando a entidade clicada não tem elo. O doc-comment em :65-66 promete o oposto: *«a pergunta é sobre a entidade **e os ancestrais dela**»*. Toda peça nascida da cópia profunda tem `InstanceOf` (instantiate.rs:99-106), mas o que for acrescentado DEPOIS não tem: *Add Child* sobre uma peça (hierarchy.rs:295-307), um reparent para dentro da cópia, um path vetorial cunhado por `vec_entities::sync` (vec_entities.rs:92-101) e depois arrastado para lá. ⇒ a recusa `VerbRefusal::InsideAnInstance` não dispara, nasce um `MasterRoot` dentro de uma cópia viva, e essa sub-árvore passa a `MasterPiece` ⇒ **um pedaço de uma instância que estava visível desaparece**, com a instância à volta a continuar a desenhar.

**Como reproduzir.** cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_INSTANCE_SMOKE=1 cargo run -p ph2d-host-desktop --release — botão direito numa PEÇA de uma das três cópias de baixo → *Add Child* (nasce `Child`), depois botão direito em `Child` → *Make Component*. O toast é o de sucesso («Made a component…»), não o aviso «Inside an instance — detach it first».

**O gate que faltava.** O caso ancestral que o doc promete: uma entidade **sem** `InstanceOf` pendurada dentro de uma instância tem de dar `Err(InsideAnInstance)`. E, do lado do fim, «nenhuma sub-árvore de uma instância viva fica `MasterPiece`».

**O gate que estava VERDE, e porquê.** `instance_verbs::tests::make_master_refuses_a_master_and_a_piece_of_an_instance` (instance_verbs_tests.rs:262-277, verde). O comentário em :270 diz *«E uma PEÇA no meio da cópia também: a pergunta é sobre os ANCESTRAIS»*, mas a peça usada (`piece(&sim, inst, "Arm")`) TEM `InstanceOf` — a travessia ancestral nunca corre. O oráculo confirma o caminho curto e assina o caminho longo.

**Veredito do cético.** O MECANISMO confere e fui ao código verificá-lo linha a linha; o que não confere é a CONSEQUÊNCIA — e é ela que dá severidade ao achado.

CONFIRMADO (por isso não refuto):
1. `instance_verbs.rs:364` é um bail literal (`get::<InstanceOf>(clicked)?;`) antes do laço. O laço de `:366-376` sobe por `ChildOf` e NÃO exige elo em cada nível (`is_root` é um teste, não condição de continuação) — logo a linha 364 é exactamente o que apaga a travessia de ancestrais que o doc de `:65-66` promete. `belongs_to_an_instance` (:380-382) devolve `false` para uma entidade sem elo próprio dentro de uma instância viva, e a recusa `InsideAnInstance` não dispara.
2. Alcançável: o menu da Hierarquia é PLANO — `menu_rows.rs:190` (`Add Child`) e `:251` (`Make Component`) estão os dois em `ContextMenuKind::HierarchyRow` sem filtro por tipo de linha. `hierarchy.rs:299-304` faz `spawn((Transform::IDENTITY, Name, ChildOf(parent)))`, SEM `InstanceOf`. Grep do repo inteiro: só `instantiate.rs:105` e `:113` constroem `InstanceOf`; nenhum passe re-carimba filhos nascidos depois. `instance_sync.rs:161-176` só emparelha quem tem elo e NÃO despawna o intruso ⇒ ele sobrevive aos quadros.
3. O gate verde está verde pela razão alegada: `instance_verbs_tests.rs:271-277` usa `piece(&sim, inst, "Arm")`, e toda peça da cópia profunda tem elo próprio (`instantiate.rs:99-106`). A fixtura não contém o fenómeno, e o doc-comment do teste (*«a pergunta é sobre os ANCESTRAIS»*) afirma o que o teste não mede.

REFUTADO (a metade que dava peso ao achado): «um pedaço de uma instância que estava visível desaparece». Não desaparece. Ver `correction`.

O que mudaria a minha posição: um caso em que `instantiate_master` recuse DEPOIS de o `MasterRoot` entrar — mas ele repõe o estado (`instance_verbs.rs:81-83`), e os dois modos de recusa (`WouldNestInItself`, `NotAMaster`) são inalcançáveis aqui (`parent` é o pai do clicado, nunca descendente dele; `assign_missing_stable_ids` corre antes).

⚠️ **Correções do cético ao achado** (leia-as: em três casos elas MUDAM o preço ou o mecanismo).
A CONSEQUÊNCIA está trocada — nada desaparece da tela.

`make_master` lê `parent = ChildOf(entity)` (`instance_verbs.rs:70`) e passa-o a `instantiate_master`; `deep_copy_subtree` copia o `Transform` verbatim (não é `owned_document`) e pendura a cópia em `ChildOf(parent)` (`crates/ph2d-ecs/src/instantiate.rs:216-219`). `is_off_canvas` é PER-ENTIDADE (`render_loop/off_canvas.rs:43-47`), então só a sub-árvore do novo `MasterRoot` sai de cena e a instância nasce no MESMO pai, na MESMA pose, com a sub-árvore inteira — que é o que os gates `make_master_leaves_an_instance_in_its_place` e `the_whole_recipe_leaves_the_canvas_and_the_instance_stays` já provam. No repro literal é ainda mais fraco: o `Child` é `Transform + Name + ChildOf` (`hierarchy.rs:299-304`), sem `Sprite` e sem `VecPathRef` — não desenhava um pixel antes nem depois. O artista vê UMA linha extra na Hierarquia e o toast de sucesso, e mais nada.

O dano real é de ESTADO, e é de segunda ordem:
(a) `instance_sync.rs:161-176` — a travessia da instância EXTERNA percorre `Children` e emparelha qualquer entidade com `InstanceOf` cujo mestre resolva. A cópia interna (elo → o `Child` promovido, que é `MasterRoot`) cai DENTRO dessa travessia ⇒ ela ganha um `Pair` no `Live` da instância de fora ao mesmo tempo que é raiz do seu próprio `Live`, e chaves de override de duas instâncias diferentes passam a coabitar o `ObjectInstance` da raiz externa (`instance_sync.rs:213-216`).
(b) `detach` (`instance_verbs.rs:117-122`) varre `subtree(root)` e arranca `InstanceOf` de TUDO — destacar a instância de fora corta em silêncio o elo da instância aninhada lá dentro.
(c) O mesmo buraco aceita `Make Component` sobre uma PEÇA DE UMA RECEITA (a peça não tem `InstanceOf`, e `AlreadyAMaster` só olha a própria entidade) ⇒ `MasterRoot` aninhado dentro de `MasterRoot`, que o F5 também não suporta.

Nada disto explica o report do Enio («master fica invisível, instâncias ficam invisíveis»): esta rota não esconde nada que estivesse visível.

---

## §2 — O que ficou por julgar

- **Dois motores de instância no mesmo quadro, em fases opostas: o antigo cozinha a `8348`, o novo propaga a `main.rs:1180`** — O sistema VECTORIAL antigo (`VecComponentMain`/`VecInstance`) corre `instance_live.recook(vec_scene, sim, &self.vec_entities, &vec_xf)` em `mod.rs:8348` — «o mestre desenhado na pose de cada cópia», produzindo `LiveGeometry` que entra no `vec_live` a `mod.rs:8380` e é desenhada NESTE quadro. O sistema novo escreve o documento em `main.rs:1180` (`instance_sync_docs::write_content`, `instance_sync_d


---

## §3 — Os QUATRO refutados (⛔ não os reconstrua)


*Cada um destes parecia um defeito e não é. A razão da refutação vale mais que o achado: ela nomeia
a guarda que existe, e é a que uma segunda auditoria vai querer encontrar antes de os re-levantar.*

### ⛔ O olho da Hierarquia MENTE sobre a receita — e clicá-lo (o gesto óbvio) torna-a irrecuperável pelo modo de edição

**A alegação.** `HierarchyEntry.visible` é construído como `!vis.is_some_and(|v| v.hidden)` (crates/ph2d-ecs/src/scene/snapshot.rs:188) e NÃO consulta `MasterPiece`. Desde a F4.6 há duas razões para não desenhar e o painel só conhece uma ⇒ a linha da receita mostra o olho ABERTO enquanto o canvas está vazio: nada na tela explica o desaparecimento. O gesto natural do artista é então clicar o olho: `render_loop/hierarchy.rs:150-165` insere `Visibility { hidden: !was_hidden }` com `was_hidden == false` ⇒ escreve `hidden = true`. A partir daí `is_off_canvas` (off_canvas.rs:45-47) devolve `true` pelo ramo do OLHO, e o `MasterEditing` — que só neutraliza o ramo do `MasterPiece` (:43-44) — deixa de revelar a receita: selecionar a linha já não a traz de volta. O artista tem de clicar o olho outra vez, sem nenhuma

**Por que é falso.** As tres linhas citadas estao corretas e o caminho E' alcancavel (verifiquei: `ph2d-panel-hierarchy/src/event.rs:163-166` emite `HierToggleVisibility` para o olho de QUALQUER linha, e ha' ZERO referencias a `MasterRoot`/`MasterPiece`/`MasterEditing` em todo o `ph2d-editor-core` e nos crates de painel — nenhuma guarda; `Badge` e' `Transform+Name+MasterRoot` sem `ChildOf` (instance_smoke.rs:162-168), logo e' linha de raiz sob a query de snapshot.rs:113). Mesmo assim o achado cai nas suas DUAS afirmacoes portantes.

(1) A PRECEDENCIA NAO E' UM BURACO — E' LEI DECLARADA, COM GATE QUE A AFIRMA PELO NOME. `shells/desktop/src/render_loop/master_editing_tests.rs:71-84`, `a_loose_object_lights_nothing_and_the_eye_still_wins`: ele carimba `MasterEditing` na raiz, insere `Visibility::hidden()` e AFIRMA `is_off_canvas(root) == true`, com a mensagem «o modo de edicao passou por cima do olho da Hierarquia» e o comentario «O olho fechado esconde mesmo a receita que esta' a ser editada — ele e' autoria do artista.» O achado pergunta «o gate que faltava»; o gate EXISTE, esta' verde, e esta' verde porque codifica esta precedencia DE PROPOSITO: estado autorado (o olho) ganha de estado derivado (a marca de selecao) — a mesma lei do `ROOT_IS_ITS_OWN` (instance_sync.rs:104-110) e do corte `MasterPiece` vs `Visibility` documentado em off_canvas.rs:31-33. Chamar-lhe defeito e' recusar a cerca sem ler o motivo.

(2) «IRRECUPERAVEL / SEM NENHUMA RAZAO PARA SUSPEITAR DO OLHO» E' REFUTADO PELA PROPRIA LINHA QUE O ACHADO CITA. Depois do clique `hidden == true`, entao snapshot.rs:188 (`visible: !vis.is_some_and(|v| v.hidden)`) calcula `visible = false` e a linha passa a desenhar o olho FECHADO. O controlo que o artista acabou de tocar exibe o seu proprio estado novo, na mesma linha, no mesmo painel, no quadro seguinte. A recuperacao e' um clique no unico controlo que mostra estar accionado. Isso e' autoria reversivel corrente, nao uma armadilha — e e' o oposto da premissa do achado, que e' «o painel nao lhe da' nada de que suspeitar».

O QUE SOBREVIVE (mais fraco, e nao e' este achado): o olho de uma linha de receita le-se ABERTO enquanto nada e' desenhado, porque `HierarchyEntry` tem UM bit para uma pergunta que desde a F4.6 tem duas razoes. Isso e' AFFORDANCE EM FALTA (nao ha' selo/esmaecido para linhas `MasterPiece`, nada na tela diz «isto e' receita, so' desenha enquanto selecionada»), nao uma mentira do olho: o olho reporta o estado do componente que ele comanda, e reporta-o certo nos dois sentidos. Nao verifiquei no app a corrida (nao rodei o binario) — mas o mecanismo alegado e' uma expressao booleana pura e o gate acima fixa-a.

### ⛔ Make Component deixa a cópia EXACTAMENTE em cima da receita — e revelar a receita para a editar põe dois objetos idênticos empilhados, com o clique a preferir a cópia e a esconder o mestre

**A alegação.** `Verb::Make` (shells/desktop/src/instance_verbs.rs:283-300) **não** chama `cascade`; a cópia leva o `Transform` verbatim (instantiate.rs:86-90 declara-o: *«a cópia profunda leva o `Transform` verbatim, então a instância nasce no lugar»*). `Verb::Place` (:303-322) cascateia com `PASTE_OFFSET_PX`. Enquanto a receita está escondida a sobreposição é invisível — mas a ÚNICA porta para editar a receita é selecioná-la (`render_loop/master_editing.rs:36-44`), e selecioná-la fá-la reaparecer debaixo da cópia nº1, no mesmo pixel. Aí: (a) o clique de canvas resolve por `pick_sprites_at_world`, que faz `hits.reverse()` (crates/ph2d-render/src/picking.rs:152-153) ⇒ o último spawnado — a INSTÂNCIA — vem primeiro; (b) `pick_order::start_on_selection` não consegue preferir a receita, porque a raiz dela nã

**Por que é falso.** A guarda existe e o mecanismo está trocado no elo decisivo. O achado afirma que, com a receita revelada, «o clique de canvas resolve por pick_sprites_at_world, que faz hits.reverse() ⇒ a INSTÂNCIA vem primeiro» e que «start_on_selection não consegue preferir a receita, porque a raiz dela não produz hit nenhum». As duas metades caem no repro que ele próprio escreve (uma sprite → Make Component).

(1) MESMA FONTE PARA AS DUAS DECISÕES. `render_loop/mod.rs:2630` alimenta `master_editing::mark(sim, hero.gizmo.selection)`; `input_dispatch.rs:5392` alimenta `pick_order::start_on_selection(&mut hits, hero.gizmo.selection, …)`. É o MESMO campo. Portanto «a receita está desenhada» e «a receita é a preferência do 1.º clique» são a mesma condição — a visibilidade não pode ser ligada sem a preferência do pick também ser.

(2) A GUARDA. `pick_order.rs:115-131` (`start_on_selection`, Enio 2026-08-26 — «o primeiro clique é de quem já está selecionado») procura o selecionado por IDENTIDADE (`hits.iter().position(|&b| b == sel)`) e devolve o índice dele; `input_dispatch.rs:5410` põe `cycle_pick_idx = cycle_start` e `:5425` faz `picked = hits[cycle_pick_idx]`. A ordem que `hits.reverse()` (picking.rs:152-153) produziu é IRRELEVANTE: ela decide só a posição na lista, não quem é escolhido. `descendants_first` (`:5368`) também não interfere — o mestre e a instância são IRMÃOS (o `instantiate_master` recebe o mesmo `parent`), e a função só desempata pai↔filho. `bare_click` é true no gesto descrito (sem Shift/Ctrl/Cmd, `:5389`).

(3) A RAIZ DA RECEITA PRODUZ HIT, no repro alegado. Com `MasterEditing` carimbado, `off_canvas.rs:43-48` devolve false, `sim_extract.rs:344` deixa de a marcar `hidden` e ela ganha `RenderInstance` no `present` — que é exactamente o que `pick_sprites_at_world` consulta. Idem para uma receita vetorial: `vec_entities.rs:205` (`visible_chain`) chama a MESMA porta, logo o path do mestre entra em `vec_gizmo_view::pick_all_at_world`. ⇒ o clique no pixel empilhado devolve `hits = [instância, mestre]` e `start_on_selection` escolhe o MESTRE.

(4) LOGO, O ELO (c) É O CONTRÁRIO DO ALEGADO. O clique reescreve `hero.gizmo.selection` para o MESMO mestre (`replace_selection`, `input_dispatch.rs:5499`), o `mark` do quadro seguinte re-carimba `MasterEditing`, e a receita NÃO evapora. Chegar à cópia exige TRÊS cliques no mesmo ponto (o par/ímpar do ciclo, `input_dispatch.rs:5405-5418`) — que é o clique-cíclico documentado, não um defeito.

(5) E o clique na LINHA da Hierarquia põe mesmo o mestre como primário: `render_loop/hierarchy.rs:430` e `:452` (`replace_selection(Some(entity_bits))`), o mesmo campo que o `mark` lê.

MEDIÇÃO EXECUTADA na worktree `line/components` (5adaf0d7b): `cargo test -p ph2d-host-desktop --bins master_editing` → 3 passed (inclui `the_recipe_comes_back_while_it_is_being_edited`, cujo oráculo é `is_off_canvas` e não a marca); `--bins pick_order` → 10 passed (inclui `the_first_click_starts_on_what_is_already_selected`, com a mutação declarada «devolver sempre 0 ⇒ RED»).

O QUE CONFIRMEI DA PREMISSA, e só isso: `Verb::Make` não cascateia e a cópia nasce na pose do mestre. Mas isso não é um descuido — é uma decisão GATEADA: `instance_verbs_tests.rs:435` (`a_placed_instance_never_lands_on_top_of_what_it_came_from`) afirma os três lados («a 1.ª cópia sai um passo, a 2.ª sai dois, e o Criar componente NÃO desloca»), corre pelo DRENO e não pela função, e nomeia a mutação inversa: «cascatear no `Verb::Make` ⇒ RED». O doc de `cascade` (`instance_verbs.rs:222-238`) explica porquê: a contagem `instances_of − 1` já dá zero para a primeira, então cascatear ali seria um no-op.

O QUE FALTARIA PARA O CONFIRMAR: uma medição que mostrasse `hits` SEM o mestre no caso sprite — ou seja, que `sim_extract` não emitisse `RenderInstance` para uma entidade `MasterEditing`. Não é o caso (`off_canvas.rs` é a única porta, e tem dois leitores). Ou um caminho de clique que salte `start_on_selection` — os únicos são os cliques COM modificador (`:5389`), que o achado não invoca, e o `hover_highlight::pick_hovered_object`, que só pinta realce.

### ⛔ Ctrl+Z apaga a receita da tela: o restore limpa a seleção, e a seleção é o único interruptor do MasterEditing

**A alegação.** `apply_project` chama `hero.gizmo.clear_all_selection()` (shells/desktop/src/undo.rs:285) porque o respawn dá bits novos. No quadro seguinte `master_editing::mark(sim, None)` (render_loop/mod.rs:2630) cai no ramo `None => BTreeSet::new()` (master_editing.rs:43) e a metade que desmarca (:59-64) tira o `MasterEditing` de toda a receita ⇒ ela sai da cena. ⇒ **desfazer uma edição da receita esconde a receita**, sem toast e sem nada na tela a explicar. O `MasterPiece` volta sozinho no quadro seguinte (é derivado — `physics_bridge.rs:60`), mas o `MasterEditing` não volta porque o facto de que ele deriva (a seleção) foi apagado. Isto responde a pergunta (b): depois de um Ctrl+Z quem re-carimba `MasterPiece` é o `assign_master_pieces` dentro do dispatch da física, e quem re-carimba `MasterEditing`

**Por que é falso.** A guarda EXISTE, e é exatamente a que o achado não procurou: a seleção não é o único portador do facto — no caminho VETORIAL ela sobrevive ao respawn como `VecPathId` dentro do `vec_pen`, e o `sync_selection` republica os bits NOVOS no mesmo quadro.

Cadeia verificada linha a linha (worktree `line/components`, branch confirmada):

1. `apply_project` NÃO se limita a `clear_all_selection()`. A PRIMEIRA linha dela, ANTES do restore, é `let was_selected = self.vec_pen.selected_paths().to_vec();` (shells/desktop/src/undo.rs:274) e, depois do respawn, `let alive = surviving_selection(&was_selected, &gfx.vec_scene); if !alive.is_empty() { self.vec_pen.select_many(&alive); }` (undo.rs:~296 + `surviving_selection` em undo.rs:156). O doc-comment dela (undo.rs:264-272) diz-o pelo nome: *«Os bits morrem; a SELEÇÃO não precisa morrer com eles»* — foi escrito em 2026-07-18 para o report *«o undo faz os pins sumirem»*, que é o MESMO defeito uma camada acima.

2. O pen tem o path da receita no momento do Ctrl+Z, e não por acaso: clicar `Badge > Box` na Hierarquia põe os bits no gizmo (`hierarchy.rs:363`/`:430`), e no mesmo quadro o ramo 2 do `sync_selection` (vec_selection.rs:148-180) adota-o no pen — `owns_vector(Box)` é `true` (tem `VecPathRef`, vec_selection.rs:48-60) e `selection_paths` (vec_entities_selection.rs:67-79) filtra SÓ por «o path ainda existe na cena», NUNCA por visibilidade. A receita estar off-canvas não a tira do pen.

3. No quadro N+1 o ramo 1 dispara: `vector_active && (pen_now != state.paths || respawned)` (vec_selection.rs:113). `vector_active` é `true` — o gesto do repro («mover um nó») exige `vector_active && mode == DrawMode::Node` (mod.rs:8173/8199) —, e `pen_now != state.paths` é garantido porque o `apply_project` zera o `self.vec_sel = VecSelSync::default()`. Ele resolve os bits por `map.get(id)`, e o mapa restaurado vem de `vec_entities::rebuild_map` (vec_entities.rs:120-127), que indexa TODA entidade com `VecPathRef` — peças de mestre incluídas, sem filtro de visibilidade. ⇒ `gizmo.replace_selection(Some(bits novos))`.

4. A ordem intra-quadro trabalha a favor, não contra: `master_editing::mark` está a indent 8 diretamente em `run_render_frame` (mod.rs:2630) e o `sync_selection` está dentro do `if let Some(hero) = hero_screen.as_mut()` aberto em mod.rs:2699 (chamada em mod.rs:8239) — logo mark corre ANTES, e a republicação chega a tempo do mark do quadro SEGUINTE.

⇒ O saldo real do gesto descrito é UM quadro (~16 ms) com a receita apagada, e no quadro N+2 o `mark` volta a carimbar `MasterEditing` (o `master_root_of` sobe de `Box` para `Badge`, cujo `MasterRoot` é REGISTADO — registry.rs:342 — e viaja no snapshot). A linha volta acesa, porque a Hierarquia lê `hero.gizmo.selection` (hierarchy.rs:146). As duas afirmações observáveis do achado — *«a receita desaparece do canvas»* e *«a linha deixa de estar selecionada»* — não se produzem no gesto que ele manda fazer.

O gate que o achado devia ter grepado antes de escrever *«o facto de que ele deriva foi apagado»* é `the_pins_survive_an_undo` (shells/desktop/src/envelope_pins_tests.rs:284). Ele percorre o caminho REAL — captura → restore com ids novos → `sync_selection` → bits vivos — e eu corri-o agora: `test envelope_live::tests::kind::pins::the_pins_survive_an_undo ... ok`. O par dele está documentado em vec_selection.rs:220-231 (a mutação `.all()`→`.any()`), o que mostra que a distinção «sumiram» vs «morreram» foi desenhada de propósito e é a que salva este caso.

Uma correção de facto ao achado, à parte: as referências de linha dele estão certas (master_editing.rs:43 é o `None => BTreeSet::new()`, :59-63 é a metade que desmarca, physics_bridge.rs:60 é o `assign_master_pieces`, main.rs:1184 é o `drain_project_io` com o `post_frame_undo` uma linha acima) — o erro não é de endereço, é de fronteira: ele mediu o passe e parou, sem perguntar quem mais carrega a seleção.

### ⛔ A marca que decide o que se DESENHA é mantida por um passe que só o dispatch da FÍSICA chama, e nenhum gate nomeia essa dependência

**A alegação.** `assign_master_pieces` tem exatamente um chamador por quadro no produto: `render_loop/physics_bridge.rs:60`, a primeira linha de `dispatch`, com um doc-comment (`:52-59`) que justifica a posição **só** pela ponte de física (*«as SEIS consultas cacheadas da ponte filtram por `Without<MasterPiece>`»*). Desde a F4.6 essa mesma marca decide `is_off_canvas` (off_canvas.rs:43) e portanto os DOIS extractors de arte. Os outros chamadores são todos de gesto (instantiate.rs:120 e :180, instance_verbs.rs:82, instance_smoke.rs:189) e não cobrem reparent (hero_intents/hierarchy.rs), delete, restore de undo (undo.rs:277) nem load de projeto. ⇒ uma mutação que apagasse `physics_bridge.rs:60` compila, passa TODA a suíte de visibilidade (todos os gates chamam `assign_master_pieces` à mão) e no app deixa as

**Por que é falso.** A METADE FACTUAL do achado confere; a metade que o torna um ACHADO não. Medi as três coisas de que ele depende e as três o desmentem como defeito.

1) **A chamada é INCONDICIONAL e corre em TODO quadro, com a física DESARMADA.** `physics_bridge::dispatch` tem um chamador só (`render_loop/mod.rs:2476`) e está a **8 espaços** de indentação — corpo directo de `run_render_frame` (`impl crate::App` em `mod.rs:490`, `fn` em `:491`), sem `if` a envolvê-la (`let simulate_physics = …` em `:2475` é a linha irmã). Os únicos `return` antes dela em toda a função são `:1059` (`let Some(gfx) … else`) e `:1158` (`let Some(host) … else`) — quando disparam **não se desenha nada**, logo não há quadro em que a arte saia sem o passe. E dentro do `dispatch`, `assign_master_pieces` está em `physics_bridge.rs:60`, **antes** do `if !simulate { bridge.hold(…); return; }` de `:91-94`: com o toggle Physics no default (OFF) a marca ainda é refrescada. O achado escreve «só o dispatch da FÍSICA chama» como se fosse condicional à física; não é — é o corpo do quadro, e a física é apenas o vizinho de código.

2) **O passe RE-DERIVA nos dois sentidos, logo staleness não se acumula.** `master.rs:105-112` insere `want.difference(&have)`, `:113-119` remove `have.difference(&want)`. Idempotente e total. Um reparent, um restore de undo ou um load deixam a marca errada **por, no máximo, um quadro** — o seguinte reconstrói o conjunto do zero a partir dos `MasterRoot` + `Children`. «Permanentemente desactualizadas» só é verdade no binário MUTADO, nunca no que shipa.

3) **Não existe caminho de desenho que a contorne.** `sim_extract::run` tem **exactamente um** chamador (`mod.rs:2631`), 155 linhas DEPOIS do dispatch, na mesma função; `master_editing::mark` fica entre os dois (`:2630`). O leitor vectorial (`vec_entities::visible_chain`, `:197`) chama `off_canvas::is_off_canvas` e é alcançado por `view_state` (`:174`) no pinte, ainda mais tarde. `run_render_frame` tem um único chamador em todo o repo: `main.rs:1167` (o event loop real) — nenhum arnês de teste o conduz. Portanto, em qualquer quadro que produza pixels, `assign_master_pieces` já correu nesse mesmo quadro, antes.

4) **A alegação implícita de «lugar frágil / de arrumação» também cai.** O bloco de manutenção por quadro das irmãs (`assign_missing_root_order` `:8036`, `assign_missing_stable_ids` `:8052`, `assign_missing_sibling_order` `:8053`) corre **DEPOIS** do `sim_extract` de `:2631`. Ou seja: `assign_master_pieces` **não podia** viver lá — mudá-la para o sítio «arrumado» é que partiria o desenho. A posição em `:2476` é a única fatia por quadro que serve os dois consumidores (ponte + os dois extractors), e o doc-comment de `physics_bridge.rs:52-59` estar redigido só em termos da ponte é uma lacuna de PROSA, não de mecanismo.

5) **E não explica nenhum dos três sintomas do Enio.** «Master fica invisível» é o comportamento PROJECTADO (`off_canvas.rs:43-44`: `MasterPiece && !MasterEditing`) — se a marca estivesse em falta, o mestre ficaria **visível a mais**, o contrário do report. «Mudar uma instância não muda outra» não passa por `MasterPiece` em sítio nenhum — é `instance_sync`. E «cópias invisíveis» pediria marca a MAIS numa cópia, que é a metade de REMOÇÃO do passe — a que corre todo o quadro.

O que sobra de verdadeiro, e é pouco: nenhum gate prende a chamada por quadro (todos os sítios de teste chamam `assign_master_pieces` à mão; o único ficheiro de teste que sequer nomeia `physics_bridge` é `preview_drive_tests.rs`), e a prova de mutação citada — 4 filtros, 23 verdes — não prova «TODA a suíte» (a lei do repo: um filtro por nome nunca alcança um gate que varre a árvore). Isso é uma nota de cobertura honesta sobre um invariante que hoje está correcto, não um defeito. Para o converter em achado faltaria exibir um quadro real em que a arte é emitida sem o passe ter corrido — e os pontos 1-3 mostram que esse quadro não existe.

O que mudaria a minha posição: se alguém demonstrasse um segundo caminho de emissão de `RenderInstance` (ou uma segunda construção de `VecViewState`) alcançável fora de `run_render_frame`, ou um `cfg`/feature que remova o `dispatch` do quadro. Grepei os dois e não existem.
