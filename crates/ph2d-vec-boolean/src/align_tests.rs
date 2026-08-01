//! Os gates do alinhamento de traço.
//!
//! ⚠️ **O oráculo é ONDE A TINTA CAI, nunca a fórmula.** Todo teste aqui mede ÁREA — quanto da
//! faixa está dentro da forma, quanto está fora, e se a silhueta se moveu. Um gate que
//! reconstruísse `banda ∩ interior` para comparar com o resultado seria um espelho do produto:
//! passaria com a operação trocada, com a banda na largura errada, e com o recorte contra a forma
//! errada. As duas propriedades que o artista de facto pediu — *a tinta fica de um lado só* e *a
//! silhueta não se mexe* — não conhecem uma linha da implementação.

use super::*;
use ph2d_vec_scene::{Contour, FillRule, LineCap, LineJoin, Rgba8, VecVertex};

fn v(x: f64, y: f64) -> VecVertex {
    VecVertex::corner([x, y])
}

fn ink() -> Rgba8 {
    Rgba8::new(10, 20, 30, 255)
}

fn spec(width: f64, align: StrokeAlign) -> StrokeSpec {
    StrokeSpec {
        cap: LineCap::Butt,
        join: LineJoin::Miter,
        align,
        ..StrokeSpec::new(ink(), width)
    }
}

/// Um quadrado de lado 4 centrado na origem — forma cujo interior não é ambíguo.
fn square(width: f64, align: StrokeAlign) -> VecPath {
    VecPath {
        verts: vec![v(-2.0, -2.0), v(2.0, -2.0), v(2.0, 2.0), v(-2.0, 2.0)],
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(200, 200, 200, 255))),
        stroke: Some(spec(width, align)),
        ..VecPath::default()
    }
}

/// Uma rosquinha (quadrado com furo, EvenOdd) — o compound, onde *dentro* inclui as bordas do
/// FURO. É a fixture que separa "recortei contra o contorno de fora" de "recortei contra a
/// REGIÃO", e as duas coincidem em toda forma simples.
fn donut(width: f64, align: StrokeAlign) -> VecPath {
    VecPath {
        verts: vec![v(-2.0, -2.0), v(2.0, -2.0), v(2.0, 2.0), v(-2.0, 2.0)],
        closed: true,
        subpaths: vec![Contour::new_closed(vec![
            v(-0.8, -0.8),
            v(0.8, -0.8),
            v(0.8, 0.8),
            v(-0.8, 0.8),
        ])],
        fill_rule: FillRule::EvenOdd,
        fill: Some(Paint::Solid(Rgba8::new(200, 200, 200, 255))),
        stroke: Some(spec(width, align)),
        ..VecPath::default()
    }
}

/// Um CÍRCULO por quatro cúbicas — a fixture de borda CURVA. Todo o resto aqui é poligonal, e
/// uma pergunta sobre resíduo de sweep não se responde sobre arestas retas.
fn circle(r: f64, width: f64, align: StrokeAlign) -> VecPath {
    const K: f64 = 0.552_284_749_830_793_4;
    let k = K * r;
    let pts = [(r, 0.0), (0.0, r), (-r, 0.0), (0.0, -r)];
    let tans = [(0.0, k), (-k, 0.0), (0.0, -k), (k, 0.0)];
    let verts = pts
        .iter()
        .zip(tans.iter())
        .map(|(&(x, y), &(tx, ty))| VecVertex {
            in_handle: [x - tx, y - ty],
            out_handle: [x + tx, y + ty],
            ..VecVertex::corner([x, y])
        })
        .collect();
    VecPath {
        verts,
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(200, 200, 200, 255))),
        stroke: Some(spec(width, align)),
        ..VecPath::default()
    }
}

/// Um "C" — contorno ABERTO, o caso sem interior.
fn open_arc(width: f64, align: StrokeAlign) -> VecPath {
    VecPath {
        verts: vec![v(-2.0, -2.0), v(2.0, -2.0), v(2.0, 2.0)],
        closed: false,
        stroke: Some(spec(width, align)),
        ..VecPath::default()
    }
}

