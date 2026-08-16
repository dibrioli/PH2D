//! **OS GATES DA LÂMINA EM V** — [`Verb::MultiplaneScrape`], o
//! `multiplane_scrape.cc`.
//!
//! ⚠️ **O oráculo é o PERFIL da secção transversal**, e não um ajuste de plano
//! 3D: a superfície cortada **não é um plano em toda a pegada** (a projeção é
//! ponderada pelo falloff, e na borda o vértice caminha só uma fração do caminho
//! até o plano), então um ajuste sobre a pegada inteira mede a MISTURA. A sonda
//! `tests/measure_multiplane_scrape.rs` pagou esse erro primeiro — ela devolvia
//! `14,5° · 17,2° · 12,4°` para ângulos autorados de `15° · 30° · 45°`.
//!
//! ⚠️ **E as barras saem DELA**, nunca de um número escolhido; o gate do default
//! é o único que não pode citar nenhum dos dois.

use super::*;

/// ⚠️ **Mais fina que a esfera dos outros gates, e a razão é a MESMA da sonda:**
/// o perfil é lido em bandas estreitas ao longo do eixo que atravessa o traço, e
/// na malha de `96×144` cada banda apanha **dois** vértices. Uma fixture que
/// amostra menos que a estrutura que mede não mede nada.
fn sphere() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(160, 240, 1.0)
}

/// ⚠️ **O polo desta esfera é `+Y`** — `[0, 0, 1]` é um ponto do EQUADOR, cuja
/// normal é `+z`, que é o que a fixture precisa.
const TIP: [f32; 3] = [0.0, 0.0, 1.0];
const R: f32 = 0.35;
const STEP: f32 = 0.06;
const DABS: usize = 20;

/// ⚠️ **O `accumulate` é DERIVADO do verbo, e a primeira versão desta fixture
/// não o fazia** — `..Brush::default()` carrega o do [`Verb::Draw`] (que é
/// `true`), e a lei do plano CONGELADO é `!accumulate`: com o flag herdado o
/// ramo do `pre` **nunca corria**, e a mutação que tira este verbo da lista de
/// congelados saía **byte-idêntica**. É o que o painel faz ao trocar de verbo;
/// uma fixture que não o faz descreve um pincel que o artista não tem.
fn brush(verb: Verb) -> Brush {
    Brush {
        verb,
        radius: R,
        strength: 1.0,
        accumulate: verb.default_accumulate(),
        ..Brush::default()
    }
}

/// Um traço que corre sobre a esfera num arco de círculo máximo e TERMINA no
/// ponto de referência, opcionalmente girado `turn` radianos em torno da normal
/// local.
///
/// ⚠️ **Os centros correm SOBRE a superfície**, e não numa reta: numa reta os
/// dabs de trás flutuam FORA da malha e o corte medido passa a ser função de
/// quão longe o dab se afastou, não do ângulo do V.
fn walk_turned(b: &Brush, turn: f32, sym: Symmetry) -> Mesh {
    let mut mesh = sphere();
    let (st, ct) = turn.sin_cos();
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for k in 0..DABS {
        let phi = (DABS - 1 - k) as f32 * STEP * R;
        // O deslocamento no plano tangente em `TIP`, girado por `turn`.
        let (sp, cp) = phi.sin_cos();
        let c = [sp * ct, sp * st, cp];
        let eye = [-c[0], -c[1], -c[2]];
        let d = Dab::pulling(c, R, eye, [0.0; 3]);
        s.dab(&mut mesh, b, &d, sym);
    }
    mesh
}

fn walk(b: &Brush) -> Mesh {
    walk_turned(b, 0.0, Symmetry::default())
}

/// A altura média da secção transversal numa banda de `|u|`, onde `u` é a
/// coordenada que ATRAVESSA o traço (girada por `turn`).
fn band(mesh: &Mesh, turn: f32, lo: f32, hi: f32) -> Option<f64> {
    let (st, ct) = turn.sin_cos();
    let (mut acc, mut n) = (0.0f64, 0usize);
    for p in mesh.positions() {
        if p[2] <= 0.0 {
            continue;
        }
        // ao longo do traço · atravessando o traço
        let along = p[0] * ct + p[1] * st;
        let across = (-p[0] * st + p[1] * ct).abs();
        if along.abs() < 0.20 * R && across >= lo * R && across < hi * R {
            acc += f64::from(p[2]);
            n += 1;
        }
    }
    (n >= 4).then(|| acc / n as f64)
}

