//! ⭐⭐⭐ **O QUE O CHÃO PÕE NUM PIXEL DE FUNDO** — as duas metades, e elas são simétricas.
//!
//! O [`crate::ground`] responde **onde** o chão está e **quanto do céu** o alcança; este módulo
//! responde o que isso vale no pixel. E são DUAS coisas, porque o chão muda a luz nos dois sentidos:
//!
//! - **o que a peça TIRA** — a sombra das lâmpadas e o escurecimento de contacto, numa razão
//!   ESCALAR ([`ground_factors`]). Ela sai como uma camada preta com alfa, que é o que um *shadow
//!   catcher* entrega;
//! - **o que a peça PÕE** — a luz que ela devolve, COLORIDA e SOMADA ([`crate::ground_bounce`]).
//!
//! # ⛔⛔⛔ Porque uma SOMA e não uma razão por canal, com o número que o decide
//!
//! A tentação é tingir a razão: fazer `f` um `vec3` e deixar o vermelho subir. ⛔ **Isso é invisível
//! no produto**, e há um literal que o prova: o fundo do modelador é
//! `ph2d_app_field3d::smoke::BACKGROUND = [0, 0, 0, 0]` — **transparente**. Uma razão multiplica
//! `bg.rgb`, que ali é ZERO, logo toda a wave sairia num pixel que não muda um bit.
//!
//! ⇒ a luz devolvida entra **somada em pré-multiplicado com alfa zero**, que é o que um compositor
//! lê como *luz acrescentada*: `resultado = fundo·f + B`. Medido na bola vermelha da sonda
//! `tests::chao_ricochete`, o pico do campo vale `0,015114` ⇒ **`33/255`** no vermelho sobre o preto
//! do modelador — *vê-se bem*.
//!
//! ⚠️ **E a razão CONTINUA escalar**, que é a decisão que o [`crate::ground::LUMA`] já declara por
//! escrito (*uma sombra tingida pela cor de cada luz mudaria de matiz sobre um fundo colorido*). As
//! duas metades não se misturam: uma diz **quanto escurece**, a outra **que luz mais chega**.
//!
//! # ⚠️ Porque este módulo existe, e não é arrumação
//!
//! O [`crate::shade_render`] estava a `698` linhas de um tecto de `700`. O corte é por
//! RESPONSABILIDADE e nunca por isenção (CLAUDE.md §5.0): o que saiu é **tudo o que o chão faz a um
//! pixel de fundo**, e o que ficou é o pintor da PEÇA.

use crate::shade_render::{PISO_DA_LAMPADA, ViewBasis, chega_da_lampada, view_direction};
use crate::{Gbuffer, Lighting, Orbit, Screen};
use rayon::prelude::*;

/// ⭐⭐⭐ **AS DUAS METADES DE CADA PIXEL DE FUNDO** — quanto ele ESCURECE (a razão escalar) e que
/// luz a peça lhe PÕE (a irradiância colorida). Vazias quando não há chão
/// (ver [`crate::Shadows::ground`]).
///
/// ⚠️ **Os pixels de peça e os que não vêem o chão leem `1,0` e `[0,0,0]`**, e é isso que mantém o
/// caminho sem chão byte a byte o de sempre.
///
/// ⚠️⚠️ **As duas saem da MESMA travessia e do MESMO ponto `q`, e é por isso que devolvem um par em
/// vez de haver duas funções.** Duas travessias significariam duas respostas a *«onde é que este
/// pixel toca o chão?»*, e a que envelhece é a que o artista vê — a lei que esta casa já pagou no
/// desdobrar da folha de sprites.
pub(crate) fn ground_factors(
    g: &Gbuffer,
    cam: &Orbit,
    screen: &Screen,
    basis: ViewBasis,
    light: &Lighting<'_>,
) -> (Vec<f32>, Vec<[f32; 3]>) {
    let Some(chao) = light.shadows.and_then(crate::Shadows::ground) else {
        return (Vec::new(), Vec::new());
    };
    let w = g.width as usize;
    let rays = cam.rays();
    let branco = crate::catcher_surface();
    let vazio = crate::ground_bounce::GroundBounce::vazio();
    let campo = light.shadows.map_or(&vazio, crate::Shadows::ground_bounce);
    (0..g.hit.len())
        .into_par_iter()
        .map(|i| {
            if g.hit[i] {
                return (1.0, [0.0; 3]);
            }
            let (x, y) = (i % w, i / w);
            crate::ground::ground_at(&rays, *screen, chao, x, y).map_or((1.0, [0.0; 3]), |q| {
                let v = view_direction(cam, screen, x, y);
                let devolvida = campo.sample(q);
                // ⭐ **A luz devolvida atravessa o MESMO material** que a razão usa: a difusa branca
                // do [`crate::catcher_surface`]. É o que a põe nas mesmas unidades que a peça, e é
                // a mesma porta que o pintor da peça usa para a somar ao céu.
                let posta = if devolvida == [0.0; 3] {
                    [0.0; 3]
                } else {
                    let n = basis.world_to_view(crate::GROUND_UP);
                    branco.indirect(n, v, &crate::shade_render::SoIrradiancia(devolvida))
                };
                (catcher(&branco, light, basis, i, q, v), posta)
            })
        })
        .unzip()
}

