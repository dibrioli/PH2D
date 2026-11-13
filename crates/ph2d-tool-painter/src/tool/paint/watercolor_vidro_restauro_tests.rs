//! **A base da sessão molhada depois de quem escreve o vidro FORA do quadro da aguada** (ADR-0204 D3, o risco
//! de correcção): Ctrl+Z e Ctrl+Shift+Z a meio da sessão, duplicar a camada, um traço Wet Paint na mesma camada
//! (trocar de meio não seca a aguada). Oráculo independente do journal: cada janela do `vbase` = o vidro que
//! estava vivo quando a tela viva era a base da sessão (o par tela↔vidro gravado em cada pen-down), ao bit.

use super::watercolor_vidro::{GUARDA_O_INICIO_DA_SESSAO, JANELAS_DO_VBASE};
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

const W: u32 = 256;
const H: u32 = 160;

type Par = (Vec<u8>, Option<Vec<crate::compositor::vidro::Vidro>>);

struct Cena {
    t: PainterTool,
    /// (tela, vidro da camada activa) em cada pen-down.
    pares: Vec<Par>,
    janelas_conferidas: usize,
}

impl Cena {
    fn nova() -> Self {
        GUARDA_O_INICIO_DA_SESSAO.set(true);
        JANELAS_DO_VBASE.with(|j| j.borrow_mut().clear());
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
        t.set_paint_media(PaintMedia::Watercolor);
        crate::tool::paint::tests::aquarela_neutra(&mut t);
        crate::tool::paint::tests::watercolor_basic_de_antes(&mut t);
        t.set_brush_size_px(14.0);
        t.set_brush_color_srgb8([30, 60, 220]);
        Self {
            t,
            pares: Vec::new(),
            janelas_conferidas: 0,
        }
    }

    fn vidro_vivo(&self) -> Option<Vec<crate::compositor::vidro::Vidro>> {
        let c = self.t.layers.active()?;
        self.t.vidros.get(&c).map(|p| p.as_ref().clone())
    }

    /// Um traço horizontal em `y`; com `aguada`, confere cada janela do `vbase` contra o par da base.
    /// Devolve o pen-down cuja tela é a base da sessão (`None` sem aguada).
    fn traco(&mut self, y: f32, aguada: bool, rotulo: &str) -> Option<usize> {
        self.pares
            .push((self.t.canvas_rgba.to_vec(), self.vidro_vivo()));
        let p = |x: f32, phase| CanvasPointer {
            pos: [x, y],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        };
        JANELAS_DO_VBASE.with(|j| j.borrow_mut().clear());
        self.t.on_canvas_pointer(p(40.0, PointerPhase::Down));
        self.t.on_tick(1000.0 / 60.0);
        let base = (self.t.paint.wet_session_base.as_ref())
            .or(self.t.paint.watercolor_base.as_ref())
            .map(|b| b.to_vec());
        for i in 1..=8 {
            self.t
                .on_canvas_pointer(p(40.0 + 16.0 * i as f32, PointerPhase::Move));
            self.t.on_tick(1000.0 / 60.0);
        }
        self.t.on_canvas_pointer(p(170.0, PointerPhase::Up));
        for _ in 0..3 {
            self.t.on_tick(1000.0 / 60.0);
        }
        let janelas = JANELAS_DO_VBASE.with(|j| std::mem::take(&mut *j.borrow_mut()));
        if !aguada {
            return None;
        }
        assert!(!janelas.is_empty(), "{rotulo}: nenhum quadro leu o vbase");
        let base = base.unwrap_or_else(|| panic!("{rotulo}: sem base da aguada"));
        let (k, (_, vidro)) = self
            .pares
            .iter()
            .enumerate()
            .rev()
            .find(|(_, (tela, _))| *tela == base)
            .unwrap_or_else(|| panic!("{rotulo}: a base da sessão não é tela de nenhum pen-down"));
        let fw = W as usize;
        for (j, _) in &janelas {
            for (r, linha) in j.dados.chunks(j.w).enumerate() {
                let y = j.y0 + r;
                let certo: Vec<_> = match vidro {
                    Some(v) => v[y * fw + j.x0..y * fw + j.x0 + j.w].to_vec(),
                    None => vec![[0u8; 7]; j.w],
                };
                assert_eq!(
                    linha,
                    &certo[..],
                    "{rotulo}: o vbase na linha {y} não é o vidro da base da sessão"
                );
            }
            self.janelas_conferidas += 1;
        }
        Some(k)
    }
}

/// Ctrl+Z e Ctrl+Shift+Z a meio da sessão molhada da aguada: o traço seguinte lê o vidro da base dele, ao bit.
#[test]
fn o_ctrl_z_no_meio_da_sessao_molhada_da_o_vidro_da_base_ao_bit() {
    let mut c = Cena::nova();
    c.traco(60.0, true, "1.º");
    let s2 = c.traco(66.0, true, "2.º (mesma sessão)");
    let s3 = c.traco(72.0, true, "3.º (mesma sessão)");
    assert_eq!(
        (s2, s3),
        (Some(0), Some(0)),
        "controlo: os traços não ficaram na sessão do 1.º"
    );
    assert!(c.t.undo_last(), "controlo: o Ctrl+Z não desfez");
    c.traco(78.0, true, "depois de um Ctrl+Z");
    assert!(c.t.undo_last() && c.t.undo_last() && c.t.undo_last());
    c.traco(64.0, true, "depois de desfazer até ao 1.º");
    assert!(c.t.undo_last());
    assert!(c.t.redo_last(), "controlo: o Ctrl+Shift+Z não refez");
    c.traco(70.0, true, "depois de desfazer e refazer");
    c.traco(76.0, true, "a sessão nova continua");
    while c.t.undo_last() {}
    c.traco(62.0, true, "depois de desfazer tudo");
    c.traco(68.0, true, "e a seguinte");
    GUARDA_O_INICIO_DA_SESSAO.set(false);
    assert!(
        c.janelas_conferidas > 20,
        "controlo: {} janelas",
        c.janelas_conferidas
    );
}

/// Um traço Wet Paint na mesma camada entre duas aguadas da mesma sessão (trocar de meio não a seca). ⚠️ Duplicar
/// a camada a meio da sessão fica de fora: a cópia partilha o `Arc` da tela e a sessão continua nela com a base da
/// original (anterior ao D3; nota N7 item 2, passo 6).
#[test]
fn trocar_de_meio_a_meio_da_sessao_da_o_vidro_da_base_ao_bit() {
    let mut c = Cena::nova();
    c.traco(60.0, true, "1.º");
    c.traco(66.0, true, "2.º");
    c.traco(78.0, true, "3.º");
    c.t.set_paint_media(PaintMedia::WetPaint);
    c.traco(84.0, false, "Wet Paint na mesma camada");
    c.t.set_paint_media(PaintMedia::Watercolor);
    c.traco(90.0, true, "aguada depois do Wet");
    c.traco(96.0, true, "e a seguinte");
    GUARDA_O_INICIO_DA_SESSAO.set(false);
    assert!(
        c.janelas_conferidas > 20,
        "controlo: {} janelas",
        c.janelas_conferidas
    );
}
