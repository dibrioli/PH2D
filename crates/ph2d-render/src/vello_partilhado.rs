//! **UM `vello::Renderer` por placa, partilhado por todos os [`crate::VelloPass`]** (F37 §2,
//! `docs/UI_New_and_Simple/F37_plano.md`). Cada renderer guarda no seu `ResourcePool` os
//! alocadores fixos do Vello 0.10 (~166 MiB, medido por rótulo na F36 §2), e o editor tem pelo menos
//! oito donos de `VelloPass` — a cena 41 acordava três. Partilhar o renderer guarda UM reservatório.
//!
//! O que o renderer guarda passa a ser da placa: o átlas de imagens (ids globais, sem colisão), as
//! texturas registadas ([`Partilhado::registos`], que o renderer REFEITO herda — F35 §2) e o
//! episódio de falta de memória (uma recusa envenena o pool de TODOS os donos, então refazê-lo é
//! uma decisão da placa).

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use vello::{Renderer, RendererOptions};

pub(crate) struct Partilhado {
    /// O dispositivo deste renderer (o `refaz` cria o novo nele).
    device: wgpu::Device,
    pub(crate) renderer: Renderer,
    /// As texturas registadas por todos os donos, para o renderer REFEITO as herdar com os mesmos ids.
    pub(crate) registos: BTreeMap<u64, (vello::peniko::ImageData, wgpu::Texture)>,
    /// A placa recusou memória: o renderer é refeito a partir deste instante. `None` = são.
    pub(crate) refazer_em: Option<std::time::Instant>,
    /// Quantas vezes o renderer foi refeito (diagnóstico e gate).
    pub(crate) refeitos: u32,
}

pub(crate) type Porta = Arc<Mutex<Partilhado>>;

/// `(o Arc do dispositivo do GpuContext, o renderer dele)`. A chave é a IDENTIDADE do `Arc`: o
/// `PartialEq` do `wgpu::Device` deu dois dispositivos vivos por iguais (medido, G25). O `Weak`
/// segura a alocação, então o endereço não é reusado enquanto a entrada existe.
type Entrada = (Weak<wgpu::Device>, Weak<Mutex<Partilhado>>);

static POR_PLACA: Mutex<Vec<Entrada>> = Mutex::new(Vec::new());

pub(crate) fn novo_renderer(device: &wgpu::Device) -> Result<Renderer, String> {
    Renderer::new(
        device,
        RendererOptions {
            use_cpu: false,
            antialiasing_support: vello::AaSupport::all(),
            num_init_threads: None,
            pipeline_cache: None,
        },
    )
    .map_err(|e| format!("vello::Renderer::new: {e}"))
}

/// Trava sem herdar o envenenamento de um pânico alheio (o estado continua coerente: cada campo
/// é escrito de uma vez).
pub(crate) fn trava(p: &Porta) -> MutexGuard<'_, Partilhado> {
    p.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// O renderer partilhado da placa `device`: o que já existe enquanto algum dono vive, um novo
/// quando não há nenhum. Quando o último dono larga, o renderer (e o seu reservatório) sai.
pub(crate) fn da_placa(device: &Arc<wgpu::Device>) -> Result<Porta, String> {
    let mut lista = POR_PLACA
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    lista.retain(|(_, p)| p.strong_count() > 0);
    let chave = Arc::downgrade(device);
    for (d, p) in lista.iter() {
        if Weak::ptr_eq(d, &chave)
            && let Some(p) = p.upgrade()
        {
            return Ok(p);
        }
    }
    let p = Arc::new(Mutex::new(Partilhado {
        device: (**device).clone(),
        renderer: novo_renderer(device)?,
        registos: BTreeMap::new(),
        refazer_em: None,
        refeitos: 0,
    }));
    lista.push((chave, Arc::downgrade(&p)));
    Ok(p)
}

impl Partilhado {
    /// Um renderer novo (pool vazio), com as texturas registadas de todos os donos nos MESMOS ids.
    pub(crate) fn refaz(&mut self) -> Result<(), String> {
        let mut novo = novo_renderer(&self.device)?;
        for (imagem, textura) in self.registos.values() {
            novo.override_image(
                imagem,
                Some(wgpu::TexelCopyTextureInfoBase {
                    texture: textura.clone(),
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                }),
            );
        }
        self.renderer = novo;
        self.refeitos += 1;
        Ok(())
    }
}

#[cfg(test)]
#[path = "vello_partilhado_tests.rs"]
mod tests;
