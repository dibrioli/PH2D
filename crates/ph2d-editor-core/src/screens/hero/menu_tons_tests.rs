use super::*;
use crate::widget::lista_flutuante::force_cromo_on_this_thread;

/// ⭐⭐ **O CONTRASTE dos menus da barra, nos 12 temas, nas duas UIs** (onda 8): rótulo, atalho e o texto
/// da linha acesa ≥ 4,5:1; ícone e linha acesa ≥ 3:1 contra o fundo; o risco ≥ 1,5:1. Portão permanente.
#[test]
fn os_menus_da_barra_tem_contraste_em_todos_os_temas() {
    let mut falhas = Vec::new();
    let mut minimos = [f64::MAX; 6];
    for cartoes in [true, false] {
        force_cromo_on_this_thread(Some(cartoes));
        for theme in Theme::ALL {
            let t = tons_do_menu(theme);
            let pares = [
                ("rótulo", t.rotulo, t.fundo, TEXTO_MIN),
                ("atalho", t.atalho, t.fundo, TEXTO_MIN),
                ("ícone", t.icone, t.fundo, CONTROLO_MIN),
                ("aceso", t.aceso, t.fundo, CONTROLO_MIN),
                ("texto aceso", t.aceso_texto, t.aceso, TEXTO_MIN),
                ("risco", t.risco, t.fundo, PLACA_MIN),
            ];
            for (i, (o_que, fg, bg, min)) in pares.into_iter().enumerate() {
                let r = razao(fg, bg);
                minimos[i] = minimos[i].min(r);
                if r < min {
                    falhas.push(format!("{theme:?} cartões={cartoes} {o_que}: {r:.2}"));
                }
            }
            let r = razao(atalho_aceso(&t), t.aceso);
            if r < TEXTO_MIN {
                falhas.push(format!("{theme:?} cartões={cartoes} atalho aceso: {r:.2}"));
            }
        }
    }
    force_cromo_on_this_thread(None);
    eprintln!("menus: mínimos rótulo·atalho·ícone·aceso·texto aceso·risco = {minimos:.2?}");
    assert!(falhas.is_empty(), "{falhas:#?}");
}
