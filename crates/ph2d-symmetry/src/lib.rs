//! **O vocabulário da SIMETRIA de desenho** — que espelho, onde, quantas cópias.
//!
//! Folha pura (só `serde`). O motor que produz as cópias vive na [`ph2d_vec_scene::symmetry`] (ele
//! precisa de `VecPath`, `VecVertex` e da inversão de contorno) e o documento guarda a relação num
//! componente ECS; nenhum dos dois vê o outro, então o tipo partilhado mora aqui.
//!
//! # O vocabulário é o do PAINTER, e isso é uma decisão
//!
//! *"exatamente como o modo painter"* (Enio, 2026-08-01) — então os tipos são os DELE
//! (`ph2d_painter_brush::symmetry`): **Mirror X** (linha vertical, esquerda↔direita) · **Mirror Y**
//! (horizontal) · **Custom** (a linha que o artista desenha) · **Radial** (3..=12 cópias em torno
//! de um centro). Duas listas de *"que simetrias este app tem"* divergiriam no dia em que uma
//! delas ganhasse a quinta.
//!
//! ⚠️ A matemática **não** é partilhável com o Painter: ele transforma um `Dab` (pixels de canvas,
//! com vetores de orientação) e isto transforma um contorno (unidades de documento, com alças). O
//! que se partilha é a palavra.
//!
//! # Duas famílias com propriedades OPOSTAS
//!
//! Uma reflexão tem determinante **−1** (inverte o sentido do contorno, e restaurá-lo é obrigatório
//! senão a sobreposição abre um buraco sob `NonZero`); uma rotação tem **+1** (preserva por
//! construção, e **não** deve passar pela reposição). [`SymmetryKind::reflects`] é a pergunta que
//! separa as duas — uniformizar o tratamento inverteria as cópias radiais em silêncio.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Menos cópias radiais que a UI alcança — o mesmo `3` do Painter.
pub const MIN_SEGMENTS: u32 = 3;
/// Mais cópias radiais que a UI alcança — o mesmo `12` do Painter.
pub const MAX_SEGMENTS: u32 = 12;

/// **Que simetria** — o vocabulário do Painter, palavra por palavra.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SymmetryKind {
    /// Reflete esquerda↔direita numa linha **vertical** pelo centro (a "X" que os artistas
    /// esperam: ela espelha a coordenada X).
    #[default]
    MirrorX,
    /// Reflete cima↔baixo numa linha **horizontal** pelo centro.
    MirrorY,
    /// Reflete na linha arbitrária pelo centro, com a direção que o artista desenhou.
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

/// **O ESTILO da simetria** — a parte que é da FERRAMENTA, não da forma nem do lugar.
///
/// A divisão tem **três** lados e cada um mora onde só ele pode morar:
///
/// - o **estilo** (que espelho, quantas cópias, funde ou não) é uma preferência que sobrevive ao
///   desenho seguinte — o artista escolhe Radial 6 e continua em Radial 6. É isto, e o painel
///   edita-o directamente;
/// - o **lugar** enquanto ninguém desenhou é da SESSÃO, e mora na shell porque nasce do centro do
///   ECRÃ e só a shell tem câmera;
/// - o **lugar** depois de um desenho é da FORMA, capturado no espaço local dela
///   ([`SymmetrySpec`]) para que mover o objecto leve a linha junto.
///
/// Um centro guardado aqui seria um campo que a shell teria de reescrever por fora; um estilo
/// guardado só na forma obrigaria o artista a re-escolher tudo a cada desenho novo.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymmetryStyle {
    /// O modo de desenho simétrico está LIGADO?
    ///
    /// ⚠️ Ele gateia o **cozimento**, não o componente: desligar esconde as cópias e religar
    /// devolve-as inteiras, porque elas nunca estiveram no documento. E gateia a **adopção**:
    /// desligado, nenhum desenho novo captura eixo nenhum.
    pub on: bool,
    pub kind: SymmetryKind,
    pub segments: u32,
    pub fuse: bool,
}

impl Default for SymmetryStyle {
    fn default() -> Self {
        let d = SymmetrySpec::default();
        Self {
            on: false,
            kind: d.kind,
            segments: d.segments,
            fuse: d.fuse,
        }
    }
}

/// **A simetria autorada de uma forma.**
///
/// ⚠️ O `center` e o `dir` são do espaço **LOCAL** da forma — o mesmo em que os vértices do
/// `VecPath` vivem. É essa escolha que cumpre a promessa *"se o usuário mover o objeto no canvas a
/// linha de simetria acompanha mantendo a mesma distância relativa ao objeto"*: mover a forma é
/// mexer no `Transform` dela, e um ponto local viaja junto **sem que ninguém o actualize**.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    /// Monta a spec de uma forma a partir do estilo da ferramenta e do LUGAR que o artista
    /// escolheu. Porta única dos dois lados: o `style()` volta a extrair, e nenhum campo é
    /// escrito à mão em dois sítios.
    #[must_use]
    pub fn from_style(style: SymmetryStyle, center: [f64; 2], dir: [f64; 2]) -> Self {
        Self {
            kind: style.kind,
            center,
            dir,
            segments: style.segments,
            fuse: style.fuse,
        }
    }

    /// O estilo desta spec — o que o painel espelha ao seleccionar uma forma que já tem simetria.
    #[must_use]
    pub fn style(&self) -> SymmetryStyle {
        SymmetryStyle {
            on: true,
            kind: self.kind,
            segments: self.segments,
            fuse: self.fuse,
        }
    }

    /// O número de segmentos preso à faixa da UI.
    #[must_use]
    pub fn segments(&self) -> u32 {
        self.segments.clamp(MIN_SEGMENTS, MAX_SEGMENTS)
    }

    /// A direção UNITÁRIA da linha de espelho — vertical no X, horizontal no Y, a autorada no
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
