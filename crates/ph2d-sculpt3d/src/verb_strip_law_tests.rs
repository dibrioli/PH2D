//! Gates da **LEI** da faixa — quanto ela deposita, contra que plano, e onde
//! para.
//!
//! ⚠️ **Irmão do [`super::verb_strip`], e o corte é por ASSUNTO:** lá a pergunta
//! é *que barro a passada alcança* (a silhueta em caixa, os lados paralelos, as
//! quinas, o toque redondo — tudo sobre uma grade PLANA); aqui é *quanto ela
//! deposita e onde ela PARA* — o plano erguido, o vale que fecha, o auto-limite,
//! e a referência que governa. É por isso que as fixtures de superfície MOLDADA
//! (`shaped_grid`, `valley_z`, `dome_grid`) moram só neste arquivo: nenhum gate
//! de forma precisa de relevo debaixo do pincel, e todo gate de lei precisa.
//!
//! ⚠️ **O corte foi cobrado pelo teto de LOC, e ele estava VERMELHO-LATENTE:** a
//! wave do auto-limite deixou o pai em **746 > 700** e o fechamento por
//! `cargo test -p ph2d-sculpt3d` não alcança o gate, que mora na
//! `ph2d-editor-core` e só corre na varredura impactada — a mesma causa
//! estrutural que a `line/physics` e a `line/Vector` já documentaram. Ele só
//! apareceu quando um commit posterior tocou os ids do painel.

use super::verb_strip::{HALF, N, R, plane_grid, strip_brush};
use super::*;

/// Meia-largura e profundidade do vale das fixtures abaixo.
const VALLEY_W: f32 = 0.5;
const VALLEY_D: f32 = 0.4;

/// `z` de um vale liso que corre ao longo de `x`.
fn valley_z(y: f32) -> f32 {
    if y.abs() >= VALLEY_W {
        0.0
    } else {
        -VALLEY_D * 0.5 * (1.0 + (core::f32::consts::PI * y / VALLEY_W).cos())
    }
}

/// Uma grade cuja altura é dada por `f(x, y)`.
fn shaped_grid(f: impl Fn(f32, f32) -> f32) -> ph2d_mesh::Mesh {
    let mut pos = Vec::with_capacity((N + 1) * (N + 1));
    for j in 0..=N {
        for i in 0..=N {
            let g = |k: usize| (k as f32 / N as f32) * 2.0 * HALF - HALF;
            let (x, y) = (g(i), g(j));
            pos.push([x, y, f(x, y)]);
        }
    }
    let at = |i: usize, j: usize| (j * (N + 1) + i) as u32;
    let mut faces = Vec::with_capacity(N * N * 2);
    for j in 0..N {
        for i in 0..N {
            faces.push(ph2d_mesh::Face::tri(
                at(i, j),
                at(i + 1, j),
                at(i + 1, j + 1),
            ));
            faces.push(ph2d_mesh::Face::tri(
                at(i, j),
                at(i + 1, j + 1),
                at(i, j + 1),
            ));
        }
    }
    ph2d_mesh::Mesh::from_parts(pos, faces).expect("índices válidos")
}

