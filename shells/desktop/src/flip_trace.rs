//! **Shift & Trace** (`docs/Flip/04 §4`, OpenToonz) — o papel que desliza no lightbox.
//!
//! No modo **Trace**, arrastar no canvas DESLOCA o fantasma sob o cursor (Ctrl gira em
//! torno do centro da arte) — **só a exibição**: o desenho, a pose autorada da chave e o
//! documento nunca mudam. O animador posiciona a referência, volta ao Draw e traça com
//! ela deslocada; o **Reset Shifts** (painel) devolve tudo ao lugar.
//!
//! ## As decisões que carregam o desenho
//!
//! - **O deslocamento é POR CHAVE — a folha de papel.** `FlipStrip.trace` é um
//!   `BTreeMap<Frame, Pose>`: deslocar o fantasma do quadro 4 desloca a folha 4 em toda
//!   camada que a mostra (o mesmo escopo dos pins, também chaveados por quadro). É sessão
//!   do shell, como pins/seleção — e a MESMA porta `remap_session_*` o carrega quando a
//!   chave anda (mover a célula, esticar um hold).
//! - **O shift compõe DEPOIS da pose, ANTES do objeto** (`art_to_world_traced`): a arte
//!   entra na pose autorada da chave, o papel desliza por cima, e o objeto leva tudo ao
//!   mundo. Com shift identidade o caminho é o de sempre, byte a byte.
//! - **O hit segue o OLHO**: entre fantasmas sobrepostos ganha o de menor `|Δ|` — o
//!   render desenha do mais distante ao mais próximo, então o vizinho imediato está POR
//!   CIMA, e pegar outro seria pegar o que não se vê.
//! - **O Down consome SEMPRE no modo Trace** (mesmo errando o fantasma): deixá-lo cair
//!   adiante entregaria o clique ao pick/gizmo genérico, e o arrasto seguinte MOVERIA o
//!   objeto — a mesma razão do consumo do Edit.
//! - **O centro da rotação é ponto FIXO da própria rotação** (compor `R(c)` por cima do
//!   shift mantém `c` parado), então capturá-lo no Down é exato — não há deriva a
//!   realimentar.

use ph2d_core::Vec2;
use ph2d_flip::{Frame, Pose};
use ph2d_vec_scene::Xform;

use crate::flip_transform::key_xform;

/// O arrasto de trace em curso.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TraceDrag {
    /// A chave (folha) pega no Down.
    pub(crate) key: Frame,
    /// O ponteiro no espaço do OBJETO no último passo (o delta do move é obj-space).
    pub(crate) last_obj: Vec2,
    /// `Some(centro)` = rotação (Ctrl no Down), em torno do centro POSADO da arte.
    pub(crate) rotate: Option<Vec2>,
}

/// Um fantasma na tela, candidato ao hit.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TraceGhost {
    pub(crate) key: Frame,
    /// `|Δ|` — o desempate do hit (menor = por cima, a ordem do render).
    pub(crate) dist: u32,
    /// `pose_da_chave ∘ shift_atual` (arte local → espaço do objeto).
    pub(crate) to_object: Xform,
    /// Caixa LOCAL da arte (canto baixo / canto alto).
    pub(crate) lo: [f32; 2],
    pub(crate) hi: [f32; 2],
}

/// O índice do fantasma sob `p_obj` (espaço do objeto) — o de menor `|Δ|` entre os que
/// contêm o ponto. O teste inverte a cadeia e pergunta à caixa LOCAL (exato sob rotação;
/// uma caixa girada em mundo deixaria cantos falsos).
#[must_use]
pub(crate) fn pick(p_obj: [f64; 2], ghosts: &[TraceGhost]) -> Option<usize> {
    ghosts
        .iter()
        .enumerate()
        .filter(|(_, g)| {
            let Some(inv) = g.to_object.inverse() else {
                return false; // pose degenerada: não há "dentro" para acertar
            };
            let l = inv.apply(p_obj);
            (l[0] as f32) >= g.lo[0]
                && (l[0] as f32) <= g.hi[0]
                && (l[1] as f32) >= g.lo[1]
                && (l[1] as f32) <= g.hi[1]
        })
        .min_by_key(|(_, g)| g.dist)
        .map(|(i, _)| i)
}

/// Compõe uma rotação de `da` radianos em torno de `c` (espaço do objeto) POR CIMA do
/// shift atual — o papel gira onde está, sem transladar.
#[must_use]
pub(crate) fn rotated(shift: Pose, c: Vec2, da: f32) -> Pose {
    let (sin, cos) = f64::from(da).sin_cos();
    let (cx, cy) = (f64::from(c.x), f64::from(c.y));
    // R em torno de c: gira e devolve o centro ao lugar (T(c)·R·T(−c)).
    let rot = Xform([
        cos,
        sin,
        -sin,
        cos,
        cx - (cos * cx - sin * cy),
        cy - (sin * cx + cos * cy),
    ]);
    let x = key_xform(shift).then(&rot);
    Pose::from_coeffs([
        x.0[0] as f32,
        x.0[1] as f32,
        x.0[2] as f32,
        x.0[3] as f32,
        x.0[4] as f32,
        x.0[5] as f32,
    ])
}

