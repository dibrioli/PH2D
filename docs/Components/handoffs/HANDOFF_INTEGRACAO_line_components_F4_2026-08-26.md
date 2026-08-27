# Handoff de integração — `line/components`, 2026-08-26 (**F4.1..F4.5 + os três reports do smoke**: o mestre é inerte, instanciar/duplicar copiam de verdade, editar a receita muda as instâncias, a excepção do artista sobrevive — e o objeto vazio passa a ser agarrável)

> DIRETRIZ §1.5.9. Sucessor do
> [handoff de 25/08](HANDOFF_INTEGRACAO_line_components_F1_F2_F3_2026-08-25.md) (F1+F2+F3).
> ⚠️ **A fase F4 NÃO está fechada** — este handoff cobre as cinco primeiras fatias dela, que são
> mergeáveis isoladamente. O estado das sete fatias está no
> [plano vivo §F4](../05_plano_de_implementacao.md).

---

## §1 Identidade

| | |
|---|---|
| Branch | `line/components` |
| Worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-components` |
| Base | `main @ 0f5ce8040` |
| Governança | [ADR-0164](../../architecture/decisions/0164-instances-are-real-entities-linked-by-stableid-with-live-sync-and-incremental-undo.md) · [ADR-0166](../../architecture/decisions/0166-the-inspector-shows-what-the-object-has-and-components-attach-through-one-palette-filtered-by-object-type.md) |
| Fatias entregues | **F4.1**..**F4.5** · **os reports do smoke de 26/08** (§9–§12) · **F4.6a/b** (os documentos possuídos: clone e propagação por conteúdo — §13) |

---

## §2 O que MUDA para quem usa o app

| gesto | antes | **hoje** |
|---|---|---|
| duplicar um objeto na Hierarquia | copiava `Transform`+`Sprite`+`Name` e **nenhum filho** | ✅ a **subárvore inteira**, todo componente, identidade nova |
| duplicar um corpo com junta | a cópia ficava **solta** (a junta nomeava o original) | ✅ a junta da cópia prende **os corpos dela** |
| um objeto marcado como receita | simulava como qualquer outro | ✅ **não cai** — receita não é objeto de cena |
| editar uma peça da receita | não existia | ✅ **todas as instâncias mudam** no mesmo quadro |
| editar uma peça de UMA cópia | não existia | ✅ vira **excepção**: a receita já não a leva |
| transformar um objeto em componente | não existia | ✅ *Make Component* — a receita esconde-se e uma cópia fica no lugar |
| pôr outra cópia da receita | só pelo smoke | ✅ *Instantiate* no menu |
| promover a excepção a padrão | não existia | ✅ *Apply to Master* — as outras cópias recebem-na |
| soltar uma cópia da receita | não existia | ✅ *Detach from Master* |
| pegar um **objeto vazio** ou um **grupo** no canvas | **impossível** — nenhum gizmo | ✅ a caixa **dele** (o marcador), sempre do mesmo tamanho; o `Transform` dele é que anda, e os filhos seguem |
| ver onde está um objeto sem geometria | invisível | ✅ um **anel**, em **todo** objeto vazio da cena, a peso cheio |
| clicar no centro de um grupo | não selecionava nada | ✅ o anel **pega** — 4.ª fonte da porta de pick |
| clicar sobre o objeto que já está selecionado | pegava o filho que desenha por cima | ✅ o **1.º clique é dele**; os seguintes ciclam, como antes |
| *Criar componente* sobre um **grupo** | as peças da receita continuavam a desenhar (dois objetos empilhados) | ✅ a receita INTEIRA sai da tela (`MasterPiece`), e o gesto não escreve `Visibility` |
| *Revert to Master* numa peça que o artista moveu | a peça **teletransportava-se** | ✅ devolve o conteúdo e **mantém a posição** |
| pintar a sprite de uma cópia | as irmãs ficavam como estavam | ✅ os pixels sobem à receita e **todas** mudam |

⚠️ **O que ainda NÃO existe:** o `VecInstance` subsumido (F4.6) e a lane do `physics_ecs_c9` com
mestre+instância (F4.7). E a UI que **mostra** quais campos estão overridados — hoje o artista sabe
pelo comportamento e pelos verbos, não por um sinal na tela.

---

## §3 ⚠️ O que uma leitura rápida do diff entende ao contrário

1. **`MasterPiece` não é registado, e isso não é esquecimento.** Ele é DERIVADO
   (`assign_master_pieces`, chamado no topo do `render_loop::physics_bridge::dispatch`). Gravá-lo
   poria um valor derivado no arquivo, a envenenar o undo — e um mestre editado depois do gesto
   ficaria com peças **por marcar, simuladas em silêncio**. Só o `MasterRoot` (autoria) viaja.
2. **O passe tem DUAS metades e as duas são obrigatórias.** Marcar sem desmarcar deixa uma peça
   arrastada para fora do mestre **permanentemente invisível ao solver** — *um objeto que o artista
   tirou da biblioteca e que não cai* —, e o defeito é mudo.
3. **`deep_copy_subtree` não instancia nada.** Ela copia bytes; referências por identidade ficam a
   apontar para o original. A porta do produto que compõe cópia + remap é `shells/desktop/src/instantiate.rs`,
   e há gate estrutural (`only_the_instantiate_door_calls_the_deep_copy`) a mantê-la com um
   chamador só. *Duas portas em que uma tem de seguir a outra são uma porta e uma armadilha.*
4. **A ordem «remapear, depois ligar» é load-bearing.** O mapa contém `mestre → cópia do mestre`
   (tem de conter — uma junta ancorada na raiz precisa dele), então inserir o `InstanceOf` **antes**
   do remap fá-lo apontar para a **própria cópia**: a instância dir-se-ia instância de si mesma e o
   sync da F4.3 nunca propagaria nada. Gate: `the_instance_points_at_the_master_not_at_itself`
   (mutação mata).
5. **A raiz da cópia perde `RootOrder`/`SiblingOrder`; as PEÇAS mantêm os delas.** Dois irmãos com
   a mesma ordem é o empate que a casa não tem; a ordem interna é a receita.
6. **O filtro da ponte vive no TIPO da `QueryState`**, e por isso o `prepare()` passou a usar
   `query_filtered()` nas cinco — `query()` deixou de compilar para elas de propósito.
7. **A cópia profunda salta QUATRO componentes de propósito**, e não por bug: os de
   `catalog/bridges.rs`, agora declarados `owned_document`. Copiar o id de um documento possuído
   1:1 poria duas entidades a escrever nele — duplicar uma sprite pintada devolvia um sósia que
   apaga a tinta do original. ⚠️ *A cópia rasa acertava nisto por acidente*; a profunda tem de o
   decidir. ⛔ Quem decide é o **descritor**, nunca uma lista dentro do copiador.
8. **O `InstanceOf` está em TODA peça de uma instância**, e não só na raiz (F4.3) — é a
   correspondência durável de que o sync vive. A raiz não tem componente próprio: ela é *a peça
   cujo `master` é um `MasterRoot`*.
9. **O sync escreve, RELIGA, e só então conta.** Para um componente que carrega referência a
   comparação por bytes não é decidível na propagação (as duas pontas nomeiam corpos diferentes de
   propósito): sem esse desvio o passe reescrevia a junta todo o quadro. Ver §4.
10. **`instantiate_master` devolve `Result<_, Refusal>`**, não `Option`. Duas recusas que devolvem
   o mesmo `None` produzem o mesmo aviso inútil; a mensagem mora no gesto (F4.5).
11. **`ObjectInstance` é um CONJUNTO de chaves, não um mapa de bytes** — o valor já vive no
   componente da peça, que é uma entidade real. Ver §4.
12. **O item *Revert to Master* aparece em TODA linha da Hierarquia** (a tabela do menu é plana) e
   **responde** quando não se aplica. As três respostas do verbo são distinguíveis de propósito:
   *não é instância* · *é, e não tinha excepção* · *devolveu n*.

---

## §4 ⚠️ As premissas que a implementação REFUTOU, e as leis que ela pagou

1. ⛔ **A refutação 1 nomeia CINCO `QueryState` da ponte (`bridge.rs:84-127`). São SEIS.** Ela cita
   uma *faixa de linhas de um ficheiro*, e a `WheelQuery` nasceu noutro (`bridge/rope.rs`) depois
   disso. Era a pior de faltar: uma roldana é alcançada **pelo NOME da corda**, então uma dentro da
   biblioteca não só entraria no sistema vivo como **disputaria a resolução** com a da cena.
   *Uma referência por faixa de linhas envelhece à velocidade do ficheiro.*
2. ⛔ **A nota do catálogo sobre o `AnchorMount.anchor` está refutada.** Ela dizia *"é
   `RefKind::Object` quando a F1 migrar"*; o campo nomeia uma âncora **do próprio PAI**, e uma cópia
   profunda leva o pai junto — o nome continua a resolver dentro da cópia, sem remap nenhum.
   Declará-lo pediria uma reescrita que estragaria o que já funciona.
   *A estrutura da cópia apaga o caso especial.* (Corrigido em `catalog/image.rs`, com o motivo.)
3. ⛔ **O plano dizia que a cópia rasa vivia em `render_loop/hierarchy.rs:171-238`.** A faixa certa
   é outra (o ficheiro andou); o que importa é que ela era o **braço genérico** do `duplicate_row`,
   e hoje é uma chamada a `duplicate_subtree`.
4. ⛔⛔ **E a F4.3 achou uma LEI que nenhum documento tinha: *o que não PROPAGA não se REMAPEIA*.**
   O sync remapeava tudo, e o `InstanceOf.master` da raiz **é** a identidade do mestre — logo uma
   chave do mapa. O 1.º passe reescrevia-o para a identidade da **própria instância**; do 2.º
   quadro em diante a instância dizia-se instância de si mesma, o sync deixava de a encontrar, e
   **nada mais propagava**. ⚠️ **Todos os gates estavam verdes** — nenhum corria o passe DUAS vezes
   antes de medir. *Foi a mutação que SOBREVIVEU que o revelou.*
5. ⛔ **DECLARADO e não curado: a pose de repouso de uma peça DINÂMICA não propaga.** O dono do
   `Transform` de um corpo dinâmico é o solver **sempre** (a resposta sai do `BodyKind`, que não
   sabe se a cena está tocando), então mover o braço da receita não move o das instâncias, nem
   depois de um Reset. É a condição (b) da refutação à letra, e a irmã da limitação que o plano já
   declara para a config de física. Gate com o nome inteiro:
   `the_rest_pose_of_a_simulated_piece_does_not_propagate_and_that_is_declared`.

---

## §5 Superfície de colisão (o que outra linha pode tocar)

| O quê | Valor | Onde |
|---|---|---|
| Registro `ph2d-ecs` | 73 → 74 (`MasterRoot`) → 75 (`InstanceOf`) → **76** (`ObjectInstance`) | `scene/registry.rs` + `registry_tests.rs:~150` |
| Espelhos render/script | 74 → 75 → 76 → **77** cada | `ph2d-render/src/registry.rs`, `ph2d-script/src/registry.rs` |
| Variante nova de enum | `FieldKind::Ref` — **apendada no fim** | `ph2d-component-desc/src/lib.rs` |
| Campo novo no `ComponentDesc` | `owned_document: bool` + o construtor `D::owned_bridge` | idem + `catalog/bridges.rs` (os 4) |
| Campos declarados `is_ref` | `PhysicsJoint.body_a`(1)/`body_b`(2) · `PulleyWheel.rope`(1)/`body`(7) · `InstanceOf.master`(1) | `catalog/physics.rs`, `catalog/core.rs` |
| Env de smoke nova | `PH2D_INSTANCE_SMOKE=1` | `shells/desktop/src/instance_smoke.rs` + `init.rs` (cena vazia) |
| Assinatura mudada | `render_loop::hierarchy::dispatch` ganhou `registry: &ComponentRegistry` no fim | 1 sítio de chamada |
| Módulos novos (isolados) | `ph2d-ecs/src/instantiate.rs` · `ph2d-physics-ecs/src/ref_remap.rs` · `shells/desktop/src/{instantiate,instance_refs,instance_smoke,instance_sync}.rs` | append-only, irmãos |
| **Id de widget novo** | `CTX_MENU_HIER_REVERT_TO_MASTER` | `ids/menus.rs` + o gate `node_id_collisions` |
| **Ação do barramento nova** | `EditorAction::HierRevertToMaster { row }` | `action_bus.rs` + `panel-hierarchy/src/event.rs` + o dreno |
| Porta nova na ponte | `PhysicsBridge::document_owns_pose` (a condição (b) da refutação 1) | `bridge/pose_owner.rs`, ao lado do `player_liveness` |
| Passe novo no epílogo do quadro | `App::sync_instances()` entre o render loop e o `post_frame_undo` | `main.rs` — **a posição é lei**, ver o handoff §4 |
| `PROJECT_SCHEMA` | **não se mexe** — os dois componentes novos entram pelo `ComponentBlob`, que é a razão de o snapshot v2 existir | — |
| **Ids de widget novos** (F4.5) | `CTX_MENU_HIER_MAKE_COMPONENT` · `_INSTANTIATE` · `_APPLY_TO_MASTER` · `_DETACH_FROM_MASTER` | `ids/menus.rs` + o gate `node_id_collisions` |
| **Ações do barramento novas** (F4.5) | `EditorAction::Hier{MakeComponent,Instantiate,ApplyToMaster,DetachFromMaster}` | `action_bus.rs` + `panel-hierarchy/src/event.rs` + o dreno |
| Campo novo no `App` | `cycle_pick_selection: Option<u64>` (o ciclo do clique atado à seleção) | `app_state.rs` + `main.rs` |
| Campo novo no `PickWorld` | `pixels_per_meter: f32` — 3 sítios de construção | `hover_highlight.rs`, `input_dispatch.rs` (×2) |
| **Assinaturas mudadas** (F4.6) | `OwnedDocs<'_>` entrou em `instantiate_master` · `duplicate_subtree` · `make_master` · `apply_to_master` · `sync_instances` · `instance_verbs::drain`; `hierarchy::dispatch` ganhou `vec_entities` | quem chamar de outra linha **não compila**, que é o desejado |
| Módulos novos (2ª leva) | `shells/desktop/src/{group_gizmo_view,instance_verbs,instance_docs,instance_sync_docs}.rs` + `render_loop/{empty_object_overlay,off_canvas}.rs` | append-only, irmãos |
| **Extract** | `sim_extract` deixou de decidir a visibilidade no fio — a porta é `off_canvas::is_off_canvas`, e **uma peça de receita deixou de desenhar** | `render_loop/sim_extract.rs` + `off_canvas.rs` |
| Porta de pick | `hover_highlight::pick_objects_at` ganhou uma **4.ª fonte** (o anel de um objeto vazio) | o gate `the_object_pick_composite_exists_once` lista as quatro |

