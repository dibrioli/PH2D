//! **O GESTO** — o que a mão faz com a cena 3D.
//!
//! Filho (`#[path]`) de [`super`] para alcançar os campos privados da
//! [`Sculpt3dScene`]; o corte é o que o cabeçalho do pai já anunciava: *a cena e
//! o passe* lá, *o gesto* aqui. As portas daqui são as que o `input_dispatch`
//! chama, e todas recusam no primeiro `if` sem cena armada — a promessa de
//! removibilidade do `docs/3D/02.3` no nível do frame.

use super::{Dab, Drag, Grip, ORBIT_RAD_PER_PX, Sculpt3dScene, Verb};
use crate::app_state::App;
use std::sync::atomic::{AtomicBool, Ordering};

impl App {
    /// `PH2D_SCULPT3D_SMOKE=1` — a cena pronta: uma esfera de barro para
    /// esculpir. `=2` acrescenta a TELA e é a cena da **doação**
    /// (`crate::sculpt3d::donation`). Roda uma vez, no primeiro frame com GPU.
    /// **O dreno de QUADRO do puxão do Grab.** Ver
    /// [`Sculpt3dScene::pending_grab`] e `flush_pending_grab`: o evento de
    /// ponteiro regista, o quadro carimba, e o pen-up drena o resto.
    ///
    /// ⚠️ **Sem cena aberta é no-op** — e não um `expect`: este é chamado do
    /// laço de quadro incondicionalmente, como os irmãos ao lado dele.
    pub(crate) fn sculpt3d_flush_grab(&mut self) {
        if let Some(scene) = self.sculpt3d_scene_mut() {
            scene.flush_pending_grab();
        }
    }

    pub(crate) fn sculpt3d_smoke(&mut self) {
        // Guard estático, o mesmo idioma dos outros smokes do shell — evita um
        // campo em `App` que só existe para dizer "já rodei".
        static ARMED: AtomicBool = AtomicBool::new(false);
        if !crate::sculpt3d::smoke_armed()
            || self.gfx.is_none()
            || ARMED.swap(true, Ordering::Relaxed)
        {
            return;
        }
        let mesh = crate::sculpt3d::smoke_mesh();
        crate::sculpt3d::announce(&mesh);
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let size = gfx.surface.size();
        let aspect = size.width as f32 / size.height.max(1) as f32;
        let device = std::sync::Arc::clone(&gfx.surface.gpu().device);
        let mut scene = Sculpt3dScene::new(&device, mesh, aspect);
        // ⚠️ **A cena `=7` é montada DEPOIS do `new`, pela porta pública.** Um
        // construtor que recebesse a lista inteira seria a segunda resposta a
        // *"como um objeto entra na cena"* — e a primeira é a que o gesto de
        // blocagem vai usar.
        let extra = crate::sculpt3d::scene_objects();
        if !extra.is_empty() {
            for (mesh, pose) in extra {
                scene.push_object(mesh, pose);
            }
            // O enquadramento tem de conhecer a cena INTEIRA: o `new` enquadrou
            // só a primeira peça, e as outras nasceram fora do quadro.
            scene.frame_all(aspect);
        }
        gfx.sculpt3d = Some(scene);
    }

