//! ⭐⭐⭐⭐ **OS GATES DA OPÇÃO «PUXAR PELA NORMAL»** — ordem do dono
//! (2026-09-19): *«pincéis com Snake Hook e Grab ainda não têm a opção de usar a
//! normal da superfície para dar a direção da puxada. Implemente essa opção.»*
//!
//! A lei, o neutro e o congelamento vivem em [`Brush::puxa_pela_normal`]; aqui
//! mede-se **o barro**.

use super::*;

fn esfera() -> ph2d_mesh::Mesh {
    ph2d_mesh::shapes::uv_sphere(48, 96, 1.0)
}

fn norma(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// O polo virado ao artista — ali a normal é `+z` **exacta**, que é o caso que
/// o dono descreve (*a normal a apontar para o ecrã*) e onde a componente do
/// arrasto ao longo dela é **zero**.
const POLO: [f32; 3] = [0.0, 0.0, 1.0];
const OLHO: [f32; 3] = [0.0, 0.0, -1.0];

/// Um Grab de um evento, com o puxão TANGENCIAL (no `+x`).
fn grab(pela_normal: bool, len: f32) -> (ph2d_mesh::Mesh, Vec<[f32; 3]>) {
    let mut m = esfera();
    let antes = m.positions().to_vec();
    let b = Brush {
        verb: Verb::Move,
        radius: 0.35,
        strength: 1.0,
        puxa_pela_normal: pela_normal,
        ..Brush::default()
    };
    let mut st = SculptStroke::default();
    st.begin(&m);
    st.dab(
        &mut m,
        &b,
        &Dab::pulling(POLO, b.radius, OLHO, [len, 0.0, 0.0]),
        Symmetry::default(),
    );
    (m, antes)
}

/// O vértice que mais se mexeu, e o seu deslocamento.
fn maior(m: &ph2d_mesh::Mesh, antes: &[[f32; 3]]) -> [f32; 3] {
    let mut melhor = [0.0f32; 3];
    for (p, a) in m.positions().iter().zip(antes) {
        let d = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
        if norma(d) > norma(melhor) {
            melhor = d;
        }
    }
    melhor
}

/// ⭐⭐⭐⭐ **A LEI: a direcção passa a ser a NORMAL, e o comprimento é o do
/// arrasto.**
///
/// ⛔⛔ **A fixtura é o caso em que a composição NÃO exprime isto:** no polo a
/// normal aponta ao artista e o arrasto de ecrã é perpendicular a ela, logo a
/// *componente do arrasto ao longo da normal* — a lei "óbvia" — seria **zero**.
/// É por isso que o comprimento é `‖puxão‖` e não a projecção
/// ([`Brush::puxa_pela_normal`]).
#[test]
fn com_a_opcao_ligada_o_barro_vai_pela_normal_e_nao_pelo_arrasto() {
    let len = 0.30;
    let (m_off, a_off) = grab(false, len);
    let (m_on, a_on) = grab(true, len);
    let d_off = maior(&m_off, &a_off);
    let d_on = maior(&m_on, &a_on);

    // O CONTROLO: desligada, o barro segue o arrasto (o `+x`).
    assert!(
        d_off[0] > 0.9 * len && d_off[2].abs() < 0.05 * len,
        "a fixtura nao contem o fenomeno: desligada, o maior deslocamento foi \
         {d_off:?} e devia seguir o `+x`"
    );
    // A LEI: ligada, ele segue a normal (o `+z`)…
    assert!(
        d_on[2] > 0.9 * len && d_on[0].abs() < 0.05 * len,
        "ligada, o maior deslocamento foi {d_on:?} e devia seguir a NORMAL (`+z`)"
    );
    // …e o COMPRIMENTO é o mesmo, que é a metade que impede a lei de ser
    // *«puxa pela normal com outra força»*.
    let (l_off, l_on) = (norma(d_off), norma(d_on));
    assert!(
        (l_on - l_off).abs() <= 1e-4 * l_off.max(1.0),
        "o comprimento mudou com a direccao: {l_off:.6} contra {l_on:.6}"
    );
}

/// ⭐⭐⭐ **DESLIGADA, A OPÇÃO NÃO CORRE** — e a régua é o ESTADO, não o barro.
///
/// ⛔⛔ **A 1.ª redacção deste gate era uma TAUTOLOGIA, e quem o provou foi uma
/// mutação SOBREVIVENTE** (2026-09-19): ela corria o mesmo pincel duas vezes —
/// `puxa_pela_normal: false` dos dois lados, com um `ba.puxa_pela_normal =
/// false` no meio que não mudava nada — e comparava a saída consigo própria.
/// *Uma igualdade entre duas corridas da MESMA configuração é verdadeira por
/// construção*, logo o gate ficava verde mesmo quando o caminho de omissão
/// mudava: a mutação que fazia a âncora alcançar quem **não** pediu a opção
/// passou por ele.
///
/// ⚠️ **E um GOLDEN da malha não serve aqui:** a esfera nasce de `sin`/`cos` e
/// o traço soma `f32`, logo uma impressão digital seria mais uma candidata à
/// família de flakes que atravessa os três sistemas operativos do CI.
///
/// ⇒ a propriedade EXACTA é *o código da opção não corre*: com ela desligada a
/// [`SculptStroke::ancora_do_puxao`] fica **vazia** — nenhum passe escreveu
/// âncora nenhuma. O **CONTROLO** é a mesma corrida com a opção ligada, onde
/// ela tem de ter exactamente um passe: sem ele isto passaria com a feature
/// inteira morta.
#[test]
fn desligada_a_opcao_nao_corre_e_ligada_corre() {
    for verbo in [Verb::Move, Verb::SnakeHook] {
        for liga in [false, true] {
            let mut m = esfera();
            let b = Brush {
                verb: verbo,
                radius: 0.35,
                strength: 1.0,
                puxa_pela_normal: liga,
                ..Brush::default()
            };
            let mut st = SculptStroke::default();
            st.begin(&m);
            for k in 0..6 {
                let c = [0.05 * k as f32, 0.0, 1.0];
                let d = match verbo {
                    Verb::SnakeHook => Dab::hooking(c, b.radius, OLHO, [0.05, 0.0, 0.0]),
                    _ => Dab::pulling(POLO, b.radius, OLHO, [0.05 * (k + 1) as f32, 0.0, 0.0]),
                };
                st.dab(&mut m, &b, &d, Symmetry::default());
            }
            let n = st.ancora_do_puxao.len();
            if liga {
                assert_eq!(
                    n, 1,
                    "{verbo:?}: CONTROLO — com a opcao LIGADA a ancora devia existir \
                     (um passe de simetria) e ha' {n}; o gate acima passaria com a \
                     feature inteira morta"
                );
                assert!(
                    st.ancora_do_puxao[0]
                        .as_ref()
                        .is_some_and(|a| a.normal.is_some()),
                    "{verbo:?}: CONTROLO — a ancora existe e a direccao nunca foi escrita"
                );
            } else {
                assert_eq!(
                    n, 0,
                    "{verbo:?}: o caminho de OMISSAO escreveu {n} ancora(s) — o codigo \
                     da opcao corre com ela desligada"
                );
            }
        }
    }
}

/// ⭐⭐⭐⭐ **A DIRECÇÃO CONGELA NO PEN-DOWN** — e sem isto o espigão ENROLA.
///
/// Num gancho, cada dab vira a superfície debaixo do cursor; lida viva, a normal
/// do dab `k+1` é a que o dab `k` acabou de criar. O gate mede o ÂNGULO entre o
/// primeiro e o último incremento do traço: congelada, ele é **zero**.
///
/// ⛔⛔ **O pen-down é OBLÍQUO e isso é o gate:** a 1.ª redacção arrancava no
/// POLO, e ali a normal do gesto, a normal do plano e o olho são **todos**
/// `+z` — a fixtura não distinguia a lei congelada da lei viva, e a mutação
/// que apaga o congelamento **SOBREVIVIA** a ela (medido 2026-09-19). A `40°`
/// as três respostas separam-se e a mutação sangra.
///
/// ⚠️ **A régua é o incremento contra a NORMAL DO CLIQUE**, e não contra o
/// primeiro incremento: na esfera unitária o pen-down é o próprio ponto, logo
/// o alvo é conhecido em forma fechada e o primeiro dab deixa de ser uma
/// referência que o ruído do arranque pode envenenar.
#[test]
fn a_direccao_congela_no_pen_down_e_o_espigao_sai_a_direito() {
    const GRAUS: f32 = 40.0;
    let p0 = [GRAUS.to_radians().sin(), 0.0, GRAUS.to_radians().cos()];
    let mut m = esfera();
    let b = Brush {
        verb: Verb::SnakeHook,
        radius: 0.30,
        strength: 1.0,
        puxa_pela_normal: true,
        ..Brush::default()
    };
    let mut st = SculptStroke::default();
    st.begin(&m);
    let mut antes = m.positions().to_vec();
    let mut pior = 0.0f32;
    let mut mediu = 0usize;
    for k in 0..8 {
        // O centro anda no plano de profundidade do pen-down, como o `hook_step`.
        let c = [p0[0] + 0.04 * k as f32, 0.0, p0[2]];
        st.dab(
            &mut m,
            &b,
            &Dab::hooking(c, b.radius, OLHO, [0.04, 0.0, 0.0]),
            Symmetry::default(),
        );
        let d = maior(&m, &antes);
        let l = norma(d);
        if l > 1e-4 {
            let cos = (d[0] * p0[0] + d[1] * p0[1] + d[2] * p0[2]) / l;
            pior = pior.max(cos.clamp(-1.0, 1.0).acos().to_degrees());
            mediu += 1;
        }
        antes = m.positions().to_vec();
    }
    assert_eq!(
        mediu,
        8,
        "o traco nao moveu barro em {} dos 8 dabs",
        8 - mediu
    );
    assert!(
        pior < 1.0,
        "o incremento afastou-se {pior:.2}° da normal do CLIQUE: a direccao nao \
         esta' congelada e o espigao enrola"
    );
}

/// ⭐⭐⭐⭐ **A OPÇÃO É OFERECIDA EXACTAMENTE A QUEM A SENTE — e a régua é o
/// BARRO, verbo a verbo.**
///
/// ⛔⛔ **A 1.ª redacção desta porta parava no GRIP e oferecia a caixa a SEIS
/// verbos.** Medido, quatro deles não a sentem: a `Pose` e o `Boundary`
/// resolvem a própria região e nunca leem o `dab.pull`; o `Thumb` e o `Nudge`
/// **subtraem** a normal do puxão, logo pô-lo ao longo dela deixa zero — e no
/// empurrão a opção *desligava* o pincel (`0,027 → 0,0006`).
///
/// ⚠️ **É o censo dos knobs mortos aplicado a UM controlo, antes de ele
/// shipar** — e quem o apanhou primeiro foi a catraca da dobra do painel, que
/// mede o preço de uma fileira nova em `+28 px` para **todos**.
#[test]
fn a_opcao_e_oferecida_exactamente_a_quem_a_sente() {
    let mut oferecem = Vec::new();
    let mut sentem = Vec::new();
    for verbo in Verb::ALL {
        let b = Brush {
            verb: verbo,
            ..Brush::default()
        };
        if b.oferece_puxar_pela_normal() {
            oferecem.push(verbo);
        }
        // O MESMO gesto com e sem a opção, pela porta do produto.
        let mut dz = [0.0f32; 2];
        for (k, liga) in [false, true].into_iter().enumerate() {
            let br = Brush {
                verb: verbo,
                radius: 0.35,
                strength: 1.0,
                puxa_pela_normal: liga,
                ..Brush::default()
            };
            let mut m = esfera();
            let antes = m.positions().to_vec();
            let mut st = SculptStroke::default();
            st.begin(&m);
            for j in 0..3 {
                let c = [0.05 * j as f32, 0.0, 1.0];
                st.dab(
                    &mut m,
                    &br,
                    &Dab::hooking(c, br.radius, OLHO, [0.05, 0.0, 0.0]),
                    Symmetry::default(),
                );
                st.dab(
                    &mut m,
                    &br,
                    &Dab::pulling(POLO, br.radius, OLHO, [0.05 * (j + 1) as f32, 0.0, 0.0]),
                    Symmetry::default(),
                );
            }
            dz[k] = maior(&m, &antes)[2];
        }
        // ⚠️ A barra é `1e-3` e não um epsilon: o polegar move `6e-5` com a
        // opção, que é ruído de projecção e não um efeito.
        if (dz[0] - dz[1]).abs() > 1e-3 {
            sentem.push(verbo);
        }
    }
    assert_eq!(
        oferecem, sentem,
        "a caixa e' oferecida a {oferecem:?} e o barro so' muda em {sentem:?}"
    );
    assert!(
        oferecem.contains(&Verb::Move) && oferecem.contains(&Verb::SnakeHook),
        "os dois verbos que o dono nomeou tem de a oferecer: {oferecem:?}"
    );
}

/// ⭐⭐⭐⭐ **QUEM É QUE A OPÇÃO DE FACTO MOVE** — a régua é o BARRO, não uma
/// tabela de leis.
#[test]
#[ignore = "sonda: imprime a populacao medida, nao afirma nada"]
fn diag_quem_sente_a_opcao() {
    println!("\n== QUEM SENTE O `puxa_pela_normal` (o barro, nao a tabela) ==\n");
    for verbo in Verb::ALL {
        let mut a = esfera();
        let mut b = esfera();
        let mut resultado = [0.0f32; 2];
        for (k, liga) in [false, true].into_iter().enumerate() {
            let br = Brush {
                verb: verbo,
                radius: 0.35,
                strength: 1.0,
                puxa_pela_normal: liga,
                ..Brush::default()
            };
            let m = if k == 0 { &mut a } else { &mut b };
            let antes = m.positions().to_vec();
            let mut st = SculptStroke::default();
            st.begin(m);
            for j in 0..3 {
                let c = [0.05 * j as f32, 0.0, 1.0];
                st.dab(
                    m,
                    &br,
                    &Dab::hooking(c, br.radius, OLHO, [0.05, 0.0, 0.0]),
                    Symmetry::default(),
                );
                st.dab(
                    m,
                    &br,
                    &Dab::pulling(POLO, br.radius, OLHO, [0.05 * (j + 1) as f32, 0.0, 0.0]),
                    Symmetry::default(),
                );
            }
            resultado[k] = maior(m, &antes)[2];
        }
        let dif = (resultado[0] - resultado[1]).abs();
        let oferece = Brush {
            verb: verbo,
            ..Brush::default()
        }
        .oferece_puxar_pela_normal();
        if dif > 1e-6 || oferece {
            println!(
                "  {:>16} | oferece {:>5} | dz off {:>9.5} · on {:>9.5} · dif {:>9.5}{}",
                format!("{verbo:?}"),
                oferece,
                resultado[0],
                resultado[1],
                dif,
                if dif <= 1e-6 && oferece {
                    "  ⛔ MORTO"
                } else {
                    ""
                }
            );
        }
    }
}

/// ⛔⛔ **A SONDA DO REPORT DE 2026-09-19** — *«no decorrer da puxada a normal
/// muda e não se mantém firme na primeira direcção escolhida no clique»*.
///
/// O gate do congelamento mede o INCREMENTO do gancho; esta sonda mede o que o
/// dono vê — o **eixo do espigão**, dab a dab, nos dois verbos e nos dois modos
/// de referência (o `L` liga o campo elástico, cujo `r` cresce com o espigão).
#[test]
#[ignore = "sonda: imprime a direccao ao longo do traco, nao afirma nada"]
fn diag_a_direccao_ao_longo_do_traco() {
    println!("\n== A DIRECCAO AO LONGO DO TRACO (o eixo do espigao, dab a dab) ==\n");
    for modo in [crate::RefMode::S] {
        for verbo in [Verb::SnakeHook, Verb::Move] {
            let b = Brush {
                verb: verbo,
                radius: 0.35,
                strength: 1.0,
                puxa_pela_normal: true,
                mode: modo,
                ..Brush::default()
            };
            if !b.oferece_puxar_pela_normal() {
                continue;
            }
            // ⚠️ **A fixtura do POLO não distingue nada:** ali a normal, o
            // plano e o olho são todos `+z`. O gesto do dono arranca OBLÍQUO.
            for graus in [0.0f32, 40.0] {
                let rad = graus.to_radians();
                let p0 = [rad.sin(), 0.0, rad.cos()];
                let mut m = esfera();
                let repouso = m.positions().to_vec();
                let mut st = SculptStroke::default();
                st.begin(&m);
                println!(
                    "  -- {verbo:?} · modo {modo:?} · campo {:?} · pen-down a {graus:.0}° \
                 (normal {:+.3},{:+.3},{:+.3})",
                    modo.field(verbo),
                    p0[0],
                    p0[1],
                    p0[2]
                );
                let mut eixo0 = [0.0f32; 3];
                let passo = 0.05f32;
                for k in 0..8 {
                    let antes = m.positions().to_vec();
                    let dab = if verbo == Verb::SnakeHook {
                        // Como o `hook_step`: o centro anda no plano de
                        // profundidade do pen-down (`z` constante) e o dab recebe o
                        // INCREMENTO.
                        Dab::hooking(
                            [p0[0] + passo * (k + 1) as f32, 0.0, p0[2]],
                            b.radius,
                            OLHO,
                            [passo, 0.0, 0.0],
                        )
                    } else {
                        // Como o `grab_at`: o centro é a ÂNCORA e o puxão é o TOTAL.
                        Dab::pulling(p0, b.radius, OLHO, [passo * (k + 1) as f32, 0.0, 0.0])
                    };
                    st.dab(&mut m, &b, &dab, Symmetry::default());
                    let inc = maior(&m, &antes);
                    let eixo = maior(&m, &repouso);
                    if k == 0 {
                        eixo0 = eixo;
                    }
                    let ang = |u: [f32; 3], v: [f32; 3]| {
                        let (a, c) = (norma(u), norma(v));
                        if a < 1e-9 || c < 1e-9 {
                            return f32::NAN;
                        }
                        (((u[0] * v[0] + u[1] * v[1] + u[2] * v[2]) / (a * c)).clamp(-1.0, 1.0))
                            .acos()
                            .to_degrees()
                    };
                    println!(
                        "     dab {k}: inc ({:+.4},{:+.4},{:+.4}) |{:.4}| · eixo \
                     ({:+.4},{:+.4},{:+.4}) |{:.4}| · eixo vs dab0 {:>6.2}°",
                        inc[0],
                        inc[1],
                        inc[2],
                        norma(inc),
                        eixo[0],
                        eixo[1],
                        eixo[2],
                        norma(eixo),
                        ang(eixo, eixo0),
                    );
                }
            }
        }
    }
}
