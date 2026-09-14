//! **O GESTO** — o que a mão faz com a cena 3D.
//!
//! Filho de [`super`] para alcançar os campos privados da [`Sculpt3dScene`]; o corte é o que
//! o cabeçalho do pai já anunciava: *a cena e o passe* lá, *o gesto* aqui. As portas daqui são
//! as que o `input_dispatch` chama.
//!
//! ⚠️ **A promessa de removibilidade do `docs/3D/02.3` continua inteira, e passou a ser
//! cumprida de DUAS maneiras** (W2/L3-A2) — o gate `every_3d_port_is_inert_without_a_scene`
//! conhece as duas, e é lá que elas estão escritas por extenso:
//! - as portas que leem JANELA (`gfx`, `last_pointer`, `modifiers`) continuam em [`App`] e
//!   recusam no primeiro `if` sem cena armada;
//! - ⭐ as três que só precisavam da cena **recebem-na** (ver a secção no fim deste ficheiro),
//!   logo são inconstruíveis sem ela — nenhum `if` as pode esquecer.

use super::{Dab, Drag, Grip, ORBIT_RAD_PER_PX, Sculpt3dScene};
use std::sync::atomic::{AtomicBool, Ordering};

pub fn smoke(
    slot: &mut Option<Sculpt3dScene>,
    gpu: Option<(&std::sync::Arc<wgpu::Device>, (u32, u32))>,
) {
    // Guard estático, o mesmo idioma dos outros smokes do shell — evita um
    // campo em `App` que só existe para dizer "já rodei".
    static ARMED: AtomicBool = AtomicBool::new(false);
    if !crate::smoke_armed() || gpu.is_none() || ARMED.swap(true, Ordering::Relaxed) {
        return;
    }
    let mesh = crate::smoke_mesh();
    crate::announce(&mesh);
    let Some((device, size)) = gpu else {
        return;
    };
    let aspect = size.0 as f32 / size.1.max(1) as f32;
    let mut scene = Sculpt3dScene::new(device, mesh, aspect);
    // ⚠️ **A cena `=7` é montada DEPOIS do `new`, pela porta pública.** Um
    // construtor que recebesse a lista inteira seria a segunda resposta a
    // *"como um objeto entra na cena"* — e a primeira é a que o gesto de
    // blocagem vai usar.
    let extra = crate::scene_objects();
    if !extra.is_empty() {
        for (mesh, pose) in extra {
            scene.push_object(mesh, pose);
        }
        // O enquadramento tem de conhecer a cena INTEIRA: o `new` enquadrou
        // só a primeira peça, e as outras nasceram fora do quadro.
        scene.frame_all(aspect);
    }
    *slot = Some(scene);
}

/// O botão soltou.
/// **O cursor que a costura da divisão 3D pede**, ou `None`.
///
/// ⚠️ **Sem barro na tela ela é muda**: com o módulo desarmado não há canvas
/// 3D nenhum, e uma seta de redimensionar sobre a cena 2D prometeria um
/// gesto que ali não existe.
pub fn seam_cursor(
    host: &impl ph2d_app_host::AppHost,
    scene: &Sculpt3dScene,
) -> Option<winit::window::CursorIcon> {
    let (x, y) = host.pointer();
    scene
        .clay_on_screen()
        .then(|| scene.seam_cursor(x, y))
        .flatten()
}

/// A roda aproxima.
pub fn wheel(host: &impl ph2d_app_host::AppHost, scene: &mut Sculpt3dScene, steps: f32) -> bool {
    // A mesma lei do `pointer_down`: a moldura do app não é da cena. O
    // despachante já pergunta pelo PAINEL antes de chamar aqui, e a metade
    // que ele não faz é a dos fundos de chrome — mas a pergunta é feita
    // INTEIRA e neste arquivo de propósito: quem decide de quem é o gesto é
    // o módulo da cena, não o roteador. Sem isto, rolar sobre a barra do topo
    // dá DOLLY na escultura por baixo, em silêncio.
    let pos = host.pointer();
    if host.pointer_over_chrome(pos.0, pos.1) {
        return false;
    }
    // Mesma lei do `pointer_down`: barro fora da tela, roda do 2D. Sem isto
    // o zoom do canvas ficaria preso enquanto a forma acende a tinta.
    if !scene.shows_clay() {
        return false;
    }
    scene.camera.dolly(steps);
    true
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
                .project(self.pose().point_to_world(hit.point), self.viewport());
            let err = back.map(|(bx, by)| ((bx - x).hypot(by - y), bx, by));
            eprintln!(
                "[sculpt3d] clique ({x:.1}, {y:.1}) viewport {:?} -> acerto {:?} \
                 -> volta {err:?}",
                self.viewport(),
                hit.point
            );
        }
        let brush = self.armed_brush(hit.point);
        // ⚠️ **REFINA E DEPOIS CARIMBA** — ver `refine_for_dab`. E a malha que a
        // linha seguinte recebe pode ter mais vértices que a do `pick_active`
        // acima: é por isso que ela é pedida de novo, por índice, em vez de
        // segurada numa referência desde o topo.
        self.refine_for_dab(brush.verb, brush.density_modo, hit.point, brush.radius);
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

