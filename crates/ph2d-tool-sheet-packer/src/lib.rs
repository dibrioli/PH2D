#![forbid(unsafe_code)]
//! ph2d-tool-sheet-packer — **empacotar a seleção numa folha de sprites**.
//!
//! O pill que substitui o `PH2D_SHEET_SMOKE`: seleciona-se um punhado de sprites, clica-se, e
//! nasce **um objeto** — a folha (plano [`docs/Sprite_projeto/17`] §7), com as peças arranjadas
//! dentro como filhos.
//!
//! ## Ele age sobre a SELEÇÃO INTEIRA, e é a exceção da fila
//!
//! ⚠️ Os outros one-shot desta linha (`trim`, `make_square`, `real_size`, `rasterize`) são
//! **por-sprite**: a chrome emite um `OneShotImageOp` por entidade selecionada e cada um é
//! aplicado isoladamente. Este não pode ser: empacotar N sprites é **um** ato — N atos
//! independentes produziriam N folhas de uma peça cada. A shell junta os eventos da mesma leva
//! e chama a criação **uma vez**, exatamente como o `equalize_sizes` (o outro tool cross-sprite)
//! já faz. *Um verbo que fala da relação entre as peças não cabe num evento por peça.*
//!
//! ## A ilha
//!
//! - [`icon`] — o glifo do pill.
//! - [`manifest`] — a declaração.
//! - [`register`] — a entrada que o `ph2d-tool-sync` apende ao `register_all`.
//!
//! ⚠️ **Não há `algorithm` aqui, e isso é o desenho a funcionar:** o empacotamento é puro e já
//! vive na `ph2d-sprite-sheet` (`layout`/`pack`, 31 testes), e a costura com o ECS vive na shell
//! (`sheet_frame`). Uma terceira cópia da lei de arranjo dentro do tool seria a segunda resposta
//! à pergunta *"onde cada peça fica"* — e as duas divergiriam na primeira afinação.

pub mod icon;
pub mod manifest;

pub use icon::sheet_packer_bezpath;
pub use manifest::MANIFEST;

/// Regista o manifesto do Sheet Packer. Apendado ao
/// `ph2d-tool-registry-init::register_all` pelo `ph2d-tool-sync`.
pub fn register(reg: &mut ph2d_tool_registry::Registry) {
    reg.register(&MANIFEST);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_attaches_manifest_to_registry() {
        let mut reg = ph2d_tool_registry::Registry::default();
        register(&mut reg);
        reg.build()
            .expect("registry should build with sheet-packer");
        let found = reg
            .by_id("sheet_packer")
            .expect("sheet_packer should be registered by id");
        assert_eq!(found.id, "sheet_packer");
        assert_eq!(found.label_key, "tool.sheet_packer.label");
    }

    #[test]
    fn register_is_idempotent() {
        let mut reg = ph2d_tool_registry::Registry::default();
        register(&mut reg);
        register(&mut reg);
        reg.build().expect("idempotent registration should build");
        assert_eq!(reg.manifests().len(), 1);
    }
}
