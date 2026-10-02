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
    // ⚠️ O vinco ARREDONDADO (`RAIO_DO_VINCO`) é a única diferença deliberada, e ele só ACRESCENTA
    // área, e pouca: nunca mais do que um quadrado do raio por vértice da união.
    let caixa = kurbo::Shape::bounding_box(&crate::to_bez(&p));
    let r = crate::overlap::RAIO_DO_VINCO * caixa.width().hypot(caixa.height());
    assert!(
        area >= base - 1e-9 && area - base <= r * r * s.verts.len() as f64,
        "a silhueta tem área {area} e a união do caminho com o vazio tem {base} (raio {r})"
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

/// ⭐⭐ **GATE — o traço ASSADO (*Outline Stroke*) tem o mesmo limite do desenhado.**
#[test]
fn o_bico_do_traco_assado_e_o_do_documento() {
    let s = StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 1.0);
    let k = crate::expand::line_pen(&VecPath::default(), &s);
    assert!((k.miter_limit - ph2d_vec_scene::MITER_LIMIT).abs() < f64::EPSILON);
}

/// ⭐⭐ **GATE — o fecho do contorno a UM ULP do vértice de partida não é um cruzamento.**
///
/// Os números são os MEDIDOS (F41, a pose do dono a `98°`): o último segmento achatado acabava em
/// `…086966` e o primeiro começava em `…086967`, os dois quase colineares — e o detector lia um par
/// que se atravessa. ⚠️ **As duas metades são obrigatórias:** sem o CONTROLO (a fixtura tem MESMO a
/// folga de um ULP) o gate passaria sobre um fecho de bits iguais, que nunca teve o defeito.
#[test]
fn um_fecho_a_um_ulp_do_inicio_nao_e_um_cruzamento() {
    let b0: [f64; 2] = [-7.795_316_120_352_435, 4.180_012_725_086_967];
    let b1 = [-7.793_159_739_362_697, 4.143_647_140_753_746];
    let a0 = [-7.794_741_701_595_443, 4.223_965_945_657_155];
    let a1: [f64; 2] = [-7.795_316_120_352_435, 4.180_012_725_086_966];
    assert_ne!(a1, b0, "CONTROLO: a fixtura perdeu a folga de um ULP");
    assert!(
        (a1[1] - b0[1]).abs() < 1e-14,
        "CONTROLO: a folga deixou de ser de arredondamento"
    );
    let mut bez = BezPath::new();
    bez.move_to((b0[0], b0[1]));
    bez.line_to((b1[0], b1[1]));
    bez.line_to((-6.0, 4.18));
    bez.line_to((a0[0], a0[1]));
    bez.line_to((a1[0], a1[1]));
    bez.close_path();
    assert!(
        !crosses_itself(&bez),
        "um fecho a um ULP do início foi lido como cruzamento"
    );
}

/// ⭐⭐ **GATE — um passo de UM ULP no MEIO do contorno não é um cruzamento** — o caso que o gate
/// do dono de facto tinha: o achatamento emitiu o ponto calculado (`…086966`) e depois o vértice
/// guardado (`…086967`), num nó LISO longe do fecho. A cura só do fecho deixou-o vermelho.
#[test]
fn um_passo_de_um_ulp_no_meio_nao_e_um_cruzamento() {
    let calc: [f64; 2] = [-7.795_316_120_352_435, 4.180_012_725_086_966];
    let guardado: [f64; 2] = [-7.795_316_120_352_435, 4.180_012_725_086_967];
    assert_ne!(
        calc, guardado,
        "CONTROLO: a fixtura perdeu a folga de um ULP"
    );
    let mut bez = BezPath::new();
    bez.move_to((-6.0, 4.18));
    bez.line_to((-7.794_741_701_595_443, 4.223_965_945_657_155));
    bez.line_to((calc[0], calc[1]));
    bez.line_to((guardado[0], guardado[1]));
    bez.line_to((-7.793_159_739_362_697, 4.143_647_140_753_746));
    bez.close_path();
    assert!(
        !crosses_itself(&bez),
        "um passo de um ULP no meio do contorno foi lido como cruzamento"
    );
}

/// Uma estrela de cinco pontas, anti-horária, à volta de `(2, 2)`.
fn estrela() -> (VecPath, Vec<[f64; 2]>) {
    let mut pts = Vec::new();
    let mut pontas = Vec::new();
    for k in 0..10 {
        let a = std::f64::consts::FRAC_PI_2 + f64::from(k) * std::f64::consts::PI / 5.0;
        let r = if k % 2 == 0 { 1.5 } else { 0.6 };
        let p = [2.0 + r * a.cos(), 2.0 + r * a.sin()];
        if k % 2 == 0 {
            pontas.push(p);
        }
        pts.push(p);
    }
    (poligono(&pts), pontas)
}

