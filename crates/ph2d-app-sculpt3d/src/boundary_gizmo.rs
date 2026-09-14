//! **O INDICADOR DO PINCEL DE CONTORNO** — o troço da beirada que vai dobrar e
//! até onde a dobra entra, vistos **antes** de a mão premir.
//!
//! Irmão do [`super::pose_gizmo`], e pela mesma razão com outro sujeito: ⚠️ **a
//! região deste verbo também não sai do cursor**. Ela sai da **BORDA** — o
//! cursor só escolhe onde a beirada é agarrada —, então o anel sozinho promete
//! um círculo onde a ferramenta pensa numa **fita de beirada mais uma faixa
//! para dentro**.
//!
//! # A figura, e porque são DUAS coisas
//!
//! O painel deste pincel faz duas perguntas que o anel não sabe responder, e a
//! figura responde a uma cada:
//!
//! | o que o artista mexe | o que ele vê mudar |
//! |---|---|
//! | `Falloff along the edge` | **quanto da boca** entra — a fita desenhada ao longo da beirada, com cada pedaço mais forte quanto mais ele se move |
//! | `Origin offset` | **quão fundo** a dobra entra — a linha que mergulha na peça, com o anel no eixo |
//!
//! ⭐ **Um pedaço de peso zero não é desenhado**, e isso é a informação: ali a
//! beirada **não se move**. *Desenhar a boca toda com a mesma força diria que
//! ela toda entra, que é precisamente o que o selector muda.*
//!
//! # ⚠️ Os dois estados que ele distingue
//!
//! | o que se vê | o que significa |
//! |---|---|
//! | a fita + a linha com o anel no fundo | a região que o pen-down vai construir |
//! | **só** um anel apagado sobre o cursor | ⛔ **não há beirada ao alcance** — numa peça fechada, ou longe da boca, este gesto **não move um único vértice** |
//!
//! # ⚠️ Ele é traçado sob `Affine::IDENTITY`
//!
//! No Vello o transform de um `stroke` **multiplica a largura** — a mesma cerca
//! que o [`super::cursor`] e o [`super::pose_gizmo`] escrevem. Os caminhos que
//! saem daqui já estão em **pixels de janela**.

use ph2d_sculpt3d::Verb;
use ph2d_vector::BezPath;

use super::Sculpt3dScene;

/// A fita da beirada, na mesma família âmbar do anel do cursor e **mais apagada
/// do que ele** pelo motivo do irmão: o anel é onde a mão está *agora*.
///
/// ⚠️ A componente alfa é o **tecto**: cada pedaço leva-a multiplicada pelo
/// peso dele, que é o que faz a figura mostrar *quanto* da boca entra.
pub const BOUNDARY_EDGE_RGBA: [f32; 4] = [0.98, 0.83, 0.36, 0.85];
/// A linha da profundidade — mais discreta que a fita: ela é a segunda
/// pergunta, e competir com a primeira faria as duas ilegíveis.
pub const BOUNDARY_DEPTH_RGBA: [f32; 4] = [0.98, 0.83, 0.36, 0.45];
/// O **eixo** — o ponto-origem, em torno do qual a lei roda. Cheio, porque é o
/// que o `Origin offset` move.
pub const BOUNDARY_PIVOT_RGBA: [f32; 4] = [0.98, 0.83, 0.36, 0.95];
/// O aviso: **não há beirada ao alcance daqui**.
pub const BOUNDARY_INERT_RGBA: [f32; 4] = [0.98, 0.45, 0.36, 0.85];

/// O raio do anel do eixo, em pixels — o mesmo da dobradiça da pose, porque é a
/// mesma coisa dita sobre outro verbo: *o ponto em torno do qual tudo roda*.
const PIVO_PX: f64 = 6.0;
/// Quantos segmentos aproximam o anel. Pequeno, logo `16` não tem canto visível.
const ANEL_SEGS: usize = 16;
/// ⛔ **Abaixo disto o pedaço não é desenhado.** Ele nomeia um recurso: o traço
/// do overlay tem `1,5 px` e a alfa da fita é `0,85`, logo um peso abaixo de
/// `~1/255` não pinta um único pixel diferente do fundo — desenhá-lo seria
/// gastar um caminho para não mudar a imagem.
const PESO_MINIMO: f32 = 0.004;

/// O que o quadro desenha: a fita da beirada, a linha da profundidade e o eixo.
pub struct BoundaryGizmo {
    /// Um pedaço da beirada por par de vértices, com o **peso** dele.
    pub borda: Vec<(BezPath, f32)>,
    /// A linha da âncora ao eixo — quão fundo a dobra entra.
    pub profundidade: Option<BezPath>,
    /// O anel no eixo.
    pub pivo: Option<BezPath>,
    /// ⛔ Não há beirada ao alcance: este gesto não move nada.
    pub inerte: bool,
}

