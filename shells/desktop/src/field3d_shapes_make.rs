//! ⭐ **OS CONSTRUTORES DO CATÁLOGO** — uma função por forma, cada uma com os números com que ela
//! nasce.
//!
//! # Por que um arquivo irmão
//!
//! O [`super::field3d_shapes`] responde *«que formas o menu oferece, e em que família»*; este
//! responde *«com que números cada uma nasce»*. O arquivo passou as `600` linhas do gate de LOC do
//! shell — e já as passava **antes** desta wave (`690`). ⛔ *Split, nunca allowlist.*

use ph2d_field::Primitive;

/// ⭐ **O arredondamento com que uma forma nasce.**
///
/// ⚠️ **Não é zero, e é de propósito:** este é o módulo cujo argumento é o arredondamento, e uma
/// caixa de aresta viva ao nascer esconderia exatamente aquilo que ele faz melhor do que o Blender.
/// É uma **fração do tamanho**, então cabe sempre.
pub(crate) fn round_of(r: f32) -> f32 {
    r * 0.1
}

pub(crate) fn a_box(r: f32) -> Primitive {
    Primitive::Box {
        half: [r; 3],
        round: round_of(r),
        chamfer: 0.0,
    }
}

pub(crate) fn a_sphere(r: f32) -> Primitive {
    Primitive::Sphere { radius: r }
}

pub(crate) fn a_cylinder(r: f32) -> Primitive {
    Primitive::Cylinder {
        radius: r,
        half_height: r * 1.2,
        round: round_of(r),
        chamfer: 0.0,
    }
}

pub(crate) fn a_torus(r: f32) -> Primitive {
    Primitive::Torus {
        major: r,
        minor: r * 0.35,
    }
}

/// ⭐⭐ **O cone FECHADO** (W101) — `top = 0` é o ápice, e é a forma que dá nome à primitiva.
///
/// ⚠️ **Ele nasce COM filete, e a primeira versão desta função dizia o contrário com uma razão
/// inventada.** Eu escrevi que *«o filete que caberia num cone fechado seria fino ao ponto de não
/// se ver»* — o gate `every_new_shape_that_can_round_is_born_round` reprovou, e a conta refutou-me:
/// com `bottom = r` e `half_height = 1,2 r`, o [`ph2d_field::radius::cone_round_limit`] dá
/// **`0,4615 r`** e o default é `0,1 r` — cabe com folga de 4,6×. *Um palpite com cara de medição é
/// o que este repo mais paga.*
pub(crate) fn a_cone(r: f32) -> Primitive {
    Primitive::Cone {
        bottom: r,
        top: 0.0,
        half_height: r * 1.2,
        round: round_of(r),
        chamfer: 0.0,
    }
}

/// ⭐⭐ **O cone TRUNCADO** — a MESMA primitiva, com outro default.
///
/// ⚠️ **Duas linhas do catálogo, uma fórmula.** Elas não são formas diferentes: são o mesmo sólido
/// com o raio de topo em sítios diferentes, e o artista converte uma na outra arrastando um número.
/// Duas primitivas dariam duas fórmulas para a mesma superfície, e a segunda é a que envelhece.
///
pub(crate) fn a_truncated_cone(r: f32) -> Primitive {
    Primitive::Cone {
        bottom: r,
        top: r * 0.5,
        half_height: r * 1.2,
        round: round_of(r),
        chamfer: 0.0,
    }
}

pub(crate) fn a_capsule(r: f32) -> Primitive {
    Primitive::Capsule {
        radius: r * 0.6,
        half_height: r,
    }
}

/// ⭐ O prisma nasce **hexagonal** — é o polígono que um modelador desenha mais vezes (porcas,
/// flanges, favos), e é longe o bastante do triângulo e do círculo para a forma se ler à primeira.
pub(crate) fn a_prism(r: f32) -> Primitive {
    Primitive::Prism {
        sides: 6,
        bottom: r,
        top: r,
        half_height: r * 1.2,
        round: round_of(r),
        chamfer: 0.0,
    }
}

/// ⭐⭐ **A pirâmide é o prisma com o topo a ZERO** (W102) — a mesma primitiva, outro default.
///
/// ⚠️ Ela nasce de **base quadrada**: é a pirâmide que alguém desenha quando diz «pirâmide», e o
/// primeiro controlo do painel são os lados para quem quiser outra.
pub(crate) fn a_pyramid(r: f32) -> Primitive {
    Primitive::Prism {
        sides: 4,
        bottom: r,
        top: 0.0,
        half_height: r * 1.3,
        round: round_of(r),
        chamfer: 0.0,
    }
}

