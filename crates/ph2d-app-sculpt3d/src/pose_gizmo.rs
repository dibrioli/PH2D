//! **O OSSO DO PINCEL DE POSE** — o indicador que mostra onde está a dobradiça
//! e até onde vai o membro, **antes** de a mão premir.
//!
//! Irmão do [`super::cursor`] e o corte é o SUJEITO: aquele responde *onde o
//! gesto vai pousar*, este *o que o gesto vai dobrar*. São perguntas diferentes
//! porque este verbo é diferente de todos os outros — ⚠️ **ele não tem
//! atenuação radial**, e um vértice a dez raios do cursor pode mover-se por
//! inteiro porque a região cresce pela **ligação** da malha. ⇒ *o anel do
//! cursor, sozinho, descreve mal este pincel: ele mostra um círculo onde a
//! ferramenta pensa num membro.*
//!
//! # A figura, e porque é um OSSO e não uma linha
//!
//! Uma linha da dobradiça ao dedo diz *onde*, e não diz **de que lado está a
//! articulação** — que é a única coisa que o artista precisa de saber antes de
//! arrastar. A silhueta octaédrica de uma armadura diz as duas: ela é **larga
//! junto da dobradiça e afila para a mão**, e por isso lê-se num relance.
//!
//! ⚠️ **Com mais de um segmento a cadeia dobra como um braço**, e cada segmento
//! ganha o seu osso — é assim que o controlo *Segments* deixa de ser um número
//! abstracto e passa a ser uma coisa que se vê.
//!
//! # ⚠️ Os dois estados que ele distingue
//!
//! | o que se vê | o que significa |
//! |---|---|
//! | os ossos + o anel maior na ponta mais funda | a cadeia que o pen-down vai construir |
//! | **só** um anel apagado sobre o cursor | ⭐ **§11.1**: o pivô caiu em cima do cursor e este gesto **não move nada** — o alvo cala-se aqui, e o artista arrasta e não acontece coisa nenhuma |
//!
//! # ⚠️ Ele é traçado sob `Affine::IDENTITY`
//!
//! No Vello o transform de um `stroke` **multiplica a largura** — a mesma cerca
//! que o [`super::cursor`] escreve, e que transformou o realce do Flip num
//! borrão. Os caminhos que saem daqui já estão em **pixels de janela**.

use ph2d_sculpt3d::{PoseOsso, Verb};
use ph2d_vector::BezPath;

use super::Sculpt3dScene;

/// O osso, na mesma família âmbar do anel do cursor — ⚠️ **mais apagado do que
/// ele de propósito**: o anel é onde a mão está *agora* e o osso é o que vai
/// acontecer *a seguir*; com a mesma intensidade os dois competem, e o que se
/// perde é a mira.
pub const POSE_BONE_RGBA: [f32; 4] = [0.98, 0.83, 0.36, 0.55];
/// A **dobradiça** — a ponta mais funda da cadeia, que é a coisa que este
/// pincel tem e nenhum outro tem. Cheia, para ganhar o olho.
pub const POSE_PIVOT_RGBA: [f32; 4] = [0.98, 0.83, 0.36, 0.95];
/// O aviso do §11.1: **não há dobradiça aqui**.
pub const POSE_INERT_RGBA: [f32; 4] = [0.98, 0.45, 0.36, 0.85];

/// Onde o ombro do osso fica, em fracção do comprimento a contar da dobradiça.
///
/// ⚠️ **É a proporção de uma armadura** (o octaedro é mais largo junto da
/// raiz), e é ela que faz a figura dizer **de que lado** está a articulação sem
/// precisar de uma seta.
const OMBRO_FRAC: f64 = 0.15;
/// Metade da largura do ombro, em fracção do comprimento.
const CINTURA_FRAC: f64 = 0.10;
/// ⚠️ **O piso da cintura, em pixels, e ele nomeia um recurso:** o traço do
/// overlay tem `1,5 px`, logo abaixo de `2 px` os dois flancos do osso pousam
/// na mesma coluna de pixels e a figura deixa de ser um osso — passa a ser a
/// linha que este módulo existe para não desenhar.
const CINTURA_MIN_PX: f64 = 2.0;
/// O raio do anel de uma junta intermédia, em pixels.
const JUNTA_PX: f64 = 3.0;
/// O raio do anel da **dobradiça** — maior, porque é a informação principal.
const PIVO_PX: f64 = 6.0;
/// Quantos segmentos aproximam um anel de junta. Eles são pequenos (≤ `6 px`),
/// então `16` já não tem canto visível — ⛔ e não são os `48` do anel do
/// cursor, que é até `1/8` da altura do ecrã.
const JUNTA_SEGS: usize = 16;

