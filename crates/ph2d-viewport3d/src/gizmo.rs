//! ⭐ **O gizmo 3D** — mover, rodar e escalar.
//!
//! Enio, 2026-08-19: *"Não há gzimo 3d para mover os objetos. Precisamos de uma como o do blender."*
//!
//! # O que este arquivo é, e o que ele NÃO é
//!
//! É **lei pura**: projeção, apontar e arrastar, sem `App`, sem ponteiro e sem GPU. Tudo o que aqui
//! entra sai de dois números — a âncora no mundo e a câmera — e por isso todo gesto é gateável sem
//! abrir janela nenhuma. A pintura e a ligação ao ponteiro são os arquivos irmãos
//! (`ph2d_app_field3d::gizmo_paint`, `ph2d_app_field3d::input`).
//!
//! ⚠️ **Mora no SHELL**, e é a mesma razão do [ADR-0150] que já manda na navegação: uma janela 3D
//! não pode obrigar a mexer no `Tool=12`, que está **congelado**.
//!
//! # ⭐ A projeção é a MESMA do traçador
//!
//! [`ph2d_field_render::Screen`] e [`ph2d_field_render::Orbit::project`] são a conta que a marcha de
//! raios usa para construir os raios. Uma segunda cópia dela aqui divergiria meio pixel, e o sintoma
//! seria uma alça que **agarra ao lado da superfície que ela diz mover** — o tipo de defeito que
//! ninguém chama de bug de projeção. O gate `a_point_projects_where_the_march_actually_hits_it`
//! prende as duas metades.
//!
//! # Os eixos são os do MUNDO
//!
//! Como o default do Blender ("Global"). O nó pode estar rodado — os cilindros da cena 1 estão — e
//! nesse caso mover *ao longo do próprio eixo dele* é uma segunda orientação, que o Blender expõe
//! num seletor. Ela é item ABERTO, e não uma omissão: escolher a orientação é decisão de produto, e
//! entregar só a local seria escolher por quem não pediu.
//!
//! # ⛔ Por que o ESCALAR tem UMA alça, e não três
//!
//! [ADR-0161 §6] mediu e decidiu: a escala de um nó é **uniforme**, porque escala não-uniforme
//! **destrói a propriedade de distância** (‖∇f‖ = 1) de que tudo neste módulo depende — sem ela o
//! raio deixa de ser o raio e a marcha atravessa a superfície.
//!
//! Então três caixas por eixo, como as do Blender, seriam três controles a **prometer o que o
//! modelo não entrega**: arrastar a de X escalaria os três, e o artista concluiria que o app tem um
//! bug. A alça de escala é **uma**, é um punho de tamanho (não um eixo), e por isso não leva cor de
//! eixo nenhuma.
//!
//! [ADR-0150]: ../../../docs/architecture/decisions/0150-3d-sculpt-is-a-mesh-that-donates-shading-sculptgl-referenced.md
//! [ADR-0161 §6]: ../../../docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md

use ph2d_field::xform::{cross, dot};
use ph2d_field_render::{Orbit, Screen};

/// ⭐ **As medidas em pixels** vivem no irmão — ver [`field3d_gizmo_metrics`](self::metrics).
#[path = "gizmo_metrics.rs"]
mod metrics;
pub use metrics::*;

/// **O que o gizmo faz agora.** Os três verbos, num seletor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Move,
    Rotate,
    Scale,
}

impl Mode {
    /// A ordem em que eles aparecem no painel. ⚠️ **É a fonte da contagem** — quem acrescentar um
    /// verbo mexe aqui e o painel segue sozinho.
    pub const ALL: [Mode; 3] = [Mode::Move, Mode::Rotate, Mode::Scale];

