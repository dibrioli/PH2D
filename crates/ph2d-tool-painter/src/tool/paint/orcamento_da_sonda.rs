//! B44.8 (missão 4K N10): os contadores do motor que o portão `orcamento_4k` (TP/tests/it) lê pela
//! ferramenta. Só leitura; nenhuma lógica muda.

impl crate::tool::PainterTool {
    /// As escritas de tela inteira do motor do Wet Paint até agora (`Engine::marcas_da_tela_inteira`,
    /// B44.9); `0` sem sessão. Traz o motor para casa primeiro (a leitura imutável no worker panica).
    #[doc(hidden)]
    pub fn wet_marcas_da_tela_inteira(&mut self) -> u64 {
        self.paint.wetpaint.session.as_mut().map_or(0, |s| {
            s.bring_home();
            s.engine.marcas_da_tela_inteira()
        })
    }
}
