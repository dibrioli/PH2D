//! ⭐ **O tutorial «fazer» do Painter na família MACROS** (`docs/Macros/03_plano_W6.md` §6, W6b): o
//! Splash gravado num papel branco, um aluno que abre a MESMA imagem pela porta do produto. O balão
//! pede a imagem em modo Paint, depois o meio, depois cada pincelada com a área marcada; «Mostra-me»
//! pinta-a, o Ctrl+Z volta o passo, «Recomeçar» volta ao início.

use ph2d_app_macros::{Cena, Familias, Macros};
use ph2d_ecs::{Macro, ModoDaMacro, SimWorld, StableId};
use ph2d_editor_core::macro_edits::MacroRequest;
use ph2d_editor_core::macro_tutorial::{BalaoDoTutorial, PedidoDoTutorial, balao};
use ph2d_editor_core::tool::Tool;
use ph2d_timeline::{TimelineIntent, TimelineState};
use ph2d_tool_painter::{PaintMedia, PainterTool};

const LADO: u32 = 512;

/// Nenhum Motion nesta banca.
struct SemMotion;
impl ph2d_macro::Gravadora for SemMotion {
    fn abre(
        &mut self,
        _: &mut ph2d_macro::MacroRecorder,
    ) -> Result<ph2d_macro::TrackId, &'static str> {
        Err("sem Motion")
    }
    fn passos(&mut self) -> Vec<Vec<u8>> {
        Vec::new()
    }
    fn para(&mut self) -> Vec<Vec<u8>> {
        Vec::new()
    }
}
impl ph2d_editor_core::macro_tutorial::TutorialDaFamilia for SemMotion {}

struct Banca {
    sim: SimWorld,
    t: PainterTool,
    activo: bool,
    tl: TimelineState,
    intents: Vec<TimelineIntent>,
    m: Macros,
    bits: u64,
}

impl Banca {
    fn nova() -> Self {
        let (doc, _) =
            ph2d_tool_painter::macro_track::roteiro::grava_no_papel(LADO, 0, 4).expect("grava");
        let mut sim = SimWorld::new();
        let bits = sim
            .world_mut()
            .spawn((
                StableId(9),
                Macro {
                    doc,
                    modo: ModoDaMacro::Fazer,
                },
            ))
            .id()
            .to_bits();
        Self {
            sim,
            t: PainterTool::default(),
            activo: false,
            tl: TimelineState::default(),
            intents: Vec::new(),
            m: Macros::default(),
            bits,
        }
    }

    /// A imagem do tutorial aberta no Painter (a sprite branca, pela porta do produto).
    fn abre(&mut self) {
        let n = LADO as usize * LADO as usize * 4;
        self.t.bind_document(self.bits, vec![255; n], LADO, LADO);
    }

    /// A água anda (o editor drena e tica a cada quadro).
    fn assenta(&mut self) {
        for _ in 0..120 {
            drop(self.t.take_preview_arc());
            self.t.on_tick(1000.0 / 60.0);
        }
    }

    fn quadro(&mut self) -> BalaoDoTutorial {
        let mut motion = SemMotion;
        let mut f = Familias {
            painter: Some(&mut self.t),
            painter_activo: self.activo,
            motion: &mut motion,
            motion_em_edicao: None,
            motion_na_mao: false,
            etapas: &[],
        };
        let mut c = Cena {
            sim: &mut self.sim,
            timeline: &mut self.tl,
            intents: &mut self.intents,
            partidas: None,
        };
        self.m.quadro(&mut f, &mut c, 0);
        balao().expect("balão")
    }

    fn pede(&mut self, r: fn(u64) -> MacroRequest) -> BalaoDoTutorial {
        self.m.pede(r(self.bits));
        self.quadro()
    }
}

fn mostra_me(entity_bits: u64) -> MacroRequest {
    MacroRequest::MostraMe { entity_bits }
}
fn recomeca(entity_bits: u64) -> MacroRequest {
    MacroRequest::Recomeca { entity_bits }
}

/// O preparo, por ordem: a imagem em modo Paint, depois o meio da gravação.
#[test]
fn o_balao_pede_a_imagem_em_paint_e_depois_o_meio() {
    let mut b = Banca::nova();
    let x = b.quadro();
    assert_eq!((x.passo, x.total), (0, 4));
    assert_eq!(x.pedido, Some(PedidoDoTutorial::AbreParaPintar));
    assert!(!x.pode_mostrar());
    b.activo = true;
    assert_eq!(
        b.quadro().pedido,
        Some(PedidoDoTutorial::AbreParaPintar),
        "aberta noutra imagem: nenhuma"
    );
    b.abre();
    b.t.set_paint_media(PaintMedia::Digital);
    assert_eq!(
        b.quadro().pedido,
        Some(PedidoDoTutorial::Meio {
            chave_do_meio: "tool.painter.media.wet_paint"
        })
    );
    b.t.set_paint_media(PaintMedia::WetPaint);
    let x = b.quadro();
    assert_eq!(
        (x.passo, x.pedido.clone()),
        (1, Some(PedidoDoTutorial::Pinta)),
        "a tela branca não conta como pintada"
    );
    let area = x.area.clone().expect("a área marcada");
    assert_eq!((area.entity_bits, area.raio), (b.bits, 70.0));
    assert!(x.pode_mostrar());
}

/// ⭐ «Mostra-me» pinta a pincelada corrente; o Ctrl+Z volta o passo; «Recomeçar» volta ao início e
/// desfaz-se num passo.
#[test]
fn mostra_me_desfazer_e_recomecar_no_painter() {
    let mut b = Banca::nova();
    b.activo = true;
    b.abre();
    b.t.set_paint_media(PaintMedia::WetPaint);
    assert_eq!(b.quadro().passo, 1);
    b.pede(mostra_me);
    b.assenta();
    assert_eq!(b.quadro().passo, 2, "a 1.ª pincelada feita");
    b.pede(mostra_me);
    b.assenta();
    assert_eq!(b.quadro().passo, 3);
    assert!(b.t.undo_last());
    b.assenta();
    assert_eq!(b.quadro().passo, 2, "o Ctrl+Z volta o passo");
    let x = b.pede(recomeca);
    assert_eq!((x.passo, x.erro), (1, None), "de volta ao início");
    assert!(b.t.undo_last(), "o recomeçar é um passo de undo");
    assert_eq!(b.quadro().passo, 2);
}
