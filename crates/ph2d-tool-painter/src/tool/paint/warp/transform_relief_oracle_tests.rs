//! B06.A2 — o oráculo do corpo do Transform: o laço serial de antes (três planos do tamanho do sujo
//! por movimento, depois copiados), guardado verbatim, e o gate de paridade ao bit das faixas.

use super::super::transform_modes_tests as fx;
use super::*;
use std::cell::Cell;

thread_local! {
    static ORACULO: Cell<bool> = const { Cell::new(false) };
    static FAIXAS: Cell<Option<usize>> = const { Cell::new(None) };
}

pub(super) fn pelo_oraculo() -> bool {
    ORACULO.with(Cell::get)
}

pub(super) fn faixas_forcadas() -> Option<usize> {
    FAIXAS.with(Cell::get)
}

impl PainterTool {
    /// O laço SERIAL de antes do B06.A2, verbatim: o oráculo ao bit das faixas.
    pub(super) fn compose_floating_relief_serial(
        &mut self,
        dirty: Region,
        locate: &(dyn Fn(u32, u32) -> Option<[f32; 2]> + Sync),
    ) {
        if !self.paint.warp.affect_relief {
            return;
        }
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        let Some(fr) = self
            .paint
            .deform
            .xform_patch
            .as_ref()
            .and_then(|p| p.relief.clone())
        else {
            return;
        };
        // Bits de camada são reciclados: o corpo congelado no levante não pode ser vestido por outra.
        if self.layers.active() != Some(fr.layer) || fr.h.len() != n {
            return;
        }

        // Uma passada, três planos: a cobertura composta é o PESO das outras duas, então computá-la
        // aqui e reusá-la é o que impede as três de discordarem sobre quanta tinta há num texel.
        let mut out_h = vec![0.0f32; (dirty.w * dirty.h) as usize];
        let mut out_c = vec![0u8; (dirty.w * dirty.h) as usize];
        let mut out_m = vec![[0u8; 7]; (dirty.w * dirty.h) as usize];
        for ry in 0..dirty.h {
            let dy = dirty.y + ry;
            for rx in 0..dirty.w {
                let dx = dirty.x + rx;
                let gi = (dy * w + dx) as usize;
                let oi = (ry * dirty.w + rx) as usize;
                // O lado da BASE: o que ficou para trás quando o patch subiu.
                let cb = f32::from(fr.c_base.as_ref().map_or(0, |v| v[gi])) / 255.0;
                let (hb, mb) = (fr.h[gi], fr.m[gi]);
                let (cp, hp, mp) = if let Some(sp) = locate(dx, dy) {
                    (
                        f32::from(bilinear_u8(&fr.c_patch, w, h, sp[0], sp[1])) / 255.0,
                        bilinear_f32(&fr.h, w, h, sp[0], sp[1]),
                        bilinear_mat(&fr.m, w, h, sp[0], sp[1]),
                    )
                } else {
                    (0.0, 0.0, [0u8; 7])
                };
                // `over`, com a COBERTURA no lugar do alfa — a mesma lei que a cor usa uma linha ao lado.
                let back = cb * (1.0 - cp);
                let total = cp + back;
                out_c[oi] = (total * 255.0).round().clamp(0.0, 255.0) as u8;
                if total > 0.0 {
                    out_h[oi] = ((hp * cp + hb * back) / total).clamp(-H_MAX, H_MAX);
                    for c in 0..7 {
                        out_m[oi][c] = ((f32::from(mp[c]) * cp + f32::from(mb[c]) * back) / total)
                            .round()
                            .clamp(0.0, 255.0) as u8;
                    }
                }
                // …e onde não sobrou tinta nenhuma o texel fica em zero: cobertura zero é *não há tinta
                // aqui*, e uma altura órfã ali é um número que a luz não pesa e que o próximo verbo lê.
            }
        }

        let layer = fr.layer;
        let area = Some(dirty);
        if let Some(entry) = self.heights.get_mut(&layer)
            && entry.len() == n
        {
            let dst = crate::tool::paint::plane_fork::fork_heights(
                entry,
                &self.undo.write_state,
                layer,
                (w, h),
                area,
            );
            blit(dst, &out_h, dirty, w);
        }
        if let Some(entry) = self.covers.get_mut(&layer)
            && entry.len() == n
        {
            let dst = crate::tool::paint::plane_fork::fork_covers(
                entry,
                &self.undo.write_state,
                layer,
                (w, h),
                area,
            );
            blit(dst, &out_c, dirty, w);
        }
        if let Some(entry) = self.mats.get_mut(&layer)
            && entry.len() == n
        {
            let dst = crate::tool::paint::plane_fork::fork_mats(
                entry,
                &self.undo.write_state,
                layer,
                (w, h),
                area,
            );
            blit(dst, &out_m, dirty, w);
        }
        // A luz lê a VIZINHANÇA de um texel (a normal é diferença central), então um texel logo fora da
        // caixa é iluminado por uma inclinação que mudou dentro dela — o mesmo crescimento de 1 que o
        // `warp_render_relief` faz pelo mesmo motivo.
        if let Some(g) = crate::tool::paint::region::grow_region(dirty, 1, w, h) {
            self.mark_dirty(g);
        }
    }
}

