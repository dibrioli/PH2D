//! **B18.4 ao bit:** a marca do Wet que vira no 1.º traço de uma camada refaz só a janela da tinta dela, e a
//! tela, os planos do relevo (a pista da placa, persistentes) e o undo dão o bit do caminho antigo (que
//! derrubava a composição inteira, `MARCA_DERRUBA_TUDO`), no papel do dono e na sprite crua.

use super::agua_sobre_oleo_tests::ponteiro;
use super::camada_nova_toque_tests::no_papel_do_dono;
use crate::tool::PainterTool;
use crate::tool::layers::TROCA_RECOMPOE_SEMPRE;
use crate::tool::paint::impasto_fields::MARCA_DERRUBA_TUDO;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};

/// A pista da placa: os planos persistem e cada quadro escreve só a janela reivindicada; cada quadro confere-os
/// contra a dobra inteira.
#[derive(Default)]
struct Placa {
    relevo: Vec<u32>,
    cob: Vec<u8>,
    m0: Vec<u8>,
    m1: Vec<u8>,
    acesa: bool,
}

impl Placa {
    fn quadro(&mut self, t: &mut PainterTool, caso: &str) {
        if !t.take_preview_dirty() {
            return;
        }
        let (w, h) = t.source_size;
        let janela = t
            .preview_gpu_region()
            .filter(|_| self.acesa)
            .unwrap_or((0, 0, w, h));
        let Some(p) = t.impasto_gpu_planes_in(janela) else {
            self.acesa = false;
            return;
        };
        let n = (w * h) as usize;
        if self.relevo.len() != n {
            (self.relevo, self.cob) = (vec![0; n], vec![0; n]);
            (self.m0, self.m1) = (vec![0; n * 4], vec![0; n * 4]);
        }
        let (x, y, pw, ph) = p.region;
        for r in 0..ph as usize {
            let (de, para, k) = (
                r * pw as usize,
                (y as usize + r) * w as usize + x as usize,
                pw as usize,
            );
            for (d, s) in self.relevo[para..para + k]
                .iter_mut()
                .zip(&p.relief[de..de + k])
            {
                *d = s.to_bits();
            }
            self.cob[para..para + k].copy_from_slice(&p.cover[de..de + k]);
            self.m0[para * 4..(para + k) * 4].copy_from_slice(&p.mat0[de * 4..(de + k) * 4]);
            self.m1[para * 4..(para + k) * 4].copy_from_slice(&p.mat1[de * 4..(de + k) * 4]);
        }
        self.acesa = true;
        let inteira = t.impasto_gpu_planes().expect("a dobra inteira");
        let bits: Vec<u32> = inteira.relief.iter().map(|v| v.to_bits()).collect();
        assert!(
            bits == self.relevo,
            "{caso}: relevo da placa ≠ dobra inteira"
        );
        assert!(
            inteira.cover == self.cob,
            "{caso}: cobertura ≠ dobra inteira"
        );
        assert!(
            inteira.mat0 == self.m0 && inteira.mat1 == self.m1,
            "{caso}: material ≠ dobra inteira"
        );
    }
}

/// O caminho antigo: a marca que vira derruba a composição (B18.4) e o «+»/a troca recompõem sempre (B18.3).
fn antigo(ligado: bool) {
    MARCA_DERRUBA_TUDO.with(|c| c.set(ligado));
    TROCA_RECOMPOE_SEMPRE.with(|c| c.set(ligado));
}

/// Os três gémeos: a cura pela pista da placa, a cura pela pista CPU, e o oráculo (caminho antigo) pela CPU.
struct Gemeos {
    placa: (PainterTool, Placa),
    cpu: PainterTool,
    oraculo: PainterTool,
    /// O controlo: os três pelo caminho antigo (o que sobra de diferença é do teste, não da cura).
    controlo: bool,
    /// A última figura que cada gémeo CPU drenou (cura, oráculo): «nada a drenar» = a figura não mudou (B18.12: o
    /// «+» transparente com a tinta viva não marca nada), e a comparação é com a figura que ficou no ecrã.
    figura: (Option<Vec<u8>>, Option<Vec<u8>>),
}

impl Gemeos {
    /// `f` e um quadro (o mesmo relógio) nos três; a placa drena o dela.
    fn em(&mut self, caso: &str, f: &dyn Fn(&mut PainterTool)) {
        let tique = |t: &mut PainterTool| {
            f(t);
            t.on_tick(1000.0 / 60.0);
        };
        antigo(self.controlo);
        tique(&mut self.placa.0);
        self.placa.1.quadro(&mut self.placa.0, caso);
        tique(&mut self.cpu);
        antigo(true);
        tique(&mut self.oraculo);
        antigo(false);
    }