    /// ⚠️ Uma **chave** de i18n, nunca um rótulo pronto (HR-15).
    pub fn key(self) -> &'static str {
        match self {
            Mode::Move => "panel.model3d.mode.move",
            Mode::Rotate => "panel.model3d.mode.rotate",
            // ⚠️ O rótulo desta diz **uniforme**, porque é o que o modelo entrega (ver o doc do
            // módulo). Um rótulo que promete mais do que o modelo dá é como se aprende que o app
            // tem um bug que ele não tem.
            Mode::Scale => "panel.model3d.mode.scale",
        }
    }
}

/// A alça agarrada. `usize` é o índice do eixo: 0 = X, 1 = Y, 2 = Z.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handle {
    /// Mover ao longo de um eixo.
    Axis(usize),
    /// Mover no plano **perpendicular** a este eixo (o quadrado XY é `Plane(2)`).
    Plane(usize),
    /// Mover no plano da tela.
    View,
    /// Rodar em torno de um eixo do mundo.
    Ring(usize),
    /// Rodar em torno da direção da **vista** — a argola que nunca fica de perfil.
    ViewRing,
    /// Escalar **uniformemente**. Uma só, e o doc do módulo diz porquê.
    Grip,
    /// ⭐⭐⭐ **UM VÉRTICE do contorno**, pelo índice (W133) — a alça que move um ponto da forma, e
    /// não a forma inteira.
    ///
    /// Enio, 2026-09-07: *«os vertex devem aparecer no canvas em tempo real e o usuário então poderá
    /// movê-los através do gizmo no próprio canvas»*.
    ///
    /// ⚠️ **O índice é do VÉRTICE, e não da linha do painel** — quem converte é a ponte, que é quem
    /// sabe onde a tabela daquela forma põe as coordenadas ([`ph2d_field::vertex_rows`]). *A lei do
    /// gizmo não pode saber a ordem das linhas de uma tabela: seria a segunda cópia dela.*
    Vertex(usize),
}

/// ⭐⭐⭐ **O SUJEITO de um pedido de arrasto** (W133) — o nó inteiro, ou **um ponto** dele.
///
/// ⚠️ **Ele viaja com o pedido, e não é derivado no destino.** A ponte com a cena recebe um
/// deslocamento de mundo e tem de saber o que mover; perguntar «que alça estava agarrada?» ali
/// obrigaria o estado do gesto a atravessar mais uma fronteira, e é entre quadros que a selecção
/// pode mudar. *Quem sabe o sujeito é quem agarrou.*
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// O nó — mover, rodar ou escalar a peça.
    Node,
    /// Um vértice do contorno, pelo índice.
    Vertex(usize),
}

impl Handle {
    /// **Quem esta alça move.**
    ///
    /// ⚠️ **Lista FECHADA de propósito**: uma alça nova que mexa noutra coisa que não o nó (uma
    /// aresta, uma tangente) é **erro de compilação** aqui, e quem a escrever tem de dizer o
    /// sujeito. Um `_ => Node` faria a próxima nascer a mover a peça inteira, em silêncio.
    pub fn target(self) -> Target {
        match self {
            Handle::Vertex(i) => Target::Vertex(i),
            Handle::Axis(_)
            | Handle::Plane(_)
            | Handle::View
            | Handle::Ring(_)
            | Handle::ViewRing
            | Handle::Grip => Target::Node,
        }
    }
}

