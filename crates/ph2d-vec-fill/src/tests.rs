//! Gates do **BALDE** (plano 40) — a lei pura, sem cena e sem ponteiro.

use super::*;
use ph2d_vec_scene::VertexKind;

fn v(x: f64, y: f64) -> VecVertex {
    VecVertex {
        anchor: [x, y],
        in_handle: [x, y],
        out_handle: [x, y],
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    }
}

/// Um quadrado FECHADO de lado `l` centrado na origem.
fn quadrado(l: f64) -> (Vec<VecVertex>, bool) {
    let h = l * 0.5;
    (vec![v(-h, -h), v(h, -h), v(h, h), v(-h, h)], true)
}

/// ⭐ **Um anel sozinho tem UMA face limitada**, e o clique lá dentro acha-a.
///
/// ⚠️ Ele não tem cruzamento nenhum, então só existe porque a rede o transforma num **laço** — sem
/// isso um contorno fechado não tem meia-aresta e a face dele seria invisível ao passeio.
#[test]
fn a_lone_ring_has_one_bounded_face_and_the_click_finds_it() {
    let r = rede(&[quadrado(100.0)], 0.0);
    assert_eq!(r.arcos.len(), 1, "o anel tem de virar UM arco-laco");
    assert_eq!(
        r.arcos[0].de, r.arcos[0].ate,
        "e as duas pontas sao o MESMO no'"
    );
    let f = r.face_em([0.0, 0.0]).expect("o centro esta' dentro");
    assert!((f.area - 10_000.0).abs() < 1.0, "area {}", f.area);
    assert!(r.face_em([500.0, 500.0]).is_none(), "fora nao e' face");
}

/// ⭐⭐⭐ **O PEDIDO: quatro linhas SOLTAS que se cruzam fecham um quadrado.**
///
/// Nenhuma delas é fechada, nenhuma tem dentro — é exactamente o caso que o arranjo do Shape
/// Builder não sabe exprimir (`região(M) = ∩M − ∪¬M` não se define para um traço aberto).
#[test]
fn four_open_lines_that_cross_enclose_a_square() {
    let a = 60.0;
    let contornos = vec![
        (vec![v(-a, -20.0), v(a, -20.0)], false),
        (vec![v(-a, 20.0), v(a, 20.0)], false),
        (vec![v(-20.0, -a), v(-20.0, a)], false),
        (vec![v(20.0, -a), v(20.0, a)], false),
    ];
    let r = rede(&contornos, 0.0);
    let f = r
        .face_em([0.0, 0.0])
        .expect("o miolo das quatro linhas e' uma face");
    assert!(
        (f.area - 1600.0).abs() < 1.0,
        "o miolo e' 40x40: {}",
        f.area
    );
    assert_eq!(f.arcos.len(), 4, "a fronteira sao QUATRO arcos inteiros");
    // E a geometria sai com um vértice por canto — não um polígono achatado.
    let g = r.geometria(&f);
    assert_eq!(g.len(), 4, "quatro cantos: {g:?}");
}

/// ⚠️ **A de MENOR área**, e é ela que resolve o aninhamento.
#[test]
fn the_click_takes_the_innermost_face() {
    let r = rede(&[quadrado(100.0), quadrado(40.0)], 0.0);
    let f = r
        .face_em([0.0, 0.0])
        .expect("o centro esta' dentro dos dois");
    assert!(
        (f.area - 1600.0).abs() < 1.0,
        "o clique tem de apanhar o quadrado de DENTRO: {}",
        f.area
    );
    // E entre os dois quadrados a face é o anel — área = 10 000 − 1 600.
    let anel = r.face_em([-45.0, 0.0]).expect("entre os dois ha' face");
    assert!(
        anel.area > 1600.0,
        "a face entre os dois anéis nao pode ser a de dentro: {}",
        anel.area
    );
}

/// ⛔ **Fora de tudo não é face** — e a recusa é a resposta certa, não um erro.
#[test]
fn a_click_outside_everything_is_not_a_face() {
    let r = rede(&[quadrado(100.0)], 0.0);
    assert!(r.face_em([80.0, 80.0]).is_none());
}

