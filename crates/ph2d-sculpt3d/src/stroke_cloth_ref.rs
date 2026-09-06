//! ⭐⭐⭐ **A LEI DA REFERÊNCIA no pincel** — o adaptador entre a malha, o
//! [`Brush`]/[`Dab`] da casa e o gesto puro de [`ph2d_cloth::verlet_gesto`].
//!
//! ⚠️ **Ele nasce DESLIGADO** (`PH2D_CLOTH_LAW=ref` liga) e o caminho de omissão
//! continua a ser o VBD de [`super::stroke_cloth`]: a paridade com o oráculo
//! está medida a meio (os seis traços de um passo de força ao bit; a amplitude
//! Local/Dinâmica por resolver com o especificador — ver o INBOX do clean-room).
//! *Ligar por omissão antes de o gate de paridade existir seria trocar um pincel
//! reprovado por outro sem régua.*
//!
//! # O que este ficheiro traduz, e só isso
//!
//! - **a malha** → posições em `f64`, normais actuais, o anel-1 de cada vértice
//!   (pelas ARESTAS das faces poligonais, como o alvo — confirmado pelo
//!   especificador em 06/09; `PH2D_CLOTH_ANEL=tri` bissecta) e a máscara;
//! - **o pincel** → o [`Pincel`] da lei: raio, força, curva, dureza, modo, área;
//! - **o dab** → o [`Passo`]: o cursor, o delta, a normal da área (a média das
//!   normais sob o pincel) e a pressão.
//!
//! A lei em si (a área, a banda, as forças, as âncoras, o solver) vive na
//! `ph2d-cloth`, gateada contra as 46 fixtures do oráculo — nada dela é
//! reescrito aqui.

use crate::{Brush, Dab, Falloff, SculptStroke, Symmetry};
use ph2d_cloth::V3;
use ph2d_cloth::verlet::norm;
use ph2d_cloth::verlet_gesto::{Area, Curva, FalloffForca, Modo, Passo, Pincel, PincelTecido};
use ph2d_mesh::Mesh;

/// **A sessão da lei da referência de UMA cópia de simetria, num traço.**
#[derive(Clone, Debug)]
pub(super) struct ClothRef {
    tecido: PincelTecido,
}

/// A lei em vigor é a da referência? (`PH2D_CLOTH_LAW=ref`) — lido UMA vez.
pub(super) fn lei_referencia() -> bool {
    static ESCOLHA: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ESCOLHA.get_or_init(|| std::env::var("PH2D_CLOTH_LAW").as_deref() == Ok("ref"))
}

/// **O cursor deste passo tem de ser RE-APANHADO na superfície?** (espec §4.3:
/// os modos de força e o Expand re-picam a cada passo; o Grab fica no pen-down
/// e o Snake Hook anda no plano de profundidade.) A shell pergunta isto para
/// escolher entre o passo re-picado e o `hook_step` de sempre.
#[must_use]
pub fn cloth_repica() -> bool {
    lei_referencia()
        && matches!(
            modo_escolhido(),
            Modo::Arrastar
                | Modo::Empurrar
                | Modo::ApertarPonto
                | Modo::ApertarLinha
                | Modo::Inflar
                | Modo::Expandir
        )
}

/// O modo de deformação (`PH2D_CLOTH_DEFORM`), enquanto a W10c não dá o chip.
fn modo_escolhido() -> Modo {
    static ESCOLHA: std::sync::OnceLock<Modo> = std::sync::OnceLock::new();
    *ESCOLHA.get_or_init(|| match std::env::var("PH2D_CLOTH_DEFORM").as_deref() {
        Ok("push" | "empurrar") => Modo::Empurrar,
        Ok("point" | "pinch" | "apertar") => Modo::ApertarPonto,
        Ok("axis" | "pinch_line" | "linha") => Modo::ApertarLinha,
        Ok("normal" | "inflate" | "inflar") => Modo::Inflar,
        Ok("grab" | "agarrar") => Modo::Agarrar,
        Ok("hook" | "gancho") => Modo::Gancho,
        Ok("expand" | "expandir") => Modo::Expandir,
        _ => Modo::Arrastar,
    })
}