⚠️ **Nenhum contrato congelado (§6 do CLAUDE.md) foi tocado.**

---

## §6 Gate de fecho (corrido em 2026-08-26)

| | |
|---|---|
| `cargo test -p ph2d-host-desktop --bins` | ✅ **4318** passaram · 0 falharam · 251 ignorados (re-corrido depois da §9) |
| `cargo test -p ph2d-host-desktop --tests` | ✅ **3680** (⚠️ o censo de dois lados apanhou o `ObjectInstance` sem descritor — **ele fez o trabalho dele**) |
| `typos` | ✅ |
| `ph2d-ecs` · `-component-desc` · `-render` · `-script` · `-physics-ecs` · `-panel-inspector` · `-panel-hierarchy` · `-editor-core` | ✅ todos verdes |
| `cargo check --workspace --all-targets` | ✅ |
| `cargo fmt --all` | ✅ |
| `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings` | ✅ |

⚠️ **Uma flake CONHECIDA apareceu no fecho** e não é desta linha:
`flip_smooth::resample_measurement::precisao::orcamento::the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke`
— membro nomeado da família de flakes sob fan-out no `CLAUDE.md` §5.0. **Verde 3/3 sozinha**, e o
diff não tem uma linha de Flip.

⚠️⚠️ **E o gate de LOC apanhou SEIS ficheiros acima do teto de 600, todos desta linha** —
`instance_sync_tests` (870) e `render_loop/hierarchy` (613) da F4.3/F4.4; e **quatro latentes das
fatias anteriores**, que nunca correram `--tests`: `render_loop/inspector_commits` (632, `main` tem
539), `render_loop/inspector_physics_apply` (624), `input_dispatch/painter_canvas_input` (609) e
`timeline_persist_tests` (613). **Todos cortados por ASSUNTO, nenhum por allowlist**, e cada corte
leva no cabeçalho de que lado está:

| ficheiro | corte | irmão |
|---|---|---|
| `instance_sync_tests` | a propagação × a **excepção** | `instance_override_tests` (novo) |
| `render_loop/hierarchy` | a mecânica das linhas × o verbo de **instância** | o dreno foi para `instance_sync` |
| `inspector_commits` | as outras famílias × a família da **Sprite** | `inspector_commits_sprite` (já existia) |
| `inspector_physics_apply` | o que uma EDIÇÃO faz × o que a CRIAÇÃO **semeia** | `inspector_physics_seed` (novo) |
| `painter_canvas_input` | a entrega do ponteiro × o menu de alça da **curva** | `painter_curve_input` (novo) |
| `timeline_persist_tests` | o que atravessa o FICHEIRO × o que a shell **publica** | `timeline_publish_tests` (novo) |

⚠️ *A causa é a mesma dos latentes de clippy: os fechos anteriores não correram o gate INTEIRO.*

⛔⛔ **E um SÉTIMO latente, do mesmo fecho em falta, com uma lição própria:**
`the_position_commit_reseats_the_anchor_through_the_door` — um gate que lê os **3000 bytes** a
seguir à captura do pivot de um joint à procura da porta `set_joint_anchor_world`. A **F3** enfiou
as 27 linhas do `+` do Inspector **entre a captura e o dreno**, e a porta saiu da janela
(`2 555 → 3 864` bytes). ⚠️ **A lei não se partiu** — o pivot continua a chegar à porta —, mas *a
janela É a lei aqui*, e o próprio ficheiro já o diz em dois sítios: **«a cura é tirar o intruso do
meio, não alargar a janela»**, porque ela é a forma de exigir que a captura e o dreno de uma
intenção fiquem **à vista um do outro**. O bloco do `+` mudou-se para depois do dreno, com a nota
ao lado — e a distância voltou aos `2 555`. *Um gate cuja régua é a distância só dói quando alguém
escreve no meio; foi por isso que ele foi escrito assim.*

⛔ **E um OITAVO, que é o melhor dos oito:** `the_duplicate_row_asks_for_a_vec_path_before_it_spawns`
tem um **controle positivo** — ele procura o marco do caminho genérico para provar que está a ler o
ficheiro certo. A F4.2 trocou esse caminho (o `spawn_empty()` da cópia rasa virou a porta
`duplicate_subtree`), e o controle disparou com a frase exata: *«este gate mede o arquivo errado»*.
⚠️ **Ele fez o trabalho dele**, e a cura é mover o marco — não apagar o controle. *Um gate sem
controle positivo teria continuado verde a medir nada.*

⚠️⚠️ **E esse clippy apanhou CINCO erros latentes das fatias ANTERIORES desta linha** — três em
ficheiros que ela criou (`scene/incremental.rs` da F2, `component_seed.rs` da F3,
`project_migrate_sprite.rs` da F1.6) e dois em ficheiros que ela modificou (`sim_extract.rs`,
`sim_extract_slice.rs`, F1.6). **Todos corrigidos aqui.** A causa é uma só: os fechos anteriores
correram `cargo clippy -p <crates>` **sem `-D warnings`** e leram o exit code `0` como verde.
*Um lint sem `-D warnings` não reprova nada, e um gate que não reprova não é um gate.* Registado em
[`project-memory`](../../../project-memory/feedback_the_closing_clippy_must_cover_every_crate_the_line_touched.md).
⚠️ Um deles pediu um tipo novo: `sim_extract_slice::SliceSource` (o trio `região` + `grelha` +
`dimensões da fonte`, que já andava sempre junto) — 8 argumentos passaram a 6.

### Provas de mutação (a linha corre-as; o restore faz `touch`)