impl crate::App {
    /// A tool Flip quer o canvas para DESLOCAR fantasmas agora? (ativa + modo Trace.)
    #[must_use]
    pub(crate) fn flip_wants_trace(&self) -> bool {
        self.flip_active
            && matches!(
                self.flip_style.map(|s| s.mode),
                Some(ph2d_tool_flip::FlipMode::Trace)
            )
    }

    /// Os fantasmas da camada ativa AGORA, com o shift que já têm — os candidatos ao hit.
    /// Espelha os gates do passe (`flip_pass_ghosts::collect`): o que não é desenhado não
    /// pode ser pego.
    fn trace_candidates(&self) -> Vec<TraceGhost> {
        let Some(gfx) = self.gfx.as_ref() else {
            return Vec::new();
        };
        let Some((oid, lid)) = crate::flip_strip_resolve::target(&gfx.flip, self.flip_active_layer)
        else {
            return Vec::new();
        };
        let Some(obj) = gfx.flip.object(oid) else {
            return Vec::new();
        };
        let Some(layer) = obj.layer(lid) else {
            return Vec::new();
        };
        if self.playhead.is_playing() || !obj.onion.enabled || !layer.use_onion || !layer.visible {
            return Vec::new();
        }
        let src = layer.source_frame(obj.frame_at(&self.playhead));
        ph2d_flip::ghosts(
            layer,
            src,
            &obj.onion,
            self.flip_strip.selected_keys(),
            self.flip_strip.pinned_keys(),
        )
        .into_iter()
        .filter_map(|g| {
            let art = obj.drawing(g.drawing)?;
            let (c, h) = crate::flip_pose_gizmo::drawing_center_half(art)?;
            let shift = self
                .flip_strip
                .trace
                .get(&g.key)
                .copied()
                .unwrap_or_default();
            Some(TraceGhost {
                key: g.key,
                dist: g.delta.unsigned_abs(),
                to_object: key_xform(layer.frame_pose(g.key)).then(&key_xform(shift)),
                lo: [c[0] - h[0], c[1] - h[1]],
                hi: [c[0] + h[0], c[1] + h[1]],
            })
        })
        .collect()
    }

    /// O ponteiro (tela) no espaço do OBJETO do Flip ativo.
    fn trace_pointer_obj(&self, x: f32, y: f32) -> Option<Vec2> {
        let gfx = self.gfx.as_ref()?;
        let win = gfx.surface.size();
        let world = gfx.camera.screen_to_world((x, y), win);
        let p = self
            .flip_active_world_to_object()
            .apply([f64::from(world[0]), f64::from(world[1])]);
        Some(Vec2::new(p[0] as f32, p[1] as f32))
    }

    /// Pen-down no modo Trace: pega o fantasma sob o cursor (Ctrl = girar). Devolve
    /// `true` SEMPRE que o modo está ativo — o Trace é dono do canvas (ver doc do módulo).
    pub(crate) fn flip_trace_canvas_down(&mut self, x: f32, y: f32) -> bool {
        if !self.flip_wants_trace() {
            return false;
        }
        let Some(p) = self.trace_pointer_obj(x, y) else {
            return false;
        };
        let ghosts = self.trace_candidates();
        if let Some(i) = pick([f64::from(p.x), f64::from(p.y)], &ghosts) {
            let g = ghosts[i];
            let rotate = self.modifiers.control_key().then(|| {
                let c = g.to_object.apply([
                    f64::from((g.lo[0] + g.hi[0]) * 0.5),
                    f64::from((g.lo[1] + g.hi[1]) * 0.5),
                ]);
                Vec2::new(c[0] as f32, c[1] as f32)
            });
            self.flip_trace_drag = Some(TraceDrag {
                key: g.key,
                last_obj: p,
                rotate,
            });
        }
        true
    }

