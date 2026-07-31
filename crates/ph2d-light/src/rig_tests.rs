//! Gates do rig.
//!
//! O que esta crate pode errar é **um número**, e o número é a direção de uma lâmpada. Os gates abaixo
//! são, nessa ordem: o rotor é o do app · o rig vazio desiste · e os defaults que o artista herda.

use super::*;

/// Direção pela via TRIGONOMÉTRICA — o que um segundo implementador escreveria.
///
/// Existe **só** como oráculo de contraste: é o número que NÃO queremos.
fn dir_by_trig(angle_deg: u16, elev_deg: u16) -> [f32; 3] {
    let az = f32::from(angle_deg % 360).to_radians();
    let el = f32::from(elev_deg.clamp(MIN_ELEV_DEG, 90)).to_radians();
    [el.cos() * az.cos(), el.cos() * az.sin(), el.sin()]
}

/// **O rotor do app não é intercambiável com uma chamada de trigonometria, e este gate mede isso.**
///
/// A justificativa da crate inteira — e da dependência em `ph2d-painter-brush` — é que
/// `rotate_by_degrees` acumula 1° por vez, então dá um número *específico*. Se ela concordasse ao bit
/// com `sin`/`cos`, essa justificativa seria FALSA e a dependência seria gratuita. Então o gate afirma
/// as duas metades: as duas vias descrevem a mesma geometria (perto), e **não são o mesmo número**.
///
/// **Medido: 312 de 312 direções diferem nos bits**, com desvio geométrico máximo de `4,888e-6` — ou
/// seja, uma segunda implementação erraria em *toda* lâmpada, não numa borda exótica.
///
/// ⚠️ A barra é MAIORIA, não igualdade, e de propósito: o oráculo usa `sin`/`cos` da `std`, que **não
/// é pinada cross-OS**. A afirmação que importa (*não dá para substituir o rotor por trigonometria*)
/// não é enfraquecida por um ângulo que por acaso coincida noutra plataforma; a mutação real — trocar
/// o rotor por `sin`/`cos` — leva `differing` a zero e morre aqui de qualquer jeito.
#[test]
fn the_rotor_is_the_apps_and_a_trig_call_is_not_the_same_number() {
    let mut worst = 0.0f32;
    let mut differing = 0usize;
    let mut total = 0usize;
    for angle in (0..360).step_by(7) {
        for elev in [5u16, 17, 30, 45, 60, 90] {
            let rig = LightRig {
                lights: [
                    Light {
                        on: true,
                        angle_deg: angle,
                        elev_deg: elev,
                        intensity: 1.0,
                        color: [1.0; 3],
                    },
                    Light::FILL,
                    Light::FILL,
                    Light::FILL,
                ],
                selected: 0,
            };
            let got = resolve(&rig).expect("uma lâmpada acesa").lamps()[0].dir;
            let want = dir_by_trig(angle, elev);
            total += 1;
            for c in 0..3 {
                worst = worst.max((got[c] - want[c]).abs());
            }
            if got != want {
                differing += 1;
            }
        }
    }
    // Metade 1: o rotor está CERTO — descreve a mesma direção que a trigonometria descreveria.
    assert!(
        worst < 1e-3,
        "o rotor divergiu da geometria em {worst} — isso não é ruído de último bit, é um bug"
    );
    // Metade 2: e mesmo assim NÃO é o mesmo número. É esta linha que torna a dependência necessária
    // em vez de opinativa. Se ela falhar, o doc-comment da crate está mentindo.
    assert!(
        differing * 2 > total,
        "só {differing} de {total} direções diferem de sin/cos — a premissa da crate é falsa"
    );
}

/// **A IMPRESSÃO DIGITAL — a lâmpada principal, pinada no float.**
///
/// Este arquivo nasceu de uma mudança de casa: a resolução vinha de `ph2d-tool-painter` e passou a
/// morar aqui. *"A suíte do Painter passa"* prova que nada quebrou; **não** prova que nada MUDOU — e
/// mudar um número aqui move todo pixel de todo relevo já pintado, no lugar exato onde ninguém lê um
/// número de volta.
///
/// Então o número está escrito. Ele é o rig DEFAULT (a principal a 230°/30°), que é o rig de toda tela
/// que ninguém abriu o card para mexer.
///
/// ⚠️ Literais exatos são legítimos aqui e não seriam num shader: o rotor é `+`/`−`/`×` em `f32`, que o
/// IEEE-754 especifica exatamente e o Rust não contrai em FMA. É a mesma política dos literais do
/// `impasto_light_shader_constants_match_the_cpu_pass`.
#[test]
fn the_key_lamp_is_pinned_to_the_float() {
    let r = resolve(&LightRig::default()).expect("a principal nasce acesa");
    assert_eq!(r.lamps().len(), 1, "só a principal nasce acesa");
    let l = r.lamps()[0];
    assert_eq!(l.dir, [-0.5566727, -0.6634163, 0.50000024]);
    assert_eq!(l.half, [-0.32139477, -0.38302317, 0.8660246]);
    assert_eq!(l.tint, [1.0, 1.0, 1.0]);
    // ⚠️ `dir[2]` é `0.50000024` e não `0.5`: a 30° o seno EXATO é meio, e a diferença é o erro que o
    // rotor acumula em trinta passos de um grau. Não é ruído a limpar — é a assinatura de que o número
    // saiu do rotor do app, e não de uma chamada de `sin`.
    assert_ne!(
        l.dir[2], 0.5,
        "um `sin(30°)` daria meio exato; o rotor não dá"
    );
}

