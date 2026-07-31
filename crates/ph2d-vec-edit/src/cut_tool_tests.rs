//! Gates da TESOURA ([`super`]) — o 1º consumidor do corte.
//!
//! ⚠️ **As fixtures são grandes** (lados de 100 unidades) de propósito: o gesto compara distâncias
//! contra um `hit_r` de MUNDO, e numa forma de tamanho 1 todo ponto está dentro do raio — a fixture
//! não conteria a distinção entre *cortar no vértice* e *cortar no meio do segmento*, que é a única
//! decisão que este arquivo tem.

use crate::PenTool;
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecVertex, VertexKind};

const HIT: f64 = 6.0;

fn v(x: f64, y: f64) -> VecVertex {
    VecVertex {
        anchor: [x, y],
        in_handle: [x, y],
        out_handle: [x, y],
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    }
}

/// Um quadrado FECHADO de lado 100, com o canto inferior-esquerdo na origem.
fn square(scene: &mut VecScene) -> VecPathId {
    scene.push_path(VecPath {
        verts: vec![v(0.0, 0.0), v(100.0, 0.0), v(100.0, 100.0), v(0.0, 100.0)],
        closed: true,
        ..VecPath::default()
    })
}

fn line(scene: &mut VecScene, pts: &[[f64; 2]]) -> VecPathId {
    scene.push_path(VecPath {
        verts: pts.iter().map(|p| v(p[0], p[1])).collect(),
        closed: false,
        ..VecPath::default()
    })
}

#[test]
fn one_snip_opens_a_closed_shape_without_creating_a_second_object() {
    let mut scene = VecScene::default();
    let id = square(&mut scene);
    let mut pen = PenTool::new();

    // O meio da aresta de baixo.
    assert_eq!(pen.scissors_cut(&mut scene, [50.0, 0.0], HIT), Some(id));

    assert_eq!(scene.paths().len(), 1, "abrir não cria objeto");
    let p = &scene.paths()[0];
    assert!(!p.closed, "a forma abriu");
    assert_eq!(
        p.verts.len(),
        6,
        "4 originais + o que a tesoura inseriu + a costura duplicada"
    );
    assert_eq!(p.verts[0].anchor, [50.0, 0.0], "re-enraizado no corte");
    assert_eq!(p.verts[5].anchor, [50.0, 0.0], "a costura nas duas pontas");
}

/// A cena que o plano pede: **duas tesouradas numa forma fechada dão duas peças abertas.**
#[test]
fn two_snips_cut_a_closed_shape_into_two_open_pieces() {
    let mut scene = VecScene::default();
    square(&mut scene);
    let mut pen = PenTool::new();

    pen.scissors_cut(&mut scene, [50.0, 0.0], HIT).unwrap();
    pen.scissors_cut(&mut scene, [50.0, 100.0], HIT).unwrap();

    assert_eq!(scene.paths().len(), 2, "duas peças");
    for p in scene.paths() {
        assert!(!p.closed, "as duas ficam ABERTAS");
        assert!(p.verts.len() >= 3);
    }
    // As duas metades cobrem o quadrado inteiro: nenhuma aresta se perdeu.
    let total: usize = scene.paths().iter().map(|p| p.verts.len()).sum();
    assert_eq!(total, 8, "4 originais + 2 cortes, cada um duplicado");
}

/// **Clicar EM CIMA de um vértice corta NELE.** Sem esta metade, o clique inseria um segundo
/// vértice coincidente e deixava um segmento de comprimento zero — invisível no desenho e um
/// degrau em todo Simplify/Average/Delete seguinte.
#[test]
fn snipping_on_an_existing_anchor_does_not_insert_a_twin() {
    let mut scene = VecScene::default();
    let id = square(&mut scene);
    let mut pen = PenTool::new();

    // Em cima do canto (100, 0), dentro do raio de captura.
    assert_eq!(pen.scissors_cut(&mut scene, [98.0, 1.0], HIT), Some(id));

    let p = &scene.paths()[0];
    assert_eq!(p.verts.len(), 5, "4 + a costura — NENHUM vértice inserido");
    assert_eq!(p.verts[0].anchor, [100.0, 0.0], "cortou no canto");
    // E não há dois vértices no mesmo ponto no MEIO do caminho.
    for w in p.verts.windows(2) {
        assert_ne!(w[0].anchor, w[1].anchor, "segmento de comprimento zero");
    }
}

