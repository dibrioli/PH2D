//! **O `l-mode` DA FAMÍLIA QUE ACHATA** — o alvo deixa de ser um plano e passa a
//! ser a superfície local (Alexa et al. 2003; ver [`crate::stroke`]`::surface`).
//!
//! # O oráculo, e por que ele não é o deslocamento
//!
//! *"O Flatten mexeu?"* é satisfeito por qualquer verbo que empurre barro, e
//! *"o raio médio caiu?"* mede encolhimento, que é a pergunta do Smooth. O que
//! separa um plano de uma superfície é **o que sobra da CURVATURA**: o
//! `s-mode` corta uma faceta (a pegada fica CHATA, desvio ao plano ~0) e o
//! `l-mode` mantém a forma (o desvio ao plano fica onde estava).
//!
//! ⚠️ **E é por isso que o CONTROLE desta suíte é uma esfera com RUGA:** um
//! `l-mode` que seguisse o detalhe deixaria de achatar, e sobre uma esfera LISA
//! esse defeito é indistinguível do produto certo — não há ruga para ele
//! preservar por engano.

use super::*;
use crate::RefMode;
use ph2d_mesh::{Mesh, QueryScratch, shapes};

fn flat_brush(mode: RefMode) -> Brush {
    Brush {
        verb: Verb::Flatten,
        mode,
        radius: 0.4,
        strength: 1.0,
        // ⚠️ **`Constant` de propósito:** com uma curva macia o peso cai com a
        // distância e a pegada não chega ao alvo na borda — a comparação
        // passaria a falar do falloff.
        falloff: Falloff::Constant,
        ..Brush::default()
    }
}

fn dab() -> Dab {
    Dab::at([0.0, 0.0, 1.0], 0.4, [0.0, 0.0, -1.0])
}

/// Quanto a pegada se afasta do PLANO que ela própria ajusta — a curvatura que
/// sobra, medida com o mesmo plano que o dab usaria.
fn curvature_left(mesh: &mut Mesh, mode: RefMode) -> f64 {
    let b = flat_brush(mode);
    let d = dab();
    let mut s = SculptStroke::default();
    s.begin(mesh);
    let (p, n) = s.probe_plane(mesh, &b, &d);

    let mut scratch = QueryScratch::default();
    let mut ids = Vec::new();
    mesh.verts_in_sphere(d.center, d.radius, &mut scratch, &mut ids);
    let pos = mesh.positions();
    let sum: f64 = ids
        .iter()
        .map(|&i| {
            let q = pos[i as usize];
            let h = f64::from((q[0] - p[0]) * n[0] + (q[1] - p[1]) * n[1] + (q[2] - p[2]) * n[2]);
            h * h
        })
        .sum();
    (sum / ids.len() as f64).sqrt()
}

/// A RUGA de facto presente — a magnitude do laplaciano, `|p − média do anel|`.
///
/// ⚠️ **A minha primeira régua era `| |p| − raio_da_esfera |` e ela REPROVOU o
/// `s-mode`, que esta wave não toca** (0,016941 → 0,033543: a ruga *dobrava*).
/// O defeito era do oráculo: achatar uma calote esférica **tira os vértices da
/// esfera por construção**, então aquela régua soma a ruga com o próprio
/// achatamento e não consegue dizer qual das duas mediu. A magnitude do
/// laplaciano é local — ela vê o detalhe e é cega à forma grande, plana ou
/// curva —, que é exactamente a distinção que este gate existe para fazer.
fn wrinkle_left(mesh: &Mesh) -> f64 {
    let d = dab();
    let mut scratch = QueryScratch::default();
    let mut ids = Vec::new();
    mesh.verts_in_sphere(d.center, d.radius, &mut scratch, &mut ids);
    let pos = mesh.positions();
    let adj = mesh.adjacency();
    let sum: f64 = ids
        .iter()
        .map(|&i| {
            let p = pos[i as usize];
            let a = ph2d_mesh::ring_average(adj, i, p, |nb| pos[nb as usize]);
            let e = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
            f64::from(e[0].mul_add(e[0], e[1].mul_add(e[1], e[2] * e[2])))
        })
        .sum();
    (sum / ids.len() as f64).sqrt()
}

fn strike(mesh: &mut Mesh, mode: RefMode, times: usize) {
    let b = flat_brush(mode);
    for _ in 0..times {
        let mut s = SculptStroke::default();
        s.begin(mesh);
        s.dab(mesh, &b, &dab(), Symmetry::default());
    }
}

