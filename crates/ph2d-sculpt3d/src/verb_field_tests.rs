//! **OS CINCO VERBOS QUE O CAMPO ELÁSTICO SERVE** — a W5-B, irmã do
//! [`super::verb_move_field`] que fechou o Grab.
//!
//! ⚠️ **O corte entre os dois arquivos é o do PAPER, não o do tamanho:** lá mora
//! a família do agarre (eq. 5, um campo que puxa); aqui as três famílias AFINS
//! (`twist`/`scale`/`pinch`, que são a derivada direcional dela) mais o Snake
//! Hook, que reusa o agarre com a âncora a andar.
//!
//! ⚠️ **E o que estes gates medem não é o kernel — é o BARRO.** Os kernels já
//! têm gates próprios no [`crate::kelvinlet`]; o que faltava era saber se o verbo
//! CONSOME o campo que a tabela lhe declara, e se o consome pela geometria certa.
//! As duas perguntas têm um gate cada.

use crate::{Brush, Dab, RefMode, SculptStroke, Symmetry, Verb};

fn sphere() -> ph2d_mesh::Mesh {
    ph2d_mesh::shapes::uv_sphere(32, 48, 1.0)
}

const TIP: [f32; 3] = [0.0, 0.0, 1.0];
const EYE: [f32; 3] = [0.0, 0.0, -1.0];
const R: f32 = 0.5;

