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
/// ⚠️ **Os degraus de v2 a v98 estão ARQUIVADOS**, verbatim, em dois arquivos por faixa:
/// [`super::project_schema_history`] (`v2`..`v82`), [`super::project_schema_history_v83`]
/// (`v83`..`v98`) e [`super::project_schema_history_v99`] (`v99`..`v111`). O corte é por IDADE e o teto de 600 LOC do HR-18 é quem o pede — duas vezes até
/// hoje: em `v82`, e outra vez em 2026-09-07, quando o degrau `122` levou este arquivo a `618`
/// linhas (e mover a faixa para dentro do primeiro arquivo o punha em `730`).
/// O que se lê para contar o próximo degrau é a ponta, e a ponta é o que ficou aqui.
///
/// # 111 -> 112 — a TAXONOMIA da biblioteca de assets (plano 07, wave A3)
///
/// O `ProjectFile` ganhou `catalogs: Vec<u8>` — um blob **auto-versionado**
/// (`project_catalogs::CATALOG_DOC_VERSION`) com os catálogos e as atribuições `asset → catálogo`.
///
/// ⭐ **Do lado ADITIVO da regra, e mesmo assim o número SOBE.** O campo entra no fim e os índices
/// anteriores não se mexem, então um v103 lido por v104 estaria correcto até ao último campo — e é
/// exactamente aí que ele acaba: o postcard chega ao fim dos bytes (`Hit the end of buffer`) e
/// falha **longe da causa**. O número transforma isso num erro de VERSÃO, que se lê.
///
/// ⚠️ **E este é o ÚNICO bump que a taxonomia paga.** A versão do blob mora dentro dele, então
/// acrescentar um campo à taxonomia — cor de catálogo, ordem manual, o que vier — custa só o
/// `CATALOG_DOC_VERSION`. É o precedente do `timeline`, do `sculpt` e do `pattern_art`.
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 (*«não há projetos salvos»*).
///
/// # 105 — a biblioteca MUDA-SE para dentro do undo (Enio, 2026-08-30)
///
/// *«deveria ter undo/redo no painel inclusive em del»*. ⇒ a taxonomia sai do `ProjectFile` e
/// passa a viver no **`ProjectState`**, que é a unidade que o Ctrl+Z restaura, acompanhada das
/// **lápides** — as imagens que o artista mandou sair da biblioteca.
///
/// ⛔⛔ **Este degrau NÃO é aditivo, e é por isso que ele é o mais perigoso desta escada desde o
/// 102.** Um campo saiu do meio do `ProjectFile` e outro entrou no meio do `ProjectState`: os
/// bytes de um v104 não desaparecem, eles passam a **significar outra coisa**. Postcard é
/// posicional e leria a taxonomia velha como se fosse outro campo, **sem erro nenhum**.
///
/// ⚠️ **Duas respostas à mesma pergunta era a alternativa**, e é o que o número compra: manter o
/// campo no ficheiro *e* no estado deixaria o load a escolher qual acreditar.
///
/// ⚠️ **A versão do blob continua a mandar na taxonomia**: o `CATALOG_DOC_VERSION` mora dentro dos
/// bytes, então acrescentar-lhe um campo continua a custar zero aqui.
///
/// # 113 -> 114 — a UNIDADE DE ÂNGULO (`line/UIUX`)
///
/// As `SavedSettings` ganharam `display_angle: u8` — a irmã do `display_unit`, para o pedido do
/// Enio de 2026-08-30: *"devemos ter ambas as opções no app (px e metros, graus e radianos)"*.
///
/// ⚠️ **Campo APENDADO ao fim de uma struct ⇒ quebra dura**, o mesmo mecanismo do degrau v80: o
/// postcard é posicional, então um v103 lido por este layout fica **sem bytes** no último campo.
/// ⛔ `#[serde(default)]` **não** o salva — num formato não-auto-descritivo ele não sabe que o
/// campo faltou, sabe que o *stream* acabou.
///
/// ⚠️ **A tripla NÃO vê este degrau**, e é a terceira vez que isso acontece nesta escada (99, 100,
/// e agora): ela mede a forma do `FlipDoc` e da `VecScene`, e nenhuma das duas mudou — o que mudou
/// foi o `ProjectFile`. *Está escrito aqui porque a próxima pessoa olha para a tripla primeiro.*
///
/// ⭐ **O default preserva o comportamento anterior ao bit:** `DisplayAngle::Degrees` é o que os
/// sítios faziam com `to_degrees()` escrito à mão. Quem nunca abrir o menu não vê diferença.
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08.
/// # 114 -> 115 — A OPACIDADE E A MISTURA DA FORMA (`line/Vector`)
///
/// O [`ph2d_vec_scene::VecPath`] ganhou `opacity` + `blend` (`VEC_SCENE_SCHEMA_VERSION` 18 -> 19) —
/// o item 2 do estudo 42, e a metade que faltava ao report do Enio de 2026-09-04 (*"o painel não
/// mostra..."* ⇒ não havia o que mostrar: a forma não tinha opacidade própria).
///
/// ⚠️ **Campos APENDADOS ao fim de uma struct ⇒ quebra dura nos dois sentidos** — o mesmo
/// mecanismo do v80 e do v114: postcard é posicional e não sinaliza ausência, então um save v18 da
/// cena lido por este layout **acaba os bytes** no campo novo, e um v19 lido por um binário velho
/// traz bytes a mais. ⭐ **A tripla VÊ este degrau** (ao contrário dos v99/v100/v114): o que mudou
/// foi a forma da `VecScene`, que é exactamente o que ela mede.
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 2026-08-26.
///
/// # 115 -> 116 — A PILHA DE APARÊNCIA (`line/Vector`)
///
/// O [`ph2d_vec_scene::VecPath`] ganhou `paints` (`VEC_SCENE_SCHEMA_VERSION` 19 -> 20) — N
/// preenchimentos e N contornos numa forma (o item 4 do estudo 42). Sem ela, cada camada de estilo
/// obriga a **duplicar o objecto**, e duas cópias de uma forma são duas geometrias que divergem no
/// primeiro ponto que o artista mexe.
///
/// ⚠️ **Um `Vec` apendado ao fim ⇒ a mesma quebra dura dos v114 e v115**: o postcard é posicional e
/// não sinaliza ausência. ⭐ Vazio é o neutro e custa **1 byte** de comprimento zero, então uma cena
/// que nunca lhe toque escreve o que escrevia mais esse byte — e desenha byte a byte igual.
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão de 2026-08-26.
/// # 116 -> 117 — ONDE cada camada da pilha desenha (`line/Vector`)
///
/// O [`ph2d_vec_scene::PaintEntry`] ganhou `offset` (`VEC_SCENE_SCHEMA_VERSION` 20 -> 21) — o
/// deslocamento de UMA camada, relativo à forma. Report do Enio, 2026-09-05: *"o fill não deveria
/// ter um offset? não seria mais útil?"* Sem ele, dois preenchimentos desenham nos **mesmos
/// pixels**, e as duas coisas que um artista faz com um segundo preenchimento — a sombra dura e a
/// profundidade de um rótulo — são inexprimíveis sem duplicar a forma outra vez.
///
/// ⚠️ **O campo é apendado ao fim de `PaintEntry`, e não ao fim do `VecPath`** — a diferença é o
/// modo de falha: um save v116 lido por este layout rebenta **dentro** do `Vec` de camadas (o
/// postcard lê elemento a elemento), e não no fim do ficheiro. ⭐ Uma forma **sem pilha** não tem
/// entradas ⇒ zero bytes novos, e o ficheiro é byte a byte o de antes.
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão de 2026-08-26.
/// # 117 -> 118 — O OFFSET DE CAD de uma camada (`line/Vector`)
///
/// O [`ph2d_vec_scene::PaintEntry`] ganhou `dilate` + `dilate_join`
/// (`VEC_SCENE_SCHEMA_VERSION` 21 -> 22): a silhueta de UMA camada cresce ou encolhe. Pedido do
/// Enio, 2026-09-05 (*"o offset do cad, contraindo e dilatando"*), e é o que faz um adesivo ou um
/// selo sem duplicar a forma.
///
/// ⚠️ Mesmo modo de falha do degrau anterior: os campos são apendados ao fim de `PaintEntry`, então
/// um save v117 rebenta **dentro** do `Vec` de camadas. ⭐ Uma forma sem pilha não tem entradas ⇒
/// zero bytes novos.
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão de 2026-08-26.
/// # 118 -> 119 — a excepção sem alvo SABE de que peça era (`line/components`, F5 critério 3)
///
/// O `ObjectInstance.orphans` era `BTreeMap<OverrideKey, Vec<u8>>` e passou a
/// `BTreeMap<OverrideKey, OrphanOverride>` — os mesmos bytes **mais** o `Name` que a peça tinha
/// quando morreu.
///
/// ⚠️ **O nome não é uma segunda fonte, e o argumento é o MESMO que já justificou os bytes** (F5.3):
/// a refutação da F4.4 — *«guardar o valor cria duas fontes»* — vale enquanto a peça **existe**.
/// Uma peça órfã não existe: o mestre apagou-a e a F5.1 tirou-a da cópia a seguir. ⇒ não há segunda
/// fonte, há a única. Sem ele o painel pode dizer *«há três»* e **nunca** *«quais três»*, com o
/// botão que apaga as três ao lado.
///
/// ⚠️ **Campo NOVO dentro do valor de um mapa ⇒ quebra dura.** O postcard é posicional: um v118
/// lido por este layout consome os bytes do `String` de dentro do `Vec<u8>` seguinte e a cadeia
/// inteira desalinha — *ele não avisa, lê torto*. ⛔ `#[serde(default)]` não salva um formato
/// não-auto-descritivo.
///
/// ⚠️ **A tripla NÃO vê este degrau** — é a **quarta** vez nesta escada (99, 100, 114 e agora): os
/// bytes mudaram **dentro de um `ComponentBlob`**, que para ela é opaco. *Está escrito aqui porque
/// a próxima pessoa olha para a tripla primeiro.*
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08 (não há projetos gravados).
///
/// # 119 -> 120 — uma cópia pode RECUSAR uma peça da receita (`line/components`, F5.10)
///
/// O `ObjectInstance` ganha `removed: BTreeSet<u64>` — os `StableId` das peças do mestre que
/// **esta** cópia apagou. É o *Removed GameObject* do Unity, e era a única coisa que o modelo de
/// override não sabia dizer: até aqui a forma de uma cópia era **sempre** a da receita, e o gesto
/// de apagar uma peça dentro dela era recusado em voz alta.
///
/// ⚠️ **Só a DECISÃO viaja, e é o que a separa dos órfãos:** a peça recusada continua viva na
/// receita, logo o nome, a pose e os componentes dela lêem-se de lá — não há segunda fonte a criar.
/// *Guardar um valor só é honesto quando não há primeira.*
///
/// ⚠️ **Campo NOVO no fim de um componente ⇒ quebra dura na mesma.** O postcard é posicional e um
/// v115 lido por este layout fica sem os bytes do conjunto — a cadeia desalinha a partir dali, sem
/// avisar.
///
/// ⚠️ **A tripla NÃO vê este degrau** — é a **quinta** vez nesta escada (99, 100, 114, 115 e agora):
/// os bytes mudaram **dentro de um `ComponentBlob`**, que para ela é opaco. *Está escrito aqui
/// porque a próxima pessoa olha para a tripla primeiro.*
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08.///
/// # 120 -> 121 — o PLANO do espelho (report do Enio, 2026-09-04: *«Mirror não funcionou»*)
///
/// As três variantes de espelho da `ph2d_field::Unary` ganharam um `offset: f32`, e o
/// `FIELD_DOC_VERSION` subiu **16 -> 17**. ⚠️ **Sobe por arrasto pelo mesmo caminho do 104**: a
/// pilha de modificadores viaja, posicionalmente, dentro do `ComponentBlob` do
/// `ph2d_field_ecs::FieldMods`, que para a tripla é um `Vec<u8>` opaco.
///
/// ⛔⛔ **E aqui o campo NÃO é «apendado do lado aditivo», ao contrário do degrau 111:** as três
/// eram variantes de **unidade** (só o índice, zero bytes de carga), e passam a ter quatro bytes.
/// Um `Mirror` gravado num v114 lê-se, num v115, comendo os bytes do que vinha a seguir — **sem
/// erro nenhum**, que é o modo de falha que este número existe para tornar audível.
///
/// ⚠️ **Quem defende os bytes é o `the_shape_of_a_saved_modifier_stack_is_pinned`** da
/// `ph2d-field` (`82 -> 94`, quatro bytes por espelho) — o instrumento que o degrau 104 nomeou.
///
/// ⭐ **E o `offset = 0` é a lei de sempre AO BIT** (a dobra na origem do nó): o que muda é o valor
/// de NASCIMENTO, que passa a pôr o plano na face da peça — sem isso o chip acendia e o campo era
/// bit a bit igual, medido em `0.000000`.
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08.
///
/// # 121 -> 122 — o TENDÃO nomeia a IDENTIDADE do osso, não os bits dele (`line/Vector`)
///
/// O `ph2d_skeleton_ecs::Tendon::bone` deixou de ser `Entity::to_bits()` e passou a ser um
/// [`ph2d_ecs::StableId`]. ⚠️ **Os BYTES não mudam** (o postcard serializa um newtype de `u64`
/// transparentemente) — muda o **significado**, que é exactamente a espécie de degrau que este
/// número existe para tornar audível: um v121 com esqueleto lido por este layout resolveria
/// identidades a partir de ids de alocação de outra sessão, e a forma **deixaria de seguir os
/// ossos sem uma linha de erro**.
///
/// ⭐ **A cura é MEDIDA, não teórica:** o gate
/// `skeleton_live::tests::a_skin_survives_the_respawn_that_undo_and_save_do` lia **`0 de 2`**
/// tendões a resolver depois do respawn que o `ProjectState::restore` faz. O undo e o salvar são a
/// mesma máquina, ela despawna e re-spawna **no mesmo mundo**, e a geração do `Entity` sobe.
///
/// ⛔ E os bits eram também um **pânico à espera**: o `Entity::from_bits` do `bevy_ecs` aborta com
/// bits que nunca vieram de um `to_bits`, que é o que um ficheiro de outra sessão entrega.
///
/// ⚠️ **A tripla NÃO vê este degrau** — é a **sexta** vez nesta escada (99, 100, 114, 115, 120 e
/// agora): os bytes vivem dentro de um `ComponentBlob`, que para ela é opaco.
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08 — e aqui com uma razão a
/// mais, medida em 06/09: os dois `.ph2dproj` da máquina do dono são de 26/08, **onze dias antes de
/// os ossos existirem**, logo nenhum ficheiro no mundo tem um tendão para migrar.
///
/// # 122 -> 123 — A ÂNCORA DE IK (`line/Vector`)
///
/// Dois componentes REGISTADOS novos: `ph2d_skeleton_ecs::IkGoal` (a restrição, no osso da ponta) e
/// `ph2d_skeleton_ecs::IkTarget` (a marca que o alvo carrega, para ele não ganhar o anel de objecto
/// vazio por cima do losango).
///
/// ⚠️ **Um componente NOVO move o número, e o mecanismo está no `snapshot_to_world`:** ele resolve
/// cada `ComponentBlob` por `type_id` e faz `ok_or(RegistryError::UnknownTypeId)?` — um blob que o
/// binário não conhece **recusa o load inteiro**. Sem o degrau isso apareceria como um erro de tipo
/// desconhecido no meio da travessia; com ele, como *«este ficheiro é de outra versão»*, que é a
/// frase que diz ao dono o que fazer. É o mesmo motivo dos degraus `89`, `90`, `91` e `92`.
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08.
///
/// ⚠️ **A tripla NÃO vê este degrau** — é a **sétima** vez nesta escada: os componentes viajam em
/// `ComponentBlob`s, e a tripla mede a forma da `VecScene` e do `FlipDoc`.
///
/// # 123 -> 124 — o LADO DA DOBRA é autorado (`line/Vector`)
///
/// O `ph2d_skeleton_ecs::IkGoal` ganhou um **quinto campo**, `bend: ph2d_skeleton::BendSide`, e o
/// postcard é **posicional**: um v123 lido por este layout consumiria os bytes seguintes como se
/// fossem a variante do enum, e o que sai disso não e' um erro — e' um `IkGoal` com valores
/// plausiveis e errados. ⇒ o degrau é o que transforma *«lixo silencioso»* em *«este ficheiro e' de
/// outra versao»*.
///
/// ⭐ **O que ele cura, medido** (`the_elbow_flips_when_the_chain_passes_through_straight`): uma
/// corrente dobrada para um lado (`-99,498744`), esticada até ficar recta (`0,000000`) e trazida de
/// volta **ao mesmo alvo** vinha do lado oposto (`+99,498744`). O joelho invertia sozinho, porque o
/// lado saia da POSE e uma recta nao tem lado nenhum.
///
/// ⚠️ **`BendSide::Keep` é o valor por omissao e ele É o comportamento antigo, ao bit** — logo um
/// v123 hipotético nao precisaria de conversao de dados, so' de ser lido no sitio certo. ⛔ O degrau
/// fica na mesma: o que ele protege e' o ALINHAMENTO dos bytes, nao o significado deles.
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08 — e aqui com a razão a mais
/// que o 122 ja' registou: os dois `.ph2dproj` da máquina do dono sao de 26/08, **onze dias antes de
/// os ossos existirem**.
///
/// ⚠️ **A tripla NÃO vê este degrau** — a **oitava** vez, e pelo mesmo mecanismo do 122 e do 123.
///
/// # 124 -> 125 — o LIMITE DE ÂNGULO da junta (`line/Vector`)
///
/// Um componente REGISTADO novo: `ph2d_skeleton_ecs::BoneLimit` (ate' onde uma junta dobra). O
/// mecanismo e' o mesmo do degrau `123` e dos `89`..`92`: o `snapshot_to_world` resolve cada
/// `ComponentBlob` por `type_id` e faz `ok_or(RegistryError::UnknownTypeId)?`, entao um blob que o
/// binario nao conhece **recusa o load inteiro** — sem o degrau isso apareceria como um erro de
/// tipo desconhecido no meio da travessia, e com ele como *«este ficheiro e' de outra versao»*.
///
/// ⚠️ **Ele mora no OSSO e nao na restricao**, ao contrario do Godot: a afirmacao *«este cotovelo
/// nao dobra para tras»* e' sobre a ANATOMIA, logo vale sem IK nenhuma e nao pode evaporar quando
/// o artista carrega em *Remove IK*.
///
/// ⭐ **A faixa de nascimento e' a VOLTA INTEIRA**, que nao apara nada: pendurar o componente e' um
/// no-op ao bit, e por isso a paleta do Inspector pode oferece-lo (`authored`).
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08.
///
/// ⚠️ **A tripla NÃO vê este degrau** — a **nona** vez.
///
/// # 125 -> 126 — o OSSO INTELIGENTE (`line/Vector`)
///
/// Um componente REGISTADO novo: `ph2d_skeleton_ecs::SmartBone` (girar um osso percorre uma acção
/// inteira — o *Smart Bone* do Moho, o *Action Constraint* do Blender). Mesmo mecanismo dos degraus
/// `123` e `125`: um `ComponentBlob` que o binário nao conhece **recusa o load inteiro**, e o degrau
/// e' o que transforma isso em *«este ficheiro e' de outra versao»*.
///
/// ⚠️ **Ele nomeia o clip pelo NOME**, nunca pelo indice: reordenar ou apagar clips mexe em todos os
/// indices, e um indice guardado passaria a apontar para a animacao do vizinho **em silencio**.
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08.
///
/// ⚠️ **A tripla NÃO vê este degrau** — a **décima** vez.
///
/// # 126 -> 127 — o osso inteligente ganha um ALVO (`line/Vector`)
///
/// Um CAMPO novo no `SmartBone`: `target`, o NOME do objecto de que a acção trata. Report do dono
/// (2026-09-08): *«é necessário um botão de picker para selecionar o objeto … só deve aparecer as
/// animações relacionadas ao objeto selecionado»*.
///
/// ⚠️⚠️ **O degrau é obrigatório e a razão é o postcard, não o campo:** ele é **posicional**, logo
/// um ficheiro gravado com três campos seria lido com quatro **em silêncio** — os bytes do `from`
/// entrariam no `target`. Com o degrau, o load **recusa em voz alta**. É a mesma lei do degrau
/// `112`.
///
/// ⚠️ **O alvo é o NOME e não os bits** (`stable_name_id` é a referência durável desta casa): o undo
/// respawna tudo com bits novos, e bits DENTRO dos bytes de um componente envenenam o próprio undo.
///
/// ⭐ **Ele não mexe no que a acção FAZ** — ela continua a correr inteira. O alvo diz *de que este
/// controlo trata*, e é isso que filtra a lista de acções do painel: com milhares de objectos
/// animados, *«quais animações tocam este objecto»* é a única pergunta que estreita a lista.
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08.
///
/// ⚠️ **A tripla NÃO vê este degrau** — a **décima primeira** vez.
///
/// # 127 -> 128 — DOIS componentes SAEM: o motor de instância do VETOR (F4.6c, `line/components`)
///
/// ⚠️⚠️ **Este degrau foi escrito como `123 -> 124` e RENUMERADO na integração de 2026-09-10.** A
/// `line/Vector` e esta linha subiram o mesmo contador no mesmo dia — `+4` e `+1` — e o valor certo
/// (`128`) **não estava em nenhum dos dois lados**. ⛔ *Duas linhas que escolhem o mesmo literal
/// fundem MUDAS*: aqui o git conflitou por sorte (as duas tocaram a mesma linha do ficheiro), e a
/// sonda `collision-surface.sh` teria dito `124 (base: 123)` — porque a coluna «base» dela é o
/// **merge-base**, não o `main` de hoje. *Conte o DELTA contra a árvore em que vai aterrar.*
///
/// O `ph2d::ecs::VecComponentMain` e o `ph2d::ecs::VecInstance` deixaram de existir. Eles eram o
/// mestre e a cópia do **segundo** motor de instância do app — o do sistema vetorial —, e o modelo
/// geral (ADR-0164: `MasterRoot` / `InstanceOf` / `ObjectInstance`) responde por tudo o que eles
/// faziam desde 2026-09-06, quando ele passou a ser o caminho de omissão.
///
/// ⚠️ **Um componente que SAI move o número pela MESMA razão que um que entra**, e o mecanismo é
/// literalmente o mesmo `snapshot_to_world`: ele resolve cada `ComponentBlob` por `type_id` e faz
/// `ok_or(RegistryError::UnknownTypeId)?`. Um `.ph2dproj` gravado com uma instância vetorial dentro
/// **recusa o load inteiro** a partir daqui — sem o degrau isso apareceria como *«type id
/// desconhecido»* no meio da travessia; com ele, como *«este ficheiro é de outra versão»*, que é a
/// frase que diz ao dono o que aconteceu.
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08 — e aqui com a razão extra
/// que a wave anterior já tinha medido: desde 2026-09-06 **não havia como criar** uma instância
/// vetorial (o modo geral era a única porta), e os `.ph2dproj` da máquina do dono são de 26/08.
///
/// ⚠️ **A tripla NÃO vê este degrau** — é a **décima segunda** vez nesta escada, e pela razão de
/// sempre: os componentes viajam em `ComponentBlob`s, que para ela são opacos. É por isso que o
/// número tem de subir à mão. (⚠️ dizia **oitava** quando o degrau era o `124`; os quatro da
/// `line/Vector` entraram à frente dele e cada um deles é também invisível à tripla.)
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
/// # 129 -> 130 — a malha de uma imagem presa leva os PESOS dentro (`line/Vector`)
///
/// Os bytes opacos do `SkinBind::source` de uma **imagem** deixaram de ser uma
/// `ph2d_poly2d::Mesh2d` e passaram a ser uma `ph2d_skeleton_live::skinned_mesh::SkinnedMesh`
/// (`{ mesh, pesos }`): os *Bounded Biharmonic Weights* resolvem-se **uma vez ao prender**.
///
/// ⚠️⚠️ **O degrau é obrigatório e a razão é o postcard, como nos degraus `112`, `127` e `129`:**
/// ele é **posicional**, e a `SkinnedMesh` é a `Mesh2d` **seguida** do vector de pesos. Um ficheiro
/// gravado com a malha só acaba onde a nova espera o comprimento da tabela — e quem lesse os bytes
/// seguintes leria lixo como pesos. Com o degrau, o load **recusa em voz alta**.
///
/// ⚠️ **Uma FORMA VECTORIAL não muda de formato** — ela continua a guardar um `VecPath`, e continua
/// a resolver os pesos pela lei euclidiana derivada. ⛔ *Limite NOMEADO, não esquecimento:* o
/// padrão-ouro precisa de uma malha do domínio, e um caminho de Bézier não tem uma. Enquanto isso
/// não existir, um rig com as duas mídias tem duas leis — e é o que está aberto na fila.
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08 — e aqui com a razão extra
/// que a wave já mediu: uma malha antiga não tem pesos, e derivá-los no load seria correr o solver
/// (dezenas de ms por imagem) dentro do caminho de abrir um ficheiro.
///
/// ⚠️ **A tripla NÃO vê este degrau** — é a **décima quarta** vez nesta escada, e por uma razão a
/// mais: estes bytes não estão sequer num componente que a tripla percorra, estão **dentro** do
/// `source` de um, que é opaco duas vezes.
/// # 136 -> 137 — a FÁBRICA e o CICLO DE VIDA (TOP-20 #11 e #12, `line/components`)
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
/// os três são **aditivos**: um v136 não tem nenhum deles, logo lê-se inteiro por este binário. O
/// degrau existe para o sentido contrário (um v137 com uma fábrica dentro, lido por um binário
/// anterior), que é o que recusa em voz alta.
///
/// ⚠️ **A tripla NÃO vê este degrau** — é a **décima quarta** vez: os componentes viajam em
/// `ComponentBlob`s, que para ela são opacos.
/// # 137 -> 131 — o MOVER DE VISTA DE CIMA (TOP-20 #13, `line/components`)
///
/// UM componente registado novo: `ph2d::physics::TopDownPlayer`. Mesmo mecanismo dos degraus
/// `123`, `125`, `126`, `127` e `137` — um `ComponentBlob` de `type_id` desconhecido **recusa o
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
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 e pela razão aditiva: um v137 não
/// tem o componente, logo lê-se inteiro por este binário. O degrau existe para o sentido contrário.
///
/// ⚠️ **A tripla NÃO vê este degrau** — é a **décima quinta** vez.
/// # 131 -> 132 — o PROJÉCTIL (TOP-20 #14, `line/components`)
///
/// UM componente registado novo: `ph2d::physics::ProjectileMotion`. Mesmo mecanismo dos degraus
/// `123`, `125`, `126`, `127`, `137` e `131`.
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
/// Mesmo mecanismo dos degraus `123`, `125`, `126`, `127`, `137`, `131` e `132`.
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
/// (`{ mesh, pesos }`). É a ligação do padrão-ouro ao produto: os *Bounded Biharmonic Weights*
/// são a solução de um problema variacional sobre a arte inteira, logo resolvem-se **uma vez ao
/// prender** e viajam com a malha.
///
/// ⚠️⚠️ **O degrau é obrigatório e a razão é o postcard, como nos degraus `112`, `127` e `136`:**
/// ele é **posicional**, e a `SkinnedMesh` é a `Mesh2d` **seguida** do vector de pesos. Um ficheiro
/// gravado com a malha só acaba onde a nova espera o comprimento da tabela — e quem lesse os bytes
/// seguintes leria lixo como pesos. Com o degrau, o load **recusa em voz alta**.
///
/// ⛔ **Uma forma VECTORIAL não muda de formato** — guarda um `VecPath` e fica na lei euclidiana
/// derivada (o padrão-ouro precisa de uma malha do domínio, e uma Bézier não tem uma). *Limite
/// NOMEADO:* um rig com as duas mídias tem hoje duas leis.
///
/// ⛔ **Sem degrau de migração** (Enio, 26/08) — derivar os pesos no load seria correr o solver
/// dentro do caminho de abrir um ficheiro. ⚠️ **A tripla NÃO o vê** (14.ª vez): estes bytes estão
/// DENTRO do `source` de um `ComponentBlob`, opaco duas vezes./// # 130 -> 131 — a forma VECTORIAL presa também leva os pesos dentro (`line/Vector`)
///
/// O degrau `130` deixou a 1.ª mídia na lei euclidiana **por limite nomeado** (*«o padrão-ouro
/// precisa de uma malha do domínio, e uma Bézier não tem uma»*). Ela tem: o **interior** dos
/// contornos fechados. ⇒ o `SkinBind::source` de um `VecPathRef` deixou de ser um `VecPath` e passou
/// a ser um `ph2d_skeleton_live::skinned_mesh::SkinnedPath` (`{ path, pesos }`), com um peso por
/// **ponto de controlo**.
///
/// ⚠️⚠️ **Obrigatório, e a razão é o postcard** — a mesma dos degraus `112`, `127`, `129` e `130`:
/// ele é **posicional**, e a struct nova é a antiga **seguida** do vector.
///
/// ⭐ **E é o degrau que fecha a divergência que o `130` abriu:** as duas mídias voltam a responder à
/// MESMA lei. Um rig com um braço vectorial e um braço em imagem deixou de ter duas.
///
/// ⛔ **Um caminho ABERTO fica na lei derivada, e é uma resposta**: uma linha tem área zero, logo não
/// há domínio para a energia. A tabela sai **vazia**, que é a mesma forma que uma imagem sem pesos
/// usa.
///
/// ⛔ **Sem degrau de migração** (Enio, 26/08). ⚠️ **A tripla NÃO o vê** (15.ª vez): os bytes estão
/// DENTRO do `source` de um `ComponentBlob`.
///
/// # `131 → 132` — ⭐⭐⭐ o osso passou a dizer **DE ONDE VÊM AS ALÇAS** de curvatura
///
/// O `Bone` ganhou `handles` ([`ph2d_skeleton::bend::Handles`]): `Authored` (o nascimento, e o que
/// todo osso sempre foi) ou `Auto`, em que as duas alças saem das tangentes dos ossos **vizinhos** e
/// a corrente inteira vira uma curva lisa. É o último item aberto da F8.
///
/// ⚠️⚠️ **Obrigatório, e a razão é o postcard** — a mesma dos degraus `112`, `127`, `129`, `130` e
/// `131`: ele é **posicional**, e um campo APENDADO a um componente faz um ficheiro velho ser lido
/// com um campo a mais, comendo os bytes do vizinho. ⛔ O `#[serde(default)]` no campo **não** o
/// salva: ele serve formatos com nomes, e o postcard não tem nenhum.
///
/// ⛔ **Sem degrau de migração** (a mesma decisão do Enio de 26/08 — não há projectos gravados).
/// ⚠️ **A tripla NÃO o vê** (16.ª vez): os bytes mudaram dentro de um `ComponentBlob`.
pub(crate) const PROJECT_SCHEMA: u32 = 137;