/// **A ENTREGA DA WAVE: o `s-mode` corta uma FACETA e o `l-mode` mantém a
/// FORMA.**
///
/// ⚠️ **O oráculo é a curvatura que SOBRA, e a barra é uma RAZÃO** — o número
/// absoluto depende da tesselação e do raio, e um limiar escolhido seria
/// calibrado numa fixture. O que a lei afirma é qualitativo e mensurável: um
/// achata até o plano, o outro não.
#[test]
fn the_literature_mode_flattens_to_the_local_surface_instead_of_a_plane() {
    let base = shapes::uv_sphere(64, 96, 1.0);
    let before = curvature_left(&mut base.clone(), RefMode::S);

    let mut flat = base.clone();
    strike(&mut flat, RefMode::S, 4);
    let after_s = curvature_left(&mut flat, RefMode::S);

    let mut mls = base.clone();
    strike(&mut mls, RefMode::L, 4);
    let after_l = curvature_left(&mut mls, RefMode::L);

    assert!(
        after_s < before * 0.25,
        "o s-mode tem de achatar a pegada: {before:.6} -> {after_s:.6}"
    );
    assert!(
        after_l > before * 0.75,
        "o l-mode tem de MANTER a curvatura: {before:.6} -> {after_l:.6}"
    );
}

/// **O CONTROLE que impede a wave de quebrar o verbo: os DOIS modos removem a
/// RUGA.**
///
/// ⚠️ **Sem ele, um `l-mode` que simplesmente não fizesse nada passaria no gate
/// acima** — manter a curvatura e não tocar em nada são indistinguíveis numa
/// esfera lisa. Um Flatten existe para remover detalhe, e um alvo que seguisse o
/// detalhe seria a wave a destruir o verbo com todos os números bonitos.
#[test]
fn both_modes_still_remove_the_wrinkle_they_exist_to_remove() {
    let base = shapes::uv_sphere_noisy(64, 96, 1.0, 0.03);
    let before = wrinkle_left(&base);

    for mode in [RefMode::S, RefMode::L] {
        let mut m = base.clone();
        strike(&mut m, mode, 4);
        let after = wrinkle_left(&m);
        assert!(
            after < before * 0.5,
            "{mode:?}: a ruga tem de cair, {before:.6} -> {after:.6}"
        );
    }
}

