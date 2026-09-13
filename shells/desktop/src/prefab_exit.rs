//! ⭐⭐⭐ **As duas SAÍDAS do palco do prefab** — `Done` e `Cancel` (Enio, 2026-09-07: *«só permita
//! sair da edição apertando Done ou a tecla Enter»*).
//!
//! # ⛔ Por que ESTE pedaço fica na shell e a lei do palco não
//!
//! A lei do palco — pôr a receita no centro da área visível como pré-visualização, e devolvê-la ao
//! bastidor ao fechar — mudou-se para [`ph2d_app_components::prefab_stage`] na 5.ª rodada da W2
//! (2026-09-13, `line/components`). Estes dois métodos NÃO podiam ir com ela, e a razão está nos
//! TIPOS, não na arrumação: o `Cancel` repõe o documento inteiro por `App::apply_project` a partir
//! do `ProjectState` guardado na abertura — e o `ProjectState` é da fila de undo da shell
//! (`undo.rs`), que fica aqui por desenho. *O que sai são os CORPOS; o que repõe o documento fica.*
//!
//! ⚠️ Os corpos são os de antes, VERBATIM (só o caminho do `Exit` mudou), e os gates que os leem
//! (`tests/it/the_open_recipe_comes_to_the_artist.rs`) leem este ficheiro.

impl crate::App {
    /// ⭐⭐⭐ **DESCER O PALCO** — serve o pedido que a barra (ou a tecla) deixou.
    ///
    /// ⚠️ **Corre com o `self` LIVRE, no fim do quadro**, e não é conforto: o `Cancel` repõe o
    /// documento inteiro (`apply_project`), o que respawna as entidades — a meio do quadro, com o
    /// `gfx` emprestado, isso é inexprimível.
    ///
    /// ⚠️ **ANTES do `post_frame_undo`**, de propósito: o cancelamento é uma mudança do documento
    /// como qualquer outra, então o passo por DIFF regista-o e o `Ctrl+Z` **traz as edições de
    /// volta**. *Um cancelamento que não se pudesse desfazer seria a única acção irreversível do
    /// app.*
    ///
    /// ⚠️ **As DUAS saídas largam a trava e a selecção.** Sem a segunda, o carimbo do quadro
    /// seguinte reabria a sessão a partir da selecção — a trava seria solta e o modo voltaria.
    pub(crate) fn serve_prefab_exit(&mut self) {
        let exit = self
            .gfx
            .as_mut()
            .and_then(|g| g.hero_screen.as_mut())
            .and_then(|h| h.prefab_exit.take());
        let Some(exit) = exit else {
            return;
        };
        if exit == ph2d_app_components::prefab_stage::Exit::Cancel
            && let Some(state) = self.prefab_cancel.take()
        {
            // ⚠️ **O palco larga-se SEM repor a pose**: a fotografia foi tirada com o autorado no
            // lugar (a captura substitui a pré-visualização), então repô-la aqui escreveria numa
            // entidade que o restauro está prestes a matar — e a pose certa já vem lá dentro.
            self.prefab_stage = None;
            self.apply_project(&state);
        }
        self.prefab_cancel = None;
        self.prefab_cancel_pending = false;
        self.prefab_editing = None;
        self.prefab_stage_pending = None;
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.gizmo.replace_selection(None);
        }
    }

    /// **Pede a saída** — a porta que a TECLA usa, para o teclado e o botão terminarem no mesmo
    /// sítio.
    ///
    /// ⚠️ Devolve `false` quando não há sessão aberta, e é isso que deixa o `Esc`/`Enter` cair para
    /// os consumidores de sempre (o blur de um widget, um campo de texto).
    pub(crate) fn request_prefab_exit(
        &mut self,
        exit: ph2d_app_components::prefab_stage::Exit,
    ) -> bool {
        if self.prefab_editing.is_none() {
            return false;
        }
        // ⛔⛔ **E NUNCA com um campo de texto no foco.** A sessão dura minutos, então esta guarda
        // não é uma cortesia: renomear uma peça dentro da receita e carregar `Enter` para confirmar
        // o nome **fecharia a sessão**, e o `Esc` que desiste do nome **cancelaria tudo o que foi
        // feito**. *Uma tecla reivindicada por um modo longo tem de devolver o teclado a quem está
        // a escrever.*
        if self.text_entry_focused() {
            return false;
        }
        let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return false;
        };
        hero.prefab_exit = Some(exit);
        true
    }
}
