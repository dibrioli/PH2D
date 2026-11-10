//! ⭐ **O Pivot da Animated Sprite no Align** (GEF 29, pedido do dono 09/11): a grade de 25 pontos conta sobre o
//! desenho de TODOS os quadros de TODAS as animações; o «By hand» arrasta a cruz; cada escolha é UMA edição (um
//! Ctrl+Z) e o desenho de cada folha fica no mundo.

use super::super::{Mesa, centre};
use super::desalinhada;
use ph2d_app_sprite_anim::editor::align_pivot::{SIDE, grid_point};
use ph2d_ecs::{Entity, Transform};
use ph2d_editor_core::doc_surface::DocumentSurface;
use ph2d_editor_core::{AnimFieldEdit, ImageUseEdit, row_cells};
use ph2d_host::Modifiers;

/// O centro no mundo do quad de cada folha (sem rotação nem escala: a posição mais a âncora).
fn quads_no_mundo(m: &Mesa) -> Vec<[f32; 2]> {
    let w = m.sim.world();
    let e = Entity::from_bits(m.e);
    let t = w.get::<Transform>(e).expect("o herói").translation;
    let ppm = m.hero.project.pixels_per_meter;
    (0..2_u8)
        .filter_map(|k| ph2d_render::sheet_geometry(w, e, k, 0))
        .map(|(s, _)| {
            let a = s.resolve_anchor(ppm);
            [t.x + a[0], t.y + a[1]]
        })
        .collect()
}

/// O ecrã do ponto `q` da grade da moldura.
fn na_grade(m: &mut Mesa, q: [u8; 2]) -> (f32, f32) {
    let g = m.doc().layout.align.pivot_grid;
    assert!(g.w > 0.0 && g.h > 0.0, "a grade do Pivot na moldura");
    let passo = g.w / f32::from(SIDE);
    (
        g.x + (f32::from(q[0]) + 0.5) * passo, // LITERAL-PX-OK: o meio da casa
        g.y + (f32::from(q[1]) + 0.5) * passo, // LITERAL-PX-OK: o meio da casa
    )
}

/// ⭐ Os 25 pontos contam sobre todos os quadros de todas as animações: a base da caixa é o pé mais baixo de TODAS.
/// Um clique em «embaixo, meio» é uma edição só; depois dela o Pivot está nesse ponto, as duas folhas ficam no mundo,
/// e trocar de animação não muda o ponto.
/// *Mutações: a caixa só da animação aberta ⇒ a base sobe · o ponto contado de baixo para cima ⇒ cai em cima ·
/// `move_origin` só na folha visível ⇒ a folha 0 sai do lugar · a prévia sem seguir a geometria ⇒ o ponto não acende.*
#[test]
fn os_25_pontos_contam_sobre_todos_os_quadros_e_o_desenho_fica() {
    let mut m = Mesa::new();
    desalinhada(&mut m);
    m.align_on();
    m.paint();
    let caixa = m
        .doc()
        .all_frames_box()
        .expect("o desenho de todos os quadros");
    let info = m.doc().snap.info.clone().expect("a biblioteca");
    assert!(info.rows.len() >= 2, "mais de uma animação na fixtura");
    let mut mais_baixo = f32::INFINITY;
    for row in &info.rows {
        for f in row_cells(row, &info.sheets) {
            let Some(p) = m
                .doc()
                .align_point(f, ph2d_app_sprite_anim::editor::align::AlignBy::Feet)
            else {
                continue;
            };
            assert!(
                p[0] >= caixa.0[0] - 1e-5 && p[0] <= caixa.1[0] + 1e-5,
                "{f:?} dentro"
            );
            mais_baixo = mais_baixo.min(p[1]);
        }
    }
    assert!(
        (mais_baixo - caixa.0[1]).abs() < 1e-5,
        "a base é o pé mais baixo de todas"
    );
    let antes = quads_no_mundo(&m);
    assert_eq!(antes.len(), 2, "as duas folhas");
    let alvo = grid_point(caixa, [2, 4]);
    let at = na_grade(&mut m, [2, 4]);
    m.click(at);
    assert_eq!(
        m.sheet_edits(),
        [ImageUseEdit::MovePivot(alvo)],
        "uma edição: um Ctrl+Z"
    );
    assert_eq!(m.doc().pivot_on_grid(), Some([2, 4]), "a prévia segue já");
    m.frame();
    m.paint();
    assert_eq!(m.doc().pivot_on_grid(), Some([2, 4]), "o mundo concorda");
    for (a, d) in antes.iter().zip(quads_no_mundo(&m)) {
        assert!(
            (a[0] - d[0]).abs() < 1e-5 && (a[1] - d[1]).abs() < 1e-5,
            "{a:?} → {d:?}"
        );
    }
    let outra = (m.doc().selected + 1) % info.rows.len();
    m.doc().selected = outra;
    m.paint();
    assert_eq!(
        m.doc().pivot_on_grid(),
        Some([2, 4]),
        "noutra animação, o mesmo ponto"
    );
}