/// O que o quadro desenha: os ossos, as juntas, e o veredito do §11.1.
pub struct PoseGizmo {
    /// Uma silhueta por segmento, da dobradiça para a mão.
    pub ossos: Vec<BezPath>,
    /// Os anéis das juntas, **sem** o da dobradiça.
    pub juntas: Vec<BezPath>,
    /// O anel da dobradiça — o ponto em torno do qual tudo vai rodar.
    pub pivo: Option<BezPath>,
    /// ⭐ **§11.1** — o pivô caiu em cima do cursor e o gesto não move nada.
    pub inerte: bool,
}

impl Sculpt3dScene {
    /// O indicador do pincel de pose para um cursor em `(x, y)` de tela.
    ///
    /// `None` quando o verbo na mão não é o de pose, quando o barro não está na
    /// tela, ou quando não há peça sob o cursor nem gesto em curso.
    ///
    /// ⚠️ **`&mut self` é a cache**, não um efeito colateral escondido: a
    /// cadeia guardada vive no traço (ver [`ph2d_sculpt3d::pose_previa`]), e é
    /// ela que impede este indicador de reconstruir a malha inteira a cada
    /// movimento do rato — que é a queixa pública registada contra o alvo.
    pub fn pose_gizmo(&mut self, x: f32, y: f32) -> Option<PoseGizmo> {
        let (i, centro) = self.pose_gizmo_alvo(x, y)?;
        let pose = self.objects.get(i)?.pose;
        let brush = self.armed_brush_on(pose, centro);
        let sym = self.symmetry;
        // ⚠️ **Os dois empréstimos são de CAMPOS diferentes da cena** — a malha
        // vem de `objects` e a cache vem de `stroke` —, e é por isso que este
        // bloco não pode passar por um método de `&mut self`: um método pede a
        // cena inteira, e a malha que ele recebia deixaria de ser emprestável.
        let malha = self.objects.get(i)?.stack.mesh();
        let ossos: Vec<PoseOsso> = self.stroke.pose_ossos(malha, &brush, sym, centro).to_vec();
        let inerte = self.stroke.pose_previa_inerte();

        let mut figura = PoseGizmo {
            ossos: Vec::new(),
            juntas: Vec::new(),
            pivo: None,
            inerte,
        };
        if ossos.is_empty() || inerte {
            // ⭐ O aviso: um anel sobre o próprio cursor, e nada de osso. *Uma
            // figura que desenhasse um osso de comprimento zero diria que há
            // uma dobradiça ali — que é precisamente o contrário do facto.*
            let at = pose.point_to_world(centro);
            let (cx, cy) = self.project_window(at)?;
            figura.pivo = Some(anel(f64::from(cx), f64::from(cy), PIVO_PX));
            figura.inerte = true;
            return Some(figura);
        }

        let mut ultima_origem = None;
        for (k, osso) in ossos.iter().enumerate() {
            let a = self.project_window(pose.point_to_world(osso.origem))?;
            let b = self.project_window(pose.point_to_world(osso.cabeca))?;
            let (ax, ay) = (f64::from(a.0), f64::from(a.1));
            let (bx, by) = (f64::from(b.0), f64::from(b.1));
            if let Some(silhueta) = silhueta_do_osso(ax, ay, bx, by) {
                figura.ossos.push(silhueta);
            }
            // A junta entre dois segmentos é a origem deste, que é a cabeça do
            // seguinte; a do índice `0` é a **mão** e não se desenha (o anel do
            // cursor já lá está).
            if k + 1 < ossos.len() {
                figura.juntas.push(anel(ax, ay, JUNTA_PX));
            }
            ultima_origem = Some((ax, ay));
        }
        figura.pivo = ultima_origem.map(|(px, py)| anel(px, py, PIVO_PX));
        Some(figura)
    }