#[test]
fn snipping_the_middle_of_an_open_path_splits_it_in_two() {
    let mut scene = VecScene::default();
    let id = line(&mut scene, &[[0.0, 0.0], [100.0, 0.0]]);
    let mut pen = PenTool::new();

    assert_eq!(pen.scissors_cut(&mut scene, [50.0, 0.0], HIT), Some(id));

    assert_eq!(scene.paths().len(), 2);
    let xs: Vec<Vec<f64>> = scene
        .paths()
        .iter()
        .map(|p| p.verts.iter().map(|v| v.anchor[0]).collect())
        .collect();
    assert_eq!(xs[0], vec![0.0, 50.0]);
    assert_eq!(xs[1], vec![50.0, 100.0]);
}

/// A ponta de um caminho aberto não tem o que abrir — o Illustrator também recusa. A recusa tem de
/// ser LIMPA: a forma fica exatamente como estava, sem vértice inserido.
#[test]
fn snipping_the_end_of_an_open_path_is_refused_and_leaves_it_untouched() {
    let mut scene = VecScene::default();
    line(&mut scene, &[[0.0, 0.0], [100.0, 0.0]]);
    let mut pen = PenTool::new();

    assert_eq!(pen.scissors_cut(&mut scene, [1.0, 0.0], HIT), None);

    assert_eq!(scene.paths().len(), 1);
    assert_eq!(scene.paths()[0].verts.len(), 2, "nenhum vértice inserido");
}

#[test]
fn snipping_empty_canvas_does_nothing() {
    let mut scene = VecScene::default();
    square(&mut scene);
    let mut pen = PenTool::new();

    assert_eq!(pen.scissors_cut(&mut scene, [500.0, 500.0], HIT), None);
    assert_eq!(scene.paths().len(), 1);
    assert!(scene.paths()[0].closed, "nada foi aberto");
}

/// O gesto vale **sem pré-selecionar** — a tesoura pega o que está sob o cursor, como as
/// ferramentas de quina e a de largura.
#[test]
fn the_scissors_picks_the_path_under_the_cursor_without_a_prior_selection() {
    let mut scene = VecScene::default();
    let _left = line(&mut scene, &[[0.0, 0.0], [100.0, 0.0]]);
    let right = line(&mut scene, &[[0.0, 500.0], [100.0, 500.0]]);
    let mut pen = PenTool::new();
    assert_eq!(pen.selected(), None, "a premissa: nada selecionado");

    assert_eq!(
        pen.scissors_cut(&mut scene, [50.0, 500.0], HIT),
        Some(right)
    );
    assert_eq!(pen.selected(), Some(right), "e passa a ser o selecionado");
}

/// A seleção de nó não sobrevive a um corte: os índices planos a jusante andam.
#[test]
fn the_scissors_drops_the_vertex_selection_it_invalidates() {
    let mut scene = VecScene::default();
    let id = square(&mut scene);
    let mut pen = PenTool::new();
    pen.select(Some(id));
    pen.selected_verts = vec![3];

    pen.scissors_cut(&mut scene, [50.0, 0.0], HIT).unwrap();

    assert!(pen.selected_verts().is_empty());
}

/// **Cortar no segmento de FECHO** — o que vai do último vértice de volta ao primeiro. Ele só
/// existe num contorno fechado, e o "vértice seguinte" dele **dá a volta**: sem o wrap, o índice
/// pedido cai fora do contorno, a porta devolve `None` e a tesoura recusa em silêncio uma aresta
/// que está desenhada na tela.
///
/// ⚠️ Nenhuma outra fixture deste arquivo corta ali (todas caem na aresta de baixo, o segmento 0),
/// e por isso a mutação que tira o wrap sobrevivia a elas.
#[test]
fn snipping_the_closing_segment_works_because_the_next_vertex_wraps() {
    let mut scene = VecScene::default();
    let id = square(&mut scene);
    let mut pen = PenTool::new();

    // A aresta ESQUERDA (x = 0): o segmento que liga o 4º vértice de volta ao 1º.
    assert_eq!(
        pen.scissors_cut(&mut scene, [0.0, 50.0], HIT),
        Some(id),
        "a tesoura recusou a aresta de fecho -- ela esta' desenhada na tela"
    );

    let p = &scene.paths()[0];
    assert!(!p.closed);
    assert_eq!(p.verts[0].anchor, [0.0, 50.0], "cortou onde se clicou");
}

