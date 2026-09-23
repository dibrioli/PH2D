//! ⏱️ **A SONDA DO CACHE DE PIPELINES EM DISCO** — o que MEDE, separado do que despacha.
//!
//! ⚠️ **Ela saiu do [`super::trace`] por um TECTO DE LOC** (2026-09-22), e é a mesma fronteira que
//! o `ph2d-app-field3d/src/device_probes.rs` já pagou: *uma sonda responde «quanto» e um gate
//! responde «ainda é verdade»*, e os dois têm leitores diferentes.

#[cfg(test)]
mod sonda_do_cache_em_disco {
    /// ⏱️⭐⭐⭐ **A PLACA DESTA MÁQUINA OFERECE CACHE DE PIPELINES EM DISCO?**
    ///
    /// A `W9` nomeia o cache em disco como a cura do `1,3 s` que a placa paga **uma vez por
    /// sessão** a compilar o kernel do pintor (`docs/Render3d/03` §W9), e a ranhura dele
    /// (`cache: None` no [`crate::FieldPipelines::entry_with_layout`]) está vazia.
    ///
    /// ⛔⛔ **Mas o doc do `wgpu` avisa que numa plataforma de desktop isto quase não paga:** *«most
    /// desktop GPU drivers will manage their own caches, meaning that little advantage can be
    /// gained from this on those platforms»*. ⇒ *§0.0: antes de construir, meça se a porta existe.*
    ///
    /// Esta sonda imprime três coisas: se o adaptador **anuncia** a feature, a chave que ele daria
    /// a um ficheiro de cache, e o nome do backend — que é o que decide se a advertência se aplica.
    ///
    /// # ⛔⛔⛔ O veredito de VIABILIDADE (2026-09-22), e é por isso que o item DESCEU na fila
    ///
    /// | pergunta | resposta MEDIDA |
    /// |---|---|
    /// | o adaptador oferece? | **SIM** — `NVIDIA GeForce RTX 5060 Ti`, Vulkan, driver `615.71.09`, chave `wgpu_pipeline_cache_vulkan_4318_11524` |
    /// | quanto vale? | `1,3`–`2,8 s`, **uma vez por sessão** (o kernel do pintor; o resto já o paga a fita inerte) |
    /// | quanto custa construir? | uma crate/módulo **isolado com `unsafe`** — o `Device::create_pipeline_cache` é `unsafe` (o blob é entrada não confiável para o driver) e a workspace declara `unsafe_code = "forbid"`. O molde da casa é o [ADR-0116](../../../docs/architecture/decisions/0116-audio-export-opus-isolated-unsafe-crate.md): a crate desce o `forbid` a **`deny`** e só o módulo que toca a ABI o autoriza por escrito, com gate na lista. |
    /// | paga? | ⚠️ **INCERTO, e o fornecedor diz que não:** *«most desktop GPU drivers will manage their own caches, meaning that little advantage can be gained from this on those platforms»*. |
    ///
    /// ⚠️⚠️ **E a evidência desta máquina é MISTA, o que é a razão para não decidir de barato:** em
    /// duas corridas do mesmo processo o pipeline `bordas` foi de `24,70` para `0,28 ms` com o
    /// SPIR-V idêntico — o driver cacheou-o —, e a 1.ª pintura de uma peça leu `1 449` e `1 417 ms`
    /// em dois processos separados, ou seja **o cache do driver não ajudou o pipeline grande**.
    /// *É exactamente ali que a nossa ranhura poderia pagar, e é exactamente ali que não há
    /// medição.*
    ///
    /// ⛔ **A experiência de três vias que decidiria isto NÃO É ESCREVÍVEL nesta árvore** (`A` com
    /// cache vazio · `B` com o blob · `C` sem cache, que é o veredito — *se o `B` for rápido e o
    /// `C` também, quem cacheou foi o driver*): ela precisa de `unsafe`, e o `forbid` da workspace
    /// não se contorna com um `allow`. ⇒ ela é acto de uma crate irmã, no dia em que este item
    /// subir na fila.
    #[test]
    #[ignore = "sonda de diagnóstico: precisa de GPU"]
    fn sonda_a_placa_oferece_cache_de_pipelines() {
        let instance = wgpu::Instance::default();
        let Some(adapter) =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            }))
            .ok()
        else {
            println!("sem adaptador — saltado");
            return;
        };
        let info = adapter.get_info();
        let tem = adapter.features().contains(wgpu::Features::PIPELINE_CACHE);
        println!("\n  adaptador ····· {} ({:?})", info.name, info.backend);
        println!("  driver ········ {} {}", info.driver, info.driver_info);
        println!("  PIPELINE_CACHE  {}", if tem { "SIM" } else { "NÃO" });
        println!(
            "  chave ········· {}\n",
            wgpu::util::pipeline_cache_key(&info).unwrap_or_else(|| "— (sem chave)".to_string())
        );
    }
}
