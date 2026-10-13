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

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Deref;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};

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
    /// ⭐ W5o: as pipelines que um fio está a compilar agora — outro fio que peça a mesma ESPERA por
    /// ela (a estufa monta várias configurações ao mesmo tempo, e elas partilham a maioria).
    compilando: BTreeSet<String>,
}

/// ⭐ A fábrica — partilhada entre os solvers de uma mesma luz (clonar partilha o cache).
#[derive(Clone, Default)]
pub struct Fabrica(Arc<Mutex<Interno>>, Arc<Condvar>);

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

    /// ⭐ W5o — o objeto `k` guardado; senão, se outro fio o compila, espera por ele; senão compila-o
    /// AQUI, fora do cadeado (a estufa compila noutro fio enquanto o quadro corre), e guarda-o.
    fn uma_vez<T: Clone>(
        &self,
        (device, conta): (&wgpu::Device, Option<&std::sync::atomic::AtomicU64>),
        k: String,
        ler: impl Fn(&Memo, &str) -> Option<T>,
        guardar: impl FnOnce(&mut Memo, String, T),
        compilar: impl FnOnce() -> T,
    ) -> T {
        let mut m = self.memo(device);
        loop {
            if let Some(p) = ler(&m.memo, &k) {
                return p;
            }
            if !m.compilando.contains(&k) {
                break;
            }
            m = self.1.wait(m).unwrap_or_else(PoisonError::into_inner);
        }
        m.compilando.insert(k.clone());
        drop(m);
        // Solta a chave e acorda quem espera também se a compilação entrar em pânico.
        struct Solta<'a>(&'a Fabrica, &'a str);
        impl Drop for Solta<'_> {
            fn drop(&mut self) {
                let mut m = self.0.0.lock().unwrap_or_else(PoisonError::into_inner);
                m.compilando.remove(self.1);
                drop(m);
                self.0.1.notify_all();
            }
        }
        let solta = Solta(self, &k);
        let p = compilar();
        if let Some(c) = conta {
            c.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        let mut m = self.memo(device);
        m.pipelines_criadas += 1;
        guardar(&mut m.memo, k.clone(), p.clone());
        drop(m);
        drop(solta);
        p
    }
}

/// O dispositivo com a fábrica (ver o módulo).
#[derive(Clone, Copy)]
pub struct Placa<'a> {
    pub device: &'a wgpu::Device,
    pub fabrica: &'a Fabrica,
    /// ⭐ W5o: as pipelines que ESTA montagem compilou (o `Solver::pipelines_compiladas_ao_nascer`) —
    /// a conta da fábrica inteira contava também a estufa, que compila noutros fios ao mesmo tempo.
    pub conta: Option<&'a std::sync::atomic::AtomicU64>,
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
        if let Some(s) = self.fabrica.memo(self.device).memo.modulos.get(&k) {
            return s.clone();
        }
        // Compila FORA do cadeado (a estufa do app compila noutro fio enquanto o quadro corre).
        let s = self.device.create_shader_module(desc);
        let mut m = self.fabrica.memo(self.device);
        m.memo.modulos.entry(k).or_insert(s).clone()
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
        self.fabrica.uma_vez(
            (self.device, self.conta),
            format!("{desc:?}"),
            |m, k| m.computos.get(k).cloned(),
            |m, k, p| {
                m.computos.insert(k, p);
            },
            || {
                let t0 = std::time::Instant::now();
                let p = self.device.create_compute_pipeline(desc);
                // ⭐ W5o: `PH2D_LIGHT2D_COMPILA=1` — o tempo de cada pipeline (a frio, segundos).
                if std::env::var_os("PH2D_LIGHT2D_COMPILA").is_some() {
                    let ms = t0.elapsed().as_secs_f64() * 1e3;
                    eprintln!(
                        "[luz2d] compilou {ms:.1} ms {:?} {:?}",
                        desc.label, desc.entry_point
                    );
                }
                p
            },
        )
    }

    pub fn create_render_pipeline(
        &self,
        desc: &wgpu::RenderPipelineDescriptor<'_>,
    ) -> wgpu::RenderPipeline {
        self.fabrica.uma_vez(
            (self.device, self.conta),
            format!("{desc:?}"),
            |m, k| m.renders.get(k).cloned(),
            |m, k, p| {
                m.renders.insert(k, p);
            },
            || self.device.create_render_pipeline(desc),
        )
    }
}