/// ⚠️ **A face de FORA tem área negativa**, e é isso que a mantém fora da escolha sem uma regra à
/// parte. Sem esta metade, *"a de menor área"* escolheria a face errada num documento inteiro.
#[test]
fn the_outer_face_comes_out_negative() {
    let r = rede(&[quadrado(100.0)], 0.0);
    let faces = r.faces();
    assert_eq!(faces.len(), 2, "um laco da' duas faces: dentro e fora");
    assert!(
        faces.iter().any(|f| f.area < 0.0),
        "nenhuma face saiu negativa: {:?}",
        faces.iter().map(|f| f.area).collect::<Vec<_>>()
    );
}

/// ⭐⭐ **A CURVA sobrevive**: a fronteira é feita dos arcos, então as alças chegam à forma.
///
/// ⚠️ É o que separa este balde do do Inkscape (que traça pixels) e do do Flip (que devolve
/// polígono): num círculo cortado em dois arcos, a geometria da face tem de trazer alças
/// diferentes da âncora.
#[test]
fn the_filled_shape_keeps_the_curve_not_a_polygon() {
    let c = ph2d_vec_scene::ellipse([0.0, 0.0], 50.0, 50.0);
    let linha = (vec![v(-80.0, 0.0), v(80.0, 0.0)], false);
    let r = rede(&[(c.verts.clone(), true), linha], 0.0);
    let f = r.face_em([0.0, 20.0]).expect("a metade de cima do circulo");
    let g = r.geometria(&f);
    assert!(
        g.iter().any(|v| v.out_handle != v.anchor),
        "a forma saiu sem alcas — isto e' um poligono, nao a curva"
    );
    // Meia bola de raio 50: ~3927. A recta corta-a ao meio.
    assert!(
        (f.area - 3927.0).abs() < 60.0,
        "a metade de cima do circulo mede {}",
        f.area
    );
}

/// ⭐⭐⭐ **A RECEITA É O PONTO, e é por isso que o preenchimento pode ser VIVO.**
///
/// Report do Enio (2026-09-01): *"se movo os nós da linha, o preenchimento não acompanha. A área
/// deveria permanecer perfeitamente preenchida mesmo modificando o path."*
///
/// ⚠️ **Guardar a lista de ARCOS não resolveria**: um arco nasce de um corte em fracções, e mover
/// um nó **muda os cruzamentos**, logo muda a própria lista. *Qualquer receita feita de pedaços da
/// rede é uma receita sobre uma rede que já não existe.* O ponto sobrevive.
#[test]
fn the_same_seed_names_the_new_face_after_a_wall_moves() {
    let quadro = |dir: f64| {
        vec![
            (vec![v(-60.0, -20.0), v(dir, -20.0)], false),
            (vec![v(-60.0, 20.0), v(dir, 20.0)], false),
            (vec![v(-20.0, -60.0), v(-20.0, 60.0)], false),
            (vec![v(20.0, -60.0), v(20.0, 60.0)], false),
        ]
    };
    let semente = [0.0, 0.0];
    let antes = rede(&quadro(60.0), 0.0)
        .face_em(semente)
        .expect("o miolo existe antes");
    // A parede de cima sobe: o miolo passa a ser mais alto.
    //
    // ⚠️ **A alça acompanha a âncora** — a 1.ª redacção mexeu só na âncora e a recta virou uma
    // CURVA a abaular para dentro (área `1 862` em vez de `2 600`). É a mesma lei que o
    // `weld::mover_ponta` escreve, e uma fixtura que a ignora mede outra coisa.
    let mut movido = quadro(60.0);
    movido[1].0 = vec![v(-60.0, 45.0), v(60.0, 45.0)];
    let r2 = rede(&movido, 0.0);
    let depois = r2.face_em(semente).expect("o miolo ainda existe depois");
    assert!(
        depois.area > antes.area * 1.5,
        "a face nao acompanhou a parede: {} contra {}",
        depois.area,
        antes.area
    );
    // …e a área nova continua a conter o ponto que a nomeia — senão a receita perder-se-ia.
    assert!(
        r2.face_em(semente).is_some(),
        "a semente deixou de nomear a regiao que ela mesma produziu"
    );
}