| Mutação | Resultado |
|---|---|
| tirar o `remap_object_refs` da porta | ⛔ RED — *«a instancia 1 tem o braco a **19.651** do eixo dela (o pino manda 0.900)»* |
| **mover** o `InstanceOf` para antes do remap | ⛔ RED — *«a instancia diz-se instancia de SI PROPRIA»* |
| não remover o `MasterRoot` da cópia | ⛔ RED em **três** gates (*«nao balancou»*, *«peca marcada»*, a receita) |
| tirar o `assign_master_pieces` da porta | ⛔ RED — *«a receita mexeu-se (0.9, 3.4) → (0.636, 2.763)»* |
| a cópia deixa de descer a árvore | ⛔ RED em **quatro** gates do `ph2d-ecs` |
| tirar o salto do `owned_document` | ⛔ RED — *«a copia herdou o documento do Painter — as duas escrevem no mesmo»* |
| **sync**: sem a comparação por bytes | ⛔ RED — o passe deixa de ser ponto fixo |
| **sync**: `MasterRoot` fora do `NEVER_PROPAGATES` | ⛔ RED — a instância vira receita |
| **sync**: `InstanceOf` fora do `NEVER_PROPAGATES` | ⛔ RED em **dois** gates |
| **sync**: `ROOT_IS_ITS_OWN` sem `Transform`/`Name` | ⛔ RED — as instâncias saltam para cima da receita |
| **sync**: sem o `document_owns_pose` | ⛔ RED em **dois** — o corpo teleporta |
| **sync**: sem o religamento | ⛔ RED — a junta larga o rig da instância |
| **sync**: sem a rota dos que carregam referência | ⛔ RED — 3 escritas por quadro com o mundo parado |
| **override**: o override deixa de segurar | ⛔ RED — a receita apaga a edição do artista |
| **override**: nunca capturar | ⛔ RED em **quatro** |
| **override**: sem o eco do mestre | ⛔ RED em **cinco** — tudo vira override |
| **override**: o revert não esquece o eco | ⛔ RED — o override renasce no quadro seguinte |
| **override**: o conjunto nunca chega ao mundo | ⛔ RED em **quatro** |
| **override**: os que carregam referência voltam a capturar | ⛔ RED em **sete** |
| sem a recusa de ciclo | ⛔ RED — a instância aterra dentro da própria receita |

⚠️ **Uma mutação SOBREVIVEU e foi refeita:** duplicar a inserção do `InstanceOf` (em vez de a
mover) não muda nada — a inserção posterior sobrescreve. *Uma mutação que acrescenta em vez de
mover não testa a ordem.*

---

## §7 O smoke que o Enio recebeu

Ver a mensagem da linha. `PH2D_INSTANCE_SMOKE=1`, em três partes:

1. três pêndulos a balançar, a receita parada lá em cima (F4.1 + F4.2);
2. o **Duplicate** da Hierarquia sobre um deles (F4.2 — antes desta fatia devolvia uma linha vazia);
3. escolher `Ragdoll > Arm` (o de CIMA, a receita) e mudar a cor em *Color & Tint* — **os três
   braços de baixo mudam com ele** (F4.3). ⚠️ A própria cena imprime esta instrução: sem ela o
   artista vê três pêndulos e não descobre sozinho que a receita é editável;
4. pintar o `Arm` de **uma** das cópias e depois repintar o da receita — **a que ele tocou fica com
   a cor dela** (F4.4);
5. botão direito **na peça que ele pintou** → **Revert to Master** — ela volta a ouvir a receita.
   ⚠️ Numa linha fora de qualquer instância o item **responde com um aviso**: a tabela deste menu é
   plana, e um item que come o clique em silêncio é pior que um ausente.

⛔⛔ **O REPORT do Enio (mesmo dia) e a cura:** a 1.ª versão do verbo exigia a **RAIZ** da instância
e respondia *«Not an instance»* na peça — tecnicamente certa e **inutilmente** certa. Para pintar o
braço de uma cópia o artista tem de selecionar a linha do **braço**, e é lá que a mão dele está
quando ele quer desfazer. *Um aviso que diz o que a coisa NÃO é, sem dizer o que fazer, é um botão
mudo com legenda.* ⇒ o verbo aceita qualquer peça, **sobe por `ChildOf`** (nunca pelo elo — o
`InstanceOf` de uma peça aponta para a peça do MESTRE, e subir por ele sairia da instância) e o
**escopo é o que se clicou**: numa peça, só a excepção dela; na raiz, todas. ⚠️ *Devolver o rig
inteiro porque o artista pediu um braço seria apagar trabalho que ele não mandou apagar.*

---

## §8 O que fica ABERTO

- **F4.3–F4.7** no [plano vivo](../05_plano_de_implementacao.md) — o sync vivo, os overrides, os
  verbos na UI, o `VecInstance` subsumido, e a lane do `physics_ecs_c9` com mestre+instância.
- ⚠️ **A F1 continua pela metade** (herdado): a timeline ainda não aponta por identidade.
- ⚠️ **O integrador tem de apagar do `CLAUDE.md` §5 a frase «o `physics_ecs_c9` está POR
  RE-CAPTURAR»** — ela é um fantasma medido em 25/08 (os três hashes C9 comparam-se **entre si**,
  não contra baseline gravado) e continua no roteador porque o §5 se edita na integração.

---

## §9 ⭐⭐⭐ Os TRÊS reports do smoke do Enio (2026-08-26), e o que cada um custou

> *«Pintei uma sprite de uma instância e as outras não mudaram. O Objeto vazio criado na hierarquia
> é invisível e ao agregar filhos e selecionar o objeto (o pai) não se consegue transformar o
> conjunto como um objeto só. […] Outra coisa: revert to master modifica a posição global do objeto
> e isso não é uma boa idéia, melhor o objeto ficar onde está.»*

⚠️ **Os três foram MEDIDOS por sonda antes de qualquer cura** (dois `#[test]` temporários que
imprimiam os overrides capturados e a pose antes/depois). Sem isso, dois deles teriam sido curados
no sítio errado: a queixa da pose parecia ser sobre a RAIZ da instância (é sobre uma PEÇA — a raiz
já era imune) e a da pintura parecia um bug do sync (era o modelo a funcionar).

### §9.1 O objeto vazio e o grupo — [`group_gizmo_view.rs`](../../../shells/desktop/src/group_gizmo_view.rs)

`snapshots::build_view` respondia `None` para toda entidade sem geometria própria (*«grupo/outro:
sem gizmo próprio»*), e **sem `GizmoView` não há caixa, alças nem hit-rect**. O objeto que o botão
`Add` da Hierarquia acaba de criar (F3: `Transform` + `Name` e mais nada) era o único do app que o
artista não podia pegar.

As duas respostas, com a lei do container do envelope (ADR-0129 Fatia 3) generalizada:

- **com filhos visíveis** ⇒ a caixa é a **UNIÃO** deles, no espaço **LOCAL do pai**. O drag escreve
  só o `Transform` do pai e os filhos seguem por parentesco — *como um objeto só*, sem cisalhar;
- **sem nenhum** ⇒ o marcador do vazio, meia-extensão **derivada** do `HANDLE_SIZE_PX` (duas alças
  ⇒ quatro de largura, a menor em que a quina e o meio da aresta não se sobrepõem).

⚠️ **Três coisas que uma leitura rápida entende ao contrário:**

1. **A união é medida no espaço do PAI, não no mundo.** `gizmo_view_from` aplica a pose do pai à
   caixa que recebe; uma caixa já em mundo seria transformada **duas vezes** e o gizmo derivaria do
   objeto a cada grau de rotação (gate `the_box_is_measured_in_the_parents_frame`).
2. **São os QUATRO cantos de cada filho**, não o par (mín, máx): sob rotação a caixa do filho não é
   eixo-alinhada no espaço do pai (gate com um quadrado a 45°, que mede `√2/2`).
