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
/// ⚠️ **A escada só tem aqui os degraus VIVOS (v60 →).** Os de **v2 a v59** foram para
/// [`docs/archive/project-schema-ladder-v2-v59.md`](../../../docs/archive/project-schema-ladder-v2-v59.md)
/// em 2026-08-23, quando este ficheiro bateu o teto de 600 LOC com **589 das 608 linhas em
/// doc-comment**. ⛔ Cortar para o arquivo, nunca declarar exceção — e o que se lê para contar
/// o próximo degrau é a **cauda** da escada, não o começo dela.
/// v60 (plano UI/UX W4c.4 — os tokens de ESCALA no DOCUMENTO): o `ph2d_ecs::BoundProp` ganha
/// **`StrokeWidth`(2)**, **`LayoutGapMain`(3)** e **`LayoutGapCross`(4)** — a espessura de um traço
/// e o vão de um auto layout passam a poder SEGUIR um token numérico, como a cor já seguia.
/// ⚠️ **Apendar variantes NÃO move `Fill`(0) nem `StrokeColor`(1)**, então todo binding já salvo
/// continua a ler; o bump é pelo caminho INVERSO, o mesmo raciocínio do v58/v59 acima.
/// ⚠️ **PROVISÓRIO** pelo mesmo motivo que o v56.
/// v61 (plano UI/UX **W2a** — o texto sabe medir-se): o `ph2d_ecs::VecTextParams` ganha
/// **`wrap_width: Option<f64>`**, a largura da caixa a que o texto REFLUI.
/// ⚠️ **Este é o único bump que o plano UI/UX previu e NOMEOU o preço** (§6.3): campo apendado
/// a componente EXISTENTE, e o blob de um componente é postcard **posicional** ⇒ um arquivo já
/// salvo lido pelo build novo bate no fim dos bytes. Um componente NOVO teria custado zero (o
/// precedente do `VecStrokeProfile`/ADR-0148 e dos overrides da física) — e foi recusado com
/// motivo: `wrap_width` é um número de layout ao lado do `align`/`tracking`/`line_height`, e
/// pô-lo noutro componente partiria a porta única `layout_of_params` em duas, com todo
/// consumidor que esquecesse a segunda a desenhar um texto sem refluxo **em silêncio**.
/// ⚠️ A dívida que este bump paga é a que a **F1.W1 da `line/runtime`** (*uma versão por
/// `ComponentBlob`*) apagaria — ela não existe, então o preço é este, escrito.
/// ⚠️ **PROVISÓRIO** pelo mesmo motivo que o v56.
/// v62 (plano UI/UX **W7m** — a MOLA como opção): o `ph2d_ui_state::HostStates` ganha
/// **`spring: Option<Spring>`**, a alternativa ao par *duração + curva*.
/// ⚠️ **Campo apendado a um struct SERIALIZADO** ⇒ postcard posicional ⇒ quebra dura, a mesma
/// classe do v61. Um `#[serde(default)]` **não salva**: o postcard não sinaliza ausência, então
/// um arquivo antigo bate no fim dos bytes de qualquer maneira.
/// ⚠️ **O sistema de easing fica INTACTO** — `duration_s` e `easing` continuam onde estavam e um
/// hospedeiro sem mola percorre o MESMO caminho de antes, byte a byte. A mola é uma `Option`
/// porque *ter mola* e *que mola* são a mesma decisão (o desenho do `wrap_width` do v61), e
/// desligá-la **não apaga** o que o artista afinou nas outras duas.
/// ⚠️ **PROVISÓRIO** pelo mesmo motivo que o v56.
/// v63 (3D, W10.7 — a oclusão que a doação carrega): o `BakedFormDocument` ganhou
/// **`form_occ`**, a oclusão de forma de um objeto assado (cavidade × os dois AOs),
/// um byte por texel. Bump obrigatório pela razão de sempre — postcard é POSICIONAL.
/// ⚠️ **E ela viaja em vez de ser assada no `base`**, o que seria de graça em disco: um
/// re-bake REUSA o `base` (`sculpt3d_bake::bake_one`), então pré-multiplicar a oclusão
/// ali a comporia a cada gesto e o objeto escureceria sozinho — o defeito exato que o
/// `base` existe para impedir. ⚠️ **VAZIO num documento anterior**, e é a leitura
/// honesta: o neutro da oclusão é `1.0`, e um plano de zeros pintaria de preto toda arte
/// já assada.
/// ⚠️ **A linha escreveu 56; o valor CONTADO contra o `main` da integração é 63** — a
/// `line/Vector` trouxe os SETE degraus v56..v62 na mesma janela, e o número se CONTA,
/// não se escolhe ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
/// v64 (physics, W13 — AS PAREDES): o `PlatformPlayer` ganhou **`wall_slide_speed`**,
/// **`wall_jump_height`**, **`wall_jump_push`** e **`wall_reach`** — a capacidade de
/// escorregar por uma parede e pular dela. Quatro campos apendados ao componente,
/// mesmo raciocínio posicional dos v32/v33/v34/v54/v55: um save v63 lido por v64
/// chega ao fim dos bytes no primeiro campo novo, e é o número que transforma isso
/// num erro de VERSÃO em vez de num postcard a falhar longe da causa.
/// ⚠️ **A linha escreveu 56; o valor CONTADO contra o `main` da integração é 64** — a
/// `line/Vector` (v56..v62) e a `line/sculpt3d` (v63) pousaram antes na mesma janela.
/// O número se CONTA, nunca se escolhe; três linhas já colidiram nele por o terem
/// escolhido ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
/// v65 (physics, W14 — O ARRANQUE): o `PlatformPlayer` ganhou **`dash_speed`**,
/// **`dash_time`** e **`dash_cooldown`**. Três campos apendados ao componente, pelo
/// mesmo raciocínio posicional de todos os degraus acima.
/// v66 (physics, W15 — O AGACHAR): o `PlatformPlayer` ganhou **`crouch_height`** e
/// **`crouch_speed`**. Dois campos apendados, e o motivo do bump é o de sempre —
/// postcard é posicional. ⚠️ Note o que este degrau **não** traz: nenhuma forma de
/// collider muda, porque agachar aqui é uma perna mais CURTA e não um corpo menor.
/// v67 (physics, W17 — A CORRIDA SOBREVIVE AO ARQUIVO): campo de ARQUIVO novo,
/// `player_tape`, com o que o dedo do jogador fez tique a tique. ⚠️ Ele fecha o
/// último item aberto do §4 do plano 06, e é o **bake da W16** que o torna útil:
/// a fita é a entrada que o bake replaya, então reabrir um projeto e apertar Bake
/// devolve a corrida de ontem. Medido: 60 s de corrida pesam **28,1 kB**.
/// v68 (physics, W23 — O AGARRAR-SE): o `PlatformPlayer` ganhou
/// **`wall_grab_stamina`**. Um campo apendado ao componente, pelo mesmo
/// raciocínio posicional de todos os degraus acima.
/// ⚠️ **Note o que este degrau NÃO traz:** o botão novo (`PlayerInput::grab`)
/// **não** move o formato da fita — ela guarda os botões num BITMASK (`(f32,
/// u8)`), e um bit livre não muda um byte do postcard. Uma corrida gravada antes
/// desta wave volta com o agarrar em zero, que é o que ela de facto tinha; um
/// campo novo na tupla teria custado o bump por si só e recusado todo arquivo já
/// salvo.
/// v69 (`line/motion-value`, doc 88 D3 — AS SETTINGS DO PROJETO viajam no arquivo):
/// campo de ARQUIVO novo, `settings` (`SavedSettings`) — a escala do mundo
/// (`pixels_per_meter`), a unidade que o artista LÊ, os dois snaps do gizmo e o modo
/// de filtragem. Fora do `ProjectState` pela razão dos irmãos `physics`/`motion`/
/// `timeline`: o `ProjectState` é a unidade do undo GLOBAL, e um Ctrl+Z do canvas
/// não deve rebobinar a escala do mundo.
/// ⚠️ **A linha escreveu 56 e o valor CONTADO é 69** — a `line/Vector` (v56..v62), a
/// `line/sculpt3d` (v63) e a `line/physics` (v64..v68) pousaram antes na mesma janela
/// ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
/// ⚠️ **E esta linha do degrau nasceu AUSENTE:** a integração renumerou o literal de
/// 56 para 69 e não escreveu a entrada, então a escada ficou documentando até v68 sob
/// uma const que dizia 69 — o buraco que faz o próximo bump nascer mal-numerado, pois
/// quem conta o próximo degrau lê a escada e não o literal. Escrita na varredura da
/// §5 do CLAUDE.md, no fim da mesma jornada.
/// v70 (physics, W-KinPush — O EMPURRÃO): o `PlatformPlayer` ganhou
/// **`reaction_push`**, o terceiro escalar da 3ª lei — quanto de um bloqueio
/// LATERAL volta para o corpo que o causou. Um campo do componente, pelo mesmo
/// raciocínio posicional de todos os degraus acima: o postcard grava na ordem de
/// declaração, então um leitor velho leria os campos seguintes deslocados.
/// ⚠️ **PROVISÓRIO até a integração** — o valor se CONTA contra o `main` do dia,
/// e nesta janela há outras linhas vivas
/// ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
/// v71 (physics, W-Swim — NADAR): o `PlatformPlayer` ganhou **três** campos
/// apendados — `swim_speed` (a capacidade; `0` desliga, e é assim que ela
/// nasce), `swim_acceleration` (a autoridade do servo contra o empuxo) e
/// `swim_enter` (**quantos PESOS o fluido tem de carregar** para o regime
/// armar). Os três num degrau só porque são uma capacidade só: quem escreve o
/// primeiro recebe um nado que funciona.
/// ⚠️ **O limiar não é uma altura, e não podia ser** — a mesma altura significa
/// coisas diferentes em cada poça; `1.0` é *a água sozinha me sustenta*, que é
/// por construção a linha de flutuação em qualquer densidade (a tabela está em
/// `measure_the_swim_threshold`).
/// ⚠️ **A FITA não se move**: o eixo vertical do nado sai dos botões `jump`/
/// `down`, que já viajam no BITMASK — um eixo novo teria mudado a forma de
/// `(f32, u8)` e recusado toda corrida já salva.
/// ⚠️ **PROVISÓRIO, como o degrau acima** — contado contra o `main` de hoje (70).
///
/// v72 (physics, W-Probes2 — OS SENSORES FICAM EDITÁVEIS): o `PlatformPlayer`
/// ganhou **quatro** campos apendados — `corner_samples` e `corner_lookahead`
/// (quantas amostras o perfil da quina varre, e quantos tiques de antecedência
/// ele olha), `wall_samples` e `wall_spread` (quantos raios o flanco casta, e
/// onde os de fora se sentam).
/// ⚠️ **Eram `const` e passam a ser AUTORADOS, com os defaults iguais às consts**
/// — todo player já salvo fica byte-idêntico, e o que muda é só quem pode mexer
/// neles. O report do Enio: *"não temos inputs para ajustes dos tamanhos e
/// posições dos sensores nem a quantidade de sensores"*.
/// ⚠️ **Um degrau só porque é um assunto só**: a `W-Probes` fechou a metade (b)
/// da §4.55 medindo que *cada NÚMERO tem row* — e não fez a pergunta que
/// faltava, que é sobre a GEOMETRIA das amostras.
/// ⚠️ **PROVISÓRIO** — contado contra o `main` do dia na integração.
///
/// v73 (physics, W-Probes2 — A PERNA VIRA UM LEQUE): o `PlatformPlayer` ganhou
/// `foot_samples` + `foot_spread`. ⚠️ **Este degrau MOVE FÍSICA**, ao contrário
/// do v72: o default de `foot_samples` é **3, não 1**, porque uma perna de um
/// raio só afunda **0,411 m — 46% do `float_height`** parada sobre uma fenda de
/// 10 cm num corpo de 40 cm que as bordas suportam
/// (`measure_what_a_single_ground_ray_costs_over_a_gap`). Um projeto salvo em
/// v72 reabre com a perna em leque, que é a correção.
///
/// v74 (physics, W-MultiJump — O PULO MÚLTIPLO): o `PlatformPlayer` ganhou
/// `air_jumps` + `air_jump_height`, no MEIO do struct (logo depois do
/// `jump_height`, que é onde eles se leem), e o postcard é posicional ⇒ quebra
/// dura. ⚠️ **Este degrau NÃO move física:** a contagem nasce em `0`, que é a
/// capacidade DESLIGADA — o precedente do wall slide e do wall jump —, então um
/// projeto salvo em v73 reabre com o pulo exatamente como estava.
///
/// v75 (physics, W-Ledge — A BEIRADA): o `PlatformPlayer` ganhou `ledge_grab` +
/// `ledge_speed`, apendados ao FIM, e o postcard é posicional ⇒ quebra dura.
/// ⚠️ **Este degrau NÃO move física:** o alcance nasce em `0`, que é a
/// capacidade DESLIGADA (o idioma de `coyote_time`/`corner_reach`/`air_jumps`),
/// então um projeto salvo em v74 reabre exatamente como estava — e o sensor
/// novo nem sequer é castado.
///
/// v76 (physics, W-Glide — PLANAR): o `PlatformPlayer` ganhou
/// `glide_fall_speed`, apendado ao FIM, e o postcard é posicional ⇒ quebra
/// dura. ⚠️ **Este degrau também NÃO move física:** o teto nasce em `0`, que é
/// a capacidade DESLIGADA, então um projeto salvo em v75 reabre a cair
/// exatamente como caía — e o `physics_ecs_c9` sai byte-idêntico, que é a prova
/// executável.
///
/// v77 (physics, W-LedgeSensor — O SENSOR DA BEIRADA): o `PlatformPlayer` ganhou
/// `ledge_reach_y` + `ledge_span`, apendados ao FIM, e o postcard é posicional ⇒
/// quebra dura. ⚠️ **Este degrau NÃO move física, e a razão é a redução
/// literal:** o `span` nasce em `0`, onde o leque tem **uma** amostra na posição
/// exacta do raio de antes, e o `reach_y` reproduz a janela `2·grab` de antes
/// quando vale o mesmo que o `grab` — as cenas de smoke autoram os dois em 0,60,
/// então elas reabrem idênticas e o `physics_ecs_c9` sai byte-idêntico.
///
/// ⚠️ **E o degrau é o único preço da wave**: os dois campos existem porque a
/// varredura da referência (GDevelop `Grab tolerance` + `Grab offset`, Corgi
/// *origem e comprimento*, os 5 traços do Unreal) refutou a frase *"o alcance é
/// uma grandeza só"* que o `grab` carregava.
/// v78 (physics, W-LedgeSensor — O OFFSET VERTICAL): o `PlatformPlayer` ganhou
/// `ledge_offset_y`, apendado ao FIM ⇒ quebra dura. ⚠️ **Ele existe porque o
/// `reach_y` é TAMANHO e não POSIÇÃO**, e o degrau v77 tinha mapeado os dois no
/// mesmo número (report do Enio: *"não temos como mover os sensores na
/// vertical"*). ⚠️ **Também NÃO move física:** nasce em `0`, onde a janela fica
/// centrada no topo do corpo como antes, e o `physics_ecs_c9` sai byte-idêntico.
/// v79 (physics, W-Brake — FREAR NÃO É ACELERAR): o `PlatformPlayer` ganhou
/// `brake_scale`, apendado ao FIM ⇒ quebra dura. ⚠️ **Ele existe porque a lei
/// gastava `acceleration` nos DOIS sentidos** — o fator de viragem cobre
/// *inverter* e não cobre *largar o direcional*, então um personagem que arranca
/// rápido era obrigado a parar rápido. ⚠️ **E o degrau é o ÚNICO preço da wave:**
/// o campo nasce em `1`, onde a lei reduz LITERALMENTE (`x * 1.0` é `x` em
/// IEEE-754) ⇒ todo projeto salvo em v78 reabre a andar e a parar exactamente
/// como estava, e o `physics_ecs_c9` sai byte-idêntico.
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
pub(crate) const PROJECT_SCHEMA: u32 = 94;
