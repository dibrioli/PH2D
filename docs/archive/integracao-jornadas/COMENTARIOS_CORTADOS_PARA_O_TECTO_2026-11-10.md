# Cabeçalhos de ficheiro cortados para baixo do tecto de LOC (integração 2026-11-10)

História verbatim dos `//!` removidos (o porquê histórico vive aqui, não no código — CLAUDE.md §2). Cada bloco está indentado (4 espaços) sob o caminho do ficheiro; o que ficou no código é o título e o 1.º parágrafo.

## `shells/desktop/src/build_smoke.rs`

    //!
    //! - `PH2D_BUILD_SMOKE=28` — a cena do **FALLOFF** (Cavalry; pesquisa `20_*` #10): três formas —
    //!   Zig Zag uniforme (controle), Zig Zag + Falloff Linear (cristas em rampa, selecionada), e
    //!   Warp Bulge + Falloff Radial (bojo localizado). O campo modula a FORÇA do deformador abaixo.
    //! - `PH2D_BUILD_SMOKE=25` — a cena do **CONTOUR** (pesquisa `20_*` #9): uma estrela PELADA
    //!   (selecionada — é ela que prova o seam, do `Add Contour` ao Expand) e um hexágono que já vem
    //!   com 6 anéis armados, para o olho ter a que comparar.
    //! - `PH2D_BUILD_SMOKE=27` — a cena do **ENVELOPE sobre ESTRELA** (o caso do Enio, 2026-07-24): uma
    //!   estrela CÔNCAVA deformada por gaiola de perspectiva (modo NODE), para provar que a fonte
    //!   côncava deforma limpo e as alças de canto ficam sãs (vive no irmão `envelope_smoke`).
    //! - `PH2D_BUILD_SMOKE=21` — a cena da **W0** (texto + efeitos): a palavra "PATH" com um Zig Zag
    //!   ATIVO, já selecionada. Mexa em qualquer knob da seção **Text** (ou escreva mais uma letra) —
    //!   **a rugosidade tem de continuar lá**. Antes da W0 ela sumia neste gesto, em silêncio.
    //! - `PH2D_BUILD_SMOKE=22` — a cena da **W3** (texto em CAMINHO): uma onda e dois círculos,
    //!   com o texto a cavalgá-los; a 2ª linha corre paralela, o 3º está virado.
    //! - `PH2D_BUILD_SMOKE=24` — a cena do **GESTO** do Pattern Along Path (plano 23, W3): um motivo
    //!   (seta) e um guia (arco), os dois selecionados — clique **Pattern on Path** no painel e afine
    //!   Spacing/Start/Side; as cópias giradas sobre a curva, a fonte intocada.
    //! - `PH2D_BUILD_SMOKE=23` — a cena do **GESTO** (W4): um texto e uma curva já selecionados,
    //!   para o artista os prender pelo painel (seção Text on Path).
    //! - `PH2D_BUILD_SMOKE=19` — a cena do 17 com o **fluxo do report de 2026-07-20**: arma o
    //!   Corner Round ANTES, arrasta o Offset até SATURAR (o gesto natural), solta e retuna
    //!   Bevel → Miter. Com a lei da forma (±maxdim/2) o saturado é "a forma dobrada" e cada
    //!   retune muda pixels visíveis — era o regime join-inerte da faixa antiga (±4 de mundo).
    //! - `PH2D_BUILD_SMOKE=18` — a MESMA cena do 17, com o roteiro do **RETUNE auto-dirigido**
    //!   (diagnóstico): rola o painel, arrasta o slider de Offset com o ponteiro a um d
    //!   moderado, solta, clica Round, Bevel e Miter — e loga por frame o custo, o undo, a
    //!   janela e os verts.
    //! - `PH2D_BUILD_SMOKE=17` — a cena do **EXPAND**: um zig-zag só-traço, uma estrela com traço E
    //!   preenchimento, e um donut de parede fina. Outline Stroke nos dois primeiros, Offset Path no
    //!   terceiro (a borda cresce e o furo encolhe).
    //! - `PH2D_BUILD_SMOKE=16` — a cena das ferramentas de **QUINA** (Fillet / Chamfer): um retângulo
    //!   de quinas retas + uma elipse de âncoras suaves, no modo Fillet. Clique uma quina e ARRASTE
    //!   para arredondar (Fillet) ou chanfrar (Chamfer, pelo pill). Na elipse o clique transforma a
    //!   âncora suave em quina primeiro. É a consolidação da alça do Node + o toggle da seção Vertex.
    //! - `PH2D_BUILD_SMOKE=15` — a cena do **CHAMFER** (ADR-0121): um quadrado de quinas arredondadas,
    //!   no modo Node, com as 4 quinas selecionadas. Na seção **Vertex** clique **Chamfer** — cada
    //!   quina vira uma RETA de mesmo recuo. Arraste a alça de raio e o estilo sobrevive.
    //! - `PH2D_BUILD_SMOKE=14` — a cena do **APPLY / CONVERT** (ADR-0132): uma elipse com um Zig Zag
    //!   ATIVO, ESTÁTICA e já selecionada. Na seção **Effects** clique **Apply Effects** (assa a
    //!   pilha na geometria); ou **Convert to Curves** (que agora também assa efeitos). O card some e
    //!   a borda rugosa vira curva editável.
    //! - `PH2D_BUILD_SMOKE=13` — a cena da **PILHA DE EFEITOS** (ADR-0132): uma elipse que se
    //!   DESENHA sozinha (o `end` do Trim sobe de 0 a 1) e uma estrela em que a janela do Trim
    //!   GIRA à volta da forma. A ponta anda a velocidade constante — é a medida por ARCO.
    //! - `PH2D_BUILD_SMOKE=12` — a cena do **WARP GROUP** (ADR-0129 Fatia 3): DUAS elipses sob UMA
    //!   gaiola. As duas curvam pela mesma perspectiva (NODE), e o gizmo do Select abraça e move as
    //!   duas juntas (SELECT). É o que separa um container de dois envelopes soltos.
    //! - `PH2D_BUILD_SMOKE=11` — a cena do **ENVELOPE** (ADR-0129): uma elipse **já deformada** por
    //!   uma gaiola de perspectiva. OLHE O MEIO DOS SEGMENTOS — as laterais curvam liso; é aí que
    //!   o defeito ingênuo (só os cantos) apareceria, não nos cantos.
    //! - `PH2D_BUILD_SMOKE=10` — a cena do **MORPH** (o `t` animável): um quadrado e uma estrela, já
    //!   SELECIONADOS. Clique **Morph** no painel — nasce UMA forma no meio do caminho. Arraste
    //!   **Morph t**: ela caminha entre as duas, ao vivo. As fontes ficam (mexa numa e a forma
    //!   refaz-se). Para animar: com o morph selecionado, **+ Track → Morph** na timeline, ponha o
    //!   playhead e aperte **K** — o `t` vira uma curva.
    //! - `PH2D_BUILD_SMOKE=9` — estrela → **círculo**, 5 passos — o par que o Enio testou à mão. A
    //!   transição tem de ser limpa (as pontas encolhem radialmente, sem torcer). Rotate/Reverse Match
    //!   foram removidos; o ajuste é editar as formas-fonte.
    //! - `PH2D_BUILD_SMOKE=8` — a cena do **GIRO**: quadrado → **círculo**, 5 passos. É o par do 2º
    //!   smoke do Enio (*"o porquê da rotação?"*). As quinas têm de caminhar **reto** para fora — sem
    //!   rodar 45° e voltar.
    //! - `PH2D_BUILD_SMOKE=7` — a cena do **BLEND**: um quadrado e uma estrela, com 3 passos entre eles.
    //!   As quinas do quadrado casam com as PONTAS da estrela (não os vales), e as arestas retas ficam
    //!   retas em todo o caminho.
    //! - `PH2D_BUILD_SMOKE=1` — a cena, selecionada, no modo Build. **Passe o mouse** (o realce
    //!   segue o cursor), **arraste** para unir, **Alt+arraste** para apagar.
    //! - `PH2D_BUILD_SMOKE=2` — idem, e o gesto é dirigido por CÓDIGO: o dedo pousa e arrasta por
    //!   duas faces, sem soltar. É o harness visual do véu — a única parte da feature cujo
    //!   oráculo é o pixel, e a que estava sem gate quando o Enio reprovou.

