//! Os gates do editor de Region (plano GEF 16, gates 2 e 3): a área encaixa nas linhas de Columns × Rows; `Alt` e
//! 1 × 1 vão ao píxel; nunca sai da imagem; um clique escolhe a célula; OK entrega a área e a grelha; Cancel nada.

use super::doc::{Outcome, RegionDoc};
use ph2d_editor_core::zones::Rect;

/// Uma aba sobre a folha 800 × 400 do plano, grelha 8 × 4, vista 1 : 1 com a imagem em (0, 0) e a barra fora.
fn aba() -> RegionDoc {
    let mut d = RegionDoc::new(1, 1, "Player".into(), [800, 400], None, [8, 4]);
    d.layout.area = Rect::new(0.0, -100.0, 2000.0, 2000.0);
    d.layout.canvas = Rect::new(0.0, 0.0, 2000.0, 1000.0);
    d.zoom = 1.0;
    d.origin = [0.0, 0.0];
    d
}

fn multiplo_de_100(r: [f32; 4]) -> bool {
    r.iter().all(|v| v.rem_euclid(100.0) == 0.0)
}

#[test]
fn a_area_desenhada_encaixa_nas_linhas_da_grelha() {
    let mut d = aba();
    // Desenhar fora da área (a área é a imagem inteira: carregar fora dela, à direita, não acerta).
    d.rect = [0.0, 0.0, 100.0, 100.0];
    d.press(437.0, 163.0);
    d.motion(612.0, 288.0);
    d.release(612.0, 288.0);
    assert_eq!(d.rect, [400.0, 200.0, 200.0, 100.0]);
    assert!(multiplo_de_100(d.rect));
    // Mover: o canto encaixa, o tamanho fica.
    d.press(450.0, 230.0);
    d.motion(530.0, 160.0);
    d.release(530.0, 160.0);
    assert_eq!(d.rect, [500.0, 100.0, 200.0, 100.0]);
    // A alça do canto de baixo à direita.
    let (_, _, h) = d
        .handles()
        .into_iter()
        .find(|(sx, sy, _)| (*sx, *sy) == (1, 1))
        .expect("alça");
    d.press(h.x + h.w * 0.5, h.y + h.h * 0.5);
    d.motion(777.0, 349.0);
    d.release(777.0, 349.0);
    assert_eq!(d.rect, [500.0, 100.0, 300.0, 200.0]);
    assert!(multiplo_de_100(d.rect));
}

#[test]
fn alt_e_uma_coluna_vao_ao_pixel() {
    let mut d = aba();
    d.rect = [0.0, 0.0, 100.0, 100.0];
    d.set_keys(true, None);
    d.press(437.3, 163.6);
    d.motion(612.4, 288.8);
    d.release(612.4, 288.8);
    assert_eq!(d.rect, [437.0, 164.0, 175.0, 125.0]);
    let mut d = RegionDoc::new(1, 1, "x".into(), [800, 400], None, [1, 1]);
    assert_eq!(d.snap(0, 437.4), 437.0, "1 × 1: ao píxel");
    d.alt = false;
    assert_eq!(d.snap(1, 163.6), 164.0);
}

#[test]
fn a_area_nunca_sai_da_imagem() {
    let mut d = aba();
    d.rect = [600.0, 300.0, 200.0, 100.0];
    d.press(700.0, 350.0);
    d.motion(5000.0, 5000.0);
    d.release(5000.0, 5000.0);
    assert_eq!(d.rect, [600.0, 300.0, 200.0, 100.0], "presa no canto");
    d.press(700.0, 350.0);
    d.motion(-5000.0, -5000.0);
    d.release(-5000.0, -5000.0);
    assert_eq!(d.rect, [0.0, 0.0, 200.0, 100.0]);
    // A alça esquerda passa da direita: fica com uma célula, nunca invertida nem nula.
    let (_, _, h) = d
        .handles()
        .into_iter()
        .find(|(sx, sy, _)| (*sx, *sy) == (-1, 0))
        .expect("alça");
    d.press(h.x + h.w * 0.5, h.y + h.h * 0.5);
    d.motion(1900.0, 50.0);
    d.release(1900.0, 50.0);
    let [x, y, w, h] = d.rect;
    assert!(
        w >= 1.0 && h >= 1.0 && x + w <= 800.0 && y + h <= 400.0,
        "{:?}",
        d.rect
    );
}