    /// O botão apertou. Devolve `true` se a cena 3D tomou o gesto.
    pub(crate) fn sculpt3d_pointer_down(&mut self, button: winit::event::MouseButton) -> bool {
        let pos = self.last_pointer;
        // ⚠️ Um clique SOBRE A MOLDURA não é da cena. A pergunta era *"está sobre
        // um PAINEL?"*, e painel é só uma espécie de UI: a faixa do topo e o rail
        // não publicam `panel_rect`, então com o barro na tela a cena engolia o
        // clique em TODO pill do topo — inclusive no que existe para SAIR daqui
        // (Enio, 2026-08-09: *"a pill entra mas não sai do modo sculpt"*). A porta
        // nova cobre painéis E os fundos que a moldura pinta.
        //
        // O `Move` e o `Up` NÃO a fazem de propósito: um arrasto em curso continua
        // sendo do gesto que o abriu, mesmo que o cursor passeie por cima de um
        // painel (a regra de captura que todo gizmo deste shell segue).
        if crate::chrome_hit::pointer_over_chrome(self.gfx.as_ref(), pos.0, pos.1) {
            return false;
        }
        let mods = self.modifiers;
        let (ctrl, shift) = (mods.control_key(), mods.shift_key());
        let Some(scene) = self.sculpt3d_scene_mut() else {
            return false;
        };
        // ⚠️ **Com o barro fora da tela, o ponteiro NÃO é da cena.** Sem esta
        // pergunta a doação seria inalcançável pelo motivo mais bobo possível:
        // o artista troca para o modo LUZ, vai pintar, e cada clique orbita um
        // modelo invisível. É a mesma classe do clique-sobre-painel que o smoke
        // da W2 pegou — quem não está na tela não recebe o gesto.
        if !scene.shows_clay() {
            return false;
        }
        // ⚠️⚠️ **UMA CENA VAZIA NÃO RECEBE GESTO** — e sem esta pergunta o app CRASHA.
        //
        // Enio, 2026-08-22, a esculpir depois de apagar a única peça:
        // ```text
        // [sculpt3d] APAGOU: sobram 0 pecas -- Ctrl+Z a devolve INTEIRA
        // PH2D PANIC ... sculpt3d_input.rs:173 "index out of bounds: the len is 0 but the index is 0"
        // ```
        //
        // ⭐ **A cena vazia é um estado LEGÍTIMO** — o próprio `delete_active` a produz e promete o
        // Ctrl+Z de volta. O que faltava era a outra metade: os caminhos de gesto indexam
        // `objects[active]` **direto**, e com a lista vazia o `active` (que o delete prende em 0)
        // aponta para nada. *Um estado que o módulo declara legal e um caminho que o supõe
        // impossível é um pânico à espera do primeiro clique.*
        //
        // ⛔ **A cura completa é MAIOR do que este sítio, e é da `line/sculpt3d`:** medido
        // 2026-08-22, há **42** indexações `objects[…]` sem guarda em 9 arquivos deste módulo
        // (`filter`, `dyntopo`, `pull`, `transform`, `input`, `space`, `objects`, `import`), e a
        // porta segura que elas deviam usar (`obj()` / `obj_mut()`) **já existe**. A `line/3DModeling`
        // fecha aqui a porta que o artista bateu e nomeia o resto no handoff — reescrever 42 sítios
        // de um módulo alheio não é dela.
        //
        // ⚠️ E a recusa é **reportada**, que é a lei que este módulo já segue no `Delete`: um gesto
        // que não faz nada e não diz nada é indistinguível de um app partido.
        if scene.objects.is_empty() {
            eprintln!("[sculpt3d] a cena esta' VAZIA -- nao ha' o que esculpir (Ctrl+Z devolve)");
            return false;
        }
        match button {
            winit::event::MouseButton::Left => {
                // ⚠️ **Com o transform ARMADO o esquerdo transforma.** Não há
                // fallback para órbita aqui, e é deliberado: o arm é um estado
                // que o painel MOSTRA, e um botão que às vezes transforma e às
                // vezes gira a câmera — conforme o que estava sob o cursor —
                // seria o mesmo gesto significando duas coisas. A órbita
                // continua inteira no botão direito.
                // ⚠️ **Com o FILTRO armado o esquerdo filtra**, pelo mesmo
                // argumento do transform logo abaixo — e os dois nunca estão
                // armados juntos (as portas de armar se excluem). A ordem aqui
                // não escolhe um vencedor: ela é a rede que torna a exclusão
                // observável se algum dia falhar.
                if scene.filter_arm() {
                    // Mirar antes de começar, pelo motivo dos dois vizinhos: o
                    // `begin_filter` congela a foto da malha ATIVA.
                    scene.aim(pos.0, pos.1);
                    if scene.begin_filter(pos.0) {
                        scene.drag = Some(Drag::Filter);
                    } else {
                        // ⚠️ **Este ramo é uma CONTRADIÇÃO, não um caso.** A
                        // única recusa do `begin_filter` é o arm apagado, e ele
                        // está aceso três linhas acima — ao contrário do
                        // transform logo abaixo, cuja recusa (peça toda
                        // mascarada) é um estado real que o artista alcança.
                        //
                        // ⚠️ A mensagem que vivia aqui nomeava QUATRO verbos e
                        // mentia desde a W9b: o picker desacoplou a lei do
                        // verbo em mãos, então *"o verbo em mãos não filtra"*
                        // deixou de ser uma frase verdadeira sobre este app —
                        // e ela nunca teve como ser impressa para alguém a
                        // desmentir. A rede de release fica (o gesto vira
                        // órbita em vez de um botão morto); quem grita é o
                        // debug.
                        debug_assert!(
                            false,
                            "o begin_filter recusou com o arm ACESO: a exclusão dos dois arms \
                             deixou de valer, ou ele ganhou uma segunda recusa sem chamador"
                        );
                        scene.drag = Some(Drag::Orbit);
                    }
                    scene.last = pos;
                    return true;
                }
                if scene.transform_arm().is_some() {
                    // ⚠️ **MIRAR VEM ANTES DE COMEÇAR**, a mesma ordem (e o
                    // mesmo motivo) do traço logo abaixo: o `begin_transform`
                    // congela a foto da malha ATIVA, e mirar depois faria a
                    // sessão descrever a peça anterior.
                    scene.aim(pos.0, pos.1);
                    if scene.begin_transform(pos.0, pos.1) {
                        scene.drag = Some(Drag::Transform);
                    } else {
                        // ⚠️ A recusa é REPORTADA: uma malha inteiramente
                        // mascarada não tem o que mover, e um gesto que não faz
                        // nada e não diz nada é indistinguível de um botão que
                        // não chegou.
                        eprintln!(
                            "[sculpt3d] transform: a peca esta' toda PROTEGIDA -- nao ha' o que mover (I inverte a mascara)"
                        );
                        scene.drag = Some(Drag::Orbit);
                    }
                    scene.last = pos;
                    return true;
                }
                // ⚠️ Os modificadores são lidos UMA vez, no pen-down, e valem o
                // traço inteiro. Soltar o Shift no meio de uma pincelada faria
                // metade dela ser outra ferramenta — e nenhum app de escultura
                // faz isso, porque a lei do traço congela um `pre` só.
                scene.brush.invert = ctrl;
                let verb = scene.brush.verb;
                if shift {
                    scene.brush.verb = Verb::Smooth;
                }
                // ⚠️ **MIRAR VEM ANTES DE COMEÇAR**, e a ordem é a wave inteira:
                // o `begin` dimensiona os planos por-vértice na malha ATIVA, e
                // se a peça sob o cursor for outra o traço passa a escrever
                // índices de uma malha noutra. Com a peça nova maior que a
                // velha, isso é um pânico no primeiro dab.
                scene.aim(pos.0, pos.1);
                scene.stroke.begin(scene.objects[scene.active].stack.mesh());
                // ⚠️ **Depois do `aim`**: a foto é da peça que este traço vai
                // esculpir, e antes do `aim` ela seria a da peça anterior.
                scene.open_dyntopo_stroke();
                // A âncora do espaçamento nasce no pen-down: o 1º dab é o que
                // está sob o dedo, e o resíduo passa a contar a partir dele.
                scene.stroke_anchor = [pos.0, pos.1];
                scene.grab = None;
                // ⚠️ **E o pendente morre com o gesto anterior.** O pen-up
                // drena, então em regime ele já está vazio — mas um arrasto que
                // termine por outra porta (a cena fechada, o botão trocado)
                // deixaria um puxão órfão para carimbar dentro do traço
                // SEGUINTE, com a âncora nova. É a mesma razão do `twist`
                // logo abaixo.
                scene.pending_grab = None;
                // O ângulo varrido é do GESTO, então ele morre com o gesto
                // anterior — deixá-lo vivo faria o traço seguinte começar já
                // torcido, no lugar onde o anterior parou.
                scene.twist = None;
                // ⚠️ **Quem tem ÂNCORA não carimba no pen-down: ele PEGA.** O
                // primeiro toque escolhe o ponto e não move nada; o barro vem
                // quando o dedo anda. Vale para os TRÊS grips com âncora — o
                // Grab porque o puxão ainda é zero, o Snake Hook porque o
                // incremento ainda é zero, o Twist e o Local Scale porque o
                // ângulo e a fração ainda são zero —, e é por isso que a
                // pergunta é `anchors()` e não o nome de um verbo.
                let took = if scene.brush.verb.anchors() {
                    scene.take_hold(pos.0, pos.1)
                } else {
                    scene.sculpt_at(pos.0, pos.1)
                };
                if took {
                    scene.drag = Some(Drag::Sculpt);
                } else {
                    // Errou o modelo: o botão vira ÓRBITA. É o que o SculptGL
                    // faz, e é o que impede o gesto mais comum do mundo —
                    // arrastar no vazio — de não fazer nada.
                    scene.brush.verb = verb;
                    scene.drag = Some(Drag::Orbit);
                }
            }
            winit::event::MouseButton::Right => scene.drag = Some(Drag::Orbit),
            winit::event::MouseButton::Middle => scene.drag = Some(Drag::Pan),
            _ => return false,
        }
        scene.last = pos;
        true
    }