## `shells/desktop/src/ui_sound.rs`

    //!
    //! # ⭐⭐ A LEI: um som CONFIRMA o que a mão fez, nunca ANUNCIA o que o app decidiu
    //!
    //! É a única linha que separa um som de UI bom de um irritante, e ela tem duas metades:
    //!
    //! - **só o que a MÃO causou soa** — um clique, um commit, uma recusa. O que o app faz sozinho
    //!   (um quadro, uma transição a assentar, um painel a publicar) é **mudo**;
    //! - **o HOVER nunca soa.** Passar o rato por cima de uma fileira de botões é o gesto mais barato
    //!   e mais frequente do editor; sonorizá-lo transforma navegar num chocalho. É a irmã exacta da
    //!   cerca do §6.2 do estudo (*o realce de uma lista obedece ao cursor; oito rows meio-acesas ao
    //!   mesmo tempo é rasto, não vida*).
    //!
    //! Há gate para as duas.
    //!
    //! # E o vocabulário é CURTO de propósito
    //!
    //! Quatro sons. Um vocabulário grande obriga o ouvido a aprender uma língua para ganhar o que uma
    //! confirmação já dá — e cada som a mais é uma chance a mais de o artista desligar o conjunto.
    //!
    //! ⚠️ **Sintetizados, sem ficheiro nenhum**: o motor já traz os geradores
    //! ([`ph2d_audio::signals`]), e um asset de UI seria mais um binário a versionar, a licenciar e
    //! a manter afinado com o tema.
    //!
    //! # ⛔⛔ O QUINTO ELO da cadeia mora FORA do processo — e foi ele que partiu primeiro
    //!
    //! Medido 2026-08-23, no primeiro smoke desta feature: **silêncio total**, e não era nosso. Os
    //! quatro elos que o `PH2D_UI_SOUND_DIAG` sabe interrogar estavam todos verdes — `pref=true`,
    //! `dispositivo=true`, a voz alocada, o bus SFX. O `PH2D_AUDIO_SMOKE` (440 Hz, 600 ms, ganho 0,4)
    //! **também** era mudo, o que já excluía "o meu blip é curto/baixo demais". O elo partido era o
    //! **mixer por-aplicação do SO**:
    //!
    //! ```text
    //!   Sink Input #29923 · application.name = "PipeWire ALSA [ph2d-host-desktop]"
    //!   Mute: yes
    //!   module-stream-restore.id = "sink-input-by-application-name:PipeWire ALSA [ph2d-host-desktop]"
    //! ```
    //!
    //! …e o `mute:true` estava **gravado em disco**, em
    //! `~/.local/state/wireplumber/stream-properties`, indexado pelo NOME da aplicação. Um mute dado
    //! uma vez (um clique no applet de volume, meses antes) sobrevive a todo `cargo run` seguinte —
    //! *incluindo* aos builds novos, porque a chave é o nome, não o binário.
    //!
    //! ⚠️ **`cpal` não expõe isto, e por isso o diag NÃO o pode medir.** A API de saída dá o
    //! dispositivo e o formato; o volume/mute por-stream é do servidor de som, um andar acima. É um
    //! limite real da fronteira, não uma lacuna de implementação — e é exactamente por isso que ele
    //! fica escrito aqui e impresso pelo diag como **ponteiro**: a única coisa que o processo pode
    //! fazer por quem está a depurar é dizer-lhe onde olhar a seguir.
    //!
    //! O comando que responde, e a cura:
    //!
    //! ```text
    //!   pactl list sink-inputs | grep -E 'application.name|Mute:'   # com o app ABERTO
    //!   pactl set-sink-input-mute <id> 0                            # e o WirePlumber grava
    //! ```
    //!
    //! ⚠️ **O sintoma é indistinguível de "a feature não funciona"**, e essa é a lição durável: um
    //! canal de saída atravessa processos, e a suíte inteira deste repo cobre apenas o nosso lado
    //! dele. *Um gate verde sobre um canal mudo continua verde.*

