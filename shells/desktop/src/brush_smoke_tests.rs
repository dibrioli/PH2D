//! Os gates da cena do PINCEL (`PH2D_BUILD_SMOKE=77`, plano 36 W3-bis).
//!
//! ⚠️ **Uma cena de smoke é uma FIXTURA, e uma fixtura tem de conter o fenómeno.** Os gates aqui
//! não medem o motor (isso é a `ph2d-vec-scene`): medem que **esta cena** deixa a lei visível.

use super::{BOX, TRACEJADO, arte, onda, pincel};
use ph2d_vec_scene::{VecPath, VecPathId};

/// O id que a arte tem nesta fixtura (na cena real ela é a 1ª forma).
///
/// ⚠️ **Não é `0`** de propósito: `VecPathId::default()` é um id VÁLIDO, e foi ele que fez o
/// `art` do `BrushStroke` virar um `Option` na W4 — *"sem arte"* e *"a arte é a primeira forma"*
/// eram os mesmos bytes.
const ART: VecPathId = 7;

fn elipse() -> VecPath {
    ph2d_vec_scene::ellipse([0.0, 0.0], BOX * 0.5, BOX * 0.5)
}

/// ⭐⭐⭐ **O TRACEJADO DESTA CENA LEVA MAIS DE UMA CÓPIA POR TRAÇO** — sem isso a cena não
/// distingue a lei que existe para mostrar.
///
/// ⚠️ **Com uma cópia só por traço, *"a arte reinicia em cada traço"* e *"há uma bolha em cada
/// traço"* desenham exactamente a mesma coisa.** O Enio olharia para um pontilhado e concluiria que
/// o pincel não funciona com tracejado — que é o report que esta wave existe para fechar.
#[test]
fn the_smoke_dash_carries_more_than_one_copy_per_dash() {
    let forma = elipse();
    let s = pincel(ART, Some(TRACEJADO), 0.0, false);
    let dash = ph2d_vec_scene::dash_for(&forma, &s).expect("a 2a forma da cena tem tracejado");
    let total = ph2d_vec_scene::dash_fit::longest_contour(&forma)
        .expect("a elipse tem comprimento")
        .0;
    let tracos = ph2d_vec_scene::brush_spans(total, Some(dash)).len();
    let copias = ph2d_vec_scene::brush_along_path(&forma, &arte(0.0, 0.0), &s).len();
    assert!(
        tracos >= 3,
        "a cena so' mostra {tracos} tracos - poucos para se ler a cadencia"
    );
    assert!(
        copias >= tracos * 2,
        "a cena poe {copias} copias em {tracos} tracos (menos de 2 por traco) - um pontilhado e um \
         pincel tracejado desenham igual, e o smoke deixa de provar a lei"
    );
    // ⚠️ **CONTROLO — sem tracejado a MESMA forma leva MAIS cópias.** Sem esta metade o gate
    // ficaria verde num dia em que o tracejado deixasse de tirar coisa nenhuma.
    let cheio =
        ph2d_vec_scene::brush_along_path(&forma, &arte(0.0, 0.0), &pincel(ART, None, 0.0, false))
            .len();
    assert!(
        cheio > copias,
        "o tracejado da cena nao tirou copias nenhumas ({cheio} contra {copias})"
    );
}