    /// O botão soltou.
    pub(crate) fn sculpt3d_pointer_up(&mut self) -> bool {
        let Some(scene) = self.sculpt3d_scene_mut() else {
            return false;
        };
        let was = scene.drag.take();
        if was == Some(Drag::Sculpt) {
            // ⚠️ **ANTES do fecho, e sem isto o gesto perde a ponta.** O último
            // movimento do dedo chega como evento e fica pendente; se o traço
            // fechasse primeiro, o barro pararia onde o último QUADRO o deixou
            // — um erro que cresce com a velocidade da mão e some quando ela é
            // lenta, que é a forma mais cara de um bug se esconder.
            scene.flush_pending_grab();
            scene.close_stroke();
        }
        if was == Some(Drag::Transform) {
            scene.close_transform();
        }
        // ⚠️ **O filtro fecha pela porta do TRAÇO**, e não por uma sua: o
        // `filter_begin` preenche os mesmos dois arrays que o `close_stroke`
        // grava. Ver o cabeçalho do `sculpt3d_filter`.
        if was == Some(Drag::Filter) {
            scene.close_stroke();
        }
        was.is_some()
    }

    /// O ponteiro moveu. Só consome com um arrasto EM CURSO — senão a cena 3D
    /// engoliria todo hover do app.
    pub(crate) fn sculpt3d_pointer_move(&mut self, x: f32, y: f32) -> bool {
        let Some(scene) = self.sculpt3d_scene_mut() else {
            return false;
        };
        let Some(drag) = scene.drag else {
            return false;
        };
        let (dx, dy) = (x - scene.last.0, y - scene.last.1);
        scene.last = (x, y);
        let height = scene.viewport.1.max(1) as f32;
        match drag {
            // ⚠️ **Manipulação direta: o modelo segue a mão.** `yaw` positivo
            // leva o OLHO para `+X`, e a câmera indo para a direita faz o
            // modelo *parecer* ir para a esquerda — então arrastar para a
            // direita pede `yaw -= dx`. E arrastar para BAIXO mostra o TOPO
            // (o modelo tomba para a frente), que é `pitch += dy`.
            //
            // Os DOIS sinais estavam trocados e o smoke os pegou; o gate que os
            // prende (`dragging_right_turns_the_model_right`) mede o modelo NA
            // TELA em vez de argumentar sobre sinais, que foi como o erro entrou.
            Drag::Orbit => scene
                .camera
                .orbit(-dx * ORBIT_RAD_PER_PX, dy * ORBIT_RAD_PER_PX),
            Drag::Pan => scene.camera.pan(dx / height, dy / height),
            // ⚠️ **Ele NÃO percorre o caminho, e não é o motivo do Grab:** o
            // gesto do transform não é uma trilha nem um ângulo, é o vetor
            // INTEIRO do pen-down até aqui — o `x`/`y` cru, e nunca o `dx`/`dy`
            // do evento. Interpolar entre eventos daria o mesmo total em N
            // parcelas, e a lei já é função do total.
            Drag::Transform => scene.transform_at(x, y),
            // ⚠️ **O `x` CRU pelo mesmo motivo do transform**: a força é o
            // arrasto TOTAL desde o pen-down, e a lei já é função do total.
            Drag::Filter => scene.filter_at(x),
            // ⚠️ **Um evento de ponteiro NÃO é um dab.** O caminho entre a
            // âncora e o cursor é percorrido a passos de
            // [`ph2d_sculpt3d::min_spacing`], senão um gesto rápido deixa um vão
            // do tamanho do salto do mouse e um gesto lento carimba dez vezes
            // mais pelo mesmo caminho.
            //
            // ⚠️ **Cada passo RE-PICA, e um passo que erra a malha PARA o
            // gesto** (`SculptBase.js:161` devolve `pick1 || pick2` e o laço
            // usa isso para `break`). Interpolar em MUNDO entre dois acertos
            // seria outro algoritmo: ele carimbaria através do vão onde a
            // superfície não está — exatamente onde o original desiste.
            // ⚠️ **Um `match` exaustivo sobre o [`Grip`], e não uma cascata de
            // predicados.** Os três ramos abaixo respondem *o que este verbo faz
            // com o caminho*, que é exatamente a pergunta que o grip nomeia — um
            // quarto grip não compila até dizer o que significa aqui, em vez de
            // cair no `else` do último `if` e nascer se comportando como um
            // carimbo.
            Drag::Sculpt => match scene.brush.verb.grip() {
                // Quem SEGURA não percorre o caminho: o espaçamento existe para
                // não deixar buracos entre dois carimbos, e um Grab não carimba
                // — o "caminho" dele é o vetor do pen-down até aqui. Rodar o
                // walk daria N dabs idênticos no mesmo lugar.
                // ⚠️ **REGISTA, não carimba** — ver [`Sculpt3dScene::pending_grab`].
                // O alvo do `Hold` é função do `pre` congelado e do puxão TOTAL,
                // então dabs intermediários são o mesmo trabalho no mesmo lugar
                // (byte-idêntico, medido). Quem os drena é o quadro.
                Grip::Hold => scene.pending_grab = Some((x, y)),
                // ⚠️ **Quem ARRASTA percorre, e é o walk que torna o espinho um
                // fato do CAMINHO.** A lei do Hook é uma soma sobre a lista de
                // dabs; sem o passo fixo na geometria, essa soma passaria a
                // depender da taxa de polling — arrastar devagar esticaria mais
                // que arrastar rápido pelo mesmo traçado. Com ele, o número de
                // parcelas é função do comprimento percorrido.
                Grip::Hook => {
                    let spacing = ph2d_sculpt3d::min_spacing(scene.radius_px());
                    if let Some(steps) = ph2d_sculpt3d::walk(scene.stroke_anchor, [x, y], spacing) {
                        let mut prev = scene.stroke_anchor;
                        for step in steps {
                            scene.hook_step(prev, step);
                            prev = step;
                        }
                        scene.stroke_anchor = steps.anchor();
                    }
                }
                // ⚠️ **Quem GIRA não percorre o caminho tampouco, e por um
                // motivo mais forte que o do Grab: o "caminho" dele não é uma
                // trilha, é um ÂNGULO.** Rodar o walk sobre a varredura daria N
                // dabs com o mesmo total acumulado no mesmo lugar — trabalho
                // idêntico repetido, porque o alvo do [`Grip::Turn`] é função do
                // `pre` congelado e do gesto TOTAL.
                Grip::Turn(kind) => scene.turn_at(kind, x, y),
                // ⚠️⚠️ **QUEM SIMULA PERCORRE, e é o `walk` que o torna honesto.**
                // O tecido é conduzido pela VIAGEM da mão — cada passo entrega o
                // deslocamento desde o anterior —, e sem o passo fixo na geometria
                // a lei passaria a depender da taxa de polling: arrastar devagar
                // faria mais pregas que arrastar rápido pelo mesmo traçado. É a
                // lei que este módulo pagou seis vezes (*o traço é fato do
                // CAMINHO*), e aqui ela não é uma escolha de estilo — é o que
                // impede o mesmo gesto de dar dois panos diferentes.
                Grip::Simulate => {
                    let spacing = ph2d_sculpt3d::min_spacing(scene.radius_px());
                    if let Some(steps) = ph2d_sculpt3d::walk(scene.stroke_anchor, [x, y], spacing) {
                        let mut prev = scene.stroke_anchor;
                        // ⚠️ Os modos de FORÇA da lei da referência re-picam o
                        // cursor na superfície a cada passo (`cloth_step`); os
                        // de âncora — e a lei VBD de omissão — andam no plano
                        // de profundidade do pen-down, pela porta de sempre.
                        let repica = ph2d_sculpt3d::cloth_repica();
                        for step in steps {
                            if repica {
                                scene.cloth_step(prev, step);
                            } else {
                                scene.hook_step(prev, step);
                            }
                            prev = step;
                        }
                        scene.stroke_anchor = steps.anchor();
                    }
                }
                // ⚠️ **O canal PERCORRE o caminho como o carimbo**, e o ramo é
                // partilhado de propósito: o [`Grip::Paint`] nasceu para o
                // carimbo poder trocar de lei sem levar a máscara junto (ver o
                // doc dele), e essa troca é sobre o que um dab FAZ com o que já
                // está lá — não sobre como o gesto vira uma lista de dabs.
                // Esfregar uma máscara é esfregar, e o `walk` é o que impede a
                // taxa de polling de decidir a densidade dela.
                Grip::Stamp | Grip::Paint => {
                    let spacing = ph2d_sculpt3d::min_spacing(scene.radius_px());
                    if let Some(steps) = ph2d_sculpt3d::walk(scene.stroke_anchor, [x, y], spacing) {
                        // Lido ANTES do laço: o `for` consome o iterador, e o
                        // `anchor()` responde onde o walk PARA — que é o fato
                        // que a âncora precisa, tenha o dab pousado ou não.
                        let steps_anchor = steps.anchor();
                        // ⚠️ **A ÂNCORA AVANÇA MESMO QUANDO O DAB É DESCARTADO,
                        // e as DUAS referências concordam nisso.** O Blender
                        // escreve `last_mouse_position = mval`
                        // (`paint_stroke.cc:509`) **ANTES** do teste de acerto
                        // (`:536-538`), então o passo é dado e só a aplicação é
                        // suprimida; o SculptGL faz o mesmo pela outra ponta
                        // (`SculptBase.js:151-152`, `_lastMouse = mouse`, que
                        // descarta o resíduo inteiro). Nenhuma das duas deixa a
                        // âncora para trás.
                        //
                        // ⚠️ **E o artefato que isso previne está NOMEADO na
                        // fonte:** com a âncora presa no último dab APLICADO, um
                        // trecho fora da malha faz o `length` acumular, e ao
                        // reencontrar a superfície o walk despeja a lacuna
                        // inteira de uma vez sobre o ponto de reentrada —
                        // cavando um buraco. Medido (`measure_anchor_law`, o
                        // MESMO caminho em 3 eventos): **61 dabs contra 31**, o
                        // dobro, e a rajada cresce com a velocidade da mão.
                        //
                        // Eu escrevi a âncora-no-último-aplicado em 2026-08-16
                        // lendo o `break` do SculptGL como *"metade de um par"*.
                        // O par existe — mas a outra metade dele é a âncora
                        // AVANÇANDO, não ficando. *Tomar a metade que falta pelo
                        // seu oposto é como uma correção fiel à referência sai
                        // ao contrário dela.*
                        //
                        // ⚠️ **A minha primeira sonda deu a rajada como RUÍDO
                        // (2 passos) porque a fixture não continha o fenómeno:**
                        // 60 eventos densos andam ~1 passo cada, então a âncora
                        // atrasada nunca fica longe. O gesto que separa as leis
                        // é o mouse a SALTAR.
                        for [sx, sy] in steps {
                            if !scene.sculpt_at(sx, sy) {
                                break;
                            }
                        }
                        // ⚠️ **`steps.anchor()` e nunca `[x, y]`:** o resíduo
                        // ACIMA de um passo evaporaria (a dependência de
                        // amostragem que a `measure_path_invariance` mede em
                        // `6,485 % → 0,000 %`), e é ele que faz um traço lento
                        // depositar a mesma densidade que um rápido. É também o
                        // que o Blender faz — lá a âncora caminha em passos
                        // exatos de um espaçamento e para no último
                        // (`paint_stroke.cc:822` + `:509`), nunca no ponteiro.
                        //
                        // Se o `walk` RECUSOU (o carry, `None`), a âncora fica
                        // onde está — o resíduo acumula até valer um passo, e
                        // movê-la fora deste ramo o apagaria.
                        scene.stroke_anchor = steps_anchor;
                    }
                }
            },
        }
        true
    }

