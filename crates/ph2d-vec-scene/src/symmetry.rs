//! **A SIMETRIA de desenho** — o eixo é do ARTISTA, e o outro lado é derivado.
//!
//! O que o artista liga numa seção do painel, vê como linhas no canvas e consolida com um
//! **Apply**. Enquanto está ligada as cópias são **DESENHO**, não documento: desligar faz as
//! cópias sumirem sem destruir nada.
//!
//! ⚠️ **Isto nasceu como um `PathEffect` e foi REPROVADO** (Enio, 2026-08-01: *"funciona bem mas
//! não é legal como um efeito; melhor como uma opção para as tools de desenho exatamente como o
//! modo painter"*). O que estava errado era o LUGAR, não a matemática — a reflexão, a reposição
//! do winding e a fusão sobreviveram inteiras. O que mudou é quem manda no eixo: era um número
//! relativo à caixa da forma, e passa a ser um **ponto autorado** que o artista arrasta.
//!
//! # O vocabulário é o do PAINTER, e isso é uma decisão, não coincidência
//!
//! *"exatamente como o modo painter"* — então os tipos são os DELE
//! (`ph2d_painter_brush::symmetry`): **Mirror X** (linha vertical, esquerda↔direita) · **Mirror
//! Y** (horizontal) · **Custom** (a linha que o artista desenha) · **Radial** (3..=12 cópias em
//! torno de um centro). Duas listas de *"que simetrias este app tem"* divergiriam no dia em que
//! uma delas ganhasse a quinta — há gate na shell (a única crate que vê os dois lados) a exigir
//! que as duas concordem em número, discriminante de wire e faixa de segmentos.
//!
//! ⚠️ A matemática **não** é partilhável: o Painter transforma um `Dab` (pixels de canvas, com
//! vetores de orientação) e isto transforma um contorno (unidades de documento, com alças). O que
//! se partilha é a palavra.
//!
//! # A reflexão inverte o WINDING, e sem repor abre-se um buraco
//!
//! Sob [`crate::FillRule::NonZero`] dois contornos sobrepostos de sentidos OPOSTOS cancelam-se:
//! quem espelha uma forma através de um eixo que a atravessa veria um **buraco** onde esperava um
//! bloco. Cada contorno reflectido é invertido de volta ([`crate::reverse_contour`], a porta
//! única), e como a inversão é **uniforme** o buraco de um compound continua buraco.
//!
//! ⚠️ **A rotação NÃO precisa disso** — ela tem determinante +1 e preserva o sentido por
//! construção. É a diferença entre as duas famílias, e é o que impede alguém de "uniformizar" o
//! tratamento e inverter as cópias radiais em silêncio.
//!
//! # A FUSÃO: é ela que faz do meio-perfil um vaso
//!
//! Um contorno ABERTO cujas duas pontas pousam no eixo funde-se com o reflexo num **único
//! contorno fechado**, em vez de deixar duas metades que apenas se tocam e não preenchem. É o
//! *Fuse paths* do LPE de simetria do Inkscape e o *Merge* do modificador do Blender.
//!
//! ⚠️ **A costura fica lisa quando a alça da ponta é PERPENDICULAR ao eixo** — a mesma regra do
//! Blender, e não um defeito desta implementação: a alça de entrada na costura é o reflexo da de
//! saída, então as duas tangentes só ficam colineares quando a alça não tem componente ao longo
//! da linha. Uma alça oblíqua dá um bico simétrico, que às vezes é o que se quer.
//!
//! ⚠️ E quando não se aplica (contorno fechado, ou pontas fora do eixo) a fusão **degrada para o
//! espelho simples** — visivelmente, não em silêncio: vêem-se duas metades, e elas fundem-se no
//! instante em que o artista arrasta uma ponta para o eixo.

use crate::{Contour, VecPath, VecVertex, reverse_contour};

/// Abaixo disto uma distância é zero.
const EPS: f64 = 1e-12;

/// Menos cópias radiais que a UI alcança — o mesmo `3` do Painter.
pub const MIN_SEGMENTS: u32 = 3;
/// Mais cópias radiais que a UI alcança — o mesmo `12` do Painter.
pub const MAX_SEGMENTS: u32 = 12;

