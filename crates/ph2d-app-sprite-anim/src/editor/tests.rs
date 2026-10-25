//! **Os gates do editor de animações**: cada gesto dirigido pelas portas reais da aba (`pointer`,
//! `pointer_move`, `key`), conferido na edição que ele emite e, depois do [`super::frame`], na cena.

use super::*;
use ph2d_a11y::NodeId;
use ph2d_ecs::{
    AnimationTag, SpriteAnimFrames, SpriteAnimations, SpriteAnimator, SpriteGrid, Transform,
};
use ph2d_editor_core::AnimCycleMode;
use ph2d_host::Modifiers;

struct Mesa {
    sim: SimWorld,
    hero: HeroScreen,
    reg: ComponentRegistry,
    db: AssetDb,
    ts: TextSystem,
    toasts: ToastQueue,
    e: u64,
}

fn registry() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut r);
    ph2d_render::register_render_components(&mut r);
    ph2d_physics_ecs::register_physics_components(&mut r);
    r
}

const AREA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1200.0, // LITERAL-PX-OK: a janela do teste
    h: 800.0,  // LITERAL-PX-OK: a janela do teste
};

impl Mesa {
    /// O herói de fábrica (8 × 4), com `idle` (0..=5) e `run` (8..=15), e o editor aberto nele.
    fn new() -> Self {
        let mut sim = SimWorld::new();
        let mut lib = SpriteAnimations::new();
        lib.insert(AnimationTag::new("idle", 0, 5)).unwrap();
        lib.insert(AnimationTag::new("run", 8, 15)).unwrap();
        let e = sim
            .world_mut()
            .spawn((
                Transform::default(),
                ph2d_render::Sprite::atlas(ph2d_render::HERO_SHEET_KEY, [1.0, 1.0], [1.0; 4]),
                SpriteGrid {
                    hframes: 8,
                    vframes: 4,
                    frame: 0,
                },
                lib,
                SpriteAnimator::new("idle"),
                ph2d_ecs::Name("Hero".into()),
            ))
            .id()
            .to_bits();
        let mut hero = HeroScreen::new(NodeId(1));
        hero.documents
            .install_surface(Box::new(AnimDocs::default()));
        let mut m = Self {
            sim,
            hero,
            reg: registry(),
            db: AssetDb::new(),
            ts: TextSystem::without_system_fonts(),
            toasts: ToastQueue::default(),
            e,
        };
        assert!(request_open(&mut m.sim, e, &AnimFieldEdit::OpenEditor));
        m.frame();
        m
    }

    fn frame(&mut self) {
        let atlas = BTreeMap::new();
        frame(
            &mut self.sim,
            &mut self.hero,
            &self.db,
            &atlas,
            &self.reg,
            &mut self.toasts,
        );
        self.paint();
    }

    fn paint(&mut self) {
        let mut scene = VectorScene::new();
        let e = self.e;
        let mut ts = std::mem::replace(&mut self.ts, TextSystem::without_system_fonts());
        self.docs()
            .paint(e, AREA, &mut scene, &mut ts, &Theme::default());
        self.ts = ts;
    }

    fn docs(&mut self) -> &mut AnimDocs {
        self.hero
            .documents
            .surfaces_mut()
            .find_mut::<AnimDocs>()
            .expect("a família instalada")
    }

    fn doc(&mut self) -> &mut AnimDoc {
        let e = self.e;
        self.docs().doc_mut(e).expect("a aba aberta")
    }

    fn pointer(&mut self, kind: PointerKind, (x, y): (f32, f32)) {
        let e = self.e;
        let mut ts = std::mem::replace(&mut self.ts, TextSystem::without_system_fonts());
        let input = PointerInput {
            text: &mut ts,
            mods: Modifiers::default(),
            now_ns: 0,
        };
        self.docs()
            .pointer(e, input, kind, PointerButton::Primary, x, y);
        self.ts = ts;
    }

    fn drag_to(&mut self, (x, y): (f32, f32)) {
        let e = self.e;
        let mut ts = std::mem::replace(&mut self.ts, TextSystem::without_system_fonts());
        let input = PointerInput {
            text: &mut ts,
            mods: Modifiers::default(),
            now_ns: 0,
        };
        self.docs().pointer_move(e, input, x, y);
        self.ts = ts;
    }