/// A área (`PH2D_CLOTH_AREA`): a omissão é a dos 13 presets do alvo, *Dynamic*.
fn area_escolhida() -> Area {
    static ESCOLHA: std::sync::OnceLock<Area> = std::sync::OnceLock::new();
    *ESCOLHA.get_or_init(|| match std::env::var("PH2D_CLOTH_AREA").as_deref() {
        Ok("local") => Area::Local,
        Ok("global") => Area::Global,
        _ => Area::Dinamica,
    })
}

/// O anel-1 vem da TRIANGULAÇÃO, em vez das arestas dos polígonos?
///
/// ⚠️ **A omissão é ARESTAS, e mudou em 06/09 por resposta do especificador:** o
/// alvo toma os vizinhos pelas faces POLIGONAIS (um quad interior tem 4, e as
/// diagonais entram só como restrições de PAR). A grelha triangulada tinha
/// casado o `Local` do oráculo por rigidez a mais, não por ser o mecanismo dele.
/// `PH2D_CLOTH_ANEL=tri` fica como bissecção.
fn anel_por_triangulacao() -> bool {
    static ESCOLHA: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ESCOLHA.get_or_init(|| std::env::var("PH2D_CLOTH_ANEL").as_deref() == Ok("tri"))
}

/// O anel-1 de `v` pelas ARESTAS das faces (ou pela triangulação, para bissecar).
fn anel_de(mesh: &Mesh, v: u32) -> Vec<u32> {
    let adj = mesh.adjacency();
    if !anel_por_triangulacao() {
        return adj.vert_verts.neighbours(v as usize).to_vec();
    }
    let mut out = Vec::new();
    for &f in adj.vert_faces.neighbours(v as usize) {
        let face = &mesh.faces()[f as usize];
        for k in 0..face.tri_count() {
            let t = face.tri_at(k);
            if t.contains(&v) {
                out.extend(t.iter().copied().filter(|w| *w != v));
            }
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// O [`Pincel`] da lei, derivado do [`Brush`] da casa.
///
/// ⚠️ **`passagens` são as cópias de SIMETRIA do traço**, e a área *Local*
/// constrói a lista de restrições `passagens + 1` vezes (espec, emenda Q8) — é
/// daí que sai a rigidez que separa a *Local* da *Global*. ⚠️ As fixtures do
/// oráculo correm todas sem simetria (`1`), então a lei `n + 1` vem da espec e
/// **só o degrau `n = 1` está medido aqui**.
///
/// ⚠️ A força é o `strength` CRU (a lei eleva-o ao quadrado, espec §4.1), e não
/// o [`Brush::weight`] com a curva de força do modo — essa curva é de outros
/// verbos. A curva de queda mapeia o que existe dos dois lados; o resto cai na
/// `Suave`, que é a omissão do alvo.
fn pincel_de(brush: &Brush, passagens: u32) -> Pincel {
    Pincel {
        modo: modo_escolhido(),
        area: area_escolhida(),
        falloff_forca: FalloffForca::Radial,
        curva: match brush.falloff {
            Falloff::Constant => Curva::Constante,
            Falloff::Sharper => Curva::Aguda,
            _ => Curva::Suave,
        },
        raio: f64::from(brush.radius),
        forca: f64::from(brush.strength.clamp(0.0, 1.0)),
        dureza: f64::from(brush.hardness.clamp(0.0, 1.0)),
        flip: if brush.invert { -1.0 } else { 1.0 },
        passagens,
        ..Pincel::default()
    }
}

fn v3(p: [f32; 3]) -> V3 {
    [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])]
}

impl SculptStroke {
    /// **O DAB pela lei da referência** — a porta que [`super::stroke_cloth`]
    /// desvia para cá quando [`lei_referencia`] está ligada.
    pub(super) fn cloth_ref_dab(
        &mut self,
        mesh: &mut Mesh,
        brush: &Brush,
        dab: &Dab,
        sym: Symmetry,
    ) -> usize {
        let (signs, n) = sym.signs();
        self.moved.clear();
        for (copy, s) in signs.iter().take(n).enumerate() {
            let center = [
                dab.center[0] * s[0],
                dab.center[1] * s[1],
                dab.center[2] * s[2],
            ];
            let path = [
                f64::from(dab.path[0] * s[0]),
                f64::from(dab.path[1] * s[1]),
                f64::from(dab.path[2] * s[2]),
            ];
            self.cloth_ref_copy(mesh, brush, dab, center, path, copy, u32::try_from(n).unwrap_or(1));
        }
        // O tecido move geometria — a janela de upload é a das posições.
        self.last_paints_mask = false;
        if self.moved.is_empty() {
            return 0;
        }
        mesh.refresh_region(&self.moved, &mut self.region);
        self.moved.len()
    }

    /// Uma cópia de simetria: a sessão nasce no 1.º dab, corre um passo, escreve.
    fn cloth_ref_copy(
        &mut self,
        mesh: &mut Mesh,
        brush: &Brush,
        dab: &Dab,
        center: [f32; 3],
        path: V3,
        copy: usize,
        passagens: u32,
    ) {
        if self.cloth_ref.len() <= copy {
            self.cloth_ref.resize_with(copy + 1, || None);
        }
        let cursor = v3(center);
        let pos: Vec<V3> = mesh.positions().iter().map(|p| v3(*p)).collect();
        if self.cloth_ref[copy].is_none() {
            let mut tecido = PincelTecido::pen_down(pincel_de(brush, passagens), &pos, cursor);
            // A máscara da casa, pela porta de sempre: `1` = protegido.
            if let Some(masks) = mesh.masks() {
                tecido.mascara = masks
                    .iter()
                    .map(|m| 1.0 - f64::from(crate::mask_ops::free_weight(*m)))
                    .collect();
            }
            self.cloth_ref[copy] = Some(ClothRef { tecido });
        }
        let Some(mut ses) = self.cloth_ref[copy].take() else {
            return;
        };

        // As normais ACTUAIS (o Inflate lê-as) e a normal da ÁREA (a média sob o
        // pincel — espec §4.4).
        let normais: Vec<V3> = mesh.normals().iter().map(|n| v3(*n)).collect();
        mesh.verts_in_sphere(center, dab.radius, &mut self.query, &mut self.footprint);
        let mut normal_area = [0.0f64; 3];
        for v in &self.footprint {
            let n = normais[*v as usize];
            normal_area[0] += n[0];
            normal_area[1] += n[1];
            normal_area[2] += n[2];
        }
        let passo = Passo {
            cursor,
            delta: path,
            parado: norm(path) == 0.0,
            normal_area,
            normais: &normais,
            pressao: f64::from(dab.pressure.clamp(0.0, 1.0)),
        };
        let simulou = {
            let anel = |v: u32| anel_de(mesh, v);
            ses.tecido.passo(&pos, &anel, &passo)
        };
        if simulou {
            // ⚠️ Todo vértice ACTIVO é capturado antes de ser escrito: o `pre` é
            // o que o undo devolve (a mesma lei do `build_cloth` do VBD).
            for v in 0..ses.tecido.sim.activo.len() {
                if ses.tecido.sim.activo[v] {
                    self.capture(mesh, u32::try_from(v).unwrap_or(u32::MAX));
                }
            }
            let out = mesh.positions_mut();
            for (v, act) in ses.tecido.sim.activo.iter().enumerate() {
                if !*act {
                    continue;
                }
                let p = ses.tecido.sim.x[v];
                let novo = [p[0] as f32, p[1] as f32, p[2] as f32];
                if out[v] != novo {
                    out[v] = novo;
                    self.moved.push(u32::try_from(v).unwrap_or(u32::MAX));
                }
            }
        }
        self.cloth_ref[copy] = Some(ses);
    }
}