/// **Onde o gizmo está e para onde ele aponta**, no mundo. Publicado pela ponte com a cena, que é
/// quem tem o mundo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Anchor {
    /// A entidade que ele move — a identidade viaja com a âncora, senão o arrasto teria de a
    /// procurar outra vez e podia achar outra.
    pub entity: u64,
    pub origin: [f32; 3],
    /// ⭐ **Os três eixos, JÁ NO MUNDO** — a orientação escolhida ([`Frame`]).
    ///
    /// ⚠️ Eles viajam prontos de propósito: assim a lei do gizmo deixa de saber que existe uma
    /// escolha de orientação, e quem a faz é a ponte, que é quem tem a pose do nó. Sem isto, cada
    /// função daqui teria de perguntar «global ou local?» — o mesmo `if` repetido em cinco sítios,
    /// que é como um deles fica para trás.
    pub axes: [[f32; 3]; 3],
    /// ⭐⭐ **Os três eixos LOCAIS do nó, no mundo, JÁ com a escala dentro** (W133) — e eles são
    /// SEMPRE locais, ao contrário dos de cima.
    ///
    /// ⚠️ **Não é redundância com o [`Anchor::axes`], e a diferença é o sujeito.** Aqueles são os do
    /// gesto e obedecem ao seletor Global/Local, porque mover a peça ao longo do eixo do mundo é um
    /// gesto legítimo. Um **vértice** não tem essa escolha: ele mora no plano do contorno, que é o
    /// XY local, e um seletor de referencial não muda onde o ponto vive. *Ler o campo errado aqui
    /// faria a alça andar num plano e o número mudar noutro.*
    ///
    /// ⭐ **A escala vai DENTRO** porque é o que faz `origem + x·px + y·py` ser a posição de mundo do
    /// vértice `(px, py)` — a mesma conta nos dois sentidos, sem um factor solto a meio.
    pub local: [[f32; 3]; 3],
}

impl Anchor {
    /// Uma âncora nos eixos do mundo — o que a maioria dos gates quer.
    ///
    /// ⚠️ **Deixou de ser `#[cfg(test)]` em 2026-09-08**: a escultura tem-na como
    /// caminho de PRODUTO. Ali a peça não tem escolha de referencial (a
    /// `ph2d_mesh::Pose` dela não tem rotação, e há gate a dizê-lo no
    /// `ClothFilterOrientation::offered`), logo *global* e *local* são os mesmos
    /// três vectores — e um construtor a mais para dizer isso seria uma segunda
    /// resposta à mesma pergunta.
    pub fn global(entity: u64, origin: [f32; 3]) -> Self {
        Self {
            entity,
            origin,
            axes: WORLD_AXES,
            local: WORLD_AXES,
        }
    }
}

/// ⭐⭐⭐ **OS VÉRTICES DA FORMA ESCOLHIDA** (W133) — publicados pela ponte com a cena, que é quem
/// tem o mundo **e** o documento.
///
/// ⚠️ **Só com UM nó escolhido**, e não é uma simplificação: com dois, *de quem são estes pontos?*
/// não tem resposta — e a [`Anchor::origin`] passa a ser o **pivô** da selecção em vez da origem do
/// nó, o que poria o plano do contorno no sítio errado.
#[derive(Clone, Debug, PartialEq)]
pub struct Vertices {
    /// A entidade dona — a mesma da âncora. ⚠️ Ela viaja porque o arrasto **não pode** voltar a
    /// procurá-la: entre a pegada e o largar a selecção pode mudar.
    pub entity: u64,
    /// O índice, na lista de linhas daquela forma, do `x` do primeiro vértice
    /// ([`ph2d_field::vertex_rows`]).
    pub first_row: usize,
    /// Os pontos, em coordenadas **locais** — os mesmos números que o painel mostra.
    pub points: Vec<[f32; 2]>,
}

/// **Em que referencial os eixos do gizmo apontam.**
///
/// ⚠️ A distinção não é cosmética: num nó rodado, `Global` move ao longo dos eixos da cena e `Local`
/// ao longo dos do próprio objeto — e os dois são o gesto certo, em momentos diferentes. O Blender
/// expõe exatamente esta escolha num seletor, e entregar só uma seria escolher por quem não pediu.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Frame {
    #[default]
    Global,
    Local,
}

impl Frame {
    pub const ALL: [Frame; 2] = [Frame::Global, Frame::Local];