/// ⛔⛔ **UMA CÓPIA COINCIDENTE JÁ NÃO ENVENENA AS VIZINHAS** — e este gate mudou de lado.
///
/// # A história, porque ela é a lição
///
/// Ele nasceu (2026-09-01) a afirmar o CONTRÁRIO: a forma que o balde depositava, posta de volta na
/// rede, punha lá arestas coincidentes e as regiões vizinhas deixavam de fechar. Era o mecanismo que
/// justificava manter o preenchimento **fora** da rede.
///
/// ⭐ Horas depois, o report *"a depender da posição dos pontos o preenchimento some"* mostrou que a
/// mesma fraqueza tinha outra porta — **duas paredes autoradas a caírem uma em cima da outra** —, e
/// aí ela deixou de ser tolerável: nenhum artista aceita perder o preenchimento do outro lado do
/// desenho por ter encostado dois traços. O `descartar_duplicados` cura as duas.
///
/// ⇒ **A política FICA e o motivo dela mudou.** Um preenchimento continua fora da rede — mas agora
/// por ser derivado, não por envenenar: o descarte guarda **um** dos dois arcos, e se o guardado
/// fosse o da cópia, a rede passaria a depender de uma geometria que outro motor reescreve.
#[test]
fn a_duplicate_of_a_face_no_longer_poisons_its_neighbours() {
    let base = vec![
        (vec![v(-60.0, -20.0), v(60.0, -20.0)], false),
        (vec![v(-60.0, 20.0), v(60.0, 20.0)], false),
        (vec![v(-20.0, -60.0), v(-20.0, 60.0)], false),
        (vec![v(20.0, -60.0), v(20.0, 60.0)], false),
    ];
    let r = rede(&base, 0.0);
    let miolo = r.face_em([0.0, 0.0]).expect("o miolo");
    let mut com_copia = base.clone();
    com_copia.push((r.geometria(&miolo), true));
    let depois = rede(&com_copia, 0.0)
        .face_em([0.0, 0.0])
        .expect("a copia coincidente nao pode fazer a regiao desaparecer");
    assert!(
        (depois.area - miolo.area).abs() < 1.0,
        "a regiao mudou de area com a copia por cima: {} contra {}",
        depois.area,
        miolo.area
    );
}

/// ⛔⛔ **DOIS ARCOS SOBREPOSTOS não podem destruir o passeio INTEIRO.**
///
/// Report do Enio (2026-09-01): *"a depender da posição dos pontos o preenchimento some."*
/// ⚠️ **Medido:** com uma parede a cair exactamente em cima de outra, a rede passava de **3 faces a
/// 1** — e a região do outro lado do desenho perdia o preenchimento junto, sem ter nada a ver com
/// aquilo. *Duas meias-arestas com a mesma direcção de saída são indistinguíveis para o passeio.*
#[test]
fn a_wall_landing_on_another_does_not_take_the_whole_network_with_it() {
    let base = |x: f64| {
        vec![
            (
                vec![
                    v(-60.0, -40.0),
                    v(20.0, -40.0),
                    v(20.0, 40.0),
                    v(-60.0, 40.0),
                ],
                true,
            ),
            (vec![v(20.0, 40.0), v(x, 0.0), v(20.0, -40.0)], false),
        ]
    };
    // Controle: com a bolsa aberta há as duas regiões.
    let aberta = rede(&base(60.0), 0.0);
    assert!(aberta.face_em([-20.0, 0.0]).is_some(), "a de dentro");
    assert!(aberta.face_em([25.0, 0.0]).is_some(), "a bolsa");
    // E com a parede EM CIMA da aresta, a de dentro SOBREVIVE (a bolsa tem area zero e some).
    let colada = rede(&base(20.0), 0.0);
    let dentro = colada
        .face_em([-20.0, 0.0])
        .expect("a regiao do rectangulo nao tem nada a ver com a bolsa degenerada");
    assert!((dentro.area - 6400.0).abs() < 1.0, "area {}", dentro.area);
}