// ═══════════════════════════════════════════════════════════════════════════════════════
// ⭐ **AS TRÊS PORTAS QUE SÓ PRECISAM DA CENA** — funções LIVRES desde 2026-09-11 (W2/L3-A2).
//
// Medido método a método, os doze `impl App` desta família usavam **onze** `self.X` ao todo, e
// **estas três usavam UM**: o acessor da cena. Um `impl App` à volta delas não acrescentava
// nada — só tornava invisível que a lei já era independente da shell.
//
// ⛔ **As outras nove FICAM em `impl App`, e a lista é a que a `line/app-host` tem de cobrir:**
// `gfx` (o `device` e o tamanho da superfície — de onde a cena se cria), `last_pointer` e
// `modifiers` (estado de janela), e três predicados de ARBITRAGEM que perguntam por outras
// famílias (`text_entry_focused`, `vec_pen`, e quem tem o canvas). Nenhum desses é da escultura
// — é por isso que eles não saem daqui por decisão desta linha (CLAUDE.md §5: *a navegação
// orbital mora na shell de propósito*).
// ═══════════════════════════════════════════════════════════════════════════════════════

/// `PH2D_SCULPT3D_SMOKE=1` — a cena pronta: uma esfera de barro para
/// esculpir. `=2` acrescenta a TELA e é a cena da **doação**
/// (`crate::donation`). Roda uma vez, no primeiro frame com GPU.
/// **O dreno de QUADRO do puxão do Grab.** Ver
/// [`Sculpt3dScene::pending_grab`] e `flush_pending_grab`: o evento de
/// ponteiro regista, o quadro carimba, e o pen-up drena o resto.
///
/// ⚠️ **Sem cena aberta é no-op** — e não um `expect`: este é chamado do
/// laço de quadro incondicionalmente, como os irmãos ao lado dele.
pub fn flush_grab(scene: &mut Sculpt3dScene) {
    scene.flush_pending_grab();
    // ⭐ **E o passo do filtro de tecido, pela MESMA porta e pelo mesmo
    // motivo**: um evento de ponteiro regista, o quadro corre. Sem isto o
    // solver avançaria uma vez por evento do sistema, e um rato de
    // `1000 Hz` entrega dezasseis por quadro.
    scene.flush_cloth_filter();
}

pub fn pointer_up(scene: &mut Sculpt3dScene) -> bool {
    if scene.seam_release() {
        return true;
    }
    // ⭐ **Um pen-up sem movimento sobre uma bola é o CLIQUE dela** — ver
    // [`super::Sculpt3dScene::nav_pointer_up`]. Ele devolve cedo porque um
    // arrasto no gizmo nunca abriu um `Drag`.
    if scene.nav_pointer_up() {
        return true;
    }
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
        // ⚠️ **ANTES do fecho, e sem isto o gesto perde a ponta** — a mesma
        // lei (e o mesmo comentário) do `flush_pending_grab` do traço: o
        // último movimento do dedo chega como evento e fica pendente, e o
        // pano pararia onde o último QUADRO o deixou.
        scene.flush_cloth_filter();
        // ⚠️ **A sessão de TECIDO morre antes do fecho** — ela é do gesto, e
        // deixá-la viva faria o arrasto seguinte continuar a simulação deste
        // sobre uma pose que já foi gravada como um passo de undo.
        scene.end_cloth_filter();
        scene.close_stroke();
    }
    was.is_some()
}

/// O ponteiro moveu. Só consome com um arrasto EM CURSO — senão a cena 3D
/// engoliria todo hover do app.
pub fn pointer_move(scene: &mut Sculpt3dScene, x: f32, y: f32) -> bool {
    // ⚠️ **A costura primeiro, pelo motivo do pen-down.**
    if scene.seam_at(x, y) {
        scene.last = (x, y);
        return true;
    }
    // ⚠️ **O gizmo de navegação NÃO usa o `Drag`**, e a razão é a captura: um
    // arrasto nele orbita a câmera e nada mais, então ele não tem de passar
    // pela tabela de verbos nem pelo `last` da peça. Perguntar-lhe primeiro
    // é o que faz o gesto continuar dele mesmo quando o dedo sai do widget.
    if scene.nav_pointer_move(x, y) {
        scene.last = (x, y);
        return true;
    }
    let Some(drag) = scene.drag else {
        return false;
    };
    let (dx, dy) = (x - scene.last.0, y - scene.last.1);
    scene.last = (x, y);
    let height = scene.viewport().1.max(1) as f32;
    match drag {
        // ⚠️ **Manipulação direta: o modelo segue a mão.** `yaw` positivo
        // leva o OLHO para `+X`, e a câmera indo para a direita faz o
        // modelo *parecer* ir para a esquerda — então arrastar para a
        // direita pede `yaw -= dx`. E arrastar para BAIXO mostra o TOPO
        // (o modelo tomba para a frente), que é `pitch += dy`.
        //
        // Os DOIS sinais estavam trocados e o smoke os pegou; o gate que os
        // prende (`dragging_right_turns_the_model_right_and_dragging_down_shows_its_top`,
        // no `ph2d-mesh-render`) mede o modelo NA TELA em vez de argumentar sobre
        // sinais, que foi como o erro entrou.
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
                    let repica = ph2d_sculpt3d::cloth_repica(&scene.brush);
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
                    // a última posição de rato **ANTES** do teste de
                    // acerto, então o passo é dado e só a aplicação é
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
                    // exatos de um espaçamento e para no último, nunca no
                    // ponteiro.
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