    /// ⚠️ Uma **chave** de i18n, nunca um rótulo pronto (HR-15).
    pub fn key(self) -> &'static str {
        match self {
            Frame::Global => "panel.model3d.frame.global",
            Frame::Local => "panel.model3d.frame.local",
        }
    }

    /// Os três eixos deste referencial, no mundo, dada a rotação do nó.
    pub fn axes(self, rotation: [f32; 4]) -> [[f32; 3]; 3] {
        match self {
            Frame::Global => WORLD_AXES,
            Frame::Local => WORLD_AXES.map(|a| ph2d_field::xform::quat_rotate(rotation, a)),
        }
    }
}

/// A forma de uma alça já projetada — em **pixels**, pronta a pintar e a apontar.
#[derive(Clone, Debug)]
pub enum Shape {
    /// Haste (do centro para fora) + ponta.
    Arrow { from: [f32; 2], to: [f32; 2] },
    /// Quadrilátero: os quatro cantos, já projetados.
    Quad([[f32; 2]; 4]),
    /// Disco no centro.
    Disc { center: [f32; 2], radius: f32 },
    /// ⭐ **A metade da frente de uma argola**, como poligonal.
    ///
    /// ⚠️ Só a metade da frente, como no Blender: a de trás está do outro lado da peça, e desenhá-la
    /// faz uma argola vista de lado parecer duas linhas cruzadas em vez de um anel.
    Arc(Vec<[f32; 2]>),
    /// Punho de tamanho: um quadrado no fim de um traço a partir do centro.
    Grip { from: [f32; 2], to: [f32; 2] },
    /// ⭐ **Um ponto do contorno** (W133) — o quadradinho de um vértice.
    ///
    /// ⚠️ **Quadrado e não círculo**, e é a convenção que todo modelador usa para distinguir *um
    /// ponto da malha* de *um punho do gizmo* (o [`Shape::Disc`] já é o disco de vista). Um artista
    /// que veja dois círculos tem de descobrir qual é qual experimentando.
    Point { center: [f32; 2] },
}

/// Uma alça pronta. `live = false` ⇒ **nem pintada nem apontável** neste enquadramento.
#[derive(Clone, Debug)]
pub struct Projected {
    pub handle: Handle,
    pub shape: Shape,
    pub live: bool,
}

/// ⭐ **A lei do arrasto** vive no irmão — ver [`field3d_gizmo_drag`](self::drag_law).
///
/// ⚠️ **O re-export é o que mantém os caminhos antigos vivos**: `ph2d_viewport3d::gizmo::Motion`,
/// `::drag`, `::snap_step` e as duas constantes de passo continuam a resolver, e nenhum chamador
/// mudou uma linha. Um corte que obrigasse a reescrever 40 sítios seria um corte a cobrar o preço
/// errado.
#[path = "gizmo_drag.rs"]
mod drag_law;
pub use drag_law::{Motion, drag, snap_step};

/// ⭐⭐ **A projecção das ALÇAS DE VÉRTICE** vive no irmão — ver
/// [`field3d_vertex_handles`](self::vertex_handles).
///
/// ⚠️ O re-export mantém `ph2d_viewport3d::gizmo::project_vertices` — cortar um arquivo não pode custar uma
/// reescrita a cada chamador.
#[path = "vertex_handles.rs"]
mod vertex_handles;
pub use vertex_handles::project_vertices;

/// ⭐ **A metade que APONTA** vive no irmão — ver [`field3d_gizmo_pick`](self::pick_law).
#[path = "gizmo_pick.rs"]
mod pick_law;
pub use pick_law::pick;

