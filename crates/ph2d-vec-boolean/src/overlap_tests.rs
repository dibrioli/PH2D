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

/// ⭐⭐⭐ **GATE — o VINCO vira ARCO, e a quina do artista fica.** Um entalhe CÔNCAVO cuja ponta
/// NÃO é nó do desenho (nasceu no cruzamento) sai com dois vértices lisos e um arco tangente entre
/// eles, e a área só CRESCE; a MESMA ponta declarada como nó do desenho (o CONTROLO) sai intacta,
/// ao bit — e uma ponta CONVEXA nascida no cruzamento também.
#[test]
fn o_vinco_vira_arco_e_a_quina_do_artista_fica() {
    // Um rectângulo com um entalhe fundo: a ponta em (2, 0.3) vira `~166°`, como o vinco das fotos.
    let pontos = [
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 3.0],
        [2.3, 3.0],
        [2.0, 0.3],
        [1.7, 3.0],
        [0.0, 3.0],
    ];
    let v: Vec<VecVertex> = pontos.iter().map(|p| canto(*p)).collect();
    let raio = 0.05;
    let fora: Vec<[f64; 2]> = pontos
        .iter()
        .copied()
        .filter(|p| *p != [2.0, 0.3])
        .collect();
    // Os nós do desenho levam a viragem que tinham lá — é contra ela que um vinco se mede.
    let com_viragem = |ps: &[[f64; 2]], vs: &[VecVertex]| -> Vec<([f64; 2], f64)> {
        ps.iter()
            .map(|p| {
                let i = vs.iter().position(|q| q.anchor == *p).expect("nó");
                (*p, crate::overlap::viragem_do_vertice(vs, i).unwrap_or(0.0))
            })
            .collect()
    };
    let nos_fora = com_viragem(&fora, &v);
    let alfa = crate::overlap::viragem_do_vertice(&v, 4)
        .expect("tangentes")
        .to_radians();
    let s = crate::overlap::arredonda_os_vincos(v.clone(), &nos_fora, raio);
    assert_eq!(
        s.len(),
        8,
        "o vinco não foi trocado por dois vértices: {s:?}"
    );
    for (i, p) in s.iter().enumerate() {
        let vira = crate::overlap::viragem_do_vertice(&s, i).unwrap_or(0.0);
        assert!(
            fora.contains(&p.anchor) || vira < 1e-6,
            "o vértice v{i} do arco vira {vira}° — o vinco ainda é uma quina: {s:?}"
        );
    }
    // O arco começa À DISTÂNCIA do raio: `r·tan(α/2)` da ponta, nos dois lados.
    let d = raio * (0.5 * alfa).tan();
    for p in s.iter().filter(|p| !fora.contains(&p.anchor)) {
        let dd = (p.anchor[0] - 2.0).hypot(p.anchor[1] - 0.3);
        assert!(
            (dd - d).abs() < 1e-9,
            "o arco começa a {dd} da ponta, e o raio pede {d}"
        );
    }
    // E é um ARCO de círculo, não só uma curva tangente: o meio da cúbica cai a `r` do centro
    // (a alça `(4/3)·tan(θ/4)` põe-no EXACTAMENTE na circunferência).
    let k = s
        .iter()
        .position(|p| !fora.contains(&p.anchor))
        .expect("o arco");
    let (va, vb) = (&s[k], &s[k + 1]);
    let ta = [
        va.out_handle[0] - va.anchor[0],
        va.out_handle[1] - va.anchor[1],
    ];
    let nt = ta[0].hypot(ta[1]);
    let centro = [-1.0_f64, 1.0].map(|lado| {
        [
            va.anchor[0] - lado * raio * ta[1] / nt,
            va.anchor[1] + lado * raio * ta[0] / nt,
        ]
    });
    let c = *centro
        .iter()
        .min_by(|x, y| {
            let dx = |q: &[f64; 2]| ((q[0] - vb.anchor[0]).hypot(q[1] - vb.anchor[1]) - raio).abs();
            dx(x).total_cmp(&dx(y))
        })
        .expect("dois lados");
    let meio = |i: usize| {
        0.125 * (va.anchor[i] + vb.anchor[i]) + 0.375 * (va.out_handle[i] + vb.in_handle[i])
    };
    let r_meio = (meio(0) - c[0]).hypot(meio(1) - c[1]);
    assert!(
        (r_meio - raio).abs() < 1e-9 * raio.max(1.0) + 1e-12,
        "o meio do arco está a {r_meio} do centro, e o raio é {raio}"
    );
    // Um côncavo arredondado só ACRESCENTA área.
    let area = |vs: &[VecVertex]| {
        crate::area(&VecPath {
            verts: vs.to_vec(),
            closed: true,
            ..VecPath::default()
        })
    };
    assert!(area(&s) > area(&v), "o arco tirou área a um vinco côncavo");
    // CONTROLO: a mesma ponta como NÓ do desenho, que JÁ virava assim, fica intacta.
    assert_eq!(
        crate::overlap::arredonda_os_vincos(v.clone(), &com_viragem(&pontos, &v), raio),
        v,
        "uma quina que o artista desenhou foi arredondada"
    );
    // O ENCAIXE: a ponta é um nó do desenho que ali era LISO (virava `0°`) — o motor da união
    // encaixou o cruzamento nele. Ela vira mais do que virava ⇒ é vinco, e arredonda igual.
    let mut encaixado = nos_fora.clone();
    encaixado.push(([2.0, 0.3], 0.0));
    assert_eq!(
        crate::overlap::arredonda_os_vincos(v.clone(), &encaixado, raio),
        s,
        "um vinco que o motor encaixou num nó liso ficou em quina"
    );
    // CONTROLO: uma ponta CONVEXA nascida no cruzamento fica (o vinco de uma dobra é côncavo).
    let convexo = vec![
        canto([0.0, 0.0]),
        canto([2.0, 0.0]),
        canto([0.12, 0.68]),
        canto([-1.0, 1.0]),
    ];
    let so_o_resto = com_viragem(&[[0.0, 0.0], [0.12, 0.68], [-1.0, 1.0]], &convexo);
    assert_eq!(
        crate::overlap::arredonda_os_vincos(convexo.clone(), &so_o_resto, raio),
        convexo,
        "uma ponta CONVEXA foi arredondada"
    );
}