/// E cortar em cima do vértice que FECHA o contorno (o último) também tem de cair nele, não num
/// gêmeo inserido — é a mesma pergunta do wrap, do outro lado.
#[test]
fn snipping_on_the_last_anchor_of_a_closed_contour_cuts_at_it() {
    let mut scene = VecScene::default();
    let id = square(&mut scene);
    let mut pen = PenTool::new();

    assert_eq!(pen.scissors_cut(&mut scene, [1.0, 98.0], HIT), Some(id));

    let p = &scene.paths()[0];
    assert_eq!(p.verts.len(), 5, "4 + a costura — nenhum inserido");
    assert_eq!(p.verts[0].anchor, [0.0, 100.0]);
}

// ── A FACA ──────────────────────────────────────────────────────────────────

/// Um quadrado FECHADO de arestas RETAS — a faca é sobre ONDE ela cruza, e com handles as
/// "arestas" seriam curvas e nenhum cruzamento se poderia escrever à mão.
fn sharp_square(scene: &mut VecScene, x0: f64) -> VecPathId {
    scene.push_path(VecPath {
        verts: vec![
            v(x0, 0.0),
            v(x0 + 100.0, 0.0),
            v(x0 + 100.0, 100.0),
            v(x0, 100.0),
        ],
        closed: true,
        ..VecPath::default()
    })
}

/// **A lâmina a atravessar TRÊS formas de uma vez** — a cena que o plano pede. Cada uma tem de
/// virar duas peças abertas.
#[test]
fn one_blade_cuts_every_shape_it_crosses() {
    let mut scene = VecScene::default();
    for k in 0..3 {
        sharp_square(&mut scene, k as f64 * 200.0);
    }
    let mut pen = PenTool::new();

    let cuts = pen.knife_cut(&mut scene, [-50.0, 50.0], [650.0, 50.0], HIT);

    assert_eq!(cuts, 6, "duas tesouradas por forma: {cuts}");
    assert_eq!(scene.paths().len(), 6, "três formas viraram seis peças");
    for p in scene.paths() {
        assert!(!p.closed, "as peças ficam ABERTAS");
        assert!(p.verts.len() >= 3);
    }
}

/// **O que a lâmina não alcança não é tocado.** Uma faca que corta o que está para lá da ponta é a
/// ferramenta a fazer o que ninguém desenhou.
#[test]
fn the_knife_leaves_alone_what_the_blade_does_not_reach() {
    let mut scene = VecScene::default();
    let hit = sharp_square(&mut scene, 0.0);
    let miss = sharp_square(&mut scene, 400.0);
    let mut pen = PenTool::new();

    // A lâmina começa fora do 1º e PARA muito antes do 2º.
    let cuts = pen.knife_cut(&mut scene, [-50.0, 50.0], [150.0, 50.0], HIT);

    assert_eq!(cuts, 2, "só a 1ª forma");
    assert!(
        scene.paths().iter().find(|p| p.id == miss).unwrap().closed,
        "a forma distante continua FECHADA"
    );
    assert!(scene.paths().iter().any(|p| p.id == hit && !p.closed));
}

/// **A costura que o corte acaba de criar assenta EXACTAMENTE sobre a lâmina** — se ela pudesse ser
/// reencontrada, a faca cortaria o mesmo ponto para sempre. O conjunto de pontos já cortados é o
/// que fecha isso, e este gate é o que prova que ele fecha (sem ele o teto de segurança disfarçaria
/// o laço num número grande de cortes).
#[test]
fn the_knife_does_not_re_cut_the_seam_it_just_made() {
    let mut scene = VecScene::default();
    sharp_square(&mut scene, 0.0);
    let mut pen = PenTool::new();

    let cuts = pen.knife_cut(&mut scene, [-50.0, 50.0], [150.0, 50.0], HIT);

    assert_eq!(cuts, 2, "exactamente dois -- não 3, não 256");
    let total: usize = scene.paths().iter().map(|p| p.verts.len()).sum();
    assert_eq!(total, 8, "4 originais + 2 cortes, cada um duplicado");
}

