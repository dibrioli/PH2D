//! **A ESCADA do `PROJECT_SCHEMA`** — o número do formato de arquivo, e como
//! ele chegou onde está.
//!
//! ⚠️ **Corte por RESPONSABILIDADE, e por LOC:** o irmão [`super::project`]
//! responde *"o que um arquivo de projeto contém, e como ele vai e volta do
//! disco"*; este responde *"que versão ele é, e por que"*. O `project.rs`
//! cruzou o teto de 600 do HR-18 com o degrau v77, e a escada é a metade que
//! cresce **um parágrafo por wave** — separá-la é o corte que não volta.
//!
//! ⚠️ **A escada mora COLADA à constante de propósito**, e a razão está escrita
//! no degrau v69: ele chegou ao `main` com a linha da escada AUSENTE, e *quem
//! conta o próximo degrau lê a escada, não o literal*. Mover as duas juntas
//! preserva isso; mover só o literal seria o defeito outra vez.
//!
//! ⚠️ **E o valor se CONTA contra o `main` do dia, nunca se escolhe** — esta
//! colisão passa **muda** quando duas linhas escrevem o MESMO número, porque o
//! git não sabe o que ele significa.

/// Versão do formato de arquivo de projeto. Bump ⇒ migração ou hard-break.
///
/// ⚠️ **Os degraus de v2 a v128 estão ARQUIVADOS**, verbatim, em QUATRO arquivos por faixa:
/// [`super::project_schema_history`] (`v2`..`v82`), [`super::project_schema_history_v83`]
/// (`v83`..`v98`), [`super::project_schema_history_v99`] (`v99`..`v111`) e
/// [`super::project_schema_history_v112`] (`v112`..`v128`). O corte é por IDADE e o tecto de 600
/// LOC do HR-18 é quem o pede — quatro vezes até hoje, e a última (2026-09-17) foi a primeira em
/// que o gatilho não foi uma linha mas a ACUMULAÇÃO de uma rodada: seis linhas puseram dezasseis
/// degraus, e nenhuma estoura o tecto sozinha.
/// O que se lê para contar o próximo degrau é a ponta, e a ponta é o que ficou aqui.
///
/// # 128 -> 129 — as TAGS: a árvore no `ProjectState` e o alvo do `SignalAction` (TOP-20 #9)
///
/// ⚠️⚠️ **DUAS mudanças num degrau só, e nenhuma delas é aditiva no fio:**
///
/// 1. O `ProjectState` ganhou `tags: Vec<u8>` — o blob auto-versionado do
///    `ph2d_app_components::tags_doc`. ⛔ **Ele cai no MEIO do fluxo de bytes**, porque o `state` é o
///    PRIMEIRO campo do `ProjectFile`: um v128 lido com o tipo vivo não chega ao fim dos bytes, ele
///    lê *lixo bem-formado* a partir dali. É exactamente o que o degrau `105` documentou quando a
///    biblioteca entrou, e é por isso que este degrau traz um tipo CONGELADO
///    ([`crate::project_migrate::ProjectFileV128`]) em vez de reler com o vivo.
/// 2. O `SignalAction` ganhou `target_by` no fim — *por nome* ou *por tag*. Os bytes dele vivem
///    DENTRO de um `ComponentBlob`, que o parse do ficheiro atravessa sem olhar, então a migração é
///    uma travessia das linhas do snapshot (o precedente do `crate::project_migrate_sprite`).
///
/// ⭐ **E esta é a ÚNICA vez que a árvore de tags paga o número.** A versão do blob mora dentro dele
/// (`TAGS_DOC_VERSION`), então uma descrição por tag, uma cor ou uma ordem manual custam só aquele —
/// o precedente do `CATALOG_DOC_VERSION`, do `timeline` e do `sculpt`.
///
/// ⚠️ **COM degrau de migração, ao contrário dos últimos onze** (a decisão do Enio de 26/08 —
/// *«não há projetos salvos»* — valia para ficheiros de antes do navegador de assets). O `128` é o
/// schema que a integração de 10/09 pôs no `main` com o `Timer` e o `SignalActions` dentro: um
/// projecto gravado desde então TEM tabelas de acções, e recusá-lo apagaria autoria que existe.
///
/// ⚠️ **A tripla NÃO vê este degrau** — é a **décima terceira** vez: o `FlipDoc` e a `VecScene` não
/// se mexeram, e o que mudou foi o `ProjectFile` e os bytes de um blob.
/// # 129 -> 130 — a FÁBRICA e o CICLO DE VIDA (TOP-20 #11 e #12, `line/components`)
///
/// TRÊS componentes registados novos: `ph2d::ecs::Factory`, `ph2d::ecs::Lifetime` e
/// `ph2d::ecs::DestroyOutside`. Mesmo mecanismo dos degraus `123`, `125`, `126` e `127`: um
/// `ComponentBlob` de `type_id` desconhecido **recusa o load inteiro**, e o degrau é o que
/// transforma isso em *«este ficheiro é de outra versão»* em vez de *«type id desconhecido»* no
/// meio da travessia.
///
/// ⛔⛔ **E há um QUARTO componente que NÃO está no registo, de propósito: o `ph2d::ecs::Spawned`.**
/// Ele marca o que uma fábrica pôs na cena, e a lei da wave é *o que nasce numa corrida não é
/// documento* — o `world_to_snapshot` **poda** essas subárvores, então um `.ph2dproj` gravado a
/// meio de uma corrida com mil cópias vivas é byte-a-byte igual ao mesmo projecto parado. ⚠️ É por
/// isso que este degrau vale `+1` e não `+2`: o número mede o que o FICHEIRO passa a conter.
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 — e aqui com a razão extra de que
/// os três são **aditivos**: um v129 não tem nenhum deles, logo lê-se inteiro por este binário. O
/// degrau existe para o sentido contrário (um v130 com uma fábrica dentro, lido por um binário
/// anterior), que é o que recusa em voz alta.
///
/// ⚠️ **A tripla NÃO vê este degrau** — é a **décima quarta** vez: os componentes viajam em
/// `ComponentBlob`s, que para ela são opacos.
/// # 130 -> 131 — o MOVER DE VISTA DE CIMA (TOP-20 #13, `line/components`)
///
/// UM componente registado novo: `ph2d::physics::TopDownPlayer`. Mesmo mecanismo dos degraus
/// `123`, `125`, `126`, `127` e `130` — um `ComponentBlob` de `type_id` desconhecido **recusa o
/// load inteiro**, e o degrau transforma isso em *«este ficheiro é de outra versão»* em vez de
/// *«type id desconhecido»* a meio da travessia.
///
/// ⚠️⚠️ **E o doc do `PlatformPlayer` diz o CONTRÁRIO disto, por escrito** (*«Componente NOVO ⇒
/// blob-key própria ⇒ `PROJECT_SCHEMA` NÃO bumpa»*, o precedente do `PhysicsJoint`/W3). Ele é
/// anterior aos cinco degraus acima, que estabeleceram a regra de hoje. *Uma nota que descreve a
/// casa de outra época lê-se exactamente como uma que descreve a de agora* — foi corrigida no
/// mesmo commit que escreveu este degrau.
///
/// ⛔⛔ **O `TopDownState` NÃO é componente, e a ausência é a decisão:** ele é a velocidade que as
/// rampas acumulam, muda por tique, e um campo assim dentro de um componente registado faria o
/// `canonicalize` do undo ver **cada quadro como um passo** (a lei do módulo de física, medida na
/// auditoria da §11 do Sprite). Ele vive na ponte, dentro do `ControllerMemory` que entra no anel
/// de checkpoints — é isso que o faz sobreviver a um scrub.
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 e pela razão aditiva: um v130 não
/// tem o componente, logo lê-se inteiro por este binário. O degrau existe para o sentido contrário.
///
/// ⚠️ **A tripla NÃO vê este degrau** — é a **décima quinta** vez.
/// # 131 -> 132 — o PROJÉCTIL (TOP-20 #14, `line/components`)
///
/// UM componente registado novo: `ph2d::physics::ProjectileMotion`. Mesmo mecanismo dos degraus
/// `123`, `125`, `126`, `127`, `130` e `131`.
///
/// ⭐⭐⭐ **E ele só existe porque a composição foi MEDIDA primeiro** (§5.0). A sonda
/// `ph2d-physics-ecs/tests/it/mede_o_que_a_composicao_ja_da.rs` mostrou que um corpo **dinâmico**
/// com `restitution = 1` **já ricocheteia exactamente** (razão `1,000` em todos os ângulos) — logo
/// o ricochete não é a razão de este componente existir. A razão é a linha seguinte da tabela: o
/// mesmo tiro contra uma **caixa leve** sai a `10,252` em vez de `12,001` e por outro caminho.
/// *Um projéctil dinâmico é participante da física; uma bala de arcade não tem massa.*
///
/// ⛔⛔ **O `ProjectileState` NÃO é componente, pela mesma razão do `TopDownState`:** velocidade,
/// metros percorridos e saltos gastos mudam por tique, e um campo assim num componente registado
/// faria o undo ver cada quadro como um passo. Ele viaja no MESMO `ControllerMemory` — e foi ele
/// que **provou** que aquele tipo funciona: acrescentá-lo não compilou até passar pelo `record` e
/// pelo `seed`, que é exactamente o que o doc do `player_state` prometia.
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 e pela razão aditiva: um v131 não
/// tem o componente, logo lê-se inteiro por este binário.
///
/// ⚠️ **A tripla NÃO vê este degrau** — é a **décima sexta** vez.
/// # 132 -> 133 — o CÉREBRO AUTORÁVEL (TOP-20 #15, `line/components`)
///
/// UM componente registado novo: `ph2d::ecs::StateMachine` — os estados, as setas e o inicial.
/// Mesmo mecanismo dos degraus `123`, `125`, `126`, `127`, `130`, `131` e `132`.
///
/// ⭐⭐⭐ **E ele só existe porque a composição foi MEDIDA primeiro** (§5.0). A sonda
/// `ph2d-ecs/tests/it/mede_o_que_a_composicao_ja_da_ao_cerebro.rs` respondeu **NÃO** em três
/// sítios: o mesmo sinal com duas linhas contraditórias dispara **as duas** (nada escolhe uma — *e
/// escolher uma é o que um estado é*), a `SignalAction` tem **5** campos e **zero** são uma guarda,
/// e dos **7** verbos com sink **nenhum** emite um sinal. *O que faltava não eram acções: era a
/// MEMÓRIA de em que estado se está.*
///
/// ⛔⛔ **O `StateMachineRuntime` NÃO é componente registado, pela mesma razão do `TimerRuntime`:**
/// registá-lo faria **cada transição** virar um passo de `Ctrl+Z`. E a cerca é o **TIPO** — ele não
/// deriva `Serialize`, logo a linha do registo nem compila. Ele é reposto ao rebobinar pela porta
/// do `ph2d_ecs::rewind_runtime`, escrita na **W0** desta mesma jornada por o defeito existir já
/// nos três runtimes que havia.
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 e pela razão aditiva: um v132 não
/// tem o componente, logo lê-se inteiro por este binário.
///
/// ⚠️ **A tripla NÃO vê este degrau** — é a **décima sétima** vez.
/// # 133 -> 134 — o SCRIPT DO ARTISTA passa a ser GRAVADO (TOP-20 #16, `line/components`)
///
/// ⚠️ **Nenhum tipo novo, e o número sobe mesmo assim:** o `ph2d::script::LuauScript` existia desde
/// o M14 e o registador dele **não era chamado no boot** — o `WorldSnapshot` descartava-o EM
/// SILÊNCIO. Entrar no `build_component_registry` é o que muda o que o FICHEIRO contém (um blob por
/// objecto com script), e é isso que o número mede.
///
/// ⚠️ **A forma do componente também mudou na mesma wave** (`{ bytecode, lateral_key }` →
/// `{ source, own }`), e ⛔ **isso não pede migração**: a forma velha **nunca foi gravada**. A
/// antiga carregava `entity.to_bits()` dentro dos bytes — o veneno do undo que o §5 proíbe.
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 e pela razão aditiva: um v133 não
/// tem blob de script, logo lê-se inteiro por este binário.
///
/// ⚠️ **A tripla NÃO vê este degrau** — é a **décima oitava** vez.
///
/// # 134 -> 135 — o EMISSOR DE PARTÍCULAS de um objecto (TOP-20 #18, `line/components`)
///
/// O `ph2d::ecs::ParticleEmitter` passa a ser gravado: sem o degrau, um projecto do v134 lido por
/// este binário e regravado ficaria igual, mas um v135 lido por um binário velho **perderia o
/// jacto em silêncio** — que é exactamente o que o número existe para impedir.
///
/// ⛔ **O que CORRE não está no ficheiro**: as partículas vivas, o relógio local e os segmentos de
/// emissão são da corrida (a lei do `Spawned`), e a cerca é o TIPO.
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 e pela razão aditiva: um v134 não
/// tem blob de emissor, logo lê-se inteiro por este binário.
///
/// ⚠️ **A tripla NÃO vê este degrau** — é a **décima nona** vez.
/// # 135 -> 136 — o osso DOBRA: `Bone` ganha `segments` e a curvatura (`line/Vector`)
///
/// DOIS campos novos no `ph2d::skeleton::Bone`: `segments: u8` e `curve: Bend` (as duas alças do
/// *Bendy Bone*). É a F8 da fila do esqueleto.
///
/// ⚠️⚠️ **O degrau é obrigatório e a razão é o postcard, não o campo** — a mesma lei dos degraus
/// `112` e `127`: ele é **posicional**, logo um ficheiro gravado com dois campos seria lido com
/// quatro **em silêncio**, com os bytes do vizinho a entrarem no `segments`. Com o degrau, o load
/// **recusa em voz alta**.
///
/// ⭐⭐ **O NASCIMENTO É O NEUTRO, e não por promessa:** `segments = 1` **ou** a curvatura recta
/// fazem a fábrica de sub-ossos colapsar num osso só, sem passar pelos frames, e o resultado é
/// `assert_eq!`-idêntico ao que a `SkinBone::new` devolvia (gates
/// `a_straight_bone_is_still_exactly_one_bone` e `a_skin_of_straight_bones_did_not_move_a_single_bit`,
/// em `ph2d-skeleton`). ⇒ todo rig já autorado deforma-se **ao bit** como antes; o degrau é só
/// sobre o FORMATO.
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08.
///
/// ⚠️ **A tripla NÃO vê este degrau** — é a **décima terceira** vez nesta escada, e pela razão de
/// sempre: os componentes viajam em `ComponentBlob`s, que para ela são opacos.
///
/// # 136 -> 137 — a malha de uma imagem presa leva os PESOS dentro (`line/Vector`)
///
/// Os bytes opacos do `SkinBind::source` de uma **imagem** deixaram de ser uma
/// `ph2d_poly2d::Mesh2d` e passaram a ser uma `ph2d_skeleton_live::skinned_mesh::SkinnedMesh`
/// (`{ mesh, pesos }`): os *Bounded Biharmonic Weights* resolvem-se **uma vez ao prender**.
///
/// ⚠️⚠️ **Obrigatório, e a razão é o postcard** (como nos `112`, `127` e `136`): ele é
/// **posicional**, e a struct nova é a antiga **seguida** do vector ⇒ um ficheiro velho acaba onde a
/// nova espera o comprimento da tabela, e os bytes seguintes seriam lidos como pesos.
///
/// ⛔ **Uma forma VECTORIAL não muda de formato** — guarda um `VecPath` e fica na lei euclidiana
/// derivada (o padrão-ouro precisa de uma malha do domínio, e uma Bézier não tem uma). *Limite
/// NOMEADO:* um rig com as duas mídias tem hoje duas leis.
///
/// ⛔ **Sem degrau de migração** (Enio, 26/08) — derivar os pesos no load seria correr o solver
/// dentro do caminho de abrir um ficheiro. ⚠️ **A tripla NÃO o vê** (14.ª vez): estes bytes estão
/// DENTRO do `source` de um `ComponentBlob`, opaco duas vezes.
/// # 137 -> 138 — a forma VECTORIAL presa também leva os pesos dentro (`line/Vector`)
///
/// O degrau `137` deixou a 1.ª mídia na lei euclidiana **por limite nomeado** (*«o padrão-ouro
/// precisa de uma malha do domínio, e uma Bézier não tem uma»*). Ela tem: o **interior** dos
/// contornos fechados. ⇒ o `SkinBind::source` de um `VecPathRef` deixou de ser um `VecPath` e passou
/// a ser um `ph2d_skeleton_live::skinned_mesh::SkinnedPath` (`{ path, pesos }`), com um peso por
/// **ponto de controlo**.
///
/// ⚠️⚠️ **Obrigatório, e a razão é o postcard** — a mesma dos degraus `112`, `127`, `136` e `137`:
/// ele é **posicional**, e a struct nova é a antiga **seguida** do vector.
///
/// ⭐ **E é o degrau que fecha a divergência que o `137` abriu:** as duas mídias voltam a responder à
/// MESMA lei. Um rig com um braço vectorial e um braço em imagem deixou de ter duas.
///
/// ⛔ **Um caminho ABERTO fica na lei derivada, e é uma resposta**: uma linha tem área zero, logo não
/// há domínio para a energia. A tabela sai **vazia**, que é a mesma forma que uma imagem sem pesos
/// usa.
///
/// ⛔ **Sem degrau de migração** (Enio, 26/08). ⚠️ **A tripla NÃO o vê** (15.ª vez): os bytes estão
/// DENTRO do `source` de um `ComponentBlob`.
///
/// # `138 → 139` — ⭐⭐⭐ o osso passou a dizer **DE ONDE VÊM AS ALÇAS** de curvatura
///
/// O `Bone` ganhou `handles` ([`ph2d_skeleton::bend::Handles`]): `Authored` (o nascimento, e o que
/// todo osso sempre foi) ou `Auto`, em que as duas alças saem das tangentes dos ossos **vizinhos** e
/// a corrente inteira vira uma curva lisa. É o último item aberto da F8.
///
/// ⚠️⚠️ **Obrigatório, e a razão é o postcard** — a mesma dos degraus `112`, `127`, `136`, `137` e
/// `138`: ele é **posicional**, e um campo APENDADO a um componente faz um ficheiro velho ser lido
/// com um campo a mais, comendo os bytes do vizinho. ⛔ O `#[serde(default)]` no campo **não** o
/// salva: ele serve formatos com nomes, e o postcard não tem nenhum.
///
/// ⛔ **Sem degrau de migração** (a mesma decisão do Enio de 26/08 — não há projectos gravados).
/// ⚠️ **A tripla NÃO o vê** (16.ª vez): os bytes mudaram dentro de um `ComponentBlob`.
///
/// # `139 → 140` — ⭐⭐⭐ o osso passou a dizer **QUEM MANDA NA PONTA** da curva
///
/// O `Bone` ganhou `curve_tip` ([`ph2d_skeleton_ecs::CurveTip`]): `Chain` (o nascimento, e a lei que
/// sempre existiu — o único filho-osso manda, e com zero ou mais de um a ponta fica recta),
/// `Straight` (**ninguém** manda, mesmo havendo um filho) ou `Bone(StableId)` (**este** filho manda
/// — o *custom handle* do Blender). Ordem do dono, 2026-09-16, depois de ver a cena da bifurcação:
/// *«sim, escolher qual dos vários filhos manda na curva. E quero que o modo atual (ninguém manda na
/// curva) seja uma das opções»*.
///
/// ⚠️⚠️ **Obrigatório, e a razão é o postcard** — a mesma dos degraus `112`, `127`, `136`, `137`,
/// `138` e `139`: ele é **posicional**, e um campo APENDADO a um componente faz um ficheiro velho
/// ser lido com um campo a mais, comendo os bytes do vizinho.
///
/// ⭐ **O filho é um [`ph2d_ecs::StableId`] e não os bits dele** — a mesma cerca do `IkGoal::target`,
/// pelo mesmo defeito medido: bits de alocação não sobrevivem ao respawn do undo.
///
/// ⛔ **Sem degrau de migração** (a mesma decisão do Enio de 26/08).
/// ⚠️ **A tripla NÃO o vê** (17.ª vez): os bytes mudaram dentro de um `ComponentBlob`.
/// # `140 → 141` — o MATERIAL de uma forma ganha o BRILHO PRÓPRIO (`docs/Render3d/05` §20)
///
/// O `ph2d::field::FieldMaterial` passou de `5` para `9` números: `emission: f32` e
/// `emission_color: [f32; 3]` foram **apendados** aos três da cor base, à rugosidade e ao metal.
///
/// ⚠️⚠️ **Apendar campos é aditivo NUM sentido só, e nem esse.** O postcard é **posicional e sem
/// comprimento**: um blob v128 tem `5 × 4 = 20` bytes e este binário pede `36`, então a leitura de
/// um material gravado antes desta wave sai com *«Hit the end of buffer»* — e, pior, o `ComponentBlob`
/// é **opaco ao parse do `ProjectFile`**, logo o erro chegaria no meio da travessia dos componentes
/// em vez de no cabeçalho. O degrau transforma isso num **erro de versão**, que é a frase que diz ao
/// dono o que aconteceu.
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08 (*«não há projetos salvos»*) —
/// e aqui com a razão extra de que o componente é `register_default`: a ausência dele **já** é o
/// material de omissão, logo uma peça anterior a 13/09 (quando ele nasceu) nunca o carrega.
///
/// ⚠️ **A tripla NÃO vê este degrau** — é a **décima oitava** vez, e pela razão de sempre: os
/// componentes viajam em `ComponentBlob`s, que para ela são opacos.
///
/// ⚠️ **E o `FIELD_DOC_VERSION` NÃO se mexe**, o que parece estranho num degrau que fala de
/// material: o documento do campo é **geometria** — é ele que a marcha compila —, e uma cor não muda
/// uma distância. *Os dois números medem coisas diferentes, e subir o errado esconderia o certo.*
///
/// ⚠️⚠️ **Este degrau foi RE-CONTADO na integração de 2026-09-17.** A `line/3DModeling` escreveu-o
/// como `128 → 129` sobre o `main` em que ela nasceu; quando ela aterrou, o `main` estava em `140`
/// (a `line/components` e a `line/Vector` puseram doze degraus no meio). ⛔ *O valor certo não
/// estava em nenhum dos dois lados do conflito* — ele CONTA-SE contra a árvore em que se aterra, e
/// a tripla do ficheiro irmão sobe no mesmo commit.
///
/// # `141 → 142` — o MATERIAL ganha o VERNIZ (`docs/Render3d/05` §21)
///
/// O `ph2d::field::FieldMaterial` passou de `9` para `16` números: `coat`, `coat_roughness`,
/// `coat_color: [f32; 3]`, `coat_ior` e `coat_darkening`, **apendados**.
///
/// ⚠️ **Mesmo mecanismo do degrau anterior, um dia depois:** o postcard é posicional e sem
/// comprimento, e um blob v129 tem `36` bytes onde este binario pede `64`.
///
/// ⚠️ **A tripla NAO ve^ este degrau** — a SEXTA vez (99, 100, 114, 119, 129 e este).
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 — e aqui com a razão extra de que o
/// componente e' `register_default`: uma peça anterior a 13/09 não carrega material nenhum.
///
/// # `142 → 143` — o MATERIAL FECHA: as ultimas cinco entradas do OpenPBR (`docs/Render3d/05` §22)
///
/// O `ph2d::field::FieldMaterial` passou de `16` para `23` numeros — `base_weight`,
/// `base_diffuse_roughness`, `specular_weight`, `specular_color: [f32; 3]` e `specular_ior` —, e
/// ⚠️⚠️ **os campos foram RE-ORDENADOS para a ordem da nodedef**, o que e' uma quebra de layout
/// muito mais severa do que apendar: um blob v130 lido por este binario poe a rugosidade onde mora
/// o peso da base. *Postcard nao tem nome de campo para reclamar.*
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08.
///
/// ⭐ **E este e' o ULTIMO degrau que o material pede:** sao as `15` entradas do OpenPBR, e nao ha
/// mais nenhuma para apender.
/// # `143 → 144` — o PERFIL ganha os ARCOS (`docs/Render3d/06_auditoria_do_vaso.md`)
///
/// O `ph2d_field::Profile` ganhou um campo `arcs` — a decomposição exacta em rectas e ARCOS, para
/// uma quina arredondada deixar de ser oito segmentos rectos na fita que a marcha avalia por pixel.
/// O `FIELD_DOC_VERSION` subiu **22 -> 23**.
///
/// ⚠️ **Sobe por arrasto pelo mesmo caminho do 104:** o `Profile` viaja dentro de uma
/// `ph2d_field::Primitive` (`Extrude`, `Revolve`, `Polygon`), que viaja **posicionalmente** dentro
/// do blob do componente `ph2d_field_ecs::FieldNode`, que está no `WorldSnapshot`. É a regra dos
/// degraus 109/110: **acrescentar um CAMPO a uma struct que já se grava** muda os bytes de valores
/// gravados, ao contrário de apendar uma variante no fim de um `enum`.
///
/// ⭐ **O golden de forma apanhou-o**, que é o instrumento que este ficheiro nomeia para isto:
/// `the_shape_of_a_saved_profile_is_pinned` foi de `90` para **`92`** bytes (um `Vec` de um `Vec`
/// vazio = dois bytes).
///
/// ⭐ **A aparência de um documento velho não muda**: ao ler, `arcs` fica vazio, e um perfil sem
/// arcos é avaliado pelo caminho de sempre, **ao bit** (gate `um_perfil_sem_arcos_da_o_campo_de_sempre`).
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 — um v131 é **recusado em voz alta**.
/// # `144 → 145` — o HUD (TOP-20 #20, `docs/Components/15_plano_hud.md`)
///
/// Quatro componentes novos no registo — `UiCanvas` (a caixa de referência e o `Fit`), `UiLabel`
/// (a fonte do número), `UiButton` (o sinal que ele publica) e `Counter` (o nome e o `start`) —,
/// e uma variante **APENDADA** no fim do `SignalVerb` (`AddToCounter`).
///
/// ⚠️ **Os dois lados desta linha têm regimes DIFERENTES, e é por isso que o degrau existe:**
/// apendar uma variante no fim de um `enum` é compatível (um ficheiro velho nunca a escreveu),
/// mas um componente NOVO faz o `WorldSnapshot` de um binário novo carregar blobs que um binário
/// velho não sabe nomear. É a mesma regra dos degraus das tags e da fábrica.
///
/// ⛔ **O valor VIVO de um contador NÃO entra aqui, e a ausência é a lei:** o `CounterRuntime` não
/// deriva `Serialize` e não está registado — se estivesse, **cada ponto marcado** seria um passo
/// de `Ctrl+Z` e ficaria dentro do ficheiro gravado.
///
/// ⛔ **E a pose conduzida do canvas também não:** ela é reescrita a cada quadro pela fase do HUD e
/// passa pelo ledger do `preview_drive`, como a do solver e a do script.
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 — um ficheiro anterior é recusado
/// em voz alta.
/// # `145 → 146` — a CUTSCENE (TOP-20 #19, `docs/Components/16_plano_sequence_player.md`)
///
/// **UM** componente novo no registo: o `SequencePlayer`, que carrega o **nome** do container da
/// timeline que este objecto toca.
///
/// ⛔⛔ **Um componente e um degrau — e o que NÃO entra é a wave inteira.** A medição do plano 16
/// §1-bis mostrou que o relógio de corrida **já existe** no `Timer` (duração · repetir · o sinal a
/// cada disparo · um `TimerRuntime` cujo `progress()` é derivado), que um sinal já o arranca
/// (`StartTimer`) e que o `rewind_runtime` já o faz renascer. Um `SequenceRuntime` seria um
/// **segundo relógio**, e a cutscene correria num tempo e anunciar-se-ia noutro.
///
/// ⚠️ **O nome, nunca o índice:** os containers vivem num `Vec` do `TimelineDoc`, e apagar o de
/// cima renumera os de baixo — um índice gravado aqui passaria a tocar a cutscene do vizinho **em
/// silêncio**. É a lei que a casa já escreve para o `Counter`, o `Timer` e a `Tag`.
///
/// ⛔ **A timeline NÃO muda de forma:** o container já existe (ADR-0133) e ler um pelo nome não
/// move um byte do `DOC_VERSION` dela.
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 — um ficheiro anterior é recusado
/// em voz alta.
pub(crate) const PROJECT_SCHEMA: u32 = 146;