    /// A roda aproxima.
    pub(crate) fn sculpt3d_wheel(&mut self, steps: f32) -> bool {
        // A mesma lei do `pointer_down`: a moldura do app não é da cena. O
        // despachante já pergunta pelo PAINEL antes de chamar aqui, e a metade
        // que ele não faz é a dos fundos de chrome — mas a pergunta é feita
        // INTEIRA e neste arquivo de propósito: quem decide de quem é o gesto é
        // o módulo da cena, não o roteador. Sem isto, rolar sobre a barra do topo
        // dá DOLLY na escultura por baixo, em silêncio.
        let pos = self.last_pointer;
        if crate::chrome_hit::pointer_over_chrome(self.gfx.as_ref(), pos.0, pos.1) {
            return false;
        }
        let Some(scene) = self.sculpt3d_scene_mut() else {
            return false;
        };
        // Mesma lei do `pointer_down`: barro fora da tela, roda do 2D. Sem isto
        // o zoom do canvas ficaria preso enquanto a forma acende a tinta.
        if !scene.shows_clay() {
            return false;
        }
        scene.camera.dolly(steps);
        true
    }

    pub(super) fn sculpt3d_scene_mut(&mut self) -> Option<&mut Sculpt3dScene> {
        self.gfx.as_mut()?.sculpt3d.as_mut()
    }
}

