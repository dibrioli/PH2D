//! **O overlay de joints** — irmão do `physics_overlay`, separado dele quando os
//! dois juntos passaram do cap de 600 LOC do shell.
//!
//! Colliders e joints respondem duas perguntas diferentes com a mesma técnica:
//! *que forma é esta, fisicamente?* e *a que isto está preso?*. Manter os dois
//! num arquivo é o que o estourou; mantê-los VIZINHOS é o que guarda a regra de
//! espaço-de-tela (ver o cabeçalho do módulo pai) num lugar só para ambos.
//!
//! ## W-J1 — o joint DESENHA o que ele é
//!
//! Até esta wave havia uma figura só (segmento + dois anéis) para os quatro
//! tipos: o canvas dizia *"há um joint aqui"* e todo o resto — tipo, alcance de
//! limite, comprimento de repouso, folga, deformação, e **de quem é cada
//! ponta** — era número cego no §12 ou nada. Agora cada fato tem geometria:
//!
//! | fato | como se vê |
//! |---|---|
//! | qual tipo | o GLIFO (anel · quadrado · zigue-zague · fio) |
//! | de quem é cada ponta | a linha de posse: **A sólida, B tracejada** |
//! | alcance de um limite | o arco, com paredes, e a agulha no ângulo VIVO |
//! | para que lado o motor gira | o mesmo glifo de giro da zona de torque |
//! | repouso / máximo | o anel de comprimento, em MUNDO (dá zoom, cresce) |
//! | a restrição NÃO está sendo imposta | o vão entre as âncoras, em VERMELHO |
//!
//! ⚠️ **A distinção entre as pontas é GEOMÉTRICA, não de cor** — a paleta do
//! overlay já está cheia (verde estático · ciano dinâmico · branco contato ·
//! laranja força · violeta torque · amarelo lançamento · magenta sensor ·
//! ciano-claro linha d'água), e um azul-esverdeado novo leria como contorno de
//! collider. Tracejar é a diferença que sobra, e ela não colide com nada.
//!
//! ⚠️ **O desenho lê a ponte, nunca o componente ECS** ([`JointView`], plano 02
//! P2): um joint cujos corpos não resolvem não tem view, e desenhá-lo do
//! componente pintaria uma relação que o solver não está impondo.

use ph2d_ecs::SimWorld;
use ph2d_host::WindowSize;
use ph2d_physics_ecs::{JointKind, JointView};
use ph2d_render::Camera2d;
use ph2d_vector::{BezPath, Point};

use super::physics_overlay_annotations::torque_glyph;
use super::physics_overlay_joint_glyphs::{
    length_ring, limit_arc, pin_glyph, ring_px, rod_bar, rope_span, screen_of, slider_rail,
    spring_zigzag, weld_glyph, wheel_strut,
};

/// Raio do anel desenhado em cada âncora, px de tela. Grande o bastante para
/// ver, pequeno o bastante para não esconder a arte sob uma corrente deles.
const JOINT_DOT_PX: f64 = 3.0; // LITERAL-PX-OK: chrome de overlay, raio de tela

/// Traço e vão da linha de posse do corpo **B**, px de tela.
const DASH_PX: f64 = 6.0; // LITERAL-PX-OK: chrome de overlay
const GAP_PX: f64 = 4.5; // LITERAL-PX-OK: chrome de overlay

/// A partir de quantos pixels de TELA um vão entre âncoras que deveriam
/// coincidir vira a marca vermelha de deformação.
///
/// Um pixel: abaixo disso não há o que desenhar, e o resíduo do solver em
/// repouso é **muito** menor que isso (medido no gate — 0,0000 px de separação
/// num pino assentado). Um limiar em METROS precisaria de um número mágico e
/// mentiria em zoom alto; a pergunta honesta é *"dá para ver?"*.
const STRAIN_MIN_PX: f64 = 1.0; // LITERAL-PX-OK: limiar de visibilidade

/// Joints — âmbar, para lerem como uma terceira coisa ao lado do cenário verde
/// e dos movedores ciano, em vez de como um deles.
pub(super) const JOINT_RGBA: [f32; 4] = [0.98, 0.75, 0.25, 0.9]; // LITERAL-COLOR-OK: overlay de joint