/// ⭐⭐⭐ **A LEI DO CHÃO QUE SÓ RECEBE** — `luz que chega com a peça / luz que chegaria sem ela`,
/// em luminância, sobre a difusa branca de [`crate::catcher_surface`].
///
/// ⚠️ **As DUAS somas correm as mesmas contas na mesma ordem**, e a de cima multiplica por cada
/// visibilidade: onde nada tapa (`1,0`), as duas são o MESMO número bit a bit e a razão é
/// exactamente `1` — o fundo sai com os bytes de sempre, e não com um arredondamento dele.
///
/// ⚠️ **Uma luz ancorada no ECRÃ entra nas duas e em nenhuma é tapada** — ela não tem sombra (ver
/// [`Lighting::shadows`]), logo ela só dilui a sombra das outras. Hoje o modo Render não as acende.
pub(crate) fn catcher(
    branco: &ph2d_material::Surface,
    light: &Lighting<'_>,
    basis: ViewBasis,
    i: usize,
    q: [f32; 3],
    v: [f32; 3],
) -> f32 {
    let add = |a: [f32; 3], b: [f32; 3]| [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
    let n = basis.world_to_view(crate::GROUND_UP);
    let ceu = branco.indirect(n, v, light.sky);
    let visto = light.shadows.map_or(1.0, |s| s.ambient_at(i));
    let mut livre = ceu;
    let mut chega = ceu.map(|c| c * visto);
    for lamp in light.lamps {
        let d = branco.direct(n, v, lamp.to_light, lamp.radiance);
        livre = add(livre, d);
        chega = add(chega, d);
    }
    let piso = PISO_DA_LAMPADA;
    for (l, lamp) in light.points.iter().enumerate() {
        let d = [
            lamp.world[0] - q[0],
            lamp.world[1] - q[1],
            lamp.world[2] - q[2],
        ];
        let cru = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
        // ⚠️ O mesmo piso das DUAS grandezas do sombreamento da peça — ver [`radiance`].
        let to_light = if cru <= piso {
            n
        } else {
            let inv = cru.sqrt().recip();
            basis.world_to_view([d[0] * inv, d[1] * inv, d[2] * inv])
        };
        // ⚠️ **`visivel = 1`**, e a visibilidade entra na linha de baixo: aqui a razão do chão
        // precisa das DUAS luzes — a que chegaria LIVRE e a que de facto chega.
        let rad = chega_da_lampada(lamp, cru, 1.0);
        let vis = light.shadows.map_or(1.0, |s| s.at(l, i));
        livre = add(livre, branco.direct(n, v, to_light, rad));
        chega = add(chega, branco.direct(n, v, to_light, rad.map(|c| c * vis)));
    }
    let luma = |c: [f32; 3]| {
        crate::ground::LUMA[0] * c[0]
            + crate::ground::LUMA[1] * c[1]
            + crate::ground::LUMA[2] * c[2]
    };
    let (a, b) = (luma(chega), luma(livre));
    // ⚠️ O `is_finite` primeiro, pela mesma razão do `Ground::hit`: sem luz nenhuma — ou com um
    // `NaN` — a razão não existe, e o fundo fica como está.
    if !b.is_finite() || b <= 0.0 {
        return 1.0;
    }
    (a / b).clamp(0.0, 1.0)
}

/// ⭐ **O fundo com a sombra do chão por cima**, em linear pré-multiplicado.
///
/// A sombra é uma camada PRETA de opacidade `1 − f` composta sobre o fundo: a cor escurece por `f`
/// e a cobertura sobe para `(1 − f) + a·f`. ⚠️ **As duas metades servem os dois fundos do produto**:
/// num fundo opaco a cobertura fica `1`; num fundo TRANSPARENTE (o do modelador, que é composto sobre
/// o canvas) a sombra sai como tinta preta com alfa, que é o que um *shadow catcher* entrega.
pub(crate) fn shadowed_background(bg: [f32; 4], f: f32) -> [f32; 4] {
    [bg[0] * f, bg[1] * f, bg[2] * f, (1.0 - f) + bg[3] * f]
}

/// ⭐⭐ **O factor do chão de uma BORDA** — o do próprio pixel quando ele falha a peça; senão, a média
/// dos vizinhos de cruz que a falham (esquerda, direita, cima, baixo — nesta ordem, que é a do
/// dispositivo).
///
/// ⚠️ **APROXIMAÇÃO DECLARADA**, da família das outras da borda: o chão que as sub-amostras de um
/// pixel de silhueta vêem é o dos vizinhos que o vêem inteiro. Sem vizinho de fundo, `1,0`.
pub(crate) fn edge_ground_factor(g: &Gbuffer, fatores: &[f32], i: usize) -> f32 {
    if fatores.is_empty() {
        return 1.0;
    }
    if !g.hit[i] {
        return fatores[i];
    }
    let (w, h) = (g.width as usize, g.height as usize);
    let (x, y) = (i % w, i / w);
    let mut soma = 0.0f32;
    let mut n = 0u32;
    let vizinhos = [
        (x > 0).then(|| i - 1),
        (x + 1 < w).then(|| i + 1),
        (y > 0).then(|| i - w),
        (y + 1 < h).then(|| i + w),
    ];
    for j in vizinhos.into_iter().flatten() {
        if !g.hit[j] {
            soma += fatores[j];
            n += 1;
        }
    }
    if n == 0 { 1.0 } else { soma / n as f32 }
}

/// ⭐⭐ **A luz devolvida de uma BORDA** — o gémeo do [`edge_ground_factor`], com a mesma
/// aproximação declarada e a **mesma ordem de vizinhos** (esquerda, direita, cima, baixo).
///
/// ⚠️⚠️ **Sem vizinho de fundo ele devolve `[0,0,0]` e o irmão devolve `1,0`**, e os dois estão
/// certos: *uma sombra que não foi calculada é ausência de sombra; uma luz que não foi calculada é
/// ausência de LUZ*. É a mesma lei que o [`crate::Shadows::bounce_at`] já declara — inventar luz é
/// a única das duas que acende o que devia estar escuro.
pub(crate) fn edge_ground_bounce(g: &Gbuffer, postas: &[[f32; 3]], i: usize) -> [f32; 3] {
    if postas.is_empty() {
        return [0.0; 3];
    }
    if !g.hit[i] {
        return postas[i];
    }
    let (w, h) = (g.width as usize, g.height as usize);
    let (x, y) = (i % w, i / w);
    let mut soma = [0.0f32; 3];
    let mut n = 0u32;
    let vizinhos = [
        (x > 0).then(|| i - 1),
        (x + 1 < w).then(|| i + 1),
        (y > 0).then(|| i - w),
        (y + 1 < h).then(|| i + w),
    ];
    for j in vizinhos.into_iter().flatten() {
        if !g.hit[j] {
            let p = postas[j];
            soma = [soma[0] + p[0], soma[1] + p[1], soma[2] + p[2]];
            n += 1;
        }
    }
    if n == 0 {
        return [0.0; 3];
    }
    #[allow(clippy::cast_precision_loss)]
    let inv = 1.0 / n as f32;
    [soma[0] * inv, soma[1] * inv, soma[2] * inv]
}

/// ⭐⭐⭐ **A SOMA: luz acrescentada em pré-multiplicado, com alfa ZERO.**
///
/// ⚠️⚠️ **O alfa NÃO sobe**, e isso é uma decisão com mecanismo: um chão INVISÍVEL não tem albedo
/// próprio com que se tornar opaco. A sombra sobe o alfa porque ela é uma camada preta a TAPAR o
/// que está atrás; a luz devolvida é luz a SOMAR-SE ao que está atrás, que em pré-multiplicado é
/// exactamente `rgb += B, a += 0`.
///
/// ⚠️ **Com `B = [0,0,0]` ela devolve a entrada AO BIT** (`x + 0.0 == x` para todo `x` finito), que
/// é o que mantém o caminho sem luz devolvida byte a byte o de sempre.
pub(crate) fn mais_luz(base: [f32; 4], b: [f32; 3]) -> [f32; 4] {
    [base[0] + b[0], base[1] + b[1], base[2] + b[2], base[3]]
}