    /// Movimento com arrasto de trace aberto: translada (ou gira) a folha da chave.
    /// No-op sem gesto.
    pub(crate) fn flip_trace_canvas_move(&mut self, x: f32, y: f32) -> bool {
        if self.flip_trace_drag.is_none() {
            return false;
        }
        // O modo trocou sob o gesto (atalho/painel): largar é mais honesto que continuar
        // deslocando um fantasma que o modo novo nem mostra como alvo.
        if !self.flip_wants_trace() {
            self.flip_trace_drag = None;
            return false;
        }
        let Some(p) = self.trace_pointer_obj(x, y) else {
            return false;
        };
        let Some(mut d) = self.flip_trace_drag else {
            return false;
        };
        let cur = self
            .flip_strip
            .trace
            .get(&d.key)
            .copied()
            .unwrap_or_default();
        let next = match d.rotate {
            None => {
                let mut s = cur;
                s.translate(p - d.last_obj);
                s
            }
            Some(c) => {
                let a0 = (d.last_obj.y - c.y).atan2(d.last_obj.x - c.x);
                let a1 = (p.y - c.y).atan2(p.x - c.x);
                rotated(cur, c, a1 - a0)
            }
        };
        self.flip_strip.trace.insert(d.key, next);
        d.last_obj = p;
        self.flip_trace_drag = Some(d);
        true
    }

    /// Pen-up: fecha o arrasto de trace. Devolve `true` se havia um.
    pub(crate) fn flip_trace_canvas_up(&mut self) -> bool {
        self.flip_trace_drag.take().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    fn ghost(key: Frame, dist: u32, to_object: Xform) -> TraceGhost {
        TraceGhost {
            key,
            dist,
            to_object,
            lo: [-1.0, -1.0],
            hi: [1.0, 1.0],
        }
    }

    const ID: Xform = Xform([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

    /// 🔴 **Entre fantasmas sobrepostos ganha o de menor `|Δ|`** — o render desenha do
    /// mais distante ao mais próximo, então é ESSE que o olho vê por cima, e o hit tem
    /// de concordar com o olho. Mutação que sangra: `max_by_key` (pegaria o de baixo).
    #[test]
    fn the_pick_takes_the_ghost_the_eye_sees_on_top() {
        let far = ghost(0, 5, ID);
        let near = ghost(8, 1, ID);
        assert_eq!(pick([0.0, 0.0], &[far, near]), Some(1));
        assert_eq!(
            pick([9.0, 9.0], &[far, near]),
            None,
            "fora das caixas: nada"
        );
    }

    /// 🔴 **Uma folha JÁ deslocada é pega onde ESTÁ** — o hit pergunta à caixa posada
    /// (pose ∘ shift), não a onde o desenho nasceu. Sem isto, o segundo ajuste da mesma
    /// folha exigiria clicar no lugar VAZIO de onde ela saiu.
    #[test]
    fn a_shifted_sheet_is_picked_where_it_is() {
        let there = key_xform(Pose::from_translation(Vec2::new(10.0, 0.0)));
        let g = ghost(4, 1, there);
        assert_eq!(pick([10.0, 0.0], &[g]), Some(0));
        assert_eq!(
            pick([0.0, 0.0], &[g]),
            None,
            "onde o desenho nasceu nao ha mais folha"
        );
    }

    /// 🔴 **A rotação gira em torno do centro dado** — o centro é ponto FIXO (é o que
    /// permite capturá-lo no Down sem deriva), e um ponto a leste do centro vai para o
    /// norte com `+90°`. Mutação que sangra: girar em torno da ORIGEM (o centro voa).
    #[test]
    fn the_rotation_pivots_on_the_given_centre() {
        let c = Vec2::new(5.0, 3.0);
        let r = rotated(Pose::IDENTITY, c, FRAC_PI_2);
        let rc = r.apply(c);
        assert!(
            (rc.x - c.x).abs() < 1e-4 && (rc.y - c.y).abs() < 1e-4,
            "o centro nao se move: {rc:?}"
        );
        let east = r.apply(Vec2::new(6.0, 3.0));
        assert!(
            (east.x - 5.0).abs() < 1e-4 && (east.y - 4.0).abs() < 1e-4,
            "leste tinha de virar norte: {east:?}"
        );
    }

    /// A rotação compõe POR CIMA do shift atual: primeiro a folha desliza, depois gira
    /// onde está — a ordem inversa giraria em torno de um lugar onde ela não está mais.
    #[test]
    fn rotation_composes_on_top_of_the_current_shift() {
        // shift = T(4,0): a origem local está em (4,0). Girar +90° em torno de (4,0)
        // mantém a origem lá e leva o ponto local (1,0) — que estava em (5,0) — a (4,1).
        let s = Pose::from_translation(Vec2::new(4.0, 0.0));
        let r = rotated(s, Vec2::new(4.0, 0.0), FRAC_PI_2);
        let o = r.apply(Vec2::new(0.0, 0.0));
        let e = r.apply(Vec2::new(1.0, 0.0));
        assert!(
            (o.x - 4.0).abs() < 1e-4 && o.y.abs() < 1e-4,
            "origem: {o:?}"
        );
        assert!(
            (e.x - 4.0).abs() < 1e-4 && (e.y - 1.0).abs() < 1e-4,
            "leste: {e:?}"
        );
    }
}
