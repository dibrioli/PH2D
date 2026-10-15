//! **As macros do Painter em cena** (ADR-0186, W4 item 5.5 D): cada entidade com um `ph2d_ecs::Macro`
//! cuja trilha `"painter"` a Timeline conduz neste quadro (`ph2d_ecs::MacroTempo`) toca num
//! [`MacroEmCena`] próprio, e a tela dele troca a textura da sprite-alvo no extract — pela MESMA porta
//! da pré-visualização do editor (`PreviewOverride`), à frente dela: uma sprite que a macro toca mostra
//! a macro, nunca o traço do editor.
//!
//! Um palco nasce quando o `Macro` muda (o tique de mudança do componente) e fica em cache enquanto a
//! entidade o tiver; um quadro sem `MacroTempo` tira-o da cena sem o recozer. A ranhura de textura de
//! um palco que morre ou renasce devolve-se no quadro seguinte.

use ph2d_ecs::{Entity, Macro, MacroTempo, StableId, World};
use ph2d_editor_core::toast::{Toast, ToastQueue};
use ph2d_tool_painter::macro_track::em_cena::MacroEmCena;
use ph2d_tool_painter::macro_track::tocar::ErroDaMacro;
use std::collections::BTreeMap;

#[cfg(test)]
#[path = "palcos_custo_tests.rs"]
mod palcos_custo_tests;
#[cfg(test)]
#[path = "palcos_tests.rs"]
mod palcos_tests;
/// `PH2D_MACRO_SMOKE=1` — a cena do Splash.
pub mod smoke;

/// As ranhuras de textura da cena (o `IndividualTextureStore` do renderer; um duplo nos gates).
pub trait RanhurasDaCena {
    /// Uma ranhura nova com `rgba` premultiplicado.
    ///
    /// # Errors
    /// O porquê, para o artista.
    fn adquire(&mut self, w: u32, h: u32, rgba: &[u8]) -> Result<u32, String>;
    /// Substitui os pixels (e o tamanho) da ranhura `id`.
    ///
    /// # Errors
    /// O porquê, para o artista.
    fn substitui(&mut self, id: u32, w: u32, h: u32, rgba: &[u8]) -> Result<(), String>;
    /// Substitui só o retângulo `(x, y, w, h)` da ranhura `id` (`rgba` é o retângulo, premultiplicado).
    ///
    /// # Errors
    /// O porquê, para o artista.
    fn substitui_regiao(
        &mut self,
        id: u32,
        caixa: (u32, u32, u32, u32),
        rgba: &[u8],
    ) -> Result<(), String>;
    fn solta(&mut self, id: u32);
}

impl RanhurasDaCena for ph2d_render::SpriteRenderer {
    fn adquire(&mut self, w: u32, h: u32, rgba: &[u8]) -> Result<u32, String> {
        self.acquire_individual(w, h, rgba)
            .map_err(|e| e.to_string())
    }

    fn substitui(&mut self, id: u32, w: u32, h: u32, rgba: &[u8]) -> Result<(), String> {
        self.replace_individual_pixels(id, w, h, rgba)
            .map_err(|e| e.to_string())
    }

    fn substitui_regiao(
        &mut self,
        id: u32,
        (x, y, w, h): (u32, u32, u32, u32),
        rgba: &[u8],
    ) -> Result<(), String> {
        self.replace_individual_pixels_region(id, x, y, w, h, rgba)
            .map_err(|e| e.to_string())
    }

    fn solta(&mut self, id: u32) {
        self.individual_mut().release(id);
    }
}

/// A sprite cuja textura um palco troca neste quadro.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrocaDaSprite {
    pub entity_bits: u64,
    /// A ranhura com a tela do palco, premultiplicada.
    pub texture_id: u32,
}

struct PalcoDaPintura {
    em_cena: MacroEmCena,
    /// A sprite-alvo neste quadro (resolvida pelo `StableId` da trilha).
    sprite: Option<u64>,
    ranhura: Option<u32>,
    /// O tamanho da tela na ranhura (uma subida parcial só cabe na mesma).
    tamanho: Option<(u32, u32)>,
    toca: bool,
}