/// A forma sem traço nenhum — o operando de recorte que os oráculos usam.
fn bare(path: &VecPath) -> VecPath {
    VecPath {
        stroke: None,
        ..path.clone()
    }
}

/// Quanto de `paths` cai DENTRO da região de `region`.
fn area_inside(paths: &[VecPath], region: &VecPath) -> f64 {
    paths
        .iter()
        .flat_map(|p| apply(p, region, BoolOp::Intersect))
        .map(|p| crate::area(&p))
        .sum()
}

/// Quanto de `paths` cai FORA da região de `region`.
fn area_outside(paths: &[VecPath], region: &VecPath) -> f64 {
    paths
        .iter()
        .flat_map(|p| apply(p, region, BoolOp::Subtract))
        .map(|p| crate::area(&p))
        .sum()
}

fn total(paths: &[VecPath]) -> f64 {
    paths.iter().map(crate::area).sum()
}

/// **Inner deita TODA a tinta para dentro.** A propriedade que o artista pediu, medida como
/// fração — nada aqui sabe como o recorte foi feito.
#[test]
fn an_inner_stroke_lays_all_its_ink_inside_the_shape() {
    let path = square(0.4, StrokeAlign::Inner);
    let band = aligned_stroke(&path).expect("um quadrado fechado com traco tem alinhamento");
    let all = total(&band);
    assert!(all > 0.0, "a faixa nao pode sair vazia");
    let out = area_outside(&band, &bare(&path));
    assert!(
        out / all < 1e-3,
        "Inner deixou {:.4}% da tinta FORA da forma (fora {out:.6} de {all:.6})",
        100.0 * out / all
    );
}

/// **Outer deita toda a tinta para fora.** O espelho — e não é redundante: uma implementação que
/// devolvesse a banda INTEIRA passa no gate de Inner só se `Intersect` estiver certo, e passa
/// neste só se `Subtract` estiver.
#[test]
fn an_outer_stroke_lays_all_its_ink_outside_the_shape() {
    let path = square(0.4, StrokeAlign::Outer);
    let band = aligned_stroke(&path).expect("um quadrado fechado com traco tem alinhamento");
    let all = total(&band);
    assert!(all > 0.0, "a faixa nao pode sair vazia");
    let inside = area_inside(&band, &bare(&path));
    assert!(
        inside / all < 1e-3,
        "Outer deixou {:.4}% da tinta DENTRO da forma (dentro {inside:.6} de {all:.6})",
        100.0 * inside / all
    );
}

/// **A faixa tem a espessura AUTORADA, não o dobro.** É a metade que os dois gates de lado não
/// veem: uma banda de largura `2w` que nunca foi recortada está *toda de um lado* de nada, mas
/// com a forma inteira dentro dela os dois testes acima ficariam verdes sobre tinta duas vezes
/// mais grossa. O oráculo é o perímetro × largura, exato para um quadrado de quinas retas.
#[test]
fn the_aligned_band_is_as_thick_as_the_artist_asked() {
    let w = 0.4;
    let inner = aligned_stroke(&square(w, StrokeAlign::Inner)).expect("inner existe");
    // Lado 4, faixa `w` para dentro: a área é `4·4 − (4−2w)²`.
    let want = 4.0 * 4.0 - (4.0 - 2.0 * w).powi(2);
    let got = total(&inner);
    assert!(
        (got - want).abs() / want < 0.01,
        "a faixa de dentro mede {got:.4}, esperado {want:.4} (largura autorada {w})"
    );
}

