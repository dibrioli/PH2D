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
/// ⚠️ **Os degraus de v2 a v79 estão em [`super::project_schema_history`]**, verbatim — o corte é
/// por IDADE, e o teto de 600 LOC do HR-18 foi quem o pediu. O que se lê para contar o próximo
/// degrau é a ponta, e a ponta é o que ficou aqui.
/// v80 (physics, W-Fall — O TETO DE QUEDA): o `PlatformPlayer` ganhou
/// `max_fall_speed`, apendado ao FIM ⇒ quebra dura. ⚠️ **Ele existe porque NÃO
/// havia velocidade terminal, e o número é desta wave:** largando de mil metros
/// a descida chega a **142,57 m/s aos 8 s** e continua a crescer, nos DOIS
/// modos — um personagem que caia de alto o bastante atravessa o cenário a
/// velocidades que nenhum colisor discreto resolve. ⚠️ **E o degrau é o ÚNICO
/// preço da wave:** o campo nasce em `0`, que **desliga** a lei (a porta devolve
/// `None` e o motor é `Motor::default()`) ⇒ todo projeto salvo em v79 reabre a
/// cair exactamente como caía, e o `physics_ecs_c9` sai byte-idêntico.
/// v81 (physics, W-Leave — O QUE A PLATAFORMA DA AO PULO): o `PlatformPlayer`
/// ganhou `platform_lift`, apendado ao FIM ⇒ quebra dura. ⚠️ **A altura autorada
/// era medida contra a PLATAFORMA, e ninguem tinha escolhido isso:** o pulo leva
/// a subida RELATIVA ao chao ao `v0`, o que e' o `ADD_VELOCITY` do Godot — e num
/// elevador a descer a 4 m/s o pico medido cai de **1,865 para 0,016 m**, nos
/// tres modos (`measure_platform_leave`). ⚠️ **E o degrau e' o UNICO preco:** o
/// campo nasce em `Full`, onde a porta devolve `rel_up` VERBATIM ⇒ todo projeto
/// salvo em v80 reabre a pular exactamente como pulava, e o `physics_ecs_c9` sai
/// byte-identico.
/// v82 (physics, W-Brink — A TRAVA DE BEIRADA): o `PlatformPlayer` ganhou
/// `walk_off_ledges` e `crouch_walk_off_ledges`, apendados ao FIM ⇒ quebra dura.
/// O `bCanWalkOffLedges` do Unreal, que ele serve a IA e ao *andar com cuidado*.
/// ⚠️ **Os campos guardam a CAPACIDADE, nunca a trava**, e a razao e' o postcard:
/// num `stop_at_ledges` o `false` que todo arquivo antigo traz num campo novo
/// significaria *trava armada*, e a capacidade nasceria ligada em toda arte ja'
/// autorada. ⚠️ **E o degrau e' o UNICO preco:** os dois nascem em `true`, onde
/// a lei devolve o alvo VERBATIM e o sensor nem sequer casta ⇒ todo projeto
/// salvo em v81 reabre a andar exactamente como andava, e o `physics_ecs_c9` sai
/// byte-identico. ⚠️ **O alcance NAO e' um degrau:** ele e' DERIVADO
/// (`v²/2a` + meia-largura) porque o knob que ele substituiu tinha o valor certo
/// em funcao de outros dois — medido, a 8 m/s um `0,30` deixava o personagem
/// CAIR e um `0,60` o segurava, com a fronteira exactamente em `0,533`.
/// v83 (`line/Vector`, item 4 do estudo dos contêineres — A TABELA SINAL → AÇÃO): o
/// `HostStates` ganhou **`on_signal`**, a lista de ligações *nome de sinal → papel*
/// (`ph2d_ui_state::SignalBinding`). Ele mora DENTRO do `HostStates` — e não numa
/// tabela própria — porque o `retain_hosts` já corre por frame: uma forma apagada leva
/// as ligações dela sem uma linha a mais, no mesmo frame e no mesmo passo de undo. É o
/// degrau irmão do `spring` (v62), no mesmo struct e pelo mesmo motivo posicional: o
/// postcard grava na ordem de declaração, então um leitor velho leria lixo bem-formado.
/// v84 (`line/Vector`, item 5 do estudo dos contêineres — A GRADE): o `LayoutDir` ganhou
/// **`Grid`** (variante APENDADA, logo `Row`/`Column`/`RowWrap` continuam em 0/1/2 e todo
/// arquivo já salvo segue legível) e o `VecLayout` ganhou **`columns`**.
/// ⚠️ O bump é pelo caminho **INVERSO**, e é o campo que o obriga: o postcard é
/// posicional, então um leitor velho leria os bytes do `columns` como o começo do que
/// vem a seguir. O número transforma isso num erro de VERSÃO — o raciocínio do
/// `Cap::Square` do Flip e do `JointKind::Weld` da física.
/// ⚠️ A contagem mora no `VecLayout` e **não** dentro do variante, para sobreviver a uma
/// troca de direção: ir a `Row` e voltar devolve a grade intacta, como o vão e o recuo já
/// fazem.
/// v85 (`line/Sprite`, plano [`docs/Sprite_projeto/17`] §3 — OS PIXELS GANHAM NOME): o
/// `ProjectFile` ganhou **`sprite_pixels`**, o documento do `ph2d-sprite-sheet`. Campo
/// apendado ⇒ bump pelo motivo posicional de sempre.
/// ⚠️ **O que este degrau CONSERTA é perda de dados em produção, não uma capacidade nova.**
/// `SpriteSource::Individual { texture_id }` guarda um id de alocação da GPU, e o
/// `IndividualTextureStore` recomeça a numerar em `1` a cada processo: um sprite tocado por
/// QUALQUER ferramenta de imagem (trim · bgremoval · make-square · padding · upscale ·
/// rasterize · equalize) reabria **invisível**, ou a exibir os pixels de outro sprite que
/// ficou com aquele id no restore. O Painter (v3) e o bake 3D (`baked_forms`) já tinham sido
/// resgatados um a um; este degrau é o **chão** que faltava debaixo dos dois.
/// ⚠️ **E ele bumpa UMA vez.** O blob carrega a própria versão (`SHEET_DOC_VERSION`), como o
/// `TimelineDoc` e o `sculpt` — então as regiões do hand-packed, que entram neste MESMO
/// documento, não voltarão a recusar projeto salvo nenhum.
/// v86 (`line/Sprite`, §5 9-Slice — A FORMA DO `SliceNine` MUDOU TRÊS VEZES NUM DIA): o
/// componente `SliceNine`, registado em `register_ecs_components`, perdeu o campo
/// **`stretch_value`** (o slider `Stretch` do `Adaptive`, retirado porque o mecanismo dele não
/// podia funcionar) e o `SliceDrawMode` perdeu a variante **`Tiled`** (ela era o `Sliced` menos a
/// capacidade de esticar uma região).
/// ⚠️ **O componente é name-keyed, mas o BLOB dele é posicional.** Um projeto gravado hoje de
/// manhã tem a mesma chave `"ph2d::ecs::SliceNine"` com o layout velho: sem este degrau o leitor
/// novo lê o `bool` do `fill_center` onde estavam os quatro bytes do `stretch_value`. É a lei que
/// os degraus v6/v7/v8 já escreveram — *não é campo novo no arquivo, é o MESMO campo com outro
/// layout, e posicional é posicional*.
/// ⚠️ **Um bump para as três mudanças, não três.** Elas caem no mesmo dia e no mesmo componente,
/// e nenhuma chegou ao `main`: o que o número tem de separar é o formato de ontem do de hoje.
/// ⚠️ Adicionar o `SliceNine` e o `NamedAnchorList` ao registo (2026-08-21) **não** pediu degrau —
/// isso é aditivo numa tabela por NOME, e um arquivo velho apenas não os tem. O que pede degrau é
/// **mudar a forma de uma chave que já existe**.
/// v87 (`line/Vector` — O RECORTE SAIU DA MOLDURA): o `VecFrame` perdeu o campo `clip` e virou
/// **marcador**, e o recorte passou a ser o componente próprio `ph2d_ecs::VecClipContent`, que
/// qualquer forma FECHADA pode carregar (Enio: *"coloque a feature Clip Content para qualquer
/// forma vetorial fechada"*).
/// ⚠️ **Quem obriga o bump é o campo REMOVIDO**, e é o caso inverso ao do v84: o postcard é
/// posicional, então um leitor novo sobre bytes velhos leria o `bool` do `clip` como o começo do
/// componente seguinte — e ao contrário de um campo apendado, aqui nem o tamanho bate. O número
/// transforma isso num erro de VERSÃO honesto.
/// ⚠️ E o registro de componentes subiu junto (63 → 64): um componente que não passa por
/// `register_ecs_components` é descartado em silêncio pelo snapshot, e o recorte evaporaria no
/// primeiro Ctrl+Z com a arte toda no lugar — o modo de falha mais enganoso da lista.
/// ⚠️ **Este degrau nasceu como v85 na `line/Vector` e foi RECONTADO na integração de
/// 2026-08-22**: a `line/Sprite` entrou antes com v85/v86, e *número que soma entre linhas se
/// CONTA, nunca se escolhe* (CLAUDE.md §5.0) — o handoff da linha ainda diz 85.
/// v88 (`line/Vector` — OS FILTROS NOS ESTADOS DE UI): o `ph2d_ui_state::ObjectPose` ganhou
/// **`filters`**, a pilha de FX raster daquele estado (`ph2d_fx_op::FxOp`), apendada ao fim.
/// ⚠️ **É o irmão exacto do `width`**, e pela mesma razão que aquele campo existe: os dois são
/// canais que NÃO vivem no `VecPath` — são componentes ECS (`VecStrokeProfile`, `VecFilter`) —,
/// então a pose tem de os carregar por si. Sem ele um blur era o único efeito do editor incapaz
/// de diferir entre *Default* e *Hover* (Enio, 2026-08-21).
/// ⚠️ O bump é posicional como sempre: o campo entra dentro de cada `ObjectPose`, que viaja no
/// `HostStates` (v83), que viaja no `ProjectFile`.
/// ⚠️ E o degrau MUDOU DE CASA na mesma wave — `FxOp` saiu do `ph2d-ecs` para a folha
/// `ph2d-fx-op`, para a `ph2d-ui-state` (que deliberadamente não vê ECS) o poder carregar. A
/// forma serializada é a MESMA; o que mudou foi de que crate o tipo vem.
/// ⚠️ Nasceu como v86 na `line/Vector`; RECONTADO para v88 na integração de 2026-08-22 (ver v87).
/// v89 (`line/Vector` — UM VERBO POR FORMA na booleana viva): o componente novo
/// `ph2d_ecs::VecBoolOp` guarda, **por forma**, a operação com que ela dobra sobre o resultado das
/// anteriores (Enio, 2026-08-22: *"o modo do boolean é escolhido por shape e na ordem em que
/// aparece na hierarquia atua sobre o resultante das operações pregressas"*).
/// ⚠️ **Quem obriga o bump é o REGISTRO, não um campo.** O componente não muda a forma de nenhum
/// tipo já serializado — ele é uma entrada NOVA no `ComponentRegistry` (64 → 65). E o modo de
/// falha de esquecer o registro é o mais enganoso desta escada inteira: um componente que não
/// passa por `register_ecs_components` é **descartado em silêncio** pelo snapshot, então o
/// primeiro Ctrl+Z devolveria a arte toda no lugar, a combinação intacta — e a receita achatada de
/// volta no `op` do grupo, desenhando outra coisa **sem nada em falta na tela a denunciá-lo**.
/// ⚠️ Ausência do componente continua a ser **herança** do `op` do grupo, e é isso que faz todo
/// arquivo ≤ v88 desenhar byte-idêntico: nenhuma forma o tem, todas herdam, e herdar é o que o
/// grupo já fazia.
/// ⚠️ Nasceu como v87 na `line/Vector` (o handoff dela diz «86 → 87»); RECONTADO para v89 na
/// integração de 2026-08-22 — a `line/Sprite` entrou antes com v85/v86 (ver v87).
/// v90 (`line/Sprite` — QUEM MONTA numa âncora): o componente novo `ph2d_ecs::AnchorMount`
/// guarda, na entidade FILHA, o **nome** da âncora do pai de que ela parte (ADR-0072 §2.6 — o
/// consumidor que o ADR declarou em 2026-05 e que nunca existiu).
/// ⚠️ **Quem obriga o bump é o REGISTRO, não um campo** — irmão exacto do v89, e com o mesmo modo
/// de falha enganoso: um componente fora do `ComponentRegistry` é **descartado em silêncio** pelo
/// snapshot, e então reabrir o projeto devolveria a espada como filha comum do personagem —
/// no sítio certo, parada. Nada some, nada avisa, e o defeito só aparece quando o braço se mexe.
/// ⚠️ Ausência do componente é **não montar**, que é o que toda entidade fazia até v89: por isso
/// todo arquivo ≤ v89 desenha byte-idêntico.
/// ⚠️ O componente guarda o NOME, nunca o índice na lista nem os bits da entidade — apagar a
/// âncora `0` faria toda a gente descer uma casa em silêncio, e *o undo respawna tudo com bits
/// novos*.
/// v91 (`line/Sprite` — QUANDO as âncoras se desenham): o componente novo
/// `ph2d_ecs::AnchorVisibility` guarda, no DONO das âncoras, duas intenções do artista (Enio,
/// 2026-08-23): mantê-las visíveis **sem a entidade estar selecionada**, e mantê-las visíveis
/// **em runtime**.
/// ⚠️ **Irmão do v90, e pela mesma razão: quem obriga o bump é o REGISTRO.** O modo de falha aqui é
/// dos que ninguém reporta como bug — marcar a caixa, gravar, reabrir, e ver os pontos voltarem a
/// aparecer só com o dono selecionado. O artista remarcaria a caixa todos os dias sem perceber que
/// ela nunca guardou.
/// ⚠️ **Componente SEPARADO do `NamedAnchorList`** de propósito: aquele é um newtype sobre um
/// `SmallVec` e o postcard é posicional — acrescentar-lhe campos faria todo projeto anterior ser
/// lido torto em silêncio.
/// ⚠️ Ausência do componente é «só quando selecionada», que é o que toda entidade fazia até v90.
/// v92 (`line/Sprite` — §11 ANIMATION): dois componentes novos, `ph2d_ecs::SpriteAnimations` (a
/// biblioteca de tags: intervalos nomeados sobre a grelha da sprite) e `ph2d_ecs::SpriteAnimator`
/// (o estado de reprodução).
/// ⚠️ **Terceiro degrau seguido em que quem obriga o bump é o REGISTRO**, e o modo de falha é o
/// mesmo: sem ele, o artista autora `idle`/`walk`/`attack`, grava, reabre — e a sprite volta a ser
/// uma grelha parada, **sem nada em falta no ecrã a denunciá-lo**.
/// ⚠️ O `SpriteAnimator` grava também o ESTADO (frame, ciclo, acumulador de tempo), e é isso que
/// faz o replay reproduzir a mesma animação — a razão de ele ser `SimComponent`.
/// ⚠️ Ausência dos dois é «sprite parada na `frame` atual», que é o que toda sprite fazia até v91.
/// v93 (`line/Sprite` — os SINAIS da §11, spec §8.10): a `AnimationTag` ganhou
/// `signal_on_finish` e `signal_on_loop` — os nomes que uma animação grita ao acabar e ao fechar
/// um ciclo.
/// ⚠️ **Campos APENDADOS no fim do struct, e o postcard é posicional**: um projeto de v92 lido
/// como v93 tentaria ler as duas strings dos bytes da tag seguinte. Falha alto na maioria dos
/// casos e **calada** quando o resto do buffer calhar a decodificar — que é o modo caro.
/// ⚠️ **Dois campos e não um mais uma fase**: é a lei que a física já escreveu para os contatos —
/// acabar e dar a volta distinguem-se por serem NOMES diferentes, autorados em dois sítios.
/// ⚠️ Ausência dos dois é «a animação é calada», que é o que toda animação fazia até v92.
/// v94 (`line/Sprite` — a DURAÇÃO POR-QUADRO, spec §8.12): a `AnimationTag` ganhou
/// `per_frame_ms: Vec<u32>`.
/// ⚠️ **É uma recusa medida que se REABRIU**: ela dizia *«não há quem produza durações
/// por-quadro»*, e o importador de `.ase` (construído no mesmo dia) é exactamente quem as produz —
/// nos ficheiros reais elas variam. *Quem move o número que tornava algo inalcançável tem de
/// reconferir a nota.*
/// ⚠️ Campo apendado, postcard posicional — e um `Vec` **vazio** é o comportamento de sempre, então
/// um projeto de v93 lido com o modelo novo comporta-se igual **depois** de migrar; antes disso,
/// falha alto no schema, que é o que este número existe para fazer.
/// v95 (`line/Vector` — A BOOLEANA VIVA NOS ESTADOS DE UI; ⚠️ nasceu como v90 na linha e foi
/// RECONTADO para v95 na integração de 2026-08-23 — a `line/Sprite` ocupou 90..94 antes): o `ph2d_ui_state::ObjectPose` ganhou
/// **DOIS** campos apendados ao fim — `bool_op` (o verbo próprio daquela forma naquele estado) e
/// `bool_group_op` (a operação do grupo booleano acima dela). Enio, 2026-08-23: *"Sistema Live
/// Boolean compatível plenamente com o sistema de animação States, inclusive com a possibilidade
/// de mudar o tipo do boolean no meio da animação"*.
/// ⚠️ **Dois campos e não um, porque são dois FATOS.** O primeiro é *"que verbo esta forma manda"*;
/// o segundo é *"em que operação ela está metida"* — e é o segundo que faz a receita INTEIRA do
/// grupo mudar entre dois estados, inclusive as quatro receitas (`Trim`/`Crop`/`Merge`/
/// `MinusBack`), que não têm decomposição por forma nenhuma. Um campo só teria de escolher qual
/// dos dois carregar, e a escolha calada é como um `Trim` autorado no Hover não anima nada.
/// ⚠️ **O `bool_group_op` repete-se em cada operando do mesmo grupo**, e a redundância é
/// deliberada: o grupo é uma entidade **sem `VecPathId`** e a pose é chaveada por caminho, então
/// ele não tem slot próprio. Quem o governa é a única chave que já existe.
/// ⚠️ **Nenhum registro novo no `ComponentRegistry`** — os dois componentes que estes campos
/// espelham (`VecBoolOp` em v89, `VecBoolGroup` antes dele) já lá estavam. Aqui quem obriga o bump
/// é o LAYOUT: postcard é posicional, e um leitor velho leria os bytes de `bool_op` como o começo
/// do `ObjectPose` seguinte.
/// ⚠️ **`None` nos dois é a identidade byte-a-byte de todo arquivo ≤ v94**: nenhuma pose antiga
/// nomeia verbo nenhum, e `None` no primeiro é *herda* (a lei do componente) enquanto no segundo é
/// *não sei de grupo nenhum* — que **nunca** desfaz um grupo.
/// ## v96 — ⭐ **A IDENTIDADE DO OBJETO, e a PRIMEIRA migração da história do repo**
/// ([ADR-0164](../../../docs/architecture/decisions/0164-instances-are-real-entities-linked-by-stableid-with-live-sync-and-incremental-undo.md) F1).
///
/// Dois fatos ao mesmo tempo, e é por isso que este degrau **tem migração** em vez de só
/// recusar (`crate::project_migrate`):
///
/// 1. **`WorldSnapshot` v1 → v2** — a linha passa a ser chaveada e ordenada por `StableId`, e
///    o `parent` deixa de ser um índice para ser um id. ⚠️ *Um índice desloca-se*: inserir uma
///    entidade mudava os bytes de todas as linhas seguintes, o que a captura incremental da F2
///    não pode pagar. E a ordem por id **apaga o `canonicalize`** do undo (18,7 ms → 0,088 ms
///    a 10 k entidades, medido).
/// 2. **`ProjectFile.stable_id_counter`** — o próximo id livre. ⚠️ Fora do `ProjectState`, e
///    não pela razão dos outros campos (o escopo do undo): um undo que o rebobinasse faria um
///    **redo** entregar um id ainda vivo.
///
/// ⚠️ **Nenhum dos dois é aditivo no wire.** O postcard é posicional e não auto-descritivo:
/// um leitor v96 sobre bytes v95 **não erra — lê errado**. Daí o tipo congelado
/// `ProjectFileV95` e o gate que guarda os BYTES de um ficheiro v95, não o tipo.
///
/// ⚠️ **Este degrau é o primeiro que não recusa o passado.** Até aqui a política de facto era
/// *"versão diferente = recusado"* (a auditoria de 21/08 registou-a como ambiguidade §8 item
/// 7: HR-14 exige `migrate_vN_to_vN+1` e o repo tinha **zero**). Um v95 agora abre.
///
/// ## v97 (`line/Vector` — O INPUT MAP): o `ProjectFile` ganhou **`input_map`** apendado ao fim — as
/// acções nomeadas do projecto (`ph2d_input::InputMap`), com as ligações de cada uma e os dois
/// números da zona. Enio, 2026-08-24: *"precisamos do input Map completo não apenas para o jogador
/// mas para qualquer objeto do game via UI"*.
/// ⚠️ **É AUTORIA, e é por isso que viaja no arquivo e não no `prefs.txt`**: `jump` é uma decisão
/// do projecto (o jogo tem um botão de pulo), enquanto *qual tecla* um jogador prefere é dele. O
/// segundo mora fora do repo, como o `motion_character` — a mesma divisão que o Godot faz entre as
/// project settings e o remap em runtime.
/// ⚠️ **Fora do `ProjectState`**, pelo motivo de sempre: aquele é a unidade do undo GLOBAL, e um
/// Ctrl+Z do canvas não pode rebobinar o mapa de controlos.
/// ⚠️ Campo apendado, postcard posicional — um mapa **vazio** é o comportamento de sempre (nenhuma
/// acção declarada ⇒ toda leitura devolve silêncio), então um projecto antigo comporta-se igual
/// **depois** de migrar.
/// ⚠️ **Nenhum registro novo no `ComponentRegistry`** — o mapa não é um componente: ele é do
/// PROJECTO, não de uma entidade. Quem o consome pergunta pelo nome.
///
/// ⚠️⚠️ **ESTE DEGRAU NASCEU `96` E FOI RECONTADO PARA `97` NA INTEGRAÇÃO de 2026-08-24.** Duas
/// linhas paralelas apendaram um campo no `ProjectFile` na mesma jornada e **as duas escreveram
/// o literal `96`** — o valor certo não estava em nenhum dos dois lados: conta-se
/// (95 + identidade + input map). ⛔ E a `collision-surface.sh` **não podia** ver esta colisão:
/// ela compara a linha com o **ponto de fork**, não com o tip do `main`, então a segunda linha
/// da jornada lê `base: 95` para um `main` que já estava em `96`.
///
/// ⛔ **E a nota original deste degrau dizia *"antes disso falha alto no schema"* — ela envelheceu
/// no mesmo instante em que foi escrita.** A linha irmã construiu a PRIMEIRA escada de migração do
/// repo um degrau abaixo; recusar aqui deixaria o trabalho dela morto à nascença (um v95 subiria
/// um degrau e bateria numa parede). Como este campo é apendado com default vazio, o v95 sobe
/// **direto** até aqui — ver `crate::project_migrate`.
/// ⚠️ **Não há degrau `96 -> 97`, e a ausência é a decisão:** a v96 nunca existiu fora destas duas
/// worktrees (nada foi publicado nela), então não há ficheiro v96 no mundo para migrar. Quem
/// precisar de um um dia, congela o tipo primeiro — como o `ProjectFileV95` foi congelado.
/// ⚠️ **97 → 98 (plano 32 W11c):** o `ObjectPose` do `ph2d-ui-state` ganhou `morph_shape` — *em que
/// forma o conjunto de Morph States está nesta pose*. As poses viajam **dentro** do `ProjectFile`
/// (o `StateSets`), então um campo novo nelas move o esquema do projecto.
///
/// ⛔ **NÃO há degrau de migração, e a ausência é uma DECISÃO do Enio** (2026-08-26: *"não há
/// projetos salvos. esse app está em fase inicial de desenvolvimento, podemos fazer o que
/// quisermos"*). Um ficheiro v97 é **recusado em voz alta** no `project_load` (`"schema {ver} !=
/// {PROJECT_SCHEMA} — recusado"`), que é o comportamento certo: postcard é posicional e
/// não-auto-descritivo, então **sem o bump ele leria os bytes errados em silêncio**. *O bump é o
/// que transforma um mal-entendido silencioso numa recusa legível.*
///
/// ⚠️⚠️ **ESTE DEGRAU NASCEU `98` E FOI RECONTADO PARA `99` NA INTEGRAÇÃO de 2026-08-26** — a
/// SEGUNDA vez que isto acontece neste ficheiro (ver o degrau `97` acima). A `line/Vector` e a
/// `line/components` mudaram o formato **por razões diferentes** e as duas escreveram o literal
/// `98`; o valor certo não estava em nenhum dos dois lados. ⛔ **E desta vez a `collision-surface.sh`
/// ficou CEGA por outro motivo:** depois de a primeira linha aterrar, o merge-base da segunda passa
/// a ser um `main` que já diz `98`, então ela lê `98 (base: 98)` — **sem aviso nenhum**, e o git
/// funde o literal repetido **limpo**. *Conte o delta de cada linha; não confie no aviso.*
///
/// ⚠️ **Um v97 é RECUSADO** (não migrado): o degrau `98` acima é da `line/Vector` e ela decidiu,
/// com o Enio, não congelar um tipo `ProjectFileV97`. Sem tipo congelado não há como ler aqueles
/// bytes sem os reinterpretar — que é exactamente o que o bump existe para impedir. O `v95`
/// continua a subir a escada inteira.
///
/// # 99 — o CORTE DA SPRITE (ADR-0164 F1 passo 6 / ADR-0166 / ADR-0070-amendment-8)
///
/// ⚠️⚠️ **A forma do `ProjectFile` NÃO mudou, e o degrau é obrigatório na mesma.** Os 20 campos
/// da `Sprite` v4 passaram a 13 (sete saíram para `SpriteGrid`/`SpriteRegion`/`SpriteCornerTint`),
/// e esses bytes vivem **dentro** do `Vec<u8>` opaco de um `ComponentBlob` — que o parse do
/// `ProjectFile` atravessa sem olhar. Um v98 lido por este binário abriria **sem erro** e cada
/// sprite leria 20 campos com um tipo de 13: lixo bem-formado.
///
/// ⛔ **É por isso que a tripla abaixo não é a defesa aqui.** Ela mede a forma da `VecScene` e do
/// `FlipDoc`; nenhuma das duas se mexeu. *Um degrau de schema não é só «a estrutura mudou» — é
/// «os bytes deixaram de significar o mesmo».*
///
/// A migração é uma travessia das linhas do snapshot (`crate::project_migrate_sprite`), não um
/// espelho do ficheiro: congelar 14 campos que não mudaram seria a cópia errada.
/// ⭐ **100 (2026-08-27) — o `ObjectInstance` ganhou os ORFÃOS** (ADR-0164 / F5.3): um segundo
/// campo (`orphans: BTreeMap<OverrideKey, Vec<u8>>`) dentro de um componente REGISTADO, e o
/// postcard é posicional. Um v99 lido por este binário atravessaria o `Vec<u8>` opaco do
/// `ComponentBlob` sem olhar e leria o fim da lista de overrides como o início do mapa — lixo
/// bem-formado, calado.
///
/// ⛔ **Sem degrau de migração, e está certo** — é a decisão do Enio de 26/08 (não há projetos
/// gravados). O número sobe para o load **recusar em voz alta** em vez de ler errado em silêncio.
///
/// # 101 — o TEXTURE PATTERN (plano 33, W3)
///
/// O `Paint` da `ph2d-vec-scene` ganhou a 5ª variante, `Pattern(Box<PatternFill>)`, e o
/// `VEC_SCENE_SCHEMA_VERSION` subiu **14 -> 15** — logo este número sobe por arrasto, e a **tripla**
/// de `project_schema_tests` vê este degrau (ao contrário do 99, que vivia dentro de um blob opaco).
///
/// ⚠️ **Apendar uma variante é aditivo NUM sentido só.** Um save v100 lido por este binário está
/// **correcto** — os índices anteriores não se mexeram. O que quebra é o inverso: um v100 com um
/// padrão, lido por um binário v100, encontra um índice de variante que não conhece e o postcard
/// falha longe da causa. O bump é o que transforma isso num erro de versão.
///
/// ⚠️ **E o degrau carrega DUAS mudanças, não uma:** além da variante, o `ProjectFile` ganhou
/// **`pattern_art`** (apendado ao fim) — os pixels que cada `Paint::Pattern` nomeia por `AssetId`.
/// Sem esse campo a fonte não resolveria ao reabrir e toda forma com padrão pintaria a cor de
/// recurso, **sem erro nenhum**.
///
/// ⛔ **Sem degrau de migração, pela mesma decisão do Enio de 26/08** (*"não há projetos salvos"*):
/// sem um `ProjectFileV100` congelado não há forma honesta de reler aqueles bytes, e um ficheiro
/// anterior é **recusado em voz alta** no `project_load`.
///
/// ⚠️⚠️ **E aqui a recusa é OBRIGATÓRIA, ao contrário do que a 1.ª redacção desta nota dizia.** Ela
/// dizia que um v100 seria *"lido certo pela regra posicional"* — verdade para a **variante**
/// apendada (os índices anteriores não se mexem), e **falso** desde que o `pattern_art` entrou: um
/// campo novo no fim faz o postcard de um v100 chegar ao fim dos bytes (`Hit the end of buffer`, o
/// mesmo modo de falha medido na v14 da `VecScene`). *Uma nota escrita entre as duas metades da
/// mesma wave descreve só a primeira.*
/// # 102 — o PADRÃO no TRAÇO (plano 35, wave A)
///
/// O `StrokeSpec` deixou de ter `color: Rgba8` e passou a ter `paint: StrokePaint`
/// (`Solid(Rgba8)` | `Pattern(Box<PatternFill>)`), e o `VEC_SCENE_SCHEMA_VERSION` subiu
/// **15 -> 16** — logo este sobe por arrasto, e a **tripla** de `project_schema_tests` vê o degrau.
///
/// ⚠️⚠️ **Este degrau é DESTRUTIVO nos dois sentidos, ao contrário do 100.** Ali uma variante foi
/// **apendada** a um enum e os índices anteriores ficaram onde estavam; aqui um campo **mudou de
/// tipo** no meio da estrutura: onde o postcard de um v100 tem os 4 bytes de um `Rgba8`, um leitor
/// v101 espera o **discriminante** de um enum. Os bytes não deixam de existir — eles passam a
/// significar outra coisa, e é o pior modo de falha que há: ⛔ *ler torto sem erro nenhum*.
///
/// ⛔ **Sem degrau de migração, pela mesma decisão do Enio de 26/08** (*"não há projetos salvos"*):
/// sem um `ProjectFileV100` congelado não há forma honesta de reler aqueles bytes, e um ficheiro
/// anterior é **recusado em voz alta** no `project_load`.
///
/// ⭐ E o `StrokePaint` foi desenhado para que o **próximo** degrau seja barato: um gradiente no
/// traço é uma variante **apendada**, do lado aditivo da regra.
/// # 103 — o PINCEL de contorno (plano 36, wave W1)
///
/// O `StrokePaint` ganhou `Brush(Box<BrushStroke>)` e o `VEC_SCENE_SCHEMA_VERSION` subiu
/// **16 -> 17** — logo este sobe por arrasto, e a **tripla** de `project_schema_tests` vê o degrau.
///
/// ⭐ **Do lado ADITIVO da regra, e a nota do 101 previu-o:** *"o `StrokePaint` foi desenhado para
/// que o próximo degrau seja barato — uma variante apendada"*. Os índices anteriores não se mexem,
/// então um v101 lido por v102 está correcto; o que quebra é o inverso, e é o número que o
/// transforma num erro de versão em vez de num postcard a falhar longe da causa.
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08 (*"não há projetos salvos"*).
/// # 106 — a ARTE de um padrão pode AINDA NÃO TER SIDO ESCOLHIDA (report do Enio, 2026-08-30)
///
/// O `PatternSource` ganhou a variante `None` e o `VEC_SCENE_SCHEMA_VERSION` subiu **17 -> 18** —
/// logo este sobe por arrasto, e a **tripla** de `project_schema_tests` vê o degrau.
///
/// ⭐ **Do lado ADITIVO da regra:** a variante é a ÚLTIMA, os índices `0` (`Image`) e `1` (`Shape`)
/// não se mexem, e nenhum campo mudou de tipo. Um ficheiro v103 lido por v106 estaria **correcto**
/// byte a byte; o que quebra é o inverso (um v106 com a variante nova lido por código v103 acha um
/// discriminante que não conhece), e é o número que o transforma num erro de versão em vez de num
/// postcard a falhar longe da causa.
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08 (*"não há projetos salvos"*).
///
/// ⚠️⚠️ **E o 106 foi CONTADO, não escolhido** (CLAUDE.md §5.0). Medido em 2026-08-30 nas oito
/// árvores vivas: `main` em **103**, `line/UIUX` em **104**, e a `line/3DModeling` **e** a
/// `line/components` **as duas em 105** — o mesmo literal em duas linhas, que é a colisão que funde
/// **muda**, porque o git não sabe o que o número significa. Esta linha toma o primeiro livre acima
/// do maior. *Quem integrar aquelas duas tem de recontar; este degrau não as desconflita.*
/// # 107 — o PREENCHIMENTO do balde é um componente REGISTADO (plano 40, 2026-09-01)
///
/// O `VecBucketFill` entrou no `ComponentRegistry` (`ph2d::ecs::VecBucketFill`): ele guarda a
/// **receita** de uma área preenchida — o ponto que o artista apontou —, e é o que a torna VIVA
/// (a área é re-cozida quando as linhas mudam).
///
/// ⚠️ **A tripla `(PROJECT, FLIP_DOC, VEC_SCENE)` NÃO vê este degrau**, e é o mesmo caso do 99 e do
/// 100: um componente viaja dentro de um `ComponentBlob` **opaco**, chaveado por nome. A forma da
/// `VecScene` não se mexeu.
///
/// ⚠️ **O degrau existe para o caminho INVERSO**, como o do `JointKind::Weld`: um ficheiro gravado
/// aqui traz um nome de componente que um binário anterior não conhece. Do lado aditivo (um v106
/// lido por v107) não há nada a fazer — o componente simplesmente não está lá, e uma área
/// preenchida antes deste degrau volta como forma **estática**, que é o que ela era.
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08 (*"não há projetos salvos"*).
///
/// ⚠️ **E o 107 foi CONTADO**: `106` era o desta árvore em 2026-09-01, e a nota do 106 continua a
/// valer — a `line/3DModeling` e a `line/components` escreveram **105 as duas**, e quem as integrar
/// tem de recontar.
///
/// # v107 → v108 — as ÂNCORAS do preenchimento (plano 40 §11)
///
/// O `VecBucketFill` ganhou `ancoras: Vec<FillAnchor>`, e com ele a receita de uma área deixou de
/// ser *onde ela estava* e passou a ser *os pedaços de linha que a cercavam*. ⚠️ **O campo é
/// APENDADO, e o postcard é posicional**: um ficheiro v107 lido por v108 chega ao fim dos bytes no
/// campo novo, e é o número que transforma isso num erro de versão em vez de um postcard a falhar
/// longe da causa.
///
/// ⛔ Sem degrau de migração, pela mesma decisão do Enio de 26/08 (*"não há projetos salvos"*).
///
/// # 108 -> 109 — a JUNTA entre as cópias de uma repetição (pedido do Enio, 2026-08-30)
///
/// A `ph2d_field::Unary::Array` e a `::Radial` ganharam um `Joint { chamfer, fillet }`, e o
/// `FIELD_DOC_VERSION` subiu **13 -> 14**. ⚠️ **Este número sobe por arrasto, e o caminho é o que
/// engana:** o doc do `FIELD_DOC_VERSION` diz que *"nada persiste um `FieldDoc`"* e isso é
/// literalmente verdade — mas a pilha de modificadores viaja, byte a byte e **posicionalmente**,
/// dentro do blob do componente `ph2d_field_ecs::FieldMods`, que está no `WorldSnapshot`.
///
/// ⛔⛔ **E NENHUM GATE LIGA OS DOIS NÚMEROS.** A tripla de `project_schema_tests` vigia
/// `PROJECT_SCHEMA × FLIP_SCHEMA_VERSION × VEC_SCENE_SCHEMA_VERSION`; o `FIELD_DOC_VERSION` não está
/// lá. Quem mexer numa `Primitive` ou num `Unary` tem de subir os dois **à mão**, e o instrumento
/// que o avisa é o `the_shape_of_a_saved_modifier_stack_is_pinned` da `ph2d-field`.
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 — um v103 é **recusado em voz alta**.
/// # 109 -> 110 — o CHANFRO em toda forma com aresta (pedido do Enio, 2026-08-30)
///
/// As **21 primitivas** que têm `round` ganharam um `chamfer` ao lado dele, e o
/// `FIELD_DOC_VERSION` subiu **14 -> 15**. ⚠️ Sobe por arrasto pelo mesmo caminho do 104: a
/// `Primitive` viaja, posicionalmente, dentro do blob do componente `ph2d_field_ecs::FieldNode`.
///
/// ⭐ **E este degrau os DOIS goldens de forma apanham** (`151 -> 159` e `86 -> 90`), ao contrário
/// dos v11-v13 do `FIELD_DOC_VERSION` — as fixturas deles instanciam primitivas.
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08.
/// # 110 -> 111 — o EIXO de cada modificador com direcção (pedido do Enio, 2026-08-31)
///
/// A `ph2d_field::Unary::Array`, `::Taper`, `::Radial`, `::Twist` e `::Bend` ganharam um
/// `axis: Axis`, e o `FIELD_DOC_VERSION` subiu **15 -> 16**. ⚠️ Sobe por arrasto pelo mesmo caminho
/// do 104: a pilha de modificadores viaja, posicionalmente, dentro do blob do componente
/// `ph2d_field_ecs::FieldMods`.
///
/// ⭐ **Do lado ADITIVO da regra**: o campo é o **último** de cada variante, então os índices
/// anteriores não se mexem e o eixo de nascimento é o que cada modificador já usava
/// (`ph2d_field::mods::ARRAY_AXIS` e irmãos) ⇒ o comportamento de toda peça é o de antes, **ao
/// bit**.
///
/// ⭐ **E o `the_shape_of_a_saved_modifier_stack_is_pinned` apanhou-o** — `77 -> 82`, um byte por
/// modificador —, que é exactamente o instrumento que o degrau 104 diz existir para este caso.
/// *A nota do 104 previu este dia e nomeou a ferramenta certa.*
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 — um v105 é **recusado em voz alta**.
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
pub(crate) const PROJECT_SCHEMA: u32 = 113;