/// ⭐⭐⭐ **O QUE ESTA LEI PRECISA DE SABER SOBRE UMA CÂMERA** — três perguntas, e
/// mais nenhuma.
///
/// ⚠️ **Ele nasceu porque a lei ganhou um SEGUNDO consumidor** (2026-09-08, ordem
/// do Enio: *«traga esses features para esse módulo»*): a escultura tem a
/// [`ph2d_mesh_render::Camera3d`] e este módulo nasceu contra a [`Orbit`]. Tudo
/// o que a projecção das alças faz com uma câmera são estas três coisas — onde
/// um ponto do mundo cai, quantos pixels vale ali uma unidade de mundo, e para
/// onde o observador está.
///
/// ⛔ *Copiar as ~130 linhas de projecção para o outro módulo daria duas ideias
/// de onde uma alça está, e o sintoma da que envelhecesse seria um gizmo que
/// agarra ao lado do que ele diz mover — a espécie de defeito que ninguém chama
/// de defeito de projecção.*
pub trait GizmoCamera {
    /// Onde este ponto do mundo cai, em pixels — `None` se ele não tem pixel.
    fn project_px(&self, p: [f32; 3]) -> Option<[f32; 2]>;
    /// Quantos pixels vale uma unidade de mundo **naquele ponto**.
    ///
    /// ⚠️ *Naquele ponto*, e não no quadro: com a lente convergente uma unidade
    /// mede menos pixels quanto mais longe está, e um braço dimensionado pela
    /// constante do quadro encolheria com a peça a afastar-se.
    fn px_per_world(&self, at: [f32; 3]) -> f32;
    /// A direcção para o OBSERVADOR, unitária.
    fn fwd(&self) -> [f32; 3];
}

/// A [`Orbit`] deste módulo, no vocabulário do [`GizmoCamera`].
struct OrbitCam<'a>(&'a Orbit, Screen);

impl GizmoCamera for OrbitCam<'_> {
    fn project_px(&self, p: [f32; 3]) -> Option<[f32; 2]> {
        self.0.project(p, self.1).map(|(px, _)| px)
    }
    fn px_per_world(&self, at: [f32; 3]) -> f32 {
        self.0.px_per_world_at(at, self.1)
    }
    fn fwd(&self) -> [f32; 3] {
        self.0.basis().2
    }
}

/// Os três eixos do mundo.
const WORLD_AXES: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// **Projeta o gizmo inteiro**, no modo dado. A ordem é a de apontar: do centro para fora.
///
/// ⚠️ **A ordem é load-bearing** — [`pick`] devolve a primeira que casa, e o disco de vista está por
/// dentro da folga onde as setas não começam. Sem esta ordem, apontar o centro escolheria um eixo à
/// sorte.
pub fn project(anchor: Anchor, cam: &Orbit, screen: Screen, mode: Mode) -> Vec<Projected> {
    project_with(anchor, &OrbitCam(cam, screen), mode)
}

/// ⭐⭐⭐ **O MESMO, PARA QUALQUER CÂMERA** — ver [`GizmoCamera`].
pub fn project_with(anchor: Anchor, cam: &dyn GizmoCamera, mode: Mode) -> Vec<Projected> {
    // ⭐ **A escala é a DAQUELE ponto**, e não a do quadro: com a lente convergente uma unidade de
    // mundo mede menos pixels quanto mais longe está. Um braço dimensionado pela constante do quadro
    // encolheria com a peça a afastar-se, e as alças deixariam de medir o que dizem medir.
    let px_per_world = cam.px_per_world(anchor.origin).max(f32::MIN_POSITIVE);
    let arm = ARM_PX / px_per_world;
    // ⚠️ **Sem projeção não há gizmo**: a âncora está ao lado do olho ou atrás dele, e desenhar
    // alças num pixel inventado seria oferecer um gesto que agarra noutro sítio.
    let Some(o2) = cam.project_px(anchor.origin) else {
        return Vec::new();
    };
    match mode {
        Mode::Move => move_handles(anchor, cam, arm, o2),
        Mode::Rotate => rotate_handles(anchor, cam, arm),
        Mode::Scale => vec![Projected {
            handle: Handle::Grip,
            shape: Shape::Grip {
                from: o2,
                to: [o2[0] + GRIP_DIR[0] * ARM_PX, o2[1] + GRIP_DIR[1] * ARM_PX],
            },
            // Um punho de TELA não tem como degenerar: ele não é uma direção do mundo.
            live: true,
        }],
    }
}

