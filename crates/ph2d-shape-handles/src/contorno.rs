//! ⭐ **A LEI do contorno da forma de colisão** (sem pintar): o estado (`Marca`), a tinta (`Tinta`), o Look (`Tom`) e o
//! traço que o tipo do corpo dá. Mora na folha das alças da forma (e não no painel) para o cartão Look e o canvas
//! lerem a MESMA lei sem a folha subir ao painel; a cor resolvida pelo tema e a sombra ficam em
//! `ph2d_panel_physics::tinta_canvas`.
//!
//! ⭐ **O TIPO do corpo lê-se pelo traço** (dono 07/11, onda 9g: *«Static grosso, Dynamic fino e Kinematic
//! pontilhado»*). O sensor tem traços LONGOS, a solta traços curtos e espaçados; a peça de um corpo composto herda o
//! tipo do corpo dono (`governing_kind`).

use ph2d_physics_ecs::{BodyKind, Collider, ShapeLook};
use ph2d_tokens::{Spacing, StrokeToken};

/// O estado de uma forma, que decide o padrão e a opacidade da cor da sua camada.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Marca {
    Solida,
    SensorVazio,
    /// O mesmo sensor com um corpo dentro: o salto de opacidade É o gatilho a disparar.
    SensorAceso,
    /// A prancha que o player atravessa AGORA: sem isto a descida é invisível.
    Atravessavel,
    /// Sem corpo acima: o solver a ignora; tracejado curto e o mais apagado.
    Solta,
}

/// O que um contorno pinta: uma cor fixa (as anotações) ou a cor da camada sob uma marca e o tipo
/// do corpo dono (`None` = solta).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tinta {
    Fixa([f32; 4]),
    Camada {
        camada: u8,
        marca: Marca,
        corpo: Option<BodyKind>,
        /// O Look da forma (GEF02 onda 8): a cor que SUBSTITUI a da camada e o multiplicador do alfa.
        /// [`TOM_NEUTRO`] = a cor do tema.
        tom: Tom,
    },
    /// ⭐ Uma linha de sensor (o traço fino de [`SONDA_PX`]). Com `camada` (o `RaySensor` do objecto, emenda 4, dono
    /// 08/11) a matiz é a da camada e o estado fica na opacidade `rgba[3]` (parado · nada · viu), com a sombra do
    /// contorno; sem ela (os sensores do personagem, o caminho) é a cor fixa `rgba`, sem sombra.
    Sonda {
        camada: Option<u8>,
        rgba: [f32; 4],
    },
}

/// O Look da forma no contorno: `cor` substitui o RGB da camada (`None` = a do tema); `alfa` multiplica a opacidade;
/// `traco` substitui o traço do tipo do corpo (`None` = o do corpo).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tom {
    pub cor: Option<[f32; 3]>,
    pub alfa: f32,
    pub traco: Option<Traco>,
}

/// Um traço de contorno em px de tela: a largura e o `[traço, vão]` do tracejado (`None` = contínuo).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Traco {
    pub largura: f64,
    pub padrao: Option<[f64; 2]>,
}

/// O `tom` que não muda a cor da camada.
pub const TOM_NEUTRO: Tom = Tom {
    cor: None,
    alfa: 1.0,
    traco: None,
};

/// ⭐ O Look da forma como o contorno o pinta: a cor, a opacidade e o traço próprio.
impl From<&ShapeLook> for Tom {
    fn from(l: &ShapeLook) -> Self {
        let (cor, alfa) = l.tom();
        let traco = l.traco_proprio.then(|| Traco {
            largura: f64::from(l.largura),
            padrao: l.tracejado.then(|| l.traco.map(f64::from)),
        });
        Self { cor, alfa, traco }
    }
}

/// ⭐ O traço que o tipo do corpo dono (`governing_kind`) dá à forma em repouso: o que o cartão Look mostra sem traço
/// próprio.
#[must_use]
pub fn traco_do_corpo(col: &Collider, corpo: Option<BodyKind>) -> Traco {
    let Tinta::Camada { marca, .. } = marca(col, corpo, false, false) else {
        unreachable!("a marca de uma forma é sempre da camada")
    };
    Traco {
        largura: espessura(corpo),
        padrao: padrao(marca, corpo),
    }
}

/// A espessura de uma linha de sensor, mais fina que o contorno (`OUTLINE_PX` da `ph2d-app-physics`).
pub const SONDA_PX: f64 = 1.25; // LITERAL-PX-OK: chrome de overlay

impl PartialEq<[f32; 4]> for Tinta {
    fn eq(&self, rgba: &[f32; 4]) -> bool {
        *self == Tinta::Fixa(*rgba)
    }
}

/// A espessura do traço pelo tipo do corpo: Static grosso, Dynamic e Kinematic finos.
#[must_use]
pub fn espessura(corpo: Option<BodyKind>) -> f64 {
    let t = match corpo {
        Some(BodyKind::Static) => StrokeToken::Heavy,
        _ => StrokeToken::Default,
    };
    f64::from(t.px())
}

/// O padrão `[traço, vão]` em px de tela; `None` = contínuo. Sensor: traços longos; Kinematic:
/// pontos (o traço do tamanho da espessura); solta: traços curtos e espaçados.
#[must_use]
pub fn padrao(marca: Marca, corpo: Option<BodyKind>) -> Option<[f64; 2]> {
    let px = |s: Spacing| f64::from(s.px());
    match (marca, corpo) {
        (Marca::SensorVazio | Marca::SensorAceso, _) => Some([px(Spacing::Lg), px(Spacing::Sm)]),
        (Marca::Solta, _) => Some([px(Spacing::Xs), px(Spacing::Md)]),
        (_, Some(BodyKind::Kinematic)) => Some([espessura(corpo), px(Spacing::Xs)]),
        _ => None,
    }
}

/// A marca de uma forma: atravessável antes de sensor (uma one-way nunca é sensor), solta sem corpo.
#[must_use]
pub fn marca(col: &Collider, corpo: Option<BodyKind>, aceso: bool, atravessavel: bool) -> Tinta {
    let marca = match (corpo.is_some(), atravessavel, col.is_sensor, aceso) {
        (false, ..) => Marca::Solta,
        (true, true, ..) => Marca::Atravessavel,
        (true, false, true, true) => Marca::SensorAceso,
        (true, false, true, false) => Marca::SensorVazio,
        (true, false, false, _) => Marca::Solida,
    };
    Tinta::Camada {
        camada: col.layer,
        marca,
        corpo,
        tom: TOM_NEUTRO,
    }
}

/// A tinta com o `tom` do Look da forma (só a da camada o leva; as anotações de cor fixa ficam).
#[must_use]
pub fn com_tom(tinta: Tinta, tom: Tom) -> Tinta {
    match tinta {
        Tinta::Camada {
            camada,
            marca,
            corpo,
            ..
        } => Tinta::Camada {
            camada,
            marca,
            corpo,
            tom,
        },
        outra => outra,
    }
}