## `shells/desktop/src/vec_component_general.rs`

    //!
    //! | | vetorial (velho) | geral (novo) |
    //! |---|---|---|
    //! | a cópia na Hierarquia | **uma** linha (o suporte) | a sub-árvore, com as peças nomeadas |
    //! | editar uma peça da cópia | só pelos slots do painel (`MAX_INSTANCE_PIECES`) | seleccionar a peça e usar qualquer ferramenta |
    //! | onde a excepção mora | `VecInstance.overrides` | `ObjectInstance.overrides` |
    //!
    //! *Dois motores para o mesmo estado é pior que um motor lento* — e é a razão pela qual a F4.6c
    //! existe no plano desde o início.
    //!
    //! # ⭐⭐⭐ Ele é o caminho de OMISSÃO desde 2026-09-06, e a data importa
    //!
    //! Ele **nasceu desligado**, e isso não era timidez: esta secção é um fluxo que **o dono usa** e
    //! que **eu não posso smokar** — ela vive no editor vetorial, atrás de uma janela. Trocar o motor
    //! por baixo dela e entregar sem rede seria pedir-lhe que descobrisse a regressão por mim.
    //!
    //! ⇒ o precedente é **o do `PH2D_RETOPO_EXTRACT`, inteiro**: shipa desligado, o dono smoka, e
    //! **quando ele aprova, o motor novo passa a ser o caminho de omissão**. Aqui foram **três** smokes
    //! aprovados em sequência (o modo · o conta-gotas do *Swap* · a cascata + o *Make Variant*), e a
    //! última coisa que faltava — os três controlos que o motor velho tinha a mais — está medida na
    //! tabela abaixo. *A lei «tudo o que é novo shipa desligado» vale enquanto houver algo por fechar;
    //! ela não é um estado permanente.*
    //!
    //! ⛔⛔ **E O INTERRUPTOR MORREU COM O OUTRO MOTOR** (F4.6c wave 3, 2026-09-07). Aqui viveu um
    //! `armed()` — `PH2D_VEC_COMPONENT_GENERAL=0` voltava ao motor `VecInstance` — lido nos **três**
    //! sítios que decidiam. O bloqueio daquela wave era **prático e não técnico** (apagar ~2 961 LOC
    //! em 27 ficheiros com a `line/Vector` viva era catástrofe de merge), e dissolveu-se no dia em
    //! que aquela linha integrou. *Não há segundo motor, logo não há o que escolher.*
    //!
    //! ⚠️ **Uma instância vetorial GRAVADA antes de 2026-09-06 lê-se como forma comum** (o painel
    //! oferece-lhe *Make Prefab*), e a partir de então não havia como criar uma. Não há projectos
    //! gravados — decisão do Enio, registada no §5 —, e o `PROJECT_SCHEMA` sobe com este corte para
    //! que um ficheiro que carregasse aquele componente **recuse em voz alta** em vez de ser lido
    //! errado em silêncio.
    //!
    //! # ⭐⭐⭐ Os TRÊS controlos que o motor velho tinha a mais, e onde cada um foi parar
    //!
    //! A F4.6c media *«um porte MENOS três features»*. Medidas uma a uma em 2026-09-06, **duas já
    //! existiam noutra superfície do modelo geral** e a terceira foi construída:
    //!
    //! | controlo do painel vetorial | no modelo geral |
    //! |---|---|
    //! | **conta-gotas do *Swap*** | ⭐ **construído aqui** ([`swap_by_pick`]) — o mesmo gesto de duas mãos, com o alvo resolvido pelo `master_subject` e re-key determinístico, que o *Swap* do vetor nunca teve |
    //! | **lista de peças** (o olho · a cor por peça) | ⛔ **não se porta: as peças são ENTIDADES REAIS.** O olho é o da Hierarquia e a cor é qualquer ferramenta sobre a peça seleccionada — as duas portas estão medidas em `ph2d_app_components::instance_structure::instance_piece_override_tests`. *A lista existia porque naquele modelo a peça não tinha endereço; aqui tem.* |
    //! | **fileira de variants** | ⛔ **não se porta: ela já é pintada**, no cartão *Properties* do Inspector (`variant_axes::axes_for`, alimentado pelo `family_members`), derivada pelo MESMO mapa que a troca usa. Uma segunda fileira noutro painel seria a mesma pergunta com duas respostas. |
    //!
    //! ⚠️ **E é por isso que a shell publica as duas listas VAZIAS** neste modo — não por elas ficarem
    //! vazias sozinhas (o `VecInstance` viaja na cópia profunda), mas por DECLARAÇÃO. *Um controlo que
    //! aparece e não faz nada é o defeito que este repo caça.*

## `shells/desktop/src/canvas_zoom.rs`

    //!
    //! ## O papel é `Surface`, e não é gosto
    //!
    //! [`Role::Surface`] nasceu do smoke da rolagem (*«o balanço das labels ficou bem artificial»*) e
    //! diz o que uma superfície comandada pelo dedo É: **nunca ultrapassa, nos dois carácteres**. Um
    //! zoom que passa do destino e volta não lê como peso — lê como enjoo, e mostra o que os dois
    //! clamps existem para proibir. E porque a mola é criticamente amortecida, **o vivo nunca sai do
    //! intervalo** em que o destino foi cravado: a garantia é a matemática, não uma segunda checagem.
    //!
    //! ⚠️ Sob *reduced motion* a [`UiMotion::law`] devolve `None` e a roda volta a ser
    //! instantânea — **byte-idêntica** ao que sempre shipou.
    //!
    //! ## O percurso é medido em LOG, e o que ele compra é SIMETRIA
    //!
    //! O zoom é multiplicativo: a roda entrega `0,9^n`, e o que o olho julga é `ln(h)`. Interpolar
    //! `height_world` cru dá um percurso cujo ritmo PERCEBIDO depende do sentido — medido a meio
    //! caminho da mola: **41,5% percorrido a aproximar contra 58,5% a afastar** num fator de 2. O
    //! mesmo gesto, dois ritmos. Em `ln(h)` são **50,0% nos dois**, por construção.
    //!
    //! | fator | a meio da mola, aproximar | afastar | desvio |
    //! |---|---|---|---|
    //! | um entalhe (0,900) | 48,7% | 51,3% | 2,6 pontos |
    //! | uma rajada de cinco (0,590) | 43,5% | 56,5% | **13,0 pontos** |
    //! | 2× (0,500) | 41,5% | 58,5% | 17,0 pontos |
    //!
    //! Um entalhe isolado não se vê; **a rajada vê-se**, e a roda é girada em rajada.
    //!
    //! ⚠️ **E a justificação que eu ia shipar estava ERRADA — a derivação apanhou-a antes do código.**
    //! Eu tinha escrito que o espaço linear faz *um entalhe* medir «200× mais no topo do intervalo que
    //! no fundo» (`0,05` contra `10`) e que por isso a lei da interrupção (`v / span`) se comportaria
    //! de forma diferente perto e longe. O primeiro número é verdade e **o segundo é inerte**: em
    //! `h(t) = h₀·(1 + (f−1)x)` o `h₀` sai de TODAS as grandezas — do valor, da velocidade e do span —,
    //! então `v / span` não depende dele. Medido em três níveis de zoom, o espaço linear dá
    //! **48,6836% em todos**. *Um número verdadeiro sobre uma grandeza não é um argumento sobre o
    //! comportamento dela*, e o gate que eu ia escrever para o provar não tocava neste módulo — teria
    //! ficado verde por vácuo sobre a mudança que ele julgava vigiar.
    //!
    //! ## A posse da câmera volta quando o gesto acaba
    //!
    //! O destino é um [`Option`]: enquanto ninguém dá zoom, este módulo **não toca** na câmera e ela
    //! é de quem a escrever (o *fit-to-view*, o load, as cenas de smoke). Ao chegar um entalhe o
    //! destino é semeado a partir do **vivo**, então ele não pode nascer velho — o defeito das duas
    //! cópias é impossível por construção, e não por disciplina.
    //!
    //! ⚠️ E uma escrita ESTRANGEIRA no meio de um voo ganha: o tique compara a câmera com o que ele
    //! próprio publicou no quadro anterior — *enumerar os escritores apodrece, uma testemunha não*.