/// **A SILHUETA não se mexe** — a promessa inteira do Inner, dita como o olho a vê: engrossar um
/// contorno interno não pode fazer a forma crescer. Comparado contra a MESMA forma sem traço.
#[test]
fn thickening_an_inner_stroke_does_not_grow_the_silhouette() {
    let bare_area = crate::area(&bare(&square(0.0, StrokeAlign::Centre)));
    for w in [0.1, 0.4, 0.9] {
        let path = square(w, StrokeAlign::Inner);
        let band = aligned_stroke(&path).expect("inner existe");
        // A silhueta desenhada é `preenchimento ∪ faixa`.
        let union: f64 = {
            let mut acc = bare(&path);
            for b in &band {
                match apply(&acc, b, BoolOp::Union).into_iter().next() {
                    Some(u) => acc = u,
                    None => panic!("a uniao da silhueta falhou em w={w}"),
                }
            }
            crate::area(&acc)
        };
        assert!(
            (union - bare_area).abs() / bare_area < 1e-3,
            "com w={w} a silhueta mediu {union:.4} contra {bare_area:.4} da forma nua"
        );
    }
}

/// **O recorte é contra a REGIÃO, não contra o contorno de fora** — no compound a faixa de dentro
/// também abraça o FURO. Sem esta fixture, recortar contra o contorno externo passa em tudo.
#[test]
fn the_inner_band_of_a_compound_hugs_the_hole_too() {
    let path = donut(0.3, StrokeAlign::Inner);
    let band = aligned_stroke(&path).expect("a rosquinha e fechada");
    // Uma SONDA e' um quadradinho: a pergunta e' "a faixa cobre este lugar?".
    let probe = |x: f64, y: f64| VecPath {
        verts: vec![
            v(x - 0.02, y - 0.02),
            v(x + 0.02, y - 0.02),
            v(x + 0.02, y + 0.02),
            v(x - 0.02, y + 0.02),
        ],
        closed: true,
        ..VecPath::default()
    };
    // Logo FORA do furo (x = 0.8 + w/2 = 0.95) tem de haver tinta: e' a coroa que so' existe se
    // o contorno do furo participou do recorte.
    assert!(
        area_inside(&band, &probe(0.95, 0.0)) > 0.0,
        "faltou tinta na borda do furo -- o recorte olhou so o contorno de fora"
    );
    // ...e logo DENTRO do furo (x = 0.8 - w/2 = 0.65) nao pode haver tinta nenhuma.
    //
    // ⚠️ A sonda tem de ficar a MEIA LARGURA da borda, nao no meio do furo: um recorte que
    // ignorasse o furo deixaria a faixa dele INTEIRA (a coroa de dentro E a metade que invade o
    // vazio), e essa metade so' alcanca `w` para dentro. Sondar o centro (0,0) mede um lugar que
    // NENHUMA das duas leis pinta, e o gate fica verde sobre o defeito (medido).
    assert!(
        area_inside(&band, &probe(0.65, 0.0)) < 1e-9,
        "a faixa vazou para dentro do furo -- o recorte ignorou o contorno interno"
    );
}

/// **Um caminho ABERTO não tem alinhamento** — `None`, e o chamador pinta o traço como sempre.
/// Devolver uma faixa vazia aqui apagaria a linha.
#[test]
fn an_open_path_has_no_inside_so_it_refuses_alignment() {
    assert!(aligned_stroke(&open_arc(0.4, StrokeAlign::Inner)).is_none());
    assert!(aligned_stroke(&open_arc(0.4, StrokeAlign::Outer)).is_none());
}

/// **Centre, largura zero e sem traço são o mesmo `None`** — nenhum deles tem recorte a fazer, e
/// o caminho comum não paga uma booleana.
#[test]
fn the_centred_the_hairless_and_the_strokeless_all_decline() {
    assert!(aligned_stroke(&square(0.4, StrokeAlign::Centre)).is_none());
    assert!(aligned_stroke(&square(0.0, StrokeAlign::Inner)).is_none());
    assert!(aligned_stroke(&bare(&square(0.4, StrokeAlign::Inner))).is_none());
}

