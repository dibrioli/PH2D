//! ⭐ **O tutorial «fazer» do Painter na família MACROS** (`docs/Macros/03_plano_W6.md` §6, W6b): o
//! Splash gravado num papel branco, um aluno que abre a MESMA imagem pela porta do produto. O balão
//! pede a imagem em modo Paint, depois o meio, depois cada pincelada com a área marcada; «Mostra-me»
//! pinta-a, o Ctrl+Z volta o passo, «Recomeçar» volta ao início.

use ph2d_app_macros::{Cena, Familias, Macros};
use ph2d_ecs::{Macro, ModoDaMacro, SimWorld, StableId};
use ph2d_editor_core::macro_edits::MacroRequest;
use ph2d_editor_core::macro_etapas::erros;
use ph2d_editor_core::macro_tutorial::{BalaoDoTutorial, PedidoDoTutorial, balao};
use ph2d_editor_core::tool::{PanelEvent, Tool};
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
        Self::com(doc)
    }

    fn com(doc: Vec<u8>) -> Self {
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

    fn um_quadro(&mut self) -> BalaoDoTutorial {
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

    /// O quadro depois de as referências do tutorial chegarem (o fio que as mede).
    fn quadro(&mut self) -> BalaoDoTutorial {
        let mut x = self.um_quadro();
        while self.m.a_medir() {
            std::thread::sleep(std::time::Duration::from_millis(2));
            x = self.um_quadro();
        }
        x
    }

    /// O meio pelo seletor do painel (a porta do artista).
    fn meio(&mut self, m: PaintMedia) {
        self.t.handle_panel_event(PanelEvent::SelectOption(
            ph2d_tool_painter::ids::PAINTER_BRUSH_MEDIA,
            m.to_u8().to_string(),
        ));
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

/// ⭐ Os botões só agem na imagem do TUTORIAL: com o Painter na mão aberto noutra imagem (Wet Paint
/// armado), «Mostra-me» e «Recomeçar» não lhe tocam — nem um pixel, nem um passo de undo — e o balão
/// continua a pedir a imagem.
#[test]
fn os_botoes_nao_tocam_noutra_imagem() {
    let mut b = Banca::nova();
    b.activo = true;
    let outra = b.sim.world_mut().spawn(StableId(10)).id().to_bits();
    // Cinza: o papel branco do «Recomeçar» mudá-la-ia.
    let n = LADO as usize * LADO as usize * 4;
    b.t.bind_document(outra, vec![128; n], LADO, LADO);
    b.t.set_paint_media(PaintMedia::WetPaint);
    let camadas = |t: &PainterTool| -> Vec<(u64, Vec<u8>)> {
        t.camadas_rgba()
            .into_iter()
            .map(|(id, px)| (id, px.to_vec()))
            .collect()
    };
    b.assenta();
    let antes = camadas(&b.t);
    for r in [mostra_me, recomeca] {
        let x = b.pede(r);
        b.assenta();
        assert_eq!(x.pedido, Some(PedidoDoTutorial::AbreParaPintar));
        assert!(camadas(&b.t) == antes, "a outra imagem mudou");
        assert!(!b.t.undo_last(), "um passo de undo na outra imagem");
    }
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

/// ⭐ **O meio pedido é o da pincelada corrente** (W6c): a 1.ª gravada em Wet Paint, a 2.ª em
/// Watercolor (a troca é um passo do seletor do painel). Feita a 1.ª, o aluno em Watercolor tem o passo
/// 2 por pintar; em Wet Paint, o balão pede o Watercolor.
#[test]
fn o_balao_pede_o_meio_da_pincelada_corrente() {
    let (doc, _) = ph2d_tool_painter::macro_track::roteiro::grava_no_papel_nos_meios(
        LADO,
        0,
        &[PaintMedia::WetPaint, PaintMedia::Watercolor],
    )
    .expect("grava");
    let mut b = Banca::com(doc);
    b.activo = true;
    b.abre();
    b.meio(PaintMedia::WetPaint);
    assert_eq!(b.quadro().passo, 1);
    b.pede(mostra_me);
    b.assenta();
    let x = b.quadro();
    assert_eq!(
        x.pedido,
        Some(PedidoDoTutorial::Meio {
            chave_do_meio: PaintMedia::Watercolor.name_key()
        }),
        "a 2.ª pincelada foi gravada em Watercolor"
    );
    b.meio(PaintMedia::Watercolor);
    let x = b.quadro();
    assert_eq!((x.passo, x.pedido), (2, Some(PedidoDoTutorial::Pinta)));
}

/// ⭐ Com duas imagens-tutorial, o balão é o da que o Painter tem aberta (não a 1.ª do mundo).
#[test]
fn o_balao_e_o_da_imagem_em_que_o_painter_pinta() {
    let mut b = Banca::nova();
    let doc = b
        .sim
        .world()
        .get::<Macro>(ph2d_ecs::Entity::from_bits(b.bits))
        .expect("macro")
        .doc
        .clone();
    let segunda = b
        .sim
        .world_mut()
        .spawn((
            StableId(11),
            Macro {
                doc,
                modo: ModoDaMacro::Fazer,
            },
        ))
        .id()
        .to_bits();
    // Sem nada aberto o balão é o da 1.ª pelos bits; abre-se a OUTRA.
    let primeira = b.quadro().entity_bits;
    let outra = if primeira == b.bits { segunda } else { b.bits };
    let n = LADO as usize * LADO as usize * 4;
    b.t.bind_document(outra, vec![255; n], LADO, LADO);
    b.activo = true;
    assert_eq!(b.quadro().entity_bits, outra, "a aberta no Painter");
    b.activo = false;
    assert_eq!(
        b.quadro().entity_bits,
        primeira,
        "o Painter fora da mão não decide"
    );
}

/// Uma imagem de outro tamanho que a gravada: o balão di-lo (com o tamanho gravado), em vez de pedir
/// pinceladas que não se podem avaliar.
#[test]
fn outra_tela_diz_o_tamanho_gravado() {
    let mut b = Banca::nova();
    b.activo = true;
    let n = 256 * 256 * 4;
    b.t.bind_document(b.bits, vec![255; n], 256, 256);
    let x = b.quadro();
    assert_eq!(
        x.erro,
        Some(erros::erro_com(
            erros::OUTRO_TAMANHO,
            format!("{LADO}\u{d7}{LADO}")
        ))
    );
    assert!(!x.pode_mostrar());
}