    fn confere(&mut self, caso: &str) {
        for _ in 0..3 {
            self.em(caso, &|_| {});
        }
        if let Some((p, _, _)) = self.cpu.current_preview() {
            self.figura.0 = Some(p.to_vec());
        }
        if let Some((p, _, _)) = self.oraculo.current_preview() {
            self.figura.1 = Some(p.to_vec());
        }
        let (a, b) = &self.figura;
        assert!(a.is_some() && a == b, "{caso}: a tela ≠ a do oráculo");
        let bits = |t: &PainterTool| {
            t.impasto_gpu_planes().map(|p| {
                (
                    p.relief.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                    p.cover,
                )
            })
        };
        let o = bits(&self.oraculo);
        assert!(
            bits(&self.cpu) == o,
            "{caso}: o relevo da cura ≠ o do oráculo"
        );
    }

    fn traco(&mut self, caso: &str, y: f32) {
        self.em(caso, &|t| {
            t.on_canvas_pointer(ponteiro(200.0, y, PointerPhase::Down));
        });
        for k in 1..=6 {
            let x = 200.0 + 6.0 * k as f32;
            self.em(caso, &|t| {
                t.on_canvas_pointer(ponteiro(x, y, PointerPhase::Move));
            });
        }
        self.em(caso, &|t| {
            t.on_canvas_pointer(ponteiro(240.0, y, PointerPhase::Up));
        });
        self.confere(caso);
    }
}

fn corre(novo: fn() -> PainterTool, controlo: bool) {
    corre_com(novo, controlo, false);
}

fn corre_com(novo: fn() -> PainterTool, controlo: bool, com_undo: bool) {
    let fixo = || {
        let mut t = novo();
        t.set_wet_relogio_fixo(true); // a mesma tinta nos três gémeos
        t
    };
    let mut g = Gemeos {
        placa: (fixo(), Placa::default()),
        cpu: fixo(),
        oraculo: fixo(),
        controlo,
        figura: (None, None),
    };
    g.confere("o documento");
    let base = g.cpu.layers.active().expect("a base");
    g.traco("base 1.º", 100.0);
    g.traco("base 2.º", 160.0);
    g.em("o «+»", &|t| {
        let id = t.add_raster_layer("nova").expect("a camada nova");
        t.select_layer(id);
    });
    g.confere("o «+»");
    let nova = g.cpu.layers.active().expect("a nova");
    g.traco("nova 1.º", 260.0);
    g.traco("nova 2.º", 320.0);
    g.em("a troca", &|t| {
        t.select_layer(base);
    });
    g.traco("baixo depois da troca", 380.0);
    g.em("de volta", &|t| {
        t.select_layer(nova);
    });
    g.traco("nova de volta", 420.0);
    if !com_undo {
        return;
    }
    for k in 0..4 {
        g.em("undo", &|t| {
            t.undo_last();
        });
        g.confere(&format!("undo {k}"));
    }
    for k in 0..4 {
        g.em("redo", &|t| {
            t.redo_last();
        });
        g.confere(&format!("redo {k}"));
    }
}

fn sprite_crua() -> PainterTool {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; 640 * 480 * 4], 640, 480);
    t.set_paint_tool_mode("wetpaint");
    t.set_brush_size_px(20.0);
    t.set_substrate_depth(0.5);
    t
}

#[test]
fn a_marca_molhada_confinada_da_o_bit_do_caminho_antigo_no_papel_do_dono() {
    corre(no_papel_do_dono, false);
}

#[test]
fn controlo_os_gemeos_pelo_caminho_antigo_dao_o_mesmo_bit() {
    corre(no_papel_do_dono, true);
    corre(sprite_crua, true);
}

#[test]
fn a_marca_molhada_confinada_da_o_bit_do_caminho_antigo_na_sprite_crua() {
    corre(sprite_crua, false);
}

/// O undo/redo depois do «+» e dos traços (B18.5: o acordar da adormecida espera o fio no relógio fixo).
#[test]
fn o_undo_e_o_redo_dao_o_bit_do_caminho_antigo() {
    corre_com(sprite_crua, true, true);
    corre_com(no_papel_do_dono, true, true);
    corre_com(sprite_crua, false, true);
    corre_com(no_papel_do_dono, false, true);
}