pub(crate) fn a_truncated_pyramid(r: f32) -> Primitive {
    Primitive::Prism {
        sides: 4,
        bottom: r,
        top: r * 0.5,
        half_height: r * 1.2,
        round: round_of(r),
        chamfer: 0.0,
    }
}

pub(crate) fn a_wedge(r: f32) -> Primitive {
    Primitive::Wedge {
        half: [r, r * 0.7, r * 0.8],
        round: round_of(r) * 0.5,
        chamfer: 0.0,
    }
}

/// ⭐ O arco nasce em **meia volta** — é o ângulo em que a forma se lê como arco à primeira (um
/// quarto parece um canto, uma volta quase inteira parece um toro com um defeito).
pub(crate) fn a_torus_arc(r: f32) -> Primitive {
    Primitive::TorusArc {
        major: r,
        minor: r * 0.28,
        angle: std::f32::consts::PI,
        round: r * 0.28 * 0.35,
        chamfer: 0.0,
    }
}

/// ⭐⭐ **A estrela nasce de CINCO pontas** (W103) — é a que se desenha quando se diz «estrela», e
/// é o número **ímpar** que nenhuma união de polígonos exprime (uma de 6 são dois triângulos; uma de
/// 5 não é nada que se possa compor).
///
/// ⚠️ **A razão interna é `0,4`, perto do `1/φ² = 0,382`** do pentagrama — abaixo disso as pontas
/// ficam agulhas e o filete padrão deixa de caber; acima, a forma lê-se como um decágono.
///
/// ⚠️ **Baixa de propósito** (`0,35 r`): ela é a primeira das [`Family::Plates`], e uma chapa lê-se
/// como estrela num ângulo qualquer — uma coluna de estrela não.
pub(crate) fn a_star(r: f32) -> Primitive {
    Primitive::Star {
        points: 5,
        outer: r,
        inner: r * 0.4,
        half_height: r * 0.35,
        round: round_of(r),
        chamfer: 0.0,
    }
}

/// ⚠️ A viga nasce a **30 %** da meia-extensão: mais fina desaparece à distância a que a peça
/// nasce, mais grossa lê-se como uma caixa com furos.
pub(crate) fn a_box_frame(r: f32) -> Primitive {
    Primitive::BoxFrame {
        half: [r; 3],
        thickness: r * 0.3,
        round: round_of(r),
        chamfer: 0.0,
    }
}