/// O «By hand»: o arrasto na prévia move a CRUZ (a folha não anda) e só o largar pede a edição; perto de um dos 25
/// pontos encaixa nele; longe, cai num pixel inteiro do desenho. Desligado, o arrasto volta a mover a folha.
/// *Mutações: a edição a cada movimento ⇒ várias, vários Ctrl+Z · sem encaixe ⇒ o ponto não acende · sem o pixel
/// inteiro ⇒ o resto não é múltiplo.*
#[test]
fn by_hand_arrasta_a_cruz_e_encaixa_nos_pontos() {
    let mut m = Mesa::new();
    desalinhada(&mut m);
    m.align_on();
    m.paint();
    let b = m.doc().layout.align.by_hand;
    assert!(b.w > 0.0, "o botão By hand na moldura");
    m.click(centre(b));
    assert!(m.doc().align.pivot.by_hand);
    m.paint();
    let caixa = m.doc().all_frames_box().unwrap();
    let v = m.doc().align.view.expect("a prévia");
    let cruz = m.doc().align.painted.pivot.expect("a cruz");
    let destino = v.to_screen(grid_point(caixa, [4, 0]));
    let p = ph2d_host::PointerButton::Primary;
    m.pointer_with(ph2d_host::PointerKind::Down, p, cruz, Modifiers::default());
    m.pointer_with(
        ph2d_host::PointerKind::Move,
        p,
        destino,
        Modifiers::default(),
    );
    assert!(m.sheet_edits().is_empty(), "nada antes de largar");
    m.paint();
    let pintada = m.doc().align.painted.pivot.unwrap();
    assert!(
        (pintada.0 - destino.0).abs() < 0.5 && (pintada.1 - destino.1).abs() < 0.5,
        "a cruz segue o rato"
    );
    m.pointer_with(
        ph2d_host::PointerKind::Up,
        p,
        (destino.0 + 2.0, destino.1 - 1.0),
        Modifiers::default(),
    );
    assert_eq!(
        m.sheet_edits(),
        [ImageUseEdit::MovePivot(grid_point(caixa, [4, 0]))],
        "perto do ponto, encaixa; uma edição"
    );
    m.frame();
    m.paint();
    assert_eq!(m.doc().pivot_on_grid(), Some([4, 0]));
    // Longe dos pontos: um pixel inteiro do desenho.
    let caixa = m.doc().all_frames_box().unwrap();
    let v = m.doc().align.view.unwrap();
    let folha = m.doc().align_current().unwrap().1.sheet;
    let mpp = m.doc().meters_per_px(folha).unwrap();
    let meio = |a: f32, b: f32| a + (b - a) * 0.37; // LITERAL-PX-OK: um ponto qualquer entre os da grade
    let solto = v.to_screen([meio(caixa.0[0], caixa.1[0]), meio(caixa.0[1], caixa.1[1])]);
    m.drag(solto, solto, Modifiers::default());
    let [ImageUseEdit::MovePivot(q)] = m.sheet_edits()[..] else {
        panic!("uma edição: {:?}", m.sheet_edits());
    };
    for (qa, la) in q.iter().zip(caixa.0) {
        let n = (qa - la) / mpp;
        assert!((n - n.round()).abs() < 1e-3, "pixel inteiro: {n}");
    }
    m.frame();
    m.paint();
    assert_eq!(m.doc().pivot_on_grid(), None, "fora dos 25");
    // Desligado, o arrasto volta a mover a folha.
    let b = m.doc().layout.align.by_hand;
    m.click(centre(b));
    let pv = m.doc().layout.align.preview;
    m.drag(
        centre(pv),
        (centre(pv).0 + 30.0, centre(pv).1),
        Modifiers::default(),
    );
    assert!(
        m.sheet_edits()
            .iter()
            .any(|e| matches!(e, ImageUseEdit::MoveSheet(_)))
            && !m
                .sheet_edits()
                .iter()
                .any(|e| matches!(e, ImageUseEdit::MovePivot(_))),
        "{:?}",
        m.sheet_edits()
    );
}

