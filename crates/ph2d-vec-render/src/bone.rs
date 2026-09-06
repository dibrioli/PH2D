//! Desenho dos **OSSOS** (estudo 42 item 5) — módulo irmão do overlay da gaiola (`envelope.rs`),
//! teto de LOC.
//!
//! Um osso desenha-se como o **losango afilado** que toda ferramenta de rig usa (Spine, Moho, Rive,
//! Blender): largo na raiz, agudo na ponta. Não é decoração — a forma **diz a direcção**, que é a
//! única coisa que um segmento não diz e de que o artista precisa para saber para que lado o filho
//! sai.
//!
//! ⚠️ **A LARGURA é em píxeis de tela e o COMPRIMENTO é em mundo.** Um osso curto num zoom afastado
//! ficaria uma linha invisível se a espessura escalasse; e um losango de largura fixa em mundo
//! engoliria o desenho ao aproximar. É a mesma gramática das bolinhas do `envelope.rs`: *o ponto
//! sobe pelo afim, a espessura não*.
//!
//! ⛔ **Um osso NÃO é um `VecPath`** — ele não tem tinta, não exporta para SVG e não entra na cena
//! vectorial. O que se vê é isto, e só enquanto a ferramenta está na mão.

use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{
    Affine, BezPath, Brush, Circle, Color as VelloColor, Fill, Point, Stroke, VectorScene,
};

/// Metade da largura do losango na raiz, em píxeis.
const BONE_HALF_PX: f64 = 4.0;

/// Raio da bolinha da JUNTA, em píxeis — **e o mesmo alcance que o hit-test do host usa**
/// (`× px_to_world`), como o `ENVELOPE_HANDLE_R_PX` do irmão.
///
/// ⚠️ **Dois números fariam o dedo pegar a junta e a bolinha acender noutro sítio** — e aqui seria
/// pior que uma bolinha errada: a junta e o corpo executam VERBOS diferentes (deslocar × girar), e o
/// artista veria o osso andar quando queria girá-lo.
pub const BONE_JOINT_R_PX: f64 = 6.0;

/// Espessura do contorno, em píxeis.
const LINE_PX: f64 = 1.25;

/// **Desenha os ossos** `(bits, origem, ponta)` em MUNDO. `selected` acende um deles.
///
/// ⚠️ **A cor NÃO é o estado da selecção sozinha**: o seleccionado vem `Accent` **cheio** e os
/// outros `AccentSoft` **vazados**, que é a mesma gramática do `envelope.rs` (forma + preenchimento
/// carregam o estado) — assim lê-se qual está aceso sem depender de distinguir dois tons.
pub fn draw_bones(
    bones: &[(u64, [f64; 2], [f64; 2])],
    selected: Option<u64>,
    transform: Affine,
    theme: Theme,
    target: &mut VectorScene,
) {
    let vello = |t: ColorToken| {
        let c = t.resolve(theme);
        VelloColor::from_rgba8(c.r, c.g, c.b, c.a)
    };
    let (aceso, apagado) = (vello(ColorToken::Accent), vello(ColorToken::AccentSoft));
    for &(bits, a, b) in bones {
        let (pa, pb) = (
            transform * Point::new(a[0], a[1]),
            transform * Point::new(b[0], b[1]),
        );
        let (dx, dy) = (pb.x - pa.x, pb.y - pa.y);
        let comp = dx.hypot(dy);
        if comp <= f64::EPSILON {
            continue;
        }
        // A perpendicular unitária, em TELA — é ela que dá a largura constante em píxeis.
        let (nx, ny) = (-dy / comp, dx / comp);
        // O ombro do losango fica a um quarto do caminho: é o que faz a silhueta ler como uma seta
        // e não como um triângulo, e é a proporção que as três referências usam.
        let ombro = Point::new(pa.x + dx * 0.25, pa.y + dy * 0.25);
        let w = BONE_HALF_PX.min(comp * 0.25);
        let mut p = BezPath::new();
        p.move_to(pa);
        p.line_to(Point::new(ombro.x + nx * w, ombro.y + ny * w));
        p.line_to(pb);
        p.line_to(Point::new(ombro.x - nx * w, ombro.y - ny * w));
        p.close_path();
        let sel = Some(bits) == selected;
        if sel {
            target.inner_mut().fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(aceso),
                None,
                &p,
            );
        }
        target.inner_mut().stroke(
            &Stroke::new(LINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(if sel { aceso } else { apagado }),
            None,
            &p,
        );
        // A JUNTA: a bolinha na raiz é o que se agarra para posar, e é ela que mostra que dois
        // ossos partilham um ponto quando a cadeia é contínua.
        target.inner_mut().stroke(
            &Stroke::new(LINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(if sel { aceso } else { apagado }),
            None,
            &Circle::new(pa, BONE_JOINT_R_PX),
        );
    }
}
