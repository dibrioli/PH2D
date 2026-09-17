//! **A ESCADA do `PROJECT_SCHEMA`, de v112 a v128** — a QUARTA faixa arquivada.
//!
//! ⚠️ **O corte por IDADE repete-se pela quarta vez, e o gatilho é sempre o mesmo:** o tecto de
//! 600 linhas do HR-18. O irmão [`super::project_schema_history`] nasceu em `v82`; o
//! [`super::project_schema_history_v83`] em 2026-09-07, com o degrau `122`; o
//! [`super::project_schema_history_v99`] em 2026-09-15, com o degrau `132`; e este na integração
//! de 2026-09-17, quando os **quatro** degraus da `line/3DModeling` (o material do modo RENDER e
//! os arcos do perfil) levaram a escada viva a `651`.
//!
//! ⭐ **E esta faixa é a primeira cujo gatilho é a ACUMULAÇÃO de uma rodada inteira, e não uma
//! linha:** a rodada de seis pôs **dezasseis** degraus (`128` → `144`), e nenhuma delas estoura o
//! tecto sozinha — é o caso que o `CLAUDE.md` §5.0 nomeia como *o único número deste repo que soma
//! entre linhas sem ninguém a contar*, e por isso a cura é do INTEGRADOR.
//!
//! ⛔ **A cura é sempre o CORTE, nunca subir o número** — a entrada em `FILE_OVERAGE_OK` só desce.
//!
//! ⚠️⚠️ **E a fronteira desta faixa não é um número confortável: é o `128`**, o valor que o `main`
//! tinha quando as seis linhas desta rodada nasceram. Abaixo dele estão rodadas FECHADAS; acima
//! está o que alguém a contar o próximo degrau precisa de ver.
//!
//! ⚠️ **Nada saiu do código.** A história é `//!` em vez de `///` e continua a ser lida, grepada e
//! publicada pelo `cargo doc` exactamente como era; o que mudou foi de que arquivo ela vem.
//!
//! ⚠️ **E a PONTA fica na escada viva, de propósito:** *quem conta o próximo degrau lê a escada,
//! não o literal* (a lição do degrau `v69`, que chegou ao `main` com a linha ausente).