/// ⭐⭐ **GATE — a ABERTURA (F46) não come a ponta que o artista DESENHOU, mesmo afiada pela
/// deformação.** Na abertura todo nó do artista é parede, seja qual for a viragem de repouso dele:
/// aqui as viragens de repouso são `30°` mais brandas que as de agora (a deformação afiou tudo), e
/// as cinco pontas ficam AO BIT.
///
/// ⚠️ O CONTROLO: sem quinas a mesma silhueta arredonda as pontas — senão o gate não mede a parede,
/// mede uma abertura que não actua.
#[test]
fn a_abertura_nao_come_a_ponta_desenhada() {
    let (star, pontas) = estrela();
    let afiadas: Vec<([f64; 2], f64)> = quinas_de(&star)
        .into_iter()
        .map(|(p, v)| (p, v - 30.0))
        .collect();
    let com = silhueta_da_pele(&star, &afiadas).unwrap_or_else(|| star.clone());
    for p in &pontas {
        assert!(
            com.verts.iter().any(|v| v.anchor == *p),
            "a ponta {p:?} saiu da silhueta"
        );
    }
    let sem = silhueta_da_pele(&star, &[]).expect("controlo: a bola actua sem quinas");
    let ficaram = pontas
        .iter()
        .filter(|p| sem.verts.iter().any(|v| v.anchor == **p))
        .count();
    assert_eq!(
        ficaram, 0,
        "sem quinas as pontas ficaram — a abertura não actua e o gate não mede a parede"
    );
}

/// A distância de `q` à polilinha FECHADA `anel`.
fn ate_a_polilinha(q: [f64; 2], anel: &[VecVertex]) -> f64 {
    (0..anel.len())
        .map(|i| {
            let (a, b) = (anel[i].anchor, anel[(i + 1) % anel.len()].anchor);
            let d = [b[0] - a[0], b[1] - a[1]];
            let l2 = d[0] * d[0] + d[1] * d[1];
            let t = if l2 > 0.0 {
                (((q[0] - a[0]) * d[0] + (q[1] - a[1]) * d[1]) / l2).clamp(0.0, 1.0)
            } else {
                0.0
            };
            (q[0] - a[0] - t * d[0]).hypot(q[1] - a[1] - t * d[1])
        })
        .fold(f64::INFINITY, f64::min)
}

/// O maior desvio de um meio de segmento da união à polilinha de entrada.
fn desvio_da_uniao(entrada: &VecPath, da_borda: bool) -> f64 {
    let u = if da_borda {
        uniao_da_borda(entrada)
    } else {
        resolve_overlap(entrada)
    }
    .expect("o oito cruza-se");
    std::iter::once(&u.verts)
        .chain(u.subpaths.iter().map(|c| &c.verts))
        .flat_map(|vs| {
            (0..vs.len()).map(move |i| {
                let (a, b) = (vs[i].anchor, vs[(i + 1) % vs.len()].anchor);
                [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0]
            })
        })
        .map(|m| ate_a_polilinha(m, &entrada.verts))
        .fold(0.0, f64::max)
}

/// ⭐⭐⭐ **A [`uniao_da_borda`] não desloca a polilinha** — cada segmento dela está em cima da
/// entrada; o controlo (a [`resolve_overlap`] do desenho, soldada) desloca-a, senão o gate não
/// mediria nada.
///
/// ⛔ É o mecanismo do fio de `1 px` da foto de 2026-10-02 (a imagem presa a `120°`): a solda
/// trocava os segmentos mais curtos que [`SOLDA_DA_QUINA`] por cordas, o arco da bola encostava na
/// corda e o fundo aparecia entre ele e a borda da malha. A borda de uma malha densa junto de uma
/// junta TEM segmentos assim — este «oito» também (`2 000` nós, segmentos de `2,1e-3` a `4,4e-3`
/// contra uma solda de `2,2e-3`: os mais curtos fundem-se).
#[test]
fn a_uniao_do_fecho_da_borda_nao_desloca_a_polilinha() {
    let n = 2000;
    let oito = poligono(
        &(0..n)
            .map(|k| {
                let t = std::f64::consts::TAU * f64::from(k) / f64::from(n);
                [t.cos(), (2.0 * t).sin() / 2.0]
            })
            .collect::<Vec<_>>(),
    );
    // ⚠️ O piso é a precisão do MOTOR de varredura, e não zero: medido, a união sem solda fica a
    // `1,8e-7` da entrada e a soldada a `2,0e-5`. `1/1000` da solda (`2,2e-6`) fica uma ordem de
    // cada lado.
    let piso = SOLDA_DA_QUINA * 1e-3 * 2.0_f64.hypot(1.0);
    let sem_solda = desvio_da_uniao(&oito, true);
    assert!(
        sem_solda < piso,
        "a união sem solda desviou {sem_solda:e} (piso {piso:e})"
    );
    let com_solda = desvio_da_uniao(&oito, false);
    assert!(
        com_solda > piso,
        "o controlo (a solda) não desviou ({com_solda:e}) — o oito não exercita a solda"
    );
}