## `shells/desktop/src/render_loop/ui_preview.rs`

    //!
    //! 1. **Um hover que animasse a forma enquanto o artista trabalha tornaria o editor inutilizável.**
    //!    É por isso que o Figma põe a interação num *modo de apresentação* separado.
    //! 2. **O undo deste editor é por DIFF do mundo**, então uma pose escrita por hover viraria um
    //!    passo de undo a cada vez que o rato passasse por cima de um botão.
    //!
    //! ⇒ o modo resolve as duas de uma vez: **enquanto ele está ligado, o gesto de edição não existe e
    //! o undo não regista**; quando ele desliga, **o mundo volta exactamente ao que era**.
    //!
    //! # ⭐ Sair RESTAURA, e a restauração é do MUNDO — nunca do estado Default
    //!
    //! A tentação barata é *"ao sair, vá para o Default"*. Ela está errada, e o modo de falha é
    //! silencioso: o **Default é uma pose GRAVADA**, e o artista pode ter movido a forma depois de a
    //! gravar. Sair para o Default **moveria** o desenho dele — o documento mudaria por ele ter olhado.
    //!
    //! O que se restaura é a pose que o mundo tinha **no instante em que a preview foi ligada**. Ela é
    //! capturada pela MESMA porta que o botão *Rec* usa ([`crate::vec_ui_state_edit::capture`]): uma
    //! segunda leitura ao lado seria a que esquece um canal no dia em que a pose ganhar um.
    //!
    //! # O conjunto capturado cobre tudo o que a preview pode escrever — mas ele NÃO é uma lista de ids
    //!
    //! A [`ph2d_ui_state::Machine`] só emite poses cujos ids aparecem nos estados autorados (o
    //! `overlay` dela escreve o que a `Transition` produziu, e a `Transition` casa as duas listas
    //! autoradas). ⇒ capturar todo id mencionado por qualquer estado de qualquer hospedeiro é
    //! **completo por construção**, e não uma lista que envelhece. Há gate a medi-lo.
    //!
    //! ⚠️ **E desde 2026-08-23 uma pose escreve fora do próprio id**: o canal `bool_group_op` faz o
    //! `install` escrever o `VecBoolGroup` da entidade de **GRUPO** acima da forma — e um grupo não
    //! tem `VecPathId`, logo **não pode estar no conjunto capturado**. A auditoria apanhou a frase
    //! antiga (*"EXACTAMENTE o que a preview pode escrever"*) a afirmar mais do que ela mede: o gate
    //! itera IDS, e essa escrita não é endereçada por id nenhum.
    //!
    //! ⭐ **O restauro continua correto, e é por REDUNDÂNCIA:** a pose capturada de cada operando
    //! carrega a operação do grupo, então reinstalá-la devolve o grupo. Ou seja o conjunto **cobre** a
    //! escrita sem a **conter** — e as duas coisas não são a mesma, então há gate PRÓPRIO a medir a
    //! volta (`the_group_operation_comes_back_when_the_preview_leaves`).
    //!
    //! # O rato diz *o que aconteceu*; o papel é DERIVADO
    //!
    //! `hot` (sobre que hospedeiro o cursor está) + `pressed` (o botão está em baixo) são os dois
    //! fatos que a shell conhece. O papel sai deles por uma função, e não por uma tabela de gatilhos:
    //! uma tabela teria de ser mantida em dia com a lista de papéis, e a lista já é um catálogo fixo.

## `shells/desktop/src/morph_fade_smoke.rs`

    //!
    //! - um **Morph sob um fade-in teleportava 0,700 num frame** (estalava na forma A);
    //! - um prop-link `Nome.morph` / `Nome.position` resolvia **0** em silêncio, com o
    //!   parser aceitando o token e a aresta de dependência montada — um controle que MENTE.
    //!
    //! A cena mostra os dois de uma vez, e é uma cena de **SCRUB**, não de play: o defeito
    //! vivia num único quadro, e um playhead correndo passa por cima dele.
    //!
    //! **A cena** (unidades de mundo, ~±6):
    //!   - **"Morpher"** — um morph REAL entre um círculo (esquerda) e uma estrela (direita).
    //!     A pose autorada é `t = 0.85` (quase a estrela). Uma track de **Morph** keya `0.15`
    //!     (quase o círculo) num clip, e a strip que a toca vive em `[2, 6)s` com fade-in.
    //!   - **"Needle"** (âmbar) — um sprite cujo **Y é dirigido pelo prop-link
    //!     `Morpher.morph * 6 - 3`**: ele é o mostrador do canal. Sem o fix ele fica cravado
    //!     em −3 para sempre, porque o link resolve 0.
    //!
    //! **O que provar** (arraste a régua devagar — NÃO dê play):
    //! 1. **Em `t < 2`** (antes do fade abrir) a forma está na pose AUTORADA (perto da
    //!    estrela) e a agulha está no ALTO. Este é o quadro que quebrava.
    //! 2. **Atravesse `t = 2.0` devagar.** A forma tem de COMEÇAR a caminhar dali — sem estalo.
    //!    Antes do fix ela saltava para o círculo no primeiro quadro do fade.
    //! 3. **`t` de 2 a 3** — a forma caminha até quase o círculo e a **agulha desce junto**,
    //!    continuamente. A agulha se mexendo É o prop-link funcionando.
    //! 4. **`t > 3`** — segura em `0.15`, agulha embaixo.
    //!
    //! **A trajetória, MEDIDA headless nesta configuração exata** (não é estimativa):
    //!
    //! ```text
    //!      t    morph.t   agulha.y
    //!   0.00     0.8500     2.1000   <- a pose AUTORADA; sem o fix, 0.0000 / -3.0000
    //!   1.95     0.8500     2.1000
    //!   2.00     0.8500     2.1000   <- o fade ABRE aqui e parte DAQUI (era o teleporte)
    //!   2.25     0.7406     1.4438
    //!   2.50     0.5000     0.0000
    //!   2.75     0.2594    -1.4437
    //!   3.00     0.1500    -2.1000   <- o fade fecha
    //!   4.00     0.1500    -2.1000
    //!   pior passo num frame (20 Hz): 0.0523   (o defeito era um salto de 0,7000)
    //! ```
    //!
    //! ⚠️ Se a linha `[morph-fade-smoke]` não aparecer, PARE: a cena não montou, e o resto
    //! do smoke não significa nada.

## `shells/desktop/src/sequence_smoke.rs`

    //!
    //! | | o que tem | o que FAZ |
    //! |---|---|---|
    //! | **esquerda — «Door»** | um **Sequence Player** a apontar à cutscene `Abrir` | sobe, ao ritmo do relógio DELA |
    //! | **direita — «Door (no cutscene)»** | o MESMO relógio e a MESMA tabela de acções | fica parada: o relógio corre e não há sequência a correr com ele |
    //!
    //! ⭐⭐⭐ **É o CONTROLO que torna a wave legível.** Sem ele, a porta da esquerda parece uma porta
    //! animada — com ele, vê-se *o que o componente compra*: **a timeline vira uma cutscene que um
    //! objecto toca no relógio dele**, e não uma animação presa ao relógio da cena.
    //!
    //! # ⚠️ A cena tem DUAS cutscenes, e a segunda é o ponto do PAINEL
    //!
    //! `Abrir` (a porta sobe) e `Abanar` (a porta treme no sítio). Trocar uma pela outra no selector da
    //! secção *Sequence* muda o que a porta faz **sem tocar na timeline** — que é a metade desta wave
    //! que o artista alcança com o dedo. ⛔ *Uma só cutscene deixaria o selector a mostrar uma lista
    //! com um item, e um selector sem escolha não se lê como selector.*
    //!
    //! # ⛔ A arrumação da timeline fica VAZIA, e isso é load-bearing
    //!
    //! Nenhuma das duas cutscenes é colocada na faixa da cena. A vista *Arrange* com a pilha vazia
    //! **não toca nada** (a lei do `apply_scene`, ordem do dono em 2026-07-27), logo tudo o que se vê a
    //! mexer veio do `SequencePlayer` — e não do playhead da cena a passar por cima de um clip. *Com
    //! uma instância na faixa, a cena ensinaria o contrário do que diz.*
    //!
    //! # ⛔⛔ E a cena NÃO abre a timeline — a ausência é load-bearing, e a foto foi quem a mediu
    //!
    //! A vista **Keys** *sola* a clip que o animador edita e congela o relógio da cena, logo a fase das
    //! cutscenes **não corre lá** (a condição `container.is_none() && !solo` do `timeline_bridge`). E
    //! escolher um objecto **pede a aba Keys** (`selection_jumps_to_keys`) ⇒ uma cena que abrisse a
    //! timeline *e* escolhesse a porta mostraria duas portas paradas, com todos os dados certos.
    //!
    //! ⚠️ **Com o painel fechado o `keys_mode` é publicado `false`** (a lei do `paint` dele), e é isso
    //! que faz a cutscene correr. ⭐ Quem abrir a timeline para espreitar tem de escolher **Arrange** —
    //! ali ela continua a correr; na **Keys**, não. *Esta é a fronteira da wave, e ela é a mesma que o
    //! animador já conhece: a aba Keys é onde se AUTORA, não onde a cena corre.*
    //!
    //! # O que tem de acontecer
    //!
    //! A porta da **esquerda** sobe, volta ao chão e sobe outra vez, de quatro em quatro segundos. A da
    //! **direita** nunca se mexe. ⚠️ Se a linha `[sequence-smoke]` não aparecer, **PARE**: a cena não
    //! montou.

