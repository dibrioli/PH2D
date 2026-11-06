//! O nome de uma camada: derivado na tela, nunca guardado no documento.

use ph2d_i18n::{tr, tr_with};
use ph2d_sfx_recipe::{Layer, LayerId, Recipe, Source};

fn mesmo_tipo(a: &Source, b: &Source) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

fn tipo(s: &Source) -> &'static str {
    match s {
        Source::Noise(_) => tr("sfxlab.layer.noise"),
        Source::Tone(_) => tr("sfxlab.layer.tone"),
        Source::Body(_) => tr("sfxlab.layer.body"),
        Source::Recording(_) => tr("sfxlab.layer.recording"),
    }
}

/// «Noise», «Noise 2». A ordem é a dos ids: reordenar a pilha não renomeia ninguém.
#[must_use]
pub fn nome_da_camada(r: &Recipe, l: &Layer) -> String {
    let antes = |id: LayerId| id.0 < l.id.0;
    let n = 1 + r
        .layers
        .iter()
        .filter(|o| mesmo_tipo(&o.source, &l.source) && antes(o.id))
        .count();
    if n == 1 {
        tipo(&l.source).to_owned()
    } else {
        tr_with(
            "sfxlab.layer.numbered",
            &[("name", &tipo(&l.source)), ("n", &n)],
        )
    }
}