/// As linhas de posse e o arco de limite: o MESMO âmbar, apagado — não são
/// objetos novos, são o mesmo joint dizendo a quem se prende e até onde vai.
pub(super) const JOINT_DIM_RGBA: [f32; 4] = [0.98, 0.75, 0.25, 0.5]; // LITERAL-COLOR-OK: overlay de joint (posse/limite)

/// **A restrição NÃO está sendo imposta** — vermelho, a única cor do overlay
/// que diz *isto não está onde deveria*.
///
/// ⚠️ **O significado foi MEDIDO, e não é o que o nome "tensão" sugeria.** Os
/// impulse joints do rapier são RÍGIDOS: um pino segurando um corpo 500× mais
/// pesado, e outro levando um martelo de 400×, abriram **0,00000 m** entre as
/// âncoras nos 200 ticks — o vermelho por CARGA é inalcançável, e a linha
/// vermelha do RUBE descreve os joints *soft* do Box2D, que não portam.
///
/// O que ABRE o vão é a arquitetura: um joint não move um corpo **kinematic**
/// (massa infinita), então dois corpos curva-dirigidos que a animação afasta
/// ficam separados com o pino desenhado por cima — medido, **1,50 m = 150 px**.
/// É exatamente o estado em que o W-BakeJoint deixa um rig assado, e sem esta
/// marca ele desenharia um pino perfeitamente normal sobre dois objetos que já
/// não estão presos um ao outro.
pub(super) const JOINT_STRAIN_RGBA: [f32; 4] = [0.96, 0.32, 0.28, 0.95]; // LITERAL-COLOR-OK: overlay de joint nao imposto

/// **O overlay de joints.** Um caminho por cor, como o contorno faz.
///
/// `gravity` decide para onde uma corda frouxa pendura — a mesma fonte que diz
/// onde fica a superfície de uma poça (W-Buoyancy). Sem gravidade a corda
/// desenha reta.
pub(super) fn joint_marks(
    show: bool,
    views: &[JointView],
    gravity: [f32; 2],
    camera: &Camera2d,
    window: WindowSize,
) -> Vec<(BezPath, [f32; 4])> {
    if !show {
        return Vec::new();
    }
    let mut out = Vec::new();
    // A gravidade em direção de TELA, uma vez por quadro: a barriga da corda
    // cai para onde as coisas caem, sob qualquer flip da câmera.
    let g_screen = gravity_on_screen(gravity, camera, window);
    for v in views {
        let a = screen_of(camera, window, v.anchor_a);
        let b = screen_of(camera, window, v.anchor_b);
        // **Um joint ROMPIDO desenha em vermelho e SEM envelope** (W-J7). Ele
        // continua na cena, com tudo que o artista autorou — o que parou foi a
        // restrição —, então a geometria fica e só a cor muda; mas o arco de
        // limite, o anel de comprimento e a seta do motor são *o que o joint
        // IMPÕE*, e nada disso está mais em vigor. Desenhá-los descreveria uma
        // regra que o solver deixou de aplicar, que é exatamente a divergência
        // desenho×solver que o P2 do plano existe para proibir.
        //
        // **Um joint DESLIGADO desenha apagado** (W-J8) — a mesma figura, a
        // mesma geometria, a mesma cor, com um terço da tinta. Ele não rompeu e
        // não está errado: o artista o desarmou, e o desenho tem de dizer
        // *presente, não em vigor* sem tomar emprestado o vermelho, que já
        // significa *isto não está segurando por acidente*.
        //
        // ⚠️ Um joint desligado **nunca é `broken`** — a `JointView` separa as
        // duas perguntas na fonte, porque as duas escrevem a MESMA flag do
        // rapier; sem essa separação desarmar um joint o pintaria vermelho, com
        // estouro de ruptura.
        let (main, dim) = if v.broken {
            (JOINT_BROKEN_RGBA, JOINT_BROKEN_DIM_RGBA)
        } else if !v.active {
            (JOINT_OFF_RGBA, JOINT_OFF_DIM_RGBA)
        } else {
            (JOINT_RGBA, JOINT_DIM_RGBA)
        };
        out.push((ownership_lines(v, a, b, camera, window), dim));
        let (span, glyph) = kind_marks(v, a, b, g_screen, camera, window);
        out.push((span, main));
        out.push((glyph, main));
        if v.broken {
            // O ESTOURO: onde ele partiu. Desenhado do ESTADO e não do evento,
            // de propósito — um clarão de seis ticks sobre uma cena que segue
            // rompida some antes de o artista olhar, e a pergunta que ele faz
            // depois (*onde isto arrebentou?*) tem de continuar respondida.
            out.push((break_burst(a, b), JOINT_BROKEN_RGBA));
            continue;
        }
        if let Some(strain) = strain_mark(v, a, b) {
            out.push((strain, JOINT_STRAIN_RGBA));
        }
        // O ENVELOPE AUTORADO, na banda apagada: até onde a dobradiça pode ir,
        // e o comprimento que a mola/corda nomeia. Não são o joint — são o que
        // o artista permitiu a ele, e por isso lêem como fundo do glifo (a
        // mesma distinção que separa a seta de força do anel do falloff).
        // ⚠️ **The arc is the envelope of an ANGULAR range only.** A Slider is
        // limited too, and its range is a stroke in metres — drawn by
        // `slider_rail`'s end-of-travel ticks, not by a circle at 0.5 radians.
        // Without this question a rail painted BOTH, and the arc it painted
        // described a hinge that does not exist.
        //
        // ⚠️ **`dim`/`main`, não as constantes acesas** (W-J8): o envelope de um
        // joint DESLIGADO tem de apagar junto com ele, senão a dobradiça some e o
        // arco dela fica brilhando sozinho. E ele É desenhado num joint
        // desligado — ao contrário de um rompido, que sai por `continue` acima —
        // porque desligar é AUTORIA: o artista continua ajustando o alcance, e
        // esconder o que ele está ajustando é o oposto do que o botão promete.
        if let Some(arc) = v
            .limits
            .filter(|_| !v.kind.limits_in_metres())
            .map(|l| limit_arc(camera, window, v.anchor_a, v.angle_a, l, v.angle_b))
        {
            out.push((arc, dim));
        }
        if let Some(len) = v.length {
            out.push((length_ring(camera, window, v.anchor_a, len), dim));
        }
        // O motor reusa o glifo de giro da zona de torque: é a MESMA pergunta
        // (*para que lado isto gira?*), então é a mesma figura, e a cor diz de
        // quem ela é. Uma segunda figura para a mesma pergunta seria um segundo
        // vocabulário a aprender.
        if let Some(path) = v
            .motor_speed
            .filter(|s| *s != 0.0)
            .and_then(|s| torque_glyph(v.anchor_a[0], v.anchor_a[1], s, camera, window))
        {
            out.push((path, main));
        }
    }
    out
}