/// ⚠️ **Duas curvas DIFERENTES entre os mesmos dois nós são uma LENTE, não um duplicado.**
///
/// Sem esta metade, o descarte de duplicados comeria uma região legítima — e o par de nós sozinho
/// não distingue as duas coisas.
#[test]
fn two_different_curves_between_the_same_nodes_are_a_lens() {
    let arco = |k: f64| {
        vec![
            VecVertex {
                anchor: [-50.0, 0.0],
                in_handle: [-50.0, 0.0],
                out_handle: [-20.0, k],
                kind: VertexKind::Smooth,
                corner_radius: 0.0,
            },
            VecVertex {
                anchor: [50.0, 0.0],
                in_handle: [20.0, k],
                out_handle: [50.0, 0.0],
                kind: VertexKind::Smooth,
                corner_radius: 0.0,
            },
        ]
    };
    let r = rede(&[(arco(60.0), false), (arco(-60.0), false)], 0.0);
    assert_eq!(r.arcos.len(), 2, "as duas curvas nao podem colapsar numa");
    assert!(
        r.face_em([0.0, 0.0]).is_some(),
        "a lente entre as duas curvas e' uma regiao legitima"
    );
}

/// ⭐⭐⭐ **A SEMENTE RE-SEMEADA SOBREVIVE a uma parede que varre por cima do clique.**
///
/// ⚠️ **Medido antes:** com a semente a `0,5` de uma parede, arrastá-la por cima do ponto fazia a
/// face deixar de o conter, e o preenchimento parava de seguir. Re-semear no ponto mais **fundo**
/// da face a cada re-cozimento mantém a receita longe da borda.
#[test]
fn reseeding_deep_inside_survives_a_wall_sweeping_over_the_click() {
    let quadro = |topo: f64| {
        vec![
            (vec![v(-60.0, -20.0), v(60.0, -20.0)], false),
            (vec![v(-60.0, topo), v(60.0, topo)], false),
            (vec![v(-20.0, -60.0), v(-20.0, 60.0)], false),
            (vec![v(20.0, -60.0), v(20.0, 60.0)], false),
        ]
    };
    // O clique caiu perto do tecto (`y = 19,5`, com o tecto em `20`).
    let mut semente = [0.0, 19.5];
    let mut vivo = 0;
    for topo in [20.0_f64, 18.0, 15.0, 10.0, 5.0, 0.0, -10.0] {
        let r = rede(&quadro(topo), 0.0);
        let Some(f) = r.face_em(semente) else { break };
        vivo += 1;
        semente = r.interior_point(&f).expect("a face tem miolo");
    }
    assert_eq!(
        vivo, 7,
        "o preenchimento perdeu a regiao a meio da varredura (sobreviveu a {vivo} passos de 7)"
    );
    // ⛔ E o CONTROLE: sem re-semear, a mesma varredura perde-a.
    let mut fixa = [0.0, 19.5];
    let mut sem = 0;
    for topo in [20.0_f64, 18.0, 15.0, 10.0, 5.0, 0.0, -10.0] {
        if rede(&quadro(topo), 0.0).face_em(fixa).is_none() {
            break;
        }
        sem += 1;
    }
    assert!(
        sem < 7,
        "o controle nao mede nada: a semente fixa sobreviveu a varredura toda"
    );
    let _ = &mut fixa;
}

/// ⚠️ **Numa face CÔNCAVA o centroide pode cair FORA** — e aí o ponto vem da grelha.
#[test]
fn the_interior_point_of_a_concave_face_is_inside_it() {
    // Um "L": o centroide do contorno cai no canto que falta.
    let l = vec![
        v(-40.0, -40.0),
        v(40.0, -40.0),
        v(40.0, -20.0),
        v(-20.0, -20.0),
        v(-20.0, 40.0),
        v(-40.0, 40.0),
    ];
    let r = rede(&[(l, true)], 0.0);
    let f = r.face_em([-30.0, 0.0]).expect("o braco vertical do L");
    let p = r.interior_point(&f).expect("o L tem miolo");
    assert!(
        r.face_em(p).is_some_and(|g| (g.area - f.area).abs() < 1.0),
        "o ponto interior caiu fora da propria face: {p:?}"
    );
}

