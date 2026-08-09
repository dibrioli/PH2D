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
        // ⚠️ **A posição autorada dos CONTROLES entra aqui pela lei do cabeçalho, e faltava.** Ela
        // é chaveada por `VecPathId` como os sete abaixo — mas o dano dela não é geometria
        // desenhada errada, é **o valor salvo do documento novo ser DESTRUÍDO**: com um memo
        // herdado, o ramo de primeira-vista do `reconcile` (o que faz o arquivo mandar num load) é
        // pulado, cai-se em *"o artista ganha"* — e o artista é o projeto morto. Medido: o 0.8 que
        // saiu do arquivo vira o 0.0 neutro do controle, mais um passo de undo que ninguém pediu.
        self.vec_widget_applied.clear();
        self.offset_live.forget();
        self.pattern_live.forget();
        self.contour_live.forget();
        self.align_live.forget();
        self.bool_live.forget();
        self.symmetry_live.forget();
        self.fx_live.forget();
        // Não é memo, é uma decisão de sessão: qual lado do offset foi espelhado por último.
        self.vec_offset_mirrored = None;
        // **OS OBJETOS ASSADOS do documento anterior** (`docs/3D/02.2`). O mapa é chaveado por bits
        // de entidade, e o `apply_project` despawna tudo: as entradas que sobrassem descreveriam
        // objetos de outro projeto, e o passe de re-acendida ficaria acendendo, todo frame e para
        // sempre, slots de textura que ninguém mostra. O `restore_baked_forms` repovoa o mapa logo
        // depois, com os bits novos. ⚠️ O `baked_light` NÃO é limpo: ele é o passe (um pipeline),
        // não estado do documento — recriá-lo por load seria pagar uma compilação por Ctrl+O.
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.baked_forms.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    /// **O load esquece a posição autorada dos controles.**
    ///
    /// ⚠️ Gate red-first sobre a porta REAL. Ele parece higiene e não é: o memo é chaveado por
    /// `VecPathId`, e a cena nova reconta do zero — então a entrada do projeto ANTERIOR é adotada
    /// por uma forma do novo, com a mesma chave. O irmão
    /// `a_memo_from_the_previous_document_destroys_the_value_the_file_carries` mede o que isso
    /// custa: o valor que saiu do arquivo é sobrescrito pelo neutro do controle.
    ///
    /// ⚠️ E o gate mora AQUI, e não numa varredura do fonte, porque a pergunta é *"esta porta
    /// esvazia?"* — um arch-gate que procurasse o nome do campo ficaria verde no dia em que alguém
    /// o renomeasse, e vermelho no dia em que alguém o esvaziasse por outra via.
    #[test]
    fn the_load_forgets_the_authored_control_memo() {
        let mut app = crate::App::new();
        app.vec_widget_applied.insert(3, 0.30);
        app.forget_live_producers();
        assert!(
            app.vec_widget_applied.is_empty(),
            "o memo do documento anterior sobreviveu ao load — a proxima forma que herdar o \
             VecPathId 3 nasce com a posicao de um controle morto, e o valor que o arquivo \
             carrega e' destruido"
        );
    }
}