/// O campo Pivot do Inspector numa Animated Sprite abre o editor com o Align ligado; o «Edit animations» de sempre
/// abre-o sem.
/// *Mutação: o pedido do Align esquecido ⇒ abre desligado.*
#[test]
fn o_campo_pivot_abre_o_editor_com_o_align_ligado() {
    let mut m = Mesa::new();
    assert!(!m.doc().align.on);
    let e = m.e;
    m.docs().close(e);
    assert!(ph2d_app_sprite_anim::editor::request_open(
        &mut m.sim,
        e,
        &AnimFieldEdit::OpenEditorAlign
    ));
    m.frame();
    assert!(m.doc().align.on, "aberto com o Align");
}

/// ⭐ **Follow** (dono 09/11): com ele ligado, a folha redimensionada pela quina e arrastada leva o Pivot junto — ele volta ao
/// mesmo lugar da caixa de todos os quadros, no mesmo quadro do gesto (um Ctrl+Z); desligado, o Pivot fica onde
/// estava e sai do ponto.
/// *Mutações: o Follow a ler a fração a meio do gesto ⇒ fica no lugar novo · sem o Follow na família ⇒ sai do ponto ·
/// o Follow vivo também na escala ⇒ a pega foge do rato e a escala alcançada não é a pedida.*
#[test]
fn com_o_follow_o_pivot_acompanha_o_redimensionamento() {
    for follow in [false, true] {
        let mut m = Mesa::new();
        desalinhada(&mut m);
        m.align_on();
        m.paint();
        let at = na_grade(&mut m, [4, 0]);
        m.click(at);
        m.frame();
        m.paint();
        assert_eq!(m.doc().pivot_on_grid(), Some([4, 0]));
        if follow {
            let b = m.doc().layout.align.pivot_follow;
            assert!(b.w > 0.0, "o botão Follow na moldura");
            m.click(centre(b));
            assert!(m.doc().align.pivot.follow);
        }
        m.frame();
        m.paint();
        // Redimensiona pela quina e depois arrasta a folha para longe: a caixa de todos os quadros muda.
        let p = m.doc().align.painted;
        let h = centre(p.handles[2].expect("a pega de baixo à esquerda"));
        let (ox, oy) = p.pivot.unwrap();
        let perto = (ox + (h.0 - ox) * 0.6, oy + (h.1 - oy) * 0.6);
        // Em quadros distintos, como a mão: na escala o Follow espera pelo largar; no arrasto da folha age a meio.
        let b = ph2d_host::PointerButton::Primary;
        m.pointer_with(ph2d_host::PointerKind::Down, b, h, Modifiers::default());
        for i in 1..=3 {
            let t = i as f32 / 3.0; // LITERAL-PX-OK: passos do arrasto
            let at = (h.0 + (perto.0 - h.0) * t, h.1 + (perto.1 - h.1) * t);
            m.pointer_with(ph2d_host::PointerKind::Move, b, at, Modifiers::default());
            m.frame();
            m.paint();
            // A pega segue o rato: o Follow da escala espera pelo fim do gesto (audit 09/11: segui-lo a meio mudava o
            // Pivot contra o qual a escala se mede, e a pega fugia do rato).
            let pega = centre(m.doc().align.painted.handles[2].expect("a pega"));
            assert!(
                (pega.0 - at.0).abs() < 1.0 && (pega.1 - at.1).abs() < 1.0,
                "follow={follow}, passo {i}: a pega em {pega:?}, o rato em {at:?}"
            );
        }
        let tamanho = m
            .sim
            .world()
            .get::<ph2d_ecs::SpriteSheets>(Entity::from_bits(m.e))
            .and_then(|s| s.sheet(1))
            .expect("a folha torta")
            .size;
        assert!(
            (tamanho[0] - 0.6).abs() < 1e-3 && (tamanho[1] - 0.6).abs() < 1e-3, // LITERAL-PX-OK: a razão pedida
            "follow={follow}: a escala pedida é a alcançada, {tamanho:?}"
        );
        m.pointer_with(ph2d_host::PointerKind::Up, b, perto, Modifiers::default());
        m.frame();
        m.paint();
        if follow {
            assert_eq!(m.doc().pivot_on_grid(), Some([4, 0]), "depois da escala");
        }
        let pv = m.doc().layout.align.preview;
        let c = centre(pv);
        // Como na mão do dono: o movimento num quadro da tela, o soltar noutro, sem movimento novo.
        let longe = (c.0 + pv.w * 0.3, c.1);
        let b = ph2d_host::PointerButton::Primary;
        m.pointer_with(ph2d_host::PointerKind::Down, b, c, Modifiers::default());
        m.pointer_with(ph2d_host::PointerKind::Move, b, longe, Modifiers::default());
        m.frame();
        m.paint();
        if follow {
            assert_eq!(
                m.doc().pivot_on_grid(),
                Some([4, 0]),
                "em tempo real, a meio do arrasto"
            );
        }
        m.pointer_with(ph2d_host::PointerKind::Up, b, longe, Modifiers::default());
        assert!(
            !m.sheet_edits()
                .iter()
                .any(|e| matches!(e, ImageUseEdit::MovePivot(_))),
            "o Follow não é um gesto à parte"
        );
        m.frame();
        m.paint();
        let ponto = m.doc().pivot_on_grid();
        if follow {
            assert_eq!(ponto, Some([4, 0]), "o Pivot acompanha");
        } else {
            assert_ne!(ponto, Some([4, 0]), "sem o Follow ele fica onde estava");
        }
    }
}

