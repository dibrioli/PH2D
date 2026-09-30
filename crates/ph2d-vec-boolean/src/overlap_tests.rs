//! Os gates da [`super::resolve_overlap`].

use super::*;
use ph2d_vec_scene::{Paint, Rgba8, StrokeSpec, VecVertex};

fn poligono(pts: &[[f64; 2]]) -> VecPath {
    VecPath {
        verts: pts.iter().map(|&p| VecVertex::corner(p)).collect(),
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(200, 120, 40, 255))),
        stroke: Some(StrokeSpec::new(Rgba8::new(20, 20, 20, 255), 0.1)),
        ..VecPath::default()
    }
}

/// Um «L» cujo braço vertical DESCE ALÉM da banda de baixo e volta para dentro dela — o
/// cotovelo de uma dobra forte, reduzido ao essencial: a face de dentro do braço atravessa a
/// aresta de baixo da banda DUAS vezes (em `x = 3` e em `x ≈ 2,67`).
fn cotovelo_cruzado() -> VecPath {
    poligono(&[
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 4.0],
        [3.0, 4.0],
        [3.0, -0.5],
        [2.0, 1.0],
        [0.0, 1.0],
    ])
}

fn cruzes(p: &VecPath) -> bool {
    crosses_itself(&crate::to_bez(p))
}

/// ⭐⭐⭐ **GATE — a silhueta de um contorno que se cruza NÃO se cruza, e a área é a do
/// preenchimento.**
///
/// ⚠️ **As duas metades são obrigatórias:** sem a primeira (o CONTROLO: a fixtura cruza-se) o
/// gate passaria sobre uma forma que nunca teve contacto; sem a da área, uma porta que devolvesse
/// o CASCO convexo também não se cruzaria.
#[test]
fn a_silhueta_de_um_contorno_cruzado_nao_se_cruza() {
    let p = cotovelo_cruzado();
    assert!(
        cruzes(&p),
        "a fixtura deixou de se cruzar — não contém o fenómeno"
    );
    let s = resolve_overlap(&p).expect("um contorno que se cruza tem silhueta");
    assert!(!cruzes(&s), "a silhueta ainda se cruza");
    // A área é a da união do caminho com o VAZIO pelo motor — a régua é que a porta não perca
    // nem invente região ao reconstruir os contornos.
    let area = crate::area(&s);
    // ⚠️ O motor devolve UMA peça por ilha (aqui: o corpo e o triângulo que o braço deixa abaixo
    // da banda), e a silhueta leva-as todas num caminho só — a régua é a SOMA.
    let pecas = crate::apply(&p, &VecPath::default(), crate::BoolOp::Union);
    assert!(
        pecas.len() >= 2,
        "a fixtura deixou de ter duas ilhas: {}",
        pecas.len()
    );
    let base: f64 = pecas.iter().map(crate::area).sum();
    assert!(
        (area - base).abs() < 1e-9,
        "a silhueta tem área {area} e a união do caminho com o vazio tem {base}"
    );
}

/// ⭐⭐ **GATE — sem contacto a porta NÃO corre, e a forma fica como era.**
///
/// ⚠️ É o que mantém toda a pele fora do cotovelo byte-idêntica: a porta só troca a geometria
/// quando há o que trocar.
#[test]
fn sem_contacto_a_porta_nao_corre() {
    let p = poligono(&[
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 4.0],
        [3.0, 4.0],
        [3.0, 1.0],
        [0.0, 1.0],
    ]);
    assert!(!cruzes(&p), "um L limpo não se cruza");
    assert!(resolve_overlap(&p).is_none(), "a porta correu sem contacto");
}

/// ⭐⭐ **GATE — o ESTILO é o da forma.** A porta muda a geometria e mais nada: o traço, a
/// opacidade e a mistura que a forma tinha são os que ela desenha a seguir.
#[test]
fn a_silhueta_leva_o_estilo_da_forma() {
    let mut p = cotovelo_cruzado();
    p.opacity = ph2d_vec_scene::Opacity::new(0.5);
    let s = resolve_overlap(&p).expect("silhueta");
    assert_eq!(s.fill, p.fill);
    assert_eq!(s.stroke, p.stroke);
    assert_eq!(s.opacity, p.opacity);
    assert_eq!(s.id, p.id);
}

/// ⭐ **GATE — um caminho ABERTO não tem silhueta**, mesmo que se cruze: não há interior.
#[test]
fn um_caminho_aberto_nao_tem_silhueta() {
    let mut p = cotovelo_cruzado();
    p.closed = false;
    assert!(resolve_overlap(&p).is_none());
}

/// ⭐ **GATE — dois segmentos que só se TOCAM não contam** (um toque num vértice não é contacto
/// que peça silhueta), e os VIZINHOS do mesmo contorno nunca contam.
#[test]
fn um_toque_nao_e_um_cruzamento() {
    // Um «8» que se toca num ponto, sem se atravessar: dois triângulos com um vértice comum.
    let p = poligono(&[
        [0.0, 0.0],
        [1.0, 1.0],
        [2.0, 0.0],
        [2.0, 2.0],
        [1.0, 1.0],
        [0.0, 2.0],
    ]);
    assert!(!cruzes(&p), "tocar num ponto foi lido como cruzar");
}

fn canto(p: [f64; 2]) -> VecVertex {
    VecVertex::corner(p)
}