    fn click(&mut self, at: (f32, f32)) {
        self.pointer(PointerKind::Down, at);
        self.pointer(PointerKind::Up, at);
    }

    fn key(&mut self, key: Key, text: Option<&str>) {
        let e = self.e;
        let mut ts = std::mem::replace(&mut self.ts, TextSystem::without_system_fonts());
        self.docs().key(
            e,
            &mut ts,
            KeyInput {
                key,
                mods: Modifiers::default(),
                pressed: true,
                text,
                paste: None,
            },
        );
        self.ts = ts;
    }

    fn cell(&mut self, c: u32) -> (f32, f32) {
        centre(self.doc().cell_rect(c))
    }

    fn pending(&mut self) -> Vec<AnimFieldEdit> {
        self.doc().pending.clone()
    }

    fn world_frames(&self, name: &str) -> Option<Vec<u32>> {
        self.sim
            .world()
            .get::<SpriteAnimFrames>(Entity::from_bits(self.e))
            .and_then(|f| f.get(name))
            .map(<[u32]>::to_vec)
    }
}

fn centre(r: Rect) -> (f32, f32) {
    (r.x + r.w / 2.0, r.y + r.h / 2.0)
}

#[test]
fn pedir_o_editor_duas_vezes_abre_uma_aba_so_e_traz_a_frente() {
    let mut m = Mesa::new();
    let e = m.e;
    request_open(&mut m.sim, e, &AnimFieldEdit::OpenEditor);
    request_open(&mut m.sim, e, &AnimFieldEdit::OpenEditor);
    m.frame();
    let tabs = m.docs().tabs();
    assert_eq!(tabs.len(), 1);
    assert_eq!(tabs[0].title, "Animations \u{2014} Hero");
    m.hero.documents.follow_surfaces();
    assert_eq!(m.hero.documents.surface_active().map(|t| t.id), Some(e));
    assert!(
        !request_open(&mut m.sim, e, &AnimFieldEdit::Rewind),
        "só o OpenEditor é pedido"
    );
}

#[test]
fn a_folha_de_fabrica_chega_ao_editor_e_ao_cartao() {
    let mut m = Mesa::new();
    let snap = &m.doc().snap;
    let sheet = snap.sheet.as_ref().expect("os pixels do herói");
    assert_eq!((sheet.w, sheet.h), (1536, 768));
    let card = snap.info.as_ref().unwrap().card.sheet.clone();
    assert_eq!(card.as_ref(), Some(sheet), "o cartão lê a MESMA folha");
}

#[test]
fn clicar_uma_celula_junta_a_ao_fim_e_chega_a_cena() {
    let mut m = Mesa::new();
    let at = m.cell(20);
    m.click(at);
    assert_eq!(
        m.pending(),
        [AnimFieldEdit::Frames(0, vec![0, 1, 2, 3, 4, 5, 20])],
        "a idle (0..=5) ganha a célula 20 no fim"
    );
    m.frame();
    assert_eq!(m.world_frames("idle"), Some(vec![0, 1, 2, 3, 4, 5, 20]));
    assert_eq!(m.doc().shown_frames(), [0, 1, 2, 3, 4, 5, 20]);
}

#[test]
fn arrastar_por_tres_celulas_junta_as_pela_ordem_do_arrasto() {
    let mut m = Mesa::new();
    let (a, b, c) = (m.cell(11), m.cell(10), m.cell(18));
    m.pointer(PointerKind::Down, a);
    m.drag_to(a);
    m.drag_to(b);
    m.drag_to(c);
    m.pointer(PointerKind::Up, c);
    assert_eq!(
        m.pending(),
        [AnimFieldEdit::Frames(0, vec![0, 1, 2, 3, 4, 5, 11, 10, 18])]
    );
}

#[test]
fn uma_animacao_acabada_de_criar_comeca_do_primeiro_clique() {
    let mut m = Mesa::new();
    let add = centre(m.doc().layout.add);
    m.click(add);
    m.frame();
    assert_eq!(m.doc().selected, 2, "a nova fica escolhida");
    let at = m.cell(7);
    m.click(at);
    assert_eq!(m.pending(), [AnimFieldEdit::Frames(2, vec![7])]);
}

