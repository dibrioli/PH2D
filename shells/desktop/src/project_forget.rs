//! **Os memos de geometria DERIVADA não sobrevivem à troca de documento.**
//!
//! Filho de [`crate::project`] pelo teto de LOC (HR-18), cortado por responsabilidade: o load
//! inteiro é longo, e esta é uma pergunta só — *o que um documento novo apaga?*
//!
//! ⚠️ **A razão é a mesma para os sete, e é dura:** os `VecPathId` são **reciclados entre
//! documentos** (a cena nova começa a contar do zero), então um acerto de memo desenharia a
//! geometria do projeto ANTERIOR sobre uma forma do novo — com a mesma chave, e sem nada indicar
//! por quê. É a razão pela qual cada produtor tem um `forget()` em vez de confiar em invalidação
//! por conteúdo.

impl crate::App {
    /// Esquece todo memo de produtor vivo. Chamado pelo load, **antes** de o mundo novo entrar.
    pub(super) fn forget_live_producers(&mut self) {
        self.offset_live.forget();
        self.pattern_live.forget();
        self.contour_live.forget();
        self.align_live.forget();
        self.bool_live.forget();
        self.symmetry_live.forget();
        self.fx_live.forget();
        // Não é memo, é uma decisão de sessão: qual lado do offset foi espelhado por último.
        self.vec_offset_mirrored = None;
    }
}