/// **A faixa sai com a cor do TRAÇO, preenchida** — e não com a do preenchimento da forma, que é
/// o que o `apply_many` entregaria se o operando de recorte não carregasse a tinta.
#[test]
fn the_band_wears_the_strokes_colour_as_a_fill() {
    let band = aligned_stroke(&square(0.4, StrokeAlign::Inner)).expect("inner existe");
    for p in &band {
        assert_eq!(
            p.fill,
            Some(Paint::Solid(ink())),
            "a faixa tem de ser preenchida com a cor do traco"
        );
        assert!(p.stroke.is_none(), "a faixa e tinta, nao um traco");
    }
}

/// **O tracejado mantém a CADÊNCIA.** O `dash` é múltiplo da largura, então a banda dupla o
/// dobraria em comprimento — e o Inner sairia com metade da espessura e o dobro do passo. O
/// oráculo é a CONTAGEM de peças: o mesmo caminho, o mesmo tracejado, o mesmo número de traços.
#[test]
fn an_aligned_dash_keeps_the_cadence_of_the_centred_one() {
    let dash = Some((2.0, 2.0));
    let mut centred = square(0.2, StrokeAlign::Centre);
    centred.stroke = Some(StrokeSpec {
        dash,
        ..spec(0.2, StrokeAlign::Centre)
    });
    let mut inner = centred.clone();
    inner.stroke = Some(StrokeSpec {
        align: StrokeAlign::Inner,
        ..centred.stroke.expect("tem traco")
    });

    let n_centred = outline_stroke(&centred).len();
    let n_inner = aligned_stroke(&inner).expect("inner existe").len();
    assert!(
        n_centred > 1,
        "a fixture tem de conter o fenomeno (tracejado)"
    );
    assert_eq!(
        n_inner, n_centred,
        "o tracejado alinhado saiu com {n_inner} pecas contra {n_centred} do centrado — \
         a cadencia mudou com a largura"
    );
}

/// **O recorte de uma borda CURVA sai LIMPO — uma peça, zero lascas.**
///
/// ⚠️ Este gate existe porque eu afirmei o contrário. O cabeçalho deste módulo dizia que o
/// alinhamento reproduz o BUGS #16 (a lasca do Shape Builder), e a medição negou: lá os dois
/// operandos **compartilham a fronteira**, aqui a linha da forma corta o **miolo** da banda. Sem
/// este gate a próxima pessoa reconstrói o filtro de lascas como se ele fosse load-bearing — ou,
/// pior, "conserta" um resíduo que não existe.
///
/// A fixture é uma CURVA de propósito: num polígono a pergunta nem chega a ser feita.
#[test]
fn clipping_a_curved_border_leaves_one_clean_piece() {
    for align in [StrokeAlign::Inner, StrokeAlign::Outer] {
        for w in [0.05, 0.2] {
            let band = aligned_stroke(&circle(2.0, w, align)).expect("o circulo e fechado");
            let all = total(&band);
            assert!(all > 0.0, "a faixa nao pode sair vazia em {align:?} w={w}");
            // Uma lasca é área ~nula com contorno LONGO: sem área não há preenchimento, e ela
            // pinta como uma linha solta. O oráculo é a fração da área total.
            let slivers = band.iter().filter(|p| crate::area(p) <= all * 1e-4).count();
            assert_eq!(
                slivers,
                0,
                "{align:?} w={w} saiu com {slivers} lasca(s) de {} peca(s)",
                band.len()
            );
        }
    }
}

/// **`needs_a_region` é UMA porta** — o painel e a booleana perguntam a mesma coisa. Se alguém
/// acrescentar um quarto modo e esquecer este método, o gate cai aqui em vez de na tela.
#[test]
fn the_question_of_needing_a_region_has_one_answer() {
    assert!(!StrokeAlign::Centre.needs_a_region());
    assert!(StrokeAlign::Inner.needs_a_region());
    assert!(StrokeAlign::Outer.needs_a_region());
    assert!(!spec(0.4, StrokeAlign::Centre).is_aligned());
    assert!(spec(0.4, StrokeAlign::Inner).is_aligned());
    assert!(
        !spec(0.0, StrokeAlign::Outer).is_aligned(),
        "largura zero nao deita faixa, logo nao ha o que alinhar"
    );
}