fn move_handles(anchor: Anchor, cam: &dyn GizmoCamera, arm: f32, o2: [f32; 2]) -> Vec<Projected> {
    let mut out = vec![Projected {
        handle: Handle::View,
        shape: Shape::Disc {
            center: o2,
            radius: INNER_PX,
        },
        // O plano da tela nunca fica de perfil consigo mesmo: esta alça é a única que não pode
        // degenerar, e é por isso que ela é a rede de segurança do enquadramento difícil.
        live: true,
    }];

    for n in 0..3 {
        let (u, v) = ((n + 1) % 3, (n + 2) % 3);
        let corner = |a: f32, b: f32| -> Option<[f32; 2]> {
            let mut p = anchor.origin;
            for (k, c) in p.iter_mut().enumerate() {
                *c += anchor.axes[u][k] * a * arm + anchor.axes[v][k] * b * arm;
            }
            cam.project_px(p)
        };
        let (lo, hi) = (PLANE_AT, PLANE_AT + PLANE_SIDE);
        // ⚠️ **Um canto sem projeção mata a alça inteira**, e não só ele: um quadrilátero com três
        // cantos é uma forma que o teste de acerto aceitaria e o olho não reconhece.
        let corners = [
            corner(lo, lo),
            corner(hi, lo),
            corner(hi, hi),
            corner(lo, hi),
        ];
        let quad = match corners {
            [Some(a), Some(b), Some(c), Some(d)] => [a, b, c, d],
            _ => [[0.0; 2]; 4],
        };
        let projects = corners.iter().all(Option::is_some);
        // ⚠️ **De perfil, um quadrado é um traço.** A pergunta certa não é a área: é se ele ainda é
        // largo o bastante para se apontar — o lado mais estreito tem de passar do raio de agarre.
        let narrow = (0..4)
            .map(|i| dist(quad[i], quad[(i + 1) % 4]))
            .fold(f32::INFINITY, f32::min);
        out.push(Projected {
            handle: Handle::Plane(n),
            shape: Shape::Quad(quad),
            live: projects && narrow >= GRAB_PX,
        });
    }

    for (n, axis) in anchor.axes.iter().enumerate() {
        // Uma ponta sem projeção é uma seta que aponta para fora do mundo visível: ela não é
        // desenhada e não é oferecida, pelo mesmo `live` que já trata a seta vista de topo.
        let tip = cam.project_px(offset(anchor.origin, *axis, arm));
        let len = tip.map_or(0.0, |t| dist(o2, t));
        out.push(Projected {
            handle: Handle::Axis(n),
            shape: Shape::Arrow {
                from: o2,
                to: tip.unwrap_or(o2),
            },
            live: tip.is_some() && len >= MIN_ARM_PX,
        });
    }
    out
}

fn rotate_handles(anchor: Anchor, cam: &dyn GizmoCamera, arm: f32) -> Vec<Projected> {
    let fwd = cam.fwd();
    let mut out = Vec::with_capacity(4);
    for (n, axis) in anchor.axes.iter().enumerate() {
        out.push(Projected {
            handle: Handle::Ring(n),
            shape: Shape::Arc(front_arc(anchor.origin, *axis, arm, cam)),
            live: dot(*axis, fwd).abs() >= RING_MIN_DOT,
        });
    }
    // ⭐ A argola de VISTA fica por fora e é a única que não pode ficar de perfil consigo mesma —
    // a rede de segurança do enquadramento difícil, como o disco no modo de mover.
    out.push(Projected {
        handle: Handle::ViewRing,
        shape: Shape::Arc(front_arc(anchor.origin, fwd, arm * VIEW_RING_R, cam)),
        live: true,
    });
    out
}