#[test]
fn arrastar_um_quadro_na_faixa_reordena() {
    let mut m = Mesa::new();
    m.doc()
        .pending
        .push(AnimFieldEdit::Frames(0, vec![5, 2, 7]));
    m.frame();
    let l = m.doc().layout.clone();
    let from = centre(l.frames[0].0);
    // Sobre a metade direita do último: o marcador vai para depois dele.
    let to = (l.frames[2].0.x + l.frames[2].0.w * 0.75, from.1); // LITERAL-PX-OK: a metade direita
    m.pointer(PointerKind::Down, from);
    m.drag_to(to);
    m.paint();
    assert_eq!(
        m.doc().shown_frames(),
        [5, 2, 7],
        "a faixa fica como está; quem mostra o destino é o marcador"
    );
    assert_eq!(m.doc().painted.marker, Some(3));
    m.pointer(PointerKind::Up, to);
    assert_eq!(m.pending(), [AnimFieldEdit::Frames(0, vec![2, 7, 5])]);
    m.frame();
    assert_eq!(m.world_frames("idle"), Some(vec![2, 7, 5]));
}

#[test]
fn clicar_um_quadro_mostra_o_e_o_x_e_o_delete_tiram_no() {
    let mut m = Mesa::new();
    m.doc()
        .pending
        .push(AnimFieldEdit::Frames(0, vec![5, 2, 7]));
    m.frame();
    let t = centre(m.doc().layout.frames[1].0);
    m.click(t);
    assert_eq!(m.pending(), [AnimFieldEdit::SetStep(1)]);
    m.frame();
    m.key(Key::Delete, None);
    assert_eq!(m.pending(), [AnimFieldEdit::Frames(0, vec![5, 7])]);
    m.frame();
    let x = centre(m.doc().layout.frames[0].2);
    m.click(x);
    assert_eq!(m.pending(), [AnimFieldEdit::Frames(0, vec![7])]);
    m.frame();
    let x = centre(m.doc().layout.frames[0].2);
    m.click(x);
    assert_eq!(
        m.pending(),
        [AnimFieldEdit::SetStep(0)],
        "o último quadro não sai (vazio = o intervalo): o clique só o escolhe"
    );
}

#[test]
fn o_tempo_de_um_quadro_escreve_se_no_campo_dele() {
    let mut m = Mesa::new();
    let ms = centre(m.doc().layout.frames[2].1);
    m.click(ms);
    m.key(Key::Backspace, None);
    m.key(Key::Backspace, None);
    m.key(Key::Backspace, None);
    // O campo é o MULTIPLICADOR do ritmo (lei do Godot): 2,5 × 100 ms.
    m.key(Key::Char('2'), Some("2,5"));
    m.key(Key::Enter, None);
    assert_eq!(m.pending(), [AnimFieldEdit::FrameMsAt(0, 2, 250)]);
    m.frame();
    assert_eq!(m.doc().frame_ms_at(2), 250);
    assert_eq!(m.doc().ms_label(2), "2.5x");
}

/// ⭐ Report do dono 25/10: com tempos próprios o Frames/s «não funcionava». Mudar o ritmo leva o
/// quadro de 2× junto (continua 2×), como no Godot.
#[test]
fn mudar_os_quadros_por_segundo_leva_o_tempo_proprio_junto() {
    let mut m = Mesa::new();
    let quadro = centre(m.doc().layout.frames[2].1);
    m.click(quadro);
    m.key(Key::Backspace, None);
    m.key(Key::Char('2'), Some("2"));
    m.key(Key::Enter, None);
    m.frame();
    let fps = centre(m.doc().layout.fps);
    m.click(fps);
    m.key(Key::Backspace, None);
    m.key(Key::Backspace, None);
    m.key(Key::Char('2'), Some("20"));
    m.key(Key::Enter, None);
    m.frame();
    let t = &m
        .sim
        .world()
        .get::<SpriteAnimations>(Entity::from_bits(m.e))
        .unwrap()
        .0[0];
    assert_eq!(
        (t.frame_ms, t.per_frame_ms.as_slice()),
        (50, &[0, 0, 100][..])
    );
    assert_eq!(m.doc().ms_label(2), "2x");
    assert_eq!(m.doc().ms_label(0), "1x");
}

