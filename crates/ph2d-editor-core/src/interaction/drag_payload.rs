//! ⭐⭐⭐ **O QUE ATRAVESSA UM PAINEL** — a carga de um arrasto (plano `docs/Components/07`, B1).
//!
//! Até agora todo arrasto deste editor vivia **dentro** de um painel: reparentar na Hierarquia,
//! mover uma janela flutuante, agarrar uma barra de rolagem, arrastar uma célula da tira do Flip.
//! Cada um guarda o próprio estado e resolve o próprio alvo, e isso está certo enquanto o alvo é o
//! mesmo painel que iniciou o gesto.
//!
//! Arrastar da **biblioteca de assets** para a **tela** — ou para um campo do Inspector — é o
//! primeiro que sai. E o que muda não é a mecânica do ponteiro: é que **o alvo não sabe o que
//! recebeu**.
//!
//! # ⛔ A lei: a carga DIZ o que é
//!
//! Um payload opaco (um `u64` cru, um índice, um ponteiro) obriga cada alvo a **adivinhar** —
//! e um alvo que adivinha aceita o que não devia. ⇒ a carga é um **enum**, e um alvo decide com um
//! `match`: *«isto eu sei receber; isto eu recuso»*. É o que faz a recusa ser **exprimível**, e a
//! recusa visível é metade do gesto (largar num sítio errado tem de se ver, nunca ser silêncio).
//!
//! ⚠️ **E ela é feita de PRIMITIVOS, de propósito.** Este módulo é fundação de chrome; ele não
//! conhece `ph2d-asset-index` nem `ph2d-ecs`, e não pode — senão a UI passa a depender do modelo de
//! assets, e um painel que arraste outra coisa qualquer amanhã teria de o arrastar também. O que
//! ele carrega é o **endereço**, na forma mais crua que existe; quem o sabe interpretar é o alvo.

/// A carga de um arrasto em curso.
///
/// ⚠️ **Uma variante por FAMÍLIA de coisa arrastável**, e não uma por painel de origem: o alvo
/// decide pelo que a coisa **é**, não por de onde ela veio.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DragPayload {
    /// Um **prefab** da biblioteca — o `StableId` da receita.
    ///
    /// ⚠️ `u64` e não uma entidade: o `StableId` sobrevive ao respawn do undo por construção, e um
    /// arrasto pode atravessar um quadro em que o mundo foi reconstruído.
    Prefab { stable_id: u64 },
    /// Uma **imagem** da biblioteca — o blake3 do conteúdo.
    Image { asset: [u8; 32] },
}

impl DragPayload {
    /// O rótulo que a voz do arrasto mostra sob o cursor.
    #[must_use]
    pub fn kind_label(self) -> &'static str {
        match self {
            DragPayload::Prefab { .. } => "Prefab",
            DragPayload::Image { .. } => "Image",
        }
    }
}

/// **O arrasto em curso** — o que o `WidgetStore` guarda entre o `Down` e o `Up`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InFlightDrag {
    /// O que está a ser arrastado.
    pub payload: DragPayload,
    /// Onde o cursor está agora, em coordenadas de janela.
    ///
    /// ⚠️ **Ela é escrita a cada `Move`** — é o que o fantasma segue, e é o que o alvo lê para
    /// saber se o cursor está em cima dele. Sem ela, o alvo teria de perguntar ao ponteiro por
    /// fora, e passariam a existir duas respostas para *«onde está o cursor?»*.
    pub cursor: (f32, f32),
    /// O gesto já passou o limiar e é de facto um arrasto?
    ///
    /// ⛔ **Um `Down` seguido de `Up` sem movimento é um CLIQUE**, e tem de continuar a sê-lo — no
    /// navegador de assets, o clique escolhe e o duplo-clique instancia. Enquanto isto for `false`
    /// o gesto ainda pode ser qualquer um dos dois, e ninguém desenha fantasma nenhum.
    pub armed: bool,
}

/// Quanto o cursor tem de andar para um `Down` virar arrasto, em px.
///
/// ⚠️ **É o limiar da CASA** ([`super::drag::DRAG_THRESHOLD_PX`]), e ele passou a existir por causa
/// desta wave: o número estava declarado como se fosse só do scrub de caixa numérica, e esta é a
/// segunda pergunta igual — *«a mão andou o suficiente para isto não ser um toque?»*.
///
/// ⛔ A 1.ª redacção deste doc dizia *«é o mesmo número do arrasto da Hierarquia»*. **Era falso:**
/// o arrasto de reparentar não tinha limiar nenhum declarado. *Uma nota que afirma uma constante
/// sem a procurar é um palpite com cara de medição.*
pub const ASSET_DRAG_THRESHOLD_PX: f32 = super::drag::DRAG_THRESHOLD_PX;