/// ⭐⭐ **GATE — um segmento MINÚSCULO junto de uma quina é soldado**, e a quina fica UM vértice
/// com as duas tangentes reais (report do dono de 2026-09-29: a junta do traço era calculada sobre
/// um segmento de comprimento zero). CONTROLO: um segmento do tamanho da forma fica.
#[test]
fn um_segmento_minusculo_e_soldado_na_quina() {
    let tol = 1e-3;
    let v = vec![
        canto([0.0, 0.0]),
        canto([1.0, 0.0]),
        canto([1.0 + 1e-5, 1e-5]),
        canto([1.0, 1.0]),
        canto([0.0, 1.0]),
    ];
    let s = solda_os_segmentos_curtos(v, tol);
    assert_eq!(s.len(), 4, "o segmento de 1e-5 ficou: {s:?}");
    let q = vec![
        canto([0.0, 0.0]),
        canto([1.0, 0.0]),
        canto([1.0, 1.0]),
        canto([0.0, 1.0]),
    ];
    assert_eq!(
        solda_os_segmentos_curtos(q.clone(), tol),
        q,
        "o CONTROLO: um quadrado limpo foi mexido"
    );
}

/// ⭐⭐ **GATE — a alça que a SOLDA herda é limpa DEPOIS dela.** O vértice que fica leva a alça de
/// saída do fundido, que mora na âncora DELE — a distância de ruído da nova e, no caso medido
/// (a barra dobrada a `150°`), do lado de TRÁS. A junta lia aí uma meia-volta onde a quina tem `90°`.
#[test]
fn a_alca_herdada_da_solda_nao_torce_a_quina() {
    let tol = 1e-3;
    let mut minusculo = canto([1.0 + 1e-5, 0.0]);
    minusculo.out_handle = [1.0 - 1e-6, 1e-6];
    let v = vec![
        canto([0.0, 0.0]),
        canto([1.0, 0.0]),
        minusculo,
        canto([1.0, 1.0]),
        canto([0.0, 1.0]),
    ];
    let s = solda_os_segmentos_curtos(v, tol);
    assert_eq!(s.len(), 4, "o segmento minúsculo não foi soldado: {s:?}");
    let vira = crate::overlap::viragem_do_vertice(&s, 1).expect("a quina tem tangentes");
    assert!(
        (vira - 90.0).abs() < 1e-6,
        "a quina de 90° lê {vira}° — a alça herdada ficou a torcê-la: {s:?}"
    );
}

/// ⭐⭐ **GATE — a alça que caiu na ponta de LÁ encaixa nela.** Numa recta cujas alças estão
/// uma EXACTAMENTE numa ponta e a outra a ruído da MESMA ponta, a tangente na outra ponta recua para
/// a alça de ruído (a kurbo só troca por coincidência exacta) e lê uma direcção falsa. Os dois ramos
/// cruzados, cada um no seu lado de um quadrado: o de baixo tem a alça de SAÍDA caída no fim, o de
/// cima tem a de ENTRADA caída no começo — e as quatro quinas têm de ler `90°`.
///
/// ⚠️ Os dois ramos só mordem com a OUTRA alça exacta na ponta; com ela noutro sítio a kurbo nunca
/// recua até à alça de ruído, e foi por isso que a mutação que os apagava sobreviveu à barra dobrada.
#[test]
fn uma_alca_caida_na_ponta_de_la_encaixa_nela() {
    let tol = 1e-3;
    let mut v = vec![
        canto([0.0, 0.0]),
        canto([1.0, 0.0]),
        canto([1.0, 1.0]),
        canto([0.0, 1.0]),
    ];
    // Baixo (0 → 1): p1 a ruído do FIM, p2 exacto no fim.
    v[0].out_handle = [1.0 - 1e-7, 1e-7];
    // Cima (2 → 3): p1 exacto no começo, p2 a ruído do COMEÇO.
    v[3].in_handle = [1.0 - 1e-7, 1.0 - 1e-7];
    let s = solda_os_segmentos_curtos(v, tol);
    for i in 0..4 {
        let vira = crate::overlap::viragem_do_vertice(&s, i).expect("a quina tem tangentes");
        assert!(
            (vira - 90.0).abs() < 1e-6,
            "a quina {i} lê {vira}° — a alça caída na ponta de lá ficou a torcê-la: {s:?}"
        );
    }
}

/// ⭐⭐ **GATE — a viragem máxima é a do limite do bico**: numa quina que vira exactamente
/// [`viragem_maxima`] o bico mede o limite (`1/sin(θ/2)`, `θ` por dentro). Sem isto os dois números
/// derivavam cada um para o seu lado e uma quina que sobra podia virar chanfro.
#[test]
fn a_viragem_maxima_e_a_do_limite_do_bico() {
    let dentro = (180.0 - viragem_maxima()).to_radians();
    let bico = 1.0 / (dentro / 2.0).sin();
    assert!(
        (bico - ph2d_vec_scene::MITER_LIMIT).abs() < 1e-9,
        "o bico na viragem máxima mede {bico} contra o limite {}",
        ph2d_vec_scene::MITER_LIMIT
    );
    assert!(
        viragem_maxima() > 155.0,
        "a viragem máxima ({}) corta quinas verdadeiras do contacto (medidas até 153°)",
        viragem_maxima()
    );
}

/// ⭐⭐ **GATE — o traço ASSADO (*Outline Stroke*) tem o mesmo limite do desenhado.**
#[test]
fn o_bico_do_traco_assado_e_o_do_documento() {
    let s = StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 1.0);
    let k = crate::expand::line_pen(&VecPath::default(), &s);
    assert!((k.miter_limit - ph2d_vec_scene::MITER_LIMIT).abs() < f64::EPSILON);
}