/// **A tolerância da fusão**, em fracção da extensão da forma.
///
/// ⚠️ **MEDIDA, não escolhida** (CLAUDE.md §0). Meio-perfil de extensão `≈ 2,0`, pontas afastadas
/// do eixo por `g`, a varrer a fracção:
///
/// | `g / extensão`  | 0,000 | 0,002 | 0,010 | 0,020 | 0,050 |
/// |-----------------|-------|-------|-------|-------|-------|
/// | funde com 0,01? |  sim  |  sim  |  sim  |  não  |  não  |
///
/// `0,01` = **1% da forma**: folgado para a mão (a ponta larga a um ou dois pixels do eixo em
/// zoom de trabalho) e apertado para não fundir uma ponta visivelmente afastada — a `0,02` o vão
/// já se vê no ecrã. Há gate a varrer exactamente esta tabela.
const FUSE_TOL_FRAC: f64 = 0.01;

/// **Que simetria** — o vocabulário do Painter, palavra por palavra.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum SymmetryKind {
    /// Reflecte esquerda↔direita numa linha **vertical** pelo centro (a "X" que os artistas
    /// esperam: ela espelha a coordenada X).
    #[default]
    MirrorX,
    /// Reflecte cima↔baixo numa linha **horizontal** pelo centro.
    MirrorY,
    /// Reflecte na linha arbitrária pelo centro, com a direcção que o artista desenhou.
    Custom,
    /// `segments` cópias em rotação à volta do centro — a *circular*.
    Radial,
}

impl SymmetryKind {
    /// Todos, na ordem em que o painel os desenha.
    pub const ALL: &'static [Self] = &[Self::MirrorX, Self::MirrorY, Self::Custom, Self::Radial];

    /// O discriminante de wire — **o mesmo do Painter** (`0` X, `1` Y, `2` Custom), com o Radial
    /// no fim (lá ele é um bool à parte, aqui é o quarto membro da mesma lista).
    #[must_use]
    pub fn to_u8(self) -> u8 {
        match self {
            Self::MirrorX => 0,
            Self::MirrorY => 1,
            Self::Custom => 2,
            Self::Radial => 3,
        }
    }

    /// Decodifica um discriminante de wire (fora de alcance → `MirrorX`).
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::MirrorY,
            2 => Self::Custom,
            3 => Self::Radial,
            _ => Self::MirrorX,
        }
    }

    /// O rótulo que o painel mostra. Mora aqui, e não numa tabela do painel, porque uma segunda
    /// lista divergiria da primeira assim que alguém acrescentasse um tipo.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::MirrorX => "Mirror X",
            Self::MirrorY => "Mirror Y",
            Self::Custom => "Custom",
            Self::Radial => "Radial",
        }
    }

    /// Esta simetria é uma REFLEXÃO (e portanto inverte o winding)?
    #[must_use]
    pub fn reflects(self) -> bool {
        !matches!(self, Self::Radial)
    }
}

/// **A simetria autorada de uma forma.** Tudo em unidades de DOCUMENTO — o centro é um lugar que
/// o artista arrasta, não uma fracção da caixa.
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SymmetrySpec {
    pub kind: SymmetryKind,
    /// Um ponto NA linha de espelho, ou o centro da rosácea.
    pub center: [f64; 2],
    /// Direcção da linha quando `kind == Custom`. Normalizada defensivamente; `[0,0]` cai numa
    /// vertical.
    pub dir: [f64; 2],
    /// Cópias em rotação quando `kind == Radial`, presa a `[MIN_SEGMENTS, MAX_SEGMENTS]`.
    pub segments: u32,
    /// Funde as metades num contorno fechado quando as pontas de um contorno aberto pousam no
    /// eixo. Inerte no Radial (não há costura a fechar).
    pub fuse: bool,
}

impl Default for SymmetrySpec {
    fn default() -> Self {
        Self {
            kind: SymmetryKind::MirrorX,
            center: [0.0, 0.0],
            dir: [0.0, 1.0],
            segments: 6,
            fuse: true,
        }
    }
}

impl SymmetrySpec {
    /// O número de segmentos preso à faixa da UI.
    #[must_use]
    pub fn segments(&self) -> u32 {
        self.segments.clamp(MIN_SEGMENTS, MAX_SEGMENTS)
    }

    /// A direcção UNITÁRIA da linha de espelho — vertical no X, horizontal no Y, a autorada no
    /// Custom (com queda para vertical se for degenerada).
    ///
    /// ⚠️ Porta única: o kernel usa-a para reflectir e o **overlay** para desenhar a linha. Duas
    /// respostas desenhariam um eixo onde a geometria não espelha.
    #[must_use]
    pub fn mirror_dir(&self) -> [f64; 2] {
        match self.kind {
            SymmetryKind::MirrorX => [0.0, 1.0],
            SymmetryKind::MirrorY => [1.0, 0.0],
            SymmetryKind::Custom | SymmetryKind::Radial => {
                let len = self.dir[0].hypot(self.dir[1]);
                if len < 1e-9 {
                    [0.0, 1.0]
                } else {
                    [self.dir[0] / len, self.dir[1] / len]
                }
            }
        }
    }