## `shells/desktop/src/ui_motion_smoke.rs`

    //!
    //! ⚠️ **E o RELÓGIO não se julga aqui.** *A corda dá a mesma forma a 30 e a 120 fps* é propriedade
    //! de gate (`the_rope_is_a_fact_of_the_wall_clock_not_of_the_frame_rate`, medido em **0,0000 px**),
    //! e um olho não distingue 1 px numa corda a balançar. O que o smoke julga é o LOOK: os números de
    //! aparência (`NODES` · `SLACK` · a gravidade de ecrã) saem daqui, como o
    //! `RESAMPLE_STEP_FRACTION` do Flip saiu.
    //!
    //! # `=1` — O CARÁCTER
    //!
    //! Nada é montado na cena: o assunto é o **chrome**, que já está todo na tela. O roteiro pede três
    //! comparações, e a terceira é a que só um segundo arranque responde.
    //!
    //! # `=2` — A CORDA
    //!
    //! Abre o card **Fill adjust** no meio do ecrã. Ele é o caso do Pixelator: um card que **nasceu**
    //! num sítio (a largada do ColorDrop) e que o artista arrasta dali — a corda liga os dois.
    //!
    //! ⚠️ **A abertura é SETUP, o arrasto é o teste.** O card só nasce depois de um ColorDrop real no
    //! Painter, que é meia dúzia de gestos antes do assunto; o que esta wave construiu é a corda, e o
    //! gesto que a exercita é **arrastar o card**. O arrasto é o caminho real e completo (o
    //! `arm_fill_modal_drag_if_on_handle` do shell, sem atalhos).
    //!
    //! # `=3` — A DOBRA
    //!
    //! Abre o painel de **física** e manda dobrar uma secção. A F4a deu o `t` ao CABEÇALHO; a F4b
    //! deu-o ao CORPO, e a cena existe porque a segunda metade tem **quatro** modos de falha com
    //! causas diferentes — o corpo a saltar, o que está por baixo a não subir junto, uma row a
    //! transbordar por cima da secção seguinte, e uma row invisível a continuar a responder ao rato.
    //! Um roteiro que dissesse só *«veja se desliza»* deixaria três deles passar.
    //!
    //! ⚠️ **A cena não monta uma secção — ela abre um painel que já as tem.** É a mesma leitura do
    //! `=1`: o assunto é chrome, e inventar um painel de demonstração pintaria a dobra num sítio que
    //! nenhum artista visita. O painel de física é o escolhido por ser **global** (não pede ferramenta
    //! nem selecção) e por trazer secções de altura desigual, que é onde o deslize se lê.
    //!
    //! ⚠️ **E ela imprime a CONTAGEM que a torna válida**, resolvida do painel (`rows::SECTIONS`) e
    //! medida no store (`collapsible_ids`) — nunca um literal aqui, que seria a segunda cópia de um
    //! número que o painel declara.

## `shells/desktop/src/mount_smoke.rs`

    //!
    //! O [`crate::socket_smoke`] mostra o que uma âncora **é**. Este mostra para que ela **serve**: um
    //! boneco com duas âncoras, e três objetos pendurados nele.
    //!
    //! | objeto | monta em | o que prova |
    //! |---|---|---|
    //! | espada (vermelha) | `hand_r` | segue a mão |
    //! | chapéu (amarelo) | `head` | segue a cabeça |
    //! | **mancha cinzenta** | *nada* | **o controlo** — fica na origem do boneco |
    //! | quadrado verde | `head`, **deslocado** | «Reset to Anchor» tem o que fazer, e a `head` mostra `2 riding` |
    //!
    //! # ⚠️ A mancha cinzenta é metade da cena, e é a metade que se esquece
    //!
    //! Ela é filha do boneco tal como as outras duas e **não monta em âncora nenhuma**. Sem ela, uma
    //! cena em que a montagem estivesse a ser ignorada por completo pareceria exatamente igual — três
    //! filhos a acompanhar o pai é o que filhos já faziam antes deste trabalho existir. *Uma fixtura
    //! sem o controlo mede silêncio*; o que se vê aqui é a **diferença** entre os dois.
    //!
    //! # O gesto
    //!
    //! O boneco nasce selecionado. Com a seção **Sockets / Anchors** aberta:
    //!
    //! 1. Escolher `hand_r` na lista → aparecem as alças no canvas.
    //! 2. **Arrastar a cruz** → a espada vai junto, ao vivo. A mancha cinzenta não se mexe.
    //! 3. **Arrastar o braço** (roda a âncora) → a espada gira em torno da mão.
    //! 4. Selecionar a espada: a §12 mostra **«Rides Parent Anchor: hand_r»**. Pôr «—» larga-a na
    //!    origem do boneco, em cima da mancha cinzenta — que é a prova de que era a âncora a movê-la.
    //!    Voltar a escolher `hand_r` **pousa-a na mão outra vez**, e não onde ela tinha ficado.
    //! 5. Selecionar o **verde**: ele monta na `head` e está fora dela. A §12 diz *«Off anchor by
    //!    35, 30 px»* e oferece **«Reset to Anchor»** — que o põe em cima do chapéu, e desaparece.
    //! 6. Selecionar o **boneco** e marcar **«Always show anchors»**: as cruzes ficam mesmo depois de
    //!    selecionar outra coisa, ou de fechar a seção.
    //!
    //! ⚠️ **A segunda caixa, «Show anchors at runtime», grava e ainda não tem quem a leia** — não há
    //! modo de jogo neste app (`shells/game`, Runtime R1, adiado). Ver
    //! `ph2d_ecs::AnchorVisibility::at_runtime`.
    //!
    //! [ADR-0072]: ../../../docs/architecture/decisions/0072-named-anchor-unification.md