impl Sculpt3dScene {
    /// O indicador do pincel de contorno para um cursor em `(x, y)` de tela.
    ///
    /// `None` quando o verbo na mão não é o de contorno, quando o barro não
    /// está na tela, ou quando não há peça sob o cursor nem gesto em curso.
    ///
    /// ⚠️ **`&mut self` é a cache**, não um efeito escondido: a estrutura
    /// guardada vive no traço ([`ph2d_sculpt3d::boundary_previa`]) e é ela que
    /// impede o indicador de refazer o censo de bordas a cada movimento do rato.
    pub fn boundary_gizmo(&mut self, x: f32, y: f32) -> Option<BoundaryGizmo> {
        let (i, centro) = self.boundary_gizmo_alvo(x, y)?;
        let pose = self.objects.get(i)?.pose;
        let brush = self.armed_brush_on(pose, centro);
        let sym = self.symmetry;
        // ⚠️ Os dois empréstimos são de CAMPOS diferentes da cena — a malha vem
        // de `objects` e a cache vem de `stroke`. É a razão do irmão.
        let malha = self.objects.get(i)?.stack.mesh();
        let previa = self.stroke.boundary_contorno(malha, &brush, sym, centro);
        let trechos: Vec<ph2d_sculpt3d::TrechoDaBorda> = previa.borda().to_vec();
        let linha = previa.profundidade();

        let mut figura = BoundaryGizmo {
            borda: Vec::new(),
            profundidade: None,
            pivo: None,
            inerte: false,
        };
        if trechos.is_empty() {
            // ⛔ O aviso: um anel sobre o próprio cursor e mais nada. *Uma
            // figura vazia e um indicador partido leem-se igual; este diz que a
            // ferramenta não tem onde pegar.*
            let at = pose.point_to_world(centro);
            let (cx, cy) = self.project_window(at)?;
            figura.pivo = Some(anel(f64::from(cx), f64::from(cy), PIVO_PX));
            figura.inerte = true;
            return Some(figura);
        }

        for t in &trechos {
            if t.peso < PESO_MINIMO {
                continue;
            }
            let a = self.project_window(pose.point_to_world(t.a))?;
            let b = self.project_window(pose.point_to_world(t.b))?;
            let mut caminho = BezPath::new();
            caminho.move_to(ph2d_vector::Point::new(f64::from(a.0), f64::from(a.1)));
            caminho.line_to(ph2d_vector::Point::new(f64::from(b.0), f64::from(b.1)));
            figura.borda.push((caminho, t.peso));
        }
        if let Some([ancora, origem]) = linha {
            let a = self.project_window(pose.point_to_world(ancora))?;
            let o = self.project_window(pose.point_to_world(origem))?;
            let mut caminho = BezPath::new();
            caminho.move_to(ph2d_vector::Point::new(f64::from(a.0), f64::from(a.1)));
            caminho.line_to(ph2d_vector::Point::new(f64::from(o.0), f64::from(o.1)));
            figura.profundidade = Some(caminho);
            figura.pivo = Some(anel(f64::from(o.0), f64::from(o.1), PIVO_PX));
        }
        Some(figura)
    }

    /// **Sobre que peça e sobre que ponto dela a figura é desenhada.**
    ///
    /// ⚠️⚠️ **`&self`, e isso é uma PROVA e não arrumação** — a mesma razão que
    /// o [`super::pose_gizmo::Sculpt3dScene::pose_gizmo_alvo`] escreve: o gate
    /// `a_stroke_belongs_to_the_piece_it_started_on` conta os consumidores de
    /// `pick` e exige que só o `aim` mova a peça activa. Um indicador com
    /// `&mut self` seria capaz de trocar de peça a meio de uma pincelada.
    fn boundary_gizmo_alvo(&self, x: f32, y: f32) -> Option<(usize, [f32; 3])> {
        if !self.shows_clay() || self.brush.verb != Verb::Boundary {
            return None;
        }
        match self.pick(x, y) {
            Some((i, hit)) => Some((i, hit.point)),
            // `grab` é o pen-down dos gestos que PUXAM, e o de contorno é um
            // deles: a meio de um arrasto o cursor sai da malha com frequência,
            // e ali a figura tem de continuar desenhada.
            None if self.grab.is_some() => Some((self.active, [0.0; 3])),
            None => None,
        }
    }
}

/// Um anel de `r` pixels em `(cx, cy)`.
fn anel(cx: f64, cy: f64, r: f64) -> BezPath {
    let mut caminho = BezPath::new();
    for i in 0..=ANEL_SEGS {
        let a = (i as f64) * std::f64::consts::TAU / (ANEL_SEGS as f64);
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