/// O pior desvio ENTRE os dois modos sobre a MESMA pegada.
///
/// ⚠️ **É a diferença entre eles, nunca a distância de cada um ao ponto de
/// partida:** um Flatten move barro nos dois modos, e o que se quer saber é se
/// eles movem o MESMO barro.
fn modes_diverge_by(sphere: &Mesh, center: [f32; 3], radius: f32) -> f64 {
    let run = |mode: RefMode| {
        let br = Brush {
            verb: Verb::Flatten,
            mode,
            radius,
            strength: 1.0,
            falloff: Falloff::Constant,
            ..Brush::default()
        };
        let mut m = sphere.clone();
        let mut s = SculptStroke::default();
        s.begin(&m);
        s.dab(
            &mut m,
            &br,
            &Dab::at(center, radius, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
        m
    };
    let a = run(RefMode::S);
    let b = run(RefMode::L);
    a.positions()
        .iter()
        .zip(b.positions())
        .map(|(p, q)| {
            (f64::from(p[0] - q[0]).powi(2)
                + f64::from(p[1] - q[1]).powi(2)
                + f64::from(p[2] - q[2]).powi(2))
            .sqrt()
        })
        .fold(0.0f64, f64::max)
}

/// **A DIVERGÊNCIA ENTRE OS DOIS MODOS SEGUE A CURVATURA — e some com ela.**
///
/// ⚠️ **A minha primeira versão deste gate afirmava *"numa superfície PLANA os
/// dois modos concordam"* e nasceu VERMELHA sobre produto correto**, com o
/// desvio a medir `0,007362` contra uma barra de `0,0016` que eu escolhera como
/// *"uma ordem abaixo da sagita"*. O produto estava certo e a premissa é que era
/// falsa: uma esfera de raio 20 vista por um dab de 0,8 **não é plana** — a
/// sagita da pegada é `ρ²/2R = 0,016`, o quadric captura-a fielmente, e
/// `0,007362` é metade dela. *Não existe "plano o suficiente" que não seja um
/// número escolhido a dedo.*
///
/// ⇒ A propriedade sem limiar é a RELAÇÃO: a altura de um quadric sobre uma
/// esfera é `≈ κ·ρ²/2`, **linear na curvatura**, então quadruplicar o raio da
/// esfera tem de dividir a divergência por ~4. É isso que prova o colapso no
/// plano sem ninguém decidir onde o plano começa.
///
/// ⚠️ **E o CONTROLE é a primeira asserção:** sem ela, um `l-mode` que não
/// fizesse nada teria divergência zero nas duas curvaturas e a razão seria
/// `0/0` — a forma em que *"colapsa"* e *"nunca existiu"* são indistinguíveis.
#[test]
fn the_divergence_from_the_plane_follows_the_curvature() {
    let tight = shapes::uv_sphere(64, 96, 1.0);
    let d_tight = modes_diverge_by(&tight, [0.0, 0.0, 1.0], 0.4);

    // ⚠️ **A tesselação sobe com o raio de propósito:** a `64×96` uma esfera de
    // raio 4 tem os vértices a ~0,2 uns dos outros e um dab de 0,4 apanharia
    // um punhado — a comparação passaria a falar da densidade, não da curva.
    let loose = shapes::uv_sphere(256, 384, 4.0);
    let d_loose = modes_diverge_by(&loose, [0.0, 0.0, 4.0], 0.4);

    assert!(
        d_tight > 0.01,
        "o CONTROLE: numa esfera curva os dois modos TÊM de divergir ({d_tight:.6})"
    );
    assert!(
        d_loose < d_tight / 2.5,
        "quatro vezes menos curvatura tem de dar ~quatro vezes menos divergência: \
         {d_tight:.6} (R=1) contra {d_loose:.6} (R=4)"
    );
}

/// **O OFFSET LEVANTA A SUPERFÍCIE, exactamente como levantava o plano.**
///
/// ⚠️ **Este gate nasceu de uma MUTAÇÃO SOBREVIVENTE, e o que ela expôs era um
/// controle morto:** ajustando o quadric contra o ponto já deslocado, o `c0`
/// absorve o offset e o `signed_distance` subtrai-o de volta — o alvo sai o
/// MESMO e o knob `plane_offset` fica inerte sob o `l-mode`. Nenhuma fixture da
/// suíte usava offset com o `L`, então o knob morto era invisível.
///
/// ⚠️ **O oráculo é a DIFERENÇA de alturas de quem POUSOU, e a minha primeira
/// versão media TODOS os vértices da pegada — reprovando com um erro
/// exactamente igual ao levantamento pedido, ou seja com alguém a não levantar
/// nada.** A premissa era falsa: o `l-mode` herda `PlaneReach::OneSided`, então
/// um verbo que só morde um lado **não toca** quem já está abaixo do alvo, e
/// levantar o alvo só aumenta esse conjunto. O paper de Alexa fala da
/// SUPERFÍCIE e não diz nada sobre que lado o pincel morde — as duas leis são
/// ortogonais, e o gate tem de respeitar a que ele não está a testar.
///
/// ⇒ Quem pousou na superfície levantada pousou `k·raio` acima de onde pousaria
/// na não-levantada, e é isso que se afirma.
#[test]
fn the_offset_lifts_the_local_surface_the_way_it_lifted_the_plane() {
    let base = shapes::uv_sphere(64, 96, 1.0);
    let d = dab();
    let run = |off: f32| {
        let br = Brush {
            plane_offset: off,
            ..flat_brush(RefMode::L)
        };
        let mut m = base.clone();
        let mut s = SculptStroke::default();
        s.begin(&m);
        s.dab(&mut m, &br, &d, Symmetry::default());
        m
    };
    // ⚠️ **O offset é NEGATIVO, e a primeira versão positiva pousou ZERO
    // vértices** — o controle a fazer o trabalho dele. Um Flatten `OneSided`
    // morde o lado `d > 0`, ou seja RASPA; levantar o alvo acima de toda a
    // calote deixa-o sem nada para raspar, e um dab que não mexe em nada é uma
    // fixture que não contém o fenómeno. Baixar o alvo põe a pegada inteira
    // acima dele.
    let flat = run(0.0);
    let moved = run(-0.25);

    let mut scratch = QueryScratch::default();
    let mut ids = Vec::new();
    base.verts_in_sphere(d.center, d.radius, &mut scratch, &mut ids);
    let want = f64::from(-0.25 * d.radius);
    let stirred = |m: &Mesh, i: u32| {
        let a = base.positions()[i as usize];
        let b = m.positions()[i as usize];
        f64::from((b[0] - a[0]).abs() + (b[1] - a[1]).abs() + (b[2] - a[2]).abs()) > 1e-6
    };
    // ⚠️ **A INTERSEÇÃO, e não os movidos de um dos lados:** só quem pousou nos
    // DOIS alvos tem os dois pousos para comparar. Quem o `off = 0` deixou
    // quieto não tem posição de referência, e incluí-lo mediria *"quanto este
    // vértice andou"* em vez de *"a que distância os dois alvos estão"*.
    let mut landed = 0usize;
    let mut worst = 0.0f64;
    for &i in &ids {
        if !stirred(&flat, i) || !stirred(&moved, i) {
            continue;
        }
        landed += 1;
        let a = flat.positions()[i as usize];
        let b = moved.positions()[i as usize];
        worst = worst.max((f64::from(b[2] - a[2]) - want).abs());
    }
    assert!(
        landed > 20,
        "o CONTROLE: os dois dabs têm de pousar barro no mesmo sítio ({landed})"
    );
    assert!(
        worst < want.abs() * 0.15,
        "o offset tem de mover a superfície por {want:.6}; pior erro {worst:.6} \
         sobre {landed} vértices"
    );
}

/// **O AJUSTE É A MESMA SUPERFÍCIE EM QUALQUER TAMANHO DE PINCEL** — o que a
/// normalização por raio compra.
///
/// ⚠️ **O doc do [`crate::stroke`]`::surface::Quadric` já NOMEAVA este gate e ele
/// não existia** — a mutação que trocava o `1/raio` por `1` sobreviveu à suíte
/// inteira, porque toda fixture usava raio `0,4`, onde `u⁴ ≈ 0,026` e o sistema
/// nem chega perto de degenerar. As equações normais elevam as coordenadas à
/// QUARTA potência, então o defeito só existe onde o pincel é grande.
///
/// ⚠️ **O oráculo é a INVARIÂNCIA DE ESCALA:** a mesma cena a `k` vezes o
/// tamanho, esculpida por um pincel `k` vezes maior, tem de dar o mesmo desenho
/// a menos da escala.
#[test]
fn the_fit_is_the_same_surface_at_any_brush_size() {
    let shape = |scale: f32| {
        let m0 = shapes::uv_sphere(64, 96, scale);
        let br = Brush {
            radius: 0.4 * scale,
            ..flat_brush(RefMode::L)
        };
        let d = Dab::at([0.0, 0.0, scale], 0.4 * scale, [0.0, 0.0, -1.0]);
        let mut m = m0.clone();
        let mut s = SculptStroke::default();
        s.begin(&m);
        s.dab(&mut m, &br, &d, Symmetry::default());
        // O deslocamento de cada vértice, em fração do raio do dab: adimensional,
        // logo comparável entre escalas.
        m0.positions()
            .iter()
            .zip(m.positions())
            .map(|(a, b)| {
                f64::from(
                    ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt(),
                ) / f64::from(0.4 * scale)
            })
            .collect::<Vec<_>>()
    };
    let small = shape(1.0);
    // ⚠️ **A escala vai para BAIXO, e a primeira versão ia para cima (×100) —
    // onde a mutação que apaga a normalização SOBREVIVIA.** Medido (sonda
    // `where_the_unnormalised_fit_starts_to_lie`), o `f64` do solver absorve o
    // mal-condicionamento até um dab de raio **400 000**: o desvio fica em
    // `2e-16`. O que a normalização de facto compra não é precisão, é o
    // **PISO DE PIVÔ ser livre de escala de cena** — sem ela, um pincel PEQUENO
    // põe os termos de quarta ordem abaixo do piso, o ajuste é recusado, e o
    // `l-mode` colapsa no `s-mode` **em silêncio**.
    let big = shape(0.001);
    let worst = small
        .iter()
        .zip(&big)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f64, f64::max);
    assert!(
        worst < 1e-3,
        "mil vezes menor tem de esculpir a mesma forma: pior desvio relativo {worst:.6}"
    );
}

/// **O `s-mode` E O `b-mode` NÃO CARREGAM SUPERFÍCIE**, e é isso que os mantém
/// no caminho que já shipava.
///
/// ⚠️ **Gate ESTRUTURAL, e ele é o par do comportamental acima:** o
/// `signed_distance` não ramifica em modo, então a única coisa que separa o
/// mundo antigo do novo é a presença do quadric. Um censo sobre a porta é o que
/// impede um verbo novo de herdar a superfície por engano.
#[test]
fn the_surface_is_fitted_for_the_four_plane_verbs_of_the_literature_mode_and_no_one_else() {
    for verb in Verb::ALL {
        for mode in RefMode::ALL {
            let want = mode == RefMode::L
                && matches!(verb, Verb::Flatten | Verb::Fill | Verb::Scrape | Verb::Clay);
            assert_eq!(
                mode.fits_local_surface(verb),
                want,
                "{verb:?}/{mode:?}: a superfície local é dos quatro verbos de plano sob o L"
            );
        }
    }
}