#[test]
fn quadros_por_segundo_escritos_viram_o_tempo_do_quadro() {
    let mut m = Mesa::new();
    let fps = centre(m.doc().layout.fps);
    m.click(fps);
    m.key(Key::Backspace, None);
    m.key(Key::Backspace, None);
    m.key(Key::Char('1'), Some("12"));
    m.key(Key::Enter, None);
    assert_eq!(m.pending(), [AnimFieldEdit::FrameMs(0, 83)]);
    m.frame();
    let t = m
        .sim
        .world()
        .get::<SpriteAnimations>(Entity::from_bits(m.e))
        .unwrap()
        .0[0]
        .frame_ms;
    assert_eq!(t, 83);
}

#[test]
fn o_modo_do_ciclo_e_um_botao_por_modo() {
    let mut m = Mesa::new();
    let once = centre(m.doc().layout.modes[1]);
    m.click(once);
    assert_eq!(
        m.pending(),
        [AnimFieldEdit::CycleMode(0, AnimCycleMode::Once.tag())]
    );
    m.frame();
    assert_eq!(m.doc().mode(), Some(AnimCycleMode::Once));
}

#[test]
fn o_play_toca_a_animacao_aberta_e_o_segundo_clique_pausa() {
    let mut m = Mesa::new();
    let run = centre(m.doc().layout.list_rows[1]);
    m.click(run);
    assert_eq!(m.pending(), [AnimFieldEdit::SetCurrent("run".into())]);
    m.frame();
    let play = centre(m.doc().layout.play);
    m.click(play);
    assert_eq!(m.pending(), [AnimFieldEdit::Playing(true)]);
    m.frame();
    let p = m
        .sim
        .world()
        .get::<SpriteAnimator>(Entity::from_bits(m.e))
        .unwrap()
        .clone();
    assert!(p.playing && p.current == "run");
    assert!(m.doc().playing());
    m.click(play);
    assert_eq!(m.pending(), [AnimFieldEdit::Playing(false)]);
}

#[test]
fn renomear_pela_lista() {
    let mut m = Mesa::new();
    let idle = centre(m.doc().layout.list_rows[0]);
    m.click(idle);
    for _ in 0..4 {
        m.key(Key::Backspace, None);
    }
    m.key(Key::Char('p'), Some("parado"));
    m.key(Key::Enter, None);
    assert_eq!(m.pending(), [AnimFieldEdit::Rename(0, "parado".into())]);
}

#[test]
fn a_grelha_proposta_aparece_quando_difere_e_usar_aplica_a() {
    let mut m = Mesa::new();
    assert!(
        m.doc().layout.banner.is_none(),
        "8 × 4 já é a grelha do herói"
    );
    m.doc().pending.push(AnimFieldEdit::Grid(1, 1));
    m.frame();
    let (_, use_) = m.doc().layout.banner.expect("a folha parece 8 × 4");
    m.click(centre(use_));
    assert_eq!(m.pending(), [AnimFieldEdit::Grid(8, 4)]);
    m.frame();
    let g = *m
        .sim
        .world()
        .get::<SpriteGrid>(Entity::from_bits(m.e))
        .unwrap();
    assert_eq!((g.hframes, g.vframes), (8, 4));
    assert!(m.doc().layout.banner.is_none());
}

#[test]
fn a_aba_fecha_quando_a_sprite_sai_da_cena() {
    let mut m = Mesa::new();
    let e = Entity::from_bits(m.e);
    m.sim.world_mut().despawn(e);
    m.frame();
    assert!(m.docs().tabs().is_empty());
}