/// Copiar um bloco `dirty`-shaped de volta para o plano canvas-shaped.
fn blit<T: Copy>(dst: &mut [T], src: &[T], dirty: Region, w: u32) {
    for ry in 0..dirty.h {
        let dy = dirty.y + ry;
        let row = ((dy * w + dirty.x) as usize)..((dy * w + dirty.x + dirty.w) as usize);
        let sr = ((ry * dirty.w) as usize)..(((ry + 1) * dirty.w) as usize);
        dst[row].copy_from_slice(&src[sr]);
    }
}

/// Os três planos depois de um gesto do Transform, pelo caminho escolhido.
fn planos(
    oraculo: bool,
    faixas: Option<usize>,
    gesto: &dyn Fn(&mut PainterTool),
) -> (Vec<u32>, Vec<u8>, Vec<MaterialBytes>) {
    ORACULO.with(|c| c.set(oraculo));
    FAIXAS.with(|c| c.set(faixas));
    let (mut t, layer) = fx::relief_in_transform();
    gesto(&mut t);
    ORACULO.with(|c| c.set(false));
    FAIXAS.with(|c| c.set(None));
    let h = t
        .heights
        .get(&layer)
        .expect("planos")
        .iter()
        .map(|v| v.to_bits())
        .collect();
    (
        h,
        t.covers.get(&layer).expect("planos").to_vec(),
        t.mats.get(&layer).expect("planos").to_vec(),
    )
}

/// **As faixas dão o corpo do laço serial AO BIT**, em 1, 3 e 7 faixas, nos três sub-modos (Uniform,
/// Distort, Warp em malha: os dois chamadores de `compose_floating_relief`).
///
/// Mutação que tem de sangrar: `li = (ry * w + dx)` → `(ry * w + rx)` na faixa.
#[test]
fn o_corpo_do_transform_em_faixas_e_o_serial_ao_bit() {
    /// `(modo, índice, com seleção, de, para)`.
    type Caso = (&'static str, u8, bool, [f32; 2], [f32; 2]);
    let w3 = fx::W as f32 / 3.0;
    // `sel`: re-levantar com uma seleção que tira a janela suja da coluna 0 (`dirty.x > 0`).
    let cases: [Caso; 5] = [
        ("Uniform", 0, false, [80.0, 80.0], [100.0, 86.0]),
        ("Distort", 2, false, [0.0, 0.0], [30.0, 24.0]),
        ("Warp", 3, false, [w3, w3], [w3 + 18.0, w3 + 22.0]),
        ("Uniform+sel", 0, true, [80.0, 80.0], [100.0, 86.0]),
        ("Distort+sel", 2, true, [40.0, 60.0], [56.0, 70.0]),
    ];
    for (name, mode, sel, from, to) in cases {
        let gesto = |t: &mut PainterTool| {
            if sel {
                t.set_rect_selection(40, 60, 90, 44);
                t.deform_apply();
                let src = t.paint.deform.xform_patch.as_ref().expect("levante").src;
                assert!(src.x > 0, "fixture [{name}]: a janela ficou na coluna 0");
            }
            t.set_deform_transform_mode(mode);
            fx::drag(t, from, to);
        };
        let oraculo = planos(true, None, &gesto);
        assert!(
            oraculo.1.iter().any(|&c| c > 0),
            "fixture [{name}]: sem corpo"
        );
        for faixas in [1, 3, 7] {
            let novo = planos(false, Some(faixas), &gesto);
            assert!(
                novo.0 == oraculo.0,
                "[{name}] {faixas} faixas: altura difere do serial"
            );
            assert!(
                novo.1 == oraculo.1,
                "[{name}] {faixas} faixas: cobertura difere do serial"
            );
            assert!(
                novo.2 == oraculo.2,
                "[{name}] {faixas} faixas: material difere do serial"
            );
        }
    }
}
