//! A PONTE da cena `PH2D_VEC_BONE_SMOKE` — o prólogo, e só ele.
//!
//! ⭐ **Os dois tempos e o roteador vivem em [`ph2d_app_vec::smoke_bone`]** (W2 Fase B, 2.ª volta).
//! O que fica aqui é a máquina de dois tempos, porque ela pergunta ao `gfx` e ao mapa de entidades
//! da shell — *o que sai são os CORPOS; o que decide a ordem do quadro fica* (HOWTO §4).

// ⭐ Os quatro nomes que ATRAVESSAM: o `bone_smart_probe` (um diagnóstico da shell) lê-os.
// ⚠️ São exactamente os que atravessam, e nada mais — abrir a mais paga `dead_code` do outro lado
// (HOWTO §2.5).
#[allow(
    unused_imports,
    reason = "quatro destes oito só atravessam em `cfg(test)`; o build do binário vê-os por usar"
)]
pub(crate) use ph2d_skeleton_demo::{
    ARM_A, ARM_B, ARM_BONES, ARM_ELBOW_BEND, DEMO_ACTION, TENTACLE_LIMIT_HALF, cadeia,
    seed_demo_action,
};

impl crate::App {
    /// No prólogo do frame. No-op sem a env.
    pub(crate) fn vec_bone_smoke(&mut self) {
        if !ph2d_app_vec::smoke_bone::armed() || self.gfx.is_none() {
            return;
        }
        // O `ppm` do projecto, UMA vez: o 1.º tempo mede a imagem com ele e o 2.º prende-a com ele
        // — dois valores dariam à régua da imagem duas âncoras.
        let ppm = self
            .gfx
            .as_ref()
            .and_then(|gfx| gfx.hero_screen.as_ref())
            .map_or(ph2d_editor_core::DEFAULT_PIXELS_PER_METER, |h| {
                h.project.pixels_per_meter.max(crate::EPS_PIXELS_PER_METER)
            });
        match self.vec.bone_smoke_step {
            0 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let _ = gfx
                    .tools
                    .set_active(&ph2d_editor_core::ToolId::new("vector"));
                ph2d_app_vec::smoke_bone::build(
                    &mut gfx.vec_scene,
                    &mut gfx.sim,
                    &mut gfx.renderer,
                    &mut gfx.asset_db,
                    ppm,
                    &mut self.vec,
                );
            }
            // ⚠️⚠️ **NÃO se conta QUADROS aqui, pergunta-se o FATO.** Prender exige a ENTIDADE de
            // cada forma, e quem a cria (`vec_entities::sync`) corre no MEIO do quadro — que um
            // quadro inicial pode nunca alcançar (superfície ainda por configurar). Um contador
            // acertaria na máquina que testou e prenderia **zero** noutra, em silêncio, e o
            // sintoma seria exactamente *"nenhuma forma pode ser deformada"*.
            1 => {
                let prontas =
                    self.vec.bone_smoke_pend.as_ref().is_some_and(|p| {
                        p.iter().all(|(id, _)| self.vec.entities.contains_key(id))
                    });
                if prontas {
                    let gfx = self.gfx.as_mut().expect("gfx");
                    ph2d_app_vec::smoke_bone::bind(
                        &mut gfx.vec_scene,
                        &mut gfx.sim,
                        &mut self.timeline.doc,
                        &gfx.asset_db,
                        ppm,
                        &mut self.vec,
                    );
                    // ⭐⭐⭐ **O PRÓLOGO DA CENA DO ENVELOPE, e ele corre AQUI e não no 1.º tempo.**
                    //
                    // ⚠️ **Enquadrar antes de prender enquadraria a arte de REPOUSO**, que é recta
                    // — e a cena abre DOBRADA. *O que o `Frame All` tem de medir é o que o dono vai
                    // ver.*
                    //
                    // ⚠️ **A DECISÃO é da cena** (`smoke_bone_envelope::prologo`, gateada sem janela
                    // nenhuma); o que mora aqui é o EFEITO, porque fechar um painel e empurrar uma
                    // acção no barramento são duas coisas da `App`. *O molde é o da física: o que
                    // sai são os CORPOS; o que decide a ordem do quadro fica.*
                    {
                        let pro = ph2d_app_vec::smoke_bone_envelope::prologo_do_nivel(
                            ph2d_app_vec::smoke_bone::nivel(),
                        );
                        if let Some(hero) = gfx.hero_screen.as_mut() {
                            if pro.timeline_fechada {
                                <_ as ph2d_editor_core::panel::PanelHostInternal>::set_panel_visible(
                                    hero,
                                    <ph2d_panel_timeline::TimelinePanel as ph2d_editor_core::panel::Panel>::ID,
                                    false,
                                );
                            }
                            if pro.enquadrar {
                                hero.bus.push(
                                    ph2d_editor_core::action_bus::EditorAction::SetViewFocus {
                                        kind: ph2d_editor_core::ViewFocusKind::All,
                                    },
                                );
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
