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
    let r = rede(&[quadrado(100.0)]);
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
    let r = rede(&contornos);
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
    let r = rede(&[quadrado(100.0), quadrado(40.0)]);
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
    let r = rede(&[quadrado(100.0)]);
    assert!(r.face_em([80.0, 80.0]).is_none());
}

/// ⚠️ **A face de FORA tem área negativa**, e é isso que a mantém fora da escolha sem uma regra à
/// parte. Sem esta metade, *"a de menor área"* escolheria a face errada num documento inteiro.
#[test]
fn the_outer_face_comes_out_negative() {
    let r = rede(&[quadrado(100.0)]);
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
    let r = rede(&[(c.verts.clone(), true), linha]);
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
    let antes = rede(&quadro(60.0))
        .face_em(semente)
        .expect("o miolo existe antes");
    // A parede de cima sobe: o miolo passa a ser mais alto.
    //
    // ⚠️ **A alça acompanha a âncora** — a 1.ª redacção mexeu só na âncora e a recta virou uma
    // CURVA a abaular para dentro (área `1 862` em vez de `2 600`). É a mesma lei que o
    // `weld::mover_ponta` escreve, e uma fixtura que a ignora mede outra coisa.
    let mut movido = quadro(60.0);
    movido[1].0 = vec![v(-60.0, 45.0), v(60.0, 45.0)];
    let r2 = rede(&movido);
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
    let r = rede(&base);
    let miolo = r.face_em([0.0, 0.0]).expect("o miolo");
    let mut com_copia = base.clone();
    com_copia.push((r.geometria(&miolo), true));
    let depois = rede(&com_copia)
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
    let aberta = rede(&base(60.0));
    assert!(aberta.face_em([-20.0, 0.0]).is_some(), "a de dentro");
    assert!(aberta.face_em([25.0, 0.0]).is_some(), "a bolsa");
    // E com a parede EM CIMA da aresta, a de dentro SOBREVIVE (a bolsa tem area zero e some).
    let colada = rede(&base(20.0));
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
    let r = rede(&[(arco(60.0), false), (arco(-60.0), false)]);
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
        let r = rede(&quadro(topo));
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
        if rede(&quadro(topo)).face_em(fixa).is_none() {
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
    let r = rede(&[(l, true)]);
    let f = r.face_em([-30.0, 0.0]).expect("o braco vertical do L");
    let p = r.interior_point(&f).expect("o L tem miolo");
    assert!(
        r.face_em(p).is_some_and(|g| (g.area - f.area).abs() < 1.0),
        "o ponto interior caiu fora da propria face: {p:?}"
    );
}

/// ⭐⭐⭐ **UMA PONTA UM FIO FORA DA PAREDE AINDA FECHA A REGIÃO** — o report de 2026-09-02.
///
/// A fixtura é a foto do Enio: um rectângulo arredondado e uma curva que sai do lado direito e
/// volta a ele, fechando uma bolsa. ⚠️ **Medido antes:** a folga do toque era a **flecha do alvo**,
/// e ela é **zero numa recta** — a ponta tinha de encostar ao bit, e `0,05` fora já abria a bolsa.
///
/// ⚠️ **A folga é MINÚSCULA de propósito** (`1e-3` da diagonal — `0,4` neste desenho de 400): ela
/// perdoa o tremor da mão e o resíduo de vírgula flutuante, e **não fecha um vão que se veja**. A
/// segunda metade do gate é essa cerca. ⛔ A largura do traço foi tentada e **revertida** (§7).
#[test]
fn an_endpoint_a_hair_off_the_wall_still_closes_the_pocket() {
    let rr = ph2d_vec_scene::rounded_rect([-100.0, -100.0], [100.0, 100.0], 30.0);
    let curva = |ax: f64| {
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
    for ax in [100.0_f64, 100.05, 100.2] {
        let r = rede(&[(rr.verts.clone(), true), (curva(ax), false)]);
        let bolsa = r
            .face_em([150.0, 0.0])
            .unwrap_or_else(|| panic!("com a ponta a {:.2} da parede a bolsa abriu", ax - 100.0));
        assert!(
            (bolsa.area - 6513.0).abs() < 60.0,
            "ax={ax}: a bolsa tem de ser SO' a bolsa: {}",
            bolsa.area
        );
        assert!(
            r.face_em([0.0, 0.0])
                .is_some_and(|f| (f.area - 39224.0).abs() < 60.0),
            "ax={ax}: o rectangulo fundiu-se com a bolsa"
        );
    }
    // ⛔ A CERCA: um vão que se VÊ continua um vão, e a bolsa fica aberta.
    let r = rede(&[(rr.verts.clone(), true), (curva(102.0), false)]);
    assert!(
        r.face_em([150.0, 0.0]).is_none(),
        "a folga fechou um vao de 2 unidades — ela e' um perdao de fio, nao uma feature"
    );
}

/// ⛔⛔ **TRÊS LINHAS PELO MESMO PONTO mantêm os três cruzamentos.**
///
/// A fusão de travessias existe para colapsar *a mesma travessia vista por duas arestas vizinhas* —
/// **dentro do par que a produziu**. Comparada contra tudo o que já saiu, ela apagava o cruzamento
/// de um **segundo par no mesmo ponto**. ⚠️ Ficou escondida enquanto cada contorno perguntava numa
/// chamada própria (a lista nascia vazia de cada vez); a passagem única expô-la.
#[test]
fn three_concurrent_lines_keep_all_their_crossings() {
    let tres = vec![
        (vec![v(-50.0, 0.0), v(50.0, 0.0)], false),
        (vec![v(0.0, -50.0), v(0.0, 50.0)], false),
        (vec![v(-40.0, -40.0), v(40.0, 40.0)], false),
    ];
    let xs = ph2d_vec_scene::trim_tool::crossings_all(&tres, 140.0).expect("abaixo do tecto");
    for (i, f) in xs.iter().enumerate() {
        assert_eq!(
            f.len(),
            2,
            "a linha {i} tem de ser cortada pelas OUTRAS DUAS, e traz {f:?}"
        );
    }
    // E a rede parte as três em dois arcos cada.
    let r = rede(&tres);
    assert_eq!(r.arcos.len(), 6, "tres linhas concorrentes dao SEIS arcos");
    assert_eq!(r.nos.len(), 7, "um no' no centro e as seis pontas soltas");
}

/// ⛔⛔ **ACIMA DO TECTO a resposta é uma RECUSA, e não uma resposta ERRADA.**
///
/// ⚠️ **Medido:** a resposta antiga era devolver **zero cruzamentos**, e sem cruzamentos toda forma
/// volta a ser um anel inteiro — a lente entre dois círculos passava de `2 235` para `7 844` de
/// área **sem um aviso**. *Uma resposta errada em silêncio é pior que nenhuma resposta.*
#[test]
fn a_document_past_the_cap_is_refused_never_answered_wrong() {
    let poucos: Vec<(Vec<VecVertex>, bool)> = (0..8)
        .map(|i| {
            (
                ph2d_vec_scene::ellipse([f64::from(i) * 60.0, 0.0], 50.0, 50.0).verts,
                true,
            )
        })
        .collect();
    let r = rede(&poucos);
    assert!(!r.recusada);
    let lente = r
        .face_em([30.0, 0.0])
        .expect("a lente entre os dois primeiros");
    assert!((lente.area - 2235.0).abs() < 30.0, "area {}", lente.area);

    // 200 círculos = 12 800 arestas, acima do tecto de 12 288.
    let muitos: Vec<(Vec<VecVertex>, bool)> = (0..200)
        .map(|i| {
            (
                ph2d_vec_scene::ellipse([f64::from(i) * 60.0, 0.0], 50.0, 50.0).verts,
                true,
            )
        })
        .collect();
    let r = rede(&muitos);
    assert!(r.recusada, "o tecto nao foi reconhecido");
    assert!(r.arcos.is_empty(), "a rede recusada tem de sair VAZIA");
    assert!(
        r.face_em([30.0, 0.0]).is_none(),
        "acima do tecto o balde tem de RECUSAR, e nao devolver a forma inteira"
    );
}

/// ⭐⭐⭐ **O CASO QUE SÓ RECTAS PRODUZEM** — e que a fixtura curva não podia medir.
///
/// Quatro rectas a fechar um quadrado, com uma ponta a `0,2` da vizinha (o «T» quase-fechado). Aqui
/// **a flecha é zero dos dois lados**, então o piso do agrupamento não pode vir dela: se ele não
/// honrar a folga do TOQUE, a parede ganha o nó na projecção, a ponta fica noutro nó a `0,2` — e o
/// nó partido em dois é o mesmo que não haver toque nenhum.
///
/// ⚠️ **Foi uma mutação SOBREVIVENTE que pediu este gate** (`o-agrupamento-ignora-o-toque`): a
/// fixtura da bolsa tem curvas, e `2 ×` a flecha delas já cobria a folga por acidente.
#[test]
fn four_straight_walls_with_a_hair_of_a_gap_still_enclose_a_square() {
    // ⚠️ **O vão é para DENTRO**, e é essa a fixtura certa: com a ponta a passar da parede (para
    // fora) elas **CRUZAM-SE**, e o quadrado fecha pelo cruzamento — a 1.ª redacção deste gate media
    // isso e passava sem o piso. *Uma fixtura que fecha por outra razão aprova a lei que não mede.*
    let quadro = |g: f64| {
        vec![
            (vec![v(-50.0, -50.0), v(50.0, -50.0)], false),
            (vec![v(50.0, -50.0 + g), v(50.0, 50.0)], false),
            (vec![v(50.0, 50.0), v(-50.0, 50.0)], false),
            (vec![v(-50.0, 50.0), v(-50.0, -50.0)], false),
        ]
    };
    // Controle: encostadas, o miolo é 100×100.
    let r = rede(&quadro(0.0));
    assert!(
        r.face_em([0.0, 0.0])
            .is_some_and(|f| (f.area - 10_000.0).abs() < 1.0),
        "controle: o quadrado fechado"
    );
    // E com a ponta a `0,1` da parede vizinha (a folga é `0,14` neste desenho) ele continua fechado.
    let r = rede(&quadro(0.1));
    let miolo = r
        .face_em([0.0, 0.0])
        .expect("uma ponta a 0,1 da parede ainda fecha o quadrado");
    assert!(
        (miolo.area - 10_000.0).abs() < 30.0,
        "o miolo mede {}",
        miolo.area
    );
    // ⛔ E a CERCA: um vão de `2` unidades é um vão, e o quadrado abre.
    assert!(
        rede(&quadro(2.0)).face_em([0.0, 0.0]).is_none(),
        "a folga fechou um vao de 2 unidades"
    );
}

/// ⭐⭐⭐ **DUAS FACES QUE PARTILHAM UMA PAREDE DECLARAM-SE VIZINHAS, e o peso é o COMPRIMENTO dela.**
///
/// ⚠️⚠️ **Este gate existe para não deixar a adjacência por ARCO passar despercebida se partir.** A
/// irmã dela — a partilha por NÓ, com peso `0` — cobre o mesmo pedido do consumidor, então uma
/// adjacência por arco avariada cairia toda no nó e **nenhum gate do balde reprovaria**. *Uma
/// segunda rota para a mesma pergunta é uma rede de segurança que esconde a queda.*
#[test]
fn two_faces_that_share_a_wall_are_neighbours_by_its_length() {
    // Um quadrado de lado 20 cortado ao meio por uma linha horizontal: a parede mede 20.
    let linha = (vec![v(-20.0, 0.0), v(20.0, 0.0)], false);
    let r = rede(&[quadrado(20.0), linha]);
    let faces: Vec<Face> = r.faces().into_iter().filter(|f| f.area > 0.0).collect();
    assert_eq!(faces.len(), 2);

    let adj = r.adjacencias(&faces);

    assert_eq!(adj[0].len(), 1, "cada metade tem UMA vizinha: {adj:?}");
    assert_eq!(adj[1].len(), 1);
    assert_eq!(adj[0][0].0, 1, "e a vizinha da primeira e' a segunda");
    assert!(
        (adj[0][0].1 - 20.0).abs() < 1e-6,
        "o peso e' o COMPRIMENTO da parede partilhada, e nao a contagem: {}",
        adj[0][0].1
    );
}

/// ⛔⛔ **Os dois lóbulos de um contorno que se cruza NÃO fazem fronteira — tocam-se num PONTO.**
///
/// Medido no report de 2026-09-02: a espiga de um círculo dá 3 faces, e **as três declaram-se sem
/// vizinhas**. ⚠️ **Durante um dia isto foi ao contrário**: a partilha de nó entrava com peso `0`
/// para a tinta poder atravessar, e o report seguinte do Enio chamou ao resultado *"resíduo de
/// preenchimento"* — uma espiga de cor a sair do desenho. ⇒ *a tinta atravessa uma FRONTEIRA, nunca
/// um ponto*, que é a mesma lei de um balde de pixels.
#[test]
fn the_lobes_of_a_self_crossing_contour_meet_at_a_node_not_along_a_wall() {
    let base = ph2d_vec_scene::ellipse([0.0, 0.0], 100.0, 100.0);
    let mut verts = base.verts.clone();
    let topo = verts
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.anchor[1].total_cmp(&b.1.anchor[1]))
        .map(|(i, _)| i)
        .expect("a elipse tem vertices");
    let alvo = [0.0, -260.0];
    verts[topo].anchor = alvo;
    verts[topo].in_handle = alvo;
    verts[topo].out_handle = alvo;
    let r = rede(&[(verts, true)]);
    let faces: Vec<Face> = r.faces().into_iter().filter(|f| f.area > 0.0).collect();
    assert!(faces.len() >= 2, "a espiga tem de criar lobulos");

    let adj = r.adjacencias(&faces);

    assert!(
        adj.iter().all(std::vec::Vec::is_empty),
        "um lobulo nao faz fronteira com ninguem — a tinta nao pode atravessar para ele: {adj:?}"
    );
}