/// Um traço ao longo de `+x` sobre uma superfície dada, com o cursor pousando
/// NELA.
///
/// ⚠️ **O raio é o da superfície, não o `R` das fixtures planas:** o vale mede
/// `2 · VALLEY_W` de largura, e um pincel muito menor que ele nunca vê as
/// cristas — a pegada ficaria inteira dentro do chão, onde não há relevo para
/// nivelar, e a fixture não conteria o fenômeno.
fn stroke_over(f: impl Fn(f32, f32) -> f32 + Copy, radius: f32, dabs: usize) -> ph2d_mesh::Mesh {
    let mut mesh = shaped_grid(f);
    let brush = Brush {
        verb: Verb::ClayStrips,
        radius,
        strength: 0.5,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    for k in 0..dabs {
        let x = (k as f32 - (dabs - 1) as f32 * 0.5) * 0.15 * radius;
        stroke.dab(
            &mut mesh,
            &brush,
            &Dab::at([x, 0.0, f(x, 0.0)], radius, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
    }
    mesh
}

/// A profundidade do vale na secção central: crista menos chão.
///
/// ⚠️ **A secção é lida por ÍNDICE de linha, nunca por `x ≈ 0`** — os vértices
/// andam ao longo da normal da área, que se inclina, então um filtro por
/// coordenada perde parte da coluna (a lição que a sonda do Draw Sharp pagou).
fn valley_depth(mesh: &ph2d_mesh::Mesh) -> f32 {
    let col: Vec<[f32; 3]> = (0..=N)
        .map(|j| mesh.positions()[j * (N + 1) + N / 2])
        .collect();
    let ridge = col
        .iter()
        .filter(|p| p[1].abs() <= 2.0 * VALLEY_W)
        .map(|p| p[2])
        .fold(f32::NEG_INFINITY, f32::max);
    let floor = col.iter().map(|p| p[2]).fold(f32::INFINITY, f32::min);
    ridge - floor
}

/// **A FAIXA FECHA UM VALE — ela não o exagera.**
///
/// É o report do Enio de 2026-08-15, e o gate nasceu VERMELHO: com o lift em
/// `0,5` a mesma passada media `0,4269` contra os `0,4000` de repouso, ou seja
/// **aprofundava** o vale em `+0,0269`.
///
/// ⚠️ **O oráculo é a PROFUNDIDADE, não a altura do chão.** Uma passada que
/// levantasse a paisagem inteira subiria o chão sem nivelar nada, e um gate que
/// olhasse só para o chão a chamaria de correta — é exatamente o que o `0,5`
/// fazia (o chão subia `0,0447` e a crista `0,0716`).
#[test]
fn the_strip_closes_a_valley_instead_of_deepening_it() {
    let rest = valley_depth(&shaped_grid(|_, y| valley_z(y)));
    let after = valley_depth(&stroke_over(|_, y| valley_z(y), 0.8, 9));
    assert!(
        after < rest - 0.02,
        "a faixa tinha de FECHAR o vale: {rest:.4} → {after:.4}"
    );
}

/// **E ela ainda deposita SOB O CURSOR numa forma convexa** — o contra-peso.
///
/// ⚠️ **Sem este gate, "fecha o vale" é maximizado levando o lift a zero**, e
/// aí o miolo da pegada — que numa cúpula está ACIMA do plano ajustado — recebe
/// literalmente nada: medido, `0,009` do que o aro recebe. A faixa deixaria de
/// ser uma banda e viraria um anel, e o artista apontaria para um sítio onde
/// nada acontece.
///
/// ⚠️ **A banda ser MAIS FINA no miolo não é defeito** — é a mesma lei vista
/// numa superfície convexa (*"displaces vertices toward the brush plane"*), e é
/// o que faz a ferramenta cortar planos. O que o gate proíbe é o miolo ficar
/// vazio.
#[test]
fn the_band_still_lands_under_the_cursor_on_a_convex_form() {
    let dome = |x: f32, y: f32| {
        let q = (x * x + y * y) / (1.5 * 1.5);
        if q >= 1.0 { 0.0 } else { 0.5 * (1.0 - q) }
    };
    let rest = shaped_grid(dome);
    let after = stroke_over(dome, 0.8, 9);
    let disp = |j: usize| {
        let i = j * (N + 1) + N / 2;
        after.positions()[i][2] - rest.positions()[i][2]
    };
    let pick = |lo: f32, hi: f32| {
        (0..=N)
            .filter(|&j| {
                let y = rest.positions()[j * (N + 1) + N / 2][1].abs();
                (lo..=hi).contains(&y)
            })
            .map(disp)
            .fold(f32::NEG_INFINITY, f32::max)
    };
    let (core, rim) = (pick(0.0, 0.15), pick(0.35, 0.85));
    assert!(rim > 1e-4, "o aro tinha de receber barro: {rim:.4}");
    assert!(
        core > rim * 0.4,
        "o miolo não pode ficar VAZIO: miolo {core:.4} contra aro {rim:.4}"
    );
}

// --- A FAIXA É UMA TOOL DO BLENDER, e a lei dela vem de la' ------------------

/// **A FAIXA NÃO DEPOSITA NO QUE O ARTISTA NÃO VÊ.**
///
/// Report do Enio de 2026-08-15 (com foto): lâminas a atravessar a silhueta de
/// um membro. ⚠️ **Medido com o olho RASANTE — a situação da foto — a faixa
/// punha `2,3×` mais barro nas costas do que na frente** (39,86 contra 17,18),
/// e perto da silhueta isso empurra a superfície de trás para FORA do contorno:
/// é a barbatana.
///
/// ⚠️ **A causa não era a silhueta nem o plano — era a REFERÊNCIA:** o
/// `Brush::default()` shipa em [`crate::RefMode::S`], e o `S` declarava *todos*
/// os verbos, incluindo um que o **SculptGL não tem**. O `front_face: Ignored`
/// do `S` é fiel ao `Brush.js` (`if (this._culling)`, e o `_culling` nasce
/// desligado), e simplesmente **não é a lei desta ferramenta**: o
/// `clay_strips.cc::calc_faces` chama `calc_front_face` como terceira linha da
/// cadeia de fatores.
#[test]
fn the_strip_does_not_lay_clay_on_what_the_artist_cannot_see() {
    // Olhar quase de lado: é assim que a silhueta entra na pegada.
    let eye = {
        let v = [1.0f32, 0.0, -0.25];
        let l = (v[0] * v[0] + v[2] * v[2]).sqrt();
        [v[0] / l, 0.0, v[2] / l]
    };
    let dome = |x: f32, y: f32| {
        let q = (x * x + y * y) / (1.5 * 1.5);
        if q >= 1.0 { 0.0 } else { 0.5 * (1.0 - q) }
    };
    let rest = shaped_grid(dome);
    let mut mesh = shaped_grid(dome);
    let brush = Brush {
        verb: Verb::ClayStrips,
        radius: 0.8,
        strength: 0.5,
        // ⚠️ **DERIVADO da tabela do verbo, nunca `true` literal**, e é isso que
        // faz deste gate a sentinela daquela célula: o front-face passou a ser
        // um flag do pincel (o `use_frontface` da referência) e o
        // `Brush::default()` carrega o default do **Draw**, que é `false`. Um
        // literal aqui deixaria o gate verde no dia em que alguém pusesse a
        // faixa em `false` na tabela — que é precisamente a mudança que este
        // número (117,2 nas costas contra 97,8 de frente, medido com o flag
        // fora) existe para tornar visível.
        front_faces_only: Verb::ClayStrips.default_front_faces_only(),
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    for k in 0..9 {
        let y = (k as f32 - 4.0) * 0.15 * 0.8;
        stroke.dab(
            &mut mesh,
            &brush,
            &Dab::at([0.9, y, dome(0.9, y)], 0.8, eye),
            Symmetry::default(),
        );
    }
    let (mut front, mut back) = (0.0f32, 0.0f32);
    for (i, p) in mesh.positions().iter().enumerate() {
        let q = rest.positions()[i];
        let moved = (p[0] - q[0]).abs() + (p[1] - q[1]).abs() + (p[2] - q[2]).abs();
        if moved <= 1e-6 {
            continue;
        }
        let n = rest.normals()[i];
        if -(n[0] * eye[0] + n[1] * eye[1] + n[2] * eye[2]) >= 0.0 {
            front += moved;
        } else {
            back += moved;
        }
    }
    assert!(
        front > 1e-3,
        "a fixture tinha de depositar na FRENTE: {front:.4}"
    );
    assert!(
        back < front * 0.01,
        "a faixa depositou nas COSTAS: {back:.4} contra {front:.4} de frente"
    );
}

/// **UMA REFERÊNCIA SÓ GOVERNA AS FERRAMENTAS QUE ELA TEM.**
///
/// ⚠️ **As duas tabelas do [`crate::RefMode`] discordavam, e só uma estava
/// certa:** a de DEFAULTS já devolvia `None` para o `S` na faixa (o censo
/// `the_census_of_offered_chips` até escreve *"todos menos o Sharpen e o Clay
/// Strips"*), enquanto a da LEI dizia `S => true` para tudo. Este gate afirma o
/// acordo, e a consequência que ele carrega — o produto pergunta a lei **por
/// verbo**.
#[test]
fn sculptgl_does_not_declare_the_strip_so_it_does_not_govern_it() {
    use crate::RefMode;
    assert!(
        !RefMode::S.declares(Verb::ClayStrips),
        "o SculptGL não tem Clay Strips"
    );
    assert!(RefMode::B.declares(Verb::ClayStrips), "o Blender tem");
    // O chip que o painel oferece para a faixa é UM, e é o da referência dela.
    let offered: Vec<RefMode> = RefMode::offered_for(Verb::ClayStrips).collect();
    assert_eq!(offered, vec![RefMode::B], "a faixa tem uma referência só");
    // ⚠️ **E a lei que o produto usa segue a referência, não o modo guardado:**
    // o `Brush::default()` shipa em `S`, e é isso que punha o `front_face`
    // errado na ferramenta.
    assert_eq!(
        RefMode::S.kernel_for(Verb::ClayStrips).front_face,
        crate::FrontFace::Continuous,
        "a faixa herda a lei do Blender mesmo com o modo em S"
    );
    // O CONTROLE: um verbo que o `S` de facto tem continua com a lei dele.
    assert_eq!(
        RefMode::S.kernel_for(Verb::Draw).front_face,
        crate::FrontFace::Ignored,
        "o Draw é do SculptGL e a lei dele não pode ter mudado"
    );
}

/// **O BARRO SOBE ATÉ O PLANO E PARA** — o auto-limite que É um clay strip.
///
/// ⚠️ **Este gate nasceu porque a mutação do [`crate::BLENDER_REACH_FRACTION`]
/// SOBREVIVEU aos 219:** devolver à faixa o `0,1` do SculptGL deixava tudo
/// verde. O número que fazia a ferramenta parecer certa não era afirmado por
/// ninguém.
///
/// ⚠️ **E a propriedade não é o número, é o que ele DESTRAVA.** Com o `raio ·
/// força` do `clay_strips.cc:327`, a posição VIVA no portão e o plano
/// CONGELADO do `!ss.cache->accum`, o `z` do portão `z·(1−z)` encolhe à medida
/// que o barro sobe e **fecha no plano**. Medido, o pico pousa em `0,1000`
/// contra um plano em `0,1000` e lá fica.
///
/// ⚠️ **A FIXTURE VARIA UMA COISA SÓ, e a 1ª versão variava duas:** ela
/// aumentava a contagem de dabs *e*, com ela, o COMPRIMENTO do traço — a 81
/// dabs o traço media `4,8` contra uma chapa de `1,5` e saía da malha, o que
/// lia como *"não saturou"* (`1,67×`). O caminho é FIXO e o que muda é só quão
/// fino ele é amostrado; é a lei que este repo já pagou quatro vezes no relevo
/// do Painter.
#[test]
fn the_clay_rises_to_the_plane_and_stops() {
    // O plano congelado: a superfície em repouso mais o lift.
    let plane = R * crate::STRIP_PLANE_FRACTION;
    let peak = |dabs: usize| {
        let mut mesh = plane_grid(N, HALF);
        let brush = strip_brush(1.0, crate::Brush::default().tip_roundness);
        let mut stroke = SculptStroke::default();
        stroke.begin(&mesh);
        for k in 0..dabs {
            // ⚠️ O MESMO caminho, amostrado mais fino — nunca um caminho maior.
            let t = if dabs == 1 {
                0.0
            } else {
                k as f32 / (dabs - 1) as f32 - 0.5
            };
            stroke.dab(
                &mut mesh,
                &brush,
                &Dab::at([t * 0.6, 0.0, 0.0], R, [0.0, 0.0, -1.0]),
                Symmetry::default(),
            );
        }
        mesh.positions()
            .iter()
            .map(|p| p[2])
            .fold(f32::NEG_INFINITY, f32::max)
    };
    let (a, b) = (peak(9), peak(81));
    assert!(
        a > plane * 0.9,
        "a fixture tinha de chegar ao plano: {a:.4}"
    );
    // ⚠️ **NOVE vezes os dabs sobre o MESMO caminho quase não pode mover o
    // barro** — é isso que "sobe até o plano e para" significa.
    assert!(
        b < a * 1.05,
        "a faixa não saturou: 9 dabs {a:.4} → 81 dabs {b:.4} ({:.2}x)",
        b / a
    );
    // E ela para NO plano, não muito acima dele.
    assert!(
        b < plane * 1.2,
        "a faixa passou do plano ({plane:.4}): {b:.4}"
    );
}
