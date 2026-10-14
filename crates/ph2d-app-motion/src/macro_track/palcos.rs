//! **As macros em cena** (ADR-0186, W4 item 5.5): cada entidade com um `ph2d_ecs::Macro` cuja trilha
//! do Motion a Timeline conduz neste quadro (`ph2d_ecs::MacroTempo`) toca num palco próprio
//! ([`MacroEmCena`]) com a placa das formas dele. O editor deixa de cozer os sinks do dono
//! ([`crate::motion_objeto::tocados_por_macro`]) e o presente desenha cada palco pelas MESMAS portas
//! que desenham o editor, numa camada à parte que pousa no mundo ([`MotionFiltro::liga_em_camada`]).
//!
//! Um palco nasce quando o `Macro` muda (o tique de mudança do componente; nunca se comparam os
//! bytes por quadro) e fica em cache enquanto a entidade o tiver: um quadro sem `MacroTempo` tira-o
//! de quem toca sem o recozer do zero quando a Timeline volta a conduzi-lo.
//!
//! [`MotionFiltro::liga_em_camada`]: crate::motion_filter::MotionFiltro::liga_em_camada

use super::em_cena::MacroEmCena;
use super::tocar::{ErroDaMacro, Modo};
use crate::motion_leaf_images::Art;
use crate::motion_mundo::MundoDaCena;
use crate::motion_shape_placa::{self, PlacaDeFormas};
use crate::motion_state::MotionState;
use ph2d_ecs::{Entity, Macro, MacroTempo, ModoDaMacro, World};
use ph2d_gpu::GpuContext;
use ph2d_light2d_gpu::CamadaDeOclusao;
use ph2d_vector::Affine;
use std::collections::{BTreeMap, BTreeSet};

/// O palco de uma macro e a placa das formas dele (estado POR FONTE, como a do editor).
pub struct PalcoEmCena {
    pub em_cena: MacroEmCena,
    pub placa: PlacaDeFormas,
    /// A Timeline conduz a macro neste quadro.
    toca: bool,
}

/// O que um `Macro` deu da última vez que mudou.
struct Lido {
    tique: u32,
    /// `Ok(None)` = nada gravado, ou a gravação não tem trilha do Motion.
    palco: Result<Option<PalcoEmCena>, ErroDaMacro>,
}

/// Os palcos das macros, por entidade (bits) — vive no `MotionShellState`.
#[derive(Default)]
pub struct PalcosDasMacros {
    lidos: BTreeMap<u64, Lido>,
}

fn modo(m: ModoDaMacro) -> Modo {
    match m {
        ModoDaMacro::Fiel => Modo::Fiel,
        ModoDaMacro::Explicativo => Modo::Explicativo,
    }
}

fn palco_de(m: &Macro) -> Result<Option<PalcoEmCena>, ErroDaMacro> {
    if m.doc.is_empty() {
        return Ok(None);
    }
    Ok(
        MacroEmCena::de(&m.doc, modo(m.modo))?.map(|em_cena| PalcoEmCena {
            em_cena,
            placa: PlacaDeFormas::default(),
            toca: false,
        }),
    )
}

