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

/// ⭐⭐⭐ **O NEUTRO É BYTE-IDÊNTICO** — desligada, a opção não existe para o
/// motor.
///
/// ⚠️ Sem este gate, a lei poderia estar a passar por um caminho novo que
/// **recalcula** o mesmo número por outra ordem, e os oráculos dos dois verbos
/// mediriam outro programa ao último bit.
#[test]
fn desligada_ela_nao_muda_um_bit() {
    for verbo in [Verb::Move, Verb::SnakeHook] {
        let mut a = esfera();
        let mut b = esfera();
        let mut ba = Brush {
            verb: verbo,
            radius: 0.35,
            strength: 1.0,
            puxa_pela_normal: false,
            ..Brush::default()
        };
        let mut sa = SculptStroke::default();
        let mut sb = SculptStroke::default();
        sa.begin(&a);
        sb.begin(&b);
        for k in 0..6 {
            let c = [0.05 * k as f32, 0.0, 1.0];
            let d = match verbo {
                Verb::SnakeHook => Dab::hooking(c, ba.radius, OLHO, [0.05, 0.0, 0.0]),
                _ => Dab::pulling(POLO, ba.radius, OLHO, [0.05 * (k + 1) as f32, 0.0, 0.0]),
            };
            sa.dab(&mut a, &ba, &d, Symmetry::default());
            // O mesmo, com o campo do pincel a declarar a opção DESLIGADA de
            // outra maneira (o `Default` já a põe a `false`).
            ba.puxa_pela_normal = false;
            sb.dab(&mut b, &ba, &d, Symmetry::default());
        }
        assert_eq!(
            a.positions(),
            b.positions(),
            "{verbo:?}: o caminho de omissao deixou de ser byte-identico"
        );
    }
}

/// ⭐⭐⭐⭐ **A DIRECÇÃO CONGELA NO PEN-DOWN** — e sem isto o espigão ENROLA.
///
/// Num gancho, cada dab vira a superfície debaixo do cursor; lida viva, a normal
/// do dab `k+1` é a que o dab `k` acabou de criar. O gate mede o ÂNGULO entre o
/// primeiro e o último incremento do traço: congelada, ele é **zero**.
#[test]
fn a_direccao_congela_no_pen_down_e_o_espigao_sai_a_direito() {
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
    let mut primeiro = [0.0f32; 3];
    let mut ultimo = [0.0f32; 3];
    for k in 0..8 {
        let c = [0.04 * k as f32, 0.0, 1.0];
        st.dab(
            &mut m,
            &b,
            &Dab::hooking(c, b.radius, OLHO, [0.04, 0.0, 0.0]),
            Symmetry::default(),
        );
        let d = maior(&m, &antes);
        if k == 0 {
            primeiro = d;
        }
        ultimo = d;
        antes = m.positions().to_vec();
    }
    let (l0, l1) = (norma(primeiro), norma(ultimo));
    assert!(
        l0 > 1e-4 && l1 > 1e-4,
        "o traco nao moveu barro ({l0:.6}, {l1:.6})"
    );
    let cos =
        (primeiro[0] * ultimo[0] + primeiro[1] * ultimo[1] + primeiro[2] * ultimo[2]) / (l0 * l1);
    let graus = cos.clamp(-1.0, 1.0).acos().to_degrees();
    assert!(
        graus < 1.0,
        "o incremento virou {graus:.2}° entre o primeiro dab e o oitavo: a \
         direccao nao esta' congelada e o espigao enrola"
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