/// Uma lâmina que não toca nada é um no-op limpo — e um gesto de arrasto acidental no vazio é o
/// caso mais comum de todos.
#[test]
fn a_blade_that_touches_nothing_changes_nothing() {
    let mut scene = VecScene::default();
    sharp_square(&mut scene, 0.0);
    let mut pen = PenTool::new();
    let before = scene.paths()[0].verts.len();

    assert_eq!(
        pen.knife_cut(&mut scene, [0.0, 500.0], [100.0, 500.0], HIT),
        0
    );

    assert_eq!(scene.paths().len(), 1);
    assert_eq!(scene.paths()[0].verts.len(), before);
    assert!(scene.paths()[0].closed);
}

/// Uma faca sobre um caminho ABERTO parte-o em tantas peças quantas as travessias — nenhuma
/// re-fecha, e a contagem é a que se conta a olho.
#[test]
fn the_knife_slices_an_open_path_into_one_more_piece_than_crossings() {
    let mut scene = VecScene::default();
    // Um zigue-zague que atravessa a linha y=50 duas vezes.
    scene.push_path(VecPath {
        verts: vec![v(0.0, 0.0), v(50.0, 100.0), v(100.0, 0.0)],
        closed: false,
        ..VecPath::default()
    });
    let mut pen = PenTool::new();

    let cuts = pen.knife_cut(&mut scene, [-10.0, 50.0], [110.0, 50.0], HIT);

    assert_eq!(cuts, 2);
    assert_eq!(scene.paths().len(), 3, "2 travessias = 3 pedaços");
}

/// **A lâmina é MUNDO; a curva é LOCAL** (ADR-0111), e converter é o que faz a faca cortar onde o
/// artista a desenhou.
///
/// ⚠️ **Nenhuma outra fixture deste arquivo publica um afim**, e sem afim `Xform::IDENTITY` faz a
/// conversão ser um no-op — *converter* e *não converter* eram indistinguíveis, e a mutação que
/// apaga a conversão sobrevivia a todas elas. Aqui o quadrado vive DESLOCADO no mundo.
#[test]
fn the_blade_is_converted_into_each_paths_own_space() {
    let mut scene = VecScene::default();
    let id = sharp_square(&mut scene, 0.0); // local: (0,0)-(100,100)
    let mut pen = PenTool::new();
    // O mesmo quadrado, mas 1000 unidades à direita no MUNDO.
    let mut xf = ph2d_vec_scene::VecXforms::new();
    xf.insert(id, ph2d_vec_scene::Xform([1.0, 0.0, 0.0, 1.0, 1000.0, 0.0]));
    pen.set_xforms(xf);

    // Uma lâmina no MUNDO, atravessando onde a forma de facto está.
    let cuts = pen.knife_cut(&mut scene, [950.0, 50.0], [1150.0, 50.0], HIT);
    assert_eq!(cuts, 2, "a faca não achou a forma onde ela está desenhada");
    assert_eq!(scene.paths().len(), 2);

    // E uma lâmina sobre as coordenadas LOCAIS (onde a forma NÃO está) não pode cortar nada.
    let mut scene2 = VecScene::default();
    let id2 = sharp_square(&mut scene2, 0.0);
    let mut pen2 = PenTool::new();
    let mut xf2 = ph2d_vec_scene::VecXforms::new();
    xf2.insert(
        id2,
        ph2d_vec_scene::Xform([1.0, 0.0, 0.0, 1.0, 1000.0, 0.0]),
    );
    pen2.set_xforms(xf2);
    assert_eq!(
        pen2.knife_cut(&mut scene2, [-50.0, 50.0], [150.0, 50.0], HIT),
        0,
        "a faca cortou nas coordenadas LOCAIS -- a forma não está lá"
    );
}