/// ⭐⭐ **GATE — os nós LISOS dentro do arco SAEM.** O assado põe nós a `~0,01` junto da junta, e o
/// corte cai além deles: se ficassem, o contorno iria ao corte, voltaria ao nó e seguiria — um
/// laço que nenhuma viragem acusa (o nó é do desenho e as alças dele apontam para a frente). A
/// fixtura é o entalhe do gate irmão com um nó liso a meio de cada lado, DENTRO do alcance do
/// arco; a saída tem de ser a MESMA do entalhe sem eles.
#[test]
fn os_nos_lisos_dentro_do_arco_saem() {
    let ponta = [2.0, 0.3];
    let (esq, dir) = ([1.7, 3.0], [2.3, 3.0]);
    let a_meio = |a: [f64; 2], t: f64| {
        [
            ponta[0] + (a[0] - ponta[0]) * t,
            ponta[1] + (a[1] - ponta[1]) * t,
        ]
    };
    let sem_nos = [
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 3.0],
        dir,
        ponta,
        esq,
        [0.0, 3.0],
    ];
    let com_nos = [
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 3.0],
        dir,
        a_meio(dir, 0.05),
        ponta,
        a_meio(esq, 0.05),
        esq,
        [0.0, 3.0],
    ];
    let raio = 0.05;
    let nos = |ps: &[[f64; 2]]| -> Vec<([f64; 2], f64)> {
        ps.iter()
            .copied()
            .filter(|p| *p != ponta)
            .map(|p| (p, 0.0))
            .collect()
    };
    let arredonda = |ps: &[[f64; 2]]| {
        let v: Vec<VecVertex> = ps.iter().map(|p| canto(*p)).collect();
        crate::overlap::arredonda_os_vincos(v, &nos(ps), raio)
    };
    let (a, b) = (arredonda(&sem_nos), arredonda(&com_nos));
    // O CONTROLO de que os nós caem DENTRO do alcance do arco (senão a fixtura não contém nada).
    let alcance = a
        .iter()
        .map(|v| (v.anchor[0] - ponta[0]).hypot(v.anchor[1] - ponta[1]))
        .fold(f64::INFINITY, f64::min);
    let no = a_meio(dir, 0.05);
    assert!(
        (no[0] - ponta[0]).hypot(no[1] - ponta[1]) < alcance,
        "o nó a meio do lado está FORA do arco — a fixtura não o testa"
    );
    assert_eq!(b.len(), a.len(), "um nó liso dentro do arco ficou: {b:?}");
    // As âncoras de todos, e as alças do ARCO (o par que não estava na entrada). ⚠️ As alças das
    // rectas cortadas NÃO se comparam: são a mesma recta parametrizada a partir de outro nó.
    let perto = |p: [f64; 2], q: [f64; 2]| (p[0] - q[0]).hypot(p[1] - q[1]) < 1e-9;
    for (x, y) in a.iter().zip(&b) {
        assert!(
            perto(x.anchor, y.anchor),
            "o arco muda com um nó liso dentro dele: {:?} contra {:?}",
            x.anchor,
            y.anchor
        );
    }
    let k = a
        .iter()
        .position(|v| !sem_nos.contains(&v.anchor))
        .expect("o arco");
    assert!(
        perto(a[k].out_handle, b[k].out_handle) && perto(a[k + 1].in_handle, b[k + 1].in_handle)
    );
}

/// ⭐⭐ **GATE — o traço ASSADO (*Outline Stroke*) tem o mesmo limite do desenhado.**
#[test]
fn o_bico_do_traco_assado_e_o_do_documento() {
    let s = StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 1.0);
    let k = crate::expand::line_pen(&VecPath::default(), &s);
    assert!((k.miter_limit - ph2d_vec_scene::MITER_LIMIT).abs() < f64::EPSILON);
}