    /// Quantas cópias esta simetria produz ao TODO, contando o original.
    #[must_use]
    pub fn copy_count(&self) -> usize {
        match self.kind {
            SymmetryKind::Radial => self.segments() as usize,
            _ => 2,
        }
    }
}

/// **Uma linha de espelho**: um ponto `at` e a normal UNITÁRIA `n`.
#[derive(Copy, Clone, Debug)]
pub struct Axis {
    pub at: [f64; 2],
    pub n: [f64; 2],
}

impl Axis {
    /// A linha desta simetria (sem sentido para o Radial, que não tem eixo).
    #[must_use]
    pub fn of(spec: &SymmetrySpec) -> Self {
        let d = spec.mirror_dir();
        Self {
            at: spec.center,
            n: [-d[1], d[0]],
        }
    }

    /// O reflexo de `p`. Como `n` é unitária, `p − 2·((p−at)·n)·n` é exacto e sem divisão.
    #[must_use]
    pub fn reflect(self, p: [f64; 2]) -> [f64; 2] {
        let d = (p[0] - self.at[0]).mul_add(self.n[0], (p[1] - self.at[1]) * self.n[1]);
        [
            (-2.0 * d).mul_add(self.n[0], p[0]),
            (-2.0 * d).mul_add(self.n[1], p[1]),
        ]
    }

    /// A distância COM SINAL de `p` à linha — positiva do lado para onde `n` aponta.
    #[must_use]
    pub fn signed_distance(self, p: [f64; 2]) -> f64 {
        (p[0] - self.at[0]).mul_add(self.n[0], (p[1] - self.at[1]) * self.n[1])
    }
}

/// Reflecte um vértice. O `corner_radius` é um comprimento LOCAL e a reflexão é uma isometria,
/// então ele sobrevive intacto.
fn reflect_vert(v: &VecVertex, ax: Axis) -> VecVertex {
    VecVertex {
        anchor: ax.reflect(v.anchor),
        in_handle: ax.reflect(v.in_handle),
        out_handle: ax.reflect(v.out_handle),
        kind: v.kind,
        corner_radius: v.corner_radius,
    }
}

/// Roda um ponto em torno de `c` pelo rotor unitário `r = (cos θ, sin θ)`.
fn rotate_about(p: [f64; 2], c: [f64; 2], r: [f64; 2]) -> [f64; 2] {
    let (x, y) = (p[0] - c[0], p[1] - c[1]);
    [
        x.mul_add(r[0], -(y * r[1])) + c[0],
        x.mul_add(r[1], y * r[0]) + c[1],
    ]
}

/// Roda um vértice — âncora e as duas alças, pelo mesmo rotor.
fn rotate_vert(v: &VecVertex, c: [f64; 2], r: [f64; 2]) -> VecVertex {
    VecVertex {
        anchor: rotate_about(v.anchor, c, r),
        in_handle: rotate_about(v.in_handle, c, r),
        out_handle: rotate_about(v.out_handle, c, r),
        kind: v.kind,
        corner_radius: v.corner_radius,
    }
}

