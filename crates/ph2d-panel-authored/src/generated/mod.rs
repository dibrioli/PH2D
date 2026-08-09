//! **A tabela GERADA, compilada** (plano UI/UX W8b.2).
//!
//! ⚠️ O arquivo incluído abaixo é a saída literal do `ph2d-ui-codegen` para a moldura que a cena
//! `PH2D_BUILD_SMOKE=62` desenha, commitada. Ele é **código de produto**: as consts que ele
//! declara são a lista que o [`crate::rows`] percorre, e é isso que faz do compilador o juiz de
//! que o gerador emite Rust válido contra o catálogo REAL — um `WidgetKind::Slidr` não chega ao
//! `main`, porque o build cai antes.
//!
//! ⚠️ **Na W8b.1 ele era compilado sob `cfg(test)`** e as consts não tinham leitor: aquilo provava
//! que o texto era válido e nada mais. Agora ele é a fonte da lista viva — a mesma prova, mais
//! forte, porque um formato que o runtime não consegue percorrer também deixa de compilar.
//!
//! ⚠️ **`WidgetKind` e `RowConst` têm de estar em escopo AQUI**, porque o gerado os nomeia sem
//! importar: ele não sabe onde será incluído, e um `use` dentro dele fixaria um caminho que só
//! serve a um hospedeiro. É a mesma relação com os dois — o emissor escreve o nome, o hospedeiro
//! diz de onde ele vem, e o compilador confere.

use crate::rows::RowConst;
use ph2d_editor_core::widget::WidgetKind;

include!("panel.rs");
