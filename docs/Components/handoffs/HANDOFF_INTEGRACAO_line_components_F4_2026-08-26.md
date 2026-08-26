# Handoff de integração — `line/components`, 2026-08-26 (**F4.1..F4.4**: o mestre é inerte, instanciar/duplicar copiam de verdade, editar a receita muda as instâncias, e a excepção do artista sobrevive)

> DIRETRIZ §1.5.9. Sucessor do
> [handoff de 25/08](HANDOFF_INTEGRACAO_line_components_F1_F2_F3_2026-08-25.md) (F1+F2+F3).
> ⚠️ **A fase F4 NÃO está fechada** — este handoff cobre as quatro primeiras fatias dela, que são
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
| Fatias entregues | **F4.1** (o mestre é inerte) · **F4.2** (instanciar + duplicar profundo) · **F4.3** (sync vivo) · **F4.4** (overrides) |

---

## §2 O que MUDA para quem usa o app

| gesto | antes | **hoje** |
|---|---|---|
| duplicar um objeto na Hierarquia | copiava `Transform`+`Sprite`+`Name` e **nenhum filho** | ✅ a **subárvore inteira**, todo componente, identidade nova |
| duplicar um corpo com junta | a cópia ficava **solta** (a junta nomeava o original) | ✅ a junta da cópia prende **os corpos dela** |
| um objeto marcado como receita | simulava como qualquer outro | ✅ **não cai** — receita não é objeto de cena |
| editar uma peça da receita | não existia | ✅ **todas as instâncias mudam** no mesmo quadro |
| editar uma peça de UMA cópia | não existia | ✅ vira **excepção**: a receita já não a leva |

⚠️ **O que ainda NÃO existe:** o gesto *«criar componente»* e o botão *«Instanciar»*. A porta
`instantiate::instantiate_master` existe, é testada e é alcançada **pelo smoke**; pô-la num menu é
a fatia **F4.5**.

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

⚠️ **Nenhum contrato congelado (§6 do CLAUDE.md) foi tocado.**

---

## §6 Gate de fecho (corrido em 2026-08-26)

| | |
|---|---|
| `cargo test -p ph2d-host-desktop --bins` | ✅ **3658** passaram · 0 falharam · 245 ignorados |
| `cargo test -p ph2d-host-desktop --tests` | ✅ (⚠️ o censo de dois lados apanhou o `ObjectInstance` sem descritor — **ele fez o trabalho dele**) |
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
