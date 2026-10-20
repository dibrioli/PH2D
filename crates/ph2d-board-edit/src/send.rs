//! ⭐ B15 — **de um quadro para outro** (pedido do dono 20/10; `docs/MiroClone/17_bloco6_plano.md`
//! §2.4). Nenhum produto tem o gesto (o Miro: *«copy, and paste it onto another board»*): a lei é a
//! da colagem noutro quadro ([`Editor::paste`]) — o que não foi não existe lá — sem passar pela área
//! de transferência do artista.

use ph2d_board_model::{BoardDoc, Element, Files, History};

use crate::Editor;
use crate::command::Landing;

/// O que a selecção leva: os elementos (com o que vive nos frames dela, as setas que saem dela já
/// soltas) e os bytes das imagens deles.
#[derive(Clone, Debug)]
pub struct Parcel {
    els: Vec<Element>,
    files: Files,
    picked: usize,
}

impl Parcel {
    /// Quantos objectos o artista seleccionou (o que o aviso conta).
    #[must_use]
    pub fn picked(&self) -> usize {
        self.picked
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.els.is_empty()
    }
}

impl Editor {
    /// A selecção de `doc` pronta a ir (a mesma que um `Ctrl+C` leva); `files` = os do quadro dela.
    pub fn lift(&mut self, doc: &BoardDoc, files: &Files) -> Parcel {
        let mut els = self.selected_with_contents(doc);
        self.detach_outside(doc, &mut els);
        let files = els
            .iter()
            .filter_map(Element::image)
            .filter_map(|i| Some((i.file, files.get(&i.file)?.clone())))
            .collect();
        Parcel {
            els,
            files,
            picked: self.selection.len(),
        }
    }

    /// Pousa `parcel` em `doc` (OUTRO quadro): centrada em `centre`, seleccionada, UM passo; os bytes
    /// das imagens entram em `files`. `false` = nada pousou.
    pub fn land(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        files: &mut Files,
        parcel: &Parcel,
        centre: [f64; 2],
    ) -> bool {
        for (k, f) in &parcel.files {
            files.entry(*k).or_insert_with(|| f.clone());
        }
        self.place_copies(doc, history, &parcel.els, Landing::Elsewhere(Some(centre), 0))
    }
}