/// ⚠️ **Nasce com os três semi-eixos DIFERENTES** — iguais seria uma esfera, e a porta ao lado já a
/// tem: o default de uma forma tem de mostrar o que ela faz de diferente.
pub(crate) fn an_ellipsoid(r: f32) -> Primitive {
    Primitive::Ellipsoid {
        radii: [r, r * 0.55, r * 0.8],
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// ⭐⭐⭐ **W106 — as catorze que a fila nunca contou.**
//
// ⚠️ **O default de uma forma tem de MOSTRAR o que ela faz de diferente** (a lei que o elipsóide
// deixou escrita acima): um cone de pontas arredondadas com os dois raios iguais é uma cápsula, e
// uma engrenagem com o dente rente ao corpo é um disco. Cada um destes números nasce onde a forma
// se lê como ela própria à distância a que a peça aparece.
// ─────────────────────────────────────────────────────────────────────────────────────────────

pub(crate) fn an_octahedron(r: f32) -> Primitive {
    Primitive::Octahedron {
        radius: r,
        round: round_of(r),
        chamfer: 0.0,
    }
}

/// ⚠️ **Os dois raios DIFERENTES** — iguais seria a cápsula, que tem porta própria.
pub(crate) fn a_round_cone(r: f32) -> Primitive {
    Primitive::RoundCone {
        bottom: r * 0.55,
        top: r * 0.22,
        half_height: r * 0.75,
    }
}

/// ⚠️ **Corta ACIMA do equador** (`+0,35 r`): a meia-esfera exacta lê-se como uma esfera enterrada,
/// e o que esta forma tem de mostrar é que sobrou uma tampa plana.
pub(crate) fn a_cut_sphere(r: f32) -> Primitive {
    Primitive::CutSphere {
        radius: r,
        cut: r * 0.35,
        round: round_of(r) * 0.5,
        chamfer: 0.0,
    }
}

/// ⚠️ Corta **abaixo** do equador, para a tigela ter parede visível de dentro.
///
/// ⚠️⚠️ **O `round` NÃO pode nascer a zero**, e foi o gate `every_new_shape_that_can_round_is_born_round`
/// que mo disse: *este é o módulo cujo argumento **é** o arredondamento*, e uma forma que o aceita e
/// nasce de aresta viva esconde exactamente o que ele faz melhor que o Blender.
///
/// ⚠️ E o teto aqui é a **parede** (`thickness/2`), não o raio da esfera — daí a fracção dele em vez
/// do [`round_of`], que é dimensionado para uma peça maciça.
pub(crate) fn a_hollow_dome(r: f32) -> Primitive {
    let thickness = r * 0.12;
    Primitive::HollowDome {
        radius: r,
        cut: r * 0.15,
        thickness,
        round: thickness * 0.25,
        chamfer: 0.0,
    }
}

pub(crate) fn a_link(r: f32) -> Primitive {
    Primitive::Link {
        major: r * 0.45,
        minor: r * 0.16,
        length: r * 0.5,
    }
}

/// ⚠️ Meia-abertura de `0,6 rad` (≈ 34°): a `π/2` seria uma meia-esfera e a fatia não se lê.
pub(crate) fn a_solid_angle(r: f32) -> Primitive {
    Primitive::SolidAngle {
        radius: r,
        angle: 0.6,
        round: round_of(r) * 0.5,
        chamfer: 0.0,
    }
}

/// ⭐ **Doze dentes** — o número em que uma engrenagem se lê como uma engrenagem e não como uma
/// estrela grossa. ⚠️ O dente sai a `1,35 ×` o corpo: rente demais e ela vira um disco.
pub(crate) fn a_gear(r: f32) -> Primitive {
    Primitive::Gear {
        teeth: 12,
        root: r * 0.7,
        outer: r,
        tooth: 0.45,
        half_height: r * 0.25,
        round: round_of(r) * 0.3,
        chamfer: 0.0,
    }
}

pub(crate) fn a_cross(r: f32) -> Primitive {
    Primitive::Cross {
        arm: r,
        width: r * 0.3,
        half_height: r * 0.25,
        round: round_of(r) * 0.5,
        chamfer: 0.0,
    }
}

pub(crate) fn a_heart(r: f32) -> Primitive {
    Primitive::Heart {
        size: r * 0.6,
        half_height: r * 0.25,
        round: round_of(r) * 0.4,
        chamfer: 0.0,
    }
}

/// ⚠️ A mordida é **maior** que o disco e deslocada: é isso que faz um crescente fino em vez de um
/// disco com um buraco.
pub(crate) fn a_moon(r: f32) -> Primitive {
    Primitive::Moon {
        radius: r,
        bite: r * 0.9,
        offset: r * 0.45,
        half_height: r * 0.25,
        round: round_of(r) * 0.3,
        chamfer: 0.0,
    }
}

pub(crate) fn a_drop(r: f32) -> Primitive {
    Primitive::Drop {
        radius: r * 0.5,
        height: r * 1.3,
        half_height: r * 0.25,
        round: round_of(r) * 0.4,
        chamfer: 0.0,
    }
}

/// ⚠️ Meia-abertura de `1,0 rad` (≈ 57°): uma fatia de ~115°, que se lê como fatia.
pub(crate) fn a_pie(r: f32) -> Primitive {
    Primitive::Pie {
        radius: r,
        angle: 1.0,
        half_height: r * 0.25,
        round: round_of(r) * 0.4,
        chamfer: 0.0,
    }
}

pub(crate) fn a_trapezoid(r: f32) -> Primitive {
    Primitive::Trapezoid {
        bottom: r,
        top: r * 0.45,
        half_width: r * 0.6,
        half_height: r * 0.25,
        round: round_of(r) * 0.4,
        chamfer: 0.0,
    }
}

pub(crate) fn a_vesica(r: f32) -> Primitive {
    Primitive::Vesica {
        radius: r,
        offset: r * 0.55,
        half_height: r * 0.25,
        round: round_of(r) * 0.3,
        chamfer: 0.0,
    }
}