/// **A faca segue a CADEIA de metades novas até ao fim.** Uma lâmina que atravessa a mesma forma
/// aberta TRÊS vezes tem de a partir em quatro: cada corte devolve uma metade nova que ainda
/// carrega os cruzamentos seguintes, e é ela que volta à fila.
///
/// ⚠️ A metade que fica com o id ORIGINAL nunca retém um cruzamento, e isso é por construção, não
/// por sorte: o corte é sempre tomado no PRIMEIRO cruzamento que resta, então tudo o que sobra
/// para trás está livre — e as duas pontas dela são costuras, que o `blade_crossings` exclui.
#[test]
fn the_knife_follows_the_chain_of_new_halves_to_the_end() {
    let mut scene = VecScene::default();
    // Um "E" deitado: três pernas verticais (x=0, x=100, x=200), e a lâmina horizontal em
    // y=50 atravessa cada uma delas — três travessias, quatro pedaços.
    scene.push_path(VecPath {
        verts: vec![
            v(0.0, 0.0),
            v(0.0, 100.0),
            v(100.0, 100.0),
            v(100.0, 0.0),
            v(200.0, 0.0),
            v(200.0, 100.0),
        ],
        closed: false,
        ..VecPath::default()
    });
    let mut pen = PenTool::new();

    let cuts = pen.knife_cut(&mut scene, [-10.0, 50.0], [210.0, 50.0], HIT);

    // ⚠️ TRÊS travessias, não quatro: as pernas verticais do "E" são três e as horizontais correm
    // em y=0 e y=100, longe da lâmina. Contar mal a fixture teria feito o gate falhar sobre
    // produto correto — e foi o que aconteceu na 1ª escrita dele.
    assert_eq!(cuts, 3, "três travessias, três cortes");
    assert_eq!(scene.paths().len(), 4, "3 cortes = 4 pedaços");
}

/// A faca larga a seleção de nó que acaba de invalidar — pelas mesmas duas razões da tesoura: os
/// índices planos andam, e as peças novas nem são o mesmo objeto.
#[test]
fn the_knife_drops_the_vertex_selection_it_invalidates() {
    let mut scene = VecScene::default();
    let id = sharp_square(&mut scene, 0.0);
    let mut pen = PenTool::new();
    pen.select(Some(id));
    pen.selected_verts = vec![2, 3];

    assert_eq!(
        pen.knife_cut(&mut scene, [-50.0, 50.0], [150.0, 50.0], HIT),
        2
    );
    assert!(pen.selected_verts().is_empty());

    // ⚠️ E uma faca que NÃO corta nada não pode mexer na seleção: o artista arrastou no vazio.
    pen.selected_verts = vec![0];
    assert_eq!(
        pen.knife_cut(&mut scene, [0.0, 900.0], [10.0, 900.0], HIT),
        0
    );
    assert_eq!(pen.selected_verts(), &[0], "um no-op não é um gesto");
}

/// **A faca não corta o que a árvore ESCONDE nem o que ela TRAVA.** É a mesma lei de todo hit-test
/// desta crate (ADR-0110), e sem ela uma lâmina destruiria em silêncio uma camada travada que o
/// artista travou exactamente para não lhe tocar.
///
/// ⚠️ Nenhuma outra fixture deste arquivo publica um `VecViewState`, e por isso a mutação que
/// remove o filtro sobrevivia a todas elas.
#[test]
fn the_knife_spares_hidden_and_locked_paths() {
    let mut scene = VecScene::default();
    let visible = sharp_square(&mut scene, 0.0);
    let hidden = sharp_square(&mut scene, 200.0);
    let locked = sharp_square(&mut scene, 400.0);
    let mut pen = PenTool::new();
    pen.set_view(ph2d_vec_scene::VecViewState {
        hidden: vec![hidden],
        locked: vec![locked],
    });

    let cuts = pen.knife_cut(&mut scene, [-50.0, 50.0], [650.0, 50.0], HIT);

    assert_eq!(cuts, 2, "só a forma VISÍVEL e destravada");
    for (id, what) in [(hidden, "escondida"), (locked, "travada")] {
        assert!(
            scene.paths().iter().find(|p| p.id == id).unwrap().closed,
            "a forma {what} foi cortada"
        );
    }
    assert!(scene.paths().iter().any(|p| p.id == visible && !p.closed));
}