/// **O joint ROMPEU** (W-J7) — o mesmo vermelho da deformação, porque diz a
/// mesma família de coisa (*isto não está segurando*), e o desenho o distingue:
/// a deformação é um traço ENTRE as âncoras, o rompimento é um estouro SOBRE
/// elas, com o joint inteiro tingido.
pub(super) const JOINT_BROKEN_RGBA: [f32; 4] = [0.96, 0.32, 0.28, 0.95]; // LITERAL-COLOR-OK: overlay de joint rompido
/// As linhas de posse de um joint rompido: o mesmo vermelho, apagado — elas
/// seguem respondendo *quais dois objetos este joint NOMEIA*, que continua
/// verdadeiro depois que ele deixa de segurá-los.
pub(super) const JOINT_BROKEN_DIM_RGBA: [f32; 4] = [0.96, 0.32, 0.28, 0.45]; // LITERAL-COLOR-OK: overlay de joint rompido (posse)

/// **O joint está DESLIGADO** (W-J8) — o mesmo âmbar, com um terço da tinta.
///
/// Uma cor nova seria um sexto membro da paleta do overlay a aprender; o que
/// mudou não é *que coisa é esta* e sim *ela está em vigor?*, e a resposta certa
/// para isso é a mesma figura mais fraca. ⚠️ **Nunca o vermelho**: aquele já diz
/// *isto não está segurando e não era para ser assim* (deformação e ruptura), e
/// um joint desarmado está exatamente como o artista o quis.
pub(super) const JOINT_OFF_RGBA: [f32; 4] = [0.98, 0.75, 0.25, 0.3]; // LITERAL-COLOR-OK: overlay de joint desligado
/// As linhas de posse de um joint desligado — mais fracas ainda, pela mesma
/// razão que as de um aceso são (elas são o fundo, não o objeto).
pub(super) const JOINT_OFF_DIM_RGBA: [f32; 4] = [0.98, 0.75, 0.25, 0.17]; // LITERAL-COLOR-OK: overlay de joint desligado (posse)