## `shells/desktop/src/hud_smoke.rs`

    //!
    //! | o que está na tela | o que prova |
    //! |---|---|
    //! | os postes e a cerca, que passam ao lado | a câmera segue o herói (a cena do #7) |
    //! | **Pontos: N** no canto de cima | o rótulo lê um [`ph2d_ecs::Counter`] que o relógio soma |
    //! | **N.N s** no canto oposto | um rótulo de [`ph2d_ecs::LabelSource::TimerLeft`], que desce sozinho |
    //! | o botão **+10** | o clique publica um sinal, e a tabela do #5 soma ao MESMO contador |
    //!
    //! # ⚠️ TRÊS quadros, e a razão do terceiro é uma armadilha desta casa
    //!
    //! O `sync` do render loop é que dá **entidade** a um `VecPath` recém-nascido, e sem entidade não
    //! há onde pendurar um `UiLabel`. ⇒ o mundo e as formas nascem num quadro, os componentes no
    //! seguinte. É a mesma razão que o `text_fx_smoke` e o `text_wrap_smoke` já pagaram.
    //!
    //! # ⚠️⚠️ A vista da câmera de jogo é a da JANELA, e o editor pinta num sub-rectângulo
    //!
    //! O `fase_game_camera` calcula o enquadramento com o aspecto da **superfície inteira**
    //! (`camera_2d::aspect_of(surface.size())`), e o editor desenha o mundo na área que sobra **entre
    //! os painéis**. ⇒ no EDITOR, um HUD colado às bordas da caixa de referência cai atrás da régua e
    //! dos painéis; numa janela de jogo (sem painéis) ele assenta exactamente onde foi autorado.
    //!
    //! ⇒ **esta cena põe as quatro peças bem dentro da caixa** (`±5` de `16`, `−5` de `9`) e **fecha o
    //! painel da timeline**, para o que o dono vê ser o que a lei faz. *Uma cena cujo canto está atrás
    //! de um painel ensina que o ancoramento está partido.* As duas distâncias são MEDIDAS numa foto e
    //! a calibração está escrita ao lado de cada uma (ver [`ph2d_app_components::hud_smoke_anchors`] e
    //! o bloco do transporte).
    //!
    //! # ⚠️ As três peças vivem EM BAIXO, e a razão é uma medição
    //!
    //! Os avisos de sinal (*toasts*) empilham-se no **topo** do canvas, e o relógio desta cena publica
    //! um a cada dois segundos — com o placar no canto de cima, a foto mostrou-o **tapado** metade do
    //! tempo. *Uma cena em que o número que se vai ler está escondido não prova nada.*
    //!
    //! # ⛔ O que esta cena NÃO faz
    //!
    //! Ela **não** arma a ferramenta vectorial: um clique aqui é do jogo (o relógio anda), e a cena
    //! tem de abrir no estado em que o botão do HUD responde. *Uma cena que abre com a ferramenta
    //! errada ensina que o botão está partido.*

## `shells/desktop/src/render_loop/color_equalization_bridge.rs`

    //!
    //! Run once per frame BEFORE `paint_hero_screen`. Does, in order:
    //!
    //! 1. Drives the panel's visibility (shown iff `color_equalization`
    //!    is the active tool, keyed `"color_equalization"`).
    //! 2. Publishes the per-frame `ColorEqualizationUiSnapshot` / histogram
    //!    the panel paints next frame.
    //! 3. Drains the panel-staged dropdown close + reset requests.
    //! 4. Maintains a **shell-local cache of original sprite pixels** keyed
    //!    by entity (`SOURCE_CACHE` thread_local). The cache is populated on
    //!    first sight of each newly-selected entity (read once from
    //!    `texture_edit::read_sprite_source`), so subsequent bakes re-feed
    //!    the *original* — they never compound over previously-baked output.
    //! 5. On `params_dirty` (and NOT mid-drag), bakes every selected entity
    //!    from its cached original and writes the result back into the ECS
    //!    sprite via `texture_edit::commit_edited_texture`. No overlay paint.
    //! 6. On Apply, reverts every cached entity to its original, clears the
    //!    cache, and returns the selection to the caller — caller's
    //!    `drain_color_equalization` then re-bakes from the clean source and
    //!    creates the official `ImageEditSnapshot` (undo entry intact).
    //! 7. On Cancel / tool deactivate / selection shrink, reverts the
    //!    relevant cached entities so the sprite snaps back to its original
    //!    look (mirrors the legacy "preview disappears when you leave the
    //!    tool" UX).
    //!
    //! ### Trade-offs
    //!
    //! Cost per slider tick is roughly Apply-grade (full-res pipeline +
    //! texture upload). The `dragging_ceq` gate still throttles this to a
    //! single bake on slider release — same heuristic that protected the
    //! previous overlay rebuild.
    //!
    //! Undo: each Apply creates one entry (revert + caller's drain → undo).
    //! Live-bake ticks that never reach Apply do *not* create entries; the
    //! `SOURCE_CACHE` is the shadow source of truth and Cancel / deactivate
    //! restores from it.

## `shells/desktop/tests/it/a_paralaxe_corre_depois_da_camera_e_recebe_o_rectangulo_inteiro.rs`

    //!
    //! # ⚠️ Metade 1 — a vista é a DESTE quadro
    //!
    //! A `fase_game_camera` calcula o rectângulo da câmera do jogo; um passe de paralaxe ANTES dela
    //! deslocaria contra o enquadramento do quadro ANTERIOR, e num jogo em que a câmera segue o herói
    //! o fundo lia-se como *«atrasado um quadro»* — a mesma classe de defeito que a ordem do HUD paga,
    //! e que esta shell já pagou quatro vezes noutros sítios.
    //!
    //! # ⭐⭐ Metade 2 — a invariância ao zoom, e a PREMISSA que MORREU aqui
    //!
    //! A pesquisa mediu no alvo que o deslocamento de um `Parallax2D` **não depende do zoom nem da
    //! rotação da câmera** — a paralaxe nasce da TRANSLAÇÃO, e uma câmera que só roda não produz
    //! nenhuma ([`docs/Components/23_pesquisa_paralaxe.md`](../../../../docs/Components/23_pesquisa_paralaxe.md)
    //! §4.4).
    //!
    //! ⛔⛔ **A 1.ª redacção deste gate (e o NOME dele, até à auditoria 26) exigia que o rectângulo da câmera NÃO atravessasse a fronteira**,
    //! e ela estava certa enquanto a paralaxe fosse só um deslocamento. A **W3** (o confinamento) tem o
    //! joelho em `(região − ecrã)/2` — *ele precisa de saber quanto a vista mede* — e a premissa morreu
    //! no dia seguinte ao de ter sido escrita.
    //!
    //! ⇒ **as duas perguntas separaram-se, e a lei continua gateada pela mais forte das duas réguas:**
    //!
    //! 1. *o zoom entra no DESLOCAMENTO?* — **não**, e quem o afirma é a **ASSINATURA**:
    //!    `ScrollFactor::deslocamento(centro)` recebe um centro e mais nada, logo não HÁ zoom para ler
    //!    e nenhum gate é preciso. *Uma lei que o compilador proíbe não precisa de uma régua.*
    //! 2. *o zoom entra no CONFINAMENTO?* — **sim**, e é a lei medida; ele tem gate próprio na crate da
    //!    lei (`o_joelho_esta_onde_a_borda_da_vista_alcanca_a_regiao`).
    //!
    //! O que fica aqui é o que só a shell pode afirmar: **a fase entrega o rectângulo INTEIRO**, porque
    //! um centro sozinho tornaria o confinamento inexprimível — o defeito oposto ao que a 1.ª redacção
    //! guardava.
    //!
    //! ⚠️ **A lente é o TEXTO EMENDADO do quadro** (`frame_text::render_frame`), nunca um ficheiro: a
    //! fase que corre primeiro pode morar no ficheiro que vem depois.