3. ⛔ **Quem já tem gizmo próprio NÃO ganha caixa** (`publishes_its_own_handles`): junta e roldana
   são **pontos** com dots agarráveis — uma caixa por cima regista `Translate` no hit-index e
   **engole o clique neles** —, e uma peça de modelagem 3D tem o gizmo do MODEL e nem usa o
   `Transform` da casa (usa o `FieldPose`), pelo que a caixa sairia na origem do mundo. ⚠️ **É uma
   LISTA e ela envelhece**: uma família nova com alças próprias que não venha aqui nasce com duas
   caixas sobre o mesmo objeto.

⚠️ **Filho ESCONDIDO não entra, e a sub-árvore dele também não** — e isto não é asseio: a receita
de um componente é escondida de propósito (F4.5), e uma caixa que a envolvesse mediria um objeto que
não está na tela.

E o **anel** ([`render_loop/empty_object_overlay.rs`](../../../shells/desktop/src/render_loop/empty_object_overlay.rs)):
um objeto sem geometria não emite `RenderInstance` nenhuma. ⚠️ A pergunta *«está vazio?»* é feita
**uma** vez (`box_of` — a mesma que dimensiona a caixa), e o raio é **medido na tela** pela câmera,
nunca `raio × zoom` escrito à mão, que seria a segunda régua.

⚠️ **Duas costuras não são alcançáveis de um teste** (o closure de `build_view` pede `HeroScreen` +
`PresentWorld` + câmera; o passe de pintura pede uma superfície) ⇒
[`tests/an_empty_object_is_reachable.rs`](../../../shells/desktop/tests/an_empty_object_is_reachable.rs)
varre a FONTE dos dois fios, e o negativo dele proíbe o `return None` antigo de voltar ao lado do novo.

### §9.2 O *Revert* deixa a POSE onde está — [`instance_sync.rs`](../../../shells/desktop/src/instance_sync.rs)

Medido: arrastar uma peça captura um override de `Transform`, e o revert punha-a de volta na pose da
receita. **Decisão do Enio:** o verbo devolve tudo **menos** a pose. É a lei que a raiz já tinha
(`ROOT_IS_ITS_OWN`) descida um nível: *onde uma coisa está é do artista que a largou lá*.

⚠️ **A pose CONTINUA a ser um override** — sem ele o passe seguinte reescrevia por cima do arrasto,
que é pior. E a receita continua a poder mandar: quando o **mestre** mexe a peça dele, o empate
resolve-se a favor dele e a instância segue.

⇒ o resultado tem **dois** números (`Reverted { count, poses_kept }`) e o toast tem **quatro**
respostas. *Um `0` de «não havia nada» e um `0` de «só havia a posição, e ela fica» são coisas
diferentes para quem acabou de mover a peça.*

### §9.3 Os PIXELS são da receita — [`hero_intents/texture_rebind.rs`](../../../shells/desktop/src/hero_intents/texture_rebind.rs)

Medido: pintar mudava `Sprite` + `SpritePixels` na **cópia**, o sync lia *«só a instância mexeu»* e
capturava um override — **modelo correto, resultado errado**. A edição de pixels passa a subir até à
receita (`write_through_targets`), e o passe leva-a a todas as instâncias. Duas razões:

1. **Uma imagem é um ASSET, não uma propriedade.** Em todo motor 2D pintar a textura muda quem a
   usa; o que é per-objeto é *qual* imagem ele usa, não o **conteúdo** dela. O `tint`, a pose e a
   máscara continuam a ser da cópia — a fronteira é entre *os pixels* e *os botões*.
2. **A receita está ESCONDIDA** (F4.5), então pintá-la não é alcançável por gesto nenhum. Sem esta
   subida, os pixels de um componente eram a única coisa do app **sem forma de ser editada**.

⚠️ **Não vira override, e é por construção:** ao escrever no mestre, o passe seguinte lê *«o mestre
mexeu-se»*; a cópia pintada já tem os bytes, logo `want == have` e ela não é reescrita. **O ponto
fixo do sync fica intacto**, e há gate sobre os dois factos (`painting_one_copy_reaches_the_others`
afirma o resultado visível **e** que o conjunto de overrides ficou vazio **e** que o passe seguinte
escreve `0`).

⛔ **FRONTEIRA nomeada:** para pintar UMA cópia diferente das outras, *Detach from Master* primeiro.
*Uma cópia que ainda segue a receita não tem pixels próprios — é isso que ser cópia é.*

⚠️ A subida entra no **funil** que as oito ferramentas de imagem já atravessam
(`rebind_to_individual`), e não num sítio de chamada: as invariantes de re-alojamento continuam
escritas uma vez só. A guarda é *«sem entidade repetida»* e **não** um tecto de saltos — um elo
corrompido daria laço infinito dentro de um commit de ferramenta, e um número máximo transformaria
isso numa contagem que ninguém sabe explicar.

⚠️ **Uma guarda DECORATIVA foi retirada no caminho** (`&& get::<Sprite>(up).is_some()` na subida):
nenhuma mutação a matava, porque o corpo já desiste sozinho numa entidade sem sprite. *Uma
afirmação que mutação nenhuma mata é uma afirmação sobre nada.*

### §9.4 Prova de mutação

**11 mutações, 11 mortas** — marcador colapsado · tecto da alça · dois cantos em vez de quatro ·
guarda de `Visibility` · guarda de alças próprias · `FieldNode` fora da lista · caixa medida em
mundo · revert a voltar a mexer na pose · pintura sem subir (nos dois gates que a afirmam) · cadeia
a ignorar o *Detach*.

### §9.5 ⚠️ O que o integrador tem de saber

- **Nada fora de `shells/desktop/` foi tocado** por esta fatia — a superfície de colisão do §5 não
  muda.
- ⚠️ **`render_loop::sheet_grid_overlay` passou de `mod` a `pub(crate) mod`** e
  `vec_gizmo_view::gizmo_view_from` de privada a `pub(crate)` — duas linhas, e são o que impede a
  união de reimplementar a caixa de um sprite e a de uma forma vetorial.
- ⚠️ **`revert_all_overrides` mudou de assinatura** (`Option<usize>` → `Option<Reverted>`): quem a
  chamar noutra linha não compila, o que é o comportamento certo.

---

## §10 ⭐⭐ A 2.ª volta do smoke (2026-08-26) — três correções ao §9 e **um defeito que ele escondia**

> *«Se eu crio diretamente uma sprite não preciso do círculo — o gizmo é exclusivo do objeto que
> nasce vazio. Outro problema: se desseleciono o objeto vazio, o círculo some. O círculo só pode
> sumir no runtime. Outro: se o objeto vazio ganha filhos não consigo transformar o objeto total a
> partir do centro do objeto vazio, mas o gizmo do objeto vazio deve existir mesmo quando ele ganha
> filhos. O restante parece OK.»*

### §10.1 O anel é o CORPO do objeto, não uma marca de seleção

A 1.ª versão desenhava-o só para o selecionado, com a razão escrita *«a marca serve o gesto que
está a acontecer»*. **Errada.** O anel é para um objeto sem pixels o que o quad é para uma sprite —
e um corpo que só existe enquanto se olha para ele não é um corpo. ⇒ ele é desenhado para **todo**
objeto vazio da cena, e ter filhos não o apaga.