impl PalcosDasMacros {
    /// **Por quadro**, depois de `publica_o_mundo` e antes do `dispatch` do editor: (re)cria o palco
    /// de um `Macro` que mudou e segue o relógio de cada um que a Timeline conduz. Devolve os donos
    /// que um palco toca neste quadro (`0` = todos os objectos: uma trilha sem alvo).
    pub fn segue(
        &mut self,
        world: &mut World,
        mundo: &MundoDaCena,
        gpu: Option<&GpuContext>,
    ) -> BTreeSet<u64> {
        let mut q = world.query::<(Entity, &Macro, Option<&MacroTempo>)>();
        let vistos: Vec<(Entity, Option<f64>)> = q
            .iter(world)
            .map(|(e, _, t)| (e, t.map(|t| t.segundos)))
            .collect();
        let vivos: BTreeSet<u64> = vistos.iter().map(|(e, _)| e.to_bits()).collect();
        self.lidos.retain(|bits, _| vivos.contains(bits));
        let mut tocados = BTreeSet::new();
        for (e, segundos) in vistos {
            let tique = world
                .entity(e)
                .get_change_ticks::<Macro>()
                .map_or(0, |t| t.changed.get());
            let fresco = self
                .lidos
                .get(&e.to_bits())
                .is_none_or(|l| l.tique != tique);
            if fresco && let Some(m) = world.get::<Macro>(e) {
                let palco = palco_de(m);
                self.lidos.insert(e.to_bits(), Lido { tique, palco });
            }
            let Some(Lido {
                palco: Ok(Some(p)), ..
            }) = self.lidos.get_mut(&e.to_bits())
            else {
                continue;
            };
            p.toca = segundos.is_some();
            if let Some(s) = segundos {
                // Um passo que falha pára a macro nele e fica em `MacroEmCena::erro`: o palco
                // continua a mostrar o que tocou até lá.
                let _ = p.em_cena.segue(s, mundo, gpu);
                tocados.insert(p.em_cena.dono());
            }
        }
        // Uma mudança depois desta leitura leva um tique NOVO (a shell não corre um `Schedule` por
        // quadro, e sem isto dois quadros podiam partilhar o tique).
        world.increment_change_tick();
        tocados
    }

    fn a_tocar(&mut self) -> impl Iterator<Item = &mut PalcoEmCena> {
        self.lidos
            .values_mut()
            .filter_map(|l| l.palco.as_mut().ok().and_then(Option::as_mut))
            .filter(|p| p.toca)
    }

    /// Os palcos que tocam neste quadro, com a placa de cada um — as portas que só leem.
    pub fn fontes(&self) -> impl Iterator<Item = (&MotionState, &PlacaDeFormas)> {
        self.lidos
            .values()
            .filter_map(|l| l.palco.as_ref().ok().and_then(Option::as_ref))
            .filter(|p| p.toca)
            .map(|p| (p.em_cena.palco(), &p.placa))
    }

    /// Os mesmos, para as portas que guardam estado por quadro.
    pub fn fontes_mut(&mut self) -> impl Iterator<Item = (&mut MotionState, &mut PlacaDeFormas)> {
        self.a_tocar()
            .map(|p| (p.em_cena.palco_mut(), &mut p.placa))
    }

    /// Algum palco toca neste quadro.
    #[must_use]
    pub fn tocam(&self) -> bool {
        self.fontes().next().is_some()
    }

    /// **A placa de cada palco**, antes de o documento ser codificado — a porta do editor
    /// ([`motion_shape_placa::decide_o_quadro`]). `true` = alguma vai à placa (o documento às faixas).
    pub fn decide(&mut self, gpu: &GpuContext, ligadas: (bool, bool), cam: Affine) -> bool {
        let mut alguma = false;
        for (m, p) in self.fontes_mut() {
            alguma |= motion_shape_placa::decide_o_quadro(p, gpu, m, ligadas, cam);
        }
        alguma
    }

    /// **As formas que a placa não leva**, na cena da camada de cada palco — a porta do editor
    /// ([`crate::motion_shape_gen::encode`]).
    pub fn codifica(
        &mut self,
        art: &mut dyn FnMut(u32, [f32; 4]) -> Option<Art>,
        cam: Affine,
        janela: Option<ph2d_vector::Rect>,
        filtro: ph2d_render::ImageFilterMode,
    ) {
        for (m, p) in self.fontes_mut() {
            if p.ativa() || m.pump.vector_instances.is_empty() {
                continue;
            }
            let MotionState {
                pump,
                shape_store,
                filtro: f,
                ..
            } = m;
            let cena = f.cena_da_camada_sempre();
            let insts = &pump.vector_instances;
            crate::motion_shape_gen::encode(insts, shape_store, art, cam, janela, filtro, cena);
        }
    }