    /// **Sobre que peça e sobre que ponto dela o osso é desenhado.**
    ///
    /// ⚠️ **A peça é a que está SOB O CURSOR ao sobrevoar, e a ACTIVA durante o
    /// gesto.** Não é o mesmo: o `aim` do pen-down é que promove a peça apontada
    /// a activa, e a meio de um arrasto o cursor sai da malha com frequência —
    /// ali o osso tem de continuar a ser desenhado, senão ele pisca exactamente
    /// quando está a ser útil.
    ///
    /// ⚠️⚠️ **`&self`, e isso é uma PROVA e não arrumação.** O gate
    /// `a_stroke_belongs_to_the_piece_it_started_on` afirma que só o `aim` move
    /// a peça activa a partir de um `pick`, e que todo outro consumidor é
    /// somente-leitura — *e o que o torna somente-leitura é o compilador, não
    /// uma linha de texto*. O indicador precisa de `&mut self` (a cache vive no
    /// traço), então a escolha da peça sai para aqui: assim o `pick` continua
    /// atrás de uma fronteira que o compilador guarda. ⛔ Fundi-la de volta no
    /// chamador devolve a terceira consulta da lista a um `&mut self`.
    fn pose_gizmo_alvo(&self, x: f32, y: f32) -> Option<(usize, [f32; 3])> {
        if !self.shows_clay() || self.brush.verb != Verb::Pose {
            return None;
        }
        match self.pick(x, y) {
            Some((i, hit)) => Some((i, hit.point)),
            // `grab` é o pen-down dos gestos que PUXAM, e o de pose é um deles.
            None if self.grab.is_some() => Some((self.active, [0.0; 3])),
            None => None,
        }
    }
}

/// A silhueta octaédrica de um osso, em pixels de janela.
///
/// `None` quando os dois pontos caem praticamente no mesmo pixel — ali não há
/// direcção, e desenhar um losango de uma escolha arbitrária de normal seria
/// inventar uma articulação que a cadeia não tem.
fn silhueta_do_osso(ax: f64, ay: f64, bx: f64, by: f64) -> Option<BezPath> {
    let (dx, dy) = (bx - ax, by - ay);
    let comprimento = dx.hypot(dy);
    if !comprimento.is_finite() || comprimento < CINTURA_MIN_PX * 2.0 {
        return None;
    }
    let (ux, uy) = (dx / comprimento, dy / comprimento);
    // A normal do plano da tela — o osso é uma figura 2D sobre a projecção, e
    // não um sólido: um sólido pediria profundidade que o overlay não tem.
    let (nx, ny) = (-uy, ux);
    let cintura = (comprimento * CINTURA_FRAC).max(CINTURA_MIN_PX);
    let ombro = comprimento * OMBRO_FRAC;
    let (ox, oy) = (ax + ux * ombro, ay + uy * ombro);
    let mut caminho = BezPath::new();
    caminho.move_to(ph2d_vector::Point::new(ax, ay));
    caminho.line_to(ph2d_vector::Point::new(
        ox + nx * cintura,
        oy + ny * cintura,
    ));
    caminho.line_to(ph2d_vector::Point::new(bx, by));
    caminho.line_to(ph2d_vector::Point::new(
        ox - nx * cintura,
        oy - ny * cintura,
    ));
    caminho.close_path();
    Some(caminho)
}

/// Um anel de `r` pixels em `(cx, cy)`.
fn anel(cx: f64, cy: f64, r: f64) -> BezPath {
    let mut caminho = BezPath::new();
    for i in 0..=JUNTA_SEGS {
        let a = (i as f64) * std::f64::consts::TAU / (JUNTA_SEGS as f64);
        let p = ph2d_vector::Point::new(cx + r * a.cos(), cy + r * a.sin());
        if i == 0 {
            caminho.move_to(p);
        } else {
            caminho.line_to(p);
        }
    }
    caminho.close_path();
    caminho
}

/// Os gates da FIGURA — ver [`pose_gizmo_tests`].
#[cfg(test)]
#[path = "pose_gizmo_tests.rs"]
mod pose_gizmo_tests;