/// Um clique sem arrastar escolhe a célula por baixo — fora da área, e DENTRO dela (a Region nova é a imagem
/// inteira: tudo é «dentro», e o clique tem de escolher na mesma — a foto do 1.º smoke mostrou-o perdido).
#[test]
fn um_clique_escolhe_a_celula() {
    let mut d = aba();
    d.rect = [0.0, 0.0, 100.0, 100.0];
    d.press(350.0, 250.0);
    d.release(350.0, 250.0);
    assert_eq!(d.rect, [300.0, 200.0, 100.0, 100.0]);
    d.rect = [0.0, 0.0, 800.0, 400.0];
    d.press(150.0, 50.0);
    d.release(150.0, 50.0);
    assert_eq!(d.rect, [100.0, 0.0, 100.0, 100.0], "dentro da área inteira");
}

#[test]
fn ok_entrega_a_area_e_a_grelha_cancel_nada() {
    let mut d = aba();
    d.rect = [100.0, 0.0, 100.0, 100.0];
    d.editing = Some((super::doc::Field::Cols, "4".into()));
    d.ok();
    assert_eq!(
        d.outcome,
        Some(Outcome::Ok {
            rect: [100, 0, 100, 100],
            grid: [4, 4]
        }),
        "o campo aberto entra no OK"
    );
    let mut d = aba();
    d.layout.cancel = Rect::new(0.0, -50.0, 40.0, 20.0);
    d.press(10.0, -40.0);
    assert_eq!(d.outcome, Some(Outcome::Cancel));
}

#[test]
fn a_grelha_tem_no_maximo_um_pixel_por_celula() {
    let mut d = aba();
    d.editing = Some((super::doc::Field::Rows, "99999".into()));
    d.close_field(true);
    assert_eq!(d.grid, [8, 400]);
    d.editing = Some((super::doc::Field::Cols, "0".into()));
    d.close_field(true);
    assert_eq!(d.grid, [1, 400]);
}

/// O Player do nível 4 em Region, com o editor aberto pelo quadro da família — as portas reais da aba.
mod pela_aba {
    use super::super::*;
    use ph2d_a11y::NodeId;
    use ph2d_ecs::{ImageRegion, SpriteRegion};
    use ph2d_host::Modifiers;