/// **A CRISTA**: quanto o meio do sulco ficou ACIMA dos flancos, em raios,
/// **contra a esfera em repouso**.
///
/// ⚠️ **A subtração do repouso é load-bearing** — a esfera intocada já tem o
/// meio acima dos lados, e um valor absoluto aqui mediria a fixture. A sonda
/// reportou `0,0548` de crista num traço que moveu **zero** vértices antes de
/// isto existir.
fn ridge(mesh: &Mesh, turn: f32) -> f64 {
    let rest = sphere();
    let now = (band(mesh, turn, 0.0, 0.15), band(mesh, turn, 0.45, 0.65));
    let was = (band(&rest, turn, 0.0, 0.15), band(&rest, turn, 0.45, 0.65));
    match (now, was) {
        ((Some(a), Some(b)), (Some(c), Some(d))) => ((a - b) - (c - d)) / f64::from(R),
        _ => panic!("bandas vazias — a fixture não contém a secção"),
    }
}

fn moved(mesh: &Mesh) -> usize {
    let rest = sphere();
    mesh.positions()
        .iter()
        .zip(rest.positions())
        .filter(|(p, r)| {
            let d = [p[0] - r[0], p[1] - r[1], p[2] - r[2]];
            d[0].abs() + d[1].abs() + d[2].abs() > 1.0e-5
        })
        .count()
}

/// **O sulco tem DUAS facetas com uma aresta no meio, e o Scrape não.**
///
/// ⚠️ O [`Verb::Scrape`] é o CONTROLE, e não um vizinho decorativo: ele é o
/// mesmo verbo com o V fechado e a origem no centro de ÁREA, então ele responde
/// *"quanto desta forma é a lâmina, e quanto é raspar"*.
#[test]
fn the_vee_leaves_a_ridge_where_the_scrape_leaves_a_floor() {
    let vee = ridge(&walk(&brush(Verb::MultiplaneScrape)), 0.0);
    let flat = ridge(&walk(&brush(Verb::Scrape)), 0.0);
    assert!(
        vee > 0.10,
        "a lâmina em V tem de deixar crista: {vee:.4} raios"
    );
    assert!(
        flat < 0.02,
        "o Scrape achata, não crista: {flat:.4} raios (controle)"
    );
}

/// **O PINCEL DE FÁBRICA CORTA UM V.**
///
/// ⚠️ **Ele não menciona [`crate::DEFAULT_MULTIPLANE_ANGLE_DEG`] nem o número
/// dele** — um default só é testado por um teste que não o nomeia. A metade
/// oposta (zerar o knob) é o que impede o gate de passar por vácuo: se a crista
/// viesse da geometria da esfera em vez do V, as duas leituras seriam iguais.
#[test]
fn the_factory_angle_cuts_a_vee_and_not_a_flat_floor() {
    let factory = brush(Verb::MultiplaneScrape);
    let closed = Brush {
        scrape_angle_deg: 0.0,
        ..factory.clone()
    };
    let with = ridge(&walk(&factory), 0.0);
    let without = ridge(&walk(&closed), 0.0);
    assert!(
        with > 0.10,
        "o pincel de fábrica tem de cortar um V visível: {with:.4} raios"
    );
    assert!(
        without.abs() < 0.01,
        "com o V fechado não há sulco nenhum: {without:.4} raios"
    );
}

/// **E com o V fechado a ferramenta não move UM vértice** — o mecanismo é a
/// ORIGEM (o plano tangente ao cursor), não a força.
///
/// ⚠️ É o gate que torna o `0` do `DNA_brush_types.h` inutilizável como default,
/// e por isso vive ao lado do de cima em vez de dentro dele: um afirma a
/// APARÊNCIA, o outro o mecanismo.
#[test]
fn a_closed_vee_lays_nothing_while_the_scrape_still_cuts() {
    let closed = Brush {
        scrape_angle_deg: 0.0,
        ..brush(Verb::MultiplaneScrape)
    };
    assert_eq!(
        moved(&walk(&closed)),
        0,
        "o plano tangente não tem nada acima dele"
    );
    assert!(
        moved(&walk(&brush(Verb::Scrape))) > 200,
        "o Scrape é o controle: ele corta contra o plano de ÁREA"
    );
}

/// **A DOBRADIÇA É O TRAÇO**, e girar o gesto gira o sulco.
///
/// ⚠️ **É a propriedade que separa este verbo do [`Verb::ClayThumb`]:** os dois
/// inclinam a mesma normal de área, e o que os distingue é *em torno de quê*.
/// Um gate que só medisse a crista num traço horizontal passaria com o eixo
/// trocado — e é exatamente essa mutação que este mata.
#[test]
fn the_vee_hinges_along_the_stroke_not_across_it() {
    use std::f32::consts::FRAC_PI_2;
    let b = brush(Verb::MultiplaneScrape);
    let turned = walk_turned(&b, FRAC_PI_2, Symmetry::default());
    // Medido no frame do traço GIRADO: a crista tem de continuar lá.
    let along_new = ridge(&turned, FRAC_PI_2);
    // Medido no frame ANTIGO: ali o sulco corre AO LONGO da secção, e a crista
    // dele não aparece.
    let along_old = ridge(&turned, 0.0);
    assert!(
        along_new > 0.10,
        "o sulco tem de acompanhar o traço: {along_new:.4} raios"
    );
    assert!(
        along_new > along_old * 3.0,
        "e ser MUITO mais nítido no frame do traço ({along_new:.4}) do que no antigo ({along_old:.4})"
    );
}