struct Lido {
    tique: u32,
    /// `Ok(None)` = nada gravado, ou a gravação não tem trilha do Painter.
    palco: Result<Option<PalcoDaPintura>, ErroDaMacro>,
}

impl Lido {
    fn ranhura(&self) -> Option<u32> {
        self.palco.as_ref().ok()?.as_ref()?.ranhura
    }
}

/// Os palcos das macros do Painter, por entidade (bits).
#[derive(Default)]
pub struct PalcosDoPainter {
    lidos: BTreeMap<u64, Lido>,
    /// Ranhuras de palcos que morreram ou renasceram, a devolver no próximo [`Self::sobe`].
    soltas: Vec<u32>,
    /// O espelho premultiplicado da última tela subida (reusado entre quadros).
    premul: Vec<u8>,
}

fn palco_de(m: &Macro) -> Result<Option<PalcoDaPintura>, ErroDaMacro> {
    if m.doc.is_empty() {
        return Ok(None);
    }
    Ok(MacroEmCena::de(&m.doc)?.map(|em_cena| PalcoDaPintura {
        em_cena,
        sprite: None,
        ranhura: None,
        tamanho: None,
        toca: false,
    }))
}

/// O palco de um `Macro` que mudou: o que já tocava REEDITA-SE (recomeça do instantâneo mais tardio que
/// a trilha nova ainda tem, `MacroEmCena::reedita`); o resto nasce do zero. A ranhura de um palco que se
/// vai fica a devolver.
fn relido(velho: Option<Lido>, m: &Macro, tique: u32, soltas: &mut Vec<u32>) -> Lido {
    if let Some(Lido {
        palco: Ok(Some(mut p)),
        ..
    }) = velho
    {
        let palco = if m.doc.is_empty() {
            Ok(None)
        } else {
            p.em_cena.reedita(&m.doc).map(|ainda| ainda.then_some(()))
        };
        return match palco {
            Ok(Some(())) => Lido {
                tique,
                palco: Ok(Some(p)),
            },
            outro => {
                soltas.extend(p.ranhura);
                Lido {
                    tique,
                    palco: outro.map(|_| None),
                }
            }
        };
    }
    soltas.extend(velho.and_then(|l| l.ranhura()));
    Lido {
        tique,
        palco: palco_de(m),
    }
}

impl PalcosDoPainter {
    /// **O erro da macro do objecto `bits`** — a gravação que não se lê, ou o passo em que o palco
    /// parou (o painel MACROS di-lo ao artista, W5d).
    #[must_use]
    pub fn erro_de(&self, bits: u64) -> Option<&ErroDaMacro> {
        match &self.lidos.get(&bits)?.palco {
            Err(e) => Some(e),
            Ok(p) => p.as_ref()?.em_cena.erro(),
        }
    }

    /// **De onde conferir a trilha `track` da macro do objecto `bits`** — o instantâneo do palco dele
    /// ([`MacroEmCena::partida`]); `None` = do zero.
    #[must_use]
    pub fn partida(
        &self,
        bits: u64,
        track: &ph2d_macro::Track,
    ) -> Option<ph2d_tool_painter::macro_track::etapas::Partida> {
        match &self.lidos.get(&bits)?.palco {
            Ok(Some(p)) => p.em_cena.partida(track),
            _ => None,
        }
    }

