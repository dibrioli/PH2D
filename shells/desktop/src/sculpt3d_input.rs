//! **O GESTO** — o que a mão faz com a cena 3D.
//!
//! Filho (`#[path]`) de [`super`] para alcançar os campos privados da
//! [`Sculpt3dScene`]; o corte é o que o cabeçalho do pai já anunciava: *a cena e
//! o passe* lá, *o gesto* aqui. As portas daqui são as que o `input_dispatch`
//! chama, e todas recusam no primeiro `if` sem cena armada — a promessa de
//! removibilidade do `docs/3D/02.3` no nível do frame.

use super::{
    Drag, Grip, LIGHT_STEP_DEG, MaskOp, ORBIT_RAD_PER_PX, RADIUS_STEP, Sculpt3dScene, Verb,
};
use crate::app_state::App;
use std::sync::atomic::{AtomicBool, Ordering};

impl App {
    /// `PH2D_SCULPT3D_SMOKE=1` — a cena pronta: uma esfera de barro para
    /// esculpir. `=2` acrescenta a TELA e é a cena da **doação**
    /// (`crate::sculpt3d::donation`). Roda uma vez, no primeiro frame com GPU.
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
        gfx.sculpt3d = Some(Sculpt3dScene::new(&device, mesh, aspect));
    }

    /// O botão apertou. Devolve `true` se a cena 3D tomou o gesto.
    pub(crate) fn sculpt3d_pointer_down(&mut self, button: winit::event::MouseButton) -> bool {
        let pos = self.last_pointer;
        // ⚠️ Um clique SOBRE PAINEL não é da cena — e sem esta pergunta a cena 3D
        // engolia todo botão do app, inclusive os do rail. O `Move` e o `Up` NÃO
        // a fazem de propósito: um arrasto em curso continua sendo do gesto que
        // o abriu, mesmo que o cursor passeie por cima de um painel (a regra de
        // captura que todo gizmo deste shell segue). É a MESMA porta que a roda
        // já usava.
        if crate::forwarding::cursor_over_hero_panel(self.gfx.as_ref(), pos.0, pos.1) {
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
        match button {
            winit::event::MouseButton::Left => {
                // ⚠️ Os modificadores são lidos UMA vez, no pen-down, e valem o
                // traço inteiro. Soltar o Shift no meio de uma pincelada faria
                // metade dela ser outra ferramenta — e nenhum app de escultura
                // faz isso, porque a lei do traço congela um `pre` só.
                scene.brush.invert = ctrl;
                let verb = scene.brush.verb;
                if shift {
                    scene.brush.verb = Verb::Smooth;
                }
                scene.stroke.begin(scene.stack.mesh());
                // A âncora do espaçamento nasce no pen-down: o 1º dab é o que
                // está sob o dedo, e o resíduo passa a contar a partir dele.
                scene.stroke_anchor = [pos.0, pos.1];
                scene.grab = None;
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
                Grip::Hold => scene.grab_at(x, y),
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
                        scene.stroke_anchor = [x, y];
                    }
                }
                // ⚠️ **Quem GIRA não percorre o caminho tampouco, e por um
                // motivo mais forte que o do Grab: o "caminho" dele não é uma
                // trilha, é um ÂNGULO.** Rodar o walk sobre a varredura daria N
                // dabs com o mesmo total acumulado no mesmo lugar — trabalho
                // idêntico repetido, porque o alvo do [`Grip::Turn`] é função do
                // `pre` congelado e do gesto TOTAL.
                Grip::Turn(kind) => scene.turn_at(kind, x, y),
                Grip::Stamp => {
                    let spacing = ph2d_sculpt3d::min_spacing(scene.radius_px());
                    if let Some(steps) = ph2d_sculpt3d::walk(scene.stroke_anchor, [x, y], spacing) {
                        for [sx, sy] in steps {
                            if !scene.sculpt_at(sx, sy) {
                                break;
                            }
                        }
                        // Só aqui: se o `walk` recusou, a âncora FICA e o
                        // resíduo acumula — é o carry, e movê-la fora deste ramo
                        // o apaga.
                        scene.stroke_anchor = [x, y];
                    }
                }
            },
        }
        true
    }

    /// A roda aproxima.
    pub(crate) fn sculpt3d_wheel(&mut self, steps: f32) -> bool {
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

    /// As teclas da cena 3D. Devolve `true` se consumiu.
    pub(crate) fn sculpt3d_key(
        &mut self,
        code: winit::keyboard::KeyCode,
        ctrl: bool,
        shift: bool,
    ) -> bool {
        use winit::keyboard::KeyCode as K;
        let Some(scene) = self.sculpt3d_scene_mut() else {
            return false;
        };
        if ctrl {
            if code != K::KeyZ {
                return false;
            }
            // ⚠️ **O `shift` é o que separa desfazer de refazer, e enquanto ele
            // não chegava aqui o atalho de REFAZER desfazia mais um passo** —
            // a forma de *"o redo não funciona"* que **destrói** trabalho em vez
            // de não fazer nada. Ele é o mesmo par do resto do app (Ctrl+Z /
            // Ctrl+Shift+Z): um terceiro atalho só para esta cena seria uma
            // segunda gramática a aprender.
            return if shift {
                scene.redo_stroke()
            } else {
                scene.undo_stroke()
            };
        }
        // Os dez primeiros verbos por número; o Mask fica no `M` porque ele não
        // é uma escultura a mais, é o canal que todos os outros respeitam.
        const BY_NUMBER: [Verb; 10] = [
            Verb::Draw,
            Verb::Inflate,
            Verb::Smooth,
            Verb::Sharpen,
            Verb::Flatten,
            Verb::Fill,
            Verb::Scrape,
            Verb::Clay,
            Verb::Pinch,
            Verb::Crease,
        ];
        // ⚠️ O **Move** fica no `G` (de *grab*) e não num número: os dez números
        // já estão tomados, e `G` é a tecla que Blender e SculptGL usam para o
        // mesmo gesto — um artista a tenta antes de procurar.
        // ⚠️ O **Move** fica no `G` (de *grab*), o **Snake Hook** no `H` (de
        // *hook*), o **Twist** no `T` e o **Local Scale** no `S` (de *scale*):
        // os dez números já estão tomados, e o `G` é a tecla que Blender e
        // SculptGL usam para o mesmo gesto — um artista a tenta antes de
        // procurar. Os quatro saem pela MESMA porta que os numerados usam,
        // senão só eles perderiam o default de força.
        // ⚠️ **E o `A` é o MAGNIFY, que não tinha tecla nenhuma.** Onze verbos
        // de carimbo queriam dez dígitos, e ele foi o que transbordou — sem uma
        // linha dizendo isso: existia no enum, tinha alvo, era varrido por todo
        // gate, e o artista **não conseguia pegá-lo**. Não era cerca de
        // Chesterton, era capacidade. O `A` é de *amplify*, a mesma família da
        // palavra que o rótulo mostra; o `P`, que seria o mnemônico do PAR,
        // levaria a mão ao oposto do que ela procura.
        let held = match code {
            K::KeyG => Some(Verb::Move),
            K::KeyH => Some(Verb::SnakeHook),
            K::KeyT => Some(Verb::Twist),
            K::KeyS => Some(Verb::LocalScale),
            K::KeyA => Some(Verb::Magnify),
            _ => None,
        };
        let verb = held.or(match code {
            K::Digit1 => Some(BY_NUMBER[0]),
            K::Digit2 => Some(BY_NUMBER[1]),
            K::Digit3 => Some(BY_NUMBER[2]),
            K::Digit4 => Some(BY_NUMBER[3]),
            K::Digit5 => Some(BY_NUMBER[4]),
            K::Digit6 => Some(BY_NUMBER[5]),
            K::Digit7 => Some(BY_NUMBER[6]),
            K::Digit8 => Some(BY_NUMBER[7]),
            K::Digit9 => Some(BY_NUMBER[8]),
            K::Digit0 => Some(BY_NUMBER[9]),
            K::KeyM => Some(Verb::Mask),
            _ => None,
        });
        // As QUATRO operações de máscara. ⚠️ Elas não são verbos: um verbo pinta
        // *onde a mão passou* e estas respondem a *o que já está pintado*, então
        // elas não podem entrar na lista de números (escolher uma não é pegar
        // uma ferramenta — é executar um gesto e acabar).
        let mask_op = match code {
            K::KeyC => Some(MaskOp::Clear),
            K::KeyI => Some(MaskOp::Invert),
            K::KeyB => Some(MaskOp::Blur),
            K::KeyN => Some(MaskOp::Sharpen),
            _ => None,
        };
        if let Some(op) = mask_op {
            scene.mask_op(op);
            eprintln!("[sculpt3d] mascara: {}", op.label());
            return true;
        }
        // **SUBDIVIDIR.** ⚠️ O log imprime a contagem NOVA porque o preço desta
        // tecla é exponencial e invisível: quatro faces onde havia uma, a cada
        // toque. Um botão que quadruplica a malha sem dizer quanto ela ficou é
        // um botão que o artista aperta uma vez a mais.
        if code == K::KeyK {
            if scene.subdivide() {
                eprintln!(
                    "[sculpt3d] subdividida: nivel {} de {} -- {} vertices / {} faces / {} triangulos",
                    scene.stack.level(),
                    scene.stack.level_count() - 1,
                    scene.mesh().vert_count(),
                    scene.mesh().face_count(),
                    scene.mesh().triangle_count()
                );
            } else {
                eprintln!("[sculpt3d] so' do TOPO: suba (.) antes de subdividir");
            }
            return true;
        }
        // **TAPAR BURACO.** ⚠️ O log diz o número dos DOIS desfechos, e o segundo
        // é o que importa: uma beira que não fecha deixa a malha aberta ali, e
        // *deixar em silêncio* é como o artista conclui que a tecla não funciona.
        if code == K::KeyO {
            match scene.close_holes() {
                Some(r) if r.is_noop() => eprintln!(
                    "[sculpt3d] nenhum buraco: a malha ja' e' fechada ({} arestas de beira sobrando)",
                    r.left_open()
                ),
                Some(r) => eprintln!(
                    "[sculpt3d] tapados {} buraco(s) -- {} vertices / {} faces ({} arestas de beira sobrando)",
                    r.filled(),
                    scene.mesh().vert_count(),
                    scene.mesh().face_count(),
                    r.left_open()
                ),
                None => eprintln!(
                    "[sculpt3d] nao' tapa com a pilha montada: tapar muda a TOPOLOGIA, e todo nivel acima e' subdivisao dela -- tape ANTES de subdividir"
                ),
            }
            return true;
        }
        // **RECONSTRUIR (voxel remesh).** ⚠️ O log traz o ANTES e o DEPOIS na
        // mesma linha porque este botão não muda a forma — ele muda a MALHA, e
        // sem os dois números o artista vê a mesma escultura e não tem como
        // saber se a tecla fez alguma coisa. O número de células explica o
        // tempo: ele é o cubo da resolução (medido em `measure_remesh`).
        if code == K::KeyV {
            match scene.remesh(ph2d_sdf::DEFAULT_RESOLUTION) {
                Some(r) => eprintln!(
                    "[sculpt3d] reconstruida: {} -> {} vertices / {} -> {} faces ({} celulas, {} buraco(s) tapado(s))",
                    r.verts.0, r.verts.1, r.faces.0, r.faces.1, r.cells, r.holes_filled
                ),
                None => eprintln!(
                    "[sculpt3d] nao' reconstroi com a pilha montada: o remesh troca a TOPOLOGIA, e todo nivel acima e' subdivisao dela -- reverta os niveis antes"
                ),
            }
            return true;
        }
        // **DES-SUBDIVIDIR.** ⚠️ Fica no `J` porque é o vizinho do `K`, e o par
        // diz o que faz: `K` acrescenta um nível ACIMA, `J` reconstrói um
        // ABAIXO. O log diz a contagem NOVA pela razão inversa à do `K` — aqui a
        // malha que o artista vê não muda de forma nenhuma, e sem o número ele
        // não tem como saber se o gesto fez alguma coisa.
        if code == K::KeyJ {
            if scene.reverse_level() {
                let base = scene.stack.level_mesh(0);
                eprintln!(
                    "[sculpt3d] revertida: nivel {} de {} -- a base nova tem {} vertices / {} faces",
                    scene.stack.level(),
                    scene.stack.level_count() - 1,
                    base.map_or(0, ph2d_mesh::Mesh::vert_count),
                    base.map_or(0, ph2d_mesh::Mesh::face_count)
                );
            } else {
                eprintln!(
                    "[sculpt3d] nao' reverte: esta malha nao e' uma subdivisao (ou desca ao nivel 0 antes)"
                );
            }
            return true;
        }
        // ⚠️ **Descer e subir NÃO é uma edição** — ver `change_level`. O log diz o
        // nível porque a malha de baixo se PARECE com a de cima alisada: sem o
        // número, o artista não sabe em qual está.
        if code == K::Comma || code == K::Period {
            let up = code == K::Period;
            if scene.change_level(up) {
                eprintln!(
                    "[sculpt3d] nivel {} de {} -- {} vertices",
                    scene.stack.level(),
                    scene.stack.level_count() - 1,
                    scene.mesh().vert_count()
                );
            } else {
                eprintln!(
                    "[sculpt3d] ja' esta' no {}",
                    if up { "TOPO" } else { "nivel 0" }
                );
            }
            return true;
        }
        if let Some(v) = verb {
            // ⚠️ **Arma o default do verbo, e só se o artista ainda não mexeu.**
            // O precedente é o `arm_inflate_defaults` do Painter: um verbo pode
            // querer nascer diferente (a máscara nasce em força cheia, senão ela
            // protege pela metade e o barro se move por baixo), e nenhum verbo
            // pode APAGAR uma escolha deliberada. "Não mexeu" é a força ser
            // exatamente o default do verbo que está saindo.
            let old = scene.brush.verb;
            if (scene.brush.strength - old.default_strength()).abs() < 1e-6 {
                scene.brush.strength = v.default_strength();
            }
            scene.brush.verb = v;
            eprintln!(
                "[sculpt3d] verbo: {} (forca {:.2})",
                v.label(),
                scene.brush.strength
            );
            return true;
        }
        match code {
            K::BracketLeft | K::BracketRight => {
                let f = if code == K::BracketRight {
                    RADIUS_STEP
                } else {
                    1.0 / RADIUS_STEP
                };
                scene.radius_px *= f;
                // O clamp mora na porta, então o LOG mostra o número que o dab
                // vai de fato usar — imprimir o cru faria a tecla parecer viva
                // depois de o teto ter sido alcançado.
                scene.radius_px = scene.radius_px();
                eprintln!("[sculpt3d] raio: {:.0} px de tela", scene.radius_px);
                true
            }
            // **O INTERRUPTOR DA DOAÇÃO** — barro ⇄ luz ⇄ desligada.
            //
            // ⚠️ Gesto de SMOKE, como o `Q`/`E`/`R`/`F` da luz: a UI final é o
            // toggle *"iluminada pela forma abaixo"* na pilha de camadas
            // (`docs/3D/05.2`), e ele espera a escultura ser uma CAMADA do
            // documento. Enquanto ela é um viewport solto, uma tecla é a única
            // porta honesta — um checkbox num painel diria que a forma pertence
            // a um documento a que ela ainda não pertence.
            K::KeyD => {
                let label = scene.cycle_role();
                eprintln!("[sculpt3d] a forma agora e: {label}");
                true
            }
            K::KeyX | K::KeyY | K::KeyZ => {
                let axis = match code {
                    K::KeyX => &mut scene.symmetry.x,
                    K::KeyY => &mut scene.symmetry.y,
                    _ => &mut scene.symmetry.z,
                };
                *axis = !*axis;
                eprintln!("[sculpt3d] espelho: {:?}", scene.symmetry);
                true
            }
            // **A LUZ.** Girar a lâmpada principal em torno da cena e subi-la.
            //
            // ⚠️ Isto é o gesto do SMOKE, não a UI final: o card de Lighting do
            // Painter já é o lugar onde este rig se autora, e é ele que a M4
            // conecta. Um segundo card aqui seria a segunda porta para o mesmo
            // número. Estas teclas existem para o Enio poder ver a forma reacender
            // sem abrir um documento de pintura.
            K::KeyQ | K::KeyE => {
                let d = if code == K::KeyE {
                    LIGHT_STEP_DEG
                } else {
                    360 - LIGHT_STEP_DEG
                };
                let l = scene.rig.current_mut();
                l.angle_deg = (l.angle_deg + d) % 360;
                eprintln!(
                    "[sculpt3d] luz: azimute {}deg elevacao {}deg",
                    l.angle_deg, l.elev_deg
                );
                true
            }
            K::KeyR | K::KeyF => {
                let l = scene.rig.current_mut();
                let up = code == K::KeyR;
                // Clampado no piso do resolvedor, e não em 0: abaixo dele a
                // resposta plana vai a zero e o modelo relativo dividiria por ~0.
                l.elev_deg = if up {
                    (l.elev_deg + LIGHT_STEP_DEG).min(90)
                } else {
                    l.elev_deg
                        .saturating_sub(LIGHT_STEP_DEG)
                        .max(ph2d_light::MIN_ELEV_DEG)
                };
                eprintln!(
                    "[sculpt3d] luz: azimute {}deg elevacao {}deg",
                    l.angle_deg, l.elev_deg
                );
                true
            }
            _ => false,
        }
    }

    fn sculpt3d_scene_mut(&mut self) -> Option<&mut Sculpt3dScene> {
        self.gfx.as_mut()?.sculpt3d.as_mut()
    }
}
