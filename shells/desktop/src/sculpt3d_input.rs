//! **O GESTO** — o que a mão faz com a cena 3D.
//!
//! Filho (`#[path]`) de [`super`] para alcançar os campos privados da
//! [`Sculpt3dScene`]; o corte é o que o cabeçalho do pai já anunciava: *a cena e
//! o passe* lá, *o gesto* aqui. As portas daqui são as que o `input_dispatch`
//! chama, e todas recusam no primeiro `if` sem cena armada — a promessa de
//! removibilidade do `docs/3D/02.3` no nível do frame.

use super::{Drag, Grip, ORBIT_RAD_PER_PX, Sculpt3dScene, Verb};
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

    pub(super) fn sculpt3d_scene_mut(&mut self) -> Option<&mut Sculpt3dScene> {
        self.gfx.as_mut()?.sculpt3d.as_mut()
    }
}