/// ⭐⭐⭐ **UMA PONTA QUE POUSA A MEIO PIXEL DA PAREDE É UM TOQUE** — o report de 2026-09-02:
/// *"a depender da posição dos pontos o preenchimento ainda some"*.
///
/// A fixtura é a foto dele: um rectângulo arredondado e uma curva que sai do lado direito e volta
/// a ele, fechando uma bolsa. ⚠️ **Medido antes da cura:** com a ponta a `0,05` FORA da aresta a
/// bolsa deixava de ser região — a flecha de uma recta é zero, e o toque só contava dentro dela.
/// Mexer no nó fazia a topologia **piscar** entre uma região e duas.
#[test]
fn an_endpoint_half_a_pixel_off_the_wall_still_closes_the_pocket() {
    let rr = ph2d_vec_scene::rounded_rect([-100.0, -100.0], [100.0, 100.0], 30.0);
    let bolsa = |ax: f64| {
        vec![
            VecVertex {
                anchor: [ax, -50.0],
                in_handle: [ax, -50.0],
                out_handle: [ax + 60.0, -40.0],
                kind: VertexKind::Smooth,
                corner_radius: 0.0,
            },
            VecVertex {
                anchor: [180.0, 0.0],
                in_handle: [180.0, -40.0],
                out_handle: [180.0, 40.0],
                kind: VertexKind::Smooth,
                corner_radius: 0.0,
            },
            VecVertex {
                anchor: [ax, 50.0],
                in_handle: [ax + 60.0, 40.0],
                out_handle: [ax, 50.0],
                kind: VertexKind::Smooth,
                corner_radius: 0.0,
            },
        ]
    };
    // As pontas oscilam à volta da parede (`x = 100`): meio pixel fora, um dentro, dois fora.
    for ax in [100.05_f64, 100.5, 101.0, 102.0, 99.9, 99.0] {
        let paredes = [(rr.verts.clone(), true), (bolsa(ax), false)];
        let r = rede(&paredes, 3.0);
        let bolsa_f = r
            .face_em([150.0, 0.0])
            .unwrap_or_else(|| panic!("com a ponta em x={ax} a bolsa deixou de ser regiao"));
        assert!(
            (bolsa_f.area - 6513.0).abs() < 200.0,
            "x={ax}: a bolsa tem de ser SO' a bolsa, nao a bolsa mais o rectangulo: {}",
            bolsa_f.area
        );
        let dentro = r.face_em([0.0, 0.0]).expect("o rectangulo");
        assert!(
            (dentro.area - 39224.0).abs() < 200.0,
            "x={ax}: o rectangulo fundiu-se com a bolsa: {}",
            dentro.area
        );
    }
    // ⛔ CONTROLE: sem folga, a ponta meio pixel fora ABRE a bolsa — é o defeito que o gate mede.
    let paredes = [(rr.verts.clone(), true), (bolsa(100.5), false)];
    assert!(
        rede(&paredes, 0.0).face_em([150.0, 0.0]).is_none(),
        "o controle nao mede nada: sem folga a bolsa ja' fechava"
    );
}

/// ⚠️ **A folga é uma CERCA**: uma ponta a mais de `folga` da parede fica onde está, e a bolsa fica
/// aberta — que é a verdade daquele desenho.
#[test]
fn an_endpoint_beyond_the_gap_is_left_alone() {
    let paredes = vec![
        (vec![v(-60.0, 0.0), v(60.0, 0.0)], false),
        (vec![v(0.0, 8.0), v(0.0, 60.0)], false),
    ];
    let mut perto = paredes.clone();
    aproximar_pontas(&mut perto, 10.0);
    assert_eq!(
        perto[1].0[0].anchor,
        [0.0, 0.0],
        "a 8 de distancia, com folga 10, a ponta vai a' parede"
    );
    let mut longe = paredes.clone();
    aproximar_pontas(&mut longe, 5.0);
    assert_eq!(
        longe[1].0[0].anchor,
        [0.0, 8.0],
        "a 8 de distancia, com folga 5, a ponta fica"
    );
}

/// ⚠️ **Duas pontas que se encontram vão as duas para o MEIO** — a junta não depende da ordem.
#[test]
fn two_loose_ends_within_the_gap_meet_in_the_middle() {
    let mut c = vec![
        (vec![v(-60.0, 0.0), v(-1.0, 0.0)], false),
        (vec![v(1.0, 2.0), v(60.0, 0.0)], false),
    ];
    aproximar_pontas(&mut c, 5.0);
    assert_eq!(c[0].0[1].anchor, [0.0, 1.0]);
    assert_eq!(c[1].0[0].anchor, [0.0, 1.0]);
    // E a alça acompanhou a âncora.
    assert_eq!(c[0].0[1].out_handle, [0.0, 1.0]);
}