## `crates/ph2d-app-components/src/instance_swap_match.rs`

    //!
    //! Este ficheiro é o palpite, **pedido em voz alta**. Ele existe porque o artista às vezes quer
    //! exactamente isso — *«troca este Carro por um Camião e tenta levar o que eu mexi»* — e a única
    //! coisa que o plano proíbe é que aconteça **sozinho**:
    //!
    //! > *«Trocar para mestre NÃO aparentado: só por gesto, com os 3 modos + relatório.
    //! > ⛔ Nunca automático (HR-5).»*
    //!
    //! ⇒ os três modos são **três itens de menu diferentes**, e o caminho de omissão
    //! ([`WhenUnrelated::Refuse`]) é o que todos os chamadores de dentro do app passam.
    //!
    //! # ⚠️ O mapa derivado GANHA sempre
    //!
    //! Quando os dois mestres **são** aparentados, [`crate::instance_variant::piece_map`] responde e
    //! este ficheiro nem chega a correr — mesmo que o artista tenha escolhido *«por nome»*. Um modo de
    //! emparelhamento é a resposta para a **ausência** de uma verdade, nunca uma alternativa a ela.
    //!
    //! # ⭐⭐ A chave é um CAMINHO, e não um nome solto
    //!
    //! A 1.ª redacção emparelhava por `Name` cru, e ela produzia mapas que **partem a árvore**: a peça
    //! `Wheel` que no Carro pende da raiz e no Camião pende da `Cabin` seria emparelhada, e a cópia
    //! ficava com a roda debaixo da raiz enquanto a receita a diz debaixo da cabina —
    //! [`crate::instance_structure::reconcile`] **não muda peças de pai** (ele materializa as que
    //! faltam e apaga as que sobram), então o defeito ficaria estável e mudo.
    //!
    //! ⇒ a chave dos DOIS modos é o **caminho desde a raiz**, e a diferença entre eles é só o degrau:
    //!
    //! | modo | um degrau é | sobrevive a | perde-se com |
    //! |---|---|---|---|
    //! | [`WhenUnrelated::ByName`] | o `Name` da peça | reordenar os irmãos | renomear |
    //! | [`WhenUnrelated::ByHierarchy`] | o índice entre os irmãos | renomear | reordenar |
    //!
    //! São as duas metades do `ObjectMatchMode` do Unity, e a razão de haver duas é que **nenhuma
    //! contém a outra**.
    //!
    //! ⚠️ **E um caminho só emparelha se o PAI dele emparelhar** — a lei que torna o mapa
    //! estruturalmente consistente por construção. Ela morde de facto: com dois irmãos chamados `Arm`
    //! o caminho `Arm` é ambíguo e cai, e sem esta lei o `Arm/Hand` de um deles (que é único!)
    //! emparelharia e a mão ficaria pendurada num pai que não existe daquele lado.
    //!
    //! # ⭐⭐⭐ A RAIZ emparelha SEMPRE — inclusive no modo *«não leves nada»*
    //!
    //! Ela é o **objecto**, e não uma peça: *«não leves nada»* é sobre o que a receita deu. Mas a razão
    //! que fecha a questão é **mecânica, e não de gosto** — só ela explica por que a excepção da raiz
    //! não pode ficar sem imagem:
    //!
    //! > **A chave da raiz não tem sepultador.** [`crate::instance_structure`] enterra a excepção de uma
    //! > peça (`entomb`) **no instante em que a peça morre**, e a raiz de uma instância **nunca morre**
    //! > numa troca — o que muda é o elo dela. ⇒ uma chave de raiz sem imagem no mapa não fica *viva*
    //! > (o passe compara com a peça do mestre NOVO, e a chave aponta para a do velho) nem *sepultada*
    //! > (ninguém a enterra): ela fica **invisível**, a impedir para sempre que a receita nova alcance
    //! > aquele componente da cópia, sem uma linha em lado nenhum do painel a dizê-lo.
    //!
    //! ⚠️ E é a mesma razão pela qual o **eco** do mestre novo é esquecido na troca — o
    //! [`crate::instance_variant::swap`] percorre os valores do mapa, e a raiz estar lá é o que faz o
    //! primeiro passe cair na regra do 1.º encontro em vez de congelar a cópia com o valor do velho.
    //!
    //! ⚠️ **Nada disto move o objecto:** onde ele está, como se chama e em que ordem aparece são da raiz
    //! e **nunca foram do mestre** (a lista `ROOT_IS_ITS_OWN` do [`crate::instance_sync`]), logo não são
    //! excepções e não passam por mapa nenhum.
    //!
    //! # ⛔ O que NÃO se emparelha, e as duas ausências são decisões
    //!
    //! 1. **Uma peça SEM nome, no modo `ByName`** — e com ela a sub-árvore inteira, porque um caminho
    //!    de nomes não atravessa um degrau que não tem nome. Emparelhar dois anónimos *«porque os dois
    //!    são o primeiro filho»* seria o modo `ByHierarchy` a correr com o rótulo do outro.
    //! 2. **O que o ARTISTA pendurou na cópia** — ele não tem elo, logo não é peça de receita nenhuma,
    //!    e o passe estrutural já o deixa em paz. *Só o que a receita deu é que a receita tira.*

