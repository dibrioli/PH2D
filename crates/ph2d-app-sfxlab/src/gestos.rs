//! O rato e o teclado da aba. O que se clica foi registrado pela pintura em `Aba::alvos`.

use ph2d_editor_core::doc_surface::{Key, KeyInput, KeyOutcome, PointerLook};
use ph2d_host::PointerKind;
use ph2d_sfx_recipe::LayerId;

use crate::aba::{Aba, Alvo, Gesto};
use crate::metrics as m;

fn alvo_em(a: &Aba, x: f32, y: f32) -> Option<Alvo> {
    // O último pintado está por cima.
    a.alvos
        .iter()
        .rev()
        .find(|(_, r)| r.contains(x, y))
        .map(|(alvo, _)| *alvo)
}

pub(crate) fn ponteiro(a: &mut Aba, kind: PointerKind, x: f32, y: f32) -> bool {
    match kind {
        PointerKind::Down => desce(a, x, y),
        PointerKind::Up => sobe(a, x, y),
        PointerKind::Move => move_(a, x, y),
    }
}

fn desce(a: &mut Aba, x: f32, y: f32) -> bool {
    let alvo = alvo_em(a, x, y);
    let no_corpo = a.corpo.contains(x, y);
    let no_menu = matches!(alvo, Some(Alvo::Item(_) | Alvo::ItemOff(_) | Alvo::Add));
    if a.menu_add && !no_menu {
        a.menu_add = false;
        return no_corpo;
    }
    match alvo {
        Some(Alvo::Onda(id)) => {
            let start0 = a.receita.layer(id).map_or(0.0, |l| l.start);
            a.sel = Some(id);
            a.hist.abre_gesto(&a.receita);
            a.gesto = Some(Gesto::Inicio { id, x0: x, start0 });
        }
        Some(Alvo::Alca(id)) => {
            a.sel = Some(id);
            a.hist.abre_gesto(&a.receita);
            a.gesto = Some(Gesto::Ordem { id });
        }
        Some(Alvo::Linha(id)) => a.sel = Some(id),
        Some(Alvo::Regua) => a.cursor = tempo_em(a, x),
        Some(p) => a.prensa = Some(p),
        // Fora de tudo (a faixa do Mix, por exemplo): os cartões passam a ser os do som inteiro.
        None if no_corpo => a.sel = None,
        None => {}
    }
    no_corpo
}

fn sobe(a: &mut Aba, x: f32, y: f32) -> bool {
    if a.gesto.is_some() {
        a.fecha_gesto();
        return true;
    }
    let prensa = a.prensa.take();
    if prensa.is_some() && prensa == alvo_em(a, x, y) {
        clique(a, prensa);
    }
    a.corpo.contains(x, y)
}

fn move_(a: &mut Aba, x: f32, y: f32) -> bool {
    match a.gesto {
        Some(Gesto::Inicio { id, x0, start0 }) => {
            let pps = a.px_por_s();
            if pps > 0.0 {
                let fim = a.receita.seconds;
                a.edita(|r| {
                    if let Some(l) = r.layer_mut(id) {
                        l.start = (start0 + (x - x0) / pps).clamp(0.0, fim);
                    }
                });
            }
            true
        }
        Some(Gesto::Ordem { id }) => {
            reordena(a, id, y);
            true
        }
        None => {
            a.hover = alvo_em(a, x, y);
            a.corpo.contains(x, y)
        }
    }
}

fn reordena(a: &mut Aba, id: LayerId, y: f32) {
    let passo = m::ROW_H + m::ROW_GAP;
    let n = a.receita.layers.len();
    let linha = ((y - a.linhas.y + a.rolagem) / passo).floor();
    let para = (linha.max(0.0) as usize).min(n.saturating_sub(1));
    a.edita(|r| {
        if let Some(de) = r.layers.iter().position(|l| l.id == id)
            && de != para
        {
            let l = r.layers.remove(de);
            r.layers.insert(para, l);
        }
    });
}

fn clique(a: &mut Aba, alvo: Option<Alvo>) {
    match alvo {
        Some(Alvo::Play) => a.quer_tocar = true,
        Some(Alvo::Stop) => para(a),
        Some(Alvo::PlayOnChange) => a.play_on_change = !a.play_on_change,
        Some(Alvo::Mudo(id)) => a.edita(|r| {
            if let Some(l) = r.layer_mut(id) {
                l.muted = !l.muted;
            }
        }),
        Some(Alvo::Solo(id)) => a.alterna_solo(id),
        Some(Alvo::Add) => a.menu_add = !a.menu_add,
        Some(Alvo::Item(f)) => {
            a.menu_add = false;
            a.acrescenta(f);
        }
        _ => {}
    }
}

fn para(a: &mut Aba) {
    a.quer_tocar = false;
    a.quer_parar = true;
}

fn tempo_em(a: &Aba, x: f32) -> f32 {
    let pps = a.px_por_s();
    if pps > 0.0 {
        ((x - a.pista.x) / pps).clamp(0.0, a.receita.seconds)
    } else {
        0.0
    }
}

pub(crate) fn tecla(a: &mut Aba, k: &KeyInput<'_>) -> KeyOutcome {
    let tomada = |consumed| KeyOutcome {
        consumed,
        copy: None,
    };
    if k.mods.ctrl || k.mods.alt {
        return tomada(false);
    }
    match k.key {
        Key::Space => {
            if k.pressed {
                a.quer_tocar = true;
            }
            tomada(true)
        }
        Key::Escape => {
            if k.pressed {
                if a.menu_add {
                    a.menu_add = false;
                } else {
                    para(a);
                }
            }
            tomada(true)
        }
        Key::Delete => {
            if k.pressed
                && a.gesto.is_none()
                && let Some(id) = a.sel
            {
                a.apaga(id);
            }
            tomada(true)
        }
        _ => tomada(false),
    }
}

pub(crate) fn roda(a: &mut Aba, x: f32, y: f32, dy: f32) -> bool {
    if !a.linhas.contains(x, y) {
        return false;
    }
    a.rolagem = (a.rolagem - dy * m::WHEEL_LINE_PX).clamp(0.0, a.rolagem_max);
    true
}

pub(crate) fn aparencia(a: &Aba, x: f32, y: f32) -> PointerLook {
    if a.gesto.is_some() {
        return PointerLook::Grabbing;
    }
    match alvo_em(a, x, y) {
        Some(Alvo::Onda(_) | Alvo::Alca(_)) => PointerLook::Grab,
        Some(Alvo::Linha(_) | Alvo::ItemOff(_)) | None => PointerLook::Arrow,
        Some(_) => PointerLook::Hand,
    }
}
