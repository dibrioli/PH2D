//! ⭐ **A FÁBRICA dos objetos de placa que não dependem do tamanho da grade** — os módulos de shader,
//! os layouts e as pipelines. Uma grade nova (a janela redimensionada, a oitava do zoom, o nível)
//! recria o [`crate::Solver`]: as texturas e os buffers são DA grade, mas as pipelines só dependem do
//! shader e das constantes. Medido (bug do dono 11/10, «custa a maximizar»): recriar o solver levava
//! ~2 100 ms, dos quais ~2 090 a recompilar as pipelines da C3 — e o maximizar muda o tamanho a cada
//! quadro. Com a fábrica, cada objeto é guardado pela DESCRIÇÃO inteira (o `Debug` do descritor, que
//! traz a identidade dos objetos que referencia) e volta igual quando a descrição se repete.
//!
//! [`Placa`] é o dispositivo com a fábrica: os métodos `create_*` dos objetos guardados passam por
//! ela e o resto vai direto ao `wgpu::Device` (`Deref`).

use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// Os objetos guardados, para um dispositivo.
#[derive(Default)]
struct Memo {
    device: Option<wgpu::Device>,
    grupos: BTreeMap<String, wgpu::BindGroupLayout>,
    modulos: BTreeMap<String, wgpu::ShaderModule>,
    layouts: BTreeMap<String, wgpu::PipelineLayout>,
    computos: BTreeMap<String, wgpu::ComputePipeline>,
    renders: BTreeMap<String, wgpu::RenderPipeline>,
}

/// O cache e a régua dele — o contador fica FORA do que um dispositivo novo apaga (senão a régua
/// zerava junto com o cache que mede).
#[derive(Default)]
struct Interno {
    memo: Memo,
    pipelines_criadas: u64,
}

/// ⭐ A fábrica — partilhada entre os solvers de uma mesma luz (clonar partilha o cache).
#[derive(Clone, Default)]
pub struct Fabrica(Arc<Mutex<Interno>>);

impl Fabrica {
    fn memo(&self, device: &wgpu::Device) -> MutexGuard<'_, Interno> {
        let mut m = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        if m.memo.device.as_ref() != Some(device) {
            m.memo = Memo {
                device: Some(device.clone()),
                ..Memo::default()
            };
        }
        m
    }

    /// Quantas pipelines (de cômputo e de desenho) a fábrica COMPILOU — a régua do cache.
    #[must_use]
    pub fn pipelines_criadas(&self) -> u64 {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .pipelines_criadas
    }
}

/// O dispositivo com a fábrica (ver o módulo).
#[derive(Clone, Copy)]
pub struct Placa<'a> {
    pub device: &'a wgpu::Device,
    pub fabrica: &'a Fabrica,
}

impl Deref for Placa<'_> {
    type Target = wgpu::Device;

    fn deref(&self) -> &wgpu::Device {
        self.device
    }
}

impl Placa<'_> {
    pub fn create_bind_group_layout(
        &self,
        desc: &wgpu::BindGroupLayoutDescriptor<'_>,
    ) -> wgpu::BindGroupLayout {
        let k = format!("{desc:?}");
        let mut m = self.fabrica.memo(self.device);
        m.memo
            .grupos
            .entry(k)
            .or_insert_with(|| self.device.create_bind_group_layout(desc))
            .clone()
    }

    pub fn create_shader_module(
        &self,
        desc: wgpu::ShaderModuleDescriptor<'_>,
    ) -> wgpu::ShaderModule {
        let k = format!("{desc:?}");
        let mut m = self.fabrica.memo(self.device);
        if let Some(s) = m.memo.modulos.get(&k) {
            return s.clone();
        }
        let s = self.device.create_shader_module(desc);
        m.memo.modulos.insert(k, s.clone());
        s
    }

    pub fn create_pipeline_layout(
        &self,
        desc: &wgpu::PipelineLayoutDescriptor<'_>,
    ) -> wgpu::PipelineLayout {
        let k = format!("{desc:?}");
        let mut m = self.fabrica.memo(self.device);
        m.memo
            .layouts
            .entry(k)
            .or_insert_with(|| self.device.create_pipeline_layout(desc))
            .clone()
    }

    pub fn create_compute_pipeline(
        &self,
        desc: &wgpu::ComputePipelineDescriptor<'_>,
    ) -> wgpu::ComputePipeline {
        let k = format!("{desc:?}");
        let mut m = self.fabrica.memo(self.device);
        if let Some(p) = m.memo.computos.get(&k) {
            return p.clone();
        }
        let p = self.device.create_compute_pipeline(desc);
        m.pipelines_criadas += 1;
        m.memo.computos.insert(k, p.clone());
        p
    }

    pub fn create_render_pipeline(
        &self,
        desc: &wgpu::RenderPipelineDescriptor<'_>,
    ) -> wgpu::RenderPipeline {
        let k = format!("{desc:?}");
        let mut m = self.fabrica.memo(self.device);
        if let Some(p) = m.memo.renders.get(&k) {
            return p.clone();
        }
        let p = self.device.create_render_pipeline(desc);
        m.pipelines_criadas += 1;
        m.memo.renders.insert(k, p.clone());
        p
    }
}