Ele desaparece em **duas** situações, ambas *«não está na cena»*: o olho fechado (`Visibility`) e
ser peça de uma receita. E numa terceira, quando existir: o modo de jogo, que não pinta chrome
nenhum (o `shells/game`/R1 está adiado — é o *«só pode sumir no runtime»* do report).

⚠️ **O censo corre pelos ARQUÉTIPOS** (`empty_objects`): é a única travessia do mundo inteiro que
funciona com `&World`, porque uma `query` do bevy pede `&mut` e o passe de pintura só tem a
partilhada. Ordenado por `StableId`, nunca pelos bits de alocação, que o respawn do undo troca.

### §10.2 O anel PEGA — a 4.ª fonte da porta de pick

*«Não consigo transformar o objeto total a partir do centro do objeto vazio»* não era sobre a
caixa: era sobre **selecionar**. Um objeto sem pixels nunca foi alcançável por
`pick_sprites_at_world`, logo a única forma de o pegar era a lista da Hierarquia. *Uma alça que só
se alcança noutro sítio não está no canvas.*

⇒ `group_gizmo_view::pick_empty_at_world` entra em `hover_highlight::pick_objects_at` — a porta
ÚNICA que o clique **e** o realce de hover usam (o gate `the_object_pick_composite_exists_once` já
existia e ganhou a quarta fonte na lista de controlo).

⚠️ **Por ÚLTIMO na lista, e é a metade que importa:** um objeto vazio é quase sempre o PAI da arte
sob o anel, e `pick_order::descendants_first` **adia** o ancestral. Clique sobre a arte pega a arte;
clique no anel onde não há arte pega o grupo; o segundo clique cicla. *O contêiner não rouba o
clique dos filhos.*

