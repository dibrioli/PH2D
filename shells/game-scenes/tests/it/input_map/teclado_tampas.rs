//! Defeito do teste real (11/11): no cartão da acção da vista Keyboard a tampa do **F** saía VAZIA (só «Keyboard» por
//! baixo). A tampa media o texto no peso de fábrica (Medium) e o cortava e pintava no Regular: o «F» Regular é mais
//! largo, a reticência não cabe, e a elisão apaga a letra inteira. Cada tecla do teclado, sozinha numa acção.

use super::Mesa;
use super::lista::onze;
use super::teclado_censo::AREAS;
use ph2d_app_input_map::View;
use ph2d_app_input_map::view_keyboard::state::KbHit;
use ph2d_input::{Binding, Key};

fn teclas() -> Vec<Key> {
    let letras = ('A'..='Z').chain('0'..='9').map(|c| Key(u32::from(c)));
    let f = (1..=12).map(|n| Key::named_in_script(&format!("f{n}")).expect("F n")[0]);
    letras.chain(f).collect()
}

/// As tampas que saem cortadas ou ausentes, com `k` como a única tecla de `reload`.
fn tampa_vazia(k: Key) -> Option<String> {
    let mut map = onze();
    let reload = map.id("reload").expect("reload");
    map.get_mut(reload).expect("reload").bindings = vec![Binding::Key(k)];
    let mut m = Mesa::aberta(View::Keyboard);
    m.hero.input_map = map;
    m.frame();
    m.click(m.at(KbHit::Row(reload).id()));
    let mut falhas = Vec::new();
    for area in AREAS {
        m.paint_in(area);
        let t = &m.tab().doc.census.texts;
        // A legenda grande da tampa é o ÚLTIMO texto com o nome da tecla (o teclado de cima pinta-a antes).
        match t.iter().rev().find(|(_, s, _)| *s == k.label()) {
            Some((r, _, true)) => falhas.push(format!("{} cortada em {r:?}", k.label())),
            Some(_) => {}
            None => falhas.push(format!("{} ausente", k.label())),
        }
    }
    (!falhas.is_empty()).then(|| falhas.join(", "))
}

#[test]
fn nenhuma_tampa_do_cartao_da_accao_sai_vazia() {
    let mut vazias = Vec::new();
    super::em_cada_modo_de_texto(|| vazias.extend(teclas().into_iter().filter_map(tampa_vazia)));
    assert_eq!(
        vazias,
        Vec::<String>::new(),
        "tampas vazias (os dois modos de texto)"
    );
}

/// Controlo: o fenómeno existe na fixtura — em cada modo de texto há teclas cujo Regular é mais largo que o Medium (qual depende da fonte: «F» no app, «G» na mesa), por
/// isso medir num peso e pintar noutro corta. Sem isto o teste acima podia passar só porque as larguras coincidem.
#[test]
fn controlo_ha_teclas_mais_largas_no_regular_que_no_medium() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let px = 12.0; // LITERAL-PX-OK: o `m::VALUE_PX` da legenda da tampa (a mesa não depende do card-stack)
    let mut por_modo = Vec::new();
    super::em_cada_modo_de_texto(|| {
        let mais_largas: Vec<String> = teclas()
            .into_iter()
            .map(|k| k.label())
            .filter(|s| {
                ts.prefix_width_weighted(s, px, ph2d_text::FontWeight::NORMAL)
                    > ts.prefix_width(s, px)
            })
            .collect();
        por_modo.push(mais_largas);
    });
    assert!(por_modo.iter().all(|v| !v.is_empty()), "{por_modo:?}");
}
