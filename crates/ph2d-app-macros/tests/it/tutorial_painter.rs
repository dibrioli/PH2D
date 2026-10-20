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

/// O meio do Splash da banca: Digital, sem água — medir a água custa 960 quadros por medida (W7) e
/// estas bancas ensaiam o balão, não a água. As que medem a água ([`Banca::com_agua`]) dizem-no.
const MEIO: PaintMedia = PaintMedia::Digital;

impl Banca {
    fn nova() -> Self {
        let (doc, _) =
            ph2d_tool_painter::macro_track::roteiro::grava_no_papel_nos_meios(LADO, 0, &[MEIO; 4])
                .expect("grava");
        Self::com(doc)
    }

    /// O Splash em Wet Paint: a medição demora (a água assenta).
    fn com_agua() -> Self {
        let (doc, _) =
            ph2d_tool_painter::macro_track::roteiro::grava_no_papel(LADO, 0, 4).expect("grava");
        Self::com(doc)
    }

    /// A caixa (W13): quatro traços, o azul do C&F dentro, um traço por cima.
    fn com_a_caixa() -> Self {
        let (doc, _) =
            ph2d_tool_painter::macro_track::roteiro::grava_a_caixa_com_um_fill(LADO, 0, MEIO)
                .expect("grava");
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

    /// Uma versão nova da `Macro`: o autor dá a `dica` à 1.ª pincelada.
    fn da_dica(&mut self, dica: &str) {
        use ph2d_macro::LeitorDaFamilia;
        use ph2d_tool_painter::macro_track::tutorial::regioes;
        let e = ph2d_ecs::Entity::from_bits(self.bits);
        let doc = &mut self.sim.world_mut().get_mut::<Macro>(e).expect("macro").doc;
        let mut d = ph2d_macro::codec::read(doc).expect("lê");
        let track = &mut d.tracks[0];
        let k = regioes(track, LADO, LADO).expect("regiões")[0].k;
        let ancora =
            ph2d_tool_painter::macro_track::etapas::LeitorDoPainter.etapas(&track.events)[k].inicio;
        track.edicao_mut(ancora).dica = Some(dica.into());
        *doc = ph2d_macro::codec::write(&d);
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
    b.t.set_paint_media(PaintMedia::WetPaint);
    assert_eq!(
        b.quadro().pedido,
        Some(PedidoDoTutorial::Meio {
            chave_do_meio: MEIO.name_key()
        })
    );
    b.t.set_paint_media(MEIO);
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
    b.t.set_paint_media(MEIO);
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

/// ⭐ **O ensino prepara-se fora do quadro** (W6d, `03_plano_W6.md` §6.3): no 1.º quadro de uma versão
/// o balão está «a preparar» — passo 0, total por saber, sem «Mostra-me» nem área — mesmo com a 1.ª
/// pincelada já pintada; quando o trabalho chega, avança como antes. «Recomeçar» vale a preparar.
#[test]
fn uma_versao_nova_prepara_se_fora_do_quadro() {
    use ph2d_tool_painter::macro_track::tutorial::{mostra, regioes};
    let mut b = Banca::com_agua();
    b.activo = true;
    b.abre();
    b.t.set_paint_media(PaintMedia::WetPaint);
    let doc = b
        .sim
        .world()
        .get::<Macro>(ph2d_ecs::Entity::from_bits(b.bits))
        .expect("macro")
        .doc
        .clone();
    let track = ph2d_macro::codec::read(&doc).expect("lê").tracks.remove(0);
    let k = regioes(&track, LADO, LADO).expect("regiões")[0].k;
    mostra(&mut b.t, &track, k).expect("mostra");
    b.assenta();
    let x = b.um_quadro();
    assert_eq!(
        (x.passo, x.total, x.pedido.clone(), x.area.clone()),
        (0, 0, Some(PedidoDoTutorial::APreparar), None),
        "a preparar: nenhuma pincelada conta"
    );
    assert!(!x.pode_mostrar());
    assert!(b.m.a_medir());
    let x = b.quadro();
    assert_eq!(
        (x.passo, x.total, x.pedido),
        (2, 4, Some(PedidoDoTutorial::Pinta)),
        "o trabalho chegou: a 1.ª está feita"
    );

    // Uma versão nova (o autor deu uma dica) volta a preparar; «Recomeçar» vale.
    b.da_dica("Pinta o céu");
    assert_eq!(b.um_quadro().pedido, Some(PedidoDoTutorial::APreparar));
    b.m.pede(recomeca(b.bits));
    assert_eq!(b.um_quadro().erro, None, "recomeçar a preparar");
    let x = b.quadro();
    assert_eq!((x.passo, x.dica.as_deref()), (1, Some("Pinta o céu")));
    assert!(b.t.undo_last(), "o recomeçar é um passo de undo");
    assert_eq!(b.quadro().passo, 2);
}

/// ⭐ **A medição de uma versão velha larga-se** (W6d): a `Macro` muda ENQUANTO a versão anterior
/// ainda se mede — o trabalho dela pára e sai da conta; só o da versão nova fica até chegar. Um
/// trabalho velho guardado deixava o «a medir» aceso para sempre (ninguém o colhe) e o fio a medir
/// uma versão que já não existe.
#[test]
fn mudar_a_macro_a_meio_da_medicao_larga_a_versao_velha() {
    let mut b = Banca::com_agua();
    b.um_quadro();
    assert!(b.m.a_medir(), "a 1.ª versão mede-se");
    b.da_dica("Pinta o céu");
    let prazo = std::time::Instant::now() + std::time::Duration::from_secs(300);
    b.um_quadro();
    while b.m.a_medir() && std::time::Instant::now() < prazo {
        std::thread::sleep(std::time::Duration::from_millis(2));
        b.um_quadro();
    }
    assert!(
        !b.m.a_medir(),
        "a medição da versão velha continua na conta depois de a nova chegar"
    );
    b.activo = true;
    b.abre();
    b.t.set_paint_media(PaintMedia::WetPaint);
    let x = b.um_quadro();
    assert_eq!(
        (x.passo, x.total, x.dica.as_deref()),
        (1, 4, Some("Pinta o céu")),
        "o ensino é o da versão nova"
    );
}

// LITERAL-COLOR-OK: a cor GRAVADA do Fill da caixa (`roteiro::AZUL_DA_CAIXA`).
const AZUL: [u8; 3] = [40, 110, 210];

/// ⭐ W13: na etapa do Fill da caixa o balão pede a cor dele no C&F (o anel no botão, a amostra, o
/// código no texto) enquanto o pincel não a tem; depois o soltar, com o alvo onde a cor caiu; o
/// «Mostra-me» fá-lo e o passo segue para o traço por cima.
#[test]
fn na_etapa_do_fill_o_balao_pede_a_cor_e_depois_o_soltar() {
    use ph2d_editor_core::ids::PAINTER_RAIL_FILL;
    let mut b = Banca::com_a_caixa();
    b.activo = true;
    b.abre();
    b.t.set_paint_media(MEIO);
    let x = b.quadro();
    assert_eq!((x.passo, x.total), (1, 6));
    for _ in 0..4 {
        b.pede(mostra_me);
    }
    b.t.set_brush_color_srgb8([0, 0, 0]);
    let x = b.quadro();
    assert_eq!(
        (x.passo, x.pedido.clone()),
        (5, Some(PedidoDoTutorial::Cor { cor: AZUL }))
    );
    assert_eq!(x.alvos, vec![PAINTER_RAIL_FILL], "o anel no C&F");
    assert_eq!(x.cor(), Some(AZUL), "a amostra");
    assert!(x.frase().1.contains("#286ED2"), "{}", x.frase().1);
    let area = x.area.clone().expect("o alvo do Fill");
    assert!(area.solta && area.pontos.len() == 1, "{area:?}");
    let [u, v] = area.pontos[0];
    assert!(
        (u - 0.45 * LADO as f32).abs() < 1.0 && (v - 0.55 * LADO as f32).abs() < 1.0,
        "o alvo é onde a cor caiu: {u}, {v}"
    );
    assert!(x.pode_mostrar(), "«Mostra-me» faz o Fill");
    b.t.set_brush_color_srgb8(AZUL);
    let x = b.quadro();
    assert_eq!(x.pedido, Some(PedidoDoTutorial::Solta { cor: AZUL }));
    assert_eq!(x.alvos, vec![PAINTER_RAIL_FILL]);
    let x = b.pede(mostra_me);
    assert_eq!(
        (x.passo, x.pedido.clone()),
        (6, Some(PedidoDoTutorial::Pinta))
    );
    assert!(x.alvos.is_empty() && !x.area.expect("a área do traço").solta);
    let x = b.pede(mostra_me);
    assert_eq!((x.passo, x.pedido), (6, None), "a caixa feita");
}