/// A **metade da frente** de um círculo do mundo, projetada — ou o círculo inteiro quando ele está
/// de frente para a câmera.
///
/// ⚠️ A metade da frente é um trecho **contíguo** do círculo (um plano corta uma circunferência em
/// exatamente dois pontos), mas ele pode dar a volta ao fim do vetor de amostras. Por isso a
/// travessia começa onde o trecho começa, e não no índice zero — cortar em zero partiria a argola em
/// duas no meio da tela.
fn front_arc(
    origin: [f32; 3],
    axis: [f32; 3],
    radius: f32,
    cam: &dyn GizmoCamera,
) -> Vec<[f32; 2]> {
    let (u, v) = basis_of(axis);
    let fwd = cam.fwd();
    let world = |i: usize| -> [f32; 3] {
        let t = i as f32 / RING_SEGMENTS as f32 * std::f32::consts::TAU;
        let (s, c) = t.sin_cos();
        let mut p = origin;
        for k in 0..3 {
            p[k] += (u[k] * c + v[k] * s) * radius;
        }
        p
    };
    // ⚠️ **A profundidade sai do DESLOCAMENTO, nunca de `ponto − origem`.** A subtração cancela dois
    // números grandes e o erro que sobra é da ordem da própria origem — numa peça longe do zero, o
    // sinal da conta passa a ser ruído. Aqui o deslocamento já é o que se quer, e o erro fica da
    // ordem de `radius · 10⁻⁷`.
    let (du, dv) = (dot(u, fwd), dot(v, fwd));
    // ⚠️ **Não ter projeção CONTA como não estar à frente.** Com a lente convergente uma argola pode
    // atravessar o plano do olho, e um ponto de lá não tem pixel nenhum: tratá-lo como frente
    // deixaria um salto no meio da fita. A pergunta *"este ponto é desenhável?"* tem uma resposta,
    // e é ela que entra na máscara — em vez de dois testes que podem discordar.
    let at = |i: usize| cam.project_px(world(i));
    let front: Vec<bool> = (0..RING_SEGMENTS)
        .map(|i| {
            let t = i as f32 / RING_SEGMENTS as f32 * std::f32::consts::TAU;
            let (s, c) = t.sin_cos();
            c.mul_add(du, s * dv) >= -RING_FRONT_EPS && at(i).is_some()
        })
        .collect();
    if front.iter().all(|f| *f) {
        return (0..=RING_SEGMENTS)
            .filter_map(|i| at(i % RING_SEGMENTS))
            .collect();
    }
    let Some(start) =
        (0..RING_SEGMENTS).find(|&i| front[i] && !front[(i + RING_SEGMENTS - 1) % RING_SEGMENTS])
    else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for k in 0..RING_SEGMENTS {
        let i = (start + k) % RING_SEGMENTS;
        if !front[i] {
            break;
        }
        if let Some(px) = at(i) {
            out.push(px);
        }
    }
    out
}

#[cfg(test)]
#[path = "gizmo_tests.rs"]
mod tests;

/// Dois vetores unitários que geram o plano perpendicular a `axis`.
///
/// ⚠️ O parceiro do produto vetorial é escolhido pelo **eixo menos alinhado** com `axis`. Um
/// parceiro fixo daria produto nulo exatamente quando `axis` fosse ele — e o sintoma seria uma
/// argola que desaparece num dos três eixos, e só nele.
fn basis_of(axis: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let a = normalize(axis);
    let small = (0..3)
        .min_by(|&i, &j| a[i].abs().total_cmp(&a[j].abs()))
        .unwrap_or(0);
    let mut helper = [0.0f32; 3];
    helper[small] = 1.0;
    let u = normalize(cross(a, helper));
    (u, cross(a, u))
}

fn offset(p: [f32; 3], dir: [f32; 3], k: f32) -> [f32; 3] {
    [p[0] + dir[0] * k, p[1] + dir[1] * k, p[2] + dir[2] * k]
}

fn len3(v: [f32; 3]) -> f32 {
    dot(v, v).sqrt()
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let n = len3(v);
    if n <= 0.0 || !n.is_finite() {
        return [0.0, 0.0, 1.0];
    }
    [v[0] / n, v[1] / n, v[2] / n]
}

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}