/// O lado da célula da folha torta (`desalinhada`: 64 × 32 em 2 × 1) e os retângulos opacos de cada célula,
/// `(x0, x1, y0, y1)` em píxeis desde o canto de cima.
const CELULA_PX: f32 = 32.0; // LITERAL-PX-OK: a fixtura
const OPACOS: [(f32, f32, f32, f32); 2] = [(4.0, 12.0, 20.0, 30.0), (8.0, 20.0, 10.0, 24.0)]; // LITERAL-PX-OK: a fixtura

/// ⭐ **Com o flip, o ponto de baixo à esquerda é o do desenho que o jogo mostra** (audit 09/11): o jogo vira os
/// píxeis dentro do quad (`ponto_local`, «a caixa vira com a imagem»); a caixa dos 25 pontos vira com eles.
/// *Mutação: a caixa sem o flip (`drawing_box` com a fração crua) ⇒ o Pivot cai no canto do desenho por virar.*
#[test]
fn com_flip_os_25_pontos_contam_sobre_o_desenho_virado() {
    for flip in [[true, false], [false, true], [true, true]] {
        let mut m = Mesa::new();
        desalinhada(&mut m);
        let e = Entity::from_bits(m.e);
        {
            let w = m.sim.world_mut();
            // Todas as animações na folha torta: a caixa de todos os quadros é só dela.
            let mut frames = w
                .get_mut::<ph2d_ecs::SpriteAnimFrames>(e)
                .expect("os quadros");
            frames.set(
                "run",
                vec![
                    ph2d_ecs::FrameRef::Cell { sheet: 1, cell: 0 },
                    ph2d_ecs::FrameRef::Cell { sheet: 1, cell: 1 },
                ],
            );
            let mut s = w.get_mut::<ph2d_render::Sprite>(e).expect("o herói");
            [s.flip_x, s.flip_y] = flip;
        }
        m.frame();
        m.align_on();
        m.paint();
        let q = quads_no_mundo(&m)[1];
        let vira = |on: bool, a: f32, b: f32| {
            if on {
                (CELULA_PX - b, CELULA_PX - a)
            } else {
                (a, b)
            }
        };
        let esquerda = OPACOS
            .iter()
            .map(|r| vira(flip[0], r.0, r.1).0)
            .fold(f32::INFINITY, f32::min);
        let base = OPACOS
            .iter()
            .map(|r| vira(flip[1], r.2, r.3).1)
            .fold(f32::NEG_INFINITY, f32::max);
        let alvo = [
            q[0] - 0.5 + esquerda / CELULA_PX, // LITERAL-PX-OK: meio quad de 1 m
            q[1] + 0.5 - base / CELULA_PX,     // LITERAL-PX-OK: meio quad de 1 m
        ];
        let at = na_grade(&mut m, [0, 4]);
        m.click(at);
        m.frame();
        m.paint();
        let t = m.sim.world().get::<Transform>(e).unwrap().translation;
        assert!(
            (t.x - alvo[0]).abs() < 1e-4 && (t.y - alvo[1]).abs() < 1e-4,
            "{flip:?}: o Pivot em {t:?}, o canto do desenho virado em {alvo:?}"
        );
        assert_eq!(m.doc().pivot_on_grid(), Some([0, 4]), "{flip:?}");
    }
}