#[test]
fn largar_uma_folha_abre_o_editor_com_a_proposta_e_uma_imagem_so_nao() {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::default(),
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
        ))
        .id()
        .to_bits();
    // Três figuras de 4 px no meio de células de 8 px: uma folha 3 × 1.
    let (w, h) = (24u32, 8u32);
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for cell in 0..3 {
        for y in 2..6 {
            for x in 2..6 {
                rgba[((y * w + cell * 8 + x) * 4 + 3) as usize] = 255;
            }
        }
    }
    assert!(propose_after_drop(&mut sim, e, &rgba, w, h));
    let asked = &sim.world().get_resource::<AnimEditorRequests>().unwrap().0;
    assert_eq!(asked, &[e]);

    // Uma figura só não é folha: nada abre.
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(Transform::default()).id().to_bits();
    let mut one = vec![0u8; 8 * 8 * 4];
    one[(3 * 8 + 3) * 4 + 3] = 255;
    assert!(!propose_after_drop(&mut sim, e, &one, 8, 8));
    assert!(
        !propose_after_drop(&mut sim, e, &one, 9, 9),
        "tamanho errado"
    );
}

impl Mesa {
    /// Pinta a aba em `area` com o censo das elisões armado; devolve o que saiu CORTADO.
    fn paint_in(&mut self, area: Rect) -> Vec<String> {
        let mut scene = VectorScene::new();
        let e = self.e;
        let mut ts = std::mem::replace(&mut self.ts, TextSystem::without_system_fonts());
        let ((), medidos) = ph2d_editor_core::text_elide::elisao::medindo(|| {
            self.docs()
                .paint(e, area, &mut scene, &mut ts, &Theme::default());
        });
        self.ts = ts;
        medidos
            .into_iter()
            .filter(|m| !m.coube())
            .map(|m| m.texto)
            .collect()
    }

    /// Quantos componentes o herói (a única entidade do jogo desta mesa) carrega.
    fn hero_components(&self) -> u32 {
        let w = self.sim.world();
        w.entity(Entity::from_bits(self.e))
            .archetype()
            .components()
            .len() as u32
    }
}

/// Os seis controlos da barra de cima.
fn bar(l: &doc::Layout) -> [Rect; 7] {
    [
        l.play, l.fps, l.modes[0], l.modes[1], l.modes[2], l.cols, l.rows,
    ]
}

/// ⭐ Report do dono (foto 03): depois do «6 fps», Repeat · Once · Ping-pong e as colunas/linhas
/// saíam «…». Cada controlo mede o seu texto; numa aba estreita a barra QUEBRA em vez de cortar.
#[test]
fn a_barra_de_cima_mede_o_texto_e_quebra_na_largura() {
    for w in [1200.0, 420.0, 240.0] {
        // LITERAL-PX-OK: a janela do teste
        let mut m = Mesa::new();
        let area = Rect::new(0.0, 0.0, w, AREA.h);
        let cut = m.paint_in(area);
        let d = m.doc();
        let labels = [
            d.fps_label(),
            "Repeat".into(),
            "Once".into(),
            "Ping-pong".into(),
            d.cols_label(),
            d.rows_label(),
        ];
        for t in &labels {
            assert!(!cut.contains(t), "«{t}» saiu cortado a {w} px: {cut:?}");
        }
        if w > 300.0 {
            assert!(cut.is_empty(), "nada da aba se corta a {w} px: {cut:?}");
        }
        let rects = bar(&d.layout);
        for (i, r) in rects.iter().enumerate() {
            assert!(
                r.x >= area.x && r.x + r.w <= area.x + area.w,
                "{i} fora da aba a {w}"
            );
            for o in &rects[i + 1..] {
                let apart =
                    r.x + r.w <= o.x || o.x + o.w <= r.x || r.y + r.h <= o.y || o.y + o.h <= r.y;
                assert!(apart, "{r:?} pisa {o:?} a {w} px");
            }
        }
        if w < 300.0 {
            assert!(
                d.layout.rows.y > d.layout.play.y,
                "a {w} px a barra tinha de quebrar"
            );
        }
    }
}

