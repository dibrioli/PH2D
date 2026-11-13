use super::*;
use crate::widget::lista_flutuante::force_cromo_on_this_thread;

/// ⭐⭐ **O CONTRASTE da faixa, nos 12 temas, na UI de cartões** (dono 12/11): o campo, a marca e
/// o Play ≥ 3:1 contra o chão; cada texto ≥ 4,5:1 contra o fundo dele. Portão permanente.
#[test]
fn a_faixa_tem_contraste_em_todos_os_temas() {
    force_cromo_on_this_thread(Some(true));
    let mut falhas = Vec::new();
    let (mut min_ctl, mut min_txt) = (f64::MAX, f64::MAX);
    for theme in Theme::ALL {
        let t = tons(theme);
        let controlos = [
            ("contorno/chão", t.contorno, t.chao),
            ("contorno/campo", t.contorno, t.campo),
            ("marca/chão", t.marca, t.chao),
            ("play/chão", t.play, t.chao),
        ];
        let textos = [
            ("texto/chão", t.texto, t.chao),
            ("texto/campo", t.texto, t.campo),
            ("apagado/campo", t.apagado, t.campo),
            ("tecla/plaquinha", t.tecla_texto, t.tecla),
            ("marca/texto", t.marca_texto, t.marca),
            ("play/texto", t.play_texto, t.play),
        ];
        let r = razao(t.tecla, t.campo);
        if r < PLACA_MIN {
            falhas.push(format!("{theme:?} plaquinha/campo: {r:.2}"));
        }
        for (o_que, fg, bg) in controlos {
            let r = razao(fg, bg);
            min_ctl = min_ctl.min(r);
            if r < CONTROLO_MIN {
                falhas.push(format!("{theme:?} {o_que}: {r:.2}"));
            }
        }
        for (o_que, fg, bg) in textos {
            let r = razao(fg, bg);
            min_txt = min_txt.min(r);
            if r < TEXTO_MIN {
                falhas.push(format!("{theme:?} {o_que}: {r:.2}"));
            }
        }
    }
    force_cromo_on_this_thread(None);
    eprintln!("faixa: contraste mínimo controlo {min_ctl:.2}, texto {min_txt:.2}");
    assert!(falhas.is_empty(), "{falhas:#?}");
}