    /// **Por quadro**, depois de a Timeline escrever o `MacroTempo` e antes do extract: (re)cria o
    /// palco de um `Macro` que mudou e segue o relógio de cada um que a Timeline conduz.
    pub fn segue(&mut self, world: &mut World) {
        let mut q = world.query::<(Entity, &Macro, Option<&MacroTempo>)>();
        let vistos: Vec<(Entity, Option<f64>)> = q
            .iter(world)
            .map(|(e, _, t)| (e, t.map(|t| t.segundos)))
            .collect();
        let soltas = &mut self.soltas;
        self.lidos.retain(|bits, l| {
            let vivo = vistos.iter().any(|(e, _)| e.to_bits() == *bits);
            if !vivo {
                soltas.extend(l.ranhura());
            }
            vivo
        });
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
                let velho = self.lidos.remove(&e.to_bits());
                let lido = relido(velho, m, tique, &mut self.soltas);
                self.lidos.insert(e.to_bits(), lido);
            }
            let Some(Lido {
                palco: Ok(Some(p)), ..
            }) = self.lidos.get_mut(&e.to_bits())
            else {
                continue;
            };
            p.toca = segundos.is_some();
            let Some(s) = segundos else {
                continue;
            };
            let alvo = p.em_cena.alvo();
            p.sprite = if alvo == 0 {
                Some(e)
            } else {
                ph2d_ecs::entity_with_stable_id(world, StableId(alvo))
            }
            .map(Entity::to_bits);
            // Um passo que falha pára a macro nele e fica em `MacroEmCena::erro`: a sprite continua a
            // mostrar o que a macro pintou até lá.
            let _ = p.em_cena.segue(s);
        }
        // Uma mudança depois desta leitura leva um tique NOVO (a shell não corre um `Schedule`).
        world.increment_change_tick();
    }

    /// **Sobe a tela de cada palco que toca e mudou** — só o retângulo que mudou desde a subida anterior,
    /// quando a ranhura já tem a tela daquele tamanho — e devolve as sprites a trocar no extract. Devolve
    /// primeiro as ranhuras dos palcos que se foram.
    ///
    /// `cede` = a sprite que o Painter está a gravar: mostra o traço vivo do editor, não a macro.
    pub fn sobe(
        &mut self,
        ranhuras: &mut dyn RanhurasDaCena,
        toasts: &mut ToastQueue,
        cede: Option<u64>,
    ) -> Vec<TrocaDaSprite> {
        for id in self.soltas.drain(..) {
            ranhuras.solta(id);
        }
        let mut trocas = Vec::new();
        for l in self.lidos.values_mut() {
            let Ok(Some(p)) = &mut l.palco else {
                continue;
            };
            if !p.toca {
                continue;
            }
            if let Some((rgba, w, h)) = p.em_cena.toma_a_tela() {
                let caixa = p
                    .em_cena
                    .caixa_da_tela()
                    .filter(|&(_, _, cw, ch)| p.tamanho == Some((w, h)) && cw > 0 && ch > 0);
                self.premul.clear();
                match caixa {
                    Some((x, y, cw, ch)) => {
                        let (x, cw) = (x as usize, cw as usize);
                        let linhas = rgba.chunks_exact(w as usize * 4).skip(y as usize);
                        for linha in linhas.take(ch as usize) {
                            self.premul.extend_from_slice(&linha[x * 4..(x + cw) * 4]);
                        }
                    }
                    None => self.premul.extend_from_slice(&rgba),
                }
                drop(rgba);
                ph2d_render::premul::premultiply_rgba8(&mut self.premul);
                let subida = match (p.ranhura, caixa) {
                    (Some(id), Some(c)) => ranhuras.substitui_regiao(id, c, &self.premul),
                    (Some(id), None) => ranhuras.substitui(id, w, h, &self.premul),
                    (None, _) => ranhuras.adquire(w, h, &self.premul).map(|id| {
                        p.ranhura = Some(id);
                    }),
                };
                p.tamanho = match (&subida, caixa) {
                    (Ok(()), None) => Some((w, h)),
                    (Ok(()), Some(_)) => p.tamanho,
                    (Err(_), _) => None,
                };
                if let Err(e) = subida {
                    toasts.push(Toast::error(ph2d_i18n::tr_with(
                        "app.painter.painter_bridge_upload.preview_upload_failed",
                        &[("e", &e)],
                    )));
                }
            }
            if let (Some(entity_bits), Some(texture_id)) = (p.sprite, p.ranhura)
                && Some(entity_bits) != cede
            {
                trocas.push(TrocaDaSprite {
                    entity_bits,
                    texture_id,
                });
            }
        }
        trocas
    }
}
