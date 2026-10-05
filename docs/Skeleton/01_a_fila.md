# 01 — A FILA do módulo do esqueleto

> **O que está ABERTO, na ordem em que faz sentido pegar.** Cada item traz o **mecanismo** (ou o
> instrumento que o nomeia numa corrida) e o que as referências fazem — não uma promessa.
>
> ⚠️ **Uma nota de diferido não é uma spec.** O que torna um item desta fila pegável não é ele estar
> escrito: é ele dizer *por onde começar a medir*. Um item sem isso é trabalho a redescobrir.
>
> ⚠️ **A ordem é uma recomendação, não uma decisão.** Quem escolhe é o dono.

---

## ⚠️ ESTE DOC É UM ROTEADOR — a história está ARQUIVADA, verbatim

Ele chegou a **264 KB** (3 608 linhas) por append, uma wave de cada vez, e o joelho medido deste
repo está entre **80 e 110 KB**: acima disso um `Read` deixa de o alcançar e o acesso vira raspagem
por shell — *uma regra na linha 3 000 não é «difícil de achar», ela não é lida por ninguém*
(`CLAUDE.md` §5.0). Em 2026-09-16 ele foi cortado com prova (`scripts/doc-split.py`, remontagem
`sha256` idêntica): o que ficou aqui é **o que está ABERTO** mais o índice das **recusas medidas**.

📚 **As 47 waves fechadas (F1..F6-v) vivem verbatim em**
[`docs/archive/skeleton-fila-2026-09-16/01_a_fila.md`](../archive/skeleton-fila-2026-09-16/01_a_fila.md)
— com o mecanismo, as tabelas e as provas de mutação de cada uma. ⚠️ **Elas são o sítio onde vive
*«medido e REJEITADO»*:** consulte-as (e a tabela de recusas no fim deste ficheiro) **antes** de
propor qualquer mudança de desenho neste módulo.

### ⏳ O que está ABERTO dentro das waves FECHADAS — o endereço de cada um

*Um item aberto dentro de uma wave fechada não deixa de existir por ela fechar.* A lista sai do
arquivo (`grep -n '⏳ \*\*ABERTO' docs/archive/skeleton-fila-2026-09-16/01_a_fila.md`), e cada linha
diz onde ler o mecanismo:

| o quê | onde (linha do arquivo) |
|---|---|
| Com **escala NÃO-UNIFORME** na cadeia o arco do limite distorce-se (é geometricamente correcto; ⛔ não é o que o dono viu) | `497` |
| O **Arrange com pilha**: o `clip_time` responde `None` quando o clip toca zero ou duas vezes — nenhum fantasma, e **não foi smokado** | `2 137` |
| A cerca de *«quem autora no canvas»* vive num FIO e o gate dela é **textual** — a cura de fundo é o `Tool` declarar-se, e ele é contrato **congelado** (§6) | `2 184` |
| ~~Uma cena **muito acima** do orçamento fica com o `Smooth` igual ao `Fast`~~ — ⛔ **a premissa MORREU em 2026-09-17**: não há orçamento por quadro nem duas leis, a densidade é decisão do BIND | `3 015` |
| O campo de Hermite amostrado denso ainda **vai e volta `39,67°`** no lado de cima (ondulação abaixo da tolerância) | `3 196` |
| A `ph2d-poly2d` guarda as **duas** leis de refinamento (`PH2D_SKIN_REFINE=uniforme` bissecta) | `3 324` |

---

## Aberto de waves anteriores (as opções que o dono ainda não escolheu)

| # | O quê | Estado |
|---|---|---|
| F3 | **Smart Bones** (Moho) | ✅ **FECHADO** (2026-09-08) — ver abaixo |
| F4 | **Limites de ângulo por junta** | ✅ **FECHADO** (2026-09-07) — ver abaixo |
| F5 | ~~**Pole target**~~ → **O LADO DA DOBRA** | ✅ **FECHADO** (2026-09-07) — ver abaixo |
| F6 | **A segunda mídia** (raster/Flip) | ✅ **FECHADA para o RASTER** (2026-09-09) — ver F6 abaixo. ⛔ A nota antiga dizia *«bloqueado: precisa de uma malha sobre a imagem, que não existe»*: estava certa sobre o facto e errada sobre o preço — **duas das quatro peças já existiam**, e o doc de uma delas dizia-o por escrito. O **Flip** continua por fazer |
| **F8** | ✅ **BENDY BONES (B-Bones) — FECHADO em 2026-09-15**, da lei ao painel ([handoff](handoffs/HANDOFF_O_OSSO_QUE_DOBRA_2026-09-15.md)) | Um osso ganha `segments` + duas alças e **arqueia**: ele parte-se em `N` sub-ossos ao longo de uma Bézier, o desenho e o dedo seguem a curva, e o painel oferece os dois controlos. ⭐⭐⭐ **A LEI DA PELE NÃO MUDOU UMA LINHA** — o `Skin` já misturava `N` poses RÍGIDAS por peso, que é exactamente o que um B-Bone é; o que mudou foi **quem produz**, e era **um** sítio (`resolve_with`). ⛔⛔ **E esta célula dizia que o B-Bone «ataca na ORIGEM» a queixa das *«arestas retas ao dobrar»* — REFUTADO** pela recusa medida um bloco abaixo (subdividir com a população de amostras constante **piora**: `2,61 % → 4,94 %` a `24` sub-ossos): *o B-Bone é uma feature de AUTORIA — um rabo em S, um membro flexível —, não a cura da dobra.* ⭐⭐ **O ponto neutro é exacto POR CONSTRUÇÃO** (a fábrica colapsa num osso só quando a curva é recta, e mesmo sem colapsar o frame seria a identidade ao bit) ⇒ todo rig já autorado desenha-se e deforma-se **ao bit** como antes. ⚠️ `PROJECT_SCHEMA` **+1** — conte o DELTA. **Tecto MEDIDO: `MAX_SEGMENTS = 32`** (`17,9 %` de um quadro com um osso curvo sobre 20 000 pontos; a `64` um par come o quadro) — a tabela vive no doc da const. ✅ **OS TRÊS ABERTOS FECHARAM EM 2026-09-16.** **(1)** O esticão deixou de VARIAR ao longo do osso — os nós saem agora da **CORDA** e não do parâmetro (`12,63 % → 0,000 %` com as alças a `0,2 L`; `82,01 % → 0,000 %` a `0,6 L`; `1 051,95 % → 0,000 %` com as alças cruzadas no eixo). ⛔⛔ **E a cura publicada — equalizar o ARCO — NÃO chegava**, o que só a varredura da densidade disse: ela deixa um piso que **não desce com a tabela** (`1,22 %` a `0,6 L`, igual de `16` a `32` amostras), porque *arcos iguais dão cordas desiguais* e a grandeza que o artista vê é a corda. ⚠️ **E a objecção registada na recusa era verdadeira e não mordia** (*«um somatório de cordas não devolve `L` ao bit»*): o somatório **nunca corre** no ponto neutro — *uma recusa que nomeia um custo tem de dizer em que CAMINHO ele é pago*. **(2)** As alças **pegam-se no canvas** (duas alças de Bézier, com as hastes até à raiz e à ponta) — ⛔ e a armadilha foi que no ponto NEUTRO a alça está **em cima do eixo**, logo a competição por proximidade de sempre torná-la-ia inalcançável no único estado em que todo osso nasce: ela é a única que ignora o corpo, e paga um raio apertado cujo recurso é o comprimento que sobra para o verbo de girar. **(3)** As **tangentes dos vizinhos** existem (`Curve Handles: Manual | From Chain`), e o ponto neutro é **exacto** porque elas saem da transformação RELATIVA e não de uma volta pelo mundo. ⚠️ `PROJECT_SCHEMA` **+1** — conte o DELTA. Cena **`PH2D_VEC_BONE_SMOKE=1`**. |
| F7 | **O painel próprio do módulo** | ✅ **FECHADO** (2026-09-09, por escolha do dono) — ver F3-m abaixo. A nota antiga: ⏸️ **a condição CAIU e a medição era falsa por ~3×** — ela dizia *«adiado até F3–F5 lhe darem conteúdo (hoje são 3 botões e 5 campos)»*, e as três estão ✅ nesta mesma tabela enquanto a secção tem **10 verbos** e **9 campos** (`VECTOR_BONE_VERBS`/`_FIELDS`, comprimento verificado pelo compilador), mais uma fileira segmentada e dois selectores. ⇒ decisão do dono, não mais um adiamento medido |
| **F9** | ✅ **A PELE DEFORMADA NA GPU** (pedido do dono, 2026-09-16) | ✅ **FECHADA 2026-09-20** (`f53d48138`), por ordem do dono (*«implemente se esse é o padrão ouro»*) — e o padrão-ouro foi **MEDIDO** antes de uma linha ser escrita. `attach_skin_meshes` **`35,6 % → 12,5 %`** de um quadro a 8 imagens (`2,86×`), com o gate de PIXEL a ler **`0 px`** de diferença entre a placa e a CPU. ⛔⛔ **A lei que sobe é a da JUNTA, não a linear** — a 1.ª redacção implementou a antiga e o portão leu `2,315e-3 m`. Ver F9 abaixo
| **F10** | ✅ **O AutoKey com a corrente de ossos** (decisão do dono, 2026-09-16) | ✅ **JÁ ESTAVA FEITO — a nota envelheceu, e auditá-la contra o CÓDIGO custou dez minutos** (2026-09-18). O passe grava **a corrente INTEIRA que a mão moveu** (não só o osso seleccionado) desde 2026-09-14, e também **o ALVO de uma restrição de IK** — porque com uma restrição viva a rotação dos ossos é DERIVADA e o que o artista autora é a âncora. ⚠️ Quem filtra é o **DIFF**: um osso cuja pose é a da curva não cunha nada. Seis gates em [`autokey_bone_tests.rs`](../../shells/desktop/src/render_loop/autokey_bone_tests.rs), entre eles `autokey_records_every_bone_the_hand_moved_not_only_the_selected_one`, `dragging_the_ik_anchor_records_the_anchor` e o controlo `a_bone_the_hand_holds_but_did_not_move_keys_nothing`. ⛔ **O que FALTAVA não era a lei, era o SMOKE:** nenhuma cena do app armava o AutoKey, logo o dono nunca lhe chegou ⇒ cena **`PH2D_VEC_BONE_MEDIA_SMOKE=3`** |
| **F11** | ✅ **Imagens em 9 fatias e folhas de quadros DEFORMAM com os ossos** (ordem do dono, 2026-09-17) | ✅ **FECHADO** — ver F11 abaixo |
| **F21** | ✅ **A cena dedicada do ENVELOPE** (*«melhor montar uma cena específica para me mostrar isso»*, 2026-09-18) | ✅ **FECHADA em 2026-09-19 — e ela REFUTOU a lei da F20**: o envelope é inerte em toda forma FECHADA (amplitude `0,000000` numa faixa de `80 ×`), porque uma forma fechada também usa o padrão-ouro desde 15/09. A lei passou a perguntar ao **BIND** e não à mídia. Cena **`PH2D_VEC_BONE_SMOKE=2`** — ver F21 abaixo |
| **F22** | ⭐⭐⭐ **A ESCOLHA da lei de pele, POR DESENHO** (ordem do dono, 2026-09-19: *«construa. por desenho»*) | ✅ **FECHADA no mesmo dia** — fileira **`Deform By`** (`Artwork` \| `Bone Reach`) no painel Bones, por DESENHO e para as duas mídias. ⭐ A escolha diz se o quadro **LÊ** a tabela do padrão-ouro, nunca se a calcula ⇒ a volta é **exacta ao bit** e não re-resolve nada. `PROJECT_SCHEMA` **+1** — ver F22 abaixo |
| **F30** | ⭐⭐⭐ **A arte segue o peso ENTRE os nós** (a 2.ª saída da F26) | ✅ **CONSTRUÍDA, e a MALHA não foi precisa.** A `ph2d-vec-envelope` já deforma Bézier por um mapa não-afim, e o cabeçalho dela descreve o defeito que a pele tem hoje. Sonda: peso entre dois nós move a arte `0,000000 → 0,242375`, o fit converge, `0,163 ms` em release. ⭐⭐⭐ E ela **dissolveu a compensação da F28** — ver F30 abaixo |
| **F29** | ✅ **Os DOIS modos de atribuir peso** (ordem do dono, 2026-09-19) | ✅ **FECHADO 2026-09-20** (`0029f2fc8`). *Cumulativo* (omissão) e *Absoluto* no painel, `PROJECT_SCHEMA` **+1**. ⭐ A cura é um enum que CARREGA o número (`Especie::Soma(f64)` / `Alvo(f64)`), logo o compilador obriga todo leitor a dizer qual lê; a aplicação SEQUENCIAL faz *«a última manda»* cair de graça. ⚠️ **O caso degenerado é ordem do dono:** com os outros a zero o osso fica com **100 %** seja qual for o valor — e ele só é observável em `v = 0`, achado por uma mutação SOBREVIVENTE
| **F28** | ⭐⭐⭐ **UM PONTO NOVO NUMA FORMA PRESA** (a 1.ª das duas saídas da F26, escolhida pelo dono: *«primeiro 1 e depois o 2»*) | ✅ **FECHADA**, e o smoke dela REPROVOU a 1.ª versão. O ponto sobrevive ao quadro, já nasce com peso, **o desenho não salta** (`18,89 % → 0,000000 %`) e a caneta MOSTRA onde o clique poria o nó. ⛔⛔ Duas conclusões minhas caíram: *«custo zero de arquitectura»* (medido: o ponto evaporava-se) e *«o salto é refinamento»* (o dono recusou — ver F28-b) |
| **F27** | ⭐⭐⭐ **O CENSO DOS VERBOS DO OSSO** (o aberto que a F16 deixou por escrito) | ✅ **FECHADO no mesmo dia — ZERO verbos mortos.** Os catorze botões chegam a um efeito, medidos pela captura que o undo tira. ⛔⛔ E uma **mutação sobreviveu**: apagado o corpo do braço do *Add Smart Bone* na fase do quadro, **23 testes da shell ficaram verdes** — o terceiro elo do §5.0 não tinha instrumento nenhum. Zero schema, zero registo — ver F27 abaixo |
| **F26** | ⭐⭐⭐ **CORRIGIR UM PESO À MÃO** (auditoria, 2026-09-19) | ✅ **FECHADA no mesmo dia** — o 3.º verbo do osso (**`Weight`**) pinta a influência sobre a arte presa, com os pesos **à vista** por baixo do pincel. A correcção é uma **MANCHA no espaço** (nunca uma tabela por vértice) e é ancorada no **REPOUSO** do ponto que o dedo aponta. `PROJECT_SCHEMA` **+1** — ver F26 abaixo |

---

---

### F64 — ⏳ **O ESQUELETO É UM OBJECTO** (A14, ordem do dono 05/10) — plano [`05_plano_o_esqueleto_e_um_objecto.md`](05_plano_o_esqueleto_e_um_objecto.md)

- **M (medição, antes do código):** (1) o mundo de um osso compõe a cadeia `ChildOf` INTEIRA
  (`world_transform` → `parent_world_transform_with`, sem parar no 1.º osso) ⇒ um esqueleto-pai com
  `Transform` move todos os ossos sem código novo; a raiz de corrente já é *«sem pai, ou pai que não é
  osso»* (`bone_pick::is_a_free_chain_root`, `chain_to`). (2) Nenhum código do osso lia o olho
  (`Visibility`). (3) Não há gancho de clique por `ModeFamily`: o despacho escolhe pela FERRAMENTA
  activa ⇒ o modo do esqueleto arma uma ferramenta própria (como o Flip). (4) `DrawMode::Bone` em
  **17** ficheiros (o plano dizia 16). (5) Precedente divergente: o extract de sprites lê o olho POR
  ENTIDADE (`is_off_canvas`), o gizmo do Flip pela ÁRVORE; o esqueleto usa a árvore
  (`ph2d_ecs::is_hidden_in_tree`, nova) — o olho do esqueleto tem de esconder os ossos-filhos.
- **C0 — os ossos desenham-se seja qual for a ferramenta** (2.º report do dono: *«o esqueleto se
  move mas fica invisível»*). O `VecOverlayPlan::bones` (`= vector_active`) saiu; a fase desenha pela
  porta `skin_live::visible_bone_polylines` (sem os que o olho esconde), e o DEDO (`bone_pick` ×4,
  `goal::ring_targets`) lê a mesma porta. A cinemática e a pele continuam na `bone_polylines`
  inteira (escondido continua a deformar, como no Blender). Gates, vermelho visto antes do verde:
  `the_bone_overlay_reads_the_eye_and_never_the_vector_tool` (shell, textual, controlo = a porta
  achada 1×; vermelho sobre a fase do HEAD), `a_closed_eye_on_an_ancestor_hides_its_whole_chain_and_only_it`
  (controlo: olhos abertos = 4), `a_bone_hidden_by_the_eye_is_not_grabbed` (controlo: olho aberto
  agarra). O gate antigo `the_bones_are_drawn_in_every_mode_of_the_vector_tool` saiu com o campo
  (afirmava a lei que o dono refutou). ⚠️ Em Object os ossos aparecem com as alças de hoje; o
  desenho por modo (fino em Object) é o C6.
- **C1 — o TIPO.** Marcador vazio `ph2d_skeleton_ecs::Skeleton` (`skeleton.rs`, `register` sem default:
  chega pelo Add ou pela migração; registo do esqueleto `7 → 8`, contado no gate; os totais 105/106
  da fundação não o somam), descritor `intrinsic` no catálogo, `ObjectKind::Skeleton` ANEXADO ao fim
  de `ALL`, `kind_of` pergunta-o (gate `every_marker_derives_its_kind`, vermelho visto: lia-se
  `Empty`). ⛔ Achado: `ObjectKinds::ANY` era o literal `0b1_1111` e deixava a 6.ª variante sem paleta
  (3 gates vermelhos) ⇒ derivado de `ALL`; `DRAWABLE` exclui o esqueleto (sem pixels). ⚠️ Para o
  C5/C7: a entidade do esqueleto (Transform, sem pixels) é um «objecto vazio» para o
  `group_gizmo_view::is_empty_object` e ganharia o anel — decidir lá.
- ⚠️ **Sequência mudada:** o C2 (`ObjectMode::Pose`) junta-se ao C5 — o gate
  `the_composed_families_declare_every_creation_mode` reprova um modo sem família (D6: modo que não
  abre nada é controlo morto), e a família precisa da ferramenta (C3) e do despacho (C4).
- **C3 — a FERRAMENTA de osso** (`ph2d-tool-bone`, drop-crate sabor 2, `make` registado pelo
  `ph2d-tool-sync`). Saíram da `ph2d-tool-vector`: `DrawMode::Bone` (vocabulário 17 → 16; o
  `SEM_PILL` fica vazio), `BoneAction`, `WeightDirection`, `WeightMode`, as três constantes do
  pincel, os campos do `VectorDrawConfig`/`VectorStyleSnapshot`, os braços do painel e os ids do verbo
  e do pincel (as STRINGS dos hashes ficam `vector.bone.*`). Porta única do segmento → verbo:
  `BoneAction::of_segment`. Ponte `ph2d_app_skeleton::bone_bridge` (`in_hand`, `config`, `arm`); a
  shell espelha `skeleton.tool`/`tool_in_hand` por quadro (`fase_tool_mirrors`) e todos os leitores
  do `draw_config.bone_action`/`weight_*`/`mode == Bone` passaram a ele. O press e o release do osso
  mudaram-se (corpos iguais) para `input_dispatch/despacho_clique_osso.rs`, ramo
  `ramo_ferramenta_osso` chamado antes do vetorial (depois do Select, onde mora o pick modal). Um
  clique em *Create*/*Transform*/*Weight* no painel PÕE a ferramenta de osso na mão antes de o
  evento seguir para a activa (`fase_bus_tool_panel`) — interino até o C5 o trocar pela entrada no
  modo. Gates novos, com controlo: `arming_a_verb_puts_the_bone_tool_in_hand`,
  `the_bone_segment_arms_the_tool_before_the_forward` (vermelho visto sobre a fase do HEAD),
  `the_three_segments_arm_the_verb`, `the_bone_segments_leave_the_vector_tool_alone` (substitui o
  `the_two_bone_segments_are_the_door_to_the_mode`, que afirmava a porta antiga). ⚠️ Para o C6: as
  alças do osso (força, limite, ponta da IK) ACENDEM em Object desde o C0 mas o `Down` delas só se
  lê com a ferramenta Vector (`despacho_clique_select`) — decidir com o desenho por modo.
- **C4 junta-se ao C5:** o que sobra do osso na shell (`despacho_clique_osso.rs`) é aplicação de
  efeitos; a decisão já mora em `bone_gesture::press`. A mudança de natureza do despacho com o modo
  (a raiz do Edit pendura-se no esqueleto) entrou no C5.
- **C5 — o MODO e o Add.** `ObjectMode::Pose` ANEXADO (`ALL` 4 → 5, `object_mode.pose` «Pose Mode»,
  `OBJECT_MODE_POSE`; o `tools_of` do Flip ganha o braço vazio). `ph2d_skeleton_ecs::{skeleton_of,
  bones_of}`. Família `ph2d_app_skeleton::skeleton_mode::Family` (leis puras `mode_of` · `verb_of` ·
  `holds` · `adopt`): Edit = *Create*, Pose = *Transform*/*Weight* (volta ao último), Object larga a
  ferramenta; `parts` = os ossos (o cadeado não tropeça neles), `owner_of` = osso → esqueleto (o `Tab`
  sobre um osso entra no Edit do dono); a ferramenta que chega à mão por outra porta (o segmento do
  painel, a aresta do foco) pede o modo do verbo sobre o esqueleto da selecção. Composta em
  `fase_object_mode.rs` (lê `hero.gizmo.selection`). *Add ▸ Skeleton* (`object_add.rs`): marcador,
  nome que conta, UM osso filho do comprimento de fábrica do componente, pede o Edit. No Edit, um
  osso sem pai-osso nasce filho do esqueleto (`despacho_clique_osso`, `skeleton.target`). ⛔ Achado
  pelo gate `a_fourth_branch_on_the_canvas_wakes_this_law`: o ramo novo partilha o canvas ⇒ a
  ferramenta `"bone"` entra na condição da caixa de objecto (`fase_snapshots_publish`) — sem isso as
  alças do gizmo matavam o gesto (medido nos ossos a 13/09). Agulha de `architecture_the_ik_chain_is_drawn`
  segue a indentação do C0. Gates (vermelhos de fixtura vistos e corrigidos; a mutação vem no fecho):
  `the_pure_laws`, `add_enters_edit_with_create_and_tab_and_pose_follow`,
  `selecting_a_bone_of_the_skeleton_keeps_pose_and_a_foreign_object_does_not`,
  `tab_on_a_bone_enters_the_edit_of_its_skeleton`, `the_verb_in_hand_brings_its_mode`,
  `the_menu_skeleton_is_born_an_object_with_one_bone_and_asks_for_edit`,
  `a_bone_climbs_to_its_skeleton_and_only_its_bones_answer`; `every_object_mode_has_a_composed_family`
  (famílias 3 → 4, pares D6 4 → 6). ⚠️ Os censos do `ph2d-panel-registry-init` só valem com
  `--workspace` (aviso da própria crate) ⇒ ficam para o gate batched.
- **C6 — o desenho e as alças por modo** (Blender: em Object a Armature não se pose). As alças do osso
  (realce, pick, fundo, curvatura, anéis das pontas) só existem com a ferramenta de osso na mão (Edit
  e Pose); em Object os ossos desenham-se sem elas, e as âncoras de IK continuam à vista (são
  objectos). `refresh_bone_hover` limpa sem a ferramenta; o overlay decide `posar` uma vez (o osso em
  foco e os anéis vazios em Object — mesmas chamadas, mesma ordem, os gates de ordem intactos).
  ⛔ SAI o pick das alças «em todo modo de vector» de 08/09 (`despacho_clique_select`,
  `App::bone_handle_at`, `bone_pick::grabbable_outside_bone_mode` e os seus três gates): a lei dele
  era *o que acende tem de responder*, e fora do Edit/Pose nada acende agora. Gates
  `the_bone_handles_belong_to_edit_and_pose.rs` (3, vermelho visto sobre o HEAD, controlo positivo em
  cada).
- **C9 — os projectos antigos: uma PORTA, não uma escada.** `PROJECT_SCHEMA` `184 → 185` (o
  `Skeleton` registado; esqueleto `7 → 8`, ECS e espelhos `0`; tripla `(185, 13, 22)`) — ⛔ sem degrau
  de migração, pela decisão de sempre (um v184 é recusado). A porta
  `ph2d_app_skeleton::loose::adopt_loose_roots`, chamada em todo quadro antes das famílias
  (`fase_object_mode`), põe um esqueleto por cima de cada raiz solta: na IDENTIDADE, no lugar dela na
  Hierarquia (o mesmo pai, a mesma `RootOrder`) ⇒ pose AO BIT. Cobre o projecto antigo, as cenas que
  criam ossos à mão e o 1.º *Create* numa cena sem esqueleto (a família adopta-o no quadro seguinte).
  ⚠️ Desvio do plano: o esqueleto adoptado chama-se `Skeleton` (que conta), não o nome da raiz — dois
  nomes iguais na Hierarquia (o esqueleto e o 1.º osso) seriam ilegíveis. Gates
  `an_old_project_opens_with_one_skeleton_per_root_and_the_pose_to_the_bit` (controlos: 2.ª passagem
  vazia, corrente com esqueleto intocada) e `the_loose_roots_are_adopted_before_the_mode_families`.
  ⚠️ O degrau `185` reconta-se na integração (`python3 scripts/schema-recount.py`): a `line/UIUX` e
  outras linhas sobem o mesmo número.
- **C7 — o ESQUELETO em Object.** Clicar num osso à vista selecciona o esqueleto dono
  (`bone_pick::object_at`, à frente da arte nos `hits` do `ramo_gizmo_pick`; com a ferramenta de osso
  na mão o clique é dela). A entidade do esqueleto é um «objecto vazio» para o
  `group_gizmo_view::is_empty_object` ⇒ anel e gizmo na origem dele — DECIDIDO manter (é o ponto de
  origem da Armature do Blender, e o alvo do gizmo que move o esqueleto inteiro). O Bind junta as
  formas da caneta E as formas-objecto da selecção, e a semente sai do osso escolhido ou do esqueleto
  escolhido (`bone_pick::bind_seed`); `Ctrl+P` empurra o MESMO clique do *Bind to Skeleton* para o
  barramento. ⭐ `skin_live::skeleton_of` de um osso com objecto devolve os ossos do OBJECTO (todas as
  raízes — o Bind, a navegação, a pele de cebola e a timeline perguntam isso), e a recusa «vários
  esqueletos» conta objectos (`esqueletos::esqueletos`). Gates: `moving_the_skeleton_object_carries_the_bound_shape`
  (controlo: parado), `the_skeleton_of_a_bone_is_the_whole_object` (controlo: sem objecto, a
  corrente), `a_bone_under_the_finger_selects_its_skeleton_and_seeds_the_bind` (controlo: fora do
  osso), e os três de costura `the_skeleton_object_is_picked_and_bound_in_object_mode.rs` (vermelho
  visto sobre o HEAD).
- **C8 — o painel Bones por modo.** ⚠️ **Desvio do plano, decidido por uma lei viva:** o plano dizia
  «Edit = só *Create*; Pose = *Transform*, *Weight*; Object = só *Bind*», mas o gate
  `the_create_transform_group_is_the_door_and_starts_with_nothing_lit` guarda a porta do PRIMEIRO
  osso numa cena vazia, e com o C5 cada segmento já leva ao modo dele (*Create* → Edit,
  *Transform*/*Weight* → Pose; numa cena sem esqueleto o 1.º osso é embrulhado pela porta das raízes
  soltas). ⇒ a fileira dos verbos fica sempre — é o seletor do verbo E do modo, acende o do modo em
  curso — e o **`Bind` passa a ser só de Object** (`section::bind_e_oferecido`): em Edit/Pose o
  cadeado não deixa escolher a forma, e ele só saberia recusar. As saídas (*Expand*/*Release*) já só
  aparecem com uma forma presa escolhida, o que em Edit/Pose não acontece. Gate
  `the_two_exits_appear_only_when_something_is_bound` ganhou o controlo «em Pose o Bind some»
  (vermelho visto com a lei mutada para `true`).
- **C10 — as cenas.** As `PH2D_VEC_BONE_SMOKE=1..6` deixam de pôr a ferramenta Vector na mão ao abrir
  (`vec_bone_smoke.rs`, passo 0) — era a herança que prendia o osso ao Edit do vetor —, e os ossos
  delas ganham esqueleto pela porta das raízes soltas no 1.º quadro (abrem em Object, ossos à vista,
  sem alças). ⭐ Cena nova **`=7` — OS TRÊS MODOS** (`smoke_bone_modos.rs`, `NIVEIS` `6 → 7`): uma
  barra SOLTA e um esqueleto-OBJECTO de dois ossos montado pela cena, nada preso; o roteiro no
  terminal ensina Object (clicar no osso, Shift+clique na barra, `Ctrl+P`) → `Tab` Edit (arrastar da
  ponta cria) → *Mode* ▸ *Pose Mode* (arrastar dobra a barra) → `Tab` Object (o gizmo leva tudo). A
  barra entra no `bone_smoke_pend` para a shell chegar ao prólogo (painel de ossos aberto). Gate
  `the_three_modes_scene_has_one_skeleton_object_and_a_loose_bar` (controlo: nada preso depois do 2.º
  tempo). ⏳ Fotos das sete cenas: no fecho.

### F60 — ⭐⭐⭐ **A PONTA DO TRAÇO NA PONTA DO VINCO: quem tapa é posado pela pele EXACTA** (A10, 2026-10-05) — `1ab7e6999`…`3837e3975`

- **Causa confirmada pela régua da CONVERGÊNCIA** (sonda `cruza::tests::vinco::diag_a_ponta_do_vinco_converge`, barra 40×10 em cópias, traço 0,5): a mesma lei com os cobridores partidos em `d × d` subtriângulos posados pela pele exacta, `d = 1…16`, contra `d = 32`. Hoje (`d = 1`, a malha do campo em triângulos RECTOS): maior desvio da ponta `0,35` larg. a 110°, `0,08` a 150°, **`2,11` a 160°**, `0,15` a 170°, `0,26` a 175°; `d = 2` já `≤ 0,06`; `d = 16` `≤ 0,001`. O número de pontas não muda.
- **Lei** (`skin_desenho_frente_fina.rs`, `Fina`): só os COBRIDORES (o de chave maior de um par sem vértice comum cujas caixas — dos 6 pontos EXACTOS da grelha de lado 2, alargadas por 2× o desvio medido nos meios das arestas — se tocam) entram; cada um parte-se SOB PEDIDO (`Fina::cobre`, na 1.ª vez que um ponto lhe cai na caixa), dobrando o lado até o meio de cada aresta ficar a `≤ 0,1` largura da recta (`TOL_EM_LARGURAS`, `MAX_DIV = 16`). Triângulo com a mesma linha de pesos nos três cantos, sem correcções nem osso partido = afim, lado 1. ⛔ **A caixa da recta + folga ERROU** (desvio `1,07` larg. a 170°): o máximo amostrado nos meios das arestas erra para baixo — a caixa é a dos pontos exactos.
- **Tolerância escolhida pela medição** (lei a `tol` contra `d = 32`): `0,05` → `≤ 0,011`; `0,1` → `≤ 0,019`; `0,2` → `≤ 0,06`; `0,4` → **`1,07`** a 170° (fora).
- **Preço** (µs/forma/quadro, poses alternadas, mesma corrida, 7 rodadas × 20 quadros, mínimo; loadavg ~3): 110°↔150° campo `959` · lei `1 090`; 160°↔175° campo `928` · lei `1 405`; 0°↔30° (sem dobra, saída rápida) igual. Com a construção ANSIOSA de antes eram `+2,1–3,5 ms`.
- **Gate** `a_ponta_do_vinco_converge_para_a_pele_exacta` (150/160/170/175°: a lei a `< 0,1` larg. de `d = 16`, as mesmas pontas; controlo: a malha do campo passa de `0,5`, MEDIDO `2,11`). Gate `o_indice_da_grelha_triangular_e_a_ordem_de_construcao`. Os 291 testes da crate verdes (os do recorte, da saída rápida ao bit e da camada do traço incluídos).
- **Mutação** (agente, 8): sangram M3 (cobridor trocado), M5 (tudo afim), M7 (sem a malha fina); M1/M4/M6 (não refinar além do 1.º degrau) sobreviviam — o gate dizia `0,1` e o 1.º degrau dá `0,06` ⇒ gate a `0,03` com o controlo do 1.º degrau; M8 (vizinhos como cobridores) **equivalente** (só refina a mais: a consulta exclui a vizinhança); M2 (sem a folga da caixa) **sobrevive sem caso**: varredura `100°…180°` de 2,5° igual com e sem ela (pior `0,043` larg. a `135°`) — fica como margem do máximo amostrado, declarada.

### F61 — ⭐⭐ **AS PASSAGENS NOVAS QUE O TRAÇO ENCHE CORTAM-SE PELA CORDA** (A12, ordem do dono 2026-10-05: *«sim»*) — `…c693beae6`

- **Reproduzido e FOTOGRAFADO** (`=5`, Zig Zag a 100°): as duas marcas eram de espécies OPOSTAS — uma FENDA aberta para fora (a boca entre dois membros) e a PONTA de um dente que entra num BOLSO de fora (não num buraco: o bolso é do contorno exterior). As duas são uma passagem mais estreita que o traço criada pela dobra, e curam-se pela mesma corda.
- **Lei** (`skin_desenho_fendas.rs`, chamada na `uniao_dos_fechados` depois da `resolve_overlap`): pares de amostras (16/segmento) a `< w` com arco `> 1,5 w`; NOVA = **o arco tem um CRUZAMENTO** (âncora da união que não é âncora da fonte e mora em dois sítios dela) — entre dois cruzamentos a união segue um pedaço contínuo da fonte (o vale de um dente fica). A região (arco + corda) onde não cabe um círculo de raio `w/2` sai; as escolhidas sem se tocarem cortam-se juntas (`sem_as_fendas`, rectas nas bocas). ⛔ Um contorno que o traço engole INTEIRO (o buraco pequeno, a ilhota) fica (F59-b, recusado pelo dono); ⛔ num BURACO só sai a ponta de COR que entra nele, nunca um canto do próprio buraco (seria fechá-lo aos bocados).
- ⛔ **Três leituras erradas desta wave:** (1) «da fonte = algum contorno liga os dois lados por um caminho NÃO MAIS LONGO» — a borda de um buraco liga a base de uma ponta por um caminho CURTO; (2) recusar uma boca LARGA (o bolso, raio `0,86 w`) punha de lado tudo o que estava dentro dela — a ponta dentro do bolso ficava; (3) o sinal da cor comparado com o do PRÓPRIO contorno (num buraco a cor fica do lado do sentido do MAIOR).
- **Preço** (sobre a entrada real da `=5`, mínimo de 7×20): começou em `~12 ms` por chamada; `cabe()` com saída cedo (o raio inteiro do contorno de fora custava ms), «nova» pelo cruzamento (prefixo) em vez de procurar na fonte, grelha plana, uma passagem só → **`0,38–0,58 ms`** (100/110/120°, loadavg ~3,6).
- **Gates** (`skin_desenho_fendas_tests.rs`): `a_fenda_nova_fecha_e_o_vale_da_fonte_fica`, `a_ponta_nova_que_entra_num_buraco_corta_se`, `o_buraco_que_o_traco_engole_inteiro_fica`, `sem_a_lei_nada_muda`, `na_cena_as_passagens_novas_saem_e_os_buracos_engolidos_ficam` (`=5` a 100/110°: os contornos engolidos iguais ao `10⁻⁷`; depois da lei nenhuma passagem nova; controlo: a lei mexe). FOTOGRAFADO 100/105/110/120° (as duas marcas de 100° saem; os buracos ficam).
- **Mutação** (agente, 9): sangram N1/N2 (cruzamento sempre/nunca), N4 (canto do buraco), N6 (arco mínimo), N7 (região larga aceite); sobreviviam N3 (a ilhota engolida — os buracos já os guarda o N4), N5 (a boca larga punha de lado o que tinha dentro — o gate da cena usava a própria lei para conferir), N8 (boca recta: as fixturas eram rectas — e com o gate do contorno curvo continuou a sobreviver: as pontas de um `recorta` já são cantos ⇒ **equivalente**, os dois blocos SAÍRAM), N9 (cruzamento num só sítio: as fixturas não tinham ponto de divisão) ⇒ três gates novos e um de propriedade (`a_ilhota_que_o_traco_engole_inteira_fica`, `a_ponta_dentro_de_um_bolso_largo_corta_se_e_o_bolso_fica`, `a_boca_de_uma_passagem_e_recta_num_contorno_curvo`, `so_e_cruzamento_o_que_mora_em_dois_sitios_da_fonte`) e «a mesma boca» medida pelo COMPRIMENTO (era por amostras: numa aresta comprida a janela era larga).

### F62 — ⭐⭐ **A PELE GUARDA O ÂNGULO DE CADA OSSO E A TABELA DAS JUNTAS** (achado medindo a F60, 2026-10-05)

- A mistura (`Skin::blend_com`) calculava POR PONTO o `atan2` + `sin`/`cos` de cada osso e a junta de cada par (quatro `hypot`), que não mudam no quadro. `Skin` guarda `(θ, cos θ, sin θ)` ao construir e a tabela das juntas num `OnceLock` (na 1.ª mistura que a pede). A igualdade de `Skin` passou a ser a dos ossos e da lei (manual: a cache não entra).
- **Medido** (`centro_cache_tests::diag_o_preco_da_mistura`, mesmo processo, 7 rodadas intercaladas, mínimo): **`108,1 → 34,0 ns` por ponto** (3,2×). Vale para todo o desenho de formas e imagens presas, não só para a F60.
- **Gate** `a_cache_da_pele_da_o_ponto_de_antes_ao_bit` (400 pontos, pesos com 1–3 ossos: a mistura com a cache contra a conta por ponto de antes, ao bit). Os testes da `ph2d-skeleton` verdes. **Mutação 3/3 sangram** (cos/sin trocados na cache, `si` no lugar de `co`, a junta da diagonal).

### F63 — O efeito ANIMADO numa forma presa (herdado) — ✅ fechado no código, sem medição de relógio

- Desde a F51/F54 uma forma presa não tem efeitos vivos: `skin_live_carrega::coze_os_efeitos_presos` corre por QUADRO (`fase_vector_view_and_drives`) e coze qualquer pilha que apareça. As únicas que ficam vivas são as que a F54 deixa (com deslocamento de camada, `NaoServe`; sem campo não há gaveta de efeitos), e nelas o solver da gaveta corre em FUNDO (`efeitos_da_gaveta`, F53): o quadro paga o assado, não o solver.

### F59 — A5: OS DOIS DETALHES (2026-10-04) — (b) ⛔ RECUSADO PELO DONO no smoke; (a) ⏳ tentado e REVERTIDO

> ⛔ **(b) REVERTIDO em `501daabf4`** — o dono escolheu «fechar os buracos tão pequenos que a linha
> os cobre», viu-o no smoke da `=5` e respondeu *«o modo anterior era melhor»*: os buracos que a linha
> cobre (as manchas escuras) voltam a aparecer. A lei e os gates ficam em `f1481cd7d`/`5d3375f04`;
> ⛔ não reconstruir sem nova ordem dele. O texto abaixo é o registo.


- **(b) ✅ FEITO** por ordem do dono (04/10: «fechar os buracos tão pequenos que a linha os cobre»), `f1481cd7d` + `5d3375f04` (gate da ilhota). Lei `skin_desenho_buracos::fecha_os_buracos_que_o_traco_engole`, chamada na `uniao_dos_fechados` (só a união dos efeitos cozidos): um contorno de sentido contrário ao maior, que nenhum fechado da entrada tem (área e caixa a 10⁻³ — o buraco desenhado pelo artista fica), com raio inscrito (grelha 24×24) `<` meia largura do traço, sai. Zig Zag da `=5` (raios em larguras): 100° `[0,13 0,54 0,12]→[0,54]`; 110° `[1,24 0,25 1,25 0,42]→[1,24 1,25]`; 120° `[0,64 0,63]` igual. Gates `os_buracos_que_o_traco_cobre_fecham_e_os_outros_ficam` (régua própria 64×64; controlo sem a lei, cada lado numa thread nova) e `so_o_buraco_novo_que_o_traco_cobre_sai` (o do artista fica; ilhota pequena fica; o novo que se vê fica). Mutação 5/5 sangra (sem a guarda do artista · limiar 2× · sem o sentido · nunca sai · a união não chama). FOTOGRAFADO em SVG a 100°/110°: as manchas pretas sumiram, os dois triângulos de 110° ficam. ⏳ A 100° restam duas marquinhas escuras que NÃO são buracos: reentrâncias ABERTAS mais estreitas que a linha (handoff A12).
- **(a) ⏳ TENTADO e REVERTIDO** (ordem do dono «corrigir»): tentativa inteira em `ac8246764`, revertida em `9e39c48a5` (`git show ac8246764` para retomar). Desenho: máscara da tinta no bind (alfa `≥ 128`, corridas por linha; `SkinnedMesh.mascara`, binds antigos lêem-se); por malha, o ANEL DA ARTE (cada ponto da borda encostado à tinta, só onde a borda está a `> 1 px` dela, com o triângulo onde cai); a costura mede e cose sobre ele (posado pelos vértices do triângulo), com saída rápida; cada metade do vão estica a cor da SUA beira (1,5 px dentro da tinta). Régua de tinta (alcance 1 px; sonda `diag_a_foto_da_imagem`): cúspide −147° 26→9, −145° 39→14, −144° 104→48 — MAS −149,5° 4→12 e a −160° 3 de 16 remendos sobre a tinta de OUTRO membro (gate `onde_os_membros_se_sobrepoem_nada_se_cose_por_cima`); `skin_image_fecho.rs` a 705 linhas. Três reconstruções ⇒ parou (DIRETIVA §5).
- **Achados que ficam:** ⛔ ⭐ a malha passa da tinta DE PROPÓSITO (`GridOptions::expand = 2`; seguir o contorno está recusado na `ph2d-poly2d::grid`) ⇒ a lei de hoje só mede bem onde a arte encosta à orla; ⭐ os píxeis só existem no BIND (`asset_db` da shell) — no quadro só viaja a malha; ⭐ a costura mistura a UV dos dois membros e apanha a textura entre eles (risca castanha FOTOGRAFADA); ⛔ máscara a alfa `≥ 1` deixava meia-luz dentro do remendo; encostar também nas bordas rectas regredia; ⛔ o gate do risquinho era GEOMÉTRICO (vão entre bordas da malha) — em tinta nenhuma lei dá 0 (−144°: sem 354, antiga 91, tentativa 37).
- ~~**Próximo passo (hipótese):** o anel por marcha axial a partir da escada da grelha sai serrilhado e o teste «encara de fora» falha nele — contorno da máscara por marching squares (normais lisas) em vez da marcha.~~ ⛔ **REFUTADA (2026-10-05, experiência isolada, ramo `exp/a5a-marching`, `50328de56` sobre o cherry-pick `6d10d2ee2` da tentativa):** as três leis no MESMO processo (bandeira de teste, sonda `diag_a5a_as_tres_variantes`), mesma régua de tinta (alfa ≥ 128, alcance 1 px, amostras de ¼ px), `g1 = 36°`:

  | pose | sem costura | lei de hoje: fio / remendos sobre outra tinta | marcha axial (a tentativa) | marching squares |
  |---|---|---|---|---|
  | −144 | 367 | 104 / 0 de 140 | 59 / 0 de 236 | 66 / 0 de 232 |
  | −145 | 39 | 39 / 0 de 12 | 20 / 5 de 44 | 29 / 1 de 28 |
  | −147 | 26 | 26 / 0 de 12 | 11 / 2 de 20 | 18 / 0 de 20 |
  | −149,5 | 4 | 4 / 0 de 12 | 12 / 0 de 16 | **17** / 0 de 20 |
  | −160 | 42 | 42 / 0 de 12 | 42 / 3 de 16 | 42 / **1** de 16 |
  | 27 poses (−150…−138 de 0,5 + −155, −160): fio / janela do report / remendos | | 603 / 116 / 0 | 512 / 50 / 43 | 526 / 67 / 15 |

  Pior que a marcha na cúspide, pior que a lei de hoje a −149,5/−149, e ainda cose sobre a tinta de outro membro (−160: 1; −155: 6 de 24). ⚠️ **Os números do registo da tentativa estavam errados:** a marcha reproduzida dá −147 26→**11**, −145 39→**20**, −144 104→**59** (o registo dizia 9/14/48) — o resto bate. Quatro desenhos medidos ⇒ o A5-a fica ABERTO sem lei candidata: o que falta é perceber porque a costura sobre a tinta remenda sobre OUTRO membro (o critério de lado, não a forma do anel).

---

### F58 — A JANELA TAPADA ENTRE AMOSTRAS (A8, 2026-10-04) — ✅ fechado SEM cura de lei

O defeito existe mas fica abaixo da resolução do traço. Sonda `amostras::diag_as_janelas_que_passam_entre_amostras` (a pergunta `tapado` amostrada a 2 048 por segmento contra os intervalos do recorte), `60°…170°` de 0,5 em 0,5°: riscas 40×10 — 31 janelas perdidas, a maior `0,18` (0,36 do traço 0,5 das fixturas do A6) a 97°; barra em S da `=5` — 3, a maior `0,0203` (0,45 da largura 0,045) a 121°. Pela régua da F52 um pedaço mais curto que o traço é borrão ⇒ a amostragem pelo comprimento (construída e retirada na F55) continua sem caso. Gate `nenhuma_janela_perdida_chega_a_largura_do_traco` (as poses onde a maior apareceu e vizinhas; controlo: há janelas perdidas). Mutação `AMOSTRAS` 32→8 sangra (janela de 0,62 `>` 0,5 a 100°). O dono é informado no relatório.

---

### F57 — ⭐⭐⭐ **A PONTA DE UM CORTE DO TRAÇO ACERTA NO CRUZAMENTO DESENHADO** (A9, 2026-10-04) — `4384189db`, `a3dc835f8`, `a93bfdcf0`

- **FOTOGRAFADO** em SVG (sonda versionada `ph2d-app-vec::smoke_bone_copias::sondas::diag_a_foto_das_copias`, `SONDA_G1/G2`; `SONDA_CONTORNO=1` põe o contorno inteiro a azul fino; `magick` para PNG) na `=6` a `170°/−110°`: o traço vertical de trás passava `~0,7` largura do cruzamento com a borda da frente; e nas pontas do VINCO o traço contorna a dobra do papel e volta 0,4–3 larguras (outra família → A10).
- **Causa:** quem decide «tapado» é a malha posada em triângulos RECTOS e o desenho segue a pele exacta (o mesmo desvio medido na F56).
- **Régua** (sem malha nem chave de osso): distância AO LONGO do contorno fechado desenhado de cada ponta de trecho ao cruzamento mais perto dos contornos FECHADOS (sonda `diag_as_pontas_dos_cortes` na `=6`; gate na barra 40×10). ⛔ Duas réguas mentiram: a distância em linha recta (num grampo o cruzamento do OUTRO braço fica perto pelo ar) e a que contava cruzamentos com as riscas (uma risca toca o contorno de propósito).
- **Lei** (`skin_desenho_camadas_cruza.rs`, `Bordas`): polilinha (16/segmento) dos contornos do assado que a FONTE diz fechados, grelha de troços; `encaixa` leva cada ponta ao cruzamento mais perto ao longo do contorno até UMA largura do traço (a régua da F52: mais curto que o traço é borrão); `encaixa_trecho`: um trecho que o encaixe inverte sai. Chamada em `traco_sobre_o_assado`.
- **Medido** na `=6` (110…170°): pontas junto a cruzamentos 0,13…0,98 largura → 0,00…0,02. Gate `nenhuma_ponta_de_corte_fica_a_um_tique_do_cruzamento` (110/130/150/170 na barra 40×10 em cópias, traço 0,5: sem o encaixe 6 tiques, com 0; as pontas a mais de 1 largura — as do vinco — não se mexem). Gates unitários (barra e tiras sintéticas): `a_ponta_vai_ao_cruzamento_mais_perto`, `o_fecho_das_bordas_e_o_da_fonte`, `um_trecho_que_o_encaixe_inverte_sai`.
- ⛔ **Lição:** o memo do quadro (`skin_desenho::MEMO`) é por THREAD e guarda o desenho por forma — o 2.º desenho na mesma thread era o do 1.º; o gate desenha cada lado numa thread nova.
- **Mutação:** M9 (lei desligada), M10 (janela 3×), M12 (vizinhos contam) sangram; M11 (o mais longe), M13 (fecho do assado), M14 (trecho invertido fica) sobreviviam → ganharam os três gates unitários e sangram.

---

### F56 — ⭐⭐ **A SAÍDA RÁPIDA DO RECORTE** (A7, 2026-10-04) — `c9786bbd3`, `a93bfdcf0` (+ `4797b103b`: arte/grelha movidas para o irmão `skin_desenho_frente_malha.rs` pelo tecto de LOC; `4d91679e9`: fmt da `ph2d-skeleton-live` — dívida da linha — e os testes dos fechados no irmão `skin_desenho_frente_fechados_tests.rs`)

- **Medido antes de construir** (sonda `rapida::diag_o_desvio_e_a_folga_da_malha_posada`, release): o ponto posado pela pele (`onde`) afasta-se do triângulo posado LINEAR do dono até `0,19` aresta a 45–75° (0,6–2,1 arestas na dobra) e a folga mínima entre triângulos que não se tocam é 0,16–0,40 aresta — da mesma ordem: uma saída pela malha não é «ao bit» por construção. `rapida::diag_a_lei_corta_sem_sobreposicao`: em 0°…90° de 2,5 em 2,5° (riscas 40×10, barra em S da `=5`, cópias da `=6`) a lei NUNCA cortou numa pose sem par sobreposto nem virado.
- **Lei:** `malha::ha_sobreposicao` (pára no 1.º par de triângulos posados que se sobrepõem — eixos separadores `se_sobrepoem`, chave maior, sem vértice comum, caixa antes) e `Posada::nada_tapa` = nenhum par e (com o avesso a tapar) nenhum virado ⇒ `so_o_que_se_ve`/`cortes_dos_fechados` devolvem `None` sem amostrar (e sem construir a Arte).
- A exigência «o cobridor sobrepõe-se ao dono» em `tapado` foi construída e SAIU (mutação M3 sobrevivia: nenhuma fixtura a via) ⇒ a igualdade é MEDIDA (gate), não de construção.
- **Preço** (release, load `~5`, 3 corridas): o recorte de uma forma com 36 riscas sem dobra 174–216 → 77 µs (o resto é posar a malha, ~44–74 µs); na forma de cópias sem riscas fica igual (~120 µs: posar a malha ~50 + pares ~30). Ganho futuro possível: uma `Posada` só para as duas portas (hoje cada uma posa a sua).
- **Gates:** `a_saida_rapida_nao_muda_o_recorte_ao_bit` (0°…150°, 3 fixturas, Debug ao bit; controlo: dispara em 17/30, corta em 13; quando dispara nenhum contorno é amostrado — contador `AMOSTRAGENS`), `a_saida_rapida_so_sai_sem_par_nem_virado` (função pura).
- **Mutação** (agente, 14 corridas, 6 sangraram) e depois: M1/M2/M4 sangram; M8 (a saída nunca usada) ganhou a metade do contador e sangra; M3 a lei saiu; M5 (`<=`→`<` na chave), M6 (`>`→`>=` no SAT), M7 (a caixa) equivalentes — só tornam a saída mais conservadora ou são poda.
- **FOTO:** não se aplica (por desenho nada muda na tela; o gate ao bit é o juiz).

---

### F55 — ⭐⭐⭐ **SEM UNIÃO, O TRAÇO DOS CONTORNOS FECHADOS DE TRÁS NÃO PINTA POR CIMA DA FRENTE** (A6 da lista viva, 2026-10-04, pedido pelo dono)

- **Reproduzido** (SVG da forma desenhada, a placa estava ocupada por outra linha): duas cópias sobrepostas (*Repeater*, a 2.ª girada) presas e dobradas a `110°`/`130°`/`150°` — os contornos da parte de trás riscavam a frente em laços nas duas juntas. A união do contacto não corre ali (os contornos cruzam-se em repouso ⇒ `uniao_neutra = false`).
- ⭐⭐⭐ **Causa de fundo: o DOMÍNIO do campo era par-ímpar sempre** — a sobreposição das cópias (`NonZero`, pintada cheia) era um FURO sem malha: os contornos de dentro não tinham peso do campo nem chave de osso. Agora o domínio segue a REGRA DE PREENCHIMENTO da forma (`pesos_aneis::volta`/`pela_regra`, a mesma conta no índice e na varredura); uma forma `EvenOdd` fica igual. Gate `o_dominio_segue_a_regra_de_preenchimento_da_forma` (controlo: `EvenOdd` deixa o furo). ⚠️ Muda o campo de binds NOVOS de formas com contornos sobrepostos da mesma orientação.
- **Lei** (`skin_desenho_camadas`, `frente::tracos_a_vista`): sem união (`!contacto || neutra == Some(false)`), se a dobra tapa um contorno FECHADO o quadro sai em DUAS camadas — a forma com o preenchimento (sem traço) e a camada do TRAÇO: os trechos à vista dos fechados (cortados no repouso; o trecho que passa pela EMENDA cosido num só; a linha de peso do nó `0` também no fim da volta) assados com o mesmo `Bake`, mais as riscas abertas. `SkinDesenhado` passou a `Desenhado { forma, traco }` (lê-se como a forma por `Deref`); `funde` põe a camada DEPOIS da forma. Não com pilha de tintas nem opacidade de objecto (compor-se-iam duas vezes). Na mesma onda, duas curas da porta do recorte que também valem para a F52: ⭐ **só a ARTE tapa** (a margem da malha, além do desenho, apagava o traço de trás — triângulos de centro `(28,2; 17,5)` sobre a borda da cópia de baixo, MEDIDO) e ⭐ **o avesso não tapa o traço de um fechado** (ali ele É a borda da dobra; sem ele abria-se um vão — SVG a `130°`).
- **Medido** (`o_traco_dos_fechados_de_tras_nao_pinta_por_cima_da_frente`, régua sem a chave da lei: a frente como NUVEM de pontos do interior da arte posados — o polígono posado cruza-se na dobra e o par-ímpar errava): traço tapado pintado `0,000`/`0,012`/`0,004` contra `1,000` sem a lei a `110°`/`130°`/`150°`; frente e trás à vista `1,000` nas duas leis.
- **Preço** (`diag_o_preco_da_camada_do_traco`, release, `load ~8`, pose `110°` ↔ `150°`): `0,90 ms` com a lei, `0,31` sem ela. O recorte da F52 com a arte: `0,31 ms`, quadro `1,86` (`1,48` sem).
- **Gates:** os dois acima · `sem_dobra_que_tape_nao_ha_camada` · `as_riscas_abertas_vao_para_a_camada_do_traco` · `a_camada_do_traco_chega_ao_mundo_depois_da_forma` · `o_trecho_que_passa_pela_emenda_e_um_so_e_leva_as_linhas_da_fonte` (tabela SINTÉTICA: na fixtura os cantos de trás têm a mesma linha e a mutação sobrevivia) · `o_avesso_nao_tapa_o_traco_de_um_contorno_fechado`. **Mutação:** 10 corridas, 6 sangraram; das 4 sobreviventes, 3 ganharam gate e sangram; a 4.ª (a amostragem pelo COMPRIMENTO) SAIU do código — nasceu de uma hipótese que a medição desmentiu (a perda era a margem) e nenhuma fixtura a exprimia (aberto A8).
- **Cena `PH2D_VEC_BONE_SMOKE=6`** (`ph2d-app-vec::smoke_bone_copias`): duas barras de cópias sobrepostas presas, dobradas em S a `110°`, a 2.ª com *Hatch*; gate `as_duas_barras_saem_com_a_camada_do_traco` (controlo: rectas, nenhuma camada). FOTOGRAFADA.
- ⭐⭐⭐ **F55-b — o TRAÇO é cortado do PRÓPRIO assado do preenchimento** (report do dono de 2026-10-04, foto numa dobra agressiva: o contorno descolado do preenchimento). A 1.ª redacção assava os trechos à vista À PARTE do contorno inteiro, e as duas aproximações divergiam: MEDIDO `0,017` a `110°` e até `0,28` a `170°` numa barra de `0,75`. Agora `frente::cortes_dos_fechados` devolve cada ponta (parâmetro na fonte, ponto POSADO, fracção do comprimento posado do segmento) e `camadas::traco_sobre_o_assado` acha-a no pedaço do assado que vem do MESMO segmento (o nó `k` da fonte é, ao bit, um nó do assado), primeiro pela FRACÇÃO do comprimento (`±5 %`) e só depois pelo ponto mais perto — numa dobra forte o pedaço passa perto de si mesmo e o ponto mais perto agarrava o lado errado (MEDIDO: metade do traço sumia). Gate `o_traco_fica_sobre_a_borda_do_preenchimento` (12 poses até `170°`/`180°`: `3·10⁻⁵`). Preço: `0,61 ms` com a lei, `0,33` sem (antes `0,90`: o 2.º assado saiu). Mutação 6/6 (a fracção, o nó do contorno, a volta, a emenda, o avesso, os fechados inteiros). FOTOGRAFADO: a `=6` e, em SVG, `170°/−110°` — restam tiques curtos nas pontas de alguns cortes.
- ✅ **Smoke do dono APROVADO** (2026-10-04, *«smoke ok»*, depois da F55-b: a `=6` com o osso do meio dobrado à mão).
- ⛔ **Réguas que mentiram nesta onda:** o polígono do contorno posado (cruza-se na dobra); a nuvem com borda (passava da arte); a fixtura de lados COLINEARES (a pergunta `overlaps_itself` diz «não» e a união funde as cópias — a cópia tem de girar).

---

### F54 — ⭐⭐ **A FORMA PRESA DE UM PROJECTO ANTIGO COZE OS EFEITOS VIVOS, sem mover um pixel** (A4 da lista viva, 2026-10-04)

- **O defeito:** uma forma presa ANTES da F51 com efeitos vivos desenhava-os (lei F50) mas o painel de uma forma presa já não os mostra ⇒ invisíveis e não editáveis — a lei do dono «presa não tem efeitos» falhava por omissão.
- **Lei** (`skin_live_carrega::coze_os_efeitos_presos`, chamada pela shell NO QUADRO antes da pele, `fase_vector_view_and_drives`): a fonte guardada passa a ser a que o Bind de hoje faria — o MESMO cozido que a F50 desenha (`skin_desenho_efeitos::coze_para_guardar`: a geometria cozida em repouso com as voltas em nós, o campo do contorno cozido resolvido nos eixos dos TENDÕES guardados, a tabela dele), `efeitos_cozidos = true`, e a pilha sai da cena. Uma pilha só de efeitos desligados sai sem cozer. Fica como está: sem campo (bind anterior a 2026-09-20) e com offset de camada (`NaoServe`) — cozê-las mudaria o desenho. ⚠️ Por QUADRO e não só ao abrir: vale para abrir, desfazer e colar; depois da 1.ª vez custa ver uma pilha vazia por forma presa (sem clonar nada). Nenhum degrau de schema (o campo `efeitos_cozidos` já existe).
- **Gates** (`skin_live_carrega_tests.rs`): `cozer_no_carregamento_nao_move_um_pixel` — *Twist* num rectângulo e *Zig Zag* num de quinas redondas, ponta a `60°`: desvio `0` ao bit; a pilha sai, a fonte fica marcada, uma 2.ª vez devolve `0`; ⛔ controlo = a cura ingénua (a geometria cozida com o campo da FONTE) desvia `3,83` / `0,48`. `a_shell_coze_antes_de_desenhar_a_pele` (costura: a ordem na fase do quadro).
- **Mutação 7/7** (sem o campo do cozido · sem a marca · a pilha fica · a pele não se grava · a shell não chama · eixos vazios — e a guarda «não coze uma fonte já cozida», que SOBREVIVIA: estava errada, deixava fora da lei uma forma cozida que volta a ter pilha por colar/desfazer ⇒ saiu, gate `uma_forma_ja_cozida_com_pilha_nova_coze_outra_vez`, e a guarda reposta sangra).
- Sem smoke ao dono: por desenho NADA muda na tela (o gate é o juiz); o dono vê-o só se abrir um projecto antigo com uma forma presa e efeitos.

---

### F53 — ⭐⭐ **PRENDER UM *REPEATER* DENSO DEIXA DE PARAR A TELA** (A3 da lista viva, 2026-10-04)

- **Medido** (release, `load ~10`; `pesos_preco_tests::diag_o_preco_do_campo_por_etapa`, a barra com *Repeater* `N × N` que gira): a malha do domínio custava `0,1` / `17` / `102` / `401` / `743 ms` a `1` / `5²` / `13²` / `25²` / `39²` cópias (`1 024` anéis, `131 072` pontos) — a cerca de cobertura da grelha (`dentro` e «a fronteira toca a célula») varria TODOS os anéis em cada pergunta. O solver não pesava (com as cópias espalhadas os ossos não caem no domínio e ele responde `None` ⇒ a lei derivada). O Bind inteiro (`skin_live_efeitos_tests::diag_o_preco_do_bind_de_um_repeater_denso`, a fixtura `40 × 10`): `21` / `102` / `206` / `343 ms`. O `smoke` herda o `release` ⇒ era o que o dono sentiria como tela parada; os «10 min em DEBUG» eram da `ph2d-vec-skin` a `opt-level 0` nos testes.
- **Cura** (sem tecto de cópias — §0.0: o recurso era um algoritmo quadrático, não o hardware): `pesos_aneis::IndiceDosAneis` — as arestas por FAIXA horizontal (o raio do par-ímpar) e por CÉLULA (a caixa da aresta), `64 × 64`; cada pergunta vê só as arestas que a podem responder, com a MESMA conta por aresta (`cruza_o_raio`, `cruza_a_celula`) ⇒ a mesma resposta ao bit. Depois: malha `0,8` / `5,8` / `11,5` / `20,7 ms`; Bind `19` / `60` / `69` / `64 ms`; 1.º quadro a `39²` `30 ms` (uma vez).
- **Gate** `o_indice_dos_aneis_responde_como_a_varredura` (toda célula de `4` px da caixa de `512` com folga, num anel e num *Repeater* `13²`; controlo: as duas respostas aparecem — `2 236`/`294` e `983`/`2 340` de `17 424`).
- **Mutação 5/6 + 1 equivalente:** a faixa e a célula sem o vão da aresta, o `toca` só na 1.ª faixa e a paridade invertida sangram; o `>` → `>=` do raio SOBREVIVIA (a régua partilha a conta) ⇒ gate `a_fronteira_do_par_impar_e_semi_aberta` (lados esquerdo/de baixo dentro, direito/de cima fora), e sangra; o `floor` → `round` da célula é EQUIVALENTE (a mesma função posiciona arestas e perguntas, e é monótona).

---

### F52 — ⭐⭐⭐ **NUMA DOBRA FORTE A PARTE DA FRENTE TAPA O QUE É ABERTO NA DE TRÁS** (A2 da lista viva, 2026-10-04; FOTOGRAFADO na `=5` a `110°`: as riscas do *Hatch* da parte de trás cruzavam por cima da frente, «+» e uma laçada em cada junta)

- **Lei** (`skin_desenho_frente::so_o_que_se_ve`, chamada no `calcula` antes do bake): a malha do campo é POSADA no quadro (a lei do bake: campo → `weights_corrected` → `blend`); cada triângulo tem a chave de osso (média dos três vértices) e o de chave maior fica por cima — a ordem das faces da imagem (F48-c). Uma amostra de um contorno ABERTO (repouso) está TAPADA quando o triângulo dela está do AVESSO (a dobra virou-o: o lado de baixo do papel) ou quando, posada, cai num triângulo de chave maior que não toca o dela. O contorno parte-se no REPOUSO (`32` amostras por segmento + `12` bissecções; de Casteljau, exacto) e o bake só percorre os pedaços à vista; um pedaço cortado mais curto que a largura do traço sai (um borrão, não uma risca). A tabela dos nós da fonte é COPIADA (o fechado sai ao bit); a de um nó novo é a do campo. Os FECHADOS não se tocam (a união do contacto é deles). `PH2D_SKIN_FRENTE=0` (`Leis::frente`) desliga.
- ⭐⭐⭐ **A chave de osso é a PROFUNDIDADE na hierarquia, nunca a coluna** (`esqueletos::profundidades`, `skin_image_fecho::chave_de_osso`). MEDIDO: o `bevy_ecs` 0.19 aloca índices DECRESCENTES (`0x…fe`, `0x…fd`, `0x…fc` por ordem de criação) e os tendões vêm por `to_bits` ⇒ a coluna `0` é a PONTA. A F48-c supunha «colunas raiz → ponta» e punha a RAIZ por cima — o CONTRÁRIO da ordem do dono (*«as faces do último osso por cima»*), e o gate dela (`onde_os_membros_se_sobrepoem_o_osso_de_fora_pinta_por_cima`) não o via porque media com a MESMA chave. A imagem passa a ordenar pela profundidade (lida da hierarquia VIVA ⇒ serve aos binds já guardados); a régua dela também. Gate `a_profundidade_de_cada_coluna_e_a_da_corrente` (controlo: a coluna `0` é mesmo a ponta).
- **Medido** (`as_riscas_de_tras_nao_pintam_por_cima_da_frente`, barra `40 × 10`, dois ossos, *Hatch* `45°`; a régua NÃO usa a chave: frente = membro da ponta `x > 24` posado, trás = `x < 16`, tinta = troço desenhado aberto a `< 0,1` e PARALELO): riscas de trás debaixo da frente pintadas `0,000` contra `0,990`/`0,990`/`0,994` sem a lei a `110°`/`130°`/`150°`; frente pintada e trás à vista `0,995` nas duas leis (o resto da régua: pontas de risca de tangente nula).
- **Preço** (`diag_o_preco_do_recorte_por_quadro`, release, `load ~10`, `36` riscas, malha `924` triângulos, pose a mudar `110°` ↔ `150°`): o recorte `0,25 ms`; o quadro inteiro `1,93 ms` com a lei, `1,51 ms` sem ela (o recorte + o bake dos pedaços).
- **Gates** (`skin_desenho_frente_tests.rs`): `as_riscas_de_tras_nao_pintam_por_cima_da_frente` · `o_que_nao_se_corta_sai_ao_bit` · `nenhum_pedaco_cortado_e_mais_curto_que_o_traco` (controlo: `0,036`–`0,042` antes do filtro) · `cada_corte_cai_na_fronteira_do_que_se_ve` · `o_recorte_e_a_curva_da_fonte` (numa risca CURVA) · `uma_risca_sobre_o_avesso_da_dobra_nao_se_ve` (fonte `128` pontos no avesso a `120°`, `0` à vista).
- ⛔ **Três réguas que mentiram nesta onda** (todas medidas): (1) a régua de tinta por PONTO lia vãos de `0,39` numa cúbica de alça só de um lado (os pedaços cortados) — passou a medir ao troço; (2) a «trás à vista» chamava à vista uma risca debaixo da ZONA DA JUNTA; (3) a `150°` o membro de trás comprime-se sobre si até `x ≈ 15,3` — ali a ordem é da lei, não da régua.
- **Gates antigos ajustados:** `o_contorno_de_um_hatch_tambem_se_une_e_as_riscas_ficam` mede a união com `frente: false` (o recorte parte riscas em dois); `so_o_bake()` desliga também a frente (as pontas cortadas ficam DENTRO da forma de propósito).
- ✅ **Smoke do dono APROVADO** (2026-10-04, *«smoke ok»*: as riscas na dobra forte da `=5` e o último osso por cima na imagem da `=4`).
- **FOTOGRAFADO** a `110°` (`=5` com a `DOBRA` temporária): os «+» e a laçada sumiram; o que resta junto às juntas são pontas de risca que entram por baixo do membro seguinte. A imagem da `=4`: o membro da ponta por cima.
- **Mutação 12/12 sangraram** (ordem da chave invertida · sem a exclusão do vizinho · sem o avesso · sem o filtro do traço · sem bissecção · alça do corte · linha da tabela deslocada · a chave pela coluna · profundidade nula · a porta desligada · a imagem sem a profundidade · nenhum triângulo virado).

---

### F51 — ⭐⭐⭐ **O BIND COZE OS EFEITOS, e uma forma presa não recebe efeitos** (ordem do dono, 2026-10-03: *«acho que o melhor é simplificar. Ao aplicar os bones, os efeitos são cozidos antes. E uma vez com bones, o vetor não pode receber efeitos»*; perguntado, **nenhum** efeito — nem os que não deformam)

- **Lei** (`skin_live_prender::bind_com`, a porta única de prender): os efeitos ACTIVOS são cozidos (`cooked()`) na fonte guardada e na cena (`VecScene::bake_cooked`) — o *Expand Appearance* —, e a pilha sai vazia; um efeito desligado sai sem cozer. A porta recebe a cena para ESCRITA (`&mut VecScene`). Gates `skin_live::efeitos_tests` (coze, pilha vazia na cena e na fonte, **não move um pixel**, com controlo; o desligado sai sem cozer).
- **Painel:** `set_current_effects(has_target, bound, …)`; presa = a secção Effects diz só «Bound to bones: effects are baked into the drawing» (sem Add, cartões nem Apply). `fx_bridge::is_bound` (o `SkinBind` da entidade); a shell publica-o e o dispatch RECUSA a edição numa forma presa. Gates: seam `a_bound_shape_is_offered_no_effect` (com a metade de presença), `presas_as_barras_ja_nao_tem_efeitos_vivos` (cena `=5`).
- ⭐⭐ **A forma de efeito COZIDO guarda a lei de contacto da F50** (só a união, só quando neutra em repouso): `SkinnedPath::efeitos_cozidos` (apendado; `SkinnedPathV2` lê os binds anteriores, sem degrau de schema), `Preparado::uniao_neutra`. FOTOGRAFADO: sem isto a barra cozida ia à BOLA e ela comia os vales entre os dentes do *Zig Zag* do lado de dentro da junta a `60°`. Gate `a_baked_zigzag_keeps_its_teeth_in_the_bend` (controlo sem a marca: `1,14`). O Bind também aplica `parte_nas_voltas` ao cozido (F50-h).
- A lei F50 (efeitos vivos numa forma presa, cozidos em repouso — agora em `skin_desenho_efeitos.rs`) FICA: desenha uma forma presa que já tinha efeitos vivos (projecto anterior), sem os poder editar.
- ⛔ **RETIRADO — o botão «Before bones | After bones» por efeito** (construído, medido e aprovado em gates na mesma tarde; o dono escolheu simplificar). O que se mediu, para não reconstruir às cegas: o «depois» corria sobre o desenho dobrado com o tamanho do repouso e o centro levado pela pele (`0,006`–`0,020` do ideal, tão perto quanto a forma sem efeito; o centro parado no mundo ficava `3,78` longe quando o esqueleto andava); *Zig Zag* e *Pucker & Bloat* tinham de ficar antes (trocar a ordem em repouso mudava `0,05`–`0,21` / `0,45`–`0,48`: o desenho fiel tem `15` nós onde o artista pôs `8`); preço igual ou menor que o «antes». Commits `a715a194c`…`f5b986092`, revertidos em `fd7b0750b`.
- ⛔⛔ **FICA — o PÂNICO pré-existente curado** (achado pela varredura dos tipos): numa forma presa um *Repeater* `39 × 39` que gira rebentava o `linesweeper` 0.4 (`curve/mod.rs:364`, um `unwrap`) pela união do contacto. `catch_unwind` na porta única do motor (`engine::binary_grouped_checked`) e o «contacto neutro em repouso» passa a ser a PERGUNTA (`ph2d_vec_boolean::overlaps_itself`), sem correr a união. Gate `a_dense_spinning_repeater_union_answers_instead_of_panicking`. ⏳ **ABERTO:** prender uma forma com esse *Repeater* paga o solver do campo sobre `~1 000` contornos (em debug não acabou em 10 min) — medir em release e decidir um tecto pelo recurso.
- ⛔ **A régua da câmera da `=5`** aprovava `±4,5 m` onde a foto (`1930 × 1040`) mostra `±4,0` (a barra de ferramentas tapa `~0,95 m`) — passa a usar a faixa medida.
- **Tecto de LOC por corte** (os gates de arquitectura não tinham corrido na linha): `skin_live.rs` → `skin_live_prender.rs`, `skin_desenho.rs` → `skin_desenho_efeitos.rs`, `skinned_mesh_efeitos_tests.rs` → `…_solver_tests.rs`, `smoke_bone.rs` → `smoke_bone_corrente.rs`; e `skin_image_fecho.rs` entra no MOTOR do censo da malha posada.

---

### F50 — ⭐⭐⭐ **A FORMA PRESA COM EFEITO DOBRA COM ELE** (ordem do dono, 2026-10-02: formas vetoriais com efeitos; F50-d pelo report de 2026-10-03)

- **Reproduzido:** uma forma presa com efeito saía do desenho fiel (`o_estilo_serve`) e voltava à lei dos `8` nós, com a pilha cozida DEPOIS sobre a forma dobrada. Nove dos dez efeitos lêem o `FxCtx` (caixa, centro, `ref_size`) da forma que recebem ⇒ o efeito mudava de tamanho com a pose. FOTOGRAFADO (`=5`, `60°` em S): as cinco barras com efeito saem como uma salsicha torta que não segue o S; a hachura fica recta no ecrã.
- **Lei:** `skin_desenho::cozido_com_efeitos` coze a pilha VIVA (`estilo_de` → `Estilo::Efeitos`, só entradas `is_active`) sobre a fonte em REPOUSO (`cooked()`: quinas vivas e depois efeitos), a tabela sai de `pesos_dos_pontos` sobre o campo do bind, e o assado percorre o cozido — a lei da quina viva. Guardado na gaveta por pilha (`Gaveta.efeitos`); o desenhado sai com `effects` vazio (`skin_live` limpa-o: um consumidor que o cozesse aplicava a pilha duas vezes). O offset de camada (`paints[].dilate`) continua de fora (`Estilo::NaoServe`). `PH2D_SKIN_EFEITOS=0` (`Leis.efeitos`) volta à lei antiga.
- **Contacto numa forma com efeito: NENHUM** (F50-d). A 1.ª redacção usava só a UNIÃO (`resolve_overlap`; a silhueta inteira custava `3,5`–`7 ms` no *Zig Zag*), e o report do dono de 2026-10-03 (quatro fotos: pedaços de traço soltos, serrilha, um quarto de círculo) mediu-se nela: a união reescrevia o desenho `0,16`–`2,04` longe do ideal a `90°`–`120°` (*Bloat*, *Twist*); sem ela `0,002`–`0,006` em toda a dobra. ⚠️ Custo visível: numa dobra muito forte (`~100°+`) o contorno de um efeito cruza-se por dentro da junta e o TRAÇO desenha o cruzamento (o «olho» que a silhueta cura na forma sem efeito) — FOTOGRAFADO a `110°`.
- ⭐⭐⭐ **O domínio do campo é o contorno COZIDO** (F50-d): `CozidoFx.campo` = `campo_do_caminho(cozido, eixos)`, com os eixos do bind relidos dos tendões (`skin_live::eixos_do_bind`: repouso de cada `Tendon` + comprimento do osso, pela ordem das colunas). Medido (`um_efeito_que_sai_da_forma_nao_rasga`, esticão máx entre amostras vizinhas do ideal): sobre o campo da FONTE a cauda de um *Twist* rasga (`17`–`191×`), sobre o do cozido `≤ 3,6×`; um *Bloat* fica no alcance da barra e não rasgava. É o *Puppet* do After Effects (a malha sai do que a camada DESENHA). Preço: um solver por pilha nova, `20`–`100 ms` (`diag_o_preco_do_campo_do_cozido`, a `load 53` — refazer com a máquina calma); nunca por pose. Mutações: silhueta de volta e campo da fonte — as duas sangram.
- **Oráculo CORRIDO:** Blender 5.2, `parent_set(ARMATURE_AUTO)` num Grease Pencil — o modificador que já existe fica ANTES do `Armature`, um acrescentado depois fica DEPOIS ([`oraculo/oraculo_ordem_dos_efeitos.py`](oraculo/oraculo_ordem_dos_efeitos.py), saída no cabeçalho). A F50 escolhe a 1.ª para toda a pilha; a ordem por efeito fica ABERTA (abaixo).
- **Medido** (`skinned_mesh::efeitos_tests`, afastamento máx ao padrão-ouro ponto a ponto, DENTRO do domínio, diagonal `~7,07`), antiga → nova a `60°`/`90°` em S: *Zig Zag* `0,267`/`0,494` → `0,0020`/`0,0019` · *Twist* `0,427`/`0,655` → `0,0018` · *Warp* `1,078`/`1,270` → `0,0019`/`0,0016`. Em repouso a nova é o desenho do artista (`≤ 0,001`).
- **Preço** (`diag_o_preco_do_efeito_por_quadro`, release, load `~8`, pose a mudar): `1,2`–`1,35 ms` por forma por quadro; a barra sem efeito com o desenho fiel `1,6 ms` (`diag_o_preco_do_desenho_fiel_por_quadro`).
- **Cena `PH2D_VEC_BONE_SMOKE=5`** (`ph2d-app-vec::smoke_bone_efeitos`): seis barras `4,5 × 0,75`, uma sem efeito e *Zig Zag*, *Twist*, *Warp*, *Bloat*, *Hatch*, dobradas a `60°` depois de prender. Gates: cabem na câmera de omissão sem se tocar; cada barra com efeito tem um efeito ACTIVO; despacho nos dois tempos.
- ⭐⭐⭐ **F50-e — o AJUSTE do bake só aceita a cúbica que ANDA no sentido da fonte** (report do dono de 2026-10-03, seis fotos: quartos de círculo ao longo do traço). Um nó de quina do cozido (alça nula) deixava o `fit_to_cubic` com um braço `0` e o outro maior que a corda: uma laçada MENOR que a tolerância, que o `fecha` (distância) aceitava, e o traço grosso desenha a meia-volta. Medido a `60°` antes: *Twist* `9`, *Warp* `9` (e `2` já em repouso); barra sem efeito `0`. Cura em `ph2d_vec_skin::curva_segundo_corpo::fecha`: a derivada nunca contra a direcção da fonte (24 amostras) e, nos quartos das pontas, a conta EXACTA (`anda_para_a_frente`, projecção quadrática). Depois: as voltas do desenho = as da arte em repouso em todos os efeitos e dobras (gate `o_desenho_nao_volta_para_tras_onde_a_arte_nao_volta`; mutação sangra). ⭐ **E curou na ORIGEM o gancho da F43**: os três controlos da zona (`o_gancho_da_dobra_nao_e_quina_do_artista`, `com_a_junta_quase_recta…`, `com_as_duas_juntas_no_mesmo_sentido…`) liam o gancho no desenho sem contacto e passaram a afirmar `0` — o passe dos ganchos fica como rede para a união. Preço: barra sem efeito `1,51 ms`/quadro (era `1,6`), *Zig Zag* `0,73 ms`. FOTOGRAFADO a `3840` px: traço do *Warp* e do *Twist* limpo.
- ⭐⭐⭐ **F50-f — a UNIÃO volta, sem LASCAS e só sobre os contornos FECHADOS.** Medido depois da F50-e: a união já não serrilha (as voltas a mais são os «V» do contacto); sobravam LASCAS — ilhas de área `~1e-16` da agulha de um *Bloat* forte (`6`–`9` a `−200`), riscos soltos no traço. `resolve_overlap` descarta-as pela régua do `drop_slivers` (área relativa `1e-4`; serve também à forma sem efeito). `CozidoFx.contacto` voltou (só une se neutra em repouso) e `uniao_dos_fechados` une os fechados e devolve os abertos — as riscas do *Hatch* faziam a união recusar a forma inteira. FOTOGRAFADO a `110°`: *Warp* e *Twist* com o «V» limpo, o *Hatch* une o contorno. Gates `a_uniao_numa_forma_com_efeito_nao_deixa_lascas_nem_mexe_no_repouso`, `o_contorno_de_um_hatch_tambem_se_une_e_as_riscas_ficam` (mutação sangra); os do bake medem `so_o_bake()`. ⚠️ No *Zig Zag* a `110°` os dentes de dentro encavalam-se e fecham buraquinhos REAIS (a imagem também os mostraria); o traço à volta lê-se como manchas escuras.
- ⭐⭐ **F50-g — a AGULHA na tampa de um *Bloat* era do EFEITO, não da pele** (foto do dono, 2026-10-03). A cápsula do `RoundRect` (raio = meia espessura) tem na tampa dois nós no mesmo sítio ligados por um segmento NULO de alças recolhidas; os dois factores opostos do *Bloat* (âncoras `1−t`, alças `1+t`) tiravam essas alças da âncora e o segmento virava uma agulha de largura zero, desenhada pelo traço como uma linha — também sem ossos. `ph2d_vec_scene::fx_warp::bloat_contour`: um segmento de comprimento zero continua de comprimento zero. Gate `a_capsula_com_pucker_nao_ganha_agulha_na_tampa` (mutação sangra). FOTOGRAFADO a `−60`: a linha sumiu.
- ⭐⭐⭐ **F50-h — as voltas apertadas do cozido viram NÓS antes do bake** (foto do dono, 2026-10-03: *«bloat +60 aparecem linhas bizarras»*). O *Bloat* `+60` põe pontas que dão a volta DENTRO de um só segmento (alças a `~3,6` da âncora); o bake amostra a passo fixo e ajusta pela tangente de Catmull-Rom, que numa ponta aponta de LADO — traços rectos perpendiculares à ponta. NÃO era a união (iguais com `PH2D_SKIN_CONTACTO=0`) nem existia em repouso. Medido a `30°`/`60°`: `0,48`/`0,92` longe do ideal → `0,012`/`0,017` com `skin_desenho_voltas::parte_nas_voltas` (de Casteljau onde o segmento vira `> 90°`; exacto em repouso). Gate `um_bloat_positivo_forte_nao_solta_linhas_ao_dobrar` (mutação sangra). ⚠️ A régua amostra o repouso JÁ partido (inteiro, a polilinha lia `0,025` só de corda). Preço por forma por quadro a mexer, `load ~10`: *Zig Zag* `0,89 ms`, *Warp* `1,65`, *Twist* `2,45` (a união `~1 ms` dele); barra sem efeito `1,5`. FOTOGRAFADO: as linhas sumiram; os traços por dentro do corpo são as barbatanas do próprio efeito (iguais em repouso).
- ⭐⭐ **F50-i — o orçamento de amostras é o do CONTORNO fechado** (foto do dono, 2026-10-03: *«Hatch linhas saindo da forma»*). As pontas das riscas eram EXACTAS (`0,0000` do ideal); quem saía do sítio era o contorno: as `58` riscas abertas repartiam o `AMOSTRAS_POR_FORMA` e o contorno ficava com `16` amostras por segmento, cortando caminho na dobra (`0,034`/`0,067`/`0,12` a `60°`/`90°`/`120°`). `skin_desenho::segmentos` conta só os fechados quando os há: contorno a `0,0017`, pontas a `≤ 0,005` dele. Gate `as_riscas_de_um_hatch_nao_saem_da_forma_ao_dobrar` (mutação sangra). Preço do *Hatch* denso (`58` riscas) a mexer: `2,66 ms`/quadro a `load 15`. FOTOGRAFADO a `100°`: as riscas ficam dentro e abrem em leque no lado de dentro da junta (compressão, como a imagem).
- ⭐⭐ **F50-j — o campo do contorno cozido resolve-se numa THREAD.** Arrastar o controlo de um efeito numa forma presa pagava o solver a cada quadro (*Twist*: `59 ms` médio, `86` pior). A 1.ª vez resolve no quadro (nada aparece rasgado ao abrir o projecto); as seguintes numa thread, UMA de cada vez, e entretanto a geometria nova usa o último campo resolvido — quando ele chega o cozido e o quadro refazem-se (`Ultimo.fx`, `Rc::ptr_eq`; o laço é `ControlFlow::Poll`, logo aparece sem input). Release, a mudar a pilha a cada quadro: `2,5 ms` médio, `3,3` pior. Nos testes da crate o solver é síncrono salvo pedido (`solver_em_fundo_no_teste`). Gate `mudar_um_efeito_nao_paga_o_solver_no_quadro_e_chega_ao_exacto`. ⛔ Lição: um `git checkout -- <f>` para limpar uma sonda levou o gate por commitar do mesmo ficheiro — commit ANTES de sondar.
- ⭐ **F50-k — enquanto o campo novo não chega, mostra-se o último par EXACTO** (report do dono: *«quanto mais veloz se arrasta o valor de twist mais deformações bizarras»*). A F50-j desenhava a geometria NOVA com o campo de uma pilha anterior; num *Twist* rápido a forma saía do domínio velho e rasgava. Agora o cozido é sempre o da pilha RESOLVIDA com o campo dela: o arrasto anda aos degraus do solver. ⏳ **A outra metade do report é a LEI** (*«com alguns valores após parar fica ruim»*): o campo espacial passa as pontas enroladas de um *Twist* forte para o osso do meio (`120°`: `[33, 66, 33]` nós). ~~Decisão do dono: a lei MATERIAL~~ (escolha de 03/10, feita ANTES de o Bind cozer os efeitos). ✅ **DECIDIDO de novo pelo dono em 2026-10-04 (A1), com a F51 já em vigor: fica a lei ESPACIAL** — cada pedaço segue o osso de que está mais PERTO no desenho cozido (à *Puppet* do After Effects; `bind_com` → `campo_do_caminho(src cozido)`). Nada a construir; a lei material NÃO se reconstrói sem nova ordem dele.
- ✅ **Os abertos de 01/10 auditados contra o CÓDIGO (2026-10-03):** o **vinco do cotovelo a `90°`** — FOTOGRAFADO na `=4` a `PH2D_VEC_BONE_DOBRA=90`: o lado de dentro sai em ARCO (a bola da F41, a escolha do dono «Arredondado»); a imagem ao lado é que vinca. **Ligar o bake no desenho** — já ligado por omissão desde a F37 (`Leis::desenho`, `PH2D_SKIN_DESENHO=0` desliga). **O `Strength` inerte em arte preenchida** — o painel já o esconde onde o envelope não manda (`section_campos.rs`, `state::envelope_manda` ← `esqueletos::o_envelope_deste_osso_manda`, F17/F21); o caso de UM osso fica à vista de propósito («esconder um controlo vivo é pior»). Os três eram notas envelhecidas.
- ⏳ **ABERTO:** (1) ~~o cruzamento do traço numa dobra muito forte~~ — curado pela F50-f (a união). (2) **arrastar um slider de efeito** numa forma presa paga o solver por quadro (`20`–`100 ms`) — se o dono o sentir: solver fora do quadro ou o campo da fonte durante o arrasto. (3) **um efeito ANIMADO** paga o mesmo a cada quadro (não medido). (4) ~~**a ordem por efeito** à Blender~~ — construída e RETIRADA na **F51** (o dono escolheu: o Bind coze os efeitos). (5) a agulha de traço dentro da tampa de um *Bloat* negativo é do PRÓPRIO efeito em repouso (existe nas duas leis), não da pele.

### F49 — ⭐⭐⭐ **A IMAGEM PRESA COSE O FIO entre membros que a corrente encosta** (no lugar da bola da F48)

- **Reproduzido:** o 2.º smoke do dono reprovou a F48 (*«queda de FPS»*; *«ora redonda ora pontuda»*). Medido: a bola custava até `58 ms` por imagem por quadro (`279` nós crus a `(40°, 40°)`); de `(36°, −141,5°)` a `(36°, −148,5°)` não fechava (a escada da grelha na margem transparente era parede); a `(36°, −131,25°)` a subtracção devolvia um anel com fio de área zero.
- **Mecanismo/lei:** `ph2d_skeleton_live::skin_image_fecho::costura`. Onde a borda posada ENCARA outra parte (cada borda do lado de fora da outra) a menos de `VAO_MAXIMO_EM_TEXELS = 2` (duas bordas suaves de ~1 texel: entre elas não há texel de fundo inteiro) e entre partes a mais de `OSSOS_DE_DISTANCIA = 1,25` osso (chave de osso = `Σwⱼ·j/Σwⱼ`, a da ordem das faces), cada amostra (uma por texel) liga-se ao ponto mais perto da outra parte por dois triângulos. As pontas de cada troço são INTERPOLADAS onde a distância passa o vão ⇒ contínua na pose. Um lado só (o segmento de índice menor); UV de cada lado = a da sua beira. NÃO arredonda os «V» das juntas (são forma, aprovada pelo dono).
- **Peças:** anéis da borda (`aneis_da_borda`, `0,93 ms`) guardados por malha em `bordas_da` (chave = o `Rc` da gaveta `skin_bake_cache::desenhada_da_arte`, provado por `Weak`); saída rápida por faixas de ¼ de osso (caixas a menos de 2 vãos); grelha = vector ordenado por célula. Pedaço de 9-slice não cose (outra numeração). `PH2D_SKIN_COSTURA=0` desliga.
- **Medido:**
  - (a) Largura dos vãos, `diag_a_largura_dos_vaos` (`36°`, `−120°…−160°` de 4 em 4): risquinho `16` nós abaixo de 1,5 texel; resto `0`–`2` nós abaixo de 2; entre 2 e 3 texels `8`–`10` nós em todas (fundo dos «V»).
  - (b) Distância em ossos dos pontos cosíveis: pontas de «V» a `(120°, 120°)` e `(36°, −131,25°)` `1,000`–`1,046`; risquinho `(36°, −144°)` `1,556`–`1,637`; `(36°, −146°)` `~2,0`.
  - (c) Custo por imagem por quadro, placa (load alto durante a medição; a razão é lado-a-lado): sem costura `0,19 ms`; com ela e nada a coser `0,22 ms` (recta, `40°`, `−131°`) e `0,31 ms` (`120°`); com costura `0,70 ms` (o quadro posa na CPU). A 1.ª redacção, sem o filtro de ossos e sem saída rápida: `0,50 ms` em repouso e cosia `42` triângulos de pontas de «V» a `40°`.
- **Fotografado** a `(36°, −144°)`: o fio some; resta um traço quase invisível na cúspide junto à tampa.
- **Gates:** `nenhuma_pose_a_volta_do_report_deixa_o_fio` (`−150°…−138°`, meio grau, alcance 1 px) · `os_v_das_juntas_ficam_como_a_arte` (`40°` e `120°`: nada cosido, a placa posa) · `a_tinta_da_costura_e_a_das_beiras` (cada pedaço de 4 pontos com ≥2 texels de arte; a outra ponta pode cair na escada transparente da tampa, texel `(586,1)` alfa 0, fotografado: esmaece sem névoa) · `a_costura_nasce_ligada_e_o_zero_bissecta` · e os que ficam da F48: `o_vao_entre_os_membros_da_imagem_fecha`, `com_vao_a_placa_cede_a_cpu_e_o_vao_fecha_igual`, `sem_vao_a_malha_sai_ao_bit`, `onde_os_membros_se_sobrepoem_o_osso_de_fora_pinta_por_cima`. **Retirados com a lei:** `na_dobra_forte_os_bicos_do_v_arredondam`, `a_dobra_arredonda_em_toda_a_varredura`, `a_uniao_do_fecho_da_borda_nao_desloca_a_polilinha`. Mutação: 1.ª corrida `10/12` (sobreviveram: o teste «as duas bordas de fora», a interpolação das pontas, o acerto da gaveta dos anéis); cada uma ganhou guarda e morre na re-corrida ⇒ `12/12`: `onde_os_membros_se_sobrepoem_nada_se_cose_por_cima` (sem o teste de fora: `5`–`6` de `10` centros cosidos sobre tinta a `−150°…−160°`, `83` de `150` a `−144°`; com ele `0`) · `a_costura_nunca_passa_dos_dois_texels` · `os_aneis_guardados_sao_os_calculados_e_ficam_por_malha`.
- **Continuidade** (régua de tinta, passo `0,0025°`): a área cosida cresce contínua (ex.: `0,1→18,7` texel² em `0,6°`) e tem dois degraus de `0,6`–`0,8` texel² onde as bordas começam a cruzar-se (sobreposição); na tela a baía varia `≤ 7` amostras de ¼ px (`< 0,5 px²`) nesses passos — invisível.
- **Duas réguas** em `smoke_bone_par_fresta_tests.rs` (as sondas estão no irmão `smoke_bone_par_fresta_sondas.rs`, pelo tecto de LOC): `buracos` (geométrica — o vão entre BORDAS da malha, a pergunta da lei) e `buracos_de(.., tinta = true)` (coberto = alfa `≥ 128`). A de tinta vê a cúspide da ARTE onde a tampa redonda encosta tangente noutra borda — `24`–`35` amostras a 1 px (`~2 px²`), com e sem costura: **LIMITE CONHECIDO** da lei (a tampa vive dentro das células; a borda da malha ali é a escada da grelha). Fechá-la pede o contorno da arte (ver recusas: a 1.ª sonda disso estava errada).
- **Fotografado** a `(36°, −145,8° / −144° / −143°)`, costura desligada × ligada: os fios horizontais somem; resta a ponta tangente da arte.
- **Processo:** restaurar uma mutação com `git checkout -- <f>` apaga também o trabalho NÃO COMMITADO nesse ficheiro (a bissecção foi apagada e reaplicada) ⇒ commit antes de mutar.
- ⛔ **Recusas MEDIDAS (não reconstruir):**

  | Recusado | Porquê (medido) |
  |---|---|
  | Bola sobre a borda crua da malha | `58 ms` por imagem por quadro; parede na escada da grelha |
  | Fecho sobre o contorno da arte com `69` nós simplificados posados nas pontas | não dobra um lado recto de `300 px`; sonda apagada por enganar |
  | Costura sem filtro de ossos | cose os «V» das juntas (forma aprovada) |
| Interpolação LINEAR da distância entre amostras para cortar as pontas | mente quando o ponto mais perto muda de segmento entre duas amostras: pedaços a coser vãos de `3,9` texels (lei: `2`; sem corte nenhum `6,2`) ⇒ bissecção sobre a distância real (`12` passos): `2,0000` em `−150°…−138°` (passo `0,1°`) |
| Uma RAMPA no fim de cada troço (fechar tudo até `2` texels e uma fracção a cair até zero entre `2` e `4`) para tirar o degrau do corte | fotografada a `(36°, −143°)` e `(36°, −143,5°)` contra o corte seco: troca o degrau de `~2 px` por uma cunha FINA que afina devagar (as bordas ali são quase paralelas) — o fio volta (`145` amostras a `1 px` a `−143,5°`, `152` sem costura). O que resta nos dois é uma fenda REAL de `2`–`3` px, que a lei deixa aberta por ser mais larga que um fio |

- Commit `13285066c`. Handoff de continuação: [`HANDOFF_line_Vector_F48_O_FECHO_DA_IMAGEM_2026-10-02.md`](handoffs/HANDOFF_line_Vector_F48_O_FECHO_DA_IMAGEM_2026-10-02.md).

### F48 — ⭐⭐⭐ **A IMAGEM PRESA FECHA O VÃO ENTRE OS MEMBROS** (o aberto do handoff de 2026-10-01 §6: *«a IMAGEM presa mostra um risquinho no encontro dos membros»*)

- ⛔ **SUBSTITUÍDA pela F49** (a bola sobre a borda da malha foi retirada; a ph2d-vec-boolean voltou ao main).

- **Reproduzido:** cena `PH2D_VEC_BONE_SMOKE=4` a `(36°, −144°)` com `TRACO=0.3`. Foto: o pixel `(870, 658)` era a cor do FUNDO pura `(86, 93, 109)`, com laranja dos dois lados ⇒ **buraco**, não mistura de AA.
- **Mecanismo (bissecção):** `PH2D_SKIN_GPU=0` dá o MESMO pixel ⇒ não é a placa. A sonda `diag_de_onde_vem_o_risquinho` ([`smoke_bone_par_fresta_tests.rs`](../../crates/ph2d-app-vec/src/smoke_bone_par_fresta_tests.rs)) mede `0` nós pendurados em `9 091` triângulos (a malha conforma); os triângulos dos dois lados de cada buraco ficam a `~350 px` no REPOUSO ⇒ é um **VÃO entre dois membros** (a borda de cima do membro de baixo e a do membro dobrado de volta), uma cunha de `~20 px²`. No desenho a bola da silhueta (F41/F44) fecha-a; a imagem não passava por lei nenhuma.
- **Cura:** `ph2d_vec_boolean::fecho_da_borda(path, quinas) -> Vec<VecPath>` ([`overlap.rs`](../../crates/ph2d-vec-boolean/src/overlap.rs)) = união SEM solda (`uniao_da_borda`) → `rola_a_bola` por fora → ilhas (`fecha_as_ilhas`, extraída e agora PARTILHADA com `silhueta_da_pele`, byte-idêntica). Devolve o que o fecho ACRESCENTA (`fechado − união`). Sem ganchos/esporões/abertura: limpam artefactos de cúbicas do assado, e a abertura só tira área.
  - Porta nova [`skin_image_fecho::malha_desenhada`](../../crates/ph2d-skeleton-live/src/skin_image_fecho.rs), chamada pelo `attach_skin_meshes` (a escolha placa/CPU mudou-se para lá): anéis da borda (`aneis_da_borda`, arestas de UM só triângulo) · borda posada pela porta corrigida (malha só-de-borda) · quinas = viragem de REPOUSO > `PAREDE_MINIMA` (lei F42) · acréscimo triangulado (`ph2d_poly2d::triangulate`) com **UV = a do ponto mais perto da borda** (a tinta da beira). Quadro COM vão posa na CPU; SEM vão a placa posa ao bit. `PH2D_SKIN_CONTACTO=0` desliga (mesma porta do desenho). `PH2D_BONE_LOG=1` imprime a espessura de cada vão em texels.
  - **`ESPESSURA_MINIMA_EM_TEXELS = 1/32`**, tabela MEDIDA: `(40°, 40°)` 5 pedaços de `0,0008`–`0,0031` texel (ruído, `0,007 px²`, 40 triângulos que tiravam a malha à placa) · `(36°, −144°)` `0,79` texel (`20,3 px²`) · `(120°, 120°)` `0,12` e `0,88` texel (os bicos dos dois «V», `13,2 px²`).
- **Medido** sobre os `279` nós da borda: `silhueta_da_pele` `38 ms`, `fecho_da_borda` `0,35 ms`; mesmo enchimento `20,31 px²`; `90/90` amostras do furo cobertas.
- **Fotos:** o fio solto SUMIU a `(36°, −144°)`; o que fica é o bico do «V» da ARTE (tampa redonda tangente à borda), que a lei preserva por desenho (o `RAIO_DO_VINCO` foi calibrado para o «V» da imagem). A `120°` só os dois bicos mudam (arredondados, `5×5 px` cada).
- **Gates (todos novos):** [`smoke_bone_par_fresta_tests.rs`](../../crates/ph2d-app-vec/src/smoke_bone_par_fresta_tests.rs): `o_vao_entre_os_membros_da_imagem_fecha` (CONTROLO: com a lei desligada `> 50` amostras de fundo) · `com_vao_a_placa_cede_a_cpu_e_o_vao_fecha_igual` · `sem_vao_a_malha_sai_ao_bit` (a `40°`, nas duas portas, e a placa continua a posar) · `a_tinta_do_enchimento_e_a_da_beira` (alfa `> 0`; mínimo medido `111` no texel `(557, 0)`, a borda suave da tampa) · `na_dobra_forte_os_bicos_do_v_arredondam`. `skin_image_fecho_tests`: 3 gates dos anéis. [`overlap_tests.rs`](../../crates/ph2d-vec-boolean/src/overlap_tests.rs): `a_uniao_do_fecho_da_borda_nao_desloca_a_polilinha` (com controlo). `as_duas_midias_vivas_chegam_pela_porta_corrigida` ([`skin_image_gpu_tests.rs`](../../crates/ph2d-skeleton-live/src/skin_image_gpu_tests.rs)) passou a ler `skin_image_fecho.rs` (a agulha segue o código). Mutação **10/10** a sangrar (união sem solda→com · sem ilhas · sem bola · contacto desligado · placa sem ceder · limiar `0` e `1` · UV `[0,0]` · anéis invertidos · quinas desligadas).
- ⛔ **Recusas MEDIDAS no caminho (não reconstruir):**
  1. **Enchimento = fechado − borda CRUA:** a solda da união deslocava o contorno ⇒ a `120°` uma tira de `2,5 m × 0,26` texel, e a borda do recorte descia `1,3 px` com degrau.
  2. **Enchimento = fechado − união SOLDADA:** o arco encostava na corda e deixava `1 px` de fundo entre ele e a malha (foto, bico de baixo a `120°`). ⇒ a união da borda NÃO solda. Piso medido no «oito» de `2 000` nós: sem solda `1,8e-7`, com solda `2,0e-5`.
  3. **Régua de pixels «fundo entalado a ≤ 4 px por cima e por baixo»:** serve a um vão FECHADO mas CONTA a ponta de uma baía aberta (`25` amostras a `4 px`, `1` a `1,25 px` com o enchimento certo) ⇒ a guarda do fio é o MECANISMO (gate na `ph2d-vec-boolean`), não pixels.
- Commit `7cf837a9a`. Handoff de continuação: [`HANDOFF_line_Vector_F48_O_FECHO_DA_IMAGEM_2026-10-02.md`](handoffs/HANDOFF_line_Vector_F48_O_FECHO_DA_IMAGEM_2026-10-02.md).

### F47 — ⭐ **A CUNHA ONDE OS MEMBROS SE TOCAM** (ordem do dono, 2026-10-01: *«Parece ok. siga»* — o último aberto da varredura)

- **Medido** a `(36°, −144°)` (`21°`): os dois membros encostam e a união deixa entre eles uma cunha de `~20°`. A cúbica que chega ao fundo dela passa `~1` solda ALÉM do nó e volta pela mesma recta, e a quina VERDADEIRA (`~160°`) fica escondida DENTRO da cúbica — a bola não a lê como vinco (o nó vira `21°` pelas tangentes) e o desfazer dos ganchos não a pode trocar (a viragem real passa dos `150°` dele).
- **Cura:** o esporão também DENTRO da cúbica ([`ph2d_vec_boolean::esporao`]) — a cúbica acaba onde passou pelo nó pela 1.ª vez se o resto dela for e voltar pelo MESMO caminho; nas duas pontas. Com a quina à vista a bola fecha a cunha.
- ⛔ **E só quando a quina escondida passa do limiar do gancho (`150°`)** — a 1.ª versão aparava toda passagem dobrada e a varredura acusou uma pose NOVA, `(166°, −40°)` com `18°`: a quina escondida ali é MANSA, a bola arredondava-a com um arco, e aparar trocava o arco por um nó em bico. Abaixo do limiar quem resolve são a bola e o desfazer dos ganchos; o aparar é para o que nenhum dos dois vê.
- ⛔⛔ **E a F46 cortava o que o artista DESENHOU:** o esporão de nó ignorava as quinas do artista — um bigode traçado em ida e volta sairia. Quem o apanhou foi o CONTROLO de um gate antigo (F42: *com as quinas lidas no deformado o gancho fica*), que deixou de reproduzir. ⇒ esporão e aparar respeitam a lista de quinas.
- ⚠️ **A passagem pelo nó é procurada REFINADA**: uma cúbica rápida anda `~0,016` entre duas amostras, mais que a solda, e passa EXACTAMENTE pelo nó entre elas — a mutação que tirava o teste da dobra sobreviveu até isso. A cerca «passa PELO nó» (`0,1` solda) foi **retirada**: nenhuma fixtura a distinguia.
- **Gates:** `a_cunha_onde_os_membros_se_tocam_fecha` (3×3 poses; CONTROLO: os ganchos + a bola sem o aparar deixam a quina) · `o_esporao_dentro_da_cubica_sai` (CONTROLOS: o regresso por outro caminho, o LAÇO que passa exactamente pelo nó e a quina escondida MANSA ficam ao bit) · `o_esporao_no_inicio_da_cubica_sai` · `a_ponta_desenhada_fica`. Mutação **6 de 6** (a 1.ª corrida deu `4 de 6`: as duas sobreviventes eram uma cerca sem fixtura, retirada, e um controlo mascarado por ela).
- **Medido, varredura a passo `2` (`16 110` poses, C e Z) contra a F46:** a `(36°, −144°)` curada, **`0` pioradas**, **`0` poses acima da `PAREDE_MINIMA`** (pior viragem `10,9°`) — a varredura fecha. Mudaram mais `7` poses boas: seis andam `0,04`–`0,17` solda (ponto→segmento). ⭐ A sétima, `(0°, 148°)`, anda `20,7` solda e é **outro esporão escondido curado**: a recta chegava `0,026` além do nó da junção e voltava num caracol dentro da cúbica seguinte (viragem invisível às tangentes); sem ele a quina de `135°` fica à vista e a bola arredonda-a como a todas — a área muda `+0,05 %`.

### F46 — ⭐⭐ **O BRAÇO DOBRADO DE VOLTA** (ordem do dono, 2026-10-01: *«siga nas correções com cuidado para não danificar o que já temos»* — o aberto que a F45 deixou)

- **Medido:** com uma junta a `174°`–`180°` a pele dos dois membros quase coincide, e a união deixa no contorno de fora **quatro** defeitos diferentes — a varredura da F45 tinha `574` poses más nesta família. Cada um tem a sua peça, e cada peça tem uma mutação que a prova:
  - **Dentes e fendas de `~0,02`** (`85°`–`132°`): a bola fechava o lado côncavo e ninguém tirava a saliência convexa mais fina que ela ⇒ **ABERTURA** (a mesma bola a rolar por DENTRO) entre dois fechos. ⚠️ Na abertura **todo nó do artista é parede** (viragem `180°`), seja qual for a de repouso: uma ponta desenhada fica em ponta mesmo afiada pela deformação.
  - **O esporão** (`(0°, 174°)`, `180°`): um segmento recto que REFAZ o fim do anterior. Área zero ⇒ não é forma, de nenhum tamanho; a bola lê o sentido de uma viragem de `180°` por um produto vectorial que é ruído, e o desfazer dos ganchos pára no tamanho da bola ⇒ [`ph2d_vec_boolean::esporao`] corta o longo onde o curto acaba e tira o curto.
  - **A fenda de boca estreita** (`(176°, 100°)`): os toques são dois nós CONVEXOS, e o offset de um nó convexo é um **ARCO** de raio `r` à volta dele, não a corda entre as duas normais — com a corda o centro ficava perto demais da boca, a bola nunca estava vazia e a fenda ficava aberta ([`bola_toque::ArcoDoNo`]).
  - **O fundo do canal que se fecha** (`(176°, −142°)` `112°`, `(144°, −174°)` `150°`): a bola certa ERA encontrada e era recusada por `0,2 %`–`0,3 %` do raio — o centro tirado das CORDAS amostradas, contra a `FOLGA_DA_BOLA` de `0,1 %`. ⇒ só quando **nenhum** candidato passa pelas cordas, cada um é julgado pelo centro EXACTO. ⚠️ A ordem é load-bearing: com um candidato vazio pelas cordas a resposta é a de sempre, ao bit.
- **Medido, varredura a passo `2` (`16 110` poses, C e Z) contra o `HEAD`:** `574` curadas, **`0` pioradas**; sobra `1`, a `(36°, −144°)` com `21°`, **igual ao `HEAD`** (isolada, anterior). Das `1 222` poses boas que mudaram, **fora da família o contorno andou no máximo `0,40` solda** (ponto→segmento); dentro dela até `0,9 r`, que é a abertura a achatar o nó mais fino que a bola (visto na foto da diferença).
- **Gates:** `o_braco_dobrado_de_volta_nao_deixa_dentes` (`~270` poses da família; CONTROLO: o fecho de antes deixa bico) · `o_esporao_sai_de_qualquer_tamanho` · `o_que_nao_e_esporao_fica_intacto` · `o_esporao_de_entrada_sai` · `uma_fenda_de_boca_estreita_e_fechada` (CONTROLO: a larga fica aberta) · `a_abertura_nao_come_a_ponta_desenhada` (CONTROLO: sem quinas as pontas arredondam). Mutação **6 de 6**.
- **Cortes de LOC:** a geometria do toque saiu de `bola.rs` para `bola_toque.rs` (`778 → 672`); os gates do braço dobrado forte para `skin_desenho_dobra_tests.rs`.
- ✅ ~~**ABERTO:** a pose isolada `(36°, −144°)`~~ — fechada na F47.

### F45 — ⭐⭐ **O GANCHO QUE A UNIÃO DEIXA** (ordem do dono, 2026-10-01: *«siga nas correções com cuidado para não danificar o que já temos»* — a família que a F44 deixou aberta)

- **Medido:** com o braço em C e as duas juntas a somar `~238°` (`(130°,108°)` … `(170°,70°)`), a união corta uma cúbica do assado DENTRO da dobra dela e o nó novo sai com a 2.ª alça `0,035`–`0,056` além dele: meia-volta de `171°`–`179°` no contorno de FORA, e o traço desenha a meia-lua (foto). ⛔ A bola não lhe toca (a ponta é para fora); o desenho sem contacto vira só `0,7°` — quem cria o gancho é a união.
- **Cura em duas peças:** (1) **o desfazer dos ganchos corre também DEPOIS da união**. ⚠️ A F43 tinha-o retirado porque a mutação que o apagava sobrevivia — a varredura que a julgou só dobrava em Z; com o braço em C ele tem casos. (2) **a régua aceita o recuo SOBRE O PRÓPRIO CAMINHO**: a `(166°,74°)` a cúbica passa do nó `0,0059` (`1,4×` a solda, logo a régua da cúbica sozinha recusava) e volta pela MESMA recta — o pedaço a mais fica a `0,0003` do resto do contorno, invisível como forma. A troca faz-se quando o CONTORNO inteiro muda menos que a solda, até ao tamanho da bola (`RECUO_SOBRE_SI = RAIO_DO_VINCO / SOLDA_DA_QUINA`) — ⚠️ o gate do recuo de meio lado continua de pé.
- ⚠️ **Os buracos NÃO levam a passagem**: em `3 540` poses em C nenhuma ilha deixou gancho, e uma passagem sem caso medido seria lei sem régua.
- **Medido:** varredura a passo `4` (`4 050` poses, C e Z) contra o `HEAD`: `3` curadas, **`0` novas, `0` mudadas**; a passo `2` (`16 110`): `575` restantes, `574` com uma junta a `174°` ou mais (abaixo) e `1` isolada, `(36°,−144°)` com `21°`, **igual ao `HEAD`**.
- **Gates:** `o_gancho_que_a_uniao_deixa_sai` (CONTROLO: sem a passagem a faixa fica em gancho) · `o_recuo_sobre_o_proprio_caminho_sai_ate_ao_tamanho_da_bola` (o que volta pela recta sai; o maior que a bola fica; o que sai DA recta fica). Sonda versionada `diag_a_varredura_da_dobra`. Mutação **4 de 4**.
- ⏳ **ABERTO:** o braço dobrado de VOLTA (uma junta a `174°`–`180°`) — próximo; e a pose isolada `(36°,−144°)`.

### F44 — ⭐⭐ **AS ILHAS: a bola rola por DENTRO** (report do dono, 2026-10-01, três fotos: *«smoke ok! Parece muito bom! Falta apenas corrigir o stroke quando uma parte do membro se sobrepõe a outra formando uma ilha. Nessa ilha as quinas ainda não estão corretas»*)

- **Medido:** o braço dobrado em C fecha-se sobre si e a união deixa um BURACO (`subpaths`); a bola rolava só no contorno de fora ⇒ as ilhas ficavam com cantos até `153°`, e a junta do traço abria o espinho para dentro do preenchimento.
- **Cura em três peças, cada uma achada por medição depois da anterior:** (1) `bola::rola_a_bola_por_dentro` — o lado de fora sai da orientação do PRÓPRIO contorno (área), logo inverter os vértices não serve: um parâmetro `Lado`. (2) **A janela de procura passa a meia volta** — numa ilha de perímetro `0,89` contra `32 r ≈ 1,2` o lado esquerdo comia a volta inteira e o direito ficava com zero arestas; o contorno de fora não muda AO BIT (o perímetro de uma curva fechada é `≥ √2·diagonal = 141 r` contra os `64 r` das duas janelas). (3) **Sem nó livre a montagem parte do FIM de um arco** — numa ilha triangular todo nó é canto, e a bola desistia. E a **ilha onde a bola não cabe é cheia inteira** (o fecho), `ilha::a_bola_cabe_dentro` (grelha `64×64`, do centro para fora).
- **Medido** (`3 540` poses em C, `60°`–`176°` × `60°`–`178°` de dois em dois): poses com canto de ilha acima de `15°` **`595 → 0`**; o contorno de fora **idêntico** ao `HEAD` (os mesmos `113` nós, todos do braço dobrado de volta — abaixo).
- **Gates:** `nas_ilhas_a_bola_rola_por_dentro` (três metades: a fixtura tem ilha em bico · alguma sobrevive arredondada · alguma é fechada) · `a_bola_arredonda_os_cantos_de_uma_ilha` (ilha pequena e grande, com o CONTROLO de que por fora a bola não toca) · `a_bola_cabe_so_onde_o_raio_inscrito_e_maior_que_o_dela` (`±5 %`). Mutação **7 de 7**.
- ⚠️ **Fechar a ilha é um salto**, inerente ao fecho: quando o raio inscrito passa por `r` o buraco vai de um disco de raio `~r` a nada (com traço, o anel do buraco some). O mesmo salto que um vinco já tinha no contorno de fora.
- ⏳ **ABERTO e PRÉ-EXISTENTE:** no contorno de fora, além do braço dobrado `≥ 176°`, uma família de ganchos `171°`–`179°` com as duas juntas a somar `~238°` (`(130,108)` … `(170,70)`) — medida igual no `HEAD`, por investigar.

### F43 — ⭐⭐ **A MEIA-LUA DA JUNTA QUASE RECTA** (report do dono, 2026-09-30, com foto: *«quase perfeito, artefatos curados na quina dobrada. resquício quando quase reto»*)

- **Reproduzido** com o braço da cena: a `(125°, 34°)`/`36°` e a `(110°, 10°–17,5°)` o produto tinha um nó a virar `~180°` num lado quase RECTO. Na dobra do mapa a velocidade do contorno chega a zero mesmo com a junta a `~35°`, e o assado devolve uma cúbica que DOBRA — a tangente invertida num nó (a 2.ª alça `0,006` atrás do nó) ou um **zigue-zague de `0,0016`** de largura por dentro dela. Invisível como forma; o traço desenha a meia-lua. ⛔ Não é vinco côncavo, logo a bola não lhe toca; e a união **piorava-o** (lia o zigue-zague como cruzamento e reescrevia-o num dardo real de `~0,03`).
- **Cura:** `ph2d_vec_boolean::gancho::desfaz_os_ganchos` — por segmento, se a cúbica dobra (um passo vira `> 150°` contando com as tangentes dos vizinhos), é trocada pela Hermite que segue os vizinhos, só se a forma nova ficar a menos da solda (Hausdorff ponto→segmento). Corre **ANTES da união**; nunca toca numa quina do artista (a regra passou a ser UMA função, `bola::quina_do_artista`, com os dois leitores).
- **Medido:** `1 053` poses (`110°`/`125°`/`140°` × `0°`–`175°`) — nenhum nó acima de `15°` até `175°`; fotos limpas a `17,5°` e `34°`.
- **Gates:** `com_a_junta_quase_recta_nao_sobra_meia_lua` (com os dois CONTROLOS: a fixtura dobra, e união+bola sem o passe deixam-no) · `o_gancho_microscopico_sai_e_o_que_se_ve_fica` · `o_laco_dentro_da_cubica_sai`. Mutação **5 de 5** (o passe antes da união, o filtro das candidatas, a tolerância, a quina do artista); uma 2.ª passagem DEPOIS da união **saiu** — a mutação que a apagava sobreviveu e a varredura deu o mesmo.
- ⭐⭐ **F43-bis — o recuo MAIS FINO QUE A AMOSTRAGEM** (report do dono no mesmo dia, foto: *«quase perfeito»*, uma meia-lua por cima da junta de cima com as DUAS juntas a dobrar para o MESMO lado). ⛔ **A varredura da F43 só dobrava a junta de cima AO CONTRÁRIO** (`segunda ∈ [0, 70]` com sinal negado) — a outra metade nunca tinha sido medida. Varrida (`60°`–`150°` × `−40°`–`10°`), o defeito vivia em **`−10,5°`** com a de baixo de `60°` a `84°` — a pose da foto. ⭐ **O mecanismo:** a 2.ª alça EM CIMA do nó e a 1.ª `0,00088` ALÉM dele ⇒ a cúbica recua só para `s < 0,0136` (o último `1,4 %` do parâmetro), e as `32` amostras do `dobra` saltam-no: todos os passos leem o mesmo sentido, e a tangente de chegada aponta ao CONTRÁRIO. ⇒ a sequência do `dobra` ganha as tangentes **EXACTAS** das duas pontas (`inicio(c)` · `fim(c)`). ⛔⛔ **E isso partiu o `numa_dobra_forte_o_desenho_nao_se_cruza` à primeira:** uma alça a **`1,2e-11`** do nó (arredondamento do assado, a `45°`, sem contacto) lia-se como tangente ao contrário com o `dir` de limiar `1e-12`, e o passe «curava» um gancho que não existe ⇒ o limiar do `dir` passa a ser **a precisão do `f32` à escala das coordenadas** (quem desenha é a placa, e para ela aquela alça É o nó). **Medido:** `6 851` poses (`60°`–`150°` passo `3` × `−40°`–`70°` passo `0,5`) com **zero** nós acima de `15°`; foto a `(84°, −10,5°)` limpa. Gates `o_recuo_mais_fino_que_a_amostragem_sai` (CONTROLO: as amostras sozinhas NÃO o veem) e `com_as_duas_juntas_no_mesmo_sentido_nao_sobra_meia_lua`; mutação: tirar as tangentes exactas reprova os **dois**, e o limiar `1e-12` reprova o `numa_dobra_forte…`.
- ⏳ **ABERTO:** o braço dobrado de VOLTA sobre si mesmo (`175°`–`180°` na junta de cima) — a borda sai irregular na foto e `2` poses ficam com um nó de `~92°`. Pré-existente (já assim na F41/F42) e outro mecanismo: a sobreposição quase total de dois membros.
- ⚠️ `PAREDE_MINIMA` passou a pública (lida pelos gates).

### F42 — ⭐⭐ **O GANCHO DA DOBRA NÃO É QUINA DO ARTISTA** (report do dono, 2026-09-30, com duas fotos: *«melhorou muito o ângulo e suas transições. Restam os artefatos de imagem»* — fatias de cinzento e de laranja dentro do castanho, no vinco)

- **Reproduzido na cena `=4`** a `(125°, 85°)` com `TRACO=0.3`, e despejado do produto: na DOBRA do mapa um nó do ASSADO vira `180°` (um gancho de raio `~0,005` no vinco). A bola lia as quinas no desenho **DEFORMADO** (`nos_do_desenho(d)`), o gancho passava por quina desenhada (`viragem ≤ a dele + 1°`), ficava, e o traço sobre a meia-volta tinha `311°` de curva mais apertada que a caneta — as fatias.
- **Cura:** as quinas são os nós da **FONTE**, onde o assado os pousou, com a viragem do **REPOUSO** — `ph2d_vec_skin::curva::assa_a_pele_com_nos` (a âncora que o `rebuild` dá a cada nó) + `skin_desenho::quinas_do_artista`; a porta passa a ser `silhueta_da_pele(path, quinas)` e o `quinas_de` serve só um desenho que é o seu próprio repouso.
- **Medido:** varredura de `891` poses (`100°`–`150°` × `70°`–`110°`, meio grau) — gancho no desenho em `16`, a lei de antes deixa-o em `11`, o produto em `0`. Foto limpa a `84,5°` e `85°`.
- **Gates:** `o_gancho_da_dobra_nao_e_quina_do_artista` (fixtura nova `braco_da_dobra_forte`, nas proporções da cena — ⛔ a barra de sempre **não contém o gancho**: com ossos de `2,13` espessuras a dobra fica lisa) e `a_quina_desenhada_continua_em_bico` (um «L» cuja raiz roda `30°`, com o CONTROLO sem quinas a arredondá-lo). Mutação **3 de 4**; a 4.ª (o `ponto(0)` no lugar da âncora) é **equivalente no corpus** e está nomeada no código.
- ⛔ **RECUSA MEDIDA:** a bola com raio mínimo igual à meia-largura do traço (hipótese: arco `r < h` abre buraco pela curva paralela invertida). Fotografada de `80°` a `105°` com e sem ela: as duas limpas, e ela mudava o que o dono aprovou (o castanho/laranja de dentro passava de raio `r + h` a `2,05·h`). Registo no doc do `RAIO_DO_VINCO`.

### F41 — ⭐⭐⭐ **A BOLA QUE ROLA: nenhum canto interno da pele fica mais apertado que um raio** (report do dono, 2026-09-30, com três fotos: *«melhor mas ainda inconsistente. veja que o ângulo da linha arredonda demais, não é progressivo. e veja que ainda produz artefatos circulares»*)

⛔⛔ **A F40 estava certa sobre o QUÊ e errada sobre o QUANDO e o QUANTO.** A sonda
`diag_a_zona_da_dobra` mostrou que as três fotos são UMA lei a faltar:
- **Antes do contacto a pele já aperta até ao bico** — o raio côncavo do lado de dentro vai de
  `0,12` raio a `85°` para `0,002` a `92°` **sem o contorno se cruzar** (a dobra do mapa,
  `det J = |1 − θ̄′·r|`), e o filete da F40 só corria no cruzamento. O bico é a foto 1; o fundo a
  aparecer dentro dele com a ponta redonda da junta é o «artefato circular» da foto 3.
- **O cruzamento liga e desliga de grau para grau** (`88°`–`89°` cruza, `90°`–`92°` não, `93°`
  cruza): o canto saltava de bico para arco e de volta — o «não é progressivo».
- **O arco de um vinco raso era ENORME** (o piso `r` da F40 dava raio `r/tan(α/2)`): o «arredonda
  demais» da foto 2.

⭐⭐⭐ **A lei: o FECHO morfológico por uma bola de raio `r`, só no lado côncavo, SEMPRE**
([`bola.rs`](../../crates/ph2d-vec-boolean/src/bola.rs), chamada por
[`silhueta_da_pele`](../../crates/ph2d-vec-boolean/src/overlap.rs) — a união continua a correr só no
cruzamento). A bola rola por fora; onde não cabe, o contorno passa a ser o arco dela ⇒ **contínuo
na forma**: o canto aperta aos poucos, fica em `r`, e é o mesmo antes e depois do encosto. Um vinco
de viragem `α` recebe o arco tangente a `r·tan(α/2)` (o da F40, **sem** o piso); uma curva mais
larga que `r` sai **ao bit**.

Leis que a MEDIÇÃO impôs, cada uma por um gate vermelho:
1. **O centro é EXACTO** (Newton sobre os dois toques): o cruzamento das paralelas AMOSTRADAS
   deixava o arco um nada mais apertado que `r` e rolar outra vez trocava-o (dobra em C a `135°`).
2. **Pedaços de `45°`, não de `90°`**, e limiar `APERTO = 0,99`: a cúbica de um quarto de círculo
   desce a `0,992 r` e a bola re-lia o próprio arco como apertado — o fecho deixava de ser
   IDEMPOTENTE.
3. **A tangente nos toques é a do CÍRCULO** — num toque que cai num nó que já é quina, a da curva
   torce o 1.º pedaço (`0,98 r`, junta única a `125°`).
4. **A fusão de dois vãos vai do mais à esquerda ao mais à direita** (a 1.ª redacção perdia os dois
   a `91°`) e **um vão contido noutro sai** (dobra em Z a `80°`).
5. **Uma parede é uma quina do artista que AINDA é quina** — a união pode alisar um nó do desenho
   (dobra em C a `135°`), e o limiar do ruído é UMA solda de corda (`~5,7°`), não duas (um vinco de
   `9°` ficava em quina a `125°`).
6. **SEM solda depois da bola** — ela fundia o toque com um nó a `3,3 mm` e tirava o arco do sítio.
7. ⛔⛔ **O DETECTOR de cruzamento tinha um falso positivo**, e foi o último vermelho (pose do dono a
   `98°`): o achatamento emite o ponto CALCULADO da cúbica perto de `t = 1` e depois o vértice
   GUARDADO, que diferem por **um ULP**; os dois vizinhos desse segmento de `1e-15` deixam de
   partilhar os bits e dois segmentos quase colineares que se TOCAM liam-se como um par que se
   ATRAVESSA ⇒ a união corria sobre uma silhueta já resolvida. A cura **cola** um passo a menos de
   `FECHO_EXACTO = 1e-12` da diagonal ao segmento anterior (no meio e no fecho). ⚠️ A 1.ª cura tratou
   só o FECHO e o gate continuou vermelho: o nó era um vértice LISO no meio do contorno — *quem o
   expôs foi a bola rodar o vértice de partida, e o defeito veio atrás dele*.

8. ⛔⛔ **A bola tem de estar VAZIA** — achado pela prova de mutação, não por um report: cinco
   mutações SOBREVIVERAM (sementes convexas, a poda e a fusão dos vãos, pedaços de `90°`, a parede
   do ruído) e a sonda que se escreveu para lhes dar corpus achou um DEFEITO: um **dente convexo**
   dentro de um vinco mais estreito que a bola saía com dois arcos cruzados por cima dele e uma
   meia-volta de `180°`, não idempotente. O toque mais BARATO pousava num vale do dente e a bola
   atravessava o outro lado ⇒ os candidatos correm por custo e fica o primeiro cuja bola não tem
   ponto nenhum do contorno dentro (`FOLGA_DA_BOLA = 0,999`). ⚠️ O `v > PAREDE_MINIMA` da parede
   SAIU: as duas condições da viragem actual já o implicam (`v ≥ vira − 1° > 14°`).

- **Gates:** `um_fundo_mais_estreito_que_a_bola_e_engolido_inteiro` (fundo plano de `0,2`–`1,2 r`, em
  W de 3 a 5 cantos, e o DENTE: nenhuma zona abaixo de `0,99 r`, nenhuma quina nova, idempotente e
  o fundo inteiro engolido) · `bola_tests.rs` (o entalhe de `166°` vira o arco da bola, a quina do artista fica, o
  convexo fica · uma curva mais apertada que a bola vira o arco dela, idempotente · os nós lisos
  dentro do arco saem · o ruído do assado não é tocado) · `nenhum_canto_da_silhueta_e_mais_apertado_que_a_bola`
  (C · Z · uma junta de `60°` a `150°`, e a pose do dono de `60°` a `150°` grau a grau: nenhuma zona
  apertada abaixo de `0,9 r` maior que `12°` e **rolar outra vez não muda nada**; piso: pelo menos
  uma silhueta tocada ANTES do cruzamento) · `um_fecho_a_um_ulp_do_inicio_nao_e_um_cruzamento` ·
  `um_passo_de_um_ulp_no_meio_nao_e_um_cruzamento` (os números medidos, com o CONTROLO de que a
  folga de um ULP existe).
- ⚠️ **Os dois gates do padrão-ouro passaram a medir as DUAS etapas separadas** (`o_que_se_ve(p,
  contacto)`): a LEI do desenho contra o ideal com a barra de sempre (`0,01`), e o que se vê com a
  bola a no máximo **uma bola** além da lei (medido a `90°`: `0,038` contra `0,004 + 0,058`). Julgar
  as duas juntas acusaria o arredondamento PEDIDO de ser um defeito da lei.
- ⚠️ **Divergência DECLARADA:** uma curva côncava LISA do desenho mais apertada que `r` (`1 %` da
  diagonal) também é alargada — não há correspondência entre o repouso e o assado que a poupe.
- **Recusas medidas:** o filete da F40 só no cruzamento (bico antes do encosto) · o piso `r` no
  recuo (arco enorme num vinco raso) · janela de `8` raios (o entalhe de `166°` ficava fora) · pedaços
  de `90°` (não idempotente) · o centro das cordas amostradas · solda depois da bola.
- **Mutação `27 de 30` + controlo, com as 3 que sobram NOMEADAS**
  ([arnês](ferramentas/muta_a_silhueta_do_contacto.sh)): o bloco `V*` do filete SAIU com a lei dele;
  entram `B1`–`B14` (a bola) e `D1`–`D2` (o detector). ⚠️ **B2** (semente convexa) é equivalente na
  forma — as paralelas de fora de uma quina convexa divergem e a bola não pousa; **B8/B9** (poda e
  fusão dos vãos) foram escritas contra casos que o centro exacto e a bola vazia dissolveram, e o
  percurso já salta um vão contido; ficam como rede. ⛔ E **Q1** (a solda da união) passou a
  SOBREVIVER com a bola a engolir os restos junto do vinco — o gate voltou a medir a UNIÃO na própria
  etapa, e sangra.

### F40 — ⭐⭐⭐ **O VINCO DO CONTACTO É UM ARCO** (report do dono, 2026-09-30, com cinco fotos: *«além de inconsistente, fica tão pontudo que perfura o outro lado da forma»*; decisão dele entre três saídas: **arredondado**)

⛔⛔ **A F39 tinha duas metades e a segunda estava ERRADA.** Subir o limite do bico para `10` fazia
o `Miter` obedecer ao painel — e o bico de um vinco CÔNCAVO aponta para DENTRO do preenchimento,
com comprimento `(w/2)/sin((180 − viragem)/2)`. Medido na pose das fotos (junta de baixo a `125°`,
a de cima a `95°`–`115°`): o vinco vira `124°`–`161°`, e perto do início do contacto a viragem
tende a `180°` ⇒ o bico ia até `5×` a largura e **atravessava a peça**. Com o limite `4` ele vira
chanfro a partir de `~151°` e a quina muda de forma com o ângulo, que era a queixa de antes.
⇒ **nenhum limite serve as duas queixas**; a cura é o vinco deixar de ser quina.

⭐⭐⭐ **A lei: todo vinco que a UNIÃO cria é trocado por um arco tangente** —
[`arredonda_os_vincos`](../../crates/ph2d-vec-boolean/src/overlap.rs), de raio
[`RAIO_DO_VINCO`] = `1e-2` da diagonal. O traço desenha-se por cima dele como um arco de raio
`r + ½·largura` ⇒ **arredondado com qualquer junta** e em todo ângulo; as quinas que o ARTISTA
desenhou continuam com a junta do painel. O `MITER_LIMIT` **volta a `4`** (o de omissão de SVG,
kurbo, vello e Skia) e continua UMA porta com os três leitores da F39.

Quatro leis, todas MEDIDAS na barra dobrada, e três delas por gates vermelhos:
1. **Um vinco é um vértice que VIRA MAIS do que virava no desenho** — ⛔ não «um vértice que não é
   nó do desenho». O motor da união **ENCAIXA** o cruzamento num nó liso vizinho quando ele cai
   dentro da precisão dele: na pose das fotos o vinco de `161°` sai **exactamente** sobre um nó do
   assado que no desenho virava `0°`. Por isso os nós do desenho viajam com a viragem que tinham
   (`nos_do_desenho`).
2. **Só o CÔNCAVO** (o vinco de uma dobra são dois membros que se unem); a régua da ÁREA o prova —
   arredondar um côncavo só ACRESCENTA área.
3. **O corte mede-se ao longo do CONTORNO, engole os nós lisos, e NÃO pára numa micro-quina:** o
   assado deixa micro-quinas de `1,4°`–`1,7°` nas costuras; tratadas como parede prendiam o arco a
   `45 %` de um segmento de `0,0017`. Parede é outro vinco ou uma quina do desenho acima de
   [`PAREDE_MINIMA`] = `15°`.
4. **O corte nunca cai na solda:** a distância é `r·tan(α/2)` **com piso `r`** (um vinco raso da
   dobra em C a `95°` dava um arco de `0,0013`), e um nó liso logo além do corte é **engolido** até
   ficar a [`FOLGA_DO_CORTE`] = `0,2` raio dele (na pose das fotos a `150°` o 1.º nó ficava a
   `0,0008` do corte).

- **Gates:** `o_vinco_vira_arco_e_a_quina_do_artista_fica` (o arco começa a `r·tan(α/2)` · o meio da
  cúbica cai a `r` do centro, que é o que a alça `(4/3)·tan(θ/4)` garante · a área só cresce · o
  CONTROLO do nó que já virava · o ENCAIXE num nó liso · o CONTROLO convexo) ·
  `nenhum_vinco_da_silhueta_fica_em_quina` (C · Z · uma junta · a pose das fotos, `95°`–`150°`:
  todo vértice que vira é um nó do desenho e nenhum segmento cabe na solda; piso de `25`
  silhuetas resolvidas). ⛔ O `a_viragem_maxima_e_a_do_limite_do_bico` e a própria
  `viragem_maxima` **SAÍRAM** — a lei deles não tem mais onde morar.
- **Recusas medidas:** limite `10` (fura a peça) · limite `4` sem arco (inconsistente com o ângulo) ·
  «nó do desenho» por igualdade ao bit (o encaixe) ou por proximidade (o assado tem nós a `~0,01`
  junto da junta, e um caía dentro da solda).
- **Gate novo por mutação SOBREVIVENTE:** `os_nos_lisos_dentro_do_arco_saem` — deixar os nós lisos
  dentro do arco (a `V7`) passava a suíte inteira, porque o nó é do desenho (o gate da silhueta
  perdoa-o) e as alças dele apontam para a frente (nenhuma viragem o acusa): o contorno ia ao corte,
  voltava ao nó e seguia. A fixtura é o entalhe com um nó a meio de cada lado, e a saída tem de ser
  a mesma do entalhe sem eles.
- **Mutação `23 de 23` + controlo** ([arnês](ferramentas/muta_a_silhueta_do_contacto.sh)): as oito
  `V*` do vinco e as quinze da F38/F39. ⚠️ Q7/Q8 passaram a ser *«volta ao `10`»* — com o limite
  de volta a `4`, apagar o `.with_miter_limit` seria um mutante EQUIVALENTE (é o de omissão da kurbo).

### F39 — ⭐⭐⭐ **A QUINA DO CONTACTO OBEDECE À JUNTA DO PAINEL** (report do dono, 2026-09-30, com duas fotos: *«A depender do ângulo a quina fica inconsistente.. Faça obedecer ao que foi escolhido no painel»*)

⛔ **Duas afirmações desta secção foram SUPERADAS pela F40 no mesmo dia:** o limite do bico voltou
a `4` (o `10` fazia o bico do vinco furar a peça) e a `viragem_maxima` saiu com o gate dela — o
vinco passou a ser arco. As causas 1 e 2 continuam de pé.

⭐⭐⭐ **Eram TRÊS causas, e cada uma desviava a junta noutro ângulo** — por isso a mesma quina saía
em bico, cortada ou com um dente conforme a dobra. Medidas com a sonda
`diag_a_quina_do_contacto` (três formas de dobra × `100°`..`150°` de `5` em `5`, a viragem de
cada vértice pelas tangentes que o TRAÇO usa):

1. **Pedaços MINÚSCULOS junto do cruzamento** (comprimento `0` a `~4e-3` numa diagonal de `4`): a
   tangente de um segmento degenerado é arbitrária. ⇒ [`solda_os_segmentos_curtos`](../../crates/ph2d-vec-boolean/src/overlap.rs),
   com a tolerância [`SOLDA_DA_QUINA`] = `1e-3` da diagonal.
2. **Alças a DISTÂNCIA DE RUÍDO** (`1e-6`–`1e-9`) de uma ponta, às vezes do lado de TRÁS dela, e às
   vezes caídas na ponta de LÁ de uma recta — no motor **e** no bake. A kurbo só troca de tangente
   por coincidência EXACTA, logo a junta via uma meia-volta de `180°` onde a quina tem `~40°`.
   ⇒ [`limpa_as_alcas`] põe o ponto EXACTAMENTE sobre a ponta (um ponto a `≤ tol` muda a curva em
   `≤ tol`). ⛔⛔ **E corre DEPOIS da solda:** o vértice que fica herda a alça de saída do fundido,
   que mora na âncora DELE — foi essa a meia-volta que sobrava a `C 150°` (`1,6e-6` da âncora) com
   a limpeza escrita ANTES. ⚠️ E uma passagem antes **E** depois foi escrita e a de antes **APAGADA**:
   a mutação que a removia sobreviveu (a solda não decide nada a partir das alças limpas). ⚠️ Os
   dois ramos CRUZADOS (a alça caída na ponta de LÁ) também sobreviveram à barra dobrada — eles só
   mordem com a OUTRA alça exactamente na ponta — e ganharam fixture própria.
3. **O `Miter` era cortado em silêncio**: o traço nunca escrevia o limite do bico e a kurbo usa `4`;
   a quina de contacto vira `120°`–`152°`, e acima de `~151°` a razão do bico passa de `4` ⇒
   **o artista escolhia `Miter` e via `Bevel`**. ⇒ [`ph2d_vec_scene::MITER_LIMIT`](../../crates/ph2d-vec-scene/src/stroke_style.rs)
   = **`10`** (o de omissão do Illustrator), **UMA porta com TRÊS leitores**: o ecrã
   (`kurbo_stroke`), o *Outline Stroke* (`expand::line_pen`) e o **SVG exportado**
   (`stroke-miterlimit="10"` — ⚠️ o SVG tem `4` por omissão, e sem ele o ficheiro cortava o bico que
   o ecrã mostra).

**Medido depois da cura:** zero segmentos que caibam na solda e a pior quina a **`152,0°`** nas `33`
dobras, contra a [`viragem_maxima`] = `180 − 2·asin(1/10)` = **`168,5°`** (a viragem em que o bico
chega ao limite) ⇒ **toda quina da silhueta sai com a junta que o painel escolheu**. Gate
`toda_quina_da_silhueta_respeita_a_junta_do_painel` (piso de `20` silhuetas resolvidas).

⚠️ **O preço tem endereço, e é CORRECTO:** a caixa que o renderer reserva para um traço com junta em
bico é `½ · largura · limite` por lado — `5 × largura` em vez de `2 ×`. Ela lê o limite do MESMO
construtor (`standalone::transbordo_do_caminho`), logo acompanha sozinha; sem isso o bico mais
comprido sairia **ceifado** contra a borda do scratch.

⛔⛔ **Um passo que APAGAVA as «lascas» (vértices acima da viragem máxima) foi construído e
APAGADO:** depois da limpeza das alças não tinha **um único** vértice a tirar nas `33` dobras — as
«meias-voltas de `180°`» que ele existia para apagar eram TODAS alças a ruído, não geometria — e
ligado ele arqueava a silhueta para longe do contorno (`9,8e-2` numa diagonal de `4,05`, apanhado
por `a_silhueta_esta_sobre_o_contorno_de_antes`). *Uma linha que nenhuma dobra medida exige não é
lei.*

- **Gates:** `um_segmento_minusculo_e_soldado_na_quina` · `a_alca_herdada_da_solda_nao_torce_a_quina`
  · `uma_alca_caida_na_ponta_de_la_encaixa_nela` · `a_viragem_maxima_e_a_do_limite_do_bico` ·
  `o_bico_do_traco_assado_e_o_do_documento` · `o_bico_do_traco_desenhado_e_o_do_documento` ·
  `o_svg_leva_o_limite_do_bico` · e o do contacto acima.
- **Mutação `16 de 16` + controlo** — as sete da F38 e as nove `Q*` do
  [arnês](ferramentas/muta_a_silhueta_do_contacto.sh), com a população a ser de quem **OBSERVA**
  (o renderer e o SVG entram só nas mutações deles). Duas sobreviveram à 1.ª corrida e as duas
  mudaram o produto: uma passagem apagada, uma fixture escrita.

### F38 — ⭐⭐⭐ **A DOBRA FORTE: o contacto sai como SILHUETA** (ordem do dono, 2026-09-29: *«a dobra forte do cotovelo: tente o estado da arte diretamente»*)

⭐⭐⭐ **O defeito visível NÃO é a pele virar do avesso — é CONTACTO.** Numa dobra forte a face de
DENTRO de dois membros rígidos que rodam em torno de uma junta passa uma por cima da outra; o
preenchimento (não-zero) pinta a região certa e o TRAÇO desenha o contorno inteiro, **com o «olho»
da sobreposição por dentro**. Medido na barra da cena (três ossos, `dobra` nos dois de baixo): o
desenho fiel **não se cruza até `90°` e cruza-se de `110°` a `150°`**.

⭐⭐ **A cura é a do estado da arte, na forma exacta que um caminho vectorial permite:** no contacto
a pele fica na FRONTEIRA DA UNIÃO dos membros (*Implicit Skinning*, Vaillant et al. 2013), e para
um caminho essa fronteira é a união do caminho com o **VAZIO** pelo motor da `ph2d-vec-boolean`
(⚠️ `A ∪ ∅`, **nunca** `A ∪ A` — a regra de multiplicidade do `linesweeper` 0.4). Porta nova:
[`ph2d_vec_boolean::resolve_overlap`](../../crates/ph2d-vec-boolean/src/overlap.rs), ligada no
DESENHADO em `skin_desenho::calcula` — ⛔ **nunca no `cru`**, que são os nós que o artista edita.
⭐ **A imagem presa já faz isto de graça** (um membro por cima do outro, canto em «V»), logo a
porta põe o vector a **concordar com ela**; a imagem não muda.

- **Só corre no contacto:** fora dele o desenho sai **AO BIT** o de antes (gate). Preço medido
  (`--release`, a `load 30` — tecto, não chão): `16`–`26 µs` por forma por quadro recalculado para
  a detecção, `138 µs` quando há união.
- **Bissecção:** `PH2D_SKIN_CONTACTO=0` devolve o contorno com o «olho» (a leitura da porta é pura,
  `contacto_de`, com gate do valor de fábrica).
- **Gates:** `numa_dobra_forte_o_desenho_nao_se_cruza` (três metades: o CONTROLO cruza · com a lei
  nenhuma dobra cruza · sem contacto é ao bit) · `a_silhueta_esta_sobre_o_contorno_de_antes` (pior
  `1,8e-4` numa diagonal de `4,05` — sem ela o CASCO passaria) · `o_contacto_nasce_ligado` · e os
  cinco da porta em `overlap_tests.rs`.
- **Mutação `7 de 7` + controlo** ([`muta_a_silhueta_do_contacto.sh`](ferramentas/muta_a_silhueta_do_contacto.sh)).
  ⛔ **O salto dos VIZINHOS foi APAGADO:** a mutação que o removia SOBREVIVEU — dois segmentos
  consecutivos partilham o extremo com os mesmos bits, um teste de lado dá **zero exacto** e o teste
  estrito já o recusa. *Uma linha que a mutação não mata não é lei.*

⭐⭐ **A cena é a `PH2D_VEC_BONE_SMOKE=4` (A DOBRA FORTE)** — o par da `=3` empilhado e aberto a
`120°` por junta, e ⚠️ **o desenho leva CONTORNO nesta cena, por FOTOGRAFIA**: sem traço as fotos com
e sem `PH2D_SKIN_CONTACTO=0` saíam iguais ao pixel (o preenchimento não-zero já pinta a união) — *o
que a sobreposição estraga é o TRAÇO*. Com ele, sem a lei aparecem dois laços por dentro das juntas;
com ela, dois cantos em «V» com a forma da imagem de baixo.

⛔⛔ **DUAS curas do estado da arte foram construídas na MALHA, medidas e RECUSADAS antes desta** (as
sondas ficam, versionadas: `skinned_mesh_arap_sonda_tests.rs` e `skinned_mesh_contacto_sonda_tests.rs`,
todas `#[ignore]`) — ver a tabela de recusas no fim. ⏳ **ABERTO e nomeado:** o **bico** a `~90°`
(uma cúspide SEM cruzamento — a dobra do mapa da §1.3 da [pesquisa](04_pesquisa_ossos_sobre_desenho_vetorial.md))
continua; a porta não o vê porque não há o que unir.

### F37 — ⭐⭐⭐ **O DESENHO FIEL: os nós do artista ficam, e o que se VÊ segue o padrão-ouro** (ordem do dono, 2026-09-29: *«hoje nosso problema é o uso de osso com desenho vetorial… buscando o padrão ouro»*)

As rotas **A** e **B** da [pesquisa 04 §6](04_pesquisa_ossos_sobre_desenho_vetorial.md), juntas
(`ph2d_skeleton_live::skin_desenho`). ⭐ **A gaveta por forma presa** deriva a fonte lida e o
`IndiceDoCampo` UMA vez por FONTE (eram por quadro) e devolve o quadro anterior quando pose, bind e
leis não mudaram. ⭐ **O caminho da cena fica com os nós do artista** (modo Node, ponto novo,
`Release`); o que se vê é o bake (`assa_a_pele`) na `LiveGeometry`, com o estilo vivo, cedendo a
qualquer outro produtor. **Medido na barra da cena, 8 nós, S:** desvio máx ao padrão-ouro
`0,253 / 0,496 / 0,460 → 0,0018 / 0,0033 / 0,0043` a `30° / 60° / 90°` (`108–149×`); quinas vivas
`0,482 → 0,0048` (o bake percorre a fonte JÁ arredondada, com a tabela da mesma porta do `Bind`).
Custo `~0,2–0,3 ms` por forma **só nos quadros em que os ossos mexem** (load alto na medição — a
refazer calma). Bónus: o indicador do pincel de pesos lê a fonte da gaveta, `433 → 17 µs`.
⛔ **Fica de fora, declarado:** forma com EFEITOS no caminho vivo (o efeito corre sobre os nós) e
bind anterior a 2026-09-20 com quinas vivas (sem campo para a tabela). ⏳ **ABERTO:** o SMOKE do dono
(`PH2D_VEC_BONE_SMOKE=1`, e `PH2D_SKIN_DESENHO=0` para comparar) · a dobra do mapa no cotovelo
(`det J`, §1.3 da pesquisa) **continua** — ela é do padrão-ouro, e é a rota C/D · o `[bone] N NÓ(S)
caem FORA` passa a soar também quando a gaveta prepara uma fonte com quinas vivas.

⭐⭐ **O 1.º smoke do dono (2026-09-29): *«o melhor resultado até agora»*, com DUAS observações.**
**(1) *«uma linha anómala no stroke, atravessando a forma»* — um ESPETO, e a causa é do `kurbo`
0.13:** o `fit_to_cubic` mede o erro só no sentido FONTE → CÚBICA (raios normais), e só liga o
comprimento de arco numa fonte *«picante»*; num troço QUASE RECTO ele aceita uma cúbica que sai ao
longo da própria recta e volta (alça a `9,5` numa corda de `0,94`, C a `60°` → desvio `1,41`; sem a
cura a varredura acha `13,76` a C `40°`). ⇒ o `ajusta` da `ph2d_vec_skin::curva_segundo_corpo` é a
mesma recursão com a metade que falta — CÚBICA → FONTE a `≤ 2 × tolerância` — e a Hermite do
próprio bake no fundo. Gates: `o_ajuste_nunca_sai_da_fonte` (as `65` amostras EXACTAS do report,
com o controlo do `kurbo` sozinho a espetar `2,38`) e `o_desenho_fiel_nao_espeta_em_dobra_nenhuma`
(S e C, `30°`–`150°`). **(2) *«o osso do meio provoca ondulações discretas, que diferem da
deformação de imagens»* — a onda de fundo é da LEI e está nas duas** (a barra da cena como IMAGEM e
como FORMA, mesmos ossos, sobrepõem-se); o resto (`~1` pixel de arte, p50 `0,012`) é a resolução
dos pesos — `1 200` triângulos na forma contra `3 000` na imagem. ⛔ **Igualá-la foi CONSTRUÍDO,
MEDIDO e RECUSADO:** p50 `0,0120 → 0,0052`, mas pesos mais finos tornam a transição entre ossos mais
aguda e o **vinco de dentro do cotovelo passa a nascer a `80°` em vez de `93°`** (tabela no
`ALVO_DE_TRIANGULOS` da `ph2d_vec_skin::pesos`; e `12 000` não converge para a imagem, que também é
uma discretização). ⇒ a concordância fina passa pela cura do vinco (rotas C/D). Gate de REGRESSÃO
`a_forma_e_a_imagem_presas_aos_mesmos_ossos_dobram_igual` (p50 `< 0,015`). ⛔ A leitura `C¹` também
foi medida e **não** é a cura desta onda (move a distância à imagem `< 8 %` e custa `2,3×` o bake).

### F20 — ✅ **O GIZMO DO ENVELOPE SÓ EXISTE ONDE ELE MANDA, e agora POR OSSO** (report do dono, 2026-09-18)

*«O gizmo do envelope fica sempre visível mesmo quando não é usado?»* — **sim, ficava.** A F17 curou
o CAMPO do painel e deixou a **mancha** e a **alça** no canvas.

⭐⭐ **A lei entra na [`influence_region`] e não em quem desenha, porque essa porta tem DOIS
consumidores** — o desenho da mancha e o **hit-test da alça**. *Curar só o pintor deixaria o artista
a arrastar uma alça invisível, que é pior do que a mancha a mais.*

⛔⛔⛔ **E a pergunta passou de CENA para OSSO, porque uma premissa MINHA caiu.** Eu escrevi que *«o
`SkinBind` guarda a malha e os pesos, **não** a que ossos ficou preso»* — e ele guarda
(`Tendon::bone`, um `StableId`). Com a pergunta larga, numa cena **mista** a mancha acendia em
**todos** os ossos — e foi **a cena que o dono pediu** que expôs isso, antes de ela existir.

⭐⭐⭐ **E a medição que explica o resto do report** (*«não vi em nenhum dos casos o envelope fazer
diferença na deformação»*): com **UM** osso o envelope é **INERTE** — os pesos renormalizam e o
único osso leva sempre a fatia inteira (`[1.0]` a `0,3` **e** a `4,0`). Com **três**, ele manda
(`[1, 0, 0]` → `[0,42, 0,58, 0]`). *O alcance só decide quando DOIS ossos disputam o mesmo ponto.*
⚠️ ⇒ a porta é **necessária e não suficiente**: ela esconde o caso claro e mostra o resto, que é o
lado conservador, e o limite está escrito nela.

⛔ **A lei por CENA foi APAGADA, não guardada** — ficou sem chamador no instante em que a por osso
nasceu, e *uma lei viva que nenhum gesto consulta é uma lei órfã*.

⛔⛔ **E um gate de OUTRO assunto reprovou, com razão:** o `when_two_handles_overlap_the_nearer_one_wins`
construía a sobreposição a partir da região, e a fixtura dele **deixou de conter o fenómeno** quando
a lei mudou. *A cura é da fixtura, nunca da lei* — ela ganhou uma forma vectorial presa ao osso.

⛔ Tecto de LOC (`707` contra `700`) curado por **CORTE**: a mancha mudou de **ficheiro e não de
endereço** (`pub use`), a mesma lei que o `bend_live` já aplica no mesmo sítio.

Mutação **4 de 4** a sangrar; portão `15 101` verdes.

✅ **A CENA DEDICADA EXISTE — e ela REFUTOU a lei que esta secção acabara de shipar.** Ver **F21**.

### F21 — ⭐⭐⭐ **O ENVELOPE MANDA ONDE O PADRÃO-OURO NÃO RESOLVEU — a MÍDIA nunca foi a pergunta**
(a cena que o dono pediu, 2026-09-19)

⛔⛔⛔ **A minha resposta ao dono estava ERRADA, e a F20 shipou a lei errada por cima dela.** Eu
disse-lhe que *«numa forma vectorial o envelope manda como sempre»*, com o argumento — escrito no
doc da porta — de que *«o padrão-ouro precisa de uma malha do domínio, e uma Bézier não tem uma»*.
⚠️ **Essa premissa expirou em 2026-09-15**, quando o `ph2d_vec_skin::pesos::pesos_do_caminho` passou
a construir a malha do **INTERIOR** de um contorno fechado e a resolver os mesmos BBW. *Quem move o
número que tornava algo inalcançável tem de reconferir a nota* (§0.0) — e ninguém reconferiu.

**Medido pela porta do produto** (`sonda_do_envelope_no_vector_tests`, `ph2d-skeleton-live`),
variando o `strength` do osso do meio de `0,1` a `8,0` — uma faixa de **`80 ×`**:

| forma | fechada? | amplitude da deformação |
|---|---|---:|
| `Rectangle` · `Ellipse` · `Star` · `Polygon` · `Segment` · `Pie` | fechada | **`0,000000`** |
| `Line` | ABERTA | `2,03` |
| `Arc` | ABERTA | `4,25` |
| `Spiral` | ABERTA | `2,05` |

⇒ **o dono tinha mais razão do que a minha resposta lhe deu:** o envelope é inerte em **toda** forma
preenchida e em **toda** imagem que resolve. Ele manda num sítio só — onde a tabela de pesos do bind
está **VAZIA**, porque um caminho **ABERTO** não tem interior, logo não tem domínio para a energia.

⭐⭐ **A lei passou a perguntar ao BIND** (`caiu_na_lei_derivada`): *este bind guarda a tabela do
padrão-ouro?* As duas mídias respondem pela mesma porta — uma [`SkinnedMesh`] e um [`SkinnedPath`]
guardam a MESMA coisa —, e a mídia entra só para **escolher o descodificador**. ⛔ Uma `source` que
nem descodifica responde **não**: ali o quadro pula a pele, e acender a mancha seria prometer um
efeito que não existe.

⭐⭐⭐ **E a CENA é `PH2D_VEC_BONE_SMOKE=2`** — três fileiras, a mesma corrente de três ossos, a mesma
dobra; só muda o alcance do osso do meio:

| fileira | o que é | o envelope |
|---|---|---|
| `Corda (alcance 1)` | um traço ABERTO, alcance de fábrica | VIVO — mancha e alça |
| `Corda (alcance 4)` | o MESMO traço, alcance `4` no osso do meio | VIVO — e a corda acaba **noutro sítio** |
| `Barra preenchida` | o MESMO arco, fechado pela corda | INERTE — sem mancha, sem alça |

⚠️ **A env `PH2D_VEC_BONE_SMOKE` era de PRESENÇA e passou a ter níveis** (`NIVEIS = 2`): ilegível ou
ausente ⇒ `1`, a cena que o dono já aprovou.

⛔⛔ **A FOTO apanhou QUATRO defeitos que os gates não podiam ver** (`fotografa_cena.sh`, com um
`HOME` temporário sobre uma CÓPIA do `~/.ph2d` do dono): a terceira fileira **cortada** pela borda
de baixo (a arrumação dele abre a timeline, que come um terço da altura ⇒ a cena passou a
**fechá-la e só depois pedir o *Frame All***, nesta ordem) · as duas cordas desenhadas como um **fio
fino** (um caminho aberto não se vê pelo preenchimento ⇒ traço grosso) · o controlo como uma **barra
recta com um vinco** (a deformação vectorial corre nos PONTOS DE CONTROLO, e um `RoundRect` tem
oito ⇒ ele passou a ser o **mesmo arco fechado pela corda**, que tem os mesmos pontos) · e o
preenchimento de um arco aberto a desenhar **a corda da corda**.

⛔⛔ **E o `when_two_handles_overlap_the_nearer_one_wins` reprovou PELA SEGUNDA VEZ, pela mesma
forma:** a fixtura dele perde o fenómeno sempre que esta lei muda. ⇒ ela passou a vir de uma PORTA
que nomeia a condição (`test_support::pele_na_lei_derivada`), em vez de uma `SkinBind` montada à
mão. *Uma fixtura montada à mão fica abaixo da lei que se está a medir.*

⛔ **Uma mutação SOBREVIVEU e mudou o desenho:** trocar o guarda da ponte (`if nivel() == 2`) por
`if false` deixava **tudo verde** — *um gate de texto afirma que o código EXISTE, nunca que ele
CORRE*. ⇒ o guarda saiu: a ponte passou a ter **uma chamada incondicional** (`prologo_do_nivel(n)`)
e a inércia do `=1` virou uma lei PURA, medida pelos dois lados. O que sobra por medir — a ponte
CORRER — fica **dívida nomeada**, com um gate que reprova se alguém repuser o guarda.

⚠️ **E o número da dobra é MEDIDO:** a `40°` por junta o alcance `1 → 4` move a corda **`18,4 %`** do
comprimento dela (a `15°` são `8,9 %`; a `60°`, `27,7 %` — e aí o bloco deixa de caber no ecrã).

Mutação **10 de 10** a sangrar; portão `15 110`, com o único ✗ a ser o
`an_abandoned_march_returns_nothing_and_returns_fast` — membro **confirmado** da família de flakes de
fan-out (3 de 3 verde sozinho a `load 18,36`, zero linhas do diff naquela crate).

⏳ **ABERTO, e é decisão do dono:** com o envelope inerte em toda arte preenchida, o `Strength`
serve **um** caso — um traço aberto preso a ossos (uma corda, um cabelo, um cabo). *Manter o
controlo escondido por osso é o que shipa; tirá-lo do produto é a outra saída, e é dele.*

### F22 — ⭐⭐⭐ **A ESCOLHA: por que lei CADA DESENHO se deforma** (ordem do dono, 2026-09-19)

Ele perguntou, depois de aprovar a cena da F21: *«como se usa os dois modos? como se escolhe se os
envelopes vão ou não influenciar?»* — e a resposta honesta era **não se escolhe**.

⛔⛔⛔ **O app decidia, e decidia pelo DESENHO:** uma forma com interior (ou uma imagem que resolve)
ia para o padrão-ouro e o alcance ficava inerte; um traço ABERTO caía na lei euclidiana. *Qual lei
deforma o personagem é uma decisão de RIG, e ela estava escondida dentro de uma decisão de DESENHO.*
⚠️ Não havia interruptor nenhum — varrido: o único candidato (`PH2D_SKIN_WEIGHTS=linear`) troca como
os pesos guardados são **interpolados** ao refinar, não qual lei os **produz**.

⭐⭐ **E a capacidade já existia inteira** — medido ANTES de escrever uma linha: com a tabela de pesos
apagada, uma forma FECHADA corre na lei do envelope e move **exactamente** o mesmo que o traço
aberto (`6,4368` contra `6,4368` sobre a mesma curva; um rectângulo move `6,9835`). *O que faltava
não era motor, era o botão.* ⇒ ordem dele: **«construa. por desenho»**.

⭐⭐⭐ **A decisão de desenho que é a wave inteira: a escolha diz se o quadro LÊ a tabela, nunca se
ele a CALCULA.** O padrão-ouro custa dezenas de milissegundos a resolver e fica guardado no bind; se
a escolha mandasse no cálculo, voltar atrás obrigaria a re-resolver e o artista veria a ferramenta
engasgar ao alternar. Assim ela é **viva** — troca-se no quadro seguinte, nos dois sentidos, e a
volta é **exacta ao bit** porque a tabela nunca é tocada.

⇒ `SkinBind::law: SkinLaw` (`Auto` | `Envelope`) e **uma porta** (`SkinBind::pesos_do_quadro`) com
**TRÊS leitores**: o recook de uma forma, o desenho de uma imagem presa, e a pergunta *«o envelope
manda neste osso?»* que acende a mancha e a alça. ⛔ Escrita em três sítios, a mancha apareceria onde
o alcance não governa nada — que é, à letra, o report de 2026-09-18.

⚠️ **`PROJECT_SCHEMA` +1 — conte o DELTA** (`144 → 145`). Campo novo numa struct já gravada ⇒ regra
dos degraus 109/110; ⚠️ **a tripla NÃO vê este degrau** (a SÉTIMA vez). ⛔ Os três registos de
componente **não se mexem**: não há tipo novo.

**Na tela:** a fileira **`Deform By`** (`Artwork` | `Bone Reach`), ao lado do *Release* — ⚠️ pintada
para as **duas mídias** (ao contrário do *Expand*) e **só quando há algo preso escolhido**: *sem pele
não há lei de pele, e um selector sem sujeito é a classe de controlo morto do §5.0*.

⛔⛔⛔ **E a FOTO apanhou DOIS defeitos que os gates não podiam ver.** (1) O painel **Bones nasce
fechado** e só se abre sozinho quando um OSSO é escolhido — e o passo do roteiro manda escolher um
**DESENHO**: *um passo que nomeia uma linha de painel afirma que ela está lá, e o dono aprova o smoke
com o passo impossível dentro* ⇒ a cena passou a abri-lo no prólogo, **antes** do *Frame All* (ele é
uma coluna lateral, e abri-lo depois mudaria a área que o enquadramento mediu).

⭐⭐⭐ **(2) E o segundo era a LENTE DO PAINEL mais ESTREITA que o sujeito — a MESMA forma do report de
2026-09-18, e o defeito era MUDO.** Clicar numa linha da **Hierarquia** escreve na selecção do
**GIZMO** (`hero.gizmo.replace_selection`), nunca na lista de caminhos do pen — e o `Skinned` do
painel lia `vector` só do pen e `imagem` só do gizmo. ⇒ uma **forma vectorial** escolhida na
Hierarquia lia `{vector: false, imagem: false}`, e o *Expand*, o *Release* **e** a fileira nova **nem
chegavam a ser pintados**. *O artista não vê um botão morto: vê a ausência de um botão.* ⇒ as duas
metades passam pela porta da família (`skin_law::escolhidas`), a mesma que o dreno do chip usa.

⚠️ **A recusa é PRÓPRIA e não a do vizinho:** `RecusaDoOsso::NadaAQuemMudarALei` (a quarta) — ⛔
reaproveitar a `NadaASoltar` diria *«nada a soltar»* a quem carregou noutro botão. ⚠️ E ela **não**
cobre *«já estava nessa lei»*: escrever a lei que já lá está não é um acontecimento, e queixar-se
disso é o ruído que o artista aprende a ignorar.

Cena **`PH2D_VEC_BONE_SMOKE=2`**, passos (3) a (5). Mutação **8 de 8** a sangrar.

⏳ **ABERTO:** a escolha não tem gesto de canvas (só o painel) · e com N desenhos escolhidos em leis
diferentes o chip acende por `any` — a escolha está **declarada** no doc da porta, e mostrar a
divergência é mais honesto do que mostrar a maioria, mas ela é decisão de produto.

### F23 — ⭐⭐⭐ **A POSE DE REPOUSO, e a cura do *Reset Transform*** (auditoria contra o oráculo, 2026-09-19)

O dono mandou auditar o sistema de ossos contra o Godot 4.7.2 (MIT, corrido sem interface) e fazer o
que faltasse, menos o movimento secundário. A auditoria devolveu **um defeito vivo** antes das
ausências, e este é ele.

⛔⛔⛔ **MEDIDO no caminho do produto** (`ph2d_skeleton_live::sonda_do_reset_na_hierarquia_tests`): a
tabela do menu de contexto da Hierarquia é **PLANA** — ela não sabe o que a linha é —, e sobre um
osso `*t = Transform::IDENTITY` movia a arte presa **26,484841 unidades num desenho de 60 (44 %)**,
com uma mensagem **VERDE** a dizer que correra bem. *Num osso a direcção mora na `rotation` e a
posição na `translation`: «repor a transformação» de um osso não é a identidade, é o REPOUSO dele* —
que não existia.

| lei | quanto a arte salta | fracção da forma |
|---|---|---|
| a identidade (o que o app tinha) | **26,484841** | **44 %** |
| o repouso (esta wave) | **0,000000** | nada |

⭐⭐ **`BoneRest` é um COMPONENTE e não um campo do `Bone`**, a forma do `BoneLimit`/`IkGoal` — e a
razão é que **a ausência é uma resposta**: um `Option` dentro do osso obrigaria todo `Bone::default()`
a escolher um valor, e o valor neutro de uma pose é exactamente a **identidade**, *o mesmo byte que é
o defeito*. Com um componente, um osso sem repouso **não o tem**, e o verbo recusa em voz alta.
⭐ De graça: blob-key própria ⇒ o `PROJECT_SCHEMA` **não se mexe** (precedente da `PhysicsJoint`/W3);
registo do esqueleto **6 → 7** e catálogo **6 → 7**.

⚠️ **Ele guarda os SEIS números em `f32` e não um `Transform`:** aquele viaja num invólucro
**versionado** (`TransformVersioned`), e aninhá-lo aqui poria os bytes **fora** dele — um campo novo
lá leria todo repouso gravado errado, em silêncio.

⇒ uma porta (`pose_de_repouso`) com `guardar` · `repor` · e o veredito
`repor_transformacao -> Reposicao { NaoEOsso | SemRepouso | Reposta { ossos } }`. **Três** respostas
e não duas: um osso **sem** repouso guardado não cai de volta na identidade — era por aí que o
defeito voltaria para todo rig anterior a esta wave.

⚠️ **O sujeito é o osso ESCOLHIDO e a descendência dele**, e é estritamente mais expressivo: escolher
a raiz repõe o boneco todo, escolher o antebraço repõe o antebraço e a mão. ⛔ Uma lei que subisse à
raiz sozinha tornaria *«repor só este braço»* inexprimível.

**Na tela:** *Rest Pose* e *Set Rest Pose*, **antes** dos números do osso (o gesto mais frequente do
rig não fica no fim de uma lista que rola), e o *Reset Transform* da Hierarquia passa a perguntar à
mesma porta.

⚠️ **O que a construção refutou:** a `esqueletos::ossos_desde` devolve um **conjunto** determinístico
e **não** uma ordem hierárquica (ela ordena por `to_bits`, que aqui sai ao contrário da criação) — a
1.ª redacção dos gates presumiu-a e o gate da sub-árvore reprovou a acusar a LEI de mexer no pai
quando quem estava trocado era a fixtura. E a régua textual do gate da shell leu o **doc-comment que
EXPLICA a cura** (a armadilha que a Fase B da física já pagou por escrito) ⇒ ela passa a deitar fora
a prosa antes de medir.

Mutação **9 de 9** a sangrar. Tecto de função curado por **CORTE** (`hierarchy_reset`, irmão do
`hierarchy_delete`), nunca por uma entrada nova no `FN_OVERAGE_OK`.

### F24 — ⭐⭐⭐ **UM OSSO QUE APONTA PARA UM ALVO (*Look At*) — e o motor já existia** (2026-09-19)

A mesma auditoria nomeou o `SkeletonModification2DLookAt` como ausência nossa. ⭐⭐⭐ **A §5.0 correu
antes da primeira linha e disse NÃO:** a lei do alcance tem, escrito no corpo dela, um braço para uma
corrente de **um** osso (*«um osso só: aponta, e o comprimento manda»*), e medido pelo caminho do
produto (`goal::add` + `solve`) ele aponta com erro **`0,000000°`** em cinco direcções. *O que
faltava era o NOME, o desvio, e esconder o que ali não faz nada.*

⚠️ **O controlo que impede a conclusão de ser fabricada:** num esqueleto de um osso só a corrente
resolvida é sempre `1`, logo tudo aponta — a metade negativa corre num **braço**, onde a corrente de
`2` resolve o par pela lei dos cossenos e o ombro vai parar a outro sítio.

⭐⭐ **E a lente é MEDIDA, não escolhida.** Com a corrente resolvida em UM:

| knob | move o osso |
|---|---|
| *IK Mix* | **sim** (é o único que continua a mandar) |
| *IK Softness* | **zero** — a lei de um osso põe a ponta a `comprimento` na direcção |
| *IK Bend* | **zero** — não há cotovelo, logo não há lado |

⇒ o painel **esconde** os dois inertes e **pinta** o desvio. ⛔ Pintá-los ali seria a classe de
controlo morto que o `CLAUDE.md` §5.0 nomeia.

⇒ `IkGoal::offset` (o `additional_rotation` da referência) + o verbo **`Look At`**, que é o `add` com
a corrente em `1` — ⛔ ele **delega** no irmão e não repete o nascimento (o alvo, a marca `IkTarget`,
a semente do `StableId` e a captura do lado vivem lá).

⛔⛔ **O desvio só é LIDO quando a âncora APONTA**, e a cerca é a wave inteira: somá-lo a uma corrente
que **alcança** quebraria o alcance que ela acabou de resolver — a mão deixaria de tocar aquilo que a
restrição existe para tocar. A porta é `goal::aponta` (a corrente **resolvida**, nunca o número
escrito no campo), com **três** leitores: o solver, o espelho do painel e o verbo.

⚠️ **E ele entra ANTES da mistura e do limite**, a ordem que os dois já declaram: somá-lo depois faria
o `Mix = 0` deixar de devolver a pose autorada — o artista desligaria a restrição e o osso ficaria
rodado, sem nada que o explicasse.

⚠️ **`PROJECT_SCHEMA` +1 — conte o DELTA** (`145 → 146`). Campo novo numa struct já gravada ⇒ regra
dos degraus 109/110; ⚠️ **a tripla NÃO vê este degrau** (a OITAVA vez). ⛔ Os três registos de
componente **não se mexem**: não há tipo novo.

**Na tela:** o botão **`Look At`** ao lado do *Add IK* (as duas portas de entrada, e a diferença é o
que a restrição FAZ), e o campo **`Aim Offset`** em graus, depois do `IK Chain`.

Mutação **13 de 13** a sangrar.

⏳ **ABERTO:** o apontar não tem gesto de canvas (só o painel) · e o desvio é um número, não uma alça
— arrastar o olhar no canvas seria outro gesto, e é decisão de produto.

### F25 — ⭐⭐⭐ **ESPELHAR UM RAMO — o lado esquerdo construído a partir do direito** (2026-09-19)

O terceiro item da auditoria. ⭐⭐ **A lei é uma CONJUGAÇÃO e calcula-se à mão, sem uma única
constante escolhida.** Seja `M` a reflexão do mundo na vertical `x = c` e `G` a que troca o sinal do
`y` **dentro do referencial de um osso**. O referencial que leva a cabeça a `M(cabeça)`, a ponta a
`M(ponta)` **e** repõe o sinal do determinante é `W' = M ∘ W ∘ G` — e daí sai tudo:

| o que | como espelha | porquê |
|---|---|---|
| filho (pose local) | `translação.y := −y` · `rotação := −r` · skews `:= −` | `T' = G ∘ T ∘ G` |
| raiz do ramo | derivada da cabeça e da ponta **reflectidas**, no referencial do pai | `T' = P⁻¹MP ∘ T ∘ G` |
| `length` | **igual** | o `G` fixa o eixo `+X`, e o comprimento vive nele |
| `curve` `y` | **negado** | o arco é um desvio em `y` |
| `curve` `x` | **igual** | ele mede-se **ao longo** do eixo |
| `BoneLimit` | **`{ −max, −min }`** | sob `r ↦ −r` a faixa inverte **e troca de ponta** |

⛔ **Só negar o limite deixaria `min > max`, e a lei trava a junta no CENTRO do que estiver escrito**
— o cotovelo espelhado ficaria preso a meio caminho, sem nada na tela que o explicasse.

⚠️ **O EIXO é DERIVADO** (§0.0): a vertical que passa pela origem do osso **RAIZ** do esqueleto —
num personagem, o quadril; é o mesmo `X = 0` da armadura que o Blender espelha. ⛔ Um campo com um
número seria uma terceira coisa a manter coerente com a pose.

⭐⭐ **A cópia passa pela CÓPIA PROFUNDA da casa**, e é isso que faz o ramo novo carregar o que esta
shell não conhece (o limite, a curvatura, o repouso, e o que vier): ela copia o que o **registo**
descreve. ⛔ Uma cópia campo a campo esqueceria o primeiro componente novo, em silêncio.

⛔⛔⛔ **E a cópia profunda NÃO REMAPEIA REFERÊNCIA NENHUMA — o doc dela di-lo por escrito.** O
`Bone::curve_tip` nomeia um **filho por identidade**, logo a cópia ficaria a apontar para o filho do
**ORIGINAL**: o ramo espelhado arquearia a seguir a um osso do outro lado do corpo, e a referência
**resolve**, logo nada acusaria. ⇒ ele é remapeado pelo mapa `StableId → StableId` que a cópia
devolve. ⚠️ **Um id de FORA do ramo fica intocado** — ali a referência do original continua a ser a
resposta certa.

⛔ **O que NÃO viaja, e é decisão declarada:** a **âncora de IK** e o **osso inteligente**. Os dois
nomeiam OUTROS objectos da cena por identidade, e copiá-los daria duas correntes a puxar o **mesmo
losango** — o braço espelhado seguiria a mão do original. *Mirrorar uma referência a um objecto é
uma segunda decisão que este verbo não pode tomar sozinho.* O artista carrega em *Add IK* / *Look At*
no ramo novo.

⚠️ **O NOME troca de lado por uma TABELA e não por um `replace` cego** — trocar todo `L` por `R`
renomearia `"Leg"` para `"Reg"`. O que se troca é um **marcador**: um sufixo (`.L`, `_Right`) ou uma
**palavra inteira** (`Left Arm`). ⚠️ E o resultado passa **sempre** pela porta da unicidade: a
referência durável entre objectos nesta casa é o NOME, e dois ossos com o mesmo seriam o mesmo
sujeito para a timeline. ⭐ Um nome sem lado (`"Bone 7"`) devolve `None` — *inventar-lhe um lado seria
escrever uma decisão do artista*.

⭐⭐⭐ **A prova mais dura é a INVOLUÇÃO:** espelhar duas vezes devolve a geometria original (barra
`1e-4`, derivada do `f32` da pose). Mais: cada osso da cópia vai de `M(cabeça)` a `M(ponta)` — medido
em **MUNDO** e não nos campos locais, *senão a régua mediria a implementação e ficaria verde sobre
uma cópia que aponta ao contrário* — e o **original não se mexe**.

**Na tela:** o botão **`Mirror Branch`**, terceiro do trio que age sobre *este osso e a descendência
dele* (os outros dois são o par do repouso). Zero schema, zero registo novo.

Mutação **12 de 12** a sangrar.

⏳ **ABERTO:** o espelho não tem gesto de canvas (só o painel) · e a arte presa não é espelhada com
os ossos — o ramo novo nasce sem pele, e prendê-la é o gesto que já existe (*Bind*).

### F36 — ⭐⭐⭐ **O TRAÇO DE UMA FORMA PRESA SOBREVIVE AO QUADRO** (report do dono, 2026-09-19, *«num vector linkado aos ossos não consigo mudar a espessura do stroke»*)

⛔⛔⛔ **A causa NÃO era o painel, e é isso que a torna instrutiva.** O valor chegava ao documento —
a fileira *Width* está sempre visível e o `vector_bridge` reestiliza a selecção pelo mesmo
`SetValue(VECTOR_WIDTH)` — e o **re-cozimento da pele devolvia-o no quadro seguinte**. Medido: o
artista põe `width = 0,2` e o recook lê `None`, sem um erro e sem um pixel de aviso. *Um controlo
que o produto desfaz no quadro seguinte lê-se exactamente como um controlo morto.*

⭐⭐ **O mecanismo é a fonte de cada re-cozimento.** A [`VecPath::replace_cooked`] responde *«o que
um re-cozimento produz»* para quem re-gera a forma a partir dos **PARÂMETROS** dela — o texto a cada
tecla, o objecto de texto, o envelope —, e ali o estilo **é** produto do cozimento. ⚠️ **A pele não
é desse tipo:** a fonte dela é uma **FOTOGRAFIA** tirada no instante do `Bind`, congelada em bytes
opacos, e o que ela re-gera por quadro é a **posição** de cada ponto.

⇒ **[`VecPath::replace_geometry`]**, a porta de *«re-gerei ONDE os pontos estão»*: escreve
`verts`/`closed`/`subpaths` e preserva **todo** o estilo. ⛔ **As duas portas destruturam a struct de
forma EXAUSTIVA de propósito** — um campo novo obriga a responder **as duas** perguntas (*é produto
de um re-cozimento?* e *é GEOMETRIA?*) no commit em que ele nasce.

⚠️ O `release(Keep::Source)` passa pela mesma porta: *o Release devolve o que o artista DESENHOU, e
a cor com que ele o pintou depois do `Bind` é dele.*

⛔⛔ **E o censo dos hosts que reescrevem `verts` teve de crescer no MESMO commit:** no instante da
troca de porta o detector **deixou de ver o `skin_live.rs`**, e o piso de população é o que
transforma isso num vermelho em vez de um silêncio. *A cura de um defeito pode apagar um host do
censo que o vigia* — o doc daquele gate já avisava, e esta é a quarta assinatura.

**Gates:** `replace_geometry_carries_the_points_and_leaves_the_style_alone` (as duas metades, com a
fixtura fora do neutro dos dois lados) · `o_traco_que_o_artista_poe_numa_forma_presa_sobrevive_ao_quadro`
(produto, com o controlo de que o quadro escreveu de facto a geometria). **Mutações: 3 de 3.**

### F35 — ⭐⭐⭐ **AS DUAS ALÇAS DE UM NÓ VOLTAM A RODAR JUNTAS** (report do dono, 2026-09-19, *«muitas irregularidades na deformação de vetores. Certamente um mau tratamento das alças dos handles»*)

⛔⛔⛔ **A F33 resolvia cada segmento SOZINHO.** As duas alças que se encontram num nó — a de saída do
segmento `k` e a de entrada do `k+1` — saíam de dois sistemas de mínimos quadrados que não se
conhecem ⇒ **deixavam de ser colineares**, e o nó que o artista desenhou LISO virava uma QUINA.
Medido na barra da cena, a mudança da tangente contra a lei ingénua:

| dobra | p50 | máx |
|---|---|---|
| `30°` | `2,58°` | `5,86°` |
| `60°` | `5,08°` | `13,00°` |
| `90°` | `7,30°` | `20,92°` |
| `120°` | `9,05°` | `28,62°` |

⭐⭐ **O CONTROLO é o que nomeia a causa:** a lei INGÉNUA mede `0,000°` em **todas** aquelas dobras,
porque as três metades de um vértice passam pelo MESMO afim e *um afim preserva colinearidade*.
⇒ *a quebra não vinha da pele: vinha do ajuste.* ⚠️ E a própria curva-ALVO quebra ali (`28,2°` a
`120°`, medido por diferença central), porque o peso é interpolado no **PARÂMETRO** e a derivada
dele salta em cada nó — *seguir fielmente um alvo com uma quina é desenhar a quina*.

> ⛔⛔⛔ **SUPERADO em 2026-09-20 (commit `a986e098f`, rebaseado): a `reconcilia` foi APAGADA.** Medida
> pela porta do produto, ELA era a serpentina (`9,3×`): o ajuste livre sozinho É o chão do modelo às três
> casas. Não a reconstrua — ver [handoff de 2026-09-24](handoffs/HANDOFF_INTEGRACAO_line_Vector_O_CAMPO_E_A_PESQUISA_2026-09-24.md) §3.

⭐⭐⭐ **A cura é um quarto passe** ([`ph2d_vec_skin::curva::reconcilia`]): cada alça tem um **EIXO** —
a direcção que o afim **daquele nó** dá à tangente da fonte —, o ajuste livre afastou-a dele por um
ângulo, e o passe faz as duas metades concordarem num ângulo só (a média pesada pelo COMPRIMENTO) e
roda cada uma para lá. ⇒ quebra **`0,000°`** em todas as dobras, e o desvio à curva verdadeira
**melhora ao mesmo tempo** (`0,03371 → 0,01900` a `90°`, contra `0,03371` da lei ingénua): *conciliar
a tangente não tira graus de liberdade ao ajuste — redistribui-os.*

⚠️ **Quatro decisões, cada uma com a medição:** rodar e nunca reescrever a alça a partir do eixo
(`|h|·versor(h)` não é `h` em vírgula flutuante, e a rotação de `0` é a identidade **ao bit**) · o
eixo vem do afim do NÓ e não da cúbica já deformada (aquele vector mistura **dois** nós quando a
alça é degenerada, e as duas pontas de uma aresta recta recebiam a mesma recta) · uma alça **sem
eixo** fica fora da média e fora da rotação (o nó ali é um CANTO) · e a **cascata** da tangente foi
construída e **removida** por nenhuma mutação a conseguir matar.

⛔ **RECUSA MEDIDA — o `smoothstep` no peso:** `lerp(ra, rb, 3t²−2t³)` corta a quebra do ALVO a meio
(`p50 9,05 → 3,98`) e **não** cura o máximo (`28,62 → 26,98`); com a conciliação por cima não muda a
quebra (já é zero) e **piora** o desvio (`0,03049 → 0,03459`).

⚠️⚠️ **PREMISSA MORTA, com a morte visível no diff:** o `com_a_lei_da_curva_o_ponto_novo_nao_move_nada`
dizia *«por construção»* e media `0,00 %`. Com a conciliação o resultado num segmento depende do
**vizinho**, e partir um segmento muda a vizinhança de um nó ⇒ o salto passa a **`0,0201 %`** da peça
(`0,008` unidades numa peça de `40 × 10`). A barra sai do vale entre duas medições: `0,0201 %` hoje
contra **`11,11 %`** com a compensação da F28 ligada.

⚠️ **E três réguas tiveram de ser corrigidas antes do algoritmo:** o ângulo de VIRAGEM sozinho não
serve (um rectângulo tem quinas autoradas e lê `90°` em repouso) ⇒ mede-se a **MUDANÇA** contra a lei
ingénua; um nó com alça degenerada devolve `None` e **não** um salto, senão o emparelhamento com o
outro estado do caminho desalinha; e a fixtura tem de ser uma **ELIPSE** — num `RoundRect` o
arredondamento é um `corner_radius` **dentro do vértice**, logo a FONTE continua a ser um rectângulo
de alças degeneradas. *Uma forma que parece curva na tela pode ser recta na fonte.*

⛔ **DUAS cercas ficam DECLARADAS sem gate**, com o mecanismo escrito ao lado (a do vector nulo no
`versor` e a `soma.1 <= 0.0`): nas fixturas construídas para as provocar a alça colapsa junto com o
mapa e a `reconcilia` salta o nó antes de olhar para o eixo. *Ficam porque o modo de falha delas é a
forma do artista DESAPARECER.*

**Mutações: 12 de 12.** Dois cortes de LOC por responsabilidade, nenhuma entrada nova no
`FILE_OVERAGE_OK`.

### F34 — ⭐⭐⭐ **O ENTALHE DO COTOVELO: a arte roda em torno da JUNTA** (ordem do dono, 2026-09-19, *«vamos curar o entalhe no lado de dentro do cotovelo»*)

⛔⛔⛔ **A mistura linear interpola POSIÇÕES, e isso dá a CORDA do arco.** Um ponto a meio caminho
entre dois ossos que divergem `θ` é puxado para `cos(θ/2)` da distância à junta — a `120°` isso é
**metade**. É o *«candy-wrapper»* de toda a literatura, e o entalhe que o dono fotografou.

⚠️ **A RÉGUA é o PESCOÇO — o sítio mais estreito da forma —, e as outras três foram medidas e
recusadas:** a **área** mal se mexe (`90,6 %` a `150°`) e não distingue um entalhe de um
encolhimento; a **viragem** satura (`33°` a `90°` e a `150°`); os **cruzamentos** do contorno só
aparecem quando já é tarde (`0` até `90°`). ⛔⛔ E a régua teve de ser limpa **duas** vezes: a
`RoundRect` tem dois segmentos de comprimento ZERO, e as amostras repetidas fabricavam `64`
cruzamentos **em repouso** e um pescoço de `0,0172` numa barra de espessura `1`.

⭐⭐ **A LEI: em 2D o centro de rotação NÃO se estima — ele é SABIDO.** A literatura estima-o
(*Optimized Centers of Rotation*, Le & Hodgins 2016: uma média das posições de repouso pesada pela
SEMELHANÇA entre vectores de peso, com um `σ` a afinar). ⛔ Aqui não é preciso: **um esqueleto 2D é
uma árvore de segmentos que PARTILHAM pontas**, e dois ossos que disputam um ponto partilham uma
junta ⇒ o centro é a média das juntas de cada par, pesada pelo **produto** dos pesos do par.

| dobra | mistura linear | estimador (`σ² = 5e-4`) | **a JUNTA** |
|---|---|---|---|
| `60°` | `0,4882` | `0,9496` | **`0,9432`** |
| `90°` | `0,3343` | `0,8615` | **`0,8659`** |
| `120°` | `0,1643` | `0,3317` | **`0,3763`** |

⇒ *a junta iguala ou bate o estimador, **sem um parâmetro e sem um byte guardado**.*

⭐ **Medido na barra da cena do dono, pela porta do produto:**

| dobra | área LINEAR | área RÍGIDA | pescoço LINEAR | pescoço RÍGIDO |
|---|---|---|---|---|
| `30°` | `99,4 %` | **`100,0 %`** | `0,9655` | **`0,9833`** |
| `60°` | `97,7 %` | **`100,0 %`** | `0,8646` | **`0,9487`** |
| `90°` | `95,4 %` | **`100,0 %`** | `0,7045` | **`0,8927`** |
| `120°` | `92,8 %` | **`100,0 %`** | `0,1746` | **`0,3949`** |

⛔ **LIMITAÇÃO DECLARADA:** a `150°` o pescoço continua a fechar (`0,0017`) — ali a face de dentro
dobra-se sobre si mesma qualquer que seja a lei. *Está no gate, para ninguém ler a tabela como
«curado em todo o percurso».*

⭐ **Não há descontinuidade onde um osso manda sozinho:** ali a transformação é RÍGIDA, e uma rotação
rígida leva `p` ao mesmo sítio **qualquer que seja o centro** (`R(θ)(p−c) + M(c) = M(p)`). ⇒ o centro
deixa de importar exactamente onde ele deixa de existir.

⛔⛔ **E o `dual quaternion` continua RECUSA MEDIDA** (`2,52 % → 4,47 %` de imagem dobrada). ⚠️ **Mas
o MECANISMO que a recusa escreveu está corrigido:** ela dizia *«o colapso vem do GRADIENTE dos
pesos»*, e a varredura da largura da transição diz o contrário — **alargar** a transição PIORA
(pescoço `0,2796 → 0,0092` a `120°`, área `59,4 % → 25,0 %`). *O colapso vem de a mistura de duas
POSIÇÕES ser a corda; o `log` resolve a rotação e deixa o centro ao acaso, esta lei escolhe o
centro.*

⛔⛔⛔ **TRÊS fixturas simétricas aprovaram, cada uma, uma lei que não distinguia nada:**
1. com a junta no MEIO da barra, o centróide calha nela e o estimador devolvia `(3,500 · 0,500)` para
   **todo** peso — `σ²` de `2e-5` a `100` dava o mesmo número;
2. com DOIS ossos há um par só, e o peso dele **cancela-se na normalização** ⇒ `wᵢ·wⱼ` e `wᵢ+wⱼ`
   eram indistinguíveis (mutação sobrevivente);
3. com os dois ossos a rodar em torno da mesma junta, ela **não se mexe** ⇒ `blend_linear(junta)`
   e `junta` eram o mesmo ponto (mutação sobrevivente).
⇒ os gates de hoje pedem três ossos, pesos assimétricos e a junta a VIAJAR.

⚠️ **A lei viaja como PARÂMETRO** (`aplica_corrigido_com` · `aplica_pela_curva_com` ·
`recook_com_mistura`), e a mistura antiga fica como [`Skin::blend_linear`] — ela é o **CONTROLO** de
todo gate que mede a cura, e é a translação que a lei nova usa para o centro.

⛔ **DUAS premissas morreram:** as barras do envelope (`1,0 · 2,0 · 1,0`) foram calibradas sobre a
mistura linear e a `Line` caiu para `0,848767` — *a lei nova preserva a forma, logo mexer no alcance
move-a MENOS: o número descer é a cura*; e o gate de paridade da PLACA deixou de poder afirmar
paridade (a CPU mudou de lei e o shader não). ⇒ ele passa a afirmar o que é verdade — *a placa
reproduz a `blend_linear`, e isso NÃO é o produto* —, com a **dívida** medida (`1,789e-1`) e
gateada, para reprovar no dia em que alguém ligar o caminho da placa.

Mutação **6 de 6** a sangrar. Zero schema, zero registo novo.

⏳ **ABERTO:** o caminho de GPU ficou com a lei antiga (dívida com gate) · e a `150°` a face de
dentro dobra-se, que é geometria e não lei.

### F33 — ⭐⭐⭐ **A DEFORMAÇÃO DEIXA DE SALTAR: o refit sai, entra a correcção das ALÇAS** (report do dono, 2026-09-19, com duas fotos)

*«Em determinado momento da deformação as alças sofrem uma mudança e o path muda repentinamente,
como se o handle mudasse de tipo. Tanto a curvatura interna como a externa foram repentinamente
modificadas.»*

⛔⛔⛔ **A causa era um BOOLEANO sobre uma grandeza contínua.** A F30 perguntava *«o desvio passa da
tolerância?»* e, se sim, **refazia o contorno inteiro** com a `kurbo::fit_to_bezpath`. Medido numa
dobra a passos de `0,01 rad`, ele não dá um salto: dá **CHATTER** — a decisão oscila entre quadros
**vizinhos** a partir de `1,44 rad` (`1,44 · 1,45 · 1,56 · 1,79 · 1,96 · 2,06 · 2,10 · 2,12 · 2,15 …`),
e cada oscilação vale **`0,038`–`0,050`** numa peça de espessura `1`. *O artista arrasta a âncora e a
forma pisca.* ⚠️ E como a decisão é por CONTORNO, uma oscilação troca a representação de **todos** os
segmentos — nós, alças e contagem —, que é a *«mudança de tipo de handle»* que ele viu.

⭐⭐ **A lei nova não tem decisão nenhuma para tomar.** A lei de hoje acerta nos NÓS por construção e
erra no INTERIOR de cada segmento — e o interior de uma cúbica é exactamente o que as suas duas
alças governam. ⇒ elas são ajustadas por **mínimos quadrados** contra a curva verdadeira, com as
pontas presas. A matriz do sistema `2×2` **só depende dos `t`**, logo é constante, e a solução é
**linear** na diferença amostrada: *é daí que vem a continuidade.*

⭐⭐⭐ **E o ajuste é da DIFERENÇA, não da curva** — `verdade(t) − ingénuo(t)`. Onde o mapa é afim
sobre o segmento (em repouso, e em toda aresta cujos dois nós têm o mesmo peso e que nenhuma mancha
toca) essa diferença é **exactamente zero**, o segundo membro é zero e as alças ficam
**byte-idênticas**. ⇒ o defeito que obrigou o limiar a existir — `binding_a_shape_moves_nothing` a
acusar `40/3` em repouso, a elevação `(⅓, ⅔)` de uma recta — **não pode acontecer aqui**. Os nós não
se mexem, o `kind` e o `corner_radius` sobrevivem, e nenhum vértice nasce ou morre.

| | antes (refit) | agora (alças) |
|---|---|---|
| pior passo contra o passo mediano, varrendo `0`..`2,5 rad` | **chatter** de `0,038`–`0,050` | **`1,00×`** |
| erro contra a curva verdadeira, numa dobra de `1,2 rad` | — | `11,29` → **`1,04`** (`10,9×`) |
| preço por forma (`--release`) | `0,163 ms` | **`0,001 ms`** (`163×`) |

⛔⛔ **A troca tem um PREÇO, e ele está medido:** o ajuste de duas alças **não segue uma feição mais
fina do que um segmento**. Numa barra de oito nós, uma mancha de raio `0,4` no meio de uma aresta de
`6` move **zero** (o refit movia `0,84`). ⚠️ *Mas ali nenhuma das duas leis chega perto da verdade* —
erro `0,71` com a correcção e `0,85` sem ela, numa barra de espessura `1` — e a cura daquele mundo é
a **subdivisão** (F32), não a lei.

⭐⭐⭐ **E na forma que o `Bind` produz a correcção não compra nada, medido:**

| a barra, na pose em S, contra a verdade | com a correcção | sem ela |
|---|---|---|
| GROSSA (`8` nós) | `0,7105` | `0,8535` |
| **do PRODUTO (`34` nós)** | **`0,0310`** | **`0,0302`** |

⇒ *quem faz o trabalho é a subdivisão; a correcção das alças é a rede para uma fonte grosseira* — um
ficheiro gravado antes de 19/09, ou o caminho `bind_com(.., false)`. Ela fica por isso, e porque é
contínua e custa `0,001 ms`.

⭐⭐ **E a F30 fechou-se por outro caminho: «entre os nós» deixou de existir.** Na barra do produto o
ponto do contorno **mais longe** de um nó está a **`0,3378`**, contra um pincel de fábrica de `0,40`
⇒ o dedo alcança sempre um nó, e as duas leis passam a dar o **mesmo** número (`0,416233`).

⛔ **Dois gates mudaram de mundo**, com a razão escrita: `um_arrasto_pelo_meio_da_barra_move_a_arte`
e `com_a_lei_da_curva_o_ponto_novo_nao_move_nada` passam a medir a barra do PRODUTO (`0,416233` e
`0,000000 %`); os irmãos que medem a ÂNCORA da mancha continuam a pedir a barra grossa pelo nome.

⚠️ **O parâmetro `tolerancia` SAIU** de `aplica_pela_curva` — não há o que tolerar quando não há
decisão —, e com ele o `ParamCurveFit`, o `fit_to_bezpath` e a `TOLERANCIA`.

⛔⛔ **DUAS linhas saíram por não serem lei, e as duas foram achadas por mutações SOBREVIVENTES:** o
guarda do determinante (a matriz é **constante**, logo aquele ramo é inalcançável) e uma cerca de
`NaN` (tudo o que chegasse assim já teria passado pela lei ingénua, que corre **antes** e escreve o
`NaN` no desenho — *uma cerca a jusante do estrago protege o quê?*). ⚠️ E a regra do ponto médio
`(i+½)/N` fica **declarada como escolha e não como lei**: a mutação para `i/N` sobrevive, porque a
amostra em `t = 0` tem `B₁ = B₂ = 0` e não entra no sistema.

⚠️ **E o gate que faltava era o da QUALIDADE do ajuste:** três mutações (o determinante, a solução
desacoplada, a alça de entrada por corrigir) sobreviveram a *«a arte mexe-se»* e a *«é contínua»* —
um ajuste mau satisfaz as duas. ⇒ `as_alcas_corrigidas_seguem_a_curva_verdadeira`.

Mutação **7 de 7** a sangrar (mais 3 declaradas como não-lei).

### F32 — ⭐⭐⭐ **A SUBDIVISÃO NASCE NO BIND: os pontos ficam à vista** (ordem do dono, 2026-09-19: *«sem saber onde os pontos estão não fica legal. Melhor criar a subdivisão visível logo na associação com os ossos»*)

> ⛔⛔⛔ **SUPERADO em 2026-09-20 por ordem do dono** (*«retire a criação automática de ponto no bind»*,
> commit `37b5e8715`): o `bind` já NÃO subdivide; a lei fica alcançável só por `bind_com(.., true)`
> (contrafactual e formas gravadas entre 19 e 20/09). Custo medido: `184×` de fidelidade na dobra forte —
> ver [handoff de 2026-09-24](handoffs/HANDOFF_INTEGRACAO_line_Vector_O_CAMPO_E_A_PESQUISA_2026-09-24.md) e
> [`04_pesquisa_ossos_sobre_desenho_vetorial.md`](04_pesquisa_ossos_sobre_desenho_vetorial.md).

É a lei que a **2.ª mídia já tinha** — uma imagem presa ganha no bind uma malha graduada pelas
articulações — agora também para uma forma vectorial. A barra da cena passa de **8** nós (os oito
nas duas pontas) para **34** ao longo dela.

⭐⭐ **O passo é DERIVADO e o `3` não foi escolhido.** Ele é o **osso mais curto a dividir por
[`DIVISOES_POR_OSSO`]**, e esse valor é o menor em que a lei dos **pontos de controlo** passa a
concordar com a lei da **CURVA** (a imagem verdadeira da pele) dentro da
[`ph2d_vec_skin::curva::TOLERANCIA`] que a casa já usa:

| ossos | osso | `K=1` | `K=2` | **`K=3`** |
|---|---|---|---|---|
| 2 | `3,200` | — | `0,0606` | **`0,0000`** |
| 3 | `2,133` | `0,2100` | `0,0496` | **`0,0000`** |
| 4 | `1,600` | `0,0699` | `0,0000` | **`0,0000`** |
| 6 | `1,067` | `0,0567` | `0,0000` | **`0,0000`** |

⇒ `K = 2` **falha** com dois e três ossos; `K = 3` é suficiente nas duas poses medidas (a ponta
girada `0,8 rad` e a barra inteira enrolada). ⚠️ *A escala fina de um campo de pesos é o comprimento
de um OSSO* — é por isso que o passo se mede contra ele e não contra o tamanho da forma.

⭐⭐⭐ **E o produto da wave é FIDELIDADE, não só pontos à vista.** A verdade é a mesma forma com o
passo `48×` mais fino:

| a barra, na pose em S da cena | nós | erro contra a verdade |
|---|---|---|
| GROSSA (o caminho de antes) | `8` | **`0,3767`** |
| do PRODUTO (`osso / 3`) | `34` | **`0,0142`** |

⇒ `38 %` da espessura da barra contra `1,4 %` — **`26×`**. ⚠️⚠️ **Quem achou isto foi uma
FOTOGRAFIA e não um gate:** ao refotografar a cena do smoke, a barra apareceu **dobrada** onde antes
estava quase recta, e a primeira leitura possível era *«a wave estragou a cena»*. *Ela estava a
passar a seguir os ossos.*

⭐⭐ **De graça: o desenho ficou mais BARATO.** O refit da lei da curva só corre onde a lei ingénua se
afasta, e com a forma subdividida ele deixa de correr — o recook da barra passou de **`732 µs`** (8
nós) para **`58 µs`** (34 nós) em `debug`.

⛔ **O tecto é o RELÓGIO DO RECOOK**, medido em `--release`: `4 094` vértices custam `3,65 %` de um
quadro e um décimo de quadro compra **~11 100**. ⚠️ **Medi-lo em `debug` daria um tecto `11×` mais
baixo** — o §0.0 outra vez. `VERTICES_MAX = 4096`, generoso de propósito: o que ele impede é o caso
degenerado (um osso minúsculo sobre uma forma enorme), e a barra da cena sai com `34`.

⛔⛔ **A QUINA VIVA é protegida, e o guarda mede a CAUSA e não a consequência.** O recuo de uma quina
é clampado a *metade da menor corda vizinha*, logo partir o segmento ao lado dela encolhe-o (medido:
`2,055e-2` sem guarda). A 1.ª redacção comparava o DESENHO inteiro antes e depois de cada corte e
**pendurou a suíte** — hoje compara o **recuo**, que é `O(1)`. ⛔ Escrever a condição à mão (*«não
cortes ao lado de um vértice com raio»*) recusaria todo corte num rectângulo arredondado.
⏳ **ABERTO e declarado:** uma forma com **EFEITOS** não é subdividida — a saída de um efeito é
função do contorno inteiro e esta wave não a mediu.

⛔⛔⛔ **A wave derrubou ONZE gates, e cada um é uma premissa que morreu — nenhum era uma barra
afrouxada:**
- **A régua dos VÉRTICES** (`pior_desvio`) emparelha vértice com vértice e leu **`37,5`** sobre uma
  forma que não se mexeu um pixel. ⇒ irmã nova `pior_desvio_do_desenho`, e os três gates cuja
  pergunta é sobre o DESENHO trocaram de régua. *É a lição da F30 um nível acima.*
- **O suporte finito** (`binding_to_the_whole_scene…`) afirmava *«peso 0 ⇒ os mesmos números»* e
  media-o pelo desenho a `1e-12`. Verdade sobre os ossos LONGE (medido: **exactamente `0`**), falsa
  sobre os PERTO — o domínio do solver é construído a partir dos anéis **e dos ossos**, logo um osso
  a `400` unidades muda a TRIANGULAÇÃO. Contrafactual: `0,000e0` sem subdivisão, `1,289e-2` com. ⇒ o
  gate passa a afirmar a **LEI** em vez do proxy.
- **A F28** (o ponto novo não faz o desenho saltar) mede um mundo GROSSEIRO que o produto já não
  produz ⇒ os gates dela pedem-no pelo parâmetro (`bind_com(.., false)`), que é o caminho de antes de
  19/09 **e o de um ficheiro gravado antes dele**. ⭐ E a dissolução está medida: o salto sem
  compensação vai de **`18,89 %`** (o número exacto que a F28 curava) para **`0,000000 %`**.
- **O indicador** afirmava que o osso do MEIO *«não possui nada nesta arte»* — e é isso que tornava
  o meu passo de smoke errado. Hoje possui, e o gate guarda o **contraste** com a barra grossa.
- **A F31** (a mancha pousa entre os nós) tem como sujeito a barra grossa: com a subdivisão o nó mais
  perto do meio passa de `3,04` para `0,50` contra um pincel de `0,40`. ⇒ `const GROSSA: bool =
  false`, *senão aqueles gates ficariam verdes por vácuo*.

⚠️ **A lei viaja como PARÂMETRO** (`bind_com`), nunca numa variável de ambiente — a lição do
`recook_com`: uma porta global lida dentro da lei é um canal entre testes.

✅ **E o aberto da F31 DISSOLVEU-SE aqui, medido:** *«a mancha pintada entre dois nós não tem
representação na tela»* deixou de ter sujeito — com os nós que o `Bind` põe, uma pincelada de fábrica
muda a cor de **`5` dos `34`** pontos, com a maior mudança a valer `0,1304`. ⚠️ *A cura não foi
desenhar a mancha: foi o sítio onde ela pousa passar a ter um ponto.* · uma forma com efeitos · e `Release` devolve a forma
**subdividida** (o mesmo desenho, mais pontos de controlo), que é consequência declarada.

### F31 — ⭐⭐⭐ **O PINCEL DE PESO ALCANÇA O MEIO DE UMA ARESTA** (report do dono, 2026-09-19: *«o que vc mandou fazer não funcionou»*)

⛔⛔⛔ **A F30 shipou uma LEI SEM GESTO, e o report tinha DUAS causas — a minha e a do produto.**

**(a) A minha.** O smoke que mandei mandava escolher **«Bone 14»** e pintar na **barra laranja**. A
cena tem **dois** esqueletos de três ossos — a barra vectorial (`Bone 1..3`) e o braço **PINTADO**
(`Bone 13..15`) — e o `Bone 14` é o do MEIO do segundo. *Seguir o passo à letra não podia funcionar.*
O dono disse-o melhor do que qualquer sonda: *«Bone 14 está ligado à imagem e não ao vetor. Bones 1,
2 e 3 estão ligados na barra laranja.»* ⇒ o roteiro passa a NOMEAR quem governa a barra, **derivado
do mundo** (os nomes são o índice da entidade: acrescentar uma peça à cena renumera tudo o que vem
depois), e a listar a cadeia **em ordem** — a [`esqueletos::ossos_desde`] ordena por `to_bits`, que
no bevy é a criação INVERTIDA, e a 1.ª frase saía *«Bone 3, Bone 2, Bone 1»*.

**(b) A do produto, e é a que importa.** O gate da F30 constrói a
[`CorreccaoDePeso`](../../crates/ph2d-skeleton-ecs/src/skin_bind.rs) **à mão**, com `centro` no meio
de uma aresta — e **nada no repo perguntava se o PINCEL consegue produzir esse centro**. Ele não
conseguia: a mancha era ancorada no **NÓ mais perto** e o gesto recusava (`ForaDaArte`) quando o
dedo estava mais longe do que o raio do pincel.

| o dedo, na barra do smoke | nó mais perto | raio de fábrica | veredito |
|---|---|---|---|
| no MEIO da barra | **`3,041`** | `0,40` | **`ForaDaArte`** |
| idem, com o raio a `400 px` | `3,041` | `4,00` | `Pintada`, **com o centro na QUINA** `(−8,0 · 2,0)` |

⇒ *é o terceiro elo do `CLAUDE.md` §5.0 outra vez — o censo prova que a PORTA faz efeito, a costura
prova que o clique chega ao BARRAMENTO, e nada juntava as duas pontas.* ⚠️ **Os gates que existiam
não podiam apanhá-lo:** eles pintam **em cima de um vértice**, que é o caso em que as duas leis
concordam. *Uma fixtura que aponta sempre para um nó não testa o que acontece entre eles.*

⭐⭐ **A lei que fica** ([`ancora_da_mancha`](../../crates/ph2d-skeleton-live/src/ancora_da_mancha.rs)):
a mancha pousa no **ponto do CONTORNO** sob o dedo, e o repouso dele sai do **MESMO parâmetro** da
curva. ⚠️ O achatamento é o mesmo nos dois lados, e é isso que torna a tradução honesta — *dois
achatamentos diferentes dariam um repouso plausível e errado*. ⭐ E **estar DENTRO da forma conta**:
a barra tem meia unidade de meia-altura contra um pincel de `0,40`, logo uma régua que só olhasse o
contorno recusaria exactamente a linha por onde o artista arrasta.

⚠️ **A diferença entre as mídias é DECLARADA:** numa IMAGEM continua a ser o vértice da malha mais
perto — ali a deformação entre dois vértices é a interpolação linear deles, não há «entre» a que
pousar, e os vértices são densos. *Uma lei só para as duas teria de escolher entre recusar o meio de
uma barra e mover um mapa que o dono já aprovou em smoke.*

**Medido, de ponta a ponta pela porta do produto** (a barra da cena, dobrada `0,8` rad na ponta, seis
pinceladas com o raio e a magnitude de FÁBRICA ao longo do meio dela):

| | |
|---|---|
| a arte move-se, pela lei da curva | **`0,141983`** |
| as MESMAS manchas, pela lei dos pontos de controlo | **`0,000000`** |

⭐ O controlo é o A/B das duas leis sobre as mesmas manchas — *é a metade que prova que quem move a
arte é a F30 e não o recook a mexer-se sozinho*.

⛔⛔ **DUAS premissas morreram e foram reescritas com a morte à vista no diff:** *«a mancha é
ancorada no NÓ»* (hoje: **sobre a ARTE** — a cerca que o report original pedia fica, e mais forte,
porque uma alça de quina vive FORA da curva) e *«o meio da aresta não é um ponto que o desenho
tem»*.

⚠️ E uma mutação SOBREVIVEU: prender a fracção da projecção a `[0,1]`. Sem isso o cursor projecta-se
**para lá do fim** da corda e essa distância ganha da verdadeira — a mancha pousaria **fora da
peça**. *Um `clamp` não é defensivo: é a diferença entre projectar numa CORDA e projectar na RECTA
que a contém.*

Mutação **8 de 8** a sangrar. Zero schema, zero registo novo.

✅ **FECHADO pela F32, e não pela cura que esta linha prescrevia.** Escrito aqui: *«a barra tem oito
pontos coloridos e a mancha entre eles não move nenhum deles»*, com a cura a ser expor a mistura de
pesos da [`curva`] ao indicador. ⛔ **Essa wave não chegou a ser precisa:** a subdivisão do bind pôs
nós onde a mancha pousa, e medido na barra do produto uma pincelada de fábrica muda **`5` dos `34`**
pontos (maior mudança `0,1304`). ⚠️ *A cura certa de «o artista não vê onde o peso mora» era dar-lhe
pontos, não desenhar-lhe manchas* — e as duas leituras têm o mesmo aspecto numa lista de abertos.

### F30 — ⭐⭐⭐ **A ARTE SEGUE O PESO ENTRE OS NÓS** (ordem do dono, 2026-09-19, *«construa e veremos se fica bom»*)

**MEDIDA antes de escrita uma linha de produto**
([`sonda_da_pele_como_warp`](../../crates/ph2d-vec-skin/src/sonda_da_pele_como_warp_tests.rs)), e a
medição **mudou o desenho**.

⛔⛔⛔ **A nota que descrevia esta saída estava errada em DUAS coisas.** Ela dizia *«deformar a forma
por uma MALHA … o preço NÃO é “a `ph2d-poly2d` já existe”: ela parte de uma **grelha de ALFA**, logo
a forma teria de ser RASTERIZADA»*.

1. **A rasterização não é precisa:** a [`ph2d_poly2d::triangulate`] recebe um **anel de pontos** e a
   [`ph2d_poly2d::grid_mesh_of`] também — a grelha de alfa é **uma** das entradas
   ([`ph2d_poly2d::mesh_of`]), não a única. *É a terceira nota minha que a medição derruba nesta
   jornada.*
2. ⭐⭐⭐ **E a MALHA não é precisa para a GEOMETRIA.** A [`ph2d_vec_envelope`] já deforma geometria
   **Bézier** por um mapa **não-afim**, e o cabeçalho dela descreve, por escrito, o defeito que a
   pele tem hoje: *«só transformações afins comutam com a avaliação de Bézier … a curva resultante
   não é a imagem da curva original … ela acerta em `t=0` e `t=1` exactamente, e no interior
   nunca»*. ⇒ **é a razão de a arte não responder a peso pintado entre os nós**, e é a mesma raiz do
   salto que o dono recusou na F28.

**O que a sonda mediu** (dois ossos, rectângulo de `40 × 10`, uma mancha no meio da aresta de baixo —
o sítio exacto da pergunta do dono):

| pergunta | resposta |
|---|---|
| peso pintado ENTRE dois nós move a arte? | **hoje `0,000000`** · pela rota do warp **`0,242375`** |
| o `fit_to_bezpath` converge? | **sim**, mesmo com jacobiana por diferença finita (⇒ não morre aqui) |
| custo | **`0,163 ms`** em `--release` (`0,881` em debug), `accuracy 0,05` |
| nós da saída | `4` na fonte ⇒ **`16`** no desenho |

⛔⛔ **O QUE FALTA, e é o que torna este item pegável:**

1. ⚠️⚠️ **A sonda usou a lei DERIVADA (`weights_corrected(p, None, …)`), e o produto usa a do
   PADRÃO-OURO** — uma linha guardada **por ponto de controlo**, que **não tem forma contínua**. ⇒ a
   malha volta, mas **só como portadora do campo de pesos**, nunca da geometria: guardar a malha do
   domínio do bind e amostrá-la, ou interpolar as linhas dos dois nós ao longo do `t` (a lei que a
   F28 já escreve para o ponto novo). *É esta a decisão que abre a wave.*
2. **A jacobiana tem de ser FECHADA.** O contrato do [`ph2d_vec_envelope::Warp`] exige a derivada
   real, e o doc dele mede que uma inconsistente faz o fit **não convergir** — ela falha **alto**.
   `∂W/∂p = Σ_j [ A_j(p) ⊗ ∇w_j(p) + w_j(p) · L_j ]` ⇒ é preciso `∇w_j`, que um campo baricêntrico
   dá **descontínuo** e o campo de Hermite da [`ph2d_poly2d::hermite_attrs`] dá **suave**.
3. **O `recook` corre por quadro.** `0,163 ms` por forma é `~1 %` de um quadro; dez formas presas são
   `10 %`. ⇒ ou memo por pose, ou a `accuracy` deixa de ser `0,05` — e **nenhum dos dois números foi
   escolhido por ninguém**.
4. ⏳ **Decisão de PRODUTO, e é do dono:** o desenho cozido passa a ter **mais nós** que a fonte
   (`4 → 16`). Ninguém os edita (a fonte é que se edita), mas o ***Expand*** assa a geometria de agora
   no desenho — ali o artista fica com a forma refitada. *É o único sítio onde o número sai do
   quadro e entra no documento.*


---

#### ✅ CONSTRUÍDA no mesmo dia, e ela DISSOLVEU a F28

A lei vive em [`ph2d_vec_skin::curva`](../../crates/ph2d-vec-skin/src/curva.rs) e é o caminho de
**OMISSÃO** (`PH2D_SKIN_CURVE=0` bissecta). Num segmento de `a` para `b`, o peso do ponto `C(t)` é a
**mistura** das linhas dos dois nós — `lerp(ra, rb, t)`, com as manchas somadas **no ponto** —, a
curva é amostrada e **refitada** (`kurbo::fit_to_bezpath`), e a remontagem do contorno é a **porta**
que a [`ph2d_vec_envelope`] já tinha (⛔ duplicá-la poria a convenção `(⅓, ⅔)` da elevação de recta
em dois sítios).

| o quê | medido |
|---|---|
| mancha ENTRE dois nós move a arte | **`0,000000` → `0,836850`** |
| os NÓS mexem-se? | **`0,000000000`** — em `t = 0` e `t = 1` a mistura é a linha do próprio nó |
| em REPOUSO | **`0`** ao bit |
| custo | `0,877 ms` em debug · `0,163 ms` em `--release` |
| nós do desenho | `4` na fonte ⇒ `6` desenhados |

⛔⛔⛔ **E o REFIT só corre onde o mapa NÃO é afim — isto não é optimização, é a cura de um defeito
medido.** A 1.ª redacção refitava **sempre**, e o gate `binding_a_shape_moves_nothing` acusou
`13,333…` = **`40/3`** em REPOUSO: a elevação `(⅓, ⅔)` de uma recta desenha a **mesma** curva com
outros pontos de controlo. *O desenho estava certo e a representação é que mudava*, e **oito** gates
da casa mediam a representação. ⇒ a lei de hoje corre **sempre e primeiro** (ela preserva o `kind` e
o `corner_radius`, que um refit não pode preservar), e só os contornos que se afastam mais do que a
tolerância são refitados.

⭐⭐⭐ **E a F30 DISSOLVEU a compensação da F28.** Com o desenho a ser a imagem verdadeira da fonte,
partir a fonte **não o move** (`0,000002 %`) — e compensar **estraga** (`11,11 %` da peça). ⇒ a
compensação passa a ser da lei dos pontos de controlo, e a decisão sai da mesma porta que o `recook`
lê. *Uma cura fica errada no dia em que o defeito que ela curava deixa de existir.*

⚠️⚠️ **E a lei viaja como PARÂMETRO, nunca num estado global.** A 1.ª redacção pôs um átomo com uma
porta `forcar_lei` para os gates medirem o outro lado, e o doc dela dizia *«o nextest corre um
processo por teste»* — verdade para o `nextest`, **falsa** para o `cargo test`, que corre os testes
em THREADS do mesmo processo. A suíte **reprovava em conjunto e passava sozinha**, que é a assinatura
mais cara que há. ⇒ `recook_com` / `insere_ponto_com`, e quem lê o ambiente é a porta de cima.

⚠️ **Cinco gates da casa tiveram a premissa mudada, e a morte de cada uma está no diff:** o
instantâneo do hit-test tinha *«um ponto por ponto DESENHADO»* e passa a ter **um por ponto da
FONTE** (⭐ e ter menos é a resposta certa: o peso vive nos nós, e um ponto do indicador onde não há
peso para corrigir seria um controlo morto) · a expectativa da tabela guardada constrói-se com a
**mesma** lei que o quadro corre · e a barra da excursão do envelope desceu de `1,0` para `0,5`, com
o número medido ao lado.

⛔ **E uma MUTAÇÃO SOBREVIVEU duas vezes, nas duas crates:** apagar a lei de hoje do início da porta
não partia nada, porque **toda** fixtura dobrava um osso e o refit escrevia por cima — *o caminho
onde a lei de hoje é a única a trabalhar não tinha fixtura nenhuma*. ⇒ o gate novo é **UM** osso,
onde a deformação é afim por teoria: ali a arte tem de se mover **e** o desenho tem de ficar
byte-idêntico ao de sempre.

Mutação **7 de 7** a sangrar; `nextest-impacted` **15 686** verdes.

### F29 — ✅ **FECHADO (2026-09-20): os DOIS modos de atribuir peso** (ordem do dono, 2026-09-19, *«coloque na fila»*)

> ✅ **FECHADO em `0029f2fc8`**, 36 ficheiros, 22 gates, **17 de 17 mutações a sangrar**.
> `PROJECT_SCHEMA` **+1** (⚠️ conte o DELTA) — e o degrau é obrigatório porque o campo foi
> **TROCADO**, não apendado: um ficheiro do schema anterior lido por este binário leria o primeiro
> byte do `f64` como o discriminante da espécie, **em silêncio**.
>
> ⭐⭐ **A cura é um enum que CARREGA o número** (`Especie::Soma(f64)` · `Especie::Alvo(f64)`): um
> campo cujo significado depende de um modo guardado ao lado é um defeito à espera, e assim o
> compilador obriga **todo** leitor a dizer qual dos dois está a ler.
>
> ⭐⭐⭐ **E *«a última manda»* cai de GRAÇA da aplicação SEQUENCIAL:** fixar é idempotente, logo no
> centro da última `Alvo` (onde o *bump* vale `1`) o que fica é o valor dela. ⛔ Sem escrituração
> nenhuma — *a ORDEM DA LISTA passa a ser a lei*, e re-pintar uma absoluta sobe-a para o fim.
>
> ⚠️ **DUAS mutações SOBREVIVERAM primeiro, e as duas eram réguas minhas a medir no sítio errado:**
> a regra do dono para o caso degenerado só é observável em **`v = 0`** (com `Σoutros = 0` a
> renormalização final devolve `1` para qualquer valor positivo, logo a faixa inteira menos o zero
> não a distingue), e a normalização que precede uma `Alvo` é **invisível NO CENTRO** (ali o pino é
> total e a escala re-normaliza os outros seja qual for a soma deles) — o discriminador é ler
> **FORA** do centro.
>
> ⚠️ **E um gate meu reprovou por aritmética minha, não por lei:** no centro de `B`, o *bump* de `A`
> vale `0,5625` e não `1`, logo `A` mistura em vez de fixar. *Ler «a última manda» no centro de
> quem deve perder é medir a mistura, não a precedência.*
>
> ⛔ **O `lerp` com o *bump*, nunca uma multiplicação:** `valor · bump` daria peso **ZERO** na borda
> da mancha (um buraco); `lerp(actual, valor, bump)` volta à lei automática com derivada zero.

### F29 (o pedido original)

*«Precisamos de 2 modos de atribuir peso aos pontos.»*

1. **Valor ABSOLUTO** — o valor de *Brush Strength* é posto **imediatamente** no osso em mãos, e o
   que sobra (`1 − v`) reparte-se pelos **outros** ossos que já têm peso naquele ponto, **mantendo a
   proporção entre eles**. ⇒ neste modo os botões *Add* e *Subtract* ficam **inactivos**.
2. **Valor CUMULATIVO** — a cada pincelada o nó ganha ou perde o valor de *Brush Strength*, conforme
   o botão marcado. **É o que existe hoje.**

⚠️⚠️ **A DIFERENÇA NÃO É DE UI — É DO MODELO DE DADOS, e é por aí que se começa a medir.** A
correcção é hoje uma [`ph2d_skeleton::Correccao`] — uma **MANCHA no espaço** que **SOMA**
(`w += delta · bump · quota`, e a normalização vem depois). Duas manchas sobrepostas **acumulam-se
por construção**, que é exactamente o modo 2.

⛔ **O modo 1 não é exprimível como uma mancha de soma**, e a pergunta que o decide é uma medição:
*duas manchas ABSOLUTAS sobrepostas — o que recebe um ponto que está debaixo das duas?* Se a resposta
é *«a última que o artista pintou»*, então uma correcção absoluta **não é um campo somável** e o
`Correccao` precisa de espécie (`Soma` / `Alvo`), com a ordem da lista a passar a ter significado —
⚠️ e ela **viaja em bytes opacos dentro do `SkinBind`**, logo é degrau de `PROJECT_SCHEMA`.

⚠️ **E a repartição do modo 1 não é a normalização que já existe.** Hoje o `corrige` soma e depois
divide pela soma — o que *diminui* proporcionalmente **todos**, incluindo o osso em mãos. O modo 1
pede outra coisa: **prender** `w[alvo] = v` e escalar **só os outros** por `(1 − v) / Σoutros`.
⛔ E ele tem um caso degenerado nomeado: *e quando os outros somam ZERO?* (um ponto que só o osso em
mãos governa). Ali não há por onde repartir, e a resposta tem de ser escrita antes de o código a
escolher sozinho.

⭐ **O que já está pronto:** o `WeightDirection` (os botões *Add*/*Subtract*) tem porta própria e
**três** consumidores — esconder/inactivar os dois no modo absoluto é a lente do painel, que já
existe para o `Pose` e o `Density`. E o censo dos knobs mede se um controlo chega ao barro, logo um
botão inactivo que continue a escrever seria apanhado.

⭐⭐⭐ **A PERGUNTA QUE DECIDIA ESTÁ RESPONDIDA — ordem do dono, 2026-09-19:** *«a última manda (é o
comportamento normal de um pincel absoluto)»*.

⇒ **duas manchas absolutas sobrepostas não se somam: vence a ÚLTIMA que o artista pintou.** Isso
fecha as três consequências que a medição acima previa, e nenhuma delas é de UI:

1. a `Correccao` ganha **espécie** (`Soma` para o modo cumulativo · `Alvo` para o absoluto);
2. a **ORDEM da lista passa a ter significado** — hoje ela é um conjunto de contribuições comutativas
   e passa a ser uma pilha onde a última `Alvo` que alcança um ponto ganha;
3. e como a lista viaja em bytes opacos dentro do `SkinBind`, isto é **degrau de `PROJECT_SCHEMA`**.

⛔ **O caso degenerado continua por decidir e é do dono:** um ponto que só o osso em mãos governa
(os outros somam ZERO) não tem por onde repartir o `1 − v`. As duas saídas honestas são *o pincel
não faz nada ali* e *o `v` é ignorado e o peso fica `1`* — ⚠️ escolher em código sem perguntar é
como esta casa produz um controlo que faz duas coisas diferentes conforme o sítio.

### F28 — ⭐⭐⭐ **UM PONTO NOVO NUMA FORMA PRESA SOBREVIVE, E JÁ NASCE COM PESO** (ordem do dono, 2026-09-19)

A 1.ª das duas saídas que a F26 deixou ao dono para *«pintar peso entre os vértices de uma forma
vectorial»*, e ele escolheu-as **em ordem**: *«primeiro 1 e depois o 2»*.

⛔⛔⛔ **E a nota que descrevia esta saída estava ERRADA no ponto que decidia o preço.** Ela dizia
*«acrescentar vértices com a caneta … o gesto já existe nesta casa. **Custo: zero de
arquitectura**»*. **Medido pelo caminho do produto antes de escrever uma linha**
([`sonda_do_ponto_novo_tests`](../../crates/ph2d-app-skeleton/src/sonda_do_ponto_novo_tests.rs)): a
caneta escreve no documento **VIVO**, e o `recook` reconstrói esse documento a partir da geometria
**autorada** que o bind guardou — uma vez por quadro. *O ponto aparece sob o dedo e desaparece
sozinho*, sem erro, sem aviso e sem recusa. ⇒ *uma PRESENÇA afirmada sem olhar o caminho do produto
é um palpite com cara de medição* — a mesma família que este repo já pagou no sentido oposto.

⛔ **E o contorno óbvio — «acrescente o ponto e carregue em *Bind* outra vez» — custa o trabalho do
artista:** o `SkinBind::new` nasce com `correcoes: vazio` e `law: Auto`, logo um re-bind deita fora
**todas as correcções pintadas à mão** (a feature da F26) e a escolha de lei daquele desenho.

⭐⭐ **A lei: a FONTE é que ganha o ponto, e a tabela cresce com ele**
([`ph2d_skeleton_live::ponto_novo`](../../crates/ph2d-skeleton-live/src/ponto_novo.rs)). O ponto
entra na geometria autorada, no mesmo segmento e no mesmo parâmetro em que a mão o pediu, pelo mesmo
`split_segment` de sempre; o quadro seguinte re-deriva o desenho dali. ⛔ *Escrever também no
documento vivo seria a segunda resposta à mesma pergunta.*

⚠️ **A linha de pesos do nó novo é a MISTURA das dos dois vizinhos, no mesmo `t`** — e não a lei
automática. A tabela guardada vem do padrão-ouro (uma resolução **global** sobre a malha do domínio):
pedir a lei derivada só para este nó poria **um ponto a obedecer a outra lei** no meio de uma forma,
e re-resolver o global mudaria o peso de **todos** os outros nós, apagando a linha de base que o
artista corrigiu. ⭐ A mistura é uma combinação **convexa** de duas partições da unidade, logo não há
normalização a fazer — e há gate a afirmá-lo.

⭐⭐⭐ **E o desenho move-se um pouco ao acrescentar o ponto — o que parecia um defeito é REFINAMENTO,
e a escada prova-o.** O desenho cozido é a Bézier dos pontos de controlo **deformados**, e não a
imagem verdadeira da curva de repouso pela pele: *ele já é uma aproximação*. Cortando o mesmo
segmento `1 → 2 → 4 → 8` vezes, o desvio entre degraus cai **`18,89 % → 3,13 % → 1,00 %`** da peça —
uma sequência que converge geometricamente não corrompe nada.

| fixtura | salto ao acrescentar um ponto |
|---|---|
| esqueleto em **REPOUSO** | **`0` ao bit** |
| aresta **CRUA** (um segmento a atravessar os dois ossos) | **`18,89 %`** da peça |
| aresta **DESENHADA** em 8, pior segmento (o da junta) | **`0,91 %`** da peça |

⚠️ **Os `18,89 %` não são o custo de acrescentar um ponto — são o tamanho do erro que aquele único
segmento já tinha, e o corte mostra-o.** A barra do gate é uma **catraca MEDIDA** (`1 %`) com censo
de obsolescência nos dois sentidos, ⛔ nunca um *«acima de X o artista vê»*, que seria um palpite.

⚠️⚠️ **DUAS armadilhas de FIXTURA, as duas apanhadas pelos controlos e nenhuma pelo olho:**
1. A 1.ª régua da forma desenhada leu **`0,0000 %`** — os dois extremos do segmento `0` estão ambos
   dentro do primeiro osso, logo a lei preserva a forma **ao bit por construção** e a barra passava
   por **vácuo**. Quem a apanhou foi o controlo `gap`. O sítio onde o peso varia é a **junta**, e o
   gate passa a medir o **pior** segmento.
2. O construtor da fixtura «desenhada» subdividia sempre o **primeiro** pedaço, deixando o **último**
   a atravessar a junta inteira — ela chamava-se desenhada e media o mesmo segmento grosseiro do
   outro palco. *Uma fixtura com o nome errado responde à pergunta do vizinho.*

⭐ **TRÊS peças de substrato que a wave obrigou, e as três são melhores do que o que substituem:**
o formato guardado ganhou **porta** (`skinned_mesh::le`/`grava` — ele era descodificado **à mão em
sete sítios**, cada um com a sua cerca); a caneta passa a **reportar onde inseriu**
(`PenTool::take_insercao`, porque o `t` é do dedo e reconstruí-lo do outro lado faria o ponto nascer
noutro sítio da mesma curva); e o `SkinnedPath` ganhou `linha_do_no`.

⛔⛔ **E uma cerca SAIU por uma mutação que sobreviveu:** o `if !fonte.valida()` depois do splice é
inalcançável por construção, e o `recook` já o faz a jusante, onde ele defende do caso real (uma
fonte gravada por outra versão). *Uma linha que a mutação não consegue matar não é lei, é comentário
com sintaxe de código.*

⛔⛔⛔ **E o FIO teve DUAS mutações sobreviventes, uma em cada ponta:** apagar o registo na caneta
deixava `10` testes da shell verdes (o gate de costura de lá lê o TEXTO do despacho — ele afirma que
a shell *drena*, nunca que a caneta *grava*), e cravar `t = 0,5` no registo passava o gate novo,
porque o dedo dele estava **no meio do segmento**, onde o `t` verdadeiro *é* `0,5`. *As duas pontas
de um fio precisam cada uma do seu gate, e um corpus no ponto neutro de um valor não testa esse
valor.*

**Na tela:** nada de novo — é a CANETA de sempre, e o roteiro da cena `PH2D_VEC_BONE_SMOKE=1`
ensina-a **onde a limitação aparece**: no aviso de que a barra laranja só tem oito nós (gate a exigir
que a cura fique a menos de 400 bytes do aviso que a motiva — *duas linhas separadas por vinte lêem-se
como dois assuntos*).

⛔⛔⛔ **E o PORTÃO DE FECHO apanhou um gate MEU vermelho, com uma causa que vale para toda régua de
curva desta casa: o `t` é o PARÂMETRO DA CURVA, não a fracção ao longo da CORDA.** A minha régua
esperava `0,3` (onde o dedo estava) e leu **`0,375`** — numa quina os dois pontos de controlo
interiores colapsam nas âncoras, logo a cúbica é `P0,P0,P1,P1` e a posição avança com `3t² − 2t³`, o
*smoothstep*; resolvendo, dá exactamente `0,375`. *A régua estava errada e o código certo.* ⇒ ela
passa a medir o **PRODUTO** — onde o ponto NASCEU —, que é a pergunta do artista, não depende da
parameterização, e mata na mesma o `t` cravado.

⛔⛔⛔ **E isso expôs um furo no ARNÊS DE MUTAÇÃO que invalidava as provas daquele gate: ele não
perguntava se o teste estava VERDE antes de mutar.** Com o gate já vermelho, **todas** as mutações
sobre ele liam *«SANGRA»* — *um teste já vermelho certifica qualquer mutação*. O arnês ganhou o
controlo (`exit 5`, com a razão), e as duas provas daquela ponta foram **refeitas** com ele.

⚠️ **Promoção pedida à lista de flakes de carga do `CLAUDE.md` §5.0:**
`a_long_stroke_is_bounded_by_the_redundancy_floor_not_by_a_budget`
([`ph2d-app-flip`](../../crates/ph2d-app-flip/)) — reprovou no meio de um fan-out de **15 506** testes
e passa **3 de 3 sozinho a `load 33–38`**, com **zero** linhas do diff desta wave naquela crate. Na
mesma corrida reprovou o `no_expression_allocates_no_link_frame`, que **já é membro nomeado**.

Portão: `fmt` limpo · clippy `-D warnings` a zero · `nextest-impacted` **15 506 verdes** (eram
`15 297`) · mutação **12 de 12** a sangrar (três sobreviveram primeiro — duas viraram gate e uma
matou uma cerca —, e duas foram refeitas depois do arnês ganhar o controlo de verde).
Zero contador partilhado, zero contrato, zero ADR. Tecto de LOC curado por **CORTE** (os gates do
roteiro saíram para `smoke_bone_roteiro_tests.rs`), nunca por isenção.

⏳ **ABERTO:** a 2.ª saída que o dono pediu a seguir — **deformar a forma por uma MALHA**.

#### ⛔⛔⛔ F28-b — O SMOKE REPROVOU-A, com DOIS reports, e o primeiro derrubou uma conclusão MINHA

*«não ficou bom. O ponto criado na malha já conectada aos ossos deforma a malha»* · *«não tem
indicação visual que você está em cima da linha para criar um ponto»* (2026-09-19).

**(1) O SALTO DA FORMA ERA UM DEFEITO, e eu tinha-lhe chamado refinamento.** A F28 mediu o salto,
mostrou que a escada da subdivisão converge, concluiu *«é a aproximação a ser refinada»* e **disse-o
ao dono como se fosse normal**. ⛔ *A régua dele é a que manda: o desenho é o que o artista vê.* A
conclusão não era falsa — era uma explicação a fazer de veredito.

⭐⭐⭐ **A cura é uma INVERSÃO, e ela sai da própria estrutura da lei:** o `recook` lê **uma** linha de
pesos — a da âncora — e aplica-a às três metades do vértice; com essa linha fixa, `x ↦ blend(x, w)` é
um **AFIM**. ⇒ o corte faz-se no **DESENHO** (de Casteljau sobre os pontos de controlo já deformados)
e o ponto de repouso que desenha em `X` é `L⁻¹(X − c)`, com `L` e `c` lidos por **três** avaliações da
própria porta — sem uma segunda cópia da lei.

| fixtura | ANTES | AGORA |
|---|---|---|
| aresta CRUA (um segmento sobre os dois ossos) | `18,89 %` da peça | **`0,000000 %`** |
| aresta DESENHADA em 8, pior segmento (o da junta) | `0,91 %` | **`0,000023 %`** |
| escada `1 → 2 → 4 → 8` | `18,89 → 3,13 → 1,00 %` | **`~1e-14`** nos três |

⭐⭐ **Os dois vizinhos não entram na conta, e é por isso que a lei é barata:** o `out` do anterior e o
`in` do seguinte já saem certos **ao bit** — eles são combinações afins de pontos que usam o MESMO
peso, logo o corte comuta com a deformação ali. *Só o vértice do meio mistura os pesos das duas
pontas.*

⭐ **Em REPOUSO a compensação é a IDENTIDADE ao bit** (toda pose é a identidade ⇒ `L = I`, `c = 0`), e
há gate a afirmá-lo. *Ela só existe onde há deformação para preservar.*

⚠️ **O preço, declarado:** o que absorve a diferença é a geometria de REPOUSO. Acrescentar um ponto
com o rig POSADO deixa o repouso deslocado do corte ingénuo pela mesma grandeza que o desenho
deixaria de saltar. ⛔ **As duas coisas não podem ser preservadas ao mesmo tempo** (só o seriam se as
duas pontas do segmento tivessem o mesmo peso), e a escolha é a do dono: *o desenho é o que ele vê*.

⚠️⚠️ **O número de PASSAGENS ficou observável por uma MUTAÇÃO SOBREVIVENTE, e a fixtura mordeu DUAS
vezes antes de conter o fenómeno.** A linha de pesos depende da POSIÇÃO (as manchas do pincel, o
`quota` de um osso que dobra), e mover a âncora muda-a ⇒ a compensação repete. Mas no corpus de então
**nada** dependia da posição, e `1` passava. A fixtura nova é uma forma com mancha pintada, e ela
falhou duas vezes: a 1.ª punha o ponto no **cume** da bolha, onde o `clamp(0,1)` **satura** e o peso
volta a ser constante (*uma mancha saturada não é uma mancha, é um planalto*); a 2.ª escrevia o braço
de «uma passagem» à mão e **não fazia crescer a tabela de pesos** (*um controlo que não percorre a
MESMA porta compara dois programas*). Com ela: `0,0772 → 0,0014 → 2,6e-5 → 4,7e-7 → 0` — cada
passagem divide por **~55**, e **`6`** é onde a escada acaba. ⛔ E uma saída antecipada por
convergência **SAIU** por outra mutação sobrevivente: ela não muda um bit, só poupa passagens de custo
nulo.

**(2) A PRÉVIA DE INSERÇÃO — o gesto existia e era INVISÍVEL.** O artista tinha de adivinhar a que
distância da curva o clique deixa de acrescentar um ponto e passa a **começar uma forma nova** —
*duas coisas muito diferentes, sem nada na tela a separá-las*. ⇒ um anel VERDE e OCO com uma CRUZ,
no ponto onde o clique poria o nó.

⭐⭐ **A posição e o RAIO vêm da porta do clique** (`PenTool::previa_de_insercao` chama o `insert_hit`
e decide o raio com a mesma linha do press) — *um realce calculado por uma segunda conta acende num
sítio e insere noutro*, o defeito que os realces do Trim e do Balde já nomeiam por escrito. ⚠️ E ele
é **derivado por quadro** e **LIMPO fora do modo Pen**: um realce deixado a arder depois de trocar de
ferramenta promete um ponto que nenhum clique põe.

⚠️ **As três metades têm gates separados porque são três defeitos:** ninguém calcula (nunca acende) ·
ninguém pinta (o report volta inteiro com o trabalho feito por baixo) · e acende onde o clique **não**
insere (pior do que não acender). ⛔ **O que NÃO se pôde fotografar:** o realce precisa do cursor
sobre a linha, e o XTest da sessão virtual é ignorado — *a foto prova o que abre, não o que passa o
rato*.

Portão: `fmt` limpo · clippy `-D warnings` a zero · `nextest-impacted` **15 630 verdes** · mutação **9 de 9** a sangrar (duas sobreviveram primeiro: uma virou fixtura e a outra matou a
linha). Zero contador partilhado, zero contrato, zero ADR.

### F27 — ⭐⭐⭐ **O CENSO DOS VERBOS DO OSSO: o clique chega a um EFEITO?** (2026-09-19)

**O item que a F16 deixou aberto por escrito, fechado — e o veredito é bom: ZERO verbos mortos.** Os
**catorze** botões da secção chegam a um efeito: **treze** mexem no mundo e **um** declara que o
consumidor dele é o clique seguinte.

⛔⛔ **É a segunda metade da pergunta que o `§5.0` nomeia sobre o repo inteiro** (*«nenhum instrumento
pergunta se o VALOR chega a um consumidor»*). A F16 fechou-a para os **números** em 18/09 e escreveu
na própria célula que os verbos ficavam com censo de *chegam ao barramento* e nenhum de *chegam a um
efeito* — que é a família de metade dos reports do dono nesta linha: *o botão pinta, acende sob o
rato, o clique atravessa o painel, e o mundo não se mexe.*

⭐⭐ **A régua é o PRODUTO e a fotografia é a do UNDO.** Cada verbo corre pela **porta que a shell
chama**, sobre um palco montado para ele, e o que se mede é a captura
[`world_to_snapshot`](../../crates/ph2d-ecs/src/scene/save.rs) — a mesma que a fila do undo tira.
⚠️ **A cena vectorial entra ao lado dela**, e não por gosto: o *Expand* escreve a geometria deformada
no **documento do vector**, que não é uma entidade — sem essa metade, uma mutação que fizesse o
*Expand* chamar o *Release* ficava invisível, porque os dois tiram o `SkinBind`.

⛔⛔⛔ **E UMA MUTAÇÃO SOBREVIVEU, e é o achado da wave: NADA no repo liga as duas pontas.** Apagado o
corpo do braço do *Add Smart Bone* na fase do quadro, **`23` testes da shell ficaram verdes**. O censo
da família prova que a **PORTA** faz efeito; a costura do painel prova que o clique chega ao
**BARRAMENTO**; e o terceiro elo — *o braço que recebe chama alguma porta?* — não tinha instrumento
nenhum. É a quarta vez que esta rota morre nesta linha. ⇒ `VerboDoOsso::rastos_na_shell` mais o censo
[`todo_verbo_do_osso_deixa_rasto_na_shell`](../../shells/desktop/tests/it/os_verbos_do_osso_chegam_do_botao_ate_a_lei.rs),
que vive **ao lado dos três gates que já faziam isto à mão** para o *Look At*, o desvio e o espelho —
*uma segunda superfície para a mesma pergunta seria a lista que envelhece*.

⚠️⚠️ **Ele mede TEXTO e não uma chamada, e a limitação é DECLARADA:** as fases são métodos de `App`,
que segura uma surface de janela real, logo nenhum teste as corre. *Ele apanha o braço que deixou de
chamar a porta; o braço que a chama com o argumento errado é apanhado do outro lado* — pelo censo da
família, que corre as duas portas e exige que elas **difiram**.

⭐⭐⭐ **E a `smart::add` NASCEU por causa do censo.** Das catorze rotas, o *Add Smart Bone* era a única
cujo efeito estava escrito **dentro da fase do quadro** (um `insert` de uma linha), logo a única que o
censo não conseguia correr sem re-escrever a lei — *e uma régua que re-escreve a lei mede outro
programa*. A shell decide a ORDEM; **o que** um verbo faz é conhecimento de quem possui o componente,
que é a lei que o [`knobs`](../../crates/ph2d-app-skeleton/src/knobs.rs) já escreve.

⚠️ **A tradução `id → verbo` resolve pela POSIÇÃO na tabela** (a mesma lei do lado da dobra e do
sentido do pincel de peso), e é isso que impede uma segunda lista de catorze braços. ⛔ **O preço está
pago com gate:** trocar dois itens da `VECTOR_BONE_VERBS` faria o botão que diz *Bind* mandar
*Release* — *um botão que faz o contrário do que diz é pior do que um morto* —, e por isso o censo
pina **cada id ao verbo pelo NOME**, um a um.

⛔ **A ÚNICA isenção é NOMEADA e tem gate próprio:** o *Pick Object* arma um **MODO** e o consumidor
dele é o clique seguinte. *Uma célula sem proveniência e uma com proveniência têm o mesmo aspecto numa
tabela* — é a mesma forma do `Strength` no censo dos números, e o número de isentos está gateado em
`1`.

⚠️⚠️ **DUAS armadilhas de FIXTURA, as duas apanhadas pela primeira corrida:**
1. **Prender e assar no mesmo instante devolve a FONTE** — o `bind` guarda a pose de AGORA como
   repouso, logo *o Expand não tem nada para assar num corpo que não saiu do repouso*, e os dois
   verbos liam-se idênticos. A ordem do palco é **prender · dobrar · re-cozinhar**.
2. **O piso do censo textual era a SOMA e tinha de ser POR VERBO.** Quatro verbos declaram dois
   rastos (a porta partilhada mais o discriminador), logo esvaziar um verbo inteiro deixava
   `16 >= 14` e o censo verde. *Uma lista vazia lê-se exactamente como aprovada* — a catraca sem
   censo de obsolescência, um nível abaixo.

Mutação **16 de 16** a sangrar (duas sobreviveram primeiro e as duas viraram gate). Zero contador
partilhado, zero contrato, zero ADR, zero linha de produto mudada — a única troca no caminho do
artista é o `insert` do *Add Smart Bone* passar a ir pela porta.

### F26 — ⭐⭐⭐ **CORRIGIR UM PESO À MÃO — o pincel, a mancha e o olho** (2026-09-19)

O quarto e último item da auditoria: *«quando a conta automática erra num sítio, não há como
acertar aquele ponto»*. Hoje há — um **terceiro verbo** na fileira do osso (**`Weight`**), e
arrastar sobre a arte presa empurra a influência do osso em foco para cima (ou, com o valor
NEGATIVO, para baixo).

#### A lei: uma MANCHA no espaço, nunca uma tabela por vértice

⛔⛔ **A tabela por ordem de varredura é o *vector paralelo* que o `VecVertex::corner_radius` proíbe
por escrito**, e este módulo já a recusou uma vez (os pesos *derivam-se*, não se guardam): dezenas
de operações inserem, apagam, invertem e soldam vértices, e cada uma teria de se lembrar de a
mexer. ⇒ a correcção é **ancorada na geometria** ([`CorreccaoDePeso`]): ela diz *«aqui»*, e
continua a dizer «aqui» depois de o artista mexer no desenho.

A bossa é `(1 − x²)²`, a **mesma** da lei euclidiana — `C¹` na borda por construção, logo a
correcção não põe um degrau no campo. ⭐ E por ser somada DEPOIS da lei, ela vale nas **duas**
(`Auto` do padrão-ouro · `Envelope`): *o artista corrige aquele ponto, e de que lei veio o peso que
ele está a corrigir não é pergunta dele.*

#### ⭐⭐⭐ O que a torna correcta: a mancha é pintada na POSE e guardada no REPOUSO

O artista vê a arte **deformada** — é lá que ele vê o defeito — e a correcção tem de viver na
geometria de repouso, senão ela andaria com a pose e corrigiria o sítio errado no quadro seguinte.
⇒ o dedo escolhe o ponto **POSADO** mais perto e o que se guarda é o **repouso desse mesmo ponto**
([`peso_a_mao`]). ⚠️ E o **centro nunca é o cursor cru**: ancorá-lo ali poria a mancha no vazio
quando o dedo passa ao lado da arte, e ela deixaria de corrigir exactamente quando o artista pensa
que a pôs.

#### O olho: o pincel deixou de ser cego

⛔ Corrigir um peso sem o ver é apontar para um número que não está na tela. Com o verbo armado,
cada ponto da arte presa é um **ponto colorido** pela influência do osso em foco — a rampa
`Info → Danger` que toda ferramenta de rig usa —, mais o **anel** do pincel (raio em MUNDO, porque
o raio *é* uma distância do desenho). ⭐ **Um peso de `0` é pintado, e é a metade que importa:** sem
ele o artista vê onde o osso já manda e **não vê onde ele devia mandar e não manda**.

⚠️ **O olho lê a MESMA porta que o quadro** (`weights_corrected`, com as manchas já dentro) — *uma
pré-visualização que ignora o trabalho feito faria o artista pintar duas vezes o que já pintou*.

#### As duas constantes, e o que cada uma é

| const | valor | o recurso |
|---|---|---|
| `MANCHAS_MAX` | `128` | o **relógio do quadro**: `205 µs` sobre `2 000` pontos com o tecto cheio (`--release`), `1,2 %` de um quadro de 60 Hz — `8 ×` abaixo do décimo que o gate exige |
| `FUSAO` | `0,5` | a **distância**: duas pinceladas a menos de meio raio uma da outra são a mesma mancha, e é isso que faz um arrasto custar o que ele percorre |

⚠️⚠️ **E a `FUSAO` quase ficou sem régua:** o gate óbvio (*«pintar duas vezes no mesmo sítio dá uma
mancha»*) fica **verde com ela a zero**, porque o centro é snapado ao ponto da pele e duas
pinceladas no mesmo sítio fundem por igualdade **exacta**. Quem a mede é a irmã, com dois pontos
**vizinhos** e o raio DERIVADO da distância entre eles. *Uma mutação que sobrevive é a régua a
dizer onde ela não olha* — e o tecto pagou a mesma lição (a estrela nunca o alcança; ele é medido
na LEI).

**Na tela:** a fileira do osso passa a ter **três** segmentos, e com o `Weight` armado aparecem
**`Brush Radius`** e **`Brush Strength`** (com sinal — negativo TIRA; ⛔ não há um segundo verbo
«apagar» a lembrar nem um modificador a adivinhar). `PROJECT_SCHEMA` **+1** — conte o DELTA.

⚠️ **O traço pertence à arte em que começou** (o alvo congela no press): com duas formas presas a
encostar-se, re-perguntar a cada evento poria metade da correcção no desenho errado.

#### ⛔⛔ E a FOTO mostrou que o verbo era inalcançável na própria cena dele

O painel dos ossos é a **única** porta dos três verbos (`Create` · `Transform` · `Weight`), e o
prólogo da cena `PH2D_VEC_BONE_SMOKE` só o abria no nível `=2` — logo o `=1`, que é o que **tem
arte presa**, mostrava um esqueleto cujas ferramentas o artista não conseguia alcançar. ⭐ *A cena
estava certa como DADOS e era impossível como GESTO* — a mesma forma que o `#15` da `line/components`
pagou, e que nenhum dos gates dela via, porque todos liam a cena como dados.
⇒ `painel_do_osso` passa a ser **incondicional** (a timeline e o enquadramento continuam do `=2`,
e é isso que mantém intacta a cena que o dono aprovou), com a morte da premissa **visível no diff**
do gate que a prendia. ⚠️ E o texto que a cena imprime deixou de mandar *«abra o painel Skeleton»*:
*uma instrução que descreve o app de ontem é mais cara que instrução nenhuma.*

#### ⚠️ A escolha do alvo é LEI, e vivia no laço de desenho

O tecto de LOC da fase do overlay obrigou o corte, e ele achou o defeito: *«de quem se mostram os
pesos»* — o alvo congelado do traço, senão o que está sob o dedo — estava escrita dentro do laço de
desenho da shell, **onde teste nenhum lhe chega**. ⇒ [`peso_a_mao::pontos_do_indicador`], com gate
de **quatro** braços e a fixtura de **duas** peles que é o que o torna discriminante (com uma só,
«o congelado ganha» e «o dedo escolhe» devolvem o mesmo bloco).

#### ⛔⛔⛔ O SMOKE DO DONO REPROVOU-O, e os DOIS reports eram o mesmo defeito com duas caras

*«a barra laranja não é subdividida o bastante (só tem pontos nas extremidades)»* · *«os pontos não
ficam coloridos (não há indicativo de peso)»*. Medido na peça REAL da cena (o `RoundRect` do braço,
3 ossos, `ppm 100`):

| osso | peso `0` | peso `1` | **entre** |
|---|---|---|---|
| `Bone 1` | 12 | 12 | **0** |
| **`Bone 2`** | **24** | 0 | **0** |
| `Bone 3` | 12 | 12 | **0** |

⭐ **O peso vive por VÉRTICE, e a barra tem `14` posições distintas — todas em quatro cachos nos
cantos, nenhuma ao longo do comprimento.** O report 1 descreve o modelo com exactidão: *o esqueleto
só pode mover os pontos que o desenho tem*. E o report 2 tem três causas, **as três minhas**:

1. ⛔⛔ **O passo do smoke que eu escrevi mandava clicar no `Bone 2`** — o único osso da cadeia que
   não possui **nada** naquela arte (o miolo de uma cadeia de 3 sobre 4 cantos). Com ele em foco
   todos os pontos leem `0`, e a tela fica de uma cor só. *Um passo que nomeia uma coisa AFIRMA que
   ela serve para o que o passo diz* — a família do §0.8, outra vez.
2. ⛔⛔⛔ **O raio de fábrica era `20` unidades de MUNDO = `2 000` px**, contra `702` px da peça
   inteira e `27,6` px entre dois pontos vizinhos: **`2,85 ×` a peça**. Um clique agarrava todos os
   pontos de uma vez e o anel era maior que a janela. ⇒ o raio passa a ser de **ECRÃ** (`40` px,
   com a tabela derivada: `4 ×` o raio de pick da casa, `1,45 ×` a distância entre vizinhos, `5,7 %`
   da peça), convertido a mundo nos **dois** sítios do despacho — com gate, porque converter num só
   deixaria o pincel a escolher a peça com um raio e a pintar com outro.
   ⚠️ **E o doc do anel ARGUMENTAVA a favor da unidade errada** (*«o raio do pincel É uma distância
   do desenho»*): verdade sobre o que a **mancha guarda** e falso sobre o que o **artista escolhe**
   — e um número de mundo **não pode ter valor de fábrica**, porque teria de saber a escala da cena.
3. ⛔⛔ **As três recusas do pincel eram `eprintln!`** — *uma recusa que só o terminal vê é um botão
   mudo*, e a porta para a tela tinha sido construída por esta mesma linha **um dia antes**. Elas
   entram agora na população `RecusaDoOsso`, que é a que o censo deriva.
   ⚠️ **E o censo da família reprovou ao recebê-las, duas vezes, cada uma por uma cegueira própria:**
   ele contava **chamadas num ficheiro só** (as do pincel saem do despacho, não da fase) e uma mesma
   recusa pode ter **dois** sítios que a levantam ⇒ passou a perguntar *«alguém consegue EMITIR
   isto?»*; e a 2.ª redacção, que procurava o NOME nas superfícies, acusou de muda a
   `VariosEsqueletos` — que viaja como **valor** vindo da porta e nunca é nomeada ⇒ o universo passa
   a ser as superfícies **mais os produtores da crate**.

✅ **RESPONDIDA pelo dono em 2026-09-19: *«primeiro 1 e depois o 2»***. A primeira FECHOU no mesmo dia
(ver **F28**) e ⛔ **a nota abaixo sobre ela estava errada — *«custo: zero de arquitectura»* foi
medido e é falso**: a caneta escreve no documento vivo e o `recook` deita-o fora todo quadro. A
segunda está ABERTA, e é a próxima.

⏳ **A pergunta, como ela foi posta:** pintar peso **entre** os vértices de
uma forma vectorial era impossível, porque não há lá peso nenhum para corrigir. Duas saídas:

- **acrescentar vértices com a caneta** onde se quer controlo — o modelo do Moho/Spine, e o gesto já
  existe nesta casa. Custo: zero de arquitectura.
- **deformar a forma por uma MALHA**, como a imagem já é. ⚠️ **E o preço NÃO é «a `ph2d-poly2d` já
  existe»**: ela parte de uma **grelha de ALFA** (*«geometria pura sobre uma grelha de alfa»*, diz o
  cabeçalho dela), logo a forma teria de ser **rasterizada** para se tirar a cobertura, e o desenho
  passaria a ser deformado por uma amostragem dele em vez de pelos próprios pontos — o que muda o
  que o traço é. ⛔ Arquitectura, e não é minha para decidir.

#### ⛔⛔⛔ E O SMOKE SEGUINTE REPROVOU OUTRA VEZ — *«nada fica vermelho e nada fica azul»*

⭐⭐⭐ **A causa foi a minha PRÓPRIA cura anterior a expor um segundo defeito que ela escondia.** A
porta que responde *«que arte está sob o cursor»* media a **distância ao PONTO posado mais perto** —
e os pontos de uma forma vivem nos CANTOS dela. Medido na barra da cena, com o cursor a meio do
comprimento:

| cursor em `x` | achou a arte? |
|---|---|
| `−8,4` (canto) | sim |
| `−8,0` … `−2,0` (todo o miolo) | **NÃO** |
| `−1,6` (canto) | sim |

Sem arte encontrada não há ponto nenhum para desenhar ⇒ a tela fica **vazia**, que é o report à
letra. ⚠️⚠️ **E enquanto o raio valia `20` unidades de mundo (`2 000` px) isto era invisível**, porque
a arte era sempre encontrada — *por acidente*. ⇒ **um número a fazer dois trabalhos esconde o defeito
do segundo enquanto estiver errado no primeiro**: aqui eram *«até onde o pincel alcança»* (um
tamanho de pincel) e *«que arte está debaixo do dedo»* (um teste de acerto), e o segundo **nunca foi
uma distância**.

⇒ a porta passa a perguntar pela **SILHUETA POSADA**: dentro do contorno achatado (par/ímpar) para um
caminho, dentro de um triângulo posado para uma imagem, com a proximidade a um SEGMENTO — e nunca a
um vértice — como rede para o caminho ABERTO, que não tem interior. ⛔ Tudo em geometria pura: a
crate é folha e não traz a `kurbo` para responder a um teste de ponto. ⭐ Quem CONTÉM ganha de quem
está perto, senão com duas artes sobrepostas o dedo pintaria a que só passa por ali.

#### ⭐ E a segunda observação do dono estava CERTA, com a cura ao contrário do que parece

*«parece que os pesos não são aplicados apenas nos nós, mas também nos handles (alças)»* — **é
verdade, e tem de continuar a ser**: o esqueleto transforma a âncora **e** as duas alças, e uma alça
parada com a âncora a andar quebrava a curva. O que estava errado era o **DESENHO**: numa forma de
cantos arredondados as alças ficam em posições distintas, logo `8` nós apareciam como **`24`
pontinhos**, e a leitura era ruído. ⇒ o indicador mostra **um ponto por NÓ**; a lei continua a
devolver os três. ⚠️ **E as duas metades são gateadas juntas**, porque as curas seriam opostas:
esconder o desenho e apagar o efeito leem-se igual numa tabela.

⭐ **Medido: âncora e alças nunca divergem nesta arte** (`divergem 0` nos três ossos), o que torna o
ponto do nó uma descrição fiel e não um resumo. ⛔ Elas **podem** divergir em geral (o peso sai da
POSIÇÃO, e uma alça longa alcança território de outro osso), e é por isso que a mancha continua a
apanhá-las pelo espaço.

#### O ponto era pequeno demais para a cor ser legível

| grandeza | píxeis |
|---|---|
| dois NÓS vizinhos da barra | `70,7` |
| o ponto de então (raio) | `2,5` — **`3,5 %`** do vão |
| o ponto de hoje (raio) | `5,0` — `14 %` do vão |

⚠️ O diâmetro de `10` px não é escolhido: é a família das alças de gradiente desta casa (`~9` px).
⛔ E a rampa foi **ilibada com número** antes de se lhe tocar: os dois extremos são tokens de hue
`25` e `235` — vermelho e azul de verdade.

**Resultado, pelo caminho do produto, com o cursor no meio da barra:** `8` pontos, **`4` vermelhos e
`4` azuis** no `Bone 1` e no `Bone 3`; uma cor só no `Bone 2`, que é a resposta CERTA (o osso do
meio de uma cadeia de três sobre oito nós não possui nada) — *e foi ele que o meu passo de smoke
mandou clicar*.

#### ⛔⛔⛔ E A FOTOGRAFIA ACHOU UM TERCEIRO DEFEITO QUE GATE NENHUM PODIA VER

Com os dois curados acima, a tela continuava vazia — e o que o mostrou foi **ver**, não medir. A
sonda `PH2D_VEC_WEIGHT_PROBE=1` arma o verbo, escolhe o osso pelo NOME e pousa o cursor no meio da
barra; com ela, o diagnóstico do caminho do produto diz `pontos=8` e **a tela não os desenha**.

⭐⭐⭐ **A causa: para escolher entre CAMINHO e IMAGEM eu perguntei `tem Sprite?`** — e no app (ao
contrário da fixtura de unidade) **uma forma vectorial também carrega um `Sprite`**. Toda arte
vectorial ia pelo ramo da imagem, onde o `SkinnedMesh` não parseia, a porta respondia *«não achei»*
e não havia ponto nenhum. ⚠️ **A porta certa já existia no mesmo ficheiro** (a `e_caminho`, escrita
para o indicador uma hora antes): *duas respostas à mesma pergunta divergem, e estas divergiram em
duas horas.*

⚠️⚠️ **E a fixtura de unidade estava VERDE sobre o defeito** porque a entidade dela não tinha
`Sprite` — *uma fixtura que não contém o fenómeno não prova nada sobre ele*. Ela passou a ter um, e
a mutação que repõe a pergunta pelo `Sprite` sangra.

#### ⚠️ E o que a foto mostrou a seguir mudou o SMOKE, não o código

Com tudo certo, na barra laranja o artista **continua a ver uma cor só** — e a razão é geométrica:
os `8` nós dela estão nos dois extremos, e **a câmara da cena corta a ponta esquerda**, que é a que
o `Bone 1` possui. Com o `Bone 1` vê-se o extremo direito (todo a `0`, azul); com o `Bone 3`, o
contrário.

⭐⭐⭐ **Na IMAGEM pintada da mesma cena o indicador é o que devia ser:** `925` pontos numa nuvem
densa de **azul → vermelho**. ⇒ *o pincel é demonstrável na mídia que tem malha, e quase inútil numa
forma de oito nós* — que é a pergunta de produto abaixo, agora com foto dos dois lados.

Mutação **27 de 27** a sangrar, **cinco** delas sobreviventes à primeira e curadas com gates novos.

⏳ **ABERTO e nomeado:** o caminho de **GPU** não conhece as manchas — dívida **com gate**
(`a_pele_da_placa_nao_conhece_as_correccoes_e_isso_esta_nomeado`), inofensiva só enquanto ele não
tiver consumidor de produto · não há botão de *limpar as correcções* (o `Ctrl+Z` e o valor negativo
cobrem-no) · e a mancha não é espelhada pelo `Mirror Branch`.

#### ⭐⭐⭐ F26-b — O TERCEIRO REPORT: *«as cores não ficam tão boas como no blender · o algoritmo continua considerando pesos em alças · as cores só aparecem se o mouse estiver sobre a forma»*

Três queixas, **três mecanismos diferentes**, e nenhuma delas era a mesma coisa que as duas rondas
anteriores tinham curado.

**(1) A rampa era uma CONFUSÃO DE CATEGORIA.** Ela era `ColorToken::Info → ColorToken::Danger`, e um
token semântico é escolhido para ser **CALMO** dentro do chrome (`Info` é `oklch(0,720 0,110 235)`,
um azul de baixa croma); uma leitura de VALOR é escolhida para ser **DISTINGUÍVEL**. São requisitos
opostos. Medida em OKLab com `21` amostras:

| rampa | caminho total | **pior passo** | uniformidade |
|---|---|---|---|
| `Info → Danger` (a de ontem) | `0,305` | `0,0131` | `0,739` |
| as 5 paradas da indústria, espaçadas em `t` (o porte INGÉNUO) | `1,435` | `0,0082` | **`0,057`** |
| as 5 paradas em OKLCH, por arco | `1,397` | `0,0383` | `0,443` |
| **as 5 paradas em OKLab, por ARCO** (a que shipa) | `1,393` | **`0,0497`** | `0,651` |

⭐⭐ **O número que decide é o PIOR passo e não o caminho** — é ele que diz se dois pesos vizinhos se
distinguem —, e por ele o **porte ingénuo da rampa da indústria seria PIOR que o que havia**
(`0,63 ×`): o verde puro é um **planalto**. ⇒ a cura não é mudar as cores, é **parametrizar por
comprimento de arco perceptual**, o que põe as paradas em `0,000 · 0,380 · 0,546 · 0,680 · 1,000`.
O verde — a única referência que um artista lê («metade») — desloca-se `+0,046`; o ciano, que
ninguém lê como número, paga os `+0,130`. ⚠️ **Ela não muda com o tema, e isso é lei:** *uma rampa
que muda com o tema deixa de ser a leitura de um número.*

**(2) «Pesos em alças» — a ordem do dono, agora na LEI e não só no desenho.** A ronda anterior curou
o **DESENHO** (um ponto por nó) e deixou a lei como estava, com a objecção escrita: *«uma alça parada
com a âncora a andar quebrava a curva»* (o `CubicWeight` do Rive). ⛔ **Ele repetiu, e a objecção
fica registada e NÃO VENCIDA** — mas a medição deu-lhe razão por um mecanismo que a objecção não
via: a mancha do pincel é um **bump radial**, vale `1` no centro (a âncora) e menos nas alças, que
estão ao lado. Medido na arte dele (a barra de `PH2D_VEC_BONE_SMOKE=1`, um dab de `amount = 1`):

| ponto | lei de hoje (peso do NÓ) | lei de ontem (peso da posição) |
|---|---|---|
| âncora | `0,5000` | `0,5000` |
| alça de entrada | `0,5000` | **`0,2150`** |
| alça de saída | `0,5000` | `0,5000` |

⭐⭐ **A assimetria é o mais duro:** das duas alças do MESMO nó, uma seguia e a outra não — a
tangente partia-se exactamente no ponto que o artista acabara de pintar. *Um peso que ele não
consegue entregar ao nó inteiro num gesto não é um peso que ele controla.*

⭐ **E a cura tem TRÊS metades, porque «pesos em alças» aparecia em três sítios:** a deformação
([`ph2d_vec_skin::aplica_corrigido`] faz **uma** conta por vértice, na âncora), o instantâneo posado
que o hit-test usa, e — a que ninguém tinha visto — **onde a mancha é ANCORADA**: o
[`ponto_sob_o_cursor`] escolhia entre todos os pontos, logo o centro de uma correcção podia cair
numa alça. Os três lêem a MESMA porta ([`ph2d_vec_skin::dono_do_peso`]).

⚠️ **Sem dab as duas leis CONCORDAM nesta arte** (as alças de uma quina ficam a `0,28` da âncora) —
é a pincelada que as separa, e é por isso que a régua que só olhava o repouso não via nada.

**(3) «As cores só aparecem se o mouse estiver sobre a forma» — DUAS perguntas lidas como uma.** O
indicador perguntava *«que arte está debaixo do dedo?»* — a mesma pergunta do pen-down, com a
justificação escrita no código —, e devolvia **vazio** no vão entre as formas. ⛔ *Onde o traço vai
pintar* é do DEDO; *o que este osso governa* é do OSSO, e não tem cursor nenhum dentro. ⇒ o
indicador mostra toda a pele cujos tendões contêm o osso em foco, sempre. ⭐ **A população é exacta
e não uma escolha:** uma pele que não o tenha é a que o `pinta` recusa com `OssoDeFora` — *pintá-la
de azul prometeria um pincel que a porta ao lado recusa*. Preço medido: **`2,16 ×` o `recook` da
mesma arte** (`27,75 µs` contra `12,83 µs` na estrela), com a razão gateada contra `4 ×` — *deixou
de haver um quadro barato (o dedo no vão) e um caro; todos passaram a custar o caro.*

**E a auditoria achou um QUARTO, da família do controlo morto:** o aviso do bind
(*«N pontos de controlo caem FORA do interior da forma»*) contava as **alças**, cuja linha da tabela
deixou de ser lida. ⇒ ele conta só os **NÓS** — *queixar-se de uma condição que já não tem consumidor
é ensinar o artista a ignorar a queixa*. ⚠️ As linhas das alças continuam a ser **gravadas** (a
tabela viaja em bytes opacos dentro do `SkinBind::source`) e ficam **dívida NOMEADA**: elas são
AMOSTRAS e não incógnitas — o sistema resolve-se na malha do domínio —, logo tirá-las não mexeria
num único peso de âncora. *É dívida de tamanho, nunca de resultado.*

⛔ **Três premissas MORRERAM e as três morrem à vista no diff:** o cabeçalho do
[`ph2d-vec-skin`] (*«cada metade com os pesos da posição dela»*), o gate
`the_three_halves_of_a_vertex_answer_to_their_own_position` (substituído por
`uma_alca_move_se_pelo_peso_da_ancora_dela`, **na mesma fixtura**, com o veredito invertido e o
controlo positivo dentro) e o `o_indicador_segue_a_arte_do_traco_e_nao_o_dedo`.

**E o ROTEIRO da cena passou a ENSINAR o pincel** — ele nunca o mencionava, em **três** reports do
dono sobre ele. ⛔⛔ **E o osso que ele nomeia é o do MEIO, nunca a PONTA, medido por FOTOGRAFIA:**
a 1.ª redacção reaproveitava o nome que a lição do *Onion* já tinha à mão (o da ponta) e dali a
parte visível do braço lê-se **quase toda azul** — a zona que a ponta governa sozinha cai atrás do
painel *Bones*. ⚠️ *É a mesma armadilha do «Bone 2» do 1.º report, e ela voltou porque o nome mais
fácil de alcançar no código não era o nome certo para o gesto.* ⭐ A derivação virou **porta**
(`osso_do_meio`) por causa de mais uma mutação sobrevivente: com ela inline, o gate tinha de
**copiar** a conta — e uma cópia julga a cópia.

#### ⭐⭐⭐ F26-c — *«no lugar de valores negativos em Brush Strength prefiro botões Add e Subtract»* (smoke APROVADO + ordem, 2026-09-19)

⛔⛔⛔ **A objecção estava escrita no painel e fica REGISTADA E NÃO VENCIDA:** *«o `Amount` é COM
SINAL, e é isso que faz o gesto ser um só: negativo TIRA peso. ⛔ Um segundo chip «apagar» seria a
segunda maneira de dizer a mesma coisa.»* ⭐ **O que ela não via** está agora escrito na
[`ph2d_tool_vector::WeightDirection`]: enquanto o sinal vivia dentro do número, *«tirar peso»* era um
**estado invisível** — o artista tinha de **ler um menos** para saber o que o próximo arrasto ia
fazer. ⇒ *uma pergunta, um controlo*: o número responde **QUANTO** (uma magnitude, que não tem sinal
que faça sentido) e os dois botões respondem **PARA QUE LADO**.

⭐⭐ **A composição é uma PORTA** ([`WeightDirection::delta`]) e não um `if` no despacho da shell:
*uma lei que só existe num laço de input é uma lei que ninguém pode contradizer* — foi exactamente
assim que a escolha do alvo do pincel viveu até esta manhã, e foi preciso um report do dono para a
descobrir. Gate na shell a exigir a porta **e** a proibir o sinal escrito ao lado dela.

⚠️ **Um negativo escrito à mão entra em valor ABSOLUTO, nunca cortado a zero:** cortá-lo deixaria o
pincel **inerte e calado**, que é a família de reports que esta casa já pagou três vezes. O campo é
re-semeado do estado da ferramenta a cada quadro ⇒ *o ecrã corrige-se à vista*.

⛔ **E escolher um lado NÃO arma o verbo `Weight`**, ao contrário dos três chips acima: a secção
destes dois só é pintada com ele já na mão, logo já se está lá — a mesma regra que os chips da
largura do lápis já escrevem, com gate nas duas metades.

⭐⭐⭐ **DUAS mutações sobreviveram e as duas eram achados de DESENHO:**
- **a lista de ids podia trocar de ordem sem nada acusar** — e com `[Sub, Add]` o segmento rotulado
  *Add* passava a mandar `Subtract`. ⚠️ *«Índice-alinhadas» era uma afirmação que nada verificava.*
  ⇒ o gate ata a posição ao SIGNIFICADO (`IDS[Add.indice()] == …_ADD`);
- **o id e o RÓTULO viviam em duas listas paralelas** ⇒ passam a viajar **emparelhados** numa porta
  só. *Um controlo que faz o contrário do que o rótulo dele diz é pior do que um controlo morto: o
  morto não engana.*

⚠️ **E o `populate` — a SÉTIMA vez que esta casa paga a lição:** os dois chips entram nele, com o
gate de costura a carregar-lhes com o **ponteiro REAL** (um `Click` sintético passa com o chip
morto). Mutação **10 de 10** a sangrar.

**Mutação `11 + 2` a sangrar, TRÊS sobreviveram à primeira e as três eram achados** (a barra da
rampa cega ao espaçamento uniforme em OKLab · o filtro por tendão que era a segunda resposta à
mesma pergunta · o instantâneo posado que podia divergir do desenho sem nada acusar). Portão:
**15 280** testes verdes, censos da árvore `90/90`, clippy `-D warnings` a zero. ⚠️ A `15 281.ª`
corrida acusou o `the_cost_of_depth_is_linear_not_explosive` — **membro nomeado** da família de
flakes de fan-out do `CLAUDE.md` §5.0, com **zero linhas** do diff naquela crate e a `line/UIUX` a
correr a suíte dela na mesma máquina (`load 30`); `2` de `3` verde isolado.

### F19 — ✅ **O CHIP `Auto` DIZ QUE LADO DERIVA** (report do dono, 2026-09-18)

*«IK Bend não funcionou com Auto IK e trocando CCw por CW no painel lateral»* — ⭐ **reproduzido, e
a lei estava CERTA.**

⛔⛔⛔ **A causa, MEDIDA** (`sonda_do_lado_do_joelho_tests`, no braço em **S** da cena dele): com o
`IK Chain` de **fábrica (`2`)** o `Auto` e o `Cw` dão a **MESMA pose, ao bit** (`0,0000`), e a cena
do osso **captura `Cw`** (lido do log dela). ⇒ o artista clica em **dois** dos quatro chips e não vê
nada mudar — indistinguível de um controlo partido. *Só o `Ccw` move (`3,0000`).* ⚠️ Com
`Chain = 3` o `Auto` **deixa** de coincidir (`2,9991`), e é essa metade que impede a leitura errada
*«o Auto é sempre o Cw»* — que levaria alguém a esconder um chip vivo.

⚠️ **A cura NÃO é esconder nem mexer no solver:** o `Auto` significa *«deriva o lado da pose que
chega»* e coincide **nesta** pose, não sempre. ⇒ ele **diz**: o rótulo passa a ser `Auto (CW)`, e o
artista vê, sem clicar, que pedir esse lado é um no-op.

⭐ **A porta do lado derivado é a MESMA que o solver usa** (`goal::side_for_chain`) — uma segunda
resposta a *«de que lado a pose está?»* divergiria da que governa a corrente, e o chip mentiria.

⛔⛔ **E TRÊS hipóteses caíram por medição antes desta**, cada uma com o número: a fiação do chip
está completa (`fase_bus_clicks` → `g.bend = lado`) · o painel **já** marca o chip activo · e os
chips **só** são pintados no osso que tem a âncora (a lei do controlo morto já lá estava).

⛔⛔ **E a minha régua mediu a grandeza errada, a QUARTA vez nesta sequência:** a 1.ª redacção lia a
**translação** do cotovelo e devolvia `[10, 0]` nos três lados — *o solver escreve ÂNGULOS*, e a
translação de um osso filho é fixa. *Uma régua que mede o campo que a lei não escreve dá sempre o
mesmo número*, e ela acusava o produto pelo report do dono.

⛔⛔⛔ **E uma MUTAÇÃO expôs uma cegueira do gate do elo:** ele lê o pintor por
`include_str!("section.rs")` e **a agulha que procura estava escrita nele próprio** ⇒ apagar a
chamada deixava-o VERDE. ⇒ os gates do rótulo mudaram para um ficheiro irmão. *Um gate
`include_str!` que procura uma string escrita nele mesmo não afirma nada.*

⛔ Tecto de LOC da fase (`208` contra `200`) curado por **CORTE** (`publica_a_ancora`), nunca por
uma entrada nova no `FN_OVERAGE_OK`.

Mutação **4 de 4** a sangrar; portão `15 099` verdes.

⏳ **ABERTO e nomeado:** o **pixel** do rótulo não é alcançável de um teste — o testkit desta casa
não tem leitor de texto pintado, e o que liga a lei ao pintor é um gate `include_str!`.

### F18 — ✅ **O LADO DA DOBRA ANIMA** (pedido do dono, 2026-09-18)

Ele escolheu *«o lado da dobra é escolhido e é consistente, mas **não é animável**»*. ⇒
`PropKind::IkBendSide` (id de fio **17**, append-only): uma track como qualquer outra, na lista do
*+ Track*, com rótulo, chave de i18n e alias de expressão (`Nome.bend_side`).

⛔⛔⛔ **E NÃO é o *Pole Target* — a recusa MEDIDA fica de pé, e está no doc do
[`ph2d_skeleton::BendSide`]:** em 3D o triângulo raiz–cotovelo–ponta roda em torno do eixo
raiz→ponta (um grau de liberdade **contínuo**, que um objecto no espaço fixa) e no plano isso **não
existe**: sobra **um bit**. Um alvo arrastável que codifica um bit dá a ilusão de um controlo
contínuo e **salta** ao cruzar a recta — Godot (`flip_bend_direction`) e Spine (`bendDirection`),
independentes, escolheram o interruptor. ⭐ *O que faltava não era o alvo: era a ANIMABILIDADE do
bit*, que o Spine tem e nós não tínhamos.

⚠️ **A convenção do valor é a do Spine** (`>= 0` ⇒ anti-horário) e o empate tem **vencedor
declarado**: sem isso o instante do salto dependeria do último bit de um `f32`. ⛔ O `Keep` e o
`Mixed` **não são alcançáveis** pelo canal, e a ausência é a decisão: os dois significam *«deriva o
lado da pose que chega»*, e um canal que os animasse estaria a keyar a AUSÊNCIA de uma escolha.

⛔ **Fora do auto-key**, e a razão está escrita: o valor muda por um clique num chip, e o auto-key
desta casa grava o que a MÃO move no canvas. *Pô-lo lá faria toda troca de chip virar uma chave.*

⛔⛔⛔ **E o achado da wave foi um `_ => None`:** os **quatro** `match` exaustivos da crate
obrigaram-me a responder pelo canal novo; o `from_target` — que traduz o id **opaco que o documento
grava** — tem wildcard, e a variante caiu nele **em silêncio**. Sem aquela linha uma track gravada
**não se resolve ao carregar**, e o gate genérico de ida-e-volta **saltava o canal por vacuidade**.
*Um `match` com wildcard é onde uma variante nova desaparece, e o preço ali é a PERSISTÊNCIA e não a
compilação.* Quem o mostrou foi uma **mutação** (a sonda podia escrever qualquer número e passava).

⚠️ **Duas cegueiras de régua, as duas registadas no ficheiro:**
1. o piso `checked >= 7` do gate genérico **segurava o número enquanto a população encolhia** ⇒ hoje
   é derivado (`checked == resolviveis`);
2. ⛔ e o derivado **também não apanha** a remoção da linha do `from_target`, porque `checked` e
   `resolviveis` descem juntos — *a régua partilha a lei do produto, e um espelho não acusa*. Quem a
   apanha é o gate do id, que afirma o `17` pelos **dois** lados.

⚠️ **Os gates vivem atrás da feature `skeleton`:** um `cargo test -p ph2d-timeline` sozinho imprime
`0 passed` — *um teste que não corre lê-se como verde*. O portão do workspace corre-os (a shell liga
a feature; conferido por `nextest list --workspace`).

⭐ De graça: a `ph2d-skeleton-ecs` passou a re-exportar o `BendSide` — o **segundo** caso que a nota
dela previa por escrito (*«um campo público cujo tipo não é alcançável pelo mesmo caminho é meio
campo»*).

Mutação **5 de 5** a sangrar; portão `15 094` verdes.

### F17 — ✅ **O ENVELOPE SÓ É PINTADO ONDE AINDA MANDA** (ordem do dono, 2026-09-18)

Ele perguntou *«Por que o envelope já não influencia na deformação?»* e a resposta expôs um
**controlo morto**: com os pesos do **padrão-ouro** uma imagem deforma **igual** a `1` e a `2` —
medido, coluna a coluna, na tabela que vive no doc da cena do pincel. Num rig só de imagens aquele
campo aceitava teclas, gravava no documento e **não mudava um pixel**.

⭐ **O envelope não morreu — MUDOU DE DONO:** uma forma **vectorial** presa continua na lei
euclidiana (o padrão-ouro precisa de uma malha do domínio, e uma Bézier não tem uma), e ali ele
manda como sempre. ⇒ o campo **volta** assim que houver uma forma vectorial presa.

⚠️ **A pergunta é da CENA e não do osso, e isso é uma limitação NOMEADA:** o `SkinBind` guarda a
malha e os pesos e **não os ossos**, logo *«este esqueleto tem forma vectorial?»* não é derivável.
A pergunta mais larga erra sempre para o lado **conservador** — *esconder um controlo vivo é pior do
que mostrar um inerte*.

⚠️ **O default publicado é `true`**, e a `limpa()` do arnês repõe-no: sem isso o teste que o desliga
contamina os seguintes, e eles ficam verdes sobre um painel sem aquele campo.

⛔ **E o meu censo de ontem (F16) não o apanhou:** ele mediu o alcance pela lei **euclidiana** (os
pesos por raio), onde ele é vivo — e é falso para uma imagem. *Uma régua que mede a lei antiga não
vê o que a lei nova apagou.*

Mutação **3 de 3** a sangrar, mais uma **inerte de controlo que sobrevive** (o arnês não é
hipersensível).

⚠️ **Promoção pedida à lista de flakes de fan-out do `§5.0`:**
`the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas` (`ph2d-tool-painter`) — gate de
RAZÃO, irmã de ficheiro de um membro já listado, reprovou no meio de um fan-out de `15 090` e passa
**3 de 3 a `load 64–74`**, que é carga MAIOR do que aquela em que reprovou, com zero linhas do diff
naquela crate.

### F16 — ✅ **O CENSO DOS NÚMEROS DO OSSO: o valor chega a um CONSUMIDOR?** (2026-09-18)

**O instrumento que faltava, e o veredito é bom: ZERO knobs mortos.** Os sete números do osso chegam
a um consumidor — **seis** à pose que a lei deriva ([`bend::frames`]) e o **alcance** aos pesos da
pele, com prova própria.

⛔⛔ **É a pergunta que o `§5.0` nomeia sobre o repo inteiro** (*«nenhum instrumento pergunta se o
VALOR chega a um consumidor»*) e a família dos dois reports do dono desta semana. Os censos que já
existiam neste painel provam que o clique e o valor **chegam ao barramento**; nenhum provava que
alguma coisa acontece a seguir.

⛔⛔⛔ **E uma varredura por NOME não serve — medido, e teria produzido 12 falsos positivos:** a shell
despacha estes ids **por tabela** (`VECTOR_BONE_BEND_IDS.iter().position(…)`, `BoneKnob::of_id`),
logo um `grep` pelo nome de cada id acusa **12 controlos VIVOS**. *Um id que a régua não vê e um id
morto leem-se igual* — a mesma forma que o `hit_indexed_ids_are_registered` já pagou noutro painel.

⚠️⚠️ **E a régua mentiu DUAS vezes antes de dizer a verdade, as duas por FIXTURA:**
1. Num osso **RECTO** o afim de flexão é a identidade qualquer que seja o comprimento ⇒ o `Length`
   lia-se **morto** sobre produto certo. *Uma régua medida no ponto neutro de OUTRO knob acusa
   este* — o arranjo do censo passa a ter curvatura, e há mutação a prová-lo load-bearing.
2. Com **um** osso a normalização dá-lhe sempre a fatia inteira (`1` contra `1`), e com um segundo
   **fora** do alcance responde o caminho de recurso *«o mais próximo leva tudo»* — o vizinho tem de
   estar **dentro** do alcance para que a razão seja o que se mede.

⭐⭐ **E uma MUTAÇÃO expôs uma cegueira do censo:** trocar um item de `TODOS` por uma cópia de outro
**compila**, mantém o comprimento em `7`, e tira uma variante da população sem ninguém ver. ⇒ o gate
passa a exigir **distintos**, não só a contagem. *Uma lista guardada só pelo tamanho não é uma
população.*

⛔ **O `Strength` é excepção NOMEADA e não uma folga:** o consumidor dele são os pesos, não a pose, e
ele tem gate próprio — *uma célula sem proveniência e uma com proveniência têm o mesmo aspecto numa
tabela*.

Mutação **5 de 5** a sangrar; portão `15 088` verdes.

✅ **FECHADO na F27 (2026-09-19):** este item dizia que o censo cobria os **números** e que os
**verbos** tinham censo de *chegam ao barramento* e nenhum de *chegam a um efeito*. Os catorze têm-no
agora, e a construção devolveu o terceiro elo que faltava — *nada no repo perguntava se o braço que
recebe o pedido chama alguma porta*.

### F15 — ✅ **AS TRÊS RECUSAS DOS VERBOS DO OSSO SOBEM À TELA** (2026-09-18)

**A dívida que a F13 abriu e a F14 herdou, fechada.** As três recusas do botão de osso saíam só no
terminal — *uma recusa que só o terminal vê é um botão mudo* —, e o dono aprovou **dois** smokes em
que foi preciso dizer-lhe *«olhe na janela preta»*.

⭐ **Nenhuma superfície nova:** a [`ph2d_editor_core::ToastQueue`] já servia a irmã desta mesma
família (o *solta-se-sozinho* de uma ferramenta que muda a moldura, com chave de i18n própria). O
`FrameGfx` já a carregava — *a composição já o exprimia, e ninguém tinha medido* (§5.0).

⚠️ **As três juntas, e não só a nova:** curar uma deixaria duas maneiras de responder à mesma
pergunta. ⇒ um enum só (`RecusaDoOsso`, três variantes) e **uma** porta na shell (`avisa`) — com
três `push` espalhados, a quarta recusa nasce muda, que é como estas viveram até aqui.

⭐⭐ **Toda recusa tem chave de i18n, e o `match` da `chave()` é EXAUSTIVO** ⇒ uma variante nova **não
compila** até alguém lhe dar uma. *É a diferença entre uma lista que alguém tem de se lembrar de
estender e uma que não fica verde sem a extensão.*

⚠️ **O terminal FICA ao lado do aviso, e não é duplicação:** um smoke headless não tem tela, e é ali
que a sonda lê. *A tela é para o artista; o terminal é para quem mede* — o mesmo par que o
`PH2D_BONE_LOG` já é.

⛔ **E a agulha de um gate contou `1` de `3` sobre produto CERTO**, pela segunda vez nesta jornada: o
`cargo fmt` parte as chamadas longas em várias linhas. *Um literal lê-se do ficheiro já formatado.*

Mutação **5 de 5** a sangrar; portão `15 084` verdes.

### F14 — ✅ **DESCONECTAR A MALHA DO OSSO numa IMAGEM** (report do dono, 2026-09-18)

*«Acho que ainda não temos a opção de desconectar a malha do osso. Deveríamos ter.»* — **ele tinha
razão, e o defeito eram DUAS metades, ambas mudas.**

⛔⛔ **(a) O painel não sabia.** O facto publicado era um `bool` que só olhava
`self.vec.pen.selected_paths()`, que para uma imagem dá **zero** ⇒ com uma imagem presa escolhida
ele lia `false` e os botões *Expand* e *Release* **nem eram pintados**. *O artista não via um botão
morto — via a ausência de um botão*, que é exactamente o que ele escreveu.
⚠️⚠️ **E o cabeçalho da própria fase já prometia a lei por escrito** (*«se a selecção tem forma
PRESA ou imagem com pele»*): *um doc que declara a lei que o código não implementa lê-se como
auditado.*

⛔⛔ **(b) O verbo não alcançava.** O `release` percorre `paths`; a lei da imagem
([`skin_image::release_image`]) **existia** e o **único** chamador de produto dela era **automático**
(uma ferramenta que muda a moldura solta o osso sozinha). *Uma lei sem gesto é uma lei que o artista
não tem* — a irmã do `dock_columns::close`, que este doc já nomeia.

⇒ o facto publicado passa a ser um **TIPO** (`Skinned { vector, imagem }`, pela lei que o `state.rs`
já escreve para o `BoneSpec`: os campos viajam juntos), o *Release* solta as duas mídias, e o
**`Expand` fica de fora por LEI da mídia** — ele troca o desenho autorado pela geometria deformada
de agora, e uma imagem **não tem geometria autorada** (a malha é derivada da tinta, por quadro).
Assar a deformação nos pixels é **outra** operação, que não existe. ⇒ escondido, não pintado-e-morto.

⚠️ **Duas cercas, não uma:** o painel esconde o *Expand* e o verbo cerca-se a `Keep::Source` — para o
caso de o comando chegar por outra porta.

⚠️ **E uma medição minha falhou por um `head -5`:** li *«`set_current_skinned` só tem chamadores de
teste»* porque a janela cortou a linha da shell. *Um `head` é uma janela, não um veredito* — a lição
já estava na memória do repo, e paguei-a na mesma.

Mutação **5 de 5** a sangrar; portão `15 082` verdes.

⏳ **ABERTO e nomeado:** assar a deformação de uma imagem nos pixels (o *Expand* da 2.ª mídia) não
existe — é wave própria, e só faz sentido com quem a peça.

### F13 — ✅ **VÁRIAS IMAGENS NUM ESQUELETO SÓ (o PERSONAGEM), e o *Bind* deixou de prender ao amálgama** (2026-09-18)

**A capacidade existe, está MEDIDA e é alcançável pelo gesto.** Duas sprites presas ao mesmo osso
semente deformam as duas, com excursões **diferentes** (`1,123 m` / `1,195 m`) ⇒ cada uma tem pele
própria, não é cópia. Cena **`=4`** (`PH2D_VEC_BONE_MEDIA_SMOKE=4`): três desenhos separados, um
esqueleto em **árvore** (tronco + dois membros) — ✅ smoke do dono aprovado.

⛔⛔ **A 1.ª sonda não media partilha nenhuma, e foi uma MUTAÇÃO que o mostrou:** num mundo com um
esqueleto só, `skeleton_of(sim, None)` devolve *«todos os ossos»*, que são os mesmos ⇒ prender ao
seed e prender a `None` dão o mesmo. ⇒ corrente **ISCA** + a grandeza que separa, que é a
**CONTAGEM de ossos do bind** (`3` contra `6`). ⚠️ A distância da isca **não** é load-bearing (a
mutação que a aproxima sobrevive, e está escrito no ficheiro).

⛔⛔⛔ **E isso expôs um defeito de PRODUTO, medido:** o botão *Bind* passa `semente =
osso_selecionado`, que é `None` quando nenhum osso está aceso. Com **dois** esqueletos na cena a
forma ficava presa aos **seis** ossos das duas cadeias, em silêncio, com o log a dizer *«1 imagem
presa»*. ⇒ porta pura [`ph2d_skeleton_live::recusa_do_bind`] — ela **recusa em voz alta** e diz o
gesto que cura (*escolher também um osso na Hierarquia*). ⚠️ **A cerca é o que a torna aceitável:**
com **um** esqueleto o caminho é byte-idêntico ao de sempre; *exigir sempre o osso partiria o fluxo
que o artista já aprendeu, para curar um caso que só existe quando há ambiguidade*.

⭐ A subida à raiz virou porta ([`esqueletos::raiz_do_osso`]) com **dois** leitores — e mudá-la de
sítio **tirou** linhas do `skin_live.rs`, que estava a `697` de um tecto de `700`.

⏳ **DÍVIDA NOMEADA:** esta recusa sai no **terminal**, como as duas que o mesmo botão já tinha.
*Uma recusa que só o terminal vê é um botão mudo* — e curar só a nova deixaria duas superfícies para
a mesma pergunta. **As três sobem à tela juntas**, numa wave com superfície própria.

⚠️ **Três leituras que o diff inverte:**
1. *«zero chamadores de produto de `bind_image`»* — **falso**, era a **fachada** da shell que o grep
   não resolve (`pub(crate) use ph2d_skeleton_live::skin_live::*`). A régua das fachadas erra nos
   **dois** sentidos, e aqui fez ler *«não existe»* sobre algo que existe.
2. *«a subida é comum às três peças»* — **refutado**: os pesos dependem da distância ao osso, e a
   própria sonda já media excursões diferentes.
3. A régua da disposição comparou com o **vão** (centro a centro) quando o que cruza é a **folga**
   (borda a borda) — a foto mostrou três peças sobrepostas **com o gate verde**.

⛔ **E um gate reprovou sobre produto CERTO:** o `the_bind_verb_reaches_both_media` ancorava em
`if pending_bone_bind {`, e a recusa exigiu um bloco rotulado (`'bind: { … break 'bind }`) para não
levar com ela o **soltar** e os **knobs** do mesmo quadro. *Um gate ancorado no idioma reprova no
dia em que o idioma muda* — a afirmação ficou, só a âncora foi curada.

Mutação **11 de 11** a sangrar (6 na cena + 5 na recusa); portão `15 079` verdes.

### F12 — ⏳ **ABERTO e NOMEADO: o *Frame All* enquadra a JANELA, e os painéis tapam-lhe as bordas** (2026-09-18)

⛔⛔ **Não é da pele nem do esqueleto — é do verbo da CÂMERA, e vale para toda a casa.** O
[`drain_view_focus`](../../shells/desktop/src/hero_intents/view.rs) do `ViewFocusKind::All` calcula

```rust
let aspect = window_size.width / window_size.height;      // a JANELA, não o canvas
let need_h = span_y.max(span_x / aspect);
camera.height_world = need_h * 1.1;
```

e o mundo é desenhado na janela inteira com os painéis **por cima**. ⇒ ele enche `110 %` da janela
com o conteúdo e **tudo o que um dock tapa fica fora**.

**Medido** (foto de 2026-09-18, janela `1930 × 1040`): as colunas laterais tapam `~37 %` da largura
e a timeline aberta `~33 %` da altura. Com a timeline aberta a cena `=3` pedia `Frame All` e ficava
a mostrar `±80 px` de mundo sobre um braço de `±120` — **cortado nas duas pontas**.

⚠️⚠️ **E NENHUM tamanho de cena o resolve:** o ajuste é derivado do próprio conteúdo, logo encolher
a cena encolhe o enquadramento junto. *Uma cena larga «não caber» não é propriedade da cena — é
propriedade do verbo.* (A nota da `=1` dizia *«cenas largas nunca cabem»* e tratava-o como lei da
cena; ele é do verbo.)

⭐ **A cura tem endereço:** ajustar ao rectângulo **LIVRE** (a janela menos os docks — os rects já
existem em [`panel_ops::panel_rects`](../../crates/ph2d-editor-core/src/interaction/state/panel_ops.rs))
em vez do da janela. ⛔ **Não foi feita aqui de propósito:** ela muda o enquadramento inicial de
**todas** as cenas de **todos** os módulos, e isso é decisão do dono e da linha da UI, não de uma
linha a meio de uma wave. *Contornar por dentro da minha cena e não dizer nada seria esconder um
defeito que todo artista atinge ao carregar em «Frame All» com a timeline aberta.*

⚠️ **O que a `=3` faz enquanto isso:** não pede `Frame All` (`Prologo::enquadrar = false`) e
dimensiona-se para a **câmera de omissão** (`height_world = 10 m`), com o número derivado dela.

---

### F6-g — ⏳ **O que sobra do orçamento, MEDIDO: o tecto é um buffer do QUADRO, e as costuras existem em todo modo** (2026-09-13)

**1. O tecto duro.** O Vello guarda a informação de todo desenho num buffer FIXO
(`bin_data = 1 << 18`, *«hand picked»* no `vello_encoding::BufferSizes::new`), e `binning_size =
bin_data − layout.bin_data_start` dá a volta a um `u32` quando passa: **pânico em debug, quadro em
branco em release — painéis incluídos**. Uma peça custa **11 palavras**, linear (gate
`a_skin_piece_costs_eleven_vello_bin_info_words_and_the_cost_is_linear`; sonda do produto
`VectorScene::probe_bin_info_words`).

**2. A GPU**, sonda `ph2d-render::skin_pieces_gpu_cost` (arte opaca `320×96`, alvo limpo por
grelha; zoom 8, carga `2,2`→`9,5`, as últimas linhas de relógio valem pouco):

| peças | quadro | BURACOS | px de costura | alfa mín |
|---:|---:|---:|---:|---:|
| sem recorte | `1,25 ms` | – | – | – |
| `216` (`Fast`) | `1,58 ms` | `0` | `33 476` | `182` |
| `3 456` | `2,46 ms` | `0` | `132 164` | `182` |
| `7 776` | `5,02 ms` | `0` | `259 231` | `171` |
| `17 496` | `10,2 ms`* | `0` | `408 886` | `170` |
| `21 600` | — | **todo o miolo** | — | `0` |
| `≥ 31 104` | pânico no Vello (debug) | | | |

⇒ numa cena **só com a pele** o quadro fica em branco entre `17 496` e `21 600` peças (as 11 palavras
mais a distribuição por bins, que usa o resto do mesmo buffer).

**3. ⛔⛔ As COSTURAS existem em todo modo, o `Fast` incluído:** dois recortes vizinhos com AA
analítico compõem `1 − a·b` na aresta partilhada, e o fundo espreita até **~29 %** (alfa `182`) numa
linha por aresta — `16 580` px a zoom 4 com as `216` peças do smoke.

**4. ✅ A guarda por QUADRO** (o tecto de `1 024` por IMAGEM passava por cima da tolerância — a
`k = 2` o `Smooth` entregava `3,5 px` numa dobra forte contra `0,5 px` pedidos — e não protegia o
quadro: N imagens presas multiplicavam-no). Medido o outro consumidor do buffer, o chrome do editor
pintado sem ecrã pelo registo real (sonda `ph2d-editor-core::vello_bin_budget_of_an_editor_frame`,
texto incluído): **`109`** palavras com os painéis de omissão, **`~1 190`** com todos abertos. ⇒
`SKIN_FRAME_PIECES = (1 << 18) ÷ 2 ÷ (11 + 4) = 8 738`, repartido **proporcionalmente** pelas
imagens presas (o mesmo `k` para todas), e dentro dele **a tolerância decide**. ⚠️ A metade que
sobra é da arte do canvas, que **não foi medida**. Gate
`the_smooth_pieces_of_all_skinned_images_share_one_frame_budget` (visto RED com o tecto por imagem:
`18 t` contra `9 t`). Malhas `Fast` que sozinhas passam do orçamento não têm o que cortar: aviso
único no stderr.

⏳ **ABERTO, e nomeado:**
- ✅ **As costuras — CURADAS pela F6-i.** Sobrepor cada recorte `~0,5 px` foi medido e recusado
  (dobra a composição em arte translúcida ao longo da costura); ficou o **pipeline de triângulos
  texturados** que a F6 nomeou *«com razão medida»* — e ele não pediu pipeline nova: é a malha
  dentro do passe de sprites.
- ✅ **Medido por leitura, e é pior do que a pergunta:** ver a F6-h.

### F11 — ✅ **AS 9 FATIAS E AS FOLHAS DE QUADROS DEFORMAM** (ordem do dono, 2026-09-17)

> *«vamos lá: imagens em 9 fatias e folhas de quadros»*

**A medição veio antes da primeira linha** (sonda `sonda_as_tres_formas`, `--ignored`, fica no repo),
com a mesma arte presa a uma corrente dobrada:

| forma | instâncias | com malha | o que se via |
|---|---:|---:|---|
| sprite simples | 1 | 1 | certo |
| folha `4×1` | 1 | **1** | **ERRADO, e calado** |
| 9-slice | 9 | **0** | sem deformar (avisava no terminal) |

⛔⛔ **A folha era o caso PIOR, e não era o que o aviso descrevia.** Ela passa o guarda (a instância
É o quad da sprite) e o defeito estava no BIND: a malha era traçada sobre a folha INTEIRA e o
`pixel_to_local` espremia-a no quad de UMA célula — `1 277` peças recortadas dos quatro quadros
dentro do sítio de um, com a UV de um só esticada por cima. *O 9-slice pelo menos avisava.*

**As duas curas**

- **A folha:** a malha nasce sobre a CÉLULA, e a tinta é a **UNIÃO de todos os quadros** — uma malha
  traçada só sobre o quadro vivo RECORTA todos os outros (prende-se no `0`, dá-se play, e os braços
  do `3` somem). ⭐ Com uma célula só é a identidade byte-a-byte. A porta é a
  [`ph2d_render::SourceCells`], a lei que a shell já tinha **duas** vezes e que desceu ao motor com
  o terceiro leitor.
- **O 9-slice:** a malha é cortada nas linhas das fatias **em pixels da imagem**
  ([`ph2d_poly2d::submesh_in_rect`]) e cada pedaço é esticado no quad DELE. ⛔ Recortar o QUAD está
  refutado por construção: o pedaço do meio mostra a faixa central ESTICADA. A costura é o extract a
  **publicar** a fracção (`SlicePatchSource`) — re-derivar a cadeia região → célula → fatia numa
  segunda casa divergiria no dia em que uma das duas ganhasse uma cerca.

⚠️ **Divergência declarada:** um quad que LADRILHA não repete a silhueta (o `uv_xform` faz a tinta
repetir e a malha é o pedaço único esticado). Numa arte opaca — toda moldura — é invisível.

⚠️ **E o `pixel_to_local` passou a ser o CASO PARTICULAR da régua geral** (`rect_to_quad`), com os 18
gates que já existiam verdes: é isso que prova que a generalização é exacta.

⏳ **Aberto:** a **pré-visualização de uma folha aberta** (o quad desdobrado sob uma ferramenta de
pixels) continua sem deformar, com o aviso — e é desenho: aquele quad não é um pedaço da arte desta
sprite. *Ela já nasce suspensa quando uma ferramenta a está a editar (regra F6-s), então o caso que
sobra é estreito.*

**Smoke:** `PH2D_VEC_BONE_MEDIA_SMOKE=1` — três imagens presas ao mesmo gesto, com o CONTROLO ao
lado. ⚠️ **Sete fotos antes de ir ao dono**, e cinco defeitos que nenhum gate via: ver a mensagem do
commit `02462ca8f` (o enquadramento que nunca cabia · a arte ao contrário · a união com gargalos ·
**configurar depois de prender** · e o toast da cena irmã a nomear a fileira apagada no dia anterior).

---

### F9 — ✅ **FECHADA (2026-09-20): a pele deforma-se NO DISPOSITIVO** (pedido do dono, 2026-09-16)

> ✅ **FECHADA em `f53d48138`**, por ordem do dono de 2026-09-20: ***«O caminho da placa gráfica:
> implemente se esse é o padrão ouro»***. ⭐ **A condição dele era um SE, e ele foi respondido por
> MEDIÇÃO antes de uma linha ser escrita** — o que vem a seguir é o que a medição disse.
>
> ⛔⛔⛔ **O GATILHO DA REABERTURA DISPAROU SOZINHO, e o número que a parou estava ERRADO por
> `4,5×`.** A nota de 2026-09-17 lia `1,824 ms` / `10,9 %` a 8 imagens e concluía *«não se gasta uma
> wave a comprar 11 % de um quadro que hoje sobra»*. Medido outra vez, com a decomposição:
>
> | | ms | % de um quadro |
> |---|---:|---:|
> | `attach_skin_meshes` INTEIRO, 8 imagens | `5,980` | **`35,9 %`** |
> | só a CÓPIA (dois clones por instância) | `0,033` | `0,2 %` |
> | só a LEI por vértice | `5,774` | **`34,6 %`** |
>
> ⇒ **`96 %` do custo é a lei a correr POR VÉRTICE**, e ele é **exactamente linear nas imagens**
> (`1,012` · `4,064` · `8,164` ms) ⇒ `16` imagens são `72 %` e `23` são o quadro **inteiro**, sem
> nada mais desenhado. ⚠️ **A minha hipótese era a CÓPIA e a medição derrubou-a** — *uma sonda que
> não parte o relógio acusa o suspeito errado*. E o dispositivo desenha `100 352` triângulos em
> `0,73 ms`. *§0.0 à letra: o caminho mais lento a definir o tecto do mais rápido.*
>
> ⭐⭐⭐ **O RESULTADO, medido nos dois estados da porta** (arte do dono, `--release`, `load 5,24`):
>
> | `PH2D_SKIN_GPU` | `attach_skin_meshes` | % de um quadro |
> |---|---:|---:|
> | `1` (omissão — a placa posa) | **`2,077 ms`** | **`12,5 %`** |
> | `0` (a CPU posa) | `5,939 ms` | `35,6 %` |
>
> ⭐ E o gate de PIXEL sobre um adaptador real lê **`0 px`** de diferença entre a placa e a CPU, de
> `1150` pintados, com o controlo (a mesma malha **sem** posar) a `1250 px`.
>
> ⛔⛔⛔ **A LEI NÃO É UMA MISTURA LINEAR, E O PORTÃO DE PARIDADE APANHOU-O ANTES DE SHIPAR.** O
> desenho nasceu de um cabeçalho de 2026-09-17 que a descrevia assim. Em **2026-09-19** a CPU deixou
> de a usar: o [`Skin::blend`] passou a **rodar em torno da JUNTA** para curar o entalhe do cotovelo
> (`p' = R(θ̄)·(p − c) + Σ wᵢMᵢ(c)`, com `θ̄` a média em CÍRCULO), e a linear ficou como
> [`Skin::blend_linear`], o CONTROLO. A 1.ª redacção do shader implementou a **antiga** e o gate leu
> **`2,315e-3 m`**.
>
> ⚠️⚠️ **A dívida estava NOMEADA no repo** — o `skin_gpu_tests` tinha um gate a AFIRMAR que as duas
> leis diferem, escrito no dia em que a CPU mudou — **e eu não a li antes de desenhar**. *Uma
> paridade medida contra a lei ERRADA teria shipado o defeito com um gate verde por cima.*
>
> ⭐⭐ **O payload é DERIVADO da crate que implementa a lei:** `Skin::tabela_de_juntas` e
> `Skin::angulos_das_poses` são portas novas, porque o `junta` é privado **de propósito** e um
> produtor que o reimplementasse seria a segunda resposta a *«por onde estes dois ossos se
> encontram»*.
>
> ⚠️ **A conjugação para o espaço do quad passou a ter DUAS metades:** o AFIM (que já existia) e o
> **PONTO** (a junta é um SÍTIO, logo a translação entra — conjugá-la como direcção punha o centro
> de rotação no sítio errado em toda sprite cuja âncora não fosse a origem, *e só nessas*). ⛔ E a
> rotação `R(θ̄)` **não pode** chegar conjugada, porque o `θ̄` só nasce da mistura dos ângulos, já
> dentro do shader ⇒ ele reconstrói `S⁻¹RS` à mão com as duas razões do `size`, com gate a provar
> que a reconstrução é a mesma conjugação.
>
> ⭐⭐ **E o GÉMEO ÓRFÃO foi APAGADO:** o `skin_gpu` de 2026-09-17 (payload + lei de referência)
> tinha **ZERO consumidores de produto** e a lei ANTIGA lá dentro. Manter os dois seria deixar uma
> lei errada viva onde o próximo agente a pode ligar. ⭐ As duas propriedades dele que sobrevivem
> passaram a medir-se contra o payload VIVO, e a primeira ficou **MAIS FORTE** (a tabela de JUNTAS
> entra nela) — *uma propriedade que se herda tem de cobrir o que foi acrescentado depois dela.*
>
> ⚠️ **Três armadilhas de ferramenta, todas já escritas neste repo e todas pagas outra vez:**
> `meta` é palavra **RESERVADA** em WGSL (quem o apanhou foi o gate que compila o shader) · um
> `--exact it::sprite_mesh_gpu::…` casou **ZERO** testes e imprimiu `ok` (o prefixo do binário não
> entra no nome) · e o guarda de recursos recusa um heredoc cujo corpo contenha `cargo test`.
>
> **Mutação: 13 de 13 sangram**, e **DUAS sobreviveram primeiro**, as duas réguas minhas: a fixtura
> de **DOIS** ossos tem **um** par só, logo `Σ wᵢwⱼ·junta / Σ wᵢwⱼ` devolve a junta seja qual for o
> peso (⇒ fixtura de **TRÊS**, com todo vértice governado pelos três); e a cerca do `valida` sobre a
> tabela de juntas não partia nada, embora o defeito que ela deixa passar seja o pior da família —
> *sem a tabela o centro cai para `None` e a lei degenera na LINEAR, em silêncio*.
>
> ⏳ **O que sobra, com o número ao lado:** os `12,5 %` **não são deformação** — são a tabela de
> pesos a ser derivada por vértice a cada construção de malha. ⭐ Ela é uma grandeza do **BIND** (a
> quota sai da posição de REPOUSO), logo o **memo do payload** é o que a tira do quadro. ⏳ E a
> truncagem a `K = 4` passou a tocar o **CENTRO** e não só os pesos: num vértice com `≤ 4` ossos —
> a arte do dono, `3` tendões — é a mesma soma ao bit; acima disso é aproximação **declarada**.
>
> ---
>
> ### O registo de 2026-09-17, que a medição acima reabriu

> ⛔⛔⛔ **LEIA ISTO ANTES DE TUDO O QUE VEM ABAIXO (2026-09-17): A PREMISSA DESTA FILA ESTÁ
> REFUTADA POR MEDIÇÃO.** Report do dono, depois de a porta abrir: ***«Como eu já havia dito muitas
> vezes: Fast e Smooth estão sempre idênticos. Nada mudou»***.
>
> Ele tem razão, e o número é este: medida a distância **em pixels de ECRÃ** entre o sítio onde o
> `Fast` põe cada texto da arte e o sítio onde o `Smooth` o põe, na dobra que a cena ship (`25°`) e
> no zoom `1`, ela é **`0,04 px` na mediana e `0,34 px` no pior ponto**. E o controlo diz o resto:
> o `Fast` está a **`0,33 px`** do campo VERDADEIRO (uma malha `64×` mais fina). *Nenhum olho
> distingue um terço de pixel* — as duas desenham o mesmo.
>
> | dobra/junta | `Fast × Smooth` pior | mediana | `Fast × campo` pior |
> |---:|---:|---:|---:|
> | `25°` (a da cena) | `0,335` | `0,044` | `0,334` |
> | `60°` | `0,775` | `0,100` | `0,771` |
> | `90°` | `1,095` | `0,144` | `1,090` |
> | `150°` | `1,496` | `0,205` | `1,489` |
>
> ⛔⛔ **A premissa do botão MORREU e ninguém reconferiu.** Ele nasceu do report de 2026-09-10
> (*«arestas retas ao dobrar»*), quando a malha do bind era uma **grelha uniforme** e os pesos eram
> **euclidianos**. As duas waves seguintes — a **grelha graduada pelas articulações** (10/09) e os
> pesos do **padrão-ouro com a lei de Hermite** (16/09) — curaram a faceta **na própria malha do
> bind**. ⇒ o `Fast` passou a estar certo e o `Smooth` ficou sem nada para corrigir. *§0.0: quem
> move o número que tornava algo inalcançável tem de reconferir a nota — aqui o número moveu-se por
> baixo de uma feature inteira, e a F9 foi construída em cima dela.*
>
> ⚠️ **O que a F9 construiu continua CERTO e continua a não ser visível:** a malha assada erra
> `2,3×` menos que a do bind, e as duas erram menos de meio pixel. *Uma cura de uma grandeza que já
> estava abaixo do limiar do olho não muda nada no ecrã.*
>
> ⚠️⚠️ **E TODAS as réguas desta linha mediam a grandeza errada** — o desvio ao campo em pixels da
> ARTE, que é uma propriedade da aproximação. O dono vê **pixels de ECRÃ**. O gate que fixa isto é
> `o_fast_ja_desenha_o_campo_a_menos_de_meio_pixel` (`ph2d-app-vec`), com as duas metades: o `Fast`
> está certo **e** o `Smooth` separa-se num regime real (`150°` com zoom `8`), que é o que impede
> alguém de ler isto como *«apague o botão»* — essa é decisão do dono.
>
> ✅ **O DONO DECIDIU no mesmo dia: *«1- Pode apagar a seção deform. 2- Escolha o melhor a fazer»*.**
>
> **(1)** A fileira foi **APAGADA**, e com ela o `SkinDeform` inteiro — enum, campo, as duas rotas de
> clique, os dois ids, os dois espelhos da shell, as três chaves de texto e os dois gates de costura.
> ⚠️ *Retirar o gesto retira a CAPACIDADE:* deixar a lei viva e inalcançável é o defeito que este
> repo já pagou, e por isso ela não ficou a dormir. A fileira do painel dá lugar a um bloco que diz
> **porque** ela saiu, com a medição ao lado.
>
> **(2)** A lei que fica é **sempre a malha ASSADA no bind** (`ph2d_skeleton_live::skin_bake_cache`),
> e a escolha é medida nas duas colunas: ela erra o campo **menos** que a malha crua **e** custa
> menos (`2,7 ×` mais peças por `5,6 ×` menos relógio — refinar `~0,32 µs`/peça contra desenhar uma
> peça já fina, `~0,017 µs`). ⇒ morreram com ela o repartir do orçamento e o aviso de malha acima
> dele: os dois existiam para governar um refinamento **por quadro** que já não acontece.
>
> ⏸️ **E a F9 PÁRA aqui, com o gatilho escrito.** O que sobrava dela era a metade 2 (a deformação no
> *vertex shader*), e ela **não compra um pixel**: o ganho medido é de RELÓGIO, `~11 %` de um quadro
> a 8 imagens presas. *Não se gasta uma wave a comprar 11 % de um quadro que hoje sobra.*
>
> ⏳ **O gatilho para a reabrir** (qualquer um dos três, e todos são MEDIÇÕES, não palpites):
> uma cena do dono onde a pele passe do orçamento de peças e o log (`PH2D_BONE_LOG=1`) o mostre ·
> um report de engasgo cuja sonda aponte para o `attach_skin_meshes` · ou a arte presa passar de
> `~8` imagens por cena. O desenho está escrito abaixo e continua válido — ⚠️ com **uma** correcção
> já medida: os `@location` 0..15 do *vertex* estão CHEIOS, logo os pesos têm de chegar por
> *storage buffer* indexado pelo `@builtin(vertex_index)`, nunca por um atributo novo.


> Perguntado *«para ele alisar em qualquer cena a deformação teria de passar para a placa de vídeo —
> quer que isso entre na fila?»*, o dono respondeu: ***«Quero que isso entre na fila!»***

**O problema, medido (F6-t):** hoje a CPU deforma cada vértice de cada imagem presa a cada quadro, e
o `Smooth` refina a malha na CPU dentro de um orçamento de `5 144` peças por quadro (`1/10` de um
quadro de 60 fps). Uma cena com mais arte presa que isso — um personagem de muitas partes — fica
com o `Smooth` **igual ao `Fast`** (agora de graça, mas sem alisar). Os números: avaliar uma peça
`0,156 µs`, cada peça nova `~0,32 µs`, o `Fast` `0,024 µs` por peça; a GPU desenha centenas de
milhares de triângulos num quadro sem esforço.

**A direcção (a confirmar pela medição da W0, nada disto está decidido em código):**

1. **A densidade sai do quadro e vai para o BIND.** A malha fina é assada uma vez, em repouso,
   onde o CAMPO DE PESOS curva (a mesma lei de Hermite do `Smooth`, medida contra os pesos e não
   contra uma pose) — e fica guardada. ⚠️ A pergunta a medir primeiro: uma malha fixa assada em
   repouso alisa a dobra FORTE como o refinamento por quadro alisa? (a régua existe: a silhueta e a
   faceta de `smoke_bone_paint_silhueta_tests.rs`, com as mesmas barras).
2. **A deformação vai para o *vertex shader*:** por vértice, os índices e pesos dos ossos (enviados
   quando a malha muda); por quadro, só as poses dos ossos (`N × 6` números por esqueleto). ⚠️ O
   `Skin` mistura poses RÍGIDAS por peso, e um B-Bone é `N` sub-ossos — as duas coisas cabem num
   *uniform/storage buffer* de poses, sem lei nova.
3. **A lei da CPU fica como REFERÊNCIA**, e a paridade CPU×GPU é um gate com a barra derivada do
   formato (o molde é o do Flip: `rgba16float` ⇒ `2⁻¹¹`; aqui, posições `f32` em pixels de ecrã).

**As costuras que a W0 tem de mapear antes de qualquer código** (quem lê a malha DESENHADA na CPU,
2026-09-16): o ponteiro (`ph2d_render::mesh_uv`), o `drawn_mesh_of`/`drawn_instance_of` (o anel do
Liquify e da Remoção de fundo, o `CanvasMap`, a caixa do gizmo, a tinta da protecção), os fantasmas
do onion e o `sprite_collect` (a tira do passe de sprites). ⛔ **Nenhum deles pode passar a ler a
malha GROSSA enquanto a GPU desenha a FINA** — seria o *«controlo desenhado por um mapa e agarrado
por outro»* que esta fila já pagou (F6-m). Cada um precisa de uma resposta: CPU da mesma malha fina
só onde se pergunta (um ponto, não a malha inteira), ou leitura da GPU.

**Ondas propostas:**
- **W0 — medir:** o custo e a qualidade da malha fina ASSADA contra o refinamento por quadro (na
  dobra de `25°`/`60°`/`150°`, zoom `1`–`16`); o custo GPU real de `10⁴`–`10⁶` triângulos
  deformados no *vertex shader* nesta máquina; e o censo das costuras acima.
- ✅ **W0 — FECHADA (2026-09-17). As três metades, e uma delas reescreveu a pergunta.**
  1. **A topologia assada serve todas as poses** — medido com CONTROLO (o bind é idêntico nas 5
     dobras, `< 1e-12`), assando no pior caso e re-posando em `5 × 4` células
     (`ph2d-app-vec/src/smoke_bone_paint_assada_tests.rs`, `18aca6a75`). *A direcção da F9 aguenta.*
  2. ⭐⭐⭐ **O CENSO DAS COSTURAS achou o facto que reescreve a W0-b: a malha JÁ vai para a placa
     todos os quadros.** Desde que a pele entrou no passe de sprites, o `renderer_draw` copia o
     `SpriteMesh` para um buffer e desenha — a CPU posa **e faz upload** de `N` vértices por quadro.
     ⇒ a F9 **não acrescenta** um desenho de `N` triângulos: ela TIRA da CPU a deformação por
     vértice e o upload, trocando-o por `N_ossos × 6` números. ⛔ A prosa desta fila listava os
     leitores e a lista estava **incompleta** (faltava a grelha da folha de quadros) — hoje são
     **10**, cada um com a espécie de resposta que vai precisar (**UM PONTO** `O(1)` na CPU · a
     **MALHA** inteira), derivados por
     [`architecture_who_reads_the_posed_skin_mesh`](../../crates/ph2d-editor-core/tests/it/architecture_who_reads_the_posed_skin_mesh.rs)
     — *um leitor novo reprova ali, e não no dia do smoke*.
  3. **O tecto do passe REAL** (`ph2d-render/tests/it/skin_mesh_gpu_ceiling.rs`, `--release`,
     offscreen, mínimo de 5, ⚠️ **`load 7,53`** ⇒ a coluna do relógio pede re-leitura abaixo de `5`):

     | triângulos | upload/quadro | quadro | de `16,67 ms` |
     |---:|---:|---:|---:|
     | `10 082` | `199 KiB` | `0,11 ms` | `0,7 %` |
     | `100 352` | `1,92 MiB` | `0,73 ms` | `4,4 %` |
     | `999 698` | `19,1 MiB` | `10,25 ms` | `61,5 %` |
     | `3 998 792` | `76,3 MiB` | `52,51 ms` | `315 %` |

     ⇒ **o desenho NÃO é o tecto.** O orçamento de hoje (`SKIN_FRAME_PIECES = 8 738` ⇒ `~17 k`
     triângulos) custa à placa `~0,2 %` de um quadro, e `100 k` custam `4,4 %` — **6×** o orçamento
     actual com folga. Quem tem o tecto é a CPU (F6-t: `0,156 µs` para avaliar uma peça, `~0,32 µs`
     por peça nova ⇒ `50 k` peças ≈ `7,8 ms`), que é exactamente o que a F9 remove.
- ✅ **W1 — FECHADA (2026-09-17). ⚠️ Ela fechou com a porta DESLIGADA e o número que dizia que ela
  tinha de ficar assim; a medição do custo por quadro REFUTOU esse número no mesmo dia e a porta
  ABRIU — ver a W2c abaixo.**
  - **A porta**: [`ph2d_poly2d::refine_rest_by_attrs`] refina a malha de **repouso** onde o campo de
    atributos curva (`Σ_j |w_j(meio) − w̄_j|`), sem pose nenhuma; e
    [`ph2d_skeleton_live::skin_bake::assar`] liga-a aos pesos BBW do bind. `PH2D_SKIN_BAKE=1` abre.
  - ⭐⭐⭐ **A conta que sustenta a F9 está escrita e CORRIDA:** com ossos afins,
    `P(meio) − corda = Σ_j Δw_j · T_j(meio)` ⇒ *a única coisa não-linear numa aresta é o PESO*, e
    assar com tolerância `τ` garante `|erro| ≤ τ · dispersão` em **toda** pose. É o mecanismo por
    trás do que a W0 mediu.
  - ⛔⛔ **A tolerância que eu tinha escrito era INERTE, e foi a arte REAL que o disse:** `0,02`
    saía de uma fixtura sintética, e na cena do dono o pior desvio de peso de toda aresta já é
    **`0,0154`** — a malha do bind **já é graduada pelas articulações** (wave de 10/09), logo a
    densidade já está onde o campo vira. ⇒ a tolerância passa a ser **DERIVADA**
    (`0,5 px / diagonal da arte`), que é a mesma barra que o `Smooth` do quadro promete.
  - **Medido na arte do dono** (`512 × 320`, `2 430` peças, BBW por 3 ossos), com `τ = 8,3e-4`:

    | desenho | peças | desvio ao CAMPO |
    |---|---:|---:|
    | `Fast` (o bind de hoje) | `2 430` | `0,4143 px` |
    | `Smooth` (do quadro, zoom `8×`) | — | `0,0881 px` |
    | **assada** | **`13 996`** (`5,76×`) | **`0,1781 px`** |

    ⇒ a assada erra `2,3×` menos que o `Fast` e fica dentro da barra de `0,5 px`. ⚠️ Ela erra `2×`
    mais que o `Smooth` **por desenho**: aquele refina para ESTA pose e este zoom, e a assada é
    independente da pose — *uma aproximação que serve todas nunca bate, peça a peça, uma feita para
    uma só*.
  - ⛔⛔ **E a frase que estava aqui — *«é o `5,76×` que PROVA que a porta fica fechada até à W2»* —
    foi REFUTADA no mesmo dia (W2c):** o `SKIN_FRAME_PIECES` é um tecto de **REFINAMENTO**, e uma
    malha já assada **não refina**. *Comparar uma contagem de peças com um orçamento cuja unidade é
    «peças que a lei pode PARTIR» é somar duas grandezas diferentes* — e o resultado dessa soma
    mandava fechar a porta que a medição mandou abrir. Na placa, `13 000` triângulos custam
    `~0,15 %` de um quadro (a tabela da W0-b). **A assadura não é cara; caro é deformá-la na CPU —
    e mesmo isso cabe (`1,4 %` numa imagem).**
  - ⚠️⚠️ **A régua da silhueta NÃO serve para comparar densidades diferentes** (a fila pedia-a, e a
    medição refutou o pedido): o «vai-e-volta» soma a viragem absoluta da polilinha, logo **cresce
    com o número de nós por construção** (`Fast` `26,60°` com 46 nós · `Smooth` `26,71°` com 64 ·
    assada `29,44°` com 85). A régua com unidade e barra declarada é o **desvio ao campo**.
- ⏳ **W2 — o *vertex shader* de pele**, atrás da mesma escolha `Fast`/`Smooth` do painel, com o gate
  de paridade CPU×GPU e o caminho da CPU vivo para bissecar.
  - ✅ **A metade da CPU FECHOU (2026-09-17)** — [`ph2d_skeleton_live::skin_gpu`]: o empacotamento e
    a **lei de referência** (`posa_como_a_placa`), que é o que **define** o shader e contra o que a
    paridade se vai medir. Ela reproduz a lei do produto a **`1,4e-5`** contra uma barra derivada de
    `4 ULP` de `f32` na magnitude em jogo (`7,6e-5`), com o controlo dentro (poses erradas violam-na
    `100×`).
  - ⭐⭐⭐ **E o achado que torna o shader TRIVIAL:** a quota que reparte o peso de um tendão pelos
    sub-ossos de um osso que dobra depende de `u = projecção do ponto no eixo de REPOUSO` ⇒ ela é
    uma grandeza do **BIND**. Logo a tabela de pesos **por osso, já normalizada**, só muda quando a
    TOPOLOGIA do rig muda — nunca quando o artista posa. ⇒ *o shader não precisa de saber o que é
    um osso que dobra*: ele lê `N` pesos por vértice e `N` afins por quadro, e a mistura é a linear
    clássica. Gate `mover_um_osso_nao_muda_a_tabela_de_pesos`.
  - ⚠️⚠️ **E a 1.ª fixtura destes gates tinha a corrente toda RECTA — MEDIDO, ela deixa a mutação
    que apaga a quota passar em TODOS os três gates.** Com um osso que dobra, ela sangra. *Uma
    fixtura no ponto neutro de uma lei não testa essa lei.*
  - ✅ **A W2b FECHOU: a malha assada é DERIVADA, e o painel continua a escolher** (2026-09-17) —
    [`ph2d_skeleton_live::skin_bake_cache`]. A W1b assava **dentro do `bind_image`**, substituindo a
    malha guardada, e isso é de PRODUTO e não de relógio: ⛔ o `Fast` deixava de ser barato (passava
    a desenhar a malha `5,76×` maior), a escolha `Fast`/`Smooth` **colapsava** (as duas desenham a
    mesma malha) e a densidade ficava **congelada no ficheiro**. ⇒ a assadura sai do documento e
    passa a viver num **memo por bind**; o `Smooth` consulta-o, o `Fast` não passa por lá.
    ⭐⭐ **É o mesmo memo que a placa vai querer** — quando o *vertex shader* posar, o que sobe uma
    vez por bind é exactamente esta malha (repouso + tabela de pesos). *A casa é a mesma; muda quem
    a lê.*
    - ⚠️ **A chave é a ENTIDADE e a prova é o CONTEÚDO:** `Entity::to_bits()` é só o ENDEREÇO da
      gaveta (o degrau 122 da escada já escreveu porque ele não serve como identidade durável), e
      quem diz se o conteúdo serve é a **igualdade byte a byte** da fonte. ⛔ Uma função de dispersão
      criptográfica seria **dez vezes mais cara** que a prova exacta (`~100 KiB` de bind: memcmp
      `~10 µs` contra SipHash `~100 µs`) — *uma chave derivada só compensa quando comparar o
      original é caro.*
    - ⚠️ **O `None` também é guardado** — com a porta fechada ele é a resposta de toda a arte, e sem
      o guardar o caminho de omissão pagaria uma tentativa por imagem por QUADRO.
    - ⚠️⚠️ **O aviso de orçamento partiu-se em DOIS, porque a mesma condição passou a ter
      significados OPOSTOS:** sem assadura ela é um AVISO (*o botão que o painel diz ligado desenha
      o que o `Fast` desenha*); com assadura ela é a wave a **funcionar** (a densidade veio do bind,
      e não haver refinamento por quadro é o que a torna independente do tamanho da cena).
    - ⛔⛔ **Duas mutações SOBREVIVERAM primeiro, as duas a acusar código meu:** o `filter` que
      protegia a gaveta recém-assada do despejo era **inerte** (com `visto = agora` ela nunca pode
      ser o mínimo) — *uma linha que a mutação não consegue matar não é lei, é comentário com
      sintaxe de código* —, e o refresco do relógio no ACERTO não tinha régua nenhuma, logo o memo
      era **um FIFO com o nome de cache**. ⚠️ E a mutação que morde a primeira só é observável num
      gate cuja ordem de ENTRADA discorda da ordem dos BITS, que é o caso normal.
    - ⚠️ **A premissa de um gate MORREU e ele foi reescrito com a morte visível no diff:**
      `o_bind_da_imagem_chama_o_assador` afirmava o CONTRÁRIO do que hoje é verdade ⇒
      `o_assador_tem_um_chamador_e_ele_nao_e_o_bind`, com as duas metades.
    - **7 gates · 7 mutações, todas sangram.**
  - ✅⭐⭐⭐ **A W2c FECHOU, e é ela que responde ao pedido do dono: A PORTA ABRIU** (2026-09-17) —
    `PH2D_SKIN_BAKE=0` passa a ser a porta de **bissecar**, e o caminho de omissão do `Smooth` é a
    malha ASSADA. **Medido na arte do dono** (zoom `8×`, `N` cópias, o MÍNIMO de 30, `load 4,6`):

    | imagens | lei | porta | peças entregues | ms | % de um quadro |
    |---:|---|---|---:|---:|---:|
    | 1 | `Fast` | — | `2 430` | `0,056` | `0,3 %` |
    | 1 | `Smooth` | **fechada** | `5 143` | `1,244` | `7,5 %` |
    | 1 | `Smooth` | **aberta** | **`13 996`** | **`0,226`** | **`1,4 %`** |
    | 4 | `Smooth` | **fechada** | `9 720` ⇐ **é o `Fast`** | `0,228` | `1,4 %` |
    | 4 | `Smooth` | **aberta** | `55 984` | `0,903` | `5,4 %` |
    | 8 | `Smooth` | **fechada** | `19 440` ⇐ **é o `Fast`** | `0,459` | `2,8 %` |
    | 8 | `Smooth` | **aberta** | `111 968` | `1,824` | `10,9 %` |

    ⭐⭐⭐ **Numa imagem a assadura é `5,5×` MAIS BARATA e entrega `2,7×` MAIS peças** — *refinar* uma
    peça custa `~0,32 µs` e *desenhar* uma peça já fina custa `~0,017 µs` (números da F6-t, que
    ninguém tinha composto). ⛔⛔ **E as linhas de `4` e `8` com a porta fechada são o report do dono
    reproduzido ao número:** `peças(Smooth) == peças(Fast)`.
    - **O gate que é a F9 numa asserção:** `o_smooth_alisa_em_qualquer_cena` — *a malha assada é um
      CHÃO que o tamanho da cena não consegue erodir*, com as três metades (a cena **contém** o
      fenómeno · o `Smooth` entrega **estritamente** mais que o `Fast` · e entrega pelo menos o
      chão, senão ele degrada em vez de sumir). ⚠️ O chão sai da **lei do produto**, nunca de um
      número escrito no gate — a tolerância é derivada da diagonal da arte. **3 mutações, todas
      sangram** (a porta fechada · o `Smooth` sem consultar o memo · a tolerância de volta ao `0,02`
      que a arte real já tinha refutado).
    - ⚠️ **Assar custa `3,9 ms`, UMA vez por bind**, ao lado do solver BBW que o mesmo `bind_image`
      já paga, e **fora** do quadro.
    - ⚠️ **DUAS premissas morreram com a morte visível no diff:** a porta nascer desligada, e o
      *«sem espaço no orçamento o `Smooth` desenha o `Fast` AO BIT»* — hoje ele desenha a **assada**,
      que é o ponto.
  - ✅⭐⭐ **E A CENA PARA SE VER ISSO EXISTE** (2026-09-17): `PH2D_VEC_BONE_PAINT_SMOKE` deixou de
    ser um interruptor e o nível dele é uma **CONTAGEM de canvas** — `=1` é a cena de 8 passos que o
    dono já aprovou, **byte-idêntica**; `=3` ou mais põe a cena acima do orçamento do quadro, que é
    o regime do report. ⚠️ *Sem ela a cura estava gateada e invisível, e uma cura que ninguém pode
    ver é uma cura que ninguém julga.*
    - ⛔⛔ **A FOTO (`fotografa_cena.sh`) apanhou DOIS defeitos de cena que gate nenhum via:** a
      1.ª disposição era uma COLUNA e a arte dobrada **varre para cima** muito além da caixa de
      repouso, logo o canvas de cima ficava sempre cortado — *nenhum valor do espaçamento serve,
      logo o que estava errado era a disposição* (hoje é uma FILEIRA: dobrar **encurta** a pegada
      horizontal); e o enquadramento `All`, que eu tinha posto para caber a cena inteira, ajusta-se
      às CAIXAS das sprites e cortava as pontas de qualquer maneira. ⇒ volta ao `Selected`, e a
      leitura muda com ele: *os outros canvas existem para ENCHER o orçamento, não para serem vistos
      ao mesmo tempo.*
    - ⚠️ E o roteiro passou a ser **outro** conforme a contagem: *«um roteiro que tenta ensinar as
      duas coisas manda o dono fazer oito passos para chegar ao que ele foi ver».*
  - ⏳ **O que falta da W2 (a placa), e o que já está medido sobre isso:**
    - o **formato de vértice**: o [`ph2d_render::QuadVertex`] é **partilhado com o quad simples**
      (`pos` + `uv`, 16 bytes), logo acrescentar-lhe pesos paga em toda sprite do app ⇒ ou um
      segundo *layout*/pipeline, ou um buffer à parte indexado pelo vértice. ⚠️ Medido na arte do
      dono: `3` tendões e **nenhum vértice esparso** (`139` vértices usam 1 osso, `662` usam 2,
      `487` usam 3) — *num rig pequeno não há esparsidade a explorar, e um `K = 4` fixo do formato
      da indústria seria um TECTO a justificar, não um ganho*;
    - o **buffer por-BIND com invalidação DO LADO DA PLACA** (hoje o `MeshFrame` é reconstruído do
      zero a cada quadro) — é ele que troca o upload de `19 MiB/quadro` por `N × 6` números. ⭐ A
      metade da CPU já existe (a W2b); o que falta é o `MeshFrame` deixar de ser por-chamada;
    - ⛔ **e o formato NÃO pode crescer por atributo de vértice:** o `pipeline.rs` declara por
      escrito que *«o limite de 16 atributos do dispositivo (`@location` 0..15) está cheio»* — a
      `InstanceInput` ocupa `2..15` e o `QuadVertex` o `0..1`. ⇒ os pesos por vértice entram por
      **storage buffer** indexado pelo `@builtin(vertex_index)` (que numa chamada não-indexada é o
      índice ABSOLUTO no buffer, logo um vector paralelo ao dos vértices costurados resolve sem
      offset nenhum), e não por um atributo novo;
    - as **10 costuras** do censo da W0, cada uma com a espécie de resposta já escrita.
- **W3 — as costuras** (ponteiro, chrome, onion) contra a malha que a GPU desenha.
- **W4 — o orçamento**: ele deixa de ser um tecto de peças da CPU; o que sobra de CPU por quadro é
  enviar poses, e o recurso passa a ser memória de GPU (com o número medido ao lado).

**⛔ Não é:** subir o `SKIN_FRAME_PIECES` (o recurso dele é o tempo da CPU, e está medido) nem
refinar em *compute shader* por quadro sem primeiro medir a malha assada.

---

## ⛔ Recusas MEDIDAS deste módulo — não as reconstrua

> ⚠️ **As seis de 2026-09-07/08 entraram aqui na auditoria de 08/09** — elas viviam só em prosa e em
> doc-comments, e o §5.0 é explícito: *arquivar sem indexar as recusas seria apagá-las.*

| recusa | o mecanismo MEDIDO |
|---|---|
| **Pôr o `VecDrivenStyle` no ledger de pré-visualização** (o «quinto de cinco» da auditoria de 08/09) | Ele é **desregistado, e não por esquecimento** — não deriva `Serialize`, e o `register_default` exige-o, logo *uma linha de registo não compila*: ele nunca entra no snapshot, no save, nem num passo de undo, que é exactamente o que o ledger compra para os outros quatro. E ele **volta ao autorado TODO QUADRO** (`settle_to_authored`), contra o `release_to_authored`, que só corre quando um motor é desligado. ⇒ acrescentá-lo poria no memo um facto que nunca esteve na fotografia. |
| **Avisar em vez de COAGIR** o nome de um clip a ser único | Os dois nomes são igualmente válidos, então o artista fica com um documento que ele não consegue reparar renomeando — a forma de uma recusa com passos extra. A lei já estava escrita no `doc.rs` e honrada nas duas portas que INVENTAM um nome; faltavam as duas que o RECEBEM. |
| **Roubar `Body`/`Joint` do gizmo de sprite** ao alargar as alças de osso a todo modo de vector | Os dois verbos (girar · deslocar) **já existem** na seta, e agarrá-los aqui trocaria a lei do arrasto dela **em silêncio**: o artista escolhe um osso com a seta e o arrasto passa a fazer outra coisa. A linha é o VERBO — entram só os quatro que nenhuma outra ferramenta sabe exprimir. |
| **Reordenar as secções do painel** para a SKELETON subir quando tem sujeito | Cura o mesmo report que o *revelar-ao-focar* (o cabeçalho a `1316 px` sobre uma faixa de `900`) e muda a ordem do painel para **toda** ferramenta e todo objecto — é decisão de produto, não de correcção, e a revelação é a metade pequena e já precedentada pela timeline. |
| **O *pole target*** para escolher o lado do joelho | Em 3D o triângulo raiz–cotovelo–ponta roda em torno do eixo raiz→ponta — um **grau de liberdade contínuo**, que um objecto no espaço fixa. No plano sobra **UM BIT**. Godot (`flip_bend_direction: bool`) e Spine (`bendDirection ±1`) escolheram o interruptor, cada um por si. ⇒ o alvo de pólo resolveria com um objecto o que um booleano resolve. |
| **Priorizar a ordem no hit-test** para resolver a colisão alça↔ponta | *Não cura: só troca a vítima.* Medido: com o osso na parede a distância ponta→alça é `0,000000`, logo quem quer que ganhe a ordem, o outro fica inalcançável. A cura foi **afastar** a alça (folga derivada do dedo da casa). |
| **Adoptar o clip ABERTO** no *Add Smart Bone* | `TimelineDoc::new()` tem **um** clip, `"Main"` ⇒ todo controlo casava com a animação principal da cena, em silêncio. |
| **Criar uma acção com o nome do osso** no *Add Smart Bone* | Veredito do dono (*«porque criar Bone Action no inspector e na timeline? Melhor não criar nada»*): duas coisas fabricadas por um clique, nenhuma pedida. |
| **Herdar o encaminhamento** pendurando os ids da fileira do lado da dobra na `VECTOR_BONE_VERBS` | Reprovado pelo `table_driven_chips_are_registered_too`: ele exige que o `populate` itere a MESMA tabela que o `paint`, e sem esse laço a fileira seguinte nasce **morta sob o dedo**. |
| **Registar o chip do selector como `Button`** | Mutação medida: o clique **acende e nunca abre lista nenhuma** (`the_action_picker_lists_the_document…` fica vermelho em *«com a lista ABERTA a acção tem de ser pintada»*). É a cicatriz da swatch dos tokens e dos dois números do Input Map. |
| **A *quadtree* graduada** como malha da imagem (F6-b) | Ela deixa **nós pendurados** na transição entre níveis, e um nó pendurado abre **FENDA** numa deformação: ele move-se pelos pesos dele enquanto a aresta do vizinho grosso se move linearmente entre as pontas. Curá-los pede a tabela de moldes de transição (5 casos a menos de rotação). A **grelha-produto** entrega o mesmo adensamento e **CONFORMA por construção** — dois vizinhos partilham a aresta inteira, sempre. |
| **Guardar quadriláteros** em vez de dois triângulos (F6-b) | Um afim não leva um quadrilátero qualquer a outro qualquer: quatro pontos são **oito equações para seis incógnitas**. «Quadmesh» aqui é a DISPOSIÇÃO dos vértices, nunca o primitivo guardado. |
| **Escolher a diagonal da célula pela forma DEFORMADA** (a mais curta das duas — a resposta clássica) | A malha trocaria de diagonal a meio de um gesto ⇒ *o desenho pisca exactamente enquanto o artista dobra.* A diagonal `a–c` fixa-se no **repouso**. |
| **Dilatar cada recorte** para fechar as costuras da pele de imagem (F6-g, 2026-09-13) | Dilatar `0,5 px` (homotetia pelo incentro) fecha a costura em arte OPACA (`16 580 → 0` px com `216` peças) e, em arte TRANSLÚCIDA (alfa `128`), compõe a faixa sobreposta DUAS vezes: `10 580 → 40 050` px com alfa errado e o pior erro `20 → 111` (`3 456` peças: `41 732 → 158 046`). Sombras suaves, bordas anti-aliased e brilhos são translúcidos ⇒ a cura estraga mais do que conserta. Sonda `ph2d-render::skin_pieces_gpu_cost::measure_seams_against_clip_dilation`. A cura que resta é rasterizar a malha SEM AA nas arestas internas. |
| **O sinal de cada junta como restrição DENTRO das varreduras do FABRIK** (a forma clássica; F5-c, 2026-09-14) | **Oscila.** A ida prega a ponta no alvo e re-resolve a corrente inteira sem olhar aos sinais; a correcção desfaz isso. Medido: o erro da ponta **cresce** passagem a passagem em 2 dos 12 alvos do zig-zag (`0,52 → 1,58` num alcance de `3`; `1,87 → 2,00` num de `5`) — e os dois **têm pose exacta**, achada por busca cega sobre os ângulos com os sinais como restrição. A cura é outro solver (descida junta a junta), não outra projecção. |
| **Deitar a junta violada na FRONTEIRA** (a projecção de norma mínima) | Ela move aquela junta o mínimo e custa à CORRENTE o máximo: desfaz a **dobra**, e uma ponta que só se alcança dobrando deixa de se alcançar — `2,43` de erro num alvo a `1,52` de uma corrente de alcance `4`, que tem pose exacta com aqueles sinais. |
| **Amortecer entre a fronteira e o espelho** (`λ · ângulo`, varrido em `0,0 · 0,2 · 0,4 · 0,5 · 0,6 · 0,8 · 1,0`) | Nenhum valor resolve os dois alvos teimosos, e os intermédios são **piores que qualquer um dos extremos** (a `λ = 0,5` três alvos que o `λ = 1` resolve ao bit passam a errar `0,60`–`1,34`). *Não é afinação — é o laço.* |
| **`livre ± 2π` entre os candidatos** do passo do misto | Código defensivo **sem consumidor**: nunca venceu em `900` fixturas, e não pode vencer — a pose de partida é feita dos sinais que dela se leram, logo cada ângulo já está dentro da sua parede e o intervalo vive inteiro dentro de `(−π, π)`, onde o candidato do interior também vive. |
| **Traçar a linha do `IK Chain` pela POLILINHA das juntas** (F5-d, 2026-09-14) | Numa corrente quase esticada ela cai **exactamente** sobre os corpos dos ossos e lê-se como parte deles; numa dobrada, serpenteia. O que o controlo tem de dizer é uma EXTENSÃO, e uma extensão desenha-se como cota: recta e deslocada. |
| **DESLOCAR a recta da corrente para o lado livre** (F5-e, veredito do dono) | A folga contra os ossos em toda pose custa as PONTAS: ela deixa de tocar a junta onde o `Chain` pára e o losango do alvo, e um indicador de extensão que não encosta nas pontas não diz qual extensão é. A corda passa por fora do arco sozinha; em pose esticada a folga vem de a linha ser **fina**. |
| **Deslocar a recta da corrente por uma CONSTANTE** | Não limpa uma corrente que se enrola mais de meia volta: ela tem bojo dos DOIS lados e vem por trás da recta (medido: `18,39 px` de um osso que ocupa `18,75`). O afastamento tem de passar por fora da **excursão** do lado escolhido. |
| **Portar os pesos do Godot** (F6-k, 2026-09-14) | Não há nada para portar: ele **não os calcula**. Só `get/set_bone_weights` e uma acção de painel — *Paint Bone Weights*. |
| **Adoptar os pesos automáticos do Blender** (difusão de calor) | Na nossa fixtura eles dobram `5`–`9 %` da arte acima de `60°`, contra `0,65`–`2,3 %` dos nossos e `0 %` do alcance curado. No nosso meio (folha plana, ossos no plano dela) eles degeneram numa **partição dura** — o método é de outro meio. |
| ⭐⭐⭐ **Os CENTROS DE ROTAÇÃO optimizados** (Le & Hodgins 2016), **re-medidos em 2026-09-15 sobre pesos NÃO degenerados** | ⚠️ **A recusa de 14/09 era *«não dá para julgar por cima de pesos degenerados»* — e essa premissa DISSOLVEU-SE** (arte rígida `37,6 % → 6,6 %`). Re-medido, ele perde na mesma: `11,70 %` de dobra a `90°` contra `8,27 %` do LBS. ⭐ **A causa é OUTRA e é do MEIO:** a nossa arte é uma **folha plana com os ossos a correr pelo meio**, logo o campo de pesos é **simétrico em `y`** — medido, `1 176` pares espelhados com diferença de peso `6,7e-4`. A semelhança do artigo é função **dos pesos e de mais nada** ⇒ não distingue os dois lados, e o centro de ambos cai **no eixo** (`\|y\|` médio `0,0003` numa arte que vai de `−2,4` a `+2,4`; `2 074` de `2 243` vértices a mais de `0,5` do próprio centro). É a limitação que o próprio artigo nomeia — aqui ela **é a forma normal da nossa arte**. ⛔ **Nenhum `σ` cura**: dois pontos com o mesmo vector de peso são indistinguíveis para qualquer função deles. Bancada: `ph2d-skin-weights/src/bancada_centros.rs`. |
| ⭐⭐ **A LEI DO MEIO que as duas recusas acima partilham** | ⛔ *Duas técnicas de topo do campo (difusão de calor · centros de rotação) foram recusadas pela MESMA propriedade da nossa geometria:* uma folha plana com os ossos no plano dela. Qualquer candidata que dependa de **distinguir pontos pelos PESOS** falha aqui, porque os dois lados da folha têm o mesmo vector de peso. ⇒ *sabe-se antes de construir*, e é isso que esta linha vale. |
| **Apertar os pesos** para curar a dobra da pele (F6-j, 2026-09-14) — ⚠️ **a recusa vale; o NÚMERO dela é da lei ANTIGA** (ver a linha seguinte) | **Piora, e é a resposta intuitiva:** `0,25 ×` do osso dá `0,64 %` de arte invertida contra `0,17 %` do alcance de hoje. O que dobra a arte é o **gradiente** dos pesos — apertá-los torna-o mais íngreme. Quem cura é ALARGAR: a `2,08 ×` a meia-altura da arte são **zero** pontos invertidos até `150°`. |
| **SUBDIVIDIR o osso (o mecanismo do B-Bone) como cura da dobra** | Com a população de amostras constante ele **piora**: `2,61 % → 4,94 %` a `24` sub-ossos. Com o alcance já certo não cura nada — compra **margem** (`det_min` `0,013 → 0,367`). ⇒ a ordem é o alcance primeiro. ⛔ E a variante «raio encolhe com o sub-osso» lê `0 %` invertido a **`94,5 %` de amostras órfãs**: é a régua a não medir nada. |
| ⭐⭐ **O `smoothstep` no peso entre dois nós** (`lerp(ra, rb, 3t²−2t³)`, F35, 2026-09-19) | Ele torna a derivada do peso **nula nos dois nós**, o que faz a curva-ALVO deixar de quebrar ali por construção — e **não chega**: corta a quebra do alvo a meio (`p50 9,05° → 3,98°`) e deixa o **máximo** em `26,98°` contra `28,62°`, porque o resto da quebra é do AJUSTE e não do alvo. Com a conciliação das alças por cima ele não muda a quebra (já é `0,000°`) e **piora** o desvio à verdade (`0,03049 → 0,03459`). *Uma segunda lei que não move a régua da primeira não entra.* |
| ⭐⭐ **A CASCATA da tangente** (`C''`, depois a corda) para dar eixo a uma alça degenerada (F35) | Construída e **removida**: nos nós em que ela era lida a **outra** metade do nó tinha comprimento zero e a `reconcilia` saltava-os na mesma ⇒ **nenhuma mutação a conseguia matar**. *Uma linha que a mutação não mata não é lei.* ⚠️ E a variante óbvia — tirar o eixo da cúbica **já deformada** — é pior que inerte: aquele vector mistura DOIS nós, logo as duas pontas de uma aresta recta recebiam a **mesma recta** e o segmento não conseguia arquear. |
| ⭐⭐⭐ **A correcção das alças PRESA à direcção de cada uma** (o primeiro desenho da F35) | Ela dá quebra `0,000°` e **mata a F30**: numa aresta recta as duas pontas ficam sobre a mesma linha e o segmento **não arqueia** ⇒ pintar peso no meio de uma aresta volta a mover `0,000000`. Medido, o desvio à verdade também é pior que o da conciliação (`0,03049` contra `0,02451` a `120°`). *A tangente tem de poder RODAR — desde que os dois lados rodem juntos.* |
| ⭐⭐⭐ **ARAP onde a lei esmaga** (*As-Rigid-As-Possible*, Sorkine–Alexa 2007 — a família do Plastic do OpenToonz; F38, 2026-09-29) | Sobre a malha do domínio, os vértices com `det J < τ` soltos e a energia ARAP a decidir: desfaz os triângulos virados e **CRIA laços a `90°`** — ela resiste à compressão, e o lado de DENTRO de um cotovelo TEM de comprimir. |
| ⭐⭐⭐ **A menor correcção sem inversão** (barreira sobre `det J`, `χ(d,ε) = (d+√(ε²+d²))/2`, à la IPC / Garanzha 2021; F38) | **Zero** triângulos virados em toda a dobra (`10 → 0` a `S 100°`, `49 → 0` a `C 150°`, `18`–`25 ms`) **e o contorno continua a cruzar-se** (`2 → 2`) ⇒ *o defeito não é a pele virar do avesso, é o CONTACTO*. Acrescentar uma barreira sobre o ângulo da BORDA (a soma passa de `360°`) **trava o optimizador**: colapsa triângulos a área zero. |
| **Ler os lados do modo MISTO da pose VIVA** | Estável enquanto o alvo está ao alcance (o modo é ponto fixo, e há gate) e **apagado para sempre** no primeiro arrasto que o leve para fora dele: fora do alcance a resposta certa é a RECTA, e uma recta não tem lado nenhum para ler. «Inicial» tem de ser o DOCUMENTO. |
| ⭐⭐⭐ **A DOBRA SOB A LEI DE PESO DE HOJE, RE-MEDIDA** (2026-09-18) — *não é uma recusa, é a reconferência que o §0.0 exige* | ⛔⛔ A recusa acima diz *«zero pontos invertidos até `150°`»* e mede a lei **derivada por distância**; o bind passou ao **padrão-ouro (BBW)** em **15/09** e **ninguém reconferiu**. Re-medida pela porta do produto sobre a arte do braço da cena `=2` (sonda `sonda_da_dobra`, `ph2d-skeleton-live`): a lei **continua de pé** — `0` triângulos do avesso a `0/13/25/50/75°` por junta, e a **primeira** inversão a `90°` (`19` de `3 593`), com a corrente dobrada por completo sobre si. Pior factor de área: `1,44 · 1,22 · 0,99 · 0,49 · 0,03 · −0,18`. ⇒ **gate** `a_pele_nao_vira_um_triangulo_ate_setenta_e_cinco_graus`, com o controlo positivo a `90°` dentro dele. ⛔⛔ **E uma nota de PRODUTO caiu junto:** a cena `=2` dobrava `13°` por um doc meu que dizia que a `25°` *«a malha dobra sobre si mesma e a arte lê-se RASGADA»* — **falso**; o rasgo da foto eram os gargalos da união dos quadros e o configurar-depois-de-prender, os dois curados na mesma jornada. *Baixar o ângulo fez o sintoma encolher, e por isso pareceu uma cura.* A cena volta a `25°`. |
| **Fazer a malha SEGUIR a silhueta** em vez de a cobrir (F6-b) | Traz de volta as células deformadas da borda, que são o defeito que a wave cura. O recorte fino é do **alfa da própria arte**, de graça e ao sub-pixel — o *Expansion* do *Puppet* do AE. |


| O quê | Por quê | Onde |
|---|---|---|
| Guardar os **pesos** numa tabela por ordem de varredura | é o *vector paralelo* que o `corner_radius` proíbe por escrito; derivar custa **0,146 %** de um quadro | [`ph2d-skeleton-ecs`](../../crates/ph2d-skeleton-ecs/src/lib.rs) |
| Referenciar um osso por `Entity::to_bits()` | **medido `0 de 2`**: o undo re-spawna e os bits são ids de alocação — a pele morria em silêncio; e o `from_bits` **aborta o processo** com bits de outra sessão | degrau 122 da escada |
| Misturar FK↔IK pelas **posições** das juntas | encurta os ossos (a corda é mais curta que o arco); a mistura é sobre o **ângulo** | `ph2d_skeleton::blend_angle` |
| Um `Driver` novo no `preview_drive` para a âncora | dois memos sobre o **mesmo componente** repõem `Transform`s diferentes na mesma fotografia, e quem ganha é a ordem do `BTreeMap` | `skeleton_goal` (cabeçalho) |
| `chain = 0` (*até à raiz*) como valor de nascimento | é o default do Blender e a queixa nº 1 documentada da feature dele: ao primeiro arrasto o esqueleto inteiro dobra | `DEFAULT_CHAIN = 2` |