impl InFlightDrag {
    /// Começa um arrasto **ainda não armado** — o gesto pode acabar por ser um clique.
    #[must_use]
    pub fn started(payload: DragPayload, at: (f32, f32)) -> Self {
        Self {
            payload,
            cursor: at,
            armed: false,
        }
    }

    /// Move o cursor, armando o arrasto se ele passou o limiar a partir de `origin`.
    ///
    /// ⚠️ **Uma vez armado, fica armado** — voltar para dentro do limiar não desfaz o arrasto, que
    /// é o que todo gestor de arrasto faz e o que impede o gesto de piscar entre clique e arrasto.
    pub fn moved(&mut self, origin: (f32, f32), to: (f32, f32)) {
        self.cursor = to;
        if !self.armed {
            let dx = to.0 - origin.0;
            let dy = to.1 - origin.1;
            self.armed = dx.hypot(dy) >= ASSET_DRAG_THRESHOLD_PX;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⛔ **Um `Down` + `Up` parado continua a ser um CLIQUE.** Sem isto, escolher um cartão no
    /// navegador deixaria de funcionar — e o duplo-clique com ele.
    #[test]
    fn a_still_gesture_never_arms_the_drag() {
        let mut d = InFlightDrag::started(DragPayload::Prefab { stable_id: 1 }, (10.0, 10.0));
        assert!(!d.armed);
        d.moved((10.0, 10.0), (10.0, 10.0));
        assert!(!d.armed, "parado nao pode armar");
        d.moved((10.0, 10.0), (10.0 + ASSET_DRAG_THRESHOLD_PX * 0.9, 10.0));
        assert!(!d.armed, "abaixo do limiar ainda e' um clique");
    }

    /// E passar o limiar arma — em qualquer direcção, porque a distância é radial.
    #[test]
    fn crossing_the_threshold_arms_it_in_any_direction() {
        for (dx, dy) in [
            (ASSET_DRAG_THRESHOLD_PX, 0.0),
            (-ASSET_DRAG_THRESHOLD_PX, 0.0),
            (0.0, ASSET_DRAG_THRESHOLD_PX),
            (
                ASSET_DRAG_THRESHOLD_PX * 0.71,
                ASSET_DRAG_THRESHOLD_PX * 0.71,
            ),
        ] {
            let mut d = InFlightDrag::started(DragPayload::Image { asset: [0; 32] }, (0.0, 0.0));
            d.moved((0.0, 0.0), (dx, dy));
            assert!(d.armed, "({dx}, {dy}) devia armar");
        }
    }

    /// ⚠️ **Uma vez armado, fica armado** — senão o gesto pisca entre clique e arrasto quando a
    /// mão volta ao ponto de partida.
    #[test]
    fn an_armed_drag_never_disarms_by_coming_back() {
        let mut d = InFlightDrag::started(DragPayload::Prefab { stable_id: 1 }, (0.0, 0.0));
        d.moved((0.0, 0.0), (100.0, 100.0));
        assert!(d.armed);
        d.moved((0.0, 0.0), (0.0, 0.0));
        assert!(d.armed, "voltar ao inicio nao desfaz o arrasto");
        assert_eq!(d.cursor, (0.0, 0.0), "mas o cursor acompanha");
    }

    /// ⭐ **A carga DIZ o que é** — é isto que torna a recusa exprimível num `match`, em vez de o
    /// alvo ter de adivinhar a partir de um número.
    #[test]
    fn the_payload_names_itself() {
        assert_eq!(DragPayload::Prefab { stable_id: 7 }.kind_label(), "Prefab");
        assert_eq!(DragPayload::Image { asset: [1; 32] }.kind_label(), "Image");
        assert_ne!(
            DragPayload::Prefab { stable_id: 7 },
            DragPayload::Prefab { stable_id: 8 },
            "duas receitas diferentes nao podem ser a mesma carga"
        );
    }

    /// ⚠️ **O limiar é o da CASA** — uma constante, dois consumidores.
    #[test]
    fn the_threshold_is_the_houses_one() {
        assert_eq!(
            ASSET_DRAG_THRESHOLD_PX,
            crate::interaction::drag::DRAG_THRESHOLD_PX
        );
    }
}