/// **O Ctrl vira o telhado em VALE.**
///
/// ⚠️ Não é *"o mesmo mais fraco"*: o ângulo troca de sinal, as normais tombam
/// ao contrário **e o culling de lado se desliga** (`if (angle >= 0.0f)`), então
/// a projeção passa a ser bilateral. A crista muda de SINAL, que é o oráculo.
#[test]
fn the_ctrl_turns_the_roof_into_a_valley() {
    let up = ridge(&walk(&brush(Verb::MultiplaneScrape)), 0.0);
    let down = ridge(
        &walk(&Brush {
            invert: true,
            ..brush(Verb::MultiplaneScrape)
        }),
        0.0,
    );
    assert!(up > 0.10, "o telhado: {up:.4}");
    assert!(down < -0.10, "o vale: {down:.4}");
}

/// **Sem direção não há dobradiça, logo não há depósito** — os dois `return` da
/// referência, pela porta única [`super::target::stroke_axis`].
#[test]
fn a_scrape_without_a_path_lays_nothing() {
    let mut mesh = sphere();
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    // Um dab isolado: `Dab::pulling` com origem no próprio centro ⇒ caminho nulo.
    let d = Dab::pulling(TIP, R, [0.0, 0.0, -1.0], [0.0; 3]);
    s.dab(
        &mut mesh,
        &brush(Verb::MultiplaneScrape),
        &d,
        Symmetry::default(),
    );
    assert_eq!(moved(&mesh), 0, "um plano inclinado precisa de um eixo");
}

/// **A LÂMINA É MAIS ESTREITA AO LONGO DO TRAÇO DO QUE ATRAVÉS DELE.**
///
/// ⚠️ **A fixture é de DOIS dabs, e a primeira versão media o TRAÇO** — com
/// vinte dabs a extensão "ao longo" é o comprimento do gesto (0,52 contra 0,33
/// de largura), e o gate reprovava um produto correto afirmando o contrário do
/// que a lei diz. Uma lâmina é propriedade de **um dab**; o primeiro não
/// deposita (não tem caminho), então dois dabs coladinhos são exactamente uma.
///
/// ⚠️ **O oráculo é a EXTENSÃO do depósito**, nunca
/// [`crate::MULTIPLANE_TIP_STRETCH`] — a razão medida não é `1/k` porque o
/// falloff e o corte do meio-plano também pesam; o que a lei promete é a ORDEM.
#[test]
fn the_blade_is_squeezed_along_the_stroke() {
    let mut mesh = sphere();
    let b = brush(Verb::MultiplaneScrape);
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for k in 0..2 {
        let phi = (1 - k) as f32 * 0.02 * R;
        let (sp, cp) = phi.sin_cos();
        let c = [sp, 0.0, cp];
        let d = Dab::pulling(c, R, [-c[0], -c[1], -c[2]], [0.0; 3]);
        s.dab(&mut mesh, &b, &d, Symmetry::default());
    }
    let rest = sphere();
    let (mut max_along, mut max_across) = (0.0f32, 0.0f32);
    for (p, r) in mesh.positions().iter().zip(rest.positions()) {
        let d = [p[0] - r[0], p[1] - r[1], p[2] - r[2]];
        if d[0].abs() + d[1].abs() + d[2].abs() <= 1.0e-5 {
            continue;
        }
        max_along = max_along.max(r[0].abs());
        max_across = max_across.max(r[1].abs());
    }
    assert!(
        max_across > max_along * 1.3,
        "a lâmina corta larga e curta: através {max_across:.4} · ao longo {max_along:.4}"
    );
}