/// ⭐ **Um passo que o mundo ignora também não anda a prévia** (audit 09/11): o `move_origin` ignora menos de 1 mm;
/// clicar o ponto já aceso, ou pedir um passo desses, não pede nada e a prévia fica.
/// *Mutação: o limiar do `move_pivot` de volta a `p == [0, 0]` ⇒ a edição sai e a prévia anda.*
#[test]
fn um_passo_que_o_mundo_ignora_nao_mexe_na_previa() {
    let mut m = Mesa::new();
    desalinhada(&mut m);
    m.align_on();
    m.paint();
    let at = na_grade(&mut m, [4, 0]);
    m.click(at);
    m.frame();
    m.paint();
    assert_eq!(m.doc().pivot_on_grid(), Some([4, 0]));
    let v = m.doc().align.view;
    m.click(at);
    assert!(!m.doc().move_pivot([5e-4, -5e-4]), "abaixo do mesmo ponto");
    assert!(m.sheet_edits().is_empty(), "{:?}", m.sheet_edits());
    assert_eq!(m.doc().align.view, v, "a prévia não anda");
}

/// ⭐ **O Follow guarda o PONTO, não a medida** (sobrevivente da mutação 09/11): o Pivot a menos de meio pixel do
/// ponto (aceso, mas torto) e a caixa a crescer muito com o arrasto da folha: o ponto continua aceso.
/// *Mutação: guardar a fração medida (o braço `None` do `match self.pivot_on_grid()`) ⇒ o erro cresce e apaga.*
#[test]
fn o_follow_guarda_o_ponto_exato_e_nao_a_medida_torta() {
    let mut m = Mesa::new();
    desalinhada(&mut m);
    m.align_on();
    m.paint();
    let at = na_grade(&mut m, [4, 0]);
    m.click(at);
    m.frame();
    m.paint();
    let mpp = m.doc().meters_per_px(1).expect("a folha torta");
    let torto = mpp * 0.45; // LITERAL-PX-OK: dentro do meio pixel
    assert!(torto >= 1e-3, "o mundo aplica-o");
    assert!(m.doc().move_pivot([torto, -torto]));
    m.frame();
    m.paint();
    assert_eq!(m.doc().pivot_on_grid(), Some([4, 0]), "torto, mas aceso");
    let b = m.doc().layout.align.pivot_follow;
    m.click(centre(b));
    m.frame();
    m.paint();
    let pv = m.doc().layout.align.preview;
    let c = centre(pv);
    let caixa = m.doc().all_frames_box().unwrap();
    m.drag(
        c,
        (c.0 - pv.w * 0.45, c.1 + pv.h * 0.4),
        Modifiers::default(),
    ); // LITERAL-PX-OK: para longe
    m.frame();
    m.paint();
    let depois = m.doc().all_frames_box().unwrap();
    let largura = |c: ([f32; 2], [f32; 2])| (c.1[0] - c.0[0]).min(c.1[1] - c.0[1]);
    assert!(
        largura(depois) > 1.5 * largura(caixa), // LITERAL-PX-OK: a caixa cresce
        "{caixa:?} → {depois:?}"
    );
    assert_eq!(
        m.doc().pivot_on_grid(),
        Some([4, 0]),
        "o ponto continua aceso"
    );
}

/// ⭐ **A ferramenta Pivot do canvas recusa numa Animated Sprite** e diz onde ele mora, num aviso só; numa sprite
/// parada age como sempre (sem aviso).
/// *Mutação: `pivot_tool_refused` a devolver `false` ⇒ vermelho.*
#[test]
fn a_ferramenta_pivot_recusa_na_animada_e_age_na_parada() {
    let mut m = Mesa::new();
    let mut toasts = ph2d_editor_core::ToastQueue::default();
    let e = Entity::from_bits(m.e);
    assert!(ph2d_app_sprite_anim::editor::pivot_tool_refused(
        m.sim.world(),
        e,
        &mut toasts
    ));
    let msgs: Vec<_> = toasts.iter().map(|t| t.message.clone()).collect();
    assert_eq!(
        msgs,
        [ph2d_i18n::tr("anim_editor.pivot_in_editor").to_string()]
    );
    let parada = m
        .sim
        .world_mut()
        .spawn((
            Transform::default(),
            ph2d_render::Sprite::atlas(ph2d_render::HERO_SHEET_KEY, [1.0, 1.0], [1.0; 4]),
        ))
        .id();
    let mut toasts = ph2d_editor_core::ToastQueue::default();
    assert!(!ph2d_app_sprite_anim::editor::pivot_tool_refused(
        m.sim.world(),
        parada,
        &mut toasts
    ));
    assert!(toasts.is_empty());
}