/// Meia-largura do estouro, px de tela.
const BURST_PX: f64 = 7.0; // LITERAL-PX-OK: chrome de overlay

/// **O estouro** — uma estrela de seis pontas no ponto onde o joint partiu.
///
/// Seis e não quatro: a cruz de quatro braços já É o contato (W-Contacts,
/// branca) e o `×` de 45° já é o flash de um toque novo, então um terceiro
/// membro dessa família leria como um deles. A estrela não colide com nada no
/// vocabulário do overlay.
///
/// O ponto é o MEIO das duas âncoras, a mesma escolha que
/// [`ph2d_physics::JointBreak::point`] faz e pela mesma razão: num Pin e num
/// Weld as duas coincidem, num Spring/Rope/Slider são as pontas do vão, então o
/// meio está sobre o segmento desenhado em todos os casos.
fn break_burst(a: Point, b: Point) -> BezPath {
    let (cx, cy) = ((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
    let mut path = BezPath::new();
    // Três diâmetros a 0°, 60° e 120° — seis pontas, sem transcendental no laço
    // (os cossenos e senos de 60° e 120° são as duas constantes abaixo).
    const H: f64 = 0.5; // cos 60°
    const V: f64 = 0.866_025_4; // sin 60°
    for (dx, dy) in [(1.0, 0.0), (H, V), (-H, V)] {
        path.move_to(Point::new(cx - dx * BURST_PX, cy - dy * BURST_PX));
        path.line_to(Point::new(cx + dx * BURST_PX, cy + dy * BURST_PX));
    }
    path
}

/// **O FANTASMA do corpo B** — âmbar quase apagado, a silhueta de onde o limite
/// que está sendo arrastado deixaria o corpo parar.
///
/// É o *'L'* do RUBE sem modo: arrastar a parede JÁ posa. Sem ele o artista
/// arrasta um tracinho num arco e descobre o que autorou só depois de dar Play —
/// o arco tem uma agulha viva em `angle_b`, que diz onde o corpo ESTÁ, e nada
/// dizia onde ele PARARIA.
pub(super) const JOINT_GHOST_RGBA: [f32; 4] = [0.98, 0.75, 0.25, 0.28]; // LITERAL-COLOR-OK: overlay de joint (fantasma)

/// **A BANDA ELÁSTICA do gesto de criar** (W-J4) — âmbar, tracejada, do ponto do
/// press até o cursor.
///
/// ⚠️ **Desenhada mesmo com o overlay DESLIGADO** (tecla `B`), ao contrário de
/// todo o resto deste módulo: os outros traços são ANOTAÇÃO de coisas que
/// existem (*onde está o collider, até onde vai o limite*) e o artista escolhe
/// vê-los; esta é o **feedback de um gesto em andamento**, e um gesto que não se
/// vê é um gesto que parece não ter começado.
///
/// Tracejada porque o joint ainda **não existe** — a linha de posse sólida é a de
/// um vínculo real (W-J1), e usar o mesmo traço aqui prometeria que já há um.
pub(super) fn draw_band(
    band: Option<([f32; 2], [f32; 2])>,
    camera: &Camera2d,
    window: WindowSize,
) -> Option<BezPath> {
    let (from, to) = band?;
    let a = screen_of(camera, window, from);
    let b = screen_of(camera, window, to);
    let mut p = BezPath::new();
    // Um anel no ponto de origem: ele é a ÂNCORA que vai nascer ali, e sem ele o
    // press não deixa marca nenhuma até o cursor andar.
    ring_px(a, JOINT_DOT_PX * 2.0, &mut p);
    dashed(a, b, &mut p);
    Some(p)
}

/// **A MÃO** (W-Grab) — verde-limão, a única cor livre na paleta deste overlay
/// (verde=estático · ciano=dinâmico · violeta=kinematic/torque · magenta=sensor ·
/// âmbar=joint · vermelho=ruptura · branco=contato · laranja=força).
pub(super) const GRAB_RGBA: [f32; 4] = [0.55, 1.0, 0.30, 0.95]; // LITERAL-COLOR-OK: overlay da mão

/// **A mola da mão**, do cursor até o ponto de pega — desenhada como o **ZIGZAG**
/// de mola, com um anel no ponto pego.
///
/// ⚠️ **A FORMA diz o mecanismo e a COR diz de quem é:** o artista já aprendeu no
/// W-J1 que zigzag é mola, e a mão **é** uma mola (uma `SpringJoint` de verdade
/// para uma âncora invisível no cursor). Um traço reto diria *"isto é rígido"*, o
/// que é exactamente a coisa errada a prometer — ela cede contra parede, e é isso
/// que a distingue de um teleporte.
///
/// ⚠️ **Desenhada mesmo com o overlay DESLIGADO**, pela mesma razão da
/// [`draw_band`]: é feedback de um gesto em andamento, não anotação.
pub(super) fn draw_grab(
    grab: Option<([f32; 2], [f32; 2])>,
    camera: &Camera2d,
    window: WindowSize,
) -> Option<BezPath> {
    let (cursor, hold) = grab?;
    let a = screen_of(camera, window, cursor);
    let b = screen_of(camera, window, hold);
    let mut p = spring_zigzag(a, b);
    // O anel marca ONDE no corpo a mão pegou — o ponto que a mola persegue. Sem
    // ele, um clique sem arrasto (que não move nada, de propósito) não deixaria
    // marca nenhuma, e o gesto pareceria não ter começado.
    ring_px(b, JOINT_DOT_PX * 2.0, &mut p);
    Some(p)
}

/// A silhueta de B na pose que `limit` permite, ou `None` quando não há arrasto
/// de limite em voo / o corpo B não tem collider.
///
/// ⚠️ **Desenha e nada mais.** O fantasma nunca escreve pose: ele é uma função
/// pura da view e do número que o arrasto está autorando. O corpo real só se move
/// quando o solver o move — e é justamente essa separação que torna possível
/// posar um limite com a simulação parada.
///
/// ⚠️ **O MOVIMENTO é o do grau de liberdade livre, e esse foi o bug.** Numa
/// dobradiça o corpo GIRA em torno da âncora por `Δ = (angle_a + limit) −
/// angle_b`; num trilho ele **DESLIZA** pelo eixo, porque um curso é uma
/// distância. Até 2026-07-26 este era o **quarto** leitor de `JointView::limits`
/// que o W-J5 não avisou (os outros três: o arco, as alças e a escrita do
/// arrasto) — ele girava o corpo por *0,9 radiano* para um curso de *0,9 metro*,
/// e o resultado era a silhueta solta que o Enio fotografou: *"aparece um gizmo
/// fantasma rodando que parece não estar relacionado corretamente ao joint"*.
///
/// Deslizando, ele passa a ser a coisa mais útil que este overlay desenha num
/// Slider: **o carrinho onde ele vai PARAR**, enquanto a alça ainda está na mão.
pub(super) fn limit_ghost(
    sim: &SimWorld,
    views: &[JointView],
    posed: Option<(ph2d_ecs::Entity, f32)>,
    camera: &Camera2d,
    window: WindowSize,
) -> Option<BezPath> {
    let (joint, limit) = posed?;
    let v = views.iter().find(|v| v.entity == joint)?;
    let world = sim.world();
    let col = world.get::<ph2d_physics_ecs::Collider>(v.body_b)?;
    let mut chain = Vec::new();
    let t = ph2d_ecs::world_transform_into(world, v.body_b, &mut chain)?;
    let live = [t.translation.x, t.translation.y];

    // Onde o corpo estaria, e virado como — uma resposta por tipo de movimento.
    let (centre, d) = if v.kind.limits_in_metres() {
        // TRILHO: desliza pelo eixo. O deslocamento vivo é a separação das duas
        // âncoras ao longo dele (é isso que o rapier chama de posição do
        // prismatic), então o fantasma anda o que falta até o fim de curso.
        let axis = v.axis?;
        let along =
            |p: [f32; 2]| (p[0] - v.anchor_a[0]) * axis[0] + (p[1] - v.anchor_a[1]) * axis[1];
        let step = limit - along(v.anchor_b);
        ([live[0] + axis[0] * step, live[1] + axis[1] * step], 0.0)
    } else {
        // DOBRADIÇA: gira rígido em torno da âncora A. `Δ` leva a pose VIVA de B
        // até a que o limite nomeia, então o fantasma é o corpo tal como ele
        // encostaria na parede — não uma figura nova.
        let d = (v.angle_a + limit) - v.angle_b;
        let (sin_d, cos_d) = libm::sincosf(d);
        let (dx, dy) = (live[0] - v.anchor_a[0], live[1] - v.anchor_a[1]);
        (
            [
                v.anchor_a[0] + dx * cos_d - dy * sin_d,
                v.anchor_a[1] + dx * sin_d + dy * cos_d,
            ],
            d,
        )
    };
    // O collider onde o SOLVER o põe: offset com a escala assinada dobrada,
    // girado com o corpo — a mesma leitura do `outlines`, porque um fantasma que
    // não casa com o contorno descreveria outro corpo.
    let (ox, oy) = (col.offset[0] * t.scale.x, col.offset[1] * t.scale.y);
    let (sin_r, cos_r) = (t.rotation + d).sin_cos();
    let (wox, woy) = (ox * cos_r - oy * sin_r, ox * sin_r + oy * cos_r);
    Some(super::physics_overlay::collider_outline(
        ph2d_physics_ecs::scaled_shape(col.shape, t.scale),
        centre[0] + wox,
        centre[1] + woy,
        t.rotation + d,
        camera,
        window,
    ))
}

/// O span + o glifo de um tipo. Devolve os dois separados porque o span pode
/// ser vazio (Pin/Weld não vão a lugar nenhum) sem apagar o glifo.
fn kind_marks(
    v: &JointView,
    a: Point,
    b: Point,
    g_screen: (f64, f64),
    camera: &Camera2d,
    window: WindowSize,
) -> (BezPath, BezPath) {
    let mut span = BezPath::new();
    let mut glyph = BezPath::new();
    match v.kind {
        // Pin e Weld COMPARTILHAM um ponto: o glifo mora na âncora, e o span
        // existe só quando as duas discordam — que é a deformação, pintada em
        // vermelho logo abaixo. Aqui ele fica vazio de propósito.
        JointKind::Pin => {
            glyph = pin_glyph(a);
            ring_px(b, JOINT_DOT_PX, &mut glyph);
        }
        JointKind::Weld => {
            glyph = weld_glyph(camera, window, v.anchor_a, v.angle_a);
            ring_px(b, JOINT_DOT_PX, &mut glyph);
        }
        // **A POLIA:** a corda NÃO vai de âncora a âncora — ela sobe até uma
        // roldana, atravessa por cima, e desce até a outra ponta. Um span reto
        // A→B descreveria uma corda que não existe na cena, e é por isso que
        // este é o único tipo cuja view carrega pontos de MUNDO próprios.
        JointKind::Pulley => {
            if let Some((wa, wb)) = v.wheels {
                let pa = screen_of(camera, window, wa);
                let pb = screen_of(camera, window, wb);
                span.move_to(a);
                span.line_to(pa);
                span.line_to(pb);
                span.line_to(b);
                // As roldanas são ANÉIS — rodas, e maiores que uma âncora, que é
                // o que as separa dos dois pontos de amarração ao lado.
                ring_px(pa, JOINT_DOT_PX * 2.0, &mut glyph);
                ring_px(pb, JOINT_DOT_PX * 2.0, &mut glyph);
            }
            ring_px(a, JOINT_DOT_PX, &mut glyph);
            ring_px(b, JOINT_DOT_PX, &mut glyph);
        }
        JointKind::Spring => {
            span = spring_zigzag(a, b);
            ring_px(a, JOINT_DOT_PX, &mut glyph);
            ring_px(b, JOINT_DOT_PX, &mut glyph);
        }
        JointKind::Rope => {
            // A folga é medida em METROS (é aqui que eles existem); o glifo só
            // recebe a razão adimensional.
            let d = (v.anchor_b[0] - v.anchor_a[0]).hypot(v.anchor_b[1] - v.anchor_a[1]);
            let slack = match v.length {
                Some(l) if d > 1e-4 => f64::from(l / d) - 1.0,
                _ => 0.0,
            };
            span = rope_span(a, b, slack, g_screen);
            ring_px(a, JOINT_DOT_PX, &mut glyph);
            ring_px(b, JOINT_DOT_PX, &mut glyph);
        }
        // A BARRA tem duas pontas separadas, como a mola e a corda: o span É o
        // glifo (as duas paralelas) e os anéis marcam os dois olhais.
        JointKind::Rod => {
            span = rod_bar(a, b);
            ring_px(a, JOINT_DOT_PX, &mut glyph);
            ring_px(b, JOINT_DOT_PX, &mut glyph);
        }
        // O Slider compartilha um ponto como o Pin — o glifo é o TRILHO, e o span
        // fica vazio pela mesma razão (quando as duas âncoras discordam, isso é a
        // deformação, pintada em vermelho logo abaixo).
        JointKind::Slider => {
            if let Some(axis) = v.axis {
                glyph = slider_rail(camera, window, v.anchor_a, axis, v.limits);
            }
            ring_px(b, JOINT_DOT_PX, &mut glyph);
        }
        // A RODA compartilha um ponto (o cubo), como o Pin e o Slider — então o
        // span fica vazio e o glifo carrega tudo: o anel do cubo, a mola da
        // suspensão ao longo do eixo, e os batentes de curso.
        JointKind::Wheel => {
            if let Some(axis) = v.axis {
                glyph = wheel_strut(camera, window, v.anchor_a, axis, v.limits);
            }
            ring_px(b, JOINT_DOT_PX, &mut glyph);
        }
    }
    (span, glyph)
}

/// As duas linhas de posse: âncora→centro de cada corpo. **A sólida, B
/// tracejada** — ver o cabeçalho do módulo para o porquê de não ser cor.
fn ownership_lines(
    v: &JointView,
    a: Point,
    b: Point,
    camera: &Camera2d,
    window: WindowSize,
) -> BezPath {
    let mut p = BezPath::new();
    let ca = screen_of(camera, window, v.centre_a);
    let cb = screen_of(camera, window, v.centre_b);
    p.move_to(a);
    p.line_to(ca);
    dashed(b, cb, &mut p);
    p
}

/// Um segmento tracejado, em pixels de tela.
fn dashed(from: Point, to: Point, path: &mut BezPath) {
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    let len = dx.hypot(dy);
    if len < 1e-6 {
        return;
    }
    let (ux, uy) = (dx / len, dy / len);
    let mut t = 0.0;
    while t < len {
        let e = (t + DASH_PX).min(len);
        path.move_to(Point::new(from.x + ux * t, from.y + uy * t));
        path.line_to(Point::new(from.x + ux * e, from.y + uy * e));
        t = e + GAP_PX;
    }
}

/// A marca de **restrição não-imposta**: o vão entre duas âncoras que deveriam
/// coincidir (ver [`JOINT_STRAIN_RGBA`] para o que de fato o abre — medido).
///
/// Só existe para os tipos que compartilham um ponto — as âncoras de uma mola
/// ou de uma corda são as DUAS PONTAS e estão separadas por construção, então
/// pintá-las de vermelho chamaria de erro o funcionamento normal.
fn strain_mark(v: &JointView, a: Point, b: Point) -> Option<BezPath> {
    if !v.kind.shares_a_point() || (b.x - a.x).hypot(b.y - a.y) < STRAIN_MIN_PX {
        return None;
    }
    let mut p = BezPath::new();
    p.move_to(a);
    p.line_to(b);
    Some(p)
}

/// A gravidade como direção unitária de TELA (para onde as coisas caem).
fn gravity_on_screen(gravity: [f32; 2], camera: &Camera2d, window: WindowSize) -> (f64, f64) {
    let m = gravity[0].hypot(gravity[1]);
    if m < 1e-6 {
        return (0.0, 0.0);
    }
    let o = screen_of(camera, window, [0.0, 0.0]);
    let g = screen_of(camera, window, [gravity[0] / m, gravity[1] / m]);
    let (dx, dy) = (g.x - o.x, g.y - o.y);
    let l = dx.hypot(dy);
    if l < 1e-9 {
        (0.0, 0.0)
    } else {
        (dx / l, dy / l)
    }
}

#[cfg(test)]
#[path = "physics_overlay_joints_tests.rs"]
mod joint_tests;