/// **E a moldura sozinha diz o mesmo**, sem passar pela malha.
///
/// ⚠️ **Os dois gates não são redundantes:** este afirma a FORMA (a porta é
/// pura, o número é exato) e o de cima afirma que ela **chega ao barro** — um
/// dia em que o `dab_core` deixasse de montar a lâmina, este continuaria verde.
#[test]
fn the_blade_frame_reaches_further_across_than_along() {
    let along = [1.0, 0.0, 0.0];
    let blade = crate::Blade::new([0.0; 3], along).expect("eixo válido");
    let fp = crate::Footprint::Blade(blade);
    let d = 0.5f32;
    let (t_along, _) = fp.at([d, 0.0, 0.0], d, 1.0);
    let (t_across, _) = fp.at([0.0, d, 0.0], d, 1.0);
    assert!(
        t_along > t_across * 1.5,
        "à mesma distância, o eixo do traço conta MAIS: ao longo {t_along:.4} · através {t_across:.4}"
    );
    // ⚠️ **O controle:** um disco não distingue os dois eixos, e é ele que
    // separa *"a lâmina aperta"* de *"a distância cresce"*.
    let disc = crate::Footprint::Disc;
    assert_eq!(
        disc.at([d, 0.0, 0.0], d, 1.0).0,
        disc.at([0.0, d, 0.0], d, 1.0).0
    );
}

/// **O modo dinâmico LÊ a superfície, e o Ctrl nele ACHATA em vez de inverter.**
///
/// ⚠️ A referência escreve o porquê no próprio comentário — *"so you can trim
/// plane surfaces without changing the brush"* —, e é isso que separa este gate
/// do [`the_ctrl_turns_the_roof_into_a_valley`]: **o mesmo gesto tem
/// significados diferentes nos dois modos**, e um gate só encontra isso se
/// medir os dois.
#[test]
fn the_ctrl_flattens_in_dynamic_mode_instead_of_inverting() {
    let dyn_up = ridge(
        &walk(&Brush {
            scrape_dynamic: true,
            ..brush(Verb::MultiplaneScrape)
        }),
        0.0,
    );
    let dyn_ctrl = ridge(
        &walk(&Brush {
            scrape_dynamic: true,
            invert: true,
            ..brush(Verb::MultiplaneScrape)
        }),
        0.0,
    );
    assert!(dyn_up > 0.10, "o dinâmico também abre um V: {dyn_up:.4}");
    assert!(
        dyn_ctrl > -0.10,
        "e o Ctrl nele NÃO cava um vale — ele achata: {dyn_ctrl:.4}"
    );
}

/// **COM O KNOB EM ZERO O MODO DINÂMICO AINDA CORTA — porque ele LEU a
/// superfície.**
///
/// ⚠️ **É o gate que faltava, e a mutação que o pediu sobreviveu a nove:**
/// fazer as duas amostras lerem o MESMO lado deixa o ângulo medido em zero, e
/// com o knob somando 60° o V continua igual — *a ferramenta parece funcionar
/// enquanto ignora a superfície inteira*. Zerando o knob, tudo o que sobra é a
/// leitura, e o modo fixo é o CONTROLE que a torna visível (ele não move um
/// vértice).
#[test]
fn the_dynamic_mode_cuts_with_the_knob_at_zero_because_it_read_the_surface() {
    let flat = Brush {
        scrape_angle_deg: 0.0,
        ..brush(Verb::MultiplaneScrape)
    };
    let read = Brush {
        scrape_dynamic: true,
        ..flat.clone()
    };
    assert_eq!(
        moved(&walk(&flat)),
        0,
        "sem knob e sem leitura, nada acontece"
    );
    assert!(
        moved(&walk(&read)) > 200,
        "a leitura da superfície sozinha tem de abrir o V: {} vértices",
        moved(&walk(&read))
    );
}

/// **A abertura do V não sobrevive ao fim do traço.**
///
/// ⚠️ No modo dinâmico ela é MEMÓRIA ([`crate::MULTIPLANE_ANGLE_SMOOTH`]), então
/// herdá-la faria o primeiro dab do traço seguinte raspar com o ângulo que a
/// superfície tinha noutro lugar — a mesma lei do `last_center` e da inclinação
/// do polegar.
#[test]
fn a_new_stroke_starts_the_vee_over() {
    let mut mesh = sphere();
    let b = Brush {
        scrape_dynamic: true,
        ..brush(Verb::MultiplaneScrape)
    };
    let mut s = SculptStroke::default();
    // Um traço inteiro para carregar a memória…
    s.begin(&mesh);
    for k in 0..DABS {
        let phi = (DABS - 1 - k) as f32 * STEP * R;
        let (sp, cp) = phi.sin_cos();
        let c = [sp, 0.0, cp];
        let d = Dab::pulling(c, R, [-c[0], -c[1], -c[2]], [0.0; 3]);
        s.dab(&mut mesh, &b, &d, Symmetry::default());
    }
    let carried = s.scrape_angle_deg;
    assert!(carried.abs() > 1.0, "o traço carregou um ângulo: {carried}");
    // …e o `begin` do traço seguinte tem de o esquecer.
    s.begin(&mesh);
    assert_eq!(s.scrape_angle_deg, 0.0);
}