//! # 111 -> 112 — a TAXONOMIA da biblioteca de assets (plano 07, wave A3)
//!
//! O `ProjectFile` ganhou `catalogs: Vec<u8>` — um blob **auto-versionado**
//! (`project_catalogs::CATALOG_DOC_VERSION`) com os catálogos e as atribuições `asset → catálogo`.
//!
//! ⭐ **Do lado ADITIVO da regra, e mesmo assim o número SOBE.** O campo entra no fim e os índices
//! anteriores não se mexem, então um v103 lido por v104 estaria correcto até ao último campo — e é
//! exactamente aí que ele acaba: o postcard chega ao fim dos bytes (`Hit the end of buffer`) e
//! falha **longe da causa**. O número transforma isso num erro de VERSÃO, que se lê.
//!
//! ⚠️ **E este é o ÚNICO bump que a taxonomia paga.** A versão do blob mora dentro dele, então
//! acrescentar um campo à taxonomia — cor de catálogo, ordem manual, o que vier — custa só o
//! `CATALOG_DOC_VERSION`. É o precedente do `timeline`, do `sculpt` e do `pattern_art`.
//!
//! ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 (*«não há projetos salvos»*).
//!
//! # 105 — a biblioteca MUDA-SE para dentro do undo (Enio, 2026-08-30)
//!
//! *«deveria ter undo/redo no painel inclusive em del»*. ⇒ a taxonomia sai do `ProjectFile` e
//! passa a viver no **`ProjectState`**, que é a unidade que o Ctrl+Z restaura, acompanhada das
//! **lápides** — as imagens que o artista mandou sair da biblioteca.
//!
//! ⛔⛔ **Este degrau NÃO é aditivo, e é por isso que ele é o mais perigoso desta escada desde o
//! 102.** Um campo saiu do meio do `ProjectFile` e outro entrou no meio do `ProjectState`: os
//! bytes de um v104 não desaparecem, eles passam a **significar outra coisa**. Postcard é
//! posicional e leria a taxonomia velha como se fosse outro campo, **sem erro nenhum**.
//!
//! ⚠️ **Duas respostas à mesma pergunta era a alternativa**, e é o que o número compra: manter o
//! campo no ficheiro *e* no estado deixaria o load a escolher qual acreditar.
//!
//! ⚠️ **A versão do blob continua a mandar na taxonomia**: o `CATALOG_DOC_VERSION` mora dentro dos
//! bytes, então acrescentar-lhe um campo continua a custar zero aqui.
//!
//! # 113 -> 114 — a UNIDADE DE ÂNGULO (`line/UIUX`)
//!
//! As `SavedSettings` ganharam `display_angle: u8` — a irmã do `display_unit`, para o pedido do
//! Enio de 2026-08-30: *"devemos ter ambas as opções no app (px e metros, graus e radianos)"*.
//!
//! ⚠️ **Campo APENDADO ao fim de uma struct ⇒ quebra dura**, o mesmo mecanismo do degrau v80: o
//! postcard é posicional, então um v103 lido por este layout fica **sem bytes** no último campo.
//! ⛔ `#[serde(default)]` **não** o salva — num formato não-auto-descritivo ele não sabe que o
//! campo faltou, sabe que o *stream* acabou.
//!
//! ⚠️ **A tripla NÃO vê este degrau**, e é a terceira vez que isso acontece nesta escada (99, 100,
//! e agora): ela mede a forma do `FlipDoc` e da `VecScene`, e nenhuma das duas mudou — o que mudou
//! foi o `ProjectFile`. *Está escrito aqui porque a próxima pessoa olha para a tripla primeiro.*
//!
//! ⭐ **O default preserva o comportamento anterior ao bit:** `DisplayAngle::Degrees` é o que os
//! sítios faziam com `to_degrees()` escrito à mão. Quem nunca abrir o menu não vê diferença.
//!
//! ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08.
//! # 114 -> 115 — A OPACIDADE E A MISTURA DA FORMA (`line/Vector`)
//!
//! O [`ph2d_vec_scene::VecPath`] ganhou `opacity` + `blend` (`VEC_SCENE_SCHEMA_VERSION` 18 -> 19) —
//! o item 2 do estudo 42, e a metade que faltava ao report do Enio de 2026-09-04 (*"o painel não
//! mostra..."* ⇒ não havia o que mostrar: a forma não tinha opacidade própria).
//!
//! ⚠️ **Campos APENDADOS ao fim de uma struct ⇒ quebra dura nos dois sentidos** — o mesmo
//! mecanismo do v80 e do v114: postcard é posicional e não sinaliza ausência, então um save v18 da
//! cena lido por este layout **acaba os bytes** no campo novo, e um v19 lido por um binário velho
//! traz bytes a mais. ⭐ **A tripla VÊ este degrau** (ao contrário dos v99/v100/v114): o que mudou
//! foi a forma da `VecScene`, que é exactamente o que ela mede.
//!
//! ⛔ **Sem degrau de migração**, pela decisão do Enio de 2026-08-26.
//!
//! # 115 -> 116 — A PILHA DE APARÊNCIA (`line/Vector`)
//!
//! O [`ph2d_vec_scene::VecPath`] ganhou `paints` (`VEC_SCENE_SCHEMA_VERSION` 19 -> 20) — N
//! preenchimentos e N contornos numa forma (o item 4 do estudo 42). Sem ela, cada camada de estilo
//! obriga a **duplicar o objecto**, e duas cópias de uma forma são duas geometrias que divergem no
//! primeiro ponto que o artista mexe.
//!
//! ⚠️ **Um `Vec` apendado ao fim ⇒ a mesma quebra dura dos v114 e v115**: o postcard é posicional e
//! não sinaliza ausência. ⭐ Vazio é o neutro e custa **1 byte** de comprimento zero, então uma cena
//! que nunca lhe toque escreve o que escrevia mais esse byte — e desenha byte a byte igual.
//!
//! ⛔ **Sem degrau de migração**, pela mesma decisão de 2026-08-26.
//! # 116 -> 117 — ONDE cada camada da pilha desenha (`line/Vector`)
//!
//! O [`ph2d_vec_scene::PaintEntry`] ganhou `offset` (`VEC_SCENE_SCHEMA_VERSION` 20 -> 21) — o
//! deslocamento de UMA camada, relativo à forma. Report do Enio, 2026-09-05: *"o fill não deveria
//! ter um offset? não seria mais útil?"* Sem ele, dois preenchimentos desenham nos **mesmos
//! pixels**, e as duas coisas que um artista faz com um segundo preenchimento — a sombra dura e a
//! profundidade de um rótulo — são inexprimíveis sem duplicar a forma outra vez.
//!
//! ⚠️ **O campo é apendado ao fim de `PaintEntry`, e não ao fim do `VecPath`** — a diferença é o
//! modo de falha: um save v116 lido por este layout rebenta **dentro** do `Vec` de camadas (o
//! postcard lê elemento a elemento), e não no fim do ficheiro. ⭐ Uma forma **sem pilha** não tem
//! entradas ⇒ zero bytes novos, e o ficheiro é byte a byte o de antes.
//!
//! ⛔ **Sem degrau de migração**, pela mesma decisão de 2026-08-26.
//! # 117 -> 118 — O OFFSET DE CAD de uma camada (`line/Vector`)
//!
//! O [`ph2d_vec_scene::PaintEntry`] ganhou `dilate` + `dilate_join`
//! (`VEC_SCENE_SCHEMA_VERSION` 21 -> 22): a silhueta de UMA camada cresce ou encolhe. Pedido do
//! Enio, 2026-09-05 (*"o offset do cad, contraindo e dilatando"*), e é o que faz um adesivo ou um
//! selo sem duplicar a forma.
//!
//! ⚠️ Mesmo modo de falha do degrau anterior: os campos são apendados ao fim de `PaintEntry`, então
//! um save v117 rebenta **dentro** do `Vec` de camadas. ⭐ Uma forma sem pilha não tem entradas ⇒
//! zero bytes novos.
//!
//! ⛔ **Sem degrau de migração**, pela mesma decisão de 2026-08-26.
//! # 118 -> 119 — a excepção sem alvo SABE de que peça era (`line/components`, F5 critério 3)
//!
//! O `ObjectInstance.orphans` era `BTreeMap<OverrideKey, Vec<u8>>` e passou a
//! `BTreeMap<OverrideKey, OrphanOverride>` — os mesmos bytes **mais** o `Name` que a peça tinha
//! quando morreu.
//!
//! ⚠️ **O nome não é uma segunda fonte, e o argumento é o MESMO que já justificou os bytes** (F5.3):
//! a refutação da F4.4 — *«guardar o valor cria duas fontes»* — vale enquanto a peça **existe**.
//! Uma peça órfã não existe: o mestre apagou-a e a F5.1 tirou-a da cópia a seguir. ⇒ não há segunda
//! fonte, há a única. Sem ele o painel pode dizer *«há três»* e **nunca** *«quais três»*, com o
//! botão que apaga as três ao lado.
//!
//! ⚠️ **Campo NOVO dentro do valor de um mapa ⇒ quebra dura.** O postcard é posicional: um v118
//! lido por este layout consome os bytes do `String` de dentro do `Vec<u8>` seguinte e a cadeia
//! inteira desalinha — *ele não avisa, lê torto*. ⛔ `#[serde(default)]` não salva um formato
//! não-auto-descritivo.
//!
//! ⚠️ **A tripla NÃO vê este degrau** — é a **quarta** vez nesta escada (99, 100, 114 e agora): os
//! bytes mudaram **dentro de um `ComponentBlob`**, que para ela é opaco. *Está escrito aqui porque
//! a próxima pessoa olha para a tripla primeiro.*
//!
//! ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08 (não há projetos gravados).
//!
//! # 119 -> 120 — uma cópia pode RECUSAR uma peça da receita (`line/components`, F5.10)
//!
//! O `ObjectInstance` ganha `removed: BTreeSet<u64>` — os `StableId` das peças do mestre que
//! **esta** cópia apagou. É o *Removed GameObject* do Unity, e era a única coisa que o modelo de
//! override não sabia dizer: até aqui a forma de uma cópia era **sempre** a da receita, e o gesto
//! de apagar uma peça dentro dela era recusado em voz alta.
//!
//! ⚠️ **Só a DECISÃO viaja, e é o que a separa dos órfãos:** a peça recusada continua viva na
//! receita, logo o nome, a pose e os componentes dela lêem-se de lá — não há segunda fonte a criar.
//! *Guardar um valor só é honesto quando não há primeira.*
//!
//! ⚠️ **Campo NOVO no fim de um componente ⇒ quebra dura na mesma.** O postcard é posicional e um
//! v115 lido por este layout fica sem os bytes do conjunto — a cadeia desalinha a partir dali, sem
//! avisar.
//!
//! ⚠️ **A tripla NÃO vê este degrau** — é a **quinta** vez nesta escada (99, 100, 114, 115 e agora):
//! os bytes mudaram **dentro de um `ComponentBlob`**, que para ela é opaco. *Está escrito aqui
//! porque a próxima pessoa olha para a tripla primeiro.*
//!
//! ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08.///
//! # 120 -> 121 — o PLANO do espelho (report do Enio, 2026-09-04: *«Mirror não funcionou»*)
//!
//! As três variantes de espelho da `ph2d_field::Unary` ganharam um `offset: f32`, e o
//! `FIELD_DOC_VERSION` subiu **16 -> 17**. ⚠️ **Sobe por arrasto pelo mesmo caminho do 104**: a
//! pilha de modificadores viaja, posicionalmente, dentro do `ComponentBlob` do
//! `ph2d_field_ecs::FieldMods`, que para a tripla é um `Vec<u8>` opaco.
//!
//! ⛔⛔ **E aqui o campo NÃO é «apendado do lado aditivo», ao contrário do degrau 111:** as três
//! eram variantes de **unidade** (só o índice, zero bytes de carga), e passam a ter quatro bytes.
//! Um `Mirror` gravado num v114 lê-se, num v115, comendo os bytes do que vinha a seguir — **sem
//! erro nenhum**, que é o modo de falha que este número existe para tornar audível.
//!
//! ⚠️ **Quem defende os bytes é o `the_shape_of_a_saved_modifier_stack_is_pinned`** da
//! `ph2d-field` (`82 -> 94`, quatro bytes por espelho) — o instrumento que o degrau 104 nomeou.
//!
//! ⭐ **E o `offset = 0` é a lei de sempre AO BIT** (a dobra na origem do nó): o que muda é o valor
//! de NASCIMENTO, que passa a pôr o plano na face da peça — sem isso o chip acendia e o campo era
//! bit a bit igual, medido em `0.000000`.
//!
//! ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08.
//!
//! # 121 -> 122 — o TENDÃO nomeia a IDENTIDADE do osso, não os bits dele (`line/Vector`)
//!
//! O `ph2d_skeleton_ecs::Tendon::bone` deixou de ser `Entity::to_bits()` e passou a ser um
//! [`ph2d_ecs::StableId`]. ⚠️ **Os BYTES não mudam** (o postcard serializa um newtype de `u64`
//! transparentemente) — muda o **significado**, que é exactamente a espécie de degrau que este
//! número existe para tornar audível: um v121 com esqueleto lido por este layout resolveria
//! identidades a partir de ids de alocação de outra sessão, e a forma **deixaria de seguir os
//! ossos sem uma linha de erro**.
//!
//! ⭐ **A cura é MEDIDA, não teórica:** o gate
//! `skeleton_live::tests::a_skin_survives_the_respawn_that_undo_and_save_do` lia **`0 de 2`**
//! tendões a resolver depois do respawn que o `ProjectState::restore` faz. O undo e o salvar são a
//! mesma máquina, ela despawna e re-spawna **no mesmo mundo**, e a geração do `Entity` sobe.
//!
//! ⛔ E os bits eram também um **pânico à espera**: o `Entity::from_bits` do `bevy_ecs` aborta com
//! bits que nunca vieram de um `to_bits`, que é o que um ficheiro de outra sessão entrega.
//!
//! ⚠️ **A tripla NÃO vê este degrau** — é a **sexta** vez nesta escada (99, 100, 114, 115, 120 e
//! agora): os bytes vivem dentro de um `ComponentBlob`, que para ela é opaco.
//!
//! ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08 — e aqui com uma razão a
//! mais, medida em 06/09: os dois `.ph2dproj` da máquina do dono são de 26/08, **onze dias antes de
//! os ossos existirem**, logo nenhum ficheiro no mundo tem um tendão para migrar.
//!
//! # 122 -> 123 — A ÂNCORA DE IK (`line/Vector`)
//!
//! Dois componentes REGISTADOS novos: `ph2d_skeleton_ecs::IkGoal` (a restrição, no osso da ponta) e
//! `ph2d_skeleton_ecs::IkTarget` (a marca que o alvo carrega, para ele não ganhar o anel de objecto
//! vazio por cima do losango).
//!
//! ⚠️ **Um componente NOVO move o número, e o mecanismo está no `snapshot_to_world`:** ele resolve
//! cada `ComponentBlob` por `type_id` e faz `ok_or(RegistryError::UnknownTypeId)?` — um blob que o
//! binário não conhece **recusa o load inteiro**. Sem o degrau isso apareceria como um erro de tipo
//! desconhecido no meio da travessia; com ele, como *«este ficheiro é de outra versão»*, que é a
//! frase que diz ao dono o que fazer. É o mesmo motivo dos degraus `89`, `90`, `91` e `92`.
//!
//! ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08.
//!
//! ⚠️ **A tripla NÃO vê este degrau** — é a **sétima** vez nesta escada: os componentes viajam em
//! `ComponentBlob`s, e a tripla mede a forma da `VecScene` e do `FlipDoc`.
//!
//! # 123 -> 124 — o LADO DA DOBRA é autorado (`line/Vector`)
//!
//! O `ph2d_skeleton_ecs::IkGoal` ganhou um **quinto campo**, `bend: ph2d_skeleton::BendSide`, e o
//! postcard é **posicional**: um v123 lido por este layout consumiria os bytes seguintes como se
//! fossem a variante do enum, e o que sai disso não e' um erro — e' um `IkGoal` com valores
//! plausiveis e errados. ⇒ o degrau é o que transforma *«lixo silencioso»* em *«este ficheiro e' de
//! outra versao»*.
//!
//! ⭐ **O que ele cura, medido** (`the_elbow_flips_when_the_chain_passes_through_straight`): uma
//! corrente dobrada para um lado (`-99,498744`), esticada até ficar recta (`0,000000`) e trazida de
//! volta **ao mesmo alvo** vinha do lado oposto (`+99,498744`). O joelho invertia sozinho, porque o
//! lado saia da POSE e uma recta nao tem lado nenhum.
//!
//! ⚠️ **`BendSide::Keep` é o valor por omissao e ele É o comportamento antigo, ao bit** — logo um
//! v123 hipotético nao precisaria de conversao de dados, so' de ser lido no sitio certo. ⛔ O degrau
//! fica na mesma: o que ele protege e' o ALINHAMENTO dos bytes, nao o significado deles.
//!
//! ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08 — e aqui com a razão a mais
//! que o 122 ja' registou: os dois `.ph2dproj` da máquina do dono sao de 26/08, **onze dias antes de
//! os ossos existirem**.
//!
//! ⚠️ **A tripla NÃO vê este degrau** — a **oitava** vez, e pelo mesmo mecanismo do 122 e do 123.
//!
//! # 124 -> 125 — o LIMITE DE ÂNGULO da junta (`line/Vector`)
//!
//! Um componente REGISTADO novo: `ph2d_skeleton_ecs::BoneLimit` (ate' onde uma junta dobra). O
//! mecanismo e' o mesmo do degrau `123` e dos `89`..`92`: o `snapshot_to_world` resolve cada
//! `ComponentBlob` por `type_id` e faz `ok_or(RegistryError::UnknownTypeId)?`, entao um blob que o
//! binario nao conhece **recusa o load inteiro** — sem o degrau isso apareceria como um erro de
//! tipo desconhecido no meio da travessia, e com ele como *«este ficheiro e' de outra versao»*.
//!
//! ⚠️ **Ele mora no OSSO e nao na restricao**, ao contrario do Godot: a afirmacao *«este cotovelo
//! nao dobra para tras»* e' sobre a ANATOMIA, logo vale sem IK nenhuma e nao pode evaporar quando
//! o artista carrega em *Remove IK*.
//!
//! ⭐ **A faixa de nascimento e' a VOLTA INTEIRA**, que nao apara nada: pendurar o componente e' um
//! no-op ao bit, e por isso a paleta do Inspector pode oferece-lo (`authored`).
//!
//! ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08.
//!
//! ⚠️ **A tripla NÃO vê este degrau** — a **nona** vez.
//!
//! # 125 -> 126 — o OSSO INTELIGENTE (`line/Vector`)
//!
//! Um componente REGISTADO novo: `ph2d_skeleton_ecs::SmartBone` (girar um osso percorre uma acção
//! inteira — o *Smart Bone* do Moho, o *Action Constraint* do Blender). Mesmo mecanismo dos degraus
//! `123` e `125`: um `ComponentBlob` que o binário nao conhece **recusa o load inteiro**, e o degrau
//! e' o que transforma isso em *«este ficheiro e' de outra versao»*.
//!
//! ⚠️ **Ele nomeia o clip pelo NOME**, nunca pelo indice: reordenar ou apagar clips mexe em todos os
//! indices, e um indice guardado passaria a apontar para a animacao do vizinho **em silencio**.
//!
//! ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08.
//!
//! ⚠️ **A tripla NÃO vê este degrau** — a **décima** vez.
//!
//! # 126 -> 127 — o osso inteligente ganha um ALVO (`line/Vector`)
//!
//! Um CAMPO novo no `SmartBone`: `target`, o NOME do objecto de que a acção trata. Report do dono
//! (2026-09-08): *«é necessário um botão de picker para selecionar o objeto … só deve aparecer as
//! animações relacionadas ao objeto selecionado»*.
//!
//! ⚠️⚠️ **O degrau é obrigatório e a razão é o postcard, não o campo:** ele é **posicional**, logo
//! um ficheiro gravado com três campos seria lido com quatro **em silêncio** — os bytes do `from`
//! entrariam no `target`. Com o degrau, o load **recusa em voz alta**. É a mesma lei do degrau
//! `112`.
//!
//! ⚠️ **O alvo é o NOME e não os bits** (`stable_name_id` é a referência durável desta casa): o undo
//! respawna tudo com bits novos, e bits DENTRO dos bytes de um componente envenenam o próprio undo.
//!
//! ⭐ **Ele não mexe no que a acção FAZ** — ela continua a correr inteira. O alvo diz *de que este
//! controlo trata*, e é isso que filtra a lista de acções do painel: com milhares de objectos
//! animados, *«quais animações tocam este objecto»* é a única pergunta que estreita a lista.
//!
//! ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08.
//!
//! ⚠️ **A tripla NÃO vê este degrau** — a **décima primeira** vez.
//!
//! # 127 -> 128 — DOIS componentes SAEM: o motor de instância do VETOR (F4.6c, `line/components`)
//!
//! ⚠️⚠️ **Este degrau foi escrito como `123 -> 124` e RENUMERADO na integração de 2026-09-10.** A
//! `line/Vector` e esta linha subiram o mesmo contador no mesmo dia — `+4` e `+1` — e o valor certo
//! (`128`) **não estava em nenhum dos dois lados**. ⛔ *Duas linhas que escolhem o mesmo literal
//! fundem MUDAS*: aqui o git conflitou por sorte (as duas tocaram a mesma linha do ficheiro), e a
//! sonda `collision-surface.sh` teria dito `124 (base: 123)` — porque a coluna «base» dela é o
//! **merge-base**, não o `main` de hoje. *Conte o DELTA contra a árvore em que vai aterrar.*
//!
//! O `ph2d::ecs::VecComponentMain` e o `ph2d::ecs::VecInstance` deixaram de existir. Eles eram o
//! mestre e a cópia do **segundo** motor de instância do app — o do sistema vetorial —, e o modelo
//! geral (ADR-0164: `MasterRoot` / `InstanceOf` / `ObjectInstance`) responde por tudo o que eles
//! faziam desde 2026-09-06, quando ele passou a ser o caminho de omissão.
//!
//! ⚠️ **Um componente que SAI move o número pela MESMA razão que um que entra**, e o mecanismo é
//! literalmente o mesmo `snapshot_to_world`: ele resolve cada `ComponentBlob` por `type_id` e faz
//! `ok_or(RegistryError::UnknownTypeId)?`. Um `.ph2dproj` gravado com uma instância vetorial dentro
//! **recusa o load inteiro** a partir daqui — sem o degrau isso apareceria como *«type id
//! desconhecido»* no meio da travessia; com ele, como *«este ficheiro é de outra versão»*, que é a
//! frase que diz ao dono o que aconteceu.
//!
//! ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08 — e aqui com a razão extra
//! que a wave anterior já tinha medido: desde 2026-09-06 **não havia como criar** uma instância
//! vetorial (o modo geral era a única porta), e os `.ph2dproj` da máquina do dono são de 26/08.
//!
//! ⚠️ **A tripla NÃO vê este degrau** — é a **décima segunda** vez nesta escada, e pela razão de
//! sempre: os componentes viajam em `ComponentBlob`s, que para ela são opacos. É por isso que o
//! número tem de subir à mão. (⚠️ dizia **oitava** quando o degrau era o `124`; os quatro da
//! `line/Vector` entraram à frente dele e cada um deles é também invisível à tripla.)