## `crates/ph2d-app-components/src/camera_2d_smoke.rs`

    //!
    //! ```text
    //!   Heroi (o quadrado amarelo) ──── SETAS do teclado ────► a câmera segue
    //!   Camera (segue o Heroi)  ·  amortecimento 5  ·  janela morta 0,25
    //!   Cerca (os quatro cantos vermelhos) ─────────► a JANELA pára neles, não o centro
    //!   Postes (de 3 em 3 m) ───────────────────────► é por eles que o movimento se vê
    //! ```
    //!
    //! # ⛔⛔ Porque o herói anda pelo TECLADO e não pelo rato — o report de 2026-09-09
    //!
    //! A primeira redacção desta cena mandava **arrastar** o herói, e o dono devolveu: *«arrastar o
    //! herói provoca umas travadas no movimento dele porque o rato sai de cima do player»*. Ele tem
    //! razão, e não é um defeito da câmera: **é um LAÇO**.
    //!
    //! Um arrasto de canvas ancora o objecto na posição de MUNDO debaixo do cursor, e essa posição é
    //! derivada da câmera. Com uma câmera que segue o objecto, a cadeia fecha-se sobre si mesma:
    //!
    //! ```text
    //!   rato parado → a câmera ainda vem a caminho → o mundo debaixo do cursor MUDA
    //!               → o objecto «move-se» sem ninguém lhe tocar → a câmera segue de novo → …
    //! ```
    //!
    //! Ele converge (o amortecimento é `< 1`), mas o caminho até lá é a trepidação que o dono viu.
    //! ⛔ **Não há afinação que o cure** — a realimentação é da geometria, não de um número.
    //!
    //! ⭐ **A cura é a FONTE DO MOVIMENTO.** Num jogo, o sujeito de uma câmera nunca é arrastado pelo
    //! ponteiro: ele anda por **entrada, em metros por segundo**, e a câmera segue. Trocada a fonte, o
    //! laço deixa de existir — o herói passa a depender só do teclado e a câmera só do herói.
    //! *Uma cena de smoke tem de encenar o que o produto faz, senão ela mede um caminho que ninguém
    //! percorre.*
    //!
    //! ⚠️ **Arrastar o herói continua a funcionar** (é o canvas de sempre) e continua a trepidar
    //! enquanto a pré-visualização estiver ligada. Isso é **declarado**, não um defeito por corrigir:
    //! é o que qualquer editor faz ao arrastar dentro de uma vista que se move sozinha.
    //!
    //! # ⚠️ O que provar
    //!
    //! - **A janela morta:** dar um toque numa seta **não** mexe a câmera. Ela só começa a seguir
    //!   quando o herói passa de ~¼ do ecrã do centro. *É isto que faz um plataforma não enjoar.*
    //! - **O amortecimento:** ao largar a seta, a câmera **assenta** — ela não pára a seco.
    //! - ⭐⭐ **A CERCA prende a JANELA, não o centro:** leve o Heroi para lá dos cantos vermelhos. A
    //!   câmera pára com o canto **na borda do ecrã** — nunca com o canto no meio dele. É a diferença
    //!   que o levantamento nomeia como a entrega do P0, e a que todo jogo escreve à mão.
    //! - ⚠️ **O nascimento não viaja:** ao abrir, a câmera já está no Heroi. Ela não vem da origem.
    //!
    //! ⚠️ **Enquanto a pré-visualização está ligada, a câmera da cena é dona do pan E do zoom** — a
    //! roda e o arrasto de vista ficam por conta dela. É o que uma câmera de jogo é.
    //!
    //! ⏳ **O que esta wave NÃO entrega, e é nomeado:** o botão que liga e desliga a pré-visualização
    //! vive na secção do Inspector, que é a wave seguinte (W3). Aqui ela é ligada pela cena.
    //!
    //! ⚠️ Se a linha `[camera-2d-smoke]` não aparecer, **PARE**: a cena não montou.

## `crates/ph2d-app-sprite-anim/src/inspector.rs:3`

    //!
    //! # ⚠️ A conversão da VELOCIDADE mora aqui, nas duas pontas
    //!
    //! O motor guarda `Q16.16` (`65536` = `1,0×`) porque a multiplicação inteira é bit-idêntica entre
    //! sistemas; o artista escreve `1,5`. O snapshot divide na saída e o commit multiplica na entrada,
    //! **no mesmo ficheiro** — duplicar a conversão faria o número que ele lê e o que o motor aplica
    //! divergirem no dia em que uma metade fosse corrigida. É a mesma lei da §12 com os pixels.

## `crates/ph2d-app-sprite-anim/src/inspector_tests.rs:490`

    /// ⚠️ **Este gate existe porque uma mutação derrubou uma afirmação minha.** O tique corria um laço
    /// de `ticks` chamadas, com a justificação de que um passo grande «atravessaria o fim de um ciclo
    /// sem o fechar» — e a mutação que trocou o laço por uma chamada única **passou**. A lei tem o seu
    /// próprio laço de recuperação, e é ele que fecha cada ciclo.
    ///

## `crates/ph2d-app-sprite-anim/src/inspector_tests.rs:553`

    ///
    /// ⚠️ **A frase que estava aqui apontava para o `ph2d_ecs_dir_label` «a espelhar a
    /// `AnimDirection::label`, com um gate da shell a prender os dois» — e esse gate nunca existiu**
    /// (medido 2026-09-19). A `AnimDirection::label` foi apagada como órfã, e aquele par deixou de ter
    /// duas respostas. ⇒ o paralelo já não se aplica; o que segue vale para ESTA lei, que continua a
    /// existir duas vezes de propósito.
    ///
    /// ⚠️ **E foi a auditoria que o encontrou** (2026-08-23): a `animator_state` estava exportada,
    /// tinha gate próprio no `ph2d-ecs` e **nenhum consumidor** — enquanto o painel reimplementava a
    /// lei sem nada a prendê-los. *Uma segunda resposta sem oráculo é uma divergência com data
    /// marcada.*
    ///

## `crates/ph2d-app-sprite-anim/src/previa.rs:4`

    //!
    //! O gesto só se conhece no bus, e o mundo escrito por ele é igual ao da linha: quem viu o ▶ deixa aqui o objecto
    //! marcado, e o passe do Auto Key tira esses campos do diff enquanto a marca durar. Não é documento.


## `crates/ph2d-i18n/src/component_catalog.rs:3`

    //! ⛔⛔ **Porque ele faltava, e porque nenhum censo o via.** O fecho de 2026-09-16 pôs 26 painéis, a
    //! moldura, a shell e 8 crates de família a falar por [`super::tr`], com 30 censos a defendê-lo. A
    //! `ph2d-component-desc` **não é nenhum deles** — e o texto que ela declara é pintado por uma que
    //! É: a paleta *Add Component* fazia, à letra, `desc.display_name.to_string()` e só depois
    //! concatenava `tr(…)`. ⇒ *um censo cuja crate não é DONA do texto que ela pinta fica verde sobre
    //! texto cru*, e eram **291** palavras.


## `crates/ph2d-i18n/src/component_catalog.rs` — cabeçalho, linhas 18–31 (line/GameEngineFuncional, 10/11, tecto de 700 LOC)

```rust
//!
//! ⚠️ **O dono da chave de um campo é o BLOCO e não o componente**, e isso foi medido: `MARKER` é
//! partilhado por **5** descritores e `TAGS` por **2**. Um `FieldDesc` partilhado só pode carregar
//! UMA chave — derivá-la do componente daria duas chaves para o mesmo byte.
//!
//! ⚠️ **As chaves derivadas são ÚNICAS, e há gate**: os 86 últimos segmentos de `canonical_name` não
//! colidem (`80/80` na 1.ª medição, `86/86` com as formas que faltavam). *Uma derivação sem gate de
//! unicidade cala-se no dia em que duas famílias declararem o mesmo nome.*
//!
//! ⛔⛔ **E TRÊS rótulos só entraram porque o gerador não é a régua léxica:** `"Y-Sort"`, `"9-Slice"`
//! e `"3D"` (este saiu com o 3D, ADR-0179) eram pintados e a [`ph2d_label_census`] **não os
//! acusava** — ela não reconhece uma sigla nem um composto com hífen ou dígito como língua. *Uma
//! régua léxica tem um ponto cego exactamente onde o rótulo é mais curto*, e quem os apanhou foi o
//! cruzamento entre as duas populações.
```