/// A extensão da caixa de controlo — a régua da tolerância de fusão.
fn extent(path: &VecPath) -> f64 {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    let mut seen = false;
    for v in path.verts_all() {
        for p in [v.anchor, v.in_handle, v.out_handle] {
            seen = true;
            for k in 0..2 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
    }
    if seen {
        ((hi[0] - lo[0]) + (hi[1] - lo[1])) * 0.5
    } else {
        0.0
    }
}

/// Este contorno aberto tem as DUAS pontas no eixo?
fn touches_axis(verts: &[VecVertex], closed: bool, ax: Axis, tol: f64) -> bool {
    if closed || verts.len() < 2 {
        return false;
    }
    let (Some(a), Some(b)) = (verts.first(), verts.last()) else {
        return false;
    };
    ax.signed_distance(a.anchor).abs() <= tol && ax.signed_distance(b.anchor).abs() <= tol
}

/// **Funde um contorno aberto com o reflexo dele** num único contorno fechado.
///
/// O reflexo é percorrido ao contrário (o que também **repõe o winding**: reflectir inverte,
/// inverter de novo devolve) e as duas cópias das pontas são descartadas — elas estão no eixo,
/// logo o reflexo delas é elas próprias.
fn fuse(verts: &[VecVertex], ax: Axis) -> Vec<VecVertex> {
    let mut out: Vec<VecVertex> = verts.to_vec();
    let mut back: Vec<VecVertex> = verts.iter().map(|v| reflect_vert(v, ax)).collect();
    reverse_contour(&mut back);
    if back.len() > 2 {
        out.extend_from_slice(&back[1..back.len() - 1]);
    }
    // A costura: a alça que CHEGA à primeira ponta é o reflexo da que dela SAI, e vice-versa na
    // última. Sem isto o fecho do contorno corta reto exactamente onde a simetria devia ser mais
    // visível.
    if let (Some(first), Some(src)) = (out.first_mut(), verts.first()) {
        first.in_handle = ax.reflect(src.out_handle);
    }
    let seam = verts.len() - 1;
    if let (Some(last), Some(src)) = (out.get_mut(seam), verts.last()) {
        last.out_handle = ax.reflect(src.in_handle);
    }
    out
}

/// Constrói um caminho a partir de contornos, herdando o estilo de `template`.
fn from_contours(template: &VecPath, mut made: Vec<(Vec<VecVertex>, bool)>) -> VecPath {
    let mut out = template.clone();
    out.subpaths.clear();
    if made.is_empty() {
        out.verts.clear();
        return out;
    }
    let (verts, closed) = made.remove(0);
    out.verts = verts;
    out.closed = closed;
    for (verts, closed) in made {
        out.subpaths.push(Contour { verts, closed });
    }
    out
}

/// **A geometria que a simetria DESENHA** — o original mais as cópias, na ordem de emissão.
///
/// Devolve o conjunto INTEIRO (o original é o primeiro), porque é isso que o desenho vivo
/// substitui: quem pinta e quem aponta recebem a mesma lista, e não há uma segunda pergunta sobre
/// *"o original também entra?"*.
///
/// ⚠️ Cada cópia é um `VecPath` SEPARADO, não um contorno extra do mesmo. Numa rosácea de 6 as
/// pétalas são seis formas — e é isso que faz o **Apply** poder entregá-las como objetos, em vez
/// de um compound que ninguém consegue voltar a separar.
#[must_use]
pub fn symmetry_paths(path: &VecPath, spec: &SymmetrySpec) -> Vec<VecPath> {
    let source: Vec<(Vec<VecVertex>, bool)> = (0..path.contour_count())
        .filter_map(|k| path.contour(k).map(|(v, cl)| (v.to_vec(), cl)))
        .collect();
    if source.is_empty() {
        return vec![path.clone()];
    }

    if spec.kind == SymmetryKind::Radial {
        let n = spec.segments();
        let mut out = Vec::with_capacity(n as usize);
        out.push(path.clone());
        let step = core::f64::consts::TAU / f64::from(n);
        for k in 1..n {
            let (s, c) = (step * f64::from(k)).sin_cos();
            let made = source
                .iter()
                .map(|(verts, closed)| {
                    (
                        verts
                            .iter()
                            .map(|v| rotate_vert(v, spec.center, [c, s]))
                            .collect(),
                        *closed,
                    )
                })
                .collect();
            out.push(from_contours(path, made));
        }
        return out;
    }

    let ax = Axis::of(spec);
    let tol = (extent(path) * FUSE_TOL_FRAC).max(EPS);

    // A fusão produz UM caminho (as duas metades soldadas), não dois.
    if spec.fuse {
        let fusable: Vec<(Vec<VecVertex>, bool)> = source
            .iter()
            .filter(|(v, cl)| touches_axis(v, *cl, ax, tol))
            .map(|(v, _)| (fuse(v, ax), true))
            .collect();
        if fusable.len() == source.len() {
            return vec![from_contours(path, fusable)];
        }
    }

    let mirrored: Vec<(Vec<VecVertex>, bool)> = source
        .iter()
        .map(|(verts, closed)| {
            let mut back: Vec<VecVertex> = verts.iter().map(|v| reflect_vert(v, ax)).collect();
            // Repõe o sentido que a reflexão inverteu — sem isto a sobreposição fica com um
            // buraco sob `NonZero`. A rotação NÃO passa por aqui, e não deve.
            reverse_contour(&mut back);
            (back, *closed)
        })
        .collect();
    vec![path.clone(), from_contours(path, mirrored)]
}

#[cfg(test)]
#[path = "symmetry_tests.rs"]
mod tests;