    const AREA: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 1200.0, // LITERAL-PX-OK: a janela do teste
        h: 800.0,  // LITERAL-PX-OK: a janela do teste
    };
    const PPM: f32 = 100.0;

    struct Mesa {
        sim: SimWorld,
        hero: HeroScreen,
        db: AssetDb,
        atlas: BTreeMap<u32, AssetId>,
        ts: TextSystem,
        e: Entity,
    }

    type Visto = (
        Option<ph2d_render::Sprite>,
        Option<SpriteRegion>,
        Option<ImageRegion>,
    );

    impl Mesa {
        fn new() -> Self {
            let mut sim = SimWorld::new();
            ph2d_app_game::smoke::montar(
                &mut sim,
                &ph2d_app_components::test_support::registo(),
                4,
            );
            let w = sim.world_mut();
            let mut q = w.query::<(Entity, &ph2d_ecs::Name)>();
            let e = q
                .iter(w)
                .find(|(_, n)| n.as_str() == "Player")
                .map(|(e, _)| e)
                .expect("Player");
            let db = AssetDb::new();
            let mut atlas = BTreeMap::new();
            ph2d_app_components::factory_images::register(&db, &mut atlas);
            let mut hero = HeroScreen::new(NodeId(1));
            hero.documents
                .install_surface(Box::new(RegionDocs::default()));
            let mut m = Self {
                sim,
                hero,
                db,
                atlas,
                ts: TextSystem::without_system_fonts(),
                e,
            };
            let edit = ImageUseEdit::Use(ph2d_editor_core::InspectorImageUse::Region);
            assert!(m.edit(edit));
            m.frame();
            assert!(
                m.docs().doc(e.to_bits()).is_some(),
                "a Region nova abre o editor"
            );
            m
        }

        fn edit(&mut self, edit: ImageUseEdit) -> bool {
            crate::image_use::apply_edit(
                self.sim.world_mut(),
                self.e,
                &self.db,
                &self.atlas,
                PPM,
                edit,
            )
        }

        fn frame(&mut self) {
            frame(&mut self.sim, &mut self.hero, &self.db, &self.atlas, PPM);
            let mut scene = VectorScene::new();
            let mut ts = std::mem::replace(&mut self.ts, TextSystem::without_system_fonts());
            let id = self.e.to_bits();
            if self.docs().doc(id).is_some() {
                self.docs()
                    .paint(id, AREA, &mut scene, &mut ts, &Theme::default());
            }
            self.ts = ts;
        }

        fn docs(&mut self) -> &mut RegionDocs {
            self.hero
                .documents
                .surfaces_mut()
                .find_mut::<RegionDocs>()
                .expect("família")
        }

        fn click(&mut self, x: f32, y: f32) {
            let id = self.e.to_bits();
            for kind in [PointerKind::Down, PointerKind::Up] {
                let mut ts = std::mem::replace(&mut self.ts, TextSystem::without_system_fonts());
                let input = PointerInput {
                    text: &mut ts,
                    mods: Modifiers::default(),
                    now_ns: 0,
                };
                self.docs()
                    .pointer(id, input, kind, PointerButton::Primary, x, y);
                self.ts = ts;
            }
        }

        fn key(&mut self, key: Key) {
            let id = self.e.to_bits();
            let mut ts = std::mem::replace(&mut self.ts, TextSystem::without_system_fonts());
            self.docs().key(
                id,
                &mut ts,
                KeyInput {
                    key,
                    mods: Modifiers::default(),
                    pressed: true,
                    text: None,
                    paste: None,
                },
            );
            self.ts = ts;
        }

        fn visto(&self) -> Visto {
            let w = self.sim.world();
            (
                w.get::<ph2d_render::Sprite>(self.e).copied(),
                w.get::<SpriteRegion>(self.e).copied(),
                w.get::<ImageRegion>(self.e).copied(),
            )
        }

        /// O centro da célula `c` no ecrã.
        fn celula(&mut self, c: [u32; 2]) -> (f32, f32) {
            let id = self.e.to_bits();
            let d = self.docs().doc(id).expect("aba");
            let p: [f32; 2] = std::array::from_fn(|i| {
                d.origin[i] + (d.grid_line(i, c[i]) + d.grid_line(i, c[i] + 1)) * 0.5 * d.zoom
            });
            (p[0], p[1])
        }
    }

    /// ⭐ Gate 3: Cancel (o botão e o `Esc`) fecha a aba sem mudar nada; a célula + Enter aplica a área e a grelha
    /// num ÚNICO quadro (um passo do desfazer por diff) e fecha a aba; o quadro seguinte não muda mais nada.
    #[test]
    fn ok_aplica_num_quadro_cancel_nao_muda_nada() {
        let mut m = Mesa::new();
        let antes = m.visto();
        let (cx, cy) = m.celula([1, 0]);
        m.click(cx, cy);
        let id = m.e.to_bits();
        let cancel = m.docs().doc(id).expect("aba").layout.cancel;
        m.click(cancel.x + cancel.w * 0.5, cancel.y + cancel.h * 0.5);
        m.frame();
        assert!(m.docs().docs.is_empty(), "Cancel fecha a aba");
        assert_eq!(m.visto(), antes, "Cancel não muda nada");

        assert!(!m.edit(ImageUseEdit::EditRegion));
        m.frame();
        let (cx, cy) = m.celula([1, 0]);
        m.click(cx, cy);
        m.key(Key::Escape);
        m.frame();
        assert_eq!(m.visto(), antes, "Esc não muda nada");

        m.edit(ImageUseEdit::EditRegion);
        m.frame();
        let (cx, cy) = m.celula([1, 0]);
        m.click(cx, cy);
        assert_eq!(m.visto(), antes, "antes do OK nada se escreve");
        m.key(Key::Enter);
        m.frame();
        let depois = m.visto();
        assert!(m.docs().docs.is_empty(), "OK fecha a aba");
        let area = depois.1.expect("a área").rect;
        assert_eq!(
            area,
            [192.0, 0.0, 192.0, 192.0],
            "a célula 1:0 da folha 1536 × 768"
        );
        assert_eq!(depois.2, Some(ImageRegion { grid: [8, 4] }));
        m.frame();
        assert_eq!(m.visto(), depois, "o quadro seguinte não muda mais nada");
    }
}