/// Um traço de doze eventos com o gesto a crescer — o caminho do produto.
///
/// ⚠️ **Doze e não um**, e a razão é a §7.11: o defeito do `b-mode` do Grab só
/// existia a partir do quinto evento, e um dab solto o teria declarado são.
fn stroke(verb: Verb, mode: RefMode, amount: f32, pull: [f32; 3]) -> ph2d_mesh::Mesh {
    let mut mesh = sphere();
    let b = Brush {
        verb,
        mode,
        radius: R,
        strength: 1.0,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for k in 1..=12 {
        let t = k as f32 / 12.0;
        let d = match verb {
            Verb::Move => Dab::pulling(TIP, R, EYE, [pull[0] * t, pull[1] * t, pull[2] * t]),
            Verb::SnakeHook => Dab::pulling(
                [TIP[0] + pull[0] * t, TIP[1], TIP[2]],
                R,
                EYE,
                [pull[0] / 12.0, pull[1] / 12.0, pull[2] / 12.0],
            ),
            _ => {
                let mut d = Dab::pulling(TIP, R, EYE, [0.0; 3]);
                d.amount = amount * t;
                d
            }
        };
        s.dab(&mut mesh, &b, &d, Symmetry::default());
    }
    mesh
}

/// O gesto que faz cada verbo de campo mexer-se — o mínimo para o modo ser
/// observável, e nada além.
fn gesture(verb: Verb) -> (f32, [f32; 3]) {
    match verb {
        Verb::Move | Verb::SnakeHook => (0.0, [0.35, 0.0, 0.0]),
        Verb::Twist => (1.2, [0.0; 3]),
        Verb::LocalScale => (0.5, [0.0; 3]),
        _ => (0.0, [0.0; 3]),
    }
}

/// O deslocamento médio numa banda de distância à âncora, em raios de pincel.
///
/// ⚠️ **É o oráculo que uma SOMA sobre a pegada não é.** Uma mutação que tire o
/// braço de campo de um verbo deixa o alvo cair no modo que já shipava — e a
/// PEGADA continua a do campo, com a curva-indicadora a valer `1` em toda ela.
/// Somando, a pegada 3× mais larga **finge o sinal**; medindo por banda, ela se
/// denuncia: sem perfil o aro anda tanto quanto o bico.
fn band_displacement(rest: &ph2d_mesh::Mesh, out: &ph2d_mesh::Mesh, lo: f32, hi: f32) -> f32 {
    let (a, b) = (rest.positions(), out.positions());
    let (mut sum, mut n) = (0.0f32, 0usize);
    for i in 0..a.len() {
        let r = [a[i][0] - TIP[0], a[i][1] - TIP[1], a[i][2] - TIP[2]];
        let rm = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
        if rm < lo * R || rm >= hi * R {
            continue;
        }
        let d = [b[i][0] - a[i][0], b[i][1] - a[i][1], b[i][2] - a[i][2]];
        sum += (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        n += 1;
    }
    if n == 0 { 0.0 } else { sum / n as f32 }
}

/// **TODO CAMPO DECLARADO ALCANÇA O BARRO** — a lei §3 do plano (*cada l-mode
/// nasce CANDIDATO e só ganha o chip depois de MEDIDO*), executável.
///
/// ⚠️ **A barra é a do §3** — 1 ULP de `f32` é o piso de paridade, e um chip que
/// difere por menos que isso é um botão que não faz nada.
///
/// ⚠️ **E ele tem uma SEGUNDA metade, que a mutação obrigou a existir.** A
/// primeira versão dizia, no doc, que ele pegava um par (verbo, campo) trocado
/// — e a mutação que o instalou **passou nos 193**: o alvo caía no modo que já
/// shipava, mas a PEGADA continuava a do campo (o
/// [`crate::Brush::query_radius`] pergunta só `is_some`), então `L` diferia de
/// `S` na mesma, por uma razão que não era a lei. *Um gate que mede diferença
/// não distingue de onde ela vem.*
///
/// ⇒ Duas curas, e a primeira é estrutural: o *qual* mudou-se para o VERBO
/// ([`crate::Verb::elastic_field`]) e o par deixou de poder discordar. O que
/// sobra é o verbo que declara um campo e cujo ALVO ninguém escreveu — um
/// `l-mode` com o alcance e sem a lei —, e contra isso a defesa é um **CENSO**:
/// todo verbo desta família tem de ter aqui um gate que pina a FORMA do que ele
/// faz. Um sexto verbo entra por esta linha ou não entra.
#[test]
fn every_declared_field_reaches_the_clay() {
    for verb in Verb::ALL {
        if RefMode::L.field(verb).is_none() {
            continue;
        }
        let (amount, pull) = gesture(verb);
        let s = stroke(verb, RefMode::S, amount, pull);
        let l = stroke(verb, RefMode::L, amount, pull);
        let (a, b) = (s.positions(), l.positions());
        // ⚠️ **O CONTROLE VEM PRIMEIRO, e a ordem custou uma corrida:** com ele
        // depois, um gesto que não move barro nenhum falha a afirmação de cima
        // com o diagnóstico ERRADO (*"o campo não alcança"* quando o que faltou
        // foi o puxão). Um gate diz primeiro que a fixture contém o fenômeno.
        let rest = sphere();
        let moved = (0..a.len())
            .filter(|&i| {
                let r = rest.positions()[i];
                let d = [b[i][0] - r[0], b[i][1] - r[1], b[i][2] - r[2]];
                (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() > 1e-6
            })
            .count();
        assert!(
            moved > 50,
            "{}: o gesto do gate não move barro nenhum ({moved} vértices)",
            verb.label()
        );
        let worst = (0..a.len())
            .map(|i| {
                let d = [b[i][0] - a[i][0], b[i][1] - a[i][1], b[i][2] - a[i][2]];
                (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
            })
            .fold(0.0f32, f32::max);
        assert!(
            worst > f32::EPSILON,
            "{}: o campo declarado ({:?}) não alcança o barro — o `l-mode` é o \
             `s-mode` ao bit, e o chip é um botão morto",
            verb.label(),
            RefMode::L.field(verb).unwrap()
        );
        assert!(
            SHAPE_GATED.contains(&verb.label()),
            "{}: declara um campo e não tem gate de FORMA neste arquivo. Um \
             `l-mode` sem alvo escrito herda a PEGADA do campo e a lei do modo \
             que já shipava — diferente do vizinho, e errado.",
            verb.label()
        );
    }
}

/// Os verbos cuja FORMA este arquivo pina, um gate cada. É o censo do
/// [`every_declared_field_reaches_the_clay`] — a lista cresce no mesmo commit
/// que o gate, nunca antes.
const SHAPE_GATED: &[&str] = &[
    // O Grab tem os dele no IRMÃO (`verb_move_field_tests`), que é onde a
    // família do agarre mora — o censo pergunta se EXISTE gate, não onde.
    "Move / Grab",
    // `the_hooked_field_carries_the_neighbourhood_and_still_follows_the_anchor`
    "Snake Hook",
    // `the_elastic_turn_turns_the_clay_instead_of_inflating_it` + o irmão do perfil
    "Twist",
    // `the_elastic_scale_falls_off_from_the_tip_to_the_rim`
    "Local Scale",
    // `the_elastic_pinch_gives_back_along_the_normal_what_it_takes_from_the_plane`
    "Pinch",
    // `the_elastic_magnify_dilates_along_the_ray_not_across_the_plane`
    "Magnify",
];

/// **O GIRO ELÁSTICO GIRA — ele não INCHA.**
///
/// ⚠️ **É o gate que justifica o [`crate::kelvinlet::rigid_profile`] existir.**
/// O campo do paper é um DESLOCAMENTO, e somá-lo à posição lineariza a rotação:
/// o vértice sai da circunferência para `|r|·√(1 + (θ·perfil)²)` — medido a meio
/// raio, **1,0408 a meio radiano e 1,5271 a dois**. Consumindo o campo como
/// ÂNGULO o verbo gira o que sempre girou e o campo decide só quanto cada
/// vértice acompanha.
///
/// ⚠️ **A barra não é folgada: é `1 + 4 ULP`.** Uma rotação preserva a distância
/// ao eixo EXATAMENTE, e o que sobra é o arredondamento do seno e do cosseno —
/// qualquer inflação de verdade é ordens de grandeza acima.
#[test]
fn the_elastic_turn_turns_the_clay_instead_of_inflating_it() {
    for (verb, amount) in [(Verb::Twist, 2.0f32)] {
        let rest = sphere();
        let out = stroke(verb, RefMode::L, amount, [0.0; 3]);
        let (a, b) = (rest.positions(), out.positions());
        let mut worst: f32 = 1.0;
        let mut moved = 0;
        for i in 0..a.len() {
            let d = [b[i][0] - a[i][0], b[i][1] - a[i][1], b[i][2] - a[i][2]];
            if (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() <= 1e-6 {
                continue;
            }
            moved += 1;
            // O eixo passa pela âncora na direção do olho ⇒ o raio é o de `xy`.
            let ra = ((a[i][0] - TIP[0]).powi(2) + (a[i][1] - TIP[1]).powi(2)).sqrt();
            let rb = ((b[i][0] - TIP[0]).powi(2) + (b[i][1] - TIP[1]).powi(2)).sqrt();
            if ra > 1e-3 {
                worst = worst.max(rb / ra);
            }
        }
        assert!(
            moved > 100,
            "{}: a fixture não contém o fenômeno",
            verb.label()
        );
        assert!(
            worst <= 1.0 + 4.0 * f32::EPSILON,
            "{}: o barro INCHOU {worst:.4}× ao girar — o campo está a ser somado \
             como deslocamento em vez de consumido como ângulo",
            verb.label()
        );
    }
}

/// **O PERFIL EXISTE: a vizinhança acompanha MENOS que o bico.**
///
/// ⚠️ **Sem ele, um campo constante passaria no gate acima** — girar tudo pelo
/// mesmo ângulo também preserva raio, e seria um giro rígido de uma bola de
/// barro em vez de uma torção elástica. As duas metades são o par mínimo.
#[test]
fn the_turn_falls_off_from_the_tip_to_the_rim() {
    let rest = sphere();
    let out = stroke(Verb::Twist, RefMode::L, 2.0, [0.0; 3]);
    let (a, b) = (rest.positions(), out.positions());
    // O ângulo varrido por vértice, contra a distância à âncora.
    let angle_at = |band: (f32, f32)| -> f32 {
        let (mut sum, mut n) = (0.0f32, 0usize);
        for i in 0..a.len() {
            let r = ((a[i][0] - TIP[0]).powi(2) + (a[i][1] - TIP[1]).powi(2)).sqrt();
            if r < band.0 * R || r >= band.1 * R || r < 1e-3 {
                continue;
            }
            let ang = (a[i][0] - TIP[0]).atan2(a[i][1] - TIP[1])
                - (b[i][0] - TIP[0]).atan2(b[i][1] - TIP[1]);
            sum += ang.abs();
            n += 1;
        }
        if n == 0 { 0.0 } else { sum / n as f32 }
    };
    let near = angle_at((0.1, 0.3));
    let far = angle_at((1.5, 2.0));
    assert!(near > 1e-3, "a fixture não contém o fenômeno perto do bico");
    assert!(
        far < near * 0.5,
        "o giro não decai ({near:.4} perto, {far:.4} longe) — o campo virou uma \
         rotação rígida, e um giro rígido não é um campo elástico"
    );
}

/// **O APERTO DEVOLVE PELA NORMAL O QUE TIRA DO PLANO** — a `F` de traço zero, e
/// o que separa um vinco de material de um furo raso.
///
/// ⚠️ **O `s-mode` é o CONTROLE e ele não é um espantalho:** ele puxa
/// lateralmente com a mesma força e devolve **0,5610** pela normal contra
/// **1,7653** do campo, com o aperto lateral praticamente igual (`−3,70` contra
/// `−3,50`). *O barro que sai de lado tem de sair por algum lugar*, e é essa
/// frase que o número mede.
///
/// ⚠️ **Mas a grandeza é a RAZÃO, não a soma — e a mutação obrigou a troca.**
/// Tirar o braço de campo deixa o alvo cair no `s-mode` **com a pegada do
/// campo**, e uma soma sobre 3× mais vértices cresce sozinha: o gate passava
/// sobre um `l-mode` sem lei. A razão `normal ÷ lateral` é adimensional e as
/// duas somas crescem juntas, então ela mede o que a `F` de traço zero faz e
/// nada mais — **0,1515** no modo que já shipava contra **0,5043** no campo.
#[test]
fn the_elastic_pinch_gives_back_along_the_normal_what_it_takes_from_the_plane() {
    let rest = sphere();
    let normal_gain = |mode: RefMode| -> (f32, f32) {
        let out = stroke(Verb::Pinch, mode, 0.0, [0.0; 3]);
        let (a, b) = (rest.positions(), out.positions());
        let (mut lat, mut nrm) = (0.0f32, 0.0f32);
        for i in 0..a.len() {
            let d = [b[i][0] - a[i][0], b[i][1] - a[i][1], b[i][2] - a[i][2]];
            if (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() <= 1e-6 {
                continue;
            }
            let rad = [a[i][0] - TIP[0], a[i][1] - TIP[1]];
            let rn = (rad[0] * rad[0] + rad[1] * rad[1]).sqrt();
            if rn > 1e-4 {
                lat += (d[0] * rad[0] + d[1] * rad[1]) / rn;
            }
            nrm += d[2];
        }
        (lat, nrm)
    };
    let (s_lat, s_nrm) = normal_gain(RefMode::S);
    let (l_lat, l_nrm) = normal_gain(RefMode::L);
    assert!(
        s_lat < -1.0 && l_lat < -1.0,
        "a fixture não aperta: s={s_lat:.4} l={l_lat:.4}"
    );
    // ⚠️ **E a razão SOZINHA não bastou** — medido, ela sobrevive a tirar o
    // braço de campo, porque o `lateral_pull` para uma âncora que está SOBRE a
    // superfície também tem componente normal, e a pegada larga a amplifica. A
    // metade que sangra é o PERFIL, a mesma do gancho: sem lei, a
    // curva-indicadora leva o gesto inteiro até ao aro.
    let rest2 = sphere();
    let l_out = stroke(Verb::Pinch, RefMode::L, 0.0, [0.0; 3]);
    let (near, rim) = (
        band_displacement(&rest2, &l_out, 0.1, 0.4),
        band_displacement(&rest2, &l_out, 1.5, 2.0),
    );
    assert!(
        near > 1e-3,
        "a fixture não aperta perto do bico ({near:.4})"
    );
    assert!(
        rim < 0.5 * near,
        "o aperto elástico não decai ({near:.4} no bico, {rim:.4} no aro) — com \
         a indicadora sobre a pegada larga isto é o modo que já shipava a vestir \
         o alcance do campo"
    );
    let (s_ratio, l_ratio) = (s_nrm / -s_lat, l_nrm / -l_lat);
    assert!(
        l_ratio > 2.0 * s_ratio,
        "o aperto elástico não espirrou pela normal (razão {l_ratio:.4} contra \
         {s_ratio:.4} do modo que já shipava) — sem o termo de traço zero ele \
         remove volume, e uma SOMA maior só diria que a pegada é maior"
    );
}

/// **A DILATAÇÃO DO MAGNIFY É RADIAL, e ela ALCANÇA ALÉM DO ANEL.**
///
/// ⚠️ **O título deste gate era outro, e a medição o derrubou.** Eu afirmava que
/// o discriminante era a DIREÇÃO — *"o campo dilata ao longo de `r`, o empurrão
/// do `s-mode` vive no plano tangente"* —, e medido o `s-mode` é radial também:
/// **cos 1,0000 nos dois**. O `lateral_pull` aponta do centro do dab para o
/// vértice, e sobre a calota o raio e a tangente são a mesma direção com a
/// precisão que importa. *Um discriminante que eu não medi é um discriminante
/// que pode não existir.*
///
/// ⇒ O que fica é o par honesto: **radialidade** como CONTROLE (uma propriedade
/// do campo *scale*, que os dois partilham e que um bug de sinal quebraria) e o
/// **ALCANCE** como discriminante — o `s-mode` para no anel do cursor, o campo
/// continua até `KELVINLET_REACH · raio`.
#[test]
fn the_elastic_magnify_dilates_along_the_ray_not_across_the_plane() {
    let rest = sphere();
    let along = |mode: RefMode| -> f32 {
        let out = stroke(Verb::Magnify, mode, 0.0, [0.0; 3]);
        let (a, b) = (rest.positions(), out.positions());
        let (mut cos_sum, mut n) = (0.0f32, 0usize);
        for i in 0..a.len() {
            let d = [b[i][0] - a[i][0], b[i][1] - a[i][1], b[i][2] - a[i][2]];
            let dm = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            if dm <= 1e-5 {
                continue;
            }
            let r = [a[i][0] - TIP[0], a[i][1] - TIP[1], a[i][2] - TIP[2]];
            let rm = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
            if rm < 0.3 * R {
                continue;
            }
            cos_sum += (d[0] * r[0] + d[1] * r[1] + d[2] * r[2]) / (dm * rm);
            n += 1;
        }
        assert!(n > 50, "a fixture não contém o fenômeno");
        cos_sum / n as f32
    };
    let l = along(RefMode::L);
    assert!(
        l > 0.999,
        "o Magnify elástico não é radial (cos médio {l:.4}) — o campo *scale* \
         desloca ao longo de `r` por construção"
    );
    // O ALCANCE, que é onde os dois modos de facto divergem.
    let rest = sphere();
    let far = |mode: RefMode| -> usize {
        let out = stroke(Verb::Magnify, mode, 0.0, [0.0; 3]);
        let (a, b) = (rest.positions(), out.positions());
        (0..a.len())
            .filter(|&i| {
                let d = [b[i][0] - a[i][0], b[i][1] - a[i][1], b[i][2] - a[i][2]];
                if (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() <= 1e-6 {
                    return false;
                }
                let r = [a[i][0] - TIP[0], a[i][1] - TIP[1], a[i][2] - TIP[2]];
                (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt() > R
            })
            .count()
    };
    let (fs, fl) = (far(RefMode::S), far(RefMode::L));
    assert_eq!(
        fs, 0,
        "o modo que já shipava passou a mover barro FORA do anel do cursor \
         ({fs} vértices) — o discriminante deste gate deixou de existir"
    );
    assert!(
        fl > 100,
        "a dilatação elástica parou no anel ({fl} vértices além dele) — a pegada \
         de um campo é `KELVINLET_REACH · raio`, e é isso que o separa do vizinho"
    );
    // ⚠️ **O PERFIL, a metade que uma mutação não finge:** alcançar o aro é do
    // `query_radius`, e ele responde a `is_some` — quem prova que a LEI chegou é
    // o campo decair dentro da própria pegada.
    let out = stroke(Verb::Magnify, RefMode::L, 0.0, [0.0; 3]);
    let (near, rim) = (
        band_displacement(&rest, &out, 0.1, 0.4),
        band_displacement(&rest, &out, 1.5, 2.0),
    );
    assert!(
        near > 1e-3,
        "a fixture não dilata perto do bico ({near:.4})"
    );
    assert!(
        rim < 0.5 * near,
        "a dilatação elástica não decai ({near:.4} no bico, {rim:.4} no aro) — \
         sem a lei, a indicadora leva o gesto inteiro até ao aro"
    );
}

/// **O GANCHO COM CAMPO CARREGA A VIZINHANÇA, e continua a seguir a âncora.**
///
/// ⚠️ **O oráculo é o PERFIL, e a primeira versão deste gate usava a CONTAGEM de
/// vértices — que a mutação fingiu.** Tirar o braço de campo faz o alvo cair no
/// modo que já shipava, mas a pegada continua `KELVINLET_REACH · raio` e a curva
/// continua a indicadora: **todo** vértice dela leva o gesto inteiro, a contagem
/// sobe na mesma, e o gate ficava verde sobre um `l-mode` sem lei. Medido por
/// banda de distância, o campo decai (`aro ÷ bico = 0,1035`) e o modo sem lei
/// não decai coisa nenhuma.
///
/// ⚠️ **As três metades, porque duas sozinhas mentem:** um campo que alcança
/// longe mas larga o bico é um gancho quebrado · um bico que segue com a
/// vizinhança parada é o `s-mode` com um chip novo · e um perfil CHATO é o modo
/// que já shipava a usar a pegada do campo.
#[test]
fn the_hooked_field_carries_the_neighbourhood_and_still_follows_the_anchor() {
    let rest = sphere();
    let pull = [0.35f32, 0.0, 0.0];
    let tip_follow = |out: &ph2d_mesh::Mesh| -> f32 {
        let (a, b) = (rest.positions(), out.positions());
        let mut best = (f32::MAX, 0.0f32);
        for i in 0..a.len() {
            let to_tip = (a[i][0] - TIP[0]).powi(2)
                + (a[i][1] - TIP[1]).powi(2)
                + (a[i][2] - TIP[2]).powi(2);
            if to_tip < best.0 {
                best = (to_tip, b[i][0] - a[i][0]);
            }
        }
        best.1
    };
    let s = stroke(Verb::SnakeHook, RefMode::S, 0.0, pull);
    let l = stroke(Verb::SnakeHook, RefMode::L, 0.0, pull);
    let (s_tip, l_tip) = (tip_follow(&s), tip_follow(&l));
    assert!(
        s_tip > 0.05,
        "a fixture não contém o fenômeno (bico {s_tip:.4})"
    );
    assert!(
        l_tip > 0.5 * s_tip,
        "o bico deixou de seguir a âncora ({l_tip:.4} contra {s_tip:.4}) — um \
         gancho que larga o barro que pegou não é um gancho"
    );
    // O ALCANCE: o `s-mode` para no anel, o campo continua.
    let s_rim = band_displacement(&rest, &s, 1.5, 2.0);
    let l_rim = band_displacement(&rest, &l, 1.5, 2.0);
    assert!(
        s_rim < 1e-4,
        "o modo que já shipava passou a alcançar o aro ({s_rim:.4})"
    );
    assert!(
        l_rim > 1e-3,
        "o campo do gancho não alcançou além do anel ({l_rim:.4})"
    );
    // O PERFIL: é ele que a mutação não consegue fingir.
    let l_near = band_displacement(&rest, &l, 0.1, 0.4);
    assert!(
        l_rim < 0.5 * l_near,
        "o gancho elástico não decai ({l_near:.4} no bico, {l_rim:.4} no aro) — \
         com a curva-indicadora sobre a pegada larga isto é o modo que já \
         shipava a vestir o alcance do campo"
    );
}
/// **A ESCALA ELÁSTICA DECAI DO BICO AO ARO** — a forma do Local Scale, e a
/// outra metade da porta escalar que o Twist estreou.
///
/// ⚠️ **Ela precisa de gate PRÓPRIO mesmo partilhando o kernel com o Twist:** o
/// que se mede num giro é o ÂNGULO varrido e o que se mede aqui é o FATOR de
/// dilatação, e um consumidor que passasse o perfil ao lugar errado da fórmula
/// (ao `dab.amount` em vez de ao `w`, digamos) deixaria o gate do giro verde. O
/// censo do [`every_declared_field_reaches_the_clay`] existe exatamente para
/// esta linha não ser esquecida.
#[test]
fn the_elastic_scale_falls_off_from_the_tip_to_the_rim() {
    let rest = sphere();
    let out = stroke(Verb::LocalScale, RefMode::L, 0.5, [0.0; 3]);
    let (a, b) = (rest.positions(), out.positions());
    // O fator de dilatação medido por banda de distância à âncora.
    let factor = |band: (f32, f32)| -> f32 {
        let (mut sum, mut n) = (0.0f32, 0usize);
        for i in 0..a.len() {
            let r = [a[i][0] - TIP[0], a[i][1] - TIP[1], a[i][2] - TIP[2]];
            let rm = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
            if rm < band.0 * R || rm >= band.1 * R || rm < 1e-3 {
                continue;
            }
            let q = [b[i][0] - TIP[0], b[i][1] - TIP[1], b[i][2] - TIP[2]];
            sum += (q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt() / rm;
            n += 1;
        }
        if n == 0 { 0.0 } else { sum / n as f32 }
    };
    let near = factor((0.1, 0.4));
    let far = factor((1.5, 2.0));
    assert!(
        near > 1.05,
        "a fixture não dilata perto do bico ({near:.4})"
    );
    assert!(
        far > 1.0 && far < 1.0 + (near - 1.0) * 0.5,
        "a dilatação não decai ({near:.4} perto, {far:.4} longe) — sem perfil \
         isto é uma escala rígida da pegada inteira, e não um campo elástico"
    );
}

/// **O CAMPO ATERRISSA NA BORDA DA PEGADA, EM VEZ DE SER DECAPITADO NELA** — o
/// report *"modo L o Falloff parece ter borda dura"* (Enio, 2026-08-14).
///
/// ⚠️ **O oráculo é o DEGRAU, não o gradiente máximo** — e a distinção custou a
/// primeira sonda desta caça. O maior salto do l-mode (0,7818 por unidade de
/// aresta) é MENOR que o do s-mode (1,1969), então um máximo global declarava o
/// l-mode *o mais liso dos dois* enquanto o aro dele saltava; quem separa é
/// comparar o último anel DENTRO da pegada com o primeiro FORA.
///
/// ⚠️ **E o CONTROLE é o que impede este gate de ser verde por vácuo:** se o
/// campo já não carregasse nada no corte, não haveria o que aterrissar e a
/// asserção passaria sem tocar em nada. Ele exige primeiro que o kernel CRU
/// ainda valha ≥ 3 % do bico ali — o `0,0347` que a família [`Scales::Tri`]
/// deixa e que a tabela do plano §7.10 escolheu.
#[test]
fn the_elastic_field_lands_at_the_rim_instead_of_being_cut() {
    use crate::kelvinlet::{Scales, grab};
    let len = |v: [f32; 3]| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();

    // O CONTROLE, primeiro: há degrau a remover?
    let f = [1.0, 0.0, 0.0];
    let at_rim = len(grab(
        [crate::KELVINLET_REACH, 0.0, 0.0],
        1.0,
        f,
        Scales::default(),
    )) / len(grab([1e-6, 0.0, 0.0], 1.0, f, Scales::default()));
    assert!(
        at_rim > 0.03,
        "sem resíduo no corte não há aterrissagem a provar, e este gate seria \
         verde por vácuo: o kernel deixa {at_rim:.4} do bico em r/eps = REACH"
    );

    let rest = sphere();
    let out = stroke(Verb::Move, RefMode::L, 0.0, [0.35, 0.0, 0.0]);
    let tip = band_displacement(&rest, &out, 0.0, 0.25);
    let cut = crate::KELVINLET_REACH;
    let inner = band_displacement(&rest, &out, cut - 0.12, cut);
    let outer = band_displacement(&rest, &out, cut, cut + 0.12);
    assert!(tip > 1e-3, "a fixture não puxa o bico ({tip:.5})");
    assert!(
        outer == 0.0,
        "fora da pegada o motor não escreve: {outer:.6}"
    );
    // ⚠️ A barra é o s-mode, que shipa e que ninguém reportou: o degrau dele
    // vale 1,57 % do bico, e ele cai NO ANEL DO CURSOR, onde um degrau lê como
    // *a borda do pincel*. O do campo cai a 3× o cursor, numa costura 11× mais
    // longa — então tem de ser MUITO menor, não meramente comparável.
    let step = (inner - outer) / tip;
    assert!(
        step < 0.003,
        "o campo foi DECAPITADO na borda da pegada: último anel dentro vale \
         {:.2} % do bico ({inner:.5} contra bico {tip:.5}), e fora é zero — \
         era 2,90 % antes da aterrissagem",
        100.0 * step
    );
}