    /// **Ao planear o quadro**: cada palco desenha-se em camada, e a lente dele liga. `true` = há
    /// palcos (o quadro vai às camadas: a camada pousa no acumulador do mundo).
    pub fn liga(&mut self) -> bool {
        let mut algum = false;
        for (m, _) in self.fontes_mut() {
            m.filtro.liga_em_camada();
            m.lente.liga(true);
            algum = true;
        }
        algum
    }

    /// **A camada das formas de cada palco**, antes do halo — a porta do editor
    /// ([`motion_shape_placa::desenha_o_quadro`]).
    pub fn desenha(
        &mut self,
        gpu: &GpuContext,
        tamanho: (u32, u32),
        renderer: &ph2d_render::SpriteRenderer,
        filtro: ph2d_render::ImageFilterMode,
    ) {
        for (m, p) in self.fontes_mut() {
            let _ = motion_shape_placa::desenha_o_quadro(p, gpu, tamanho, m, renderer, filtro);
        }
    }

    /// **Cada palco pousa a sua camada no mundo** (sprites e formas, com os filtros dele), depois
    /// da do editor ([`MotionState::filtra_a_camada`]).
    pub fn pousa(&mut self, c: Colagem<'_>) {
        let Colagem {
            gpu,
            renderer,
            camera,
            window,
            scene_viewport,
            mundo,
            band_blit,
        } = c;
        for (m, p) in self.fontes_mut() {
            m.filtra_a_camada(crate::motion_filter::Engrenagem {
                gpu,
                renderer: &mut *renderer,
                camera,
                window,
                scene_viewport,
                formas: Some(&*p),
                mundo,
                band_blit,
            });
        }
    }

    /// A lente de cada palco, depois da do editor.
    pub fn lentes(
        &mut self,
        gpu: &GpuContext,
        mundo: &ph2d_render::WorldRt,
        cena: Option<[f32; 4]>,
    ) {
        for (m, _) in self.fontes_mut() {
            m.lente.aplica(gpu, mundo, cena);
        }
    }

    /// **As formas de cada palco bloqueiam a luz 2D** — a porta do editor
    /// ([`motion_shape_placa::luz::OclusaoDoMotion`]): `grava` recebe as famílias do `editor` e, a
    /// seguir, uma por palco.
    pub fn com_oclusoes<R>(
        &mut self,
        editor: &mut [&mut dyn CamadaDeOclusao],
        grava: impl FnOnce(&mut [&mut dyn CamadaDeOclusao]) -> R,
    ) -> R {
        let mut palcos: Vec<_> = self
            .fontes_mut()
            .map(|(m, p)| motion_shape_placa::luz::OclusaoDoMotion::do_quadro(p, m))
            .collect();
        let mut todas: Vec<&mut dyn CamadaDeOclusao> = editor
            .iter_mut()
            .map(|c| &mut **c as &mut dyn CamadaDeOclusao)
            .collect();
        todas.extend(palcos.iter_mut().map(|o| o as &mut dyn CamadaDeOclusao));
        grava(&mut todas)
    }
}

#[cfg(test)]
impl PalcosDasMacros {
    /// Até onde o palco da entidade `bits` tocou (tocando ou em cache).
    pub(crate) fn tempo(&self, bits: u64) -> Option<ph2d_macro::MacroTime> {
        let l = self.lidos.get(&bits)?;
        Some(l.palco.as_ref().ok()?.as_ref()?.em_cena.tempo())
    }

    pub(crate) fn quantos(&self) -> usize {
        self.lidos.len()
    }
}

/// O que a colagem dos palcos pede da shell — a [`crate::motion_filter::Engrenagem`] sem as formas
/// (cada palco dá as suas).
pub struct Colagem<'a> {
    pub gpu: &'a GpuContext,
    pub renderer: &'a mut ph2d_render::SpriteRenderer,
    pub camera: &'a ph2d_render::Camera2d,
    pub window: ph2d_host::WindowSize,
    pub scene_viewport: Option<[f32; 4]>,
    pub mundo: &'a ph2d_render::WorldRt,
    pub band_blit: &'a ph2d_render::BandBlit,
}