⚠️ **Disco, não aro** (o interior conta — um aro de 1,5 px é um alvo que se persegue), e o raio
entra pela **média geométrica** `√|sx·sy|`: a mesma lei do traço vetorial sob escala não-uniforme
(bug #27 do Vector). Um anel-elipse obrigaria o dedo a um teste de elipse, e a tinta e o dedo
divergiriam no dia em que um dos dois esquecesse. **Uma porta, dois consumidores**
(`marker_world_radius`).

### §10.3 ⛔⛔ E o report descobriu um defeito da F4.5 que o §9 não via: **`Visibility` não desce**

Ao medir *«o anel some quando a receita está escondida?»* apareceu o que estava por baixo:
**`Visibility` é per-entidade neste motor e não propaga para os descendentes.** Não é uma
suposição — o `sim_extract` diz-o pelo nome no doc do `resolve_clip_grouping`: *«Visibility is
per-entity, it does not propagate to descendants … Proper subtree-hide = visibility propagation, a
future wave.»*

⇒ o *Criar componente* da F4.5, que escondia **só a raiz** do mestre, **nunca escondeu uma receita
que fosse um grupo**: as peças dela continuavam a desenhar, e o artista via os dois objetos
empilhados que a nota daquela fatia dizia ter evitado. ⚠️ **O gate era verde**, porque media a
MARCA (`Visibility` na raiz) em vez do FIM (o que se desenha). *Um gate sobre o meio fica verde
sobre o defeito que ele existe para apanhar.*

**A cura:** quem não desenha uma receita é o extract, pela marca **derivada** `MasterPiece` (a raiz
e toda a descendência, re-carimbada por quadro por `assign_master_pieces`) —
`render_loop::off_canvas::is_off_canvas`, irmão do `sim_extract` por assunto (lá mora *como* uma
sprite vira instância; aqui mora *se* ela vira) e porque aquele ficheiro já vive sob excepção de LOC.

⛔ **Escrever `Visibility` nas peças seria o contrário do que se quer:** a `Visibility` de uma peça é
**autoria** e propaga para as instâncias — toda cópia nasceria invisível.

⇒ **o gesto deixou de tocar em `Visibility`**, e o `ROOT_IS_ITS_OWN` mantém a entrada dela por outra
razão, que sempre foi a dela: *esconder UMA cópia é sobre aquela cópia*.

⚠️ **Isto muda o que a caixa do grupo mede:** um filho escondido fica de fora, mas os **filhos dele
continuam**, porque continuam a desenhar. A 1.ª redação saltava a sub-árvore e citava a receita
escondida como razão — as duas metades estavam erradas.

### §10.4 Prova de mutação (2.ª volta)

**10 mutações, 10 mortas** — um grupo deixar de ser vazio · a receita ganhar anel · raio por um eixo
só · o anel pegar em todo o lado · a receita voltar a desenhar · o olho deixar de esconder · a caixa
voltar a saltar a sub-árvore · o censo ignorar o olho · o gesto voltar a escrever `Visibility` ·
`assign_master_pieces` marcar só a raiz.

### §10.5 ⚠️ O que o integrador tem de saber (2.ª volta)

- ⚠️ **`PickWorld` ganhou um campo** (`pixels_per_meter`) — três sítios de construção; quem
  construir um quarto noutra linha não compila, que é o comportamento certo.
- ⚠️ **`sim_extract` deixou de decidir a visibilidade no fio** — a linha `let hidden = sim…` foi
  substituída por `off_canvas::is_off_canvas(…)`, e há gate a proibir as duas de coexistirem.
- ⚠️ **`ph2d-ecs` não foi tocado** — o `MasterPiece` e o `assign_master_pieces` já existiam desde a
  F4.1; o que mudou foi quem lhes pergunta.

---

## §11 ⭐ A 3.ª volta (2026-08-26) — o anel a peso cheio, e a UNIÃO dos filhos REJEITADA

> *«O círculo desselecionado está quase invisível. Vamos fazer um ajuste, pois não ficou legal a
> questão do gizmo quando selecionamos o objeto vazio que tem filhos. O objeto vazio deve permanecer
> com seu gizmo original e não se utilizar do gizmo dos filhos.»* (com foto)

### §11.1 O anel deixou de esmaecer

A §10 pintava o anel do não-selecionado a `alpha = 0,35`, com o argumento *«senão uma cena com seis
grupos leria como seis coisas selecionadas»*. ⚠️ **Certo sobre o problema, errado sobre o remédio:**
a seleção já é dita pela **caixa e pelas oito alças** à volta — muito mais tinta que meio tom num
traço de 1,5 px. Dizer a mesma coisa duas vezes com o **único canal** que este anel tem só apaga o
corpo do objeto. ⇒ o `DIM_ALPHA` **morreu** (uma constante retirada, não afinada) e o parâmetro
`selected` saiu da assinatura do overlay.

### §11.2 ⛔ RECUSA DE PRODUTO: a caixa NÃO é a união dos filhos

A §9 fazia a caixa de um grupo ser a **união dos filhos visíveis** no espaço local do pai — a lei do
container de um `VecEnvelope` (ADR-0129 Fatia 3) generalizada. Construída, medida, e **rejeitada
pelo Enio**: um objeto vazio passava a ter um tamanho que não é dele, e a moldura mudava sozinha
sempre que um filho se mexia. *Um controlo cuja moldura muda quando o artista não lhe tocou lê-se
como o app a decidir por ele.*

⇒ a caixa é **sempre o marcador**, com a meia-extensão derivada do `HANDLE_SIZE_PX`. ⚠️ **Isto não
custa a função**: o que faz o conjunto andar como um objeto só é o gizmo escrever o `Transform` do
PAI (os filhos seguem por parentesco), e não o tamanho da moldura.

⛔ **A árvore da versão da união sobrevive em `828bc88f4`** — e o que sai com ela é a matemática dos
quatro cantos no espaço do pai, a regra do filho escondido e cinco gates. Uma 2.ª tentativa começa
perguntando **o que ficou pior**, não reconstruindo (precedente: a faixa de barras do
`value.pattern` no Motion).

⚠️ **O gate novo mede o FIM e não a fórmula**: a caixa não muda quando nasce um filho longe, **e**
ela continua centrada no pivô do objeto (a união deslocava-a para o centroide dos filhos, deixando o
pivô para trás — é essa a metade que distingue as duas versões).

### §11.3 Prova de mutação (3.ª volta)

**11 mutações, 11 mortas** — a caixa tomar emprestado dos filhos · a caixa colapsar · um grupo
deixar de ser vazio · a receita ganhar anel · raio por um eixo só · o anel pegar em todo o lado · o
censo ignorar o olho · a receita voltar a desenhar · o olho deixar de esconder · o gesto voltar a
escrever `Visibility` · `assign_master_pieces` marcar só a raiz.

---

## §12 ⭐⭐⭐ A 4.ª volta (2026-08-26) — **o primeiro clique é de quem já está selecionado**

> *«Como os objetos filhos ficam com um z-index relativamente maior que o pai, quando tentamos
> arrastar o pai (objeto previamente vazio) selecionamos um filho. Precisamos que a preferência do
> primeiro clique seja do objeto que está selecionado na hierarquia e depois a cada clique a seleção
> passe a ciclar (como já está implementado).»*

⚠️ **Isto NÃO revoga a lei do contêiner** (`descendants_first`, Enio 2026-08-19): ela responde *«em
que ORDEM os candidatos ficam»* e continua igual — um clique dentro de um grupo que **ainda não está
selecionado** pega o filho, como no Figma. O que a 4.ª volta acrescenta é *«por onde o ciclo
COMEÇA»*, e a resposta é o objeto que o artista já escolheu: *o gesto seguinte a escolher um objeto
é mexer nele*, e pedir-lhe que descubra uma cadência de cliques para voltar ao que ele acabou de
selecionar é o mesmo defeito que a lei do contêiner curou, do outro lado.

A lei é pura e vive em [`pick_order::start_on_selection`](../../../shells/desktop/src/pick_order.rs),
com quatro fronteiras e gate para cada uma:

1. o selecionado **está** nos hits ⇒ o ciclo começa nele, e a **lista não é reordenada** (os cliques
   seguintes continuam a alcançar o filho);
2. o selecionado **não** está nos hits ⇒ nada muda;
3. ⚠️ **excepto se o press for um `Translate` no gizmo PRIMÁRIO**: a caixa de um objeto vazio é um
   quadrado e o anel dele é o disco **inscrito**, então premir numa **quina** da caixa é premir o
   gizmo dele e não o corpo dele — sem esta metade o clique caía no filho por baixo. Aí o
   selecionado entra na lista **no fim** (a ordem de camada dos outros fica intacta). *É a mesma lei
   que já dizia «sem nada sob o cursor, cai na seleção atual» (Enio, 2026-07-09), com algo sob o
   cursor;*
4. ⛔ uma lista **vazia** continua vazia — é o caminho do clique no nada, que limpa a seleção.

⛔ **Nunca num clique com MODIFICADOR:** `Shift`/`Cmd` estão a curar a seleção, e preferir o primário
faria o `Shift`+clique num filho alternar o **pai**.

### ⚠️ E o ciclo passou a estar atado à SELEÇÃO — a metade que quase escapou

`same_list` comparava só a lista de hits. ⇒ o artista clicava num ponto (a lista fica gravada),
escolhia o pai **na Hierarquia**, voltava a clicar no MESMO ponto — a lista era a mesma, o ciclo
antigo continuava, e ele apanhava o filho outra vez. Com a seleção no teste (`cycle_pick_selection`),
mudar de seleção por fora abre um ciclo **novo**, que começa no que ele escolheu.

⚠️ Ela é gravada **depois** do clique (é a seleção que ficou), senão o próprio ciclo se invalidaria a
cada passo e nunca andaria.

**Prova de mutação:** 5 mutações, 5 mortas (o ciclo começar sempre no topo · uma seleção fora entrar
na lista · a quina do gizmo deixar de contar · o clique no nada voltar a selecionar · a lista ser
reordenada em vez de o selecionado ir para o fim).

⚠️ **Um campo novo no `App`** (`cycle_pick_selection`) — quem construir um `App` noutra linha não
compila, que é o comportamento certo.

---

## §13 ⭐⭐ F4.6a/b — os DOCUMENTOS possuídos entram na cópia e no sync

O que faltava para uma instância de **arte vetorial** existir: a cópia profunda salta os quatro
componentes `owned_document`, e uma peça vetorial saltada fica **sem geometria nenhuma**.

- **F4.6a** — [`instance_docs`](../../../shells/desktop/src/instance_docs.rs) clona o `VecPath` e
  aponta a cópia para o clone. ⚠️ O par `path ⟺ entidade` entra **junto** (senão o
  `vec_entities::sync` cunha uma segunda entidade), e o clone entra **sem deslocamento** (a
  geometria é LOCAL; quem põe a peça no sítio é o `Transform`). ⭐ Cura de passagem um defeito
  anterior às instâncias: **duplicar um grupo** com formas vetoriais dentro.
  ⛔ Os outros três (`PaintedDoc` · `BakedForm` · `FlipObjectRef`) continuam dropados, agora com
  **nome** no relatório e um censo de dois lados a defendê-lo.
- **F4.6b** — [`instance_sync_docs`](../../../shells/desktop/src/instance_sync_docs.rs) propaga o
  documento por **CONTEÚDO** (id normalizado), com o mesmo eco, a mesma chave de override e as
  mesmas três respostas. O *Apply* ganhou o mesmo caminho: pelo geral, o mestre passaria a apontar
  para o path da cópia.

⚠️ **`OwnedDocs` entrou na ASSINATURA** de `instantiate_master`, `duplicate_subtree`, `make_master`,
`apply_to_master`, `sync_instances` e `instance_verbs::drain` — uma cópia profunda sem os
documentos está **incompleta**, e uma invariante que dois sítios têm de lembrar é uma que um deles
vai esquecer. `PickWorld`-style: quem construir uma chamada nova noutra linha não compila.

**⏳ Falta a F4.6c** — matar o `InstanceLive` (o produtor derivado), reescrever os verbos vetoriais
sobre o mecanismo geral e **materializar no load** os documentos com `VecInstance` (degrau do
`PROJECT_SCHEMA`). Enquanto ela não vier, os dois modelos coexistem: o antigo continua a servir os
documentos antigos, e o novo já serve tudo o que se faça pelos verbos gerais.

---

## §14 ⛔⛔⛔ O DEFEITO ABERTO — a F4.6b **não passou no smoke**

> *«Não funcionou. Ao mudo o path, as instâncias não mudaram.»* (Enio, 2026-08-26, sobre o smoke da
> §13.)

⚠️ **A fatia F4.6b shipa NÃO VALIDADA.** Os gates são verdes headless — o passe propaga o conteúdo,
mantém o ponto fixo, captura override e o *Apply* funciona (5 mutações, 5 mortas) — e **no app não
acontece**. *Um gate verde sobre um caminho que o produto não percorre é a mesma classe de defeito
que a §10.3 pagou: ele mede o mecanismo, não o fim.*

⇒ **nada aqui deve ser lido como «a arte vetorial propaga»**. O que está provado é que a porta
`instance_sync_docs::sync_one` faz a coisa certa quando é chamada com um mestre, uma instância e um
documento; o que **não** está provado é que a cena que o artista monta pelo app chega a essa porta
nesse estado.

### Os suspeitos, por ordem de custo de medição (para quem pegar isto amanhã)

1. ⚠️ **A RECEITA vetorial ainda DESENHA.** A cura da §10.3 (`off_canvas::is_off_canvas`) gateia o
   **extract de sprites**; um `VecPath` é desenhado pelo renderer vetorial a partir do `VecScene`,
   e **não passa por ali**. ⇒ o mestre e a instância ficam **sobrepostos** no canvas, e uma edição
   que pareça ser «no mestre» pode estar a acontecer na cópia por cima dele (aí não propagar é o
   comportamento correto). **Meça isto primeiro** — é o único suspeito que explica o report inteiro
   sem nenhum código estar errado.
2. ⚠️ **A ORDEM dentro do quadro.** `vec_entities::sync` corre em `render_loop/mod.rs:~7544` e o
   dreno da Hierarquia em `~9611`; `App::sync_instances` corre depois de tudo (`main.rs:1167`).
   Confirmar que uma edição do pen chega ao `gfx.vec_scene` **antes** do passe, e não num buffer
   vivo que só é assado ao soltar.
3. ⚠️ **O `Make Component` pode não ter sido o gesto usado.** O módulo vetorial tem os **verbos
   antigos** (Create/Place no painel Vector) que produzem `VecComponentMain`/`VecInstance` — o
   modelo DERIVADO, que propaga por construção e **não** passa por este passe. Os dois coexistem
   até a F4.6c, e a Hierarquia não distingue as duas espécies na tela.
4. ⚠️ **A instância pode ter capturado um override de forma no 1.º passe.** Se o clone e o mestre
   divergirem por um bit no 1.º quadro (um `effects` normalizado, um `fill` re-serializado), o passe
   lê *«só a instância mexeu»* e congela a peça contra a receita **para sempre**, em silêncio. O
   sintoma é exactamente o reportado. ⇒ **imprimir `ObjectInstance.overrides` da instância no fim do
   1.º passe** é a sonda mais barata que existe para isto. ⚠️ Na fixtura headless isto **não**
   acontece (o clone é byte-idêntico e o passe sai cedo), o que aponta para um passe que só existe
   no app a reescrever **um** dos dois paths — os cooks vivos do vetor são os candidatos, e a
   pergunta a fazer-lhes é se algum escreve **de volta** no `VecScene` em vez de produzir a
   `LiveGeometry` derivada.

### O instrumento que falta

⛔ Não há smoke que monte uma receita **vetorial** — o `PH2D_INSTANCE_SMOKE=1` monta um ragdoll de
sprites. *Um subsistema sem cena de smoke é um subsistema cujo report chega sempre como «não
funcionou», sem o meio caminho.* A primeira coisa a construir amanhã é a **cena 2**: mestre vetorial
+ 3 instâncias + a instrução impressa, no molde da cena 1.

---

## §15 Fecho da linha (DIRETRIZ §1.5.9)

**35 commits**, base `main @ 0f5ce8040`, worktree `Worktrees/line-components`, branch
`line/components`.

### Gate de fecho, corrido sobre o diff acumulado

| | |
|---|---|
| `cargo test -p ph2d-host-desktop` (bins + tests) | ✅ **4338** passaram · 0 falharam · 251 ignorados |
| `ph2d-ecs` · `-component-desc` · `-render` · `-script` · `-physics-ecs` · `-editor-core` · `-panel-hierarchy` · `-panel-inspector` · `-timeline` | ✅ todos verdes |
| `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings` | ✅ (3 min 19 s, exit 0) |
| `cargo fmt --all` | ✅ |
| `typos` (repo inteiro) | ✅ |
| Provas de mutação desta jornada | **34 mutações, 34 mortas** (11 + 10 + 11 da 1ª..4ª voltas do smoke, + 7 da F4.6a, + 5 da F4.6b — as contagens por bloco estão em cada §) |

⚠️ **Flake conhecida e PRÉ-EXISTENTE** (`CLAUDE.md` §5.0): a família
`flip_smooth::resample_measurement::precisao::orcamento` reprova sob fan-out e passa sozinha. Se ela
aparecer na integração, **re-rode sozinha antes de olhar para este diff**.

### O que o integrador tem de fazer

1. `scripts/foundational-integrate.sh` — a linha **tocou foundational** (`ph2d-ecs`,
   `ph2d-component-desc`, `ph2d-render`, `ph2d-script`, `ph2d-physics-ecs`, `ph2d-editor-core`, os
   dois painéis). A superfície de colisão está no §5, com os **contadores** que somam entre linhas.
2. ⚠️ **Apagar do `CLAUDE.md` §5 a frase «o `physics_ecs_c9` está POR RE-CAPTURAR»** — é um fantasma
   medido em 25/08 (os três hashes C9 comparam-se **entre si**, não contra baseline gravado).
3. **A linha do §5** (o roteador edita-se na integração, nunca da linha):

   > **Componentes / instâncias** — … ✅ **A F4 fechou até à F4.5 + os documentos possuídos**: o
   > mestre é inerte, instanciar/duplicar copiam a subárvore inteira (e agora o **documento
   > vetorial**), editar a receita muda as cópias no mesmo quadro, e os quatro verbos têm gesto na
   > Hierarquia. ⚠️ **Uma receita não está na cena** (marca derivada `MasterPiece` no extract) — a
   > `Visibility` **não** desce aos descendentes neste motor, e era nisso que a F4.5 assentava.
   > ⚠️ **Os PIXELS são um asset**: pintar uma cópia sobe à receita e chega a todas (fronteira:
   > *Detach* para pintar uma só). ⚠️ **Um objeto vazio tem anel, pega pelo centro, e o primeiro
   > clique é de quem já está selecionado.** ⛔⛔ **ABERTO: a propagação da GEOMETRIA vetorial não
   > passou no smoke** (handoff §14) · falta a F4.6c (matar o `InstanceLive` + migração) e a F4.7.

4. ⛔ **Não** integrar nem shipar sem ordem explícita do Enio (§0.7 do `CLAUDE.md`).