impl Sculpt3dScene {
    /// Aplica um dab onde o cursor aponta. Devolve `false` se o raio errou a
    /// malha — e errar é normal: a mão sai do modelo o tempo todo.
    pub(super) fn sculpt_at(&mut self, x: f32, y: f32) -> bool {
        // Na peça ATIVA — quem a escolheu foi o `aim` do pen-down. Ver o doc
        // dele: um traço pertence a uma peça, e trocar no meio é um pânico.
        let Some(hit) = self.pick_active(x, y) else {
            return false;
        };
        let ray = self.ray_at(x, y);
        if std::env::var("PH2D_SCULPT3D_DIAG").ok().as_deref() == Some("1") {
            // ⚠️ **O instrumento que responde *"o pincel cai onde o cursor
            // aponta?"* com um NÚMERO.** Ele reprojeta o acerto pela porta
            // `project` — o inverso exato do `ray_through` — e imprime o erro em
            // pixels. Um desvio grande acusa a fiação (viewport, escala, um
            // flip); zero acusa a percepção, e aí a causa é outra.
            let back = self
                .camera
                .project(self.pose().point_to_world(hit.point), self.viewport);
            let err = back.map(|(bx, by)| ((bx - x).hypot(by - y), bx, by));
            eprintln!(
                "[sculpt3d] clique ({x:.1}, {y:.1}) viewport {:?} -> acerto {:?} \
                 -> volta {err:?}",
                self.viewport, hit.point
            );
        }
        let brush = self.armed_brush(hit.point);
        // ⚠️ **REFINA E DEPOIS CARIMBA** — ver `refine_for_dab`. E a malha que a
        // linha seguinte recebe pode ter mais vértices que a do `pick_active`
        // acima: é por isso que ela é pedida de novo, por índice, em vez de
        // segurada numa referência desde o topo.
        self.refine_for_dab(hit.point, brush.radius);
        let eye = self.dir_to_local(ray.dir());
        self.stroke.dab(
            self.objects[self.active].stack.mesh_mut(),
            &brush,
            // ⚠️ **O olho é o `dir` do raio que ACABOU de produzir este acerto**,
            // e não uma direção derivada da câmera de novo: duas respostas para
            // *"de onde se está olhando"* divergem no frame em que a câmera se
            // move entre o pick e o dab.
            &Dab::at(hit.point, brush.radius, eye),
            self.symmetry,
        );
        Self::mesh_changed(
            &mut self.objects[self.active].dirty,
            &mut self.edits,
            // ⚠️ **`last_gpu_dirty`, não `last_refreshed`.** Um traço de máscara
            // não move geometria, então ele não refresca normal nenhuma — e
            // perguntar *"o que refresquei?"* devolveria VAZIO, deixando a
            // máscara invisível na GPU com todos os gates de CPU verdes.
            self.stroke.last_gpu_dirty(),
        );
        true
    }
}
