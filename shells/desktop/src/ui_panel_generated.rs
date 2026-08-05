//! **O painel GERADO da cena de smoke, COMPILADO** (plano UI/UX W8b).
//!
//! ⚠️ Este módulo existe por uma razão só, e ela é o risco §10.4 do plano — *"codegen que o CI
//! recusa"*. O arquivo abaixo é a saída literal do gerador, commitada; incluí-lo aqui faz o
//! **compilador** ser o juiz de que ela é Rust válido que referencia o catálogo REAL. Um gerador
//! que emitisse `WidgetKind::Slidr` não chega ao `main`: o build do teste cai antes, e o gate de
//! staleness (`the_generated_panel_is_not_stale`) é quem garante que este arquivo é o que o
//! gerador ainda produz.
//!
//! ⚠️ **`cfg(test)` de propósito:** nada no PRODUTO lê estas consts ainda — quem as vai ler é o
//! runtime de rows do W8b.2. Compilá-las no build normal produziria `dead_code`, e o `allow` que
//! o calaria esconderia a única coisa que o aviso diz de verdade: *esta wave emite o artefato, e a
//! seguinte o liga*.
//!
//! ⚠️ **E quem as lê é o gate** (`the_compiled_golden_carries_the_same_panel`), não um `allow`.
//! Isso não é higiene: ele confere o conteúdo **depois de o compilador ter aceitado o arquivo**,
//! que é uma pergunta que a comparação de bytes não faz.

use ph2d_editor::widget::WidgetKind;

include!("generated/ui_panel_demo.rs");