/// ⭐ **A quina com o Follow ligado: a pega segue o rato e o Pivot volta ao ponto ao largar** (audit 09/11). A escala
/// mede-se contra o Pivot no ecrã e faz-se em volta dele; com uma segunda folha (que a quina não escala) a caixa de
/// todos os quadros muda de forma, e um Follow a meio do gesto mudava o Pivot debaixo da escala: a pega fugia do
/// rato. O Follow da escala espera pelo largar, no mesmo quadro do último passo.
/// *Mutação: o Follow vivo também na escala (`live = true`) ⇒ a escala alcançada não é a pedida.*
#[test]
fn a_quina_com_follow_segue_o_rato_e_o_pivot_volta_ao_ponto_ao_largar() {
    let mut m = Mesa::new();
    desalinhada(&mut m);
    let (w, h) = (32_usize, 32_usize);
    let mut px = vec![0_u8; w * h * 4];
    for y in 0..4 {
        for x in 0..4 {
            px[(y * w + x) * 4..][..4].copy_from_slice(&[40, 80, 200, 255]);
        }
    }
    let canto = m.db.insert_image_rgba8(32, 32, px);
    let e = Entity::from_bits(m.e);
    {
        let world = m.sim.world_mut();
        world
            .get_mut::<ph2d_ecs::SpriteSheets>(e)
            .expect("as folhas")
            .0
            .push(ph2d_ecs::ExtraSheet {
                asset: canto,
                size: [1.0, 1.0],
                anchor: [0.0, 0.0],
                hframes: 1,
                vframes: 1,
                offsets: Default::default(),
            });
        world
            .get_mut::<ph2d_ecs::SpriteAnimFrames>(e)
            .expect("os quadros")
            .set("run", vec![ph2d_ecs::FrameRef::Cell { sheet: 2, cell: 0 }]);
    }
    m.frame();
    m.align_on();
    m.paint();
    let at = na_grade(&mut m, [2, 2]);
    m.click(at);
    m.frame();
    m.paint();
    assert_eq!(m.doc().pivot_on_grid(), Some([2, 2]));
    let b = m.doc().layout.align.pivot_follow;
    m.click(centre(b));
    m.frame();
    m.paint();
    let p = m.doc().align.painted;
    let h = centre(p.handles[2].expect("a pega de baixo à esquerda"));
    let (ox, oy) = p.pivot.unwrap();
    let alvo = (ox + (h.0 - ox) * 1.6, oy + (h.1 - oy) * 1.6); // LITERAL-PX-OK: a razão pedida
    let bt = ph2d_host::PointerButton::Primary;
    m.pointer_with(ph2d_host::PointerKind::Down, bt, h, Modifiers::default());
    for i in 1..=4 {
        let t = i as f32 / 4.0; // LITERAL-PX-OK: passos do arrasto
        let at = (h.0 + (alvo.0 - h.0) * t, h.1 + (alvo.1 - h.1) * t);
        m.pointer_with(ph2d_host::PointerKind::Move, bt, at, Modifiers::default());
        m.frame();
        m.paint();
        let pega = centre(m.doc().align.painted.handles[2].expect("a pega"));
        assert!(
            (pega.0 - at.0).abs() < 1.0 && (pega.1 - at.1).abs() < 1.0,
            "passo {i}: a pega em {pega:?}, o rato em {at:?}"
        );
    }
    let tamanho = m
        .sim
        .world()
        .get::<ph2d_ecs::SpriteSheets>(e)
        .and_then(|s| s.sheet(1))
        .expect("a folha torta")
        .size;
    assert!(
        (tamanho[0] - 1.6).abs() < 1e-3 && (tamanho[1] - 1.6).abs() < 1e-3, // LITERAL-PX-OK: a razão pedida
        "a escala pedida é a alcançada: {tamanho:?}"
    );
    m.pointer_with(ph2d_host::PointerKind::Up, bt, alvo, Modifiers::default());
    m.frame();
    m.paint();
    assert_eq!(m.doc().pivot_on_grid(), Some([2, 2]), "ao largar, o Follow");
}