/// ⭐ **A ARTE É ASSIMÉTRICA NOS DOIS EIXOS** — é isso que deixa ver a *Rotation* (assimetria em
/// `x`) e o *Flip* (assimetria em `y`).
///
/// ⛔ Um losango simétrico esconderia as duas, e a cena teria dois botões cujo efeito não se lê.
///
/// ⚠️ **A 1.ª régua deste gate era um PROXY e reprovou produto correto:** ela comparava o centro do
/// bbox com a média dos vértices, e uma forma visivelmente torta deu `0,0125` — a média de quatro
/// pontos não sabe nada sobre a forma entre eles. A régua a sério **ESPELHA a arte e mede o quanto
/// ela deixou de coincidir consigo mesma**, normalizado pela diagonal do bbox. MEDIDO nesta arte:
/// `0,19` em `x` e `0,37` em `y`; a barra é `0,10`.
#[test]
fn the_smoke_art_is_asymmetric_on_both_axes() {
    let a = arte(0.0, 0.0);
    let p: Vec<[f64; 2]> = a.verts.iter().map(|v| v.anchor).collect();
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for q in &p {
        for k in 0..2 {
            lo[k] = lo[k].min(q[k]);
            hi[k] = hi[k].max(q[k]);
        }
    }
    let diag = (hi[0] - lo[0]).hypot(hi[1] - lo[1]);
    let assimetria = |eixo: usize| {
        let meio = (lo[eixo] + hi[eixo]) * 0.5;
        p.iter()
            .map(|q| {
                let mut espelhado = *q;
                espelhado[eixo] = 2.0 * meio - q[eixo];
                // A distância do ponto espelhado ao ORIGINAL mais próximo: zero se a forma é
                // simétrica naquele eixo.
                p.iter()
                    .map(|o| (o[0] - espelhado[0]).hypot(o[1] - espelhado[1]))
                    .fold(f64::MAX, f64::min)
            })
            .fold(0.0, f64::max)
            / diag
    };
    for (k, nome) in [(0usize, "x (a Rotation)"), (1, "y (o Flip)")] {
        let v = assimetria(k);
        assert!(
            v > 0.10,
            "a arte da cena e' quase simetrica em {nome}: o botao correspondente nao tem efeito \
             visivel (assimetria {v:.4})"
        );
    }
    // ⚠️ **CONTROLO — a régua devolve ZERO numa forma simétrica.** Sem esta metade ela poderia
    // estar a medir qualquer coisa e a passar por acidente.
    let quadrado: Vec<[f64; 2]> = vec![[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]];
    let sim = |eixo: usize| {
        quadrado
            .iter()
            .map(|q| {
                let mut e = *q;
                e[eixo] = -q[eixo];
                quadrado
                    .iter()
                    .map(|o| (o[0] - e[0]).hypot(o[1] - e[1]))
                    .fold(f64::MAX, f64::min)
            })
            .fold(0.0, f64::max)
    };
    assert!(
        sim(0) < 1e-12 && sim(1) < 1e-12,
        "a regua acusa assimetria num QUADRADO - ela nao mede simetria"
    );
}

/// ⚠️ **A ONDA é ABERTA e tem curvatura** — ela existe para mostrar as duas PONTAS, e uma reta
/// não distingue *"as cópias giram com a curva"* de *"as cópias estão todas no mesmo ângulo"*.
#[test]
fn the_smoke_wave_is_open_and_actually_curves() {
    let v = onda(0.0, 0.0, BOX * 0.5);
    assert!(v.len() >= 5, "a onda ficou com {} nos", v.len());
    let ys: Vec<f64> = v.iter().map(|p| p.anchor[1]).collect();
    let lo = ys.iter().copied().fold(f64::MAX, f64::min);
    let hi = ys.iter().copied().fold(f64::MIN, f64::max);
    assert!(
        hi - lo > BOX * 0.4,
        "a onda e' quase uma reta (excursao {:.3}) - as copias saem todas no mesmo angulo e a \
         forma nao prova nada",
        hi - lo
    );
}

/// ⛔⛔ **NENHUM PINCEL DESTA CENA APONTA PARA A PRÓPRIA FORMA** — o ciclo que pararia o app.
///
/// ⚠️ A recusa vive no produto (`brush_live::art_of`), e é PURA. Este gate é sobre a **fixtura**:
/// uma cena que a exercitasse estaria a testar o guarda em vez de mostrar a feature.
#[test]
fn no_brush_in_the_smoke_points_at_its_own_shape() {
    // A arte da cena é sempre a forma que nasce primeiro; as hospedeiras nascem depois e têm ids
    // diferentes por construção. A régua é a que o produto usa.
    let s = pincel(ART, None, 0.0, false);
    let b = s.brush().expect("a cena poe pinceis");
    assert_eq!(b.art, Some(ART), "o pincel da cena perdeu a arte");
    assert!(
        b.art != Some(VecPathId::default()),
        "a arte da cena e' o id default - 'sem arte' e 'a primeira forma' voltariam a ser os \
         mesmos bytes"
    );
}