/// A resposta plana é o DIVISOR do modelo relativo, e a 0° de elevação ela vai a zero. O clamp existe
/// para essa divisão nunca acontecer — e ele é o mesmo em qualquer entrada absurda.
#[test]
fn an_impossible_elevation_is_clamped_instead_of_dividing_by_zero() {
    for elev in [0u16, 1, 3, 4] {
        let mut rig = LightRig::default();
        rig.lights[0].elev_deg = elev;
        let z = resolve(&rig).expect("acesa").lamps()[0].dir[2];
        let floor = resolve(&{
            let mut r = LightRig::default();
            r.lights[0].elev_deg = MIN_ELEV_DEG;
            r
        })
        .expect("acesa")
        .lamps()[0]
            .dir[2];
        assert_eq!(z, floor, "elevação {elev} tinha de bater no piso");
        assert!(
            z > 0.0,
            "a resposta plana de uma lâmpada rasante seria zero"
        );
    }
    // E o teto também: 90° é zênite, e não há mais para onde subir.
    let mut rig = LightRig::default();
    rig.lights[0].elev_deg = 400;
    let d = resolve(&rig).expect("acesa").lamps()[0].dir;
    assert!(
        d[2] > 0.999,
        "acima de 90° a lâmpada tem de ficar no zênite"
    );
}

/// **Baixar as luzes até o fim é uma tela SEM LUZ, não uma tela escura.**
///
/// O filtro `intensity > 0` é o que faz a lista sair vazia — e com a lista não-vazia o consumidor
/// dividiria uma difusa zero por um piso 1 e empurraria todo pixel para o ambiente, escurecendo a
/// pintura a 35% em vez de deixá-la em paz. Duas rotas para o mesmo `None`, porque são dois gestos
/// diferentes do artista.
#[test]
fn a_rig_with_nothing_lit_is_no_rig_at_all() {
    let mut all_off = LightRig::default();
    for l in &mut all_off.lights {
        l.on = false;
    }
    assert!(resolve(&all_off).is_none(), "toda lâmpada desligada");

    let mut all_dark = LightRig::default();
    for l in &mut all_dark.lights {
        l.on = true;
        l.intensity = 0.0;
    }
    assert!(
        resolve(&all_dark).is_none(),
        "acesas mas em potência zero — o caso que o filtro existe para pegar"
    );
    assert!(!all_dark.any_on(), "e `any_on` concorda com `resolve`");
}

/// Uma lâmpada apagada nunca atravessa, e a ordem das acesas é a do rig.
#[test]
fn only_the_lit_lamps_cross_and_they_keep_their_order() {
    let mut rig = LightRig::default();
    rig.lights[1].on = false;
    rig.lights[2] = Light {
        on: true,
        angle_deg: 90,
        elev_deg: 60,
        intensity: 0.5,
        color: [1.0; 3],
    };
    let r = resolve(&rig).expect("duas acesas");
    assert_eq!(r.lamps().len(), 2, "a apagada do meio não pode entrar");
    // A segunda resolvida é a lâmpada 2 do rig — não a 1.
    assert_eq!(
        r.lamps()[1].dir,
        resolve(&{
            let mut solo = LightRig::default();
            solo.lights[0] = rig.lights[2];
            solo
        })
        .expect("acesa")
        .lamps()[0]
            .dir
    );
    for (i, l) in r.lamps().iter().enumerate() {
        let len = (l.dir[0] * l.dir[0] + l.dir[1] * l.dir[1] + l.dir[2] * l.dir[2]).sqrt();
        assert!(
            (len - 1.0).abs() < 1e-5,
            "lâmpada {i} não é unitária: {len}"
        );
        let hl = (l.half[0] * l.half[0] + l.half[1] * l.half[1] + l.half[2] * l.half[2]).sqrt();
        assert!(
            (hl - 1.0).abs() < 1e-5,
            "o half da {i} não é unitário: {hl}"
        );
    }
}

/// O caminho rápido existe para o rig que todo mundo tem, e some no instante em que alguém colore uma
/// lâmpada de propósito. As duas metades, porque um predicado que nunca é falso não é um predicado.
#[test]
fn a_grey_rig_takes_the_fast_lane_and_a_coloured_one_does_not() {
    assert!(
        resolve(&LightRig::default()).expect("acesa").achromatic(),
        "o rig DEFAULT é o rig do caminho rápido — se não for, ele não serve para nada"
    );
    let mut warm = LightRig::default();
    warm.lights[0].color = [1.0, 0.9, 0.8];
    assert!(!resolve(&warm).expect("acesa").achromatic());
    // ⚠️ E a INTENSIDADE não quebra o caminho: ela pesa os três canais igualmente.
    let mut dim = LightRig::default();
    dim.lights[0].intensity = 0.37;
    assert!(resolve(&dim).expect("acesa").achromatic());
}

/// Os defaults que o artista herda, e a regra que o segundo deles carrega.
#[test]
fn the_second_lamp_is_not_the_first_one_in_the_same_place() {
    let rig = LightRig::default();
    assert!(rig.lights[0].on, "a principal nasce acesa");
    assert!(
        rig.lights[1..].iter().all(|l| !l.on),
        "as outras nascem apagadas — é isso que mantém uma tela nova byte-idêntica"
    );
    // Quem marca "Enable" na lâmpada 2 e não vê nada mudar chama o interruptor de quebrado.
    assert_ne!(
        rig.lights[1].angle_deg, rig.lights[0].angle_deg,
        "duas lâmpadas no mesmo lugar são uma lâmpada"
    );
    assert_eq!(rig.current(), &Light::KEY);
    // Seleção fora da faixa CLAMPA — um snapshot velho não derruba o painel.
    let stale = LightRig {
        selected: 200,
        ..LightRig::default()
    };
    assert_eq!(stale.current(), &Light::FILL);
}