/// ⭐ Report do dono (foto 03): a faixa de quadros ficava debaixo da pílula de estado (EDIT · fps…)
/// e os tempos liam «10:…». A faixa inteira fica acima da pílula, e cada «N ms» cabe.
#[test]
fn a_faixa_fica_acima_da_pilula_e_os_tempos_cabem() {
    let mut m = Mesa::new();
    let cut = m.paint_in(AREA);
    let d = m.doc();
    let s = d.layout.strip;
    let pill_top = AREA.y + AREA.h - ph2d_tokens::HUD_BOTTOM_PAD_PX - ph2d_tokens::HUD_H_PX;
    assert!(
        s.y + s.h <= pill_top,
        "a faixa acaba em {} e a pílula começa em {pill_top}",
        s.y + s.h
    );
    assert_eq!(d.layout.frames.len(), 6, "a idle tem seis quadros");
    for (i, f) in d.layout.frames.iter().enumerate() {
        assert!(
            f.1.y + f.1.h <= s.y + s.h,
            "o tempo do quadro {i} sai da faixa"
        );
        let ms = d.ms_label(i);
        assert!(!cut.contains(&ms), "«{ms}» saiu cortado: {cut:?}");
    }
}

/// ⭐ Report do dono: depois do «+ Add» do editor o cartão continuava a realçar «idle». O «+» escolhe
/// a nova E põe-na no ecrã (o nome é o `next_free_name` da biblioteca antes do Add).
#[test]
fn o_mais_escolhe_a_nova_e_poe_na_no_ecra() {
    let mut m = Mesa::new();
    let add = centre(m.doc().layout.add);
    m.click(add);
    m.frame();
    let current = m
        .sim
        .world()
        .get::<SpriteAnimator>(Entity::from_bits(m.e))
        .map(|p| p.current.clone());
    assert_eq!(
        current.as_deref(),
        Some("anim_0"),
        "a nova tem de ficar no ecrã"
    );
    let d = m.doc();
    assert_eq!(d.selected, 2);
    assert_eq!(
        d.snap.info.as_ref().map(|i| i.current.as_str()),
        Some("anim_0")
    );
}

/// E o contrário: quem põe outra no ecrã fora do editor (o cartão) muda a aberta do editor.
#[test]
fn outra_no_ecra_muda_a_aberta_do_editor() {
    let mut m = Mesa::new();
    assert_eq!(m.doc().selected, 0);
    let e = Entity::from_bits(m.e);
    m.sim
        .world_mut()
        .get_mut::<SpriteAnimator>(e)
        .expect("o tocador")
        .current = "run".into();
    m.frame();
    assert_eq!(m.doc().selected, 1);
}

/// ⭐ Report do dono: o «N components» da Hierarquia subia `+2` a cada gesto no editor (161 → 163 ao
/// abrir, → 165 depois do «+ Add» e de três cliques). Medido aqui, passo a passo, com a conta antiga
/// (Σ arquétipos, recursos incluídos) ao lado: abrir `6 → 12` (o `AnimEditorRequests` e a
/// `SheetCache` são entidades-recurso de dois componentes cada, mais o interno do bevy), o 1.º
/// clique numa célula `+2` (a `SpriteAnimFrames` e a `SpriteAnimCursor` aparecem UMA vez no herói), e
/// mais nada. A cura: o cabeçalho conta só as entidades do jogo.
#[test]
fn o_cabecalho_da_hierarquia_conta_so_o_jogo_e_nao_cresce_com_os_gestos() {
    let mut m = Mesa::new();
    assert_eq!(m.sim.component_count(), m.hero_components(), "ao abrir");
    assert_eq!(m.hero_components(), 6);
    let add = centre(m.doc().layout.add);
    m.click(add);
    m.frame();
    assert_eq!(m.sim.component_count(), 6, "o «+» não junta componentes");
    let mut after_first = None;
    for c in [3u32, 4, 5, 9, 10] {
        let at = m.cell(c);
        m.click(at);
        m.frame();
        assert_eq!(m.sim.component_count(), m.hero_components(), "célula {c}");
        after_first.get_or_insert(m.sim.component_count());
        assert_eq!(
            Some(m.sim.component_count()),
            after_first,
            "célula {c}: cresceu"
        );
    }
    assert_eq!(
        after_first,
        Some(8),
        "a lista de quadros e o cursor, uma vez"
    );
    m.click(add);
    m.frame();
    assert_eq!(m.sim.component_count(), 8, "o 2.º «+» não junta nada");
}

#[path = "tests_strip.rs"]
mod strip;
