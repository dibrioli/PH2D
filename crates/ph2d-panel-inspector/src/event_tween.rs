//! **O despacho da secção TWEEN** (suplente #22).
//!
//! ⚠️ **Irmão do [`crate::event`] por CAP de função** — o mesmo padrão das irmãs.
//!
//! # ⚠️ O CLIQUE afirma o contrário do que está no ECRÃ, e o ecrã vem do SNAPSHOT
//!
//! Nunca do store, que guarda o visual. É a lei que a §11 pagou com um report: ler o store fazia o
//! primeiro clique depois de trocar de objecto mandar o valor do objecto **anterior**.
//!
//! # ⭐ E a ESCOLHA DA LINHA não vai ao barramento
//!
//! Qual tween está aberto é um facto da UI e vive no [`crate::state::InspectorState`], como o dos
//! timers, o da vigia e o do gatilho. *Um `Tweens` não tem «o tween actual» — os N correm todos ao
//! mesmo tempo.*

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::TimerFieldEdit;
use ph2d_editor_core::tween_edits::TweenFieldEdit;

/// Despacha um evento da secção TWEEN. `true` = consumido.
pub(crate) fn apply_tween_event(
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
    selected: &mut usize,
) -> bool {
    let Some(info) = crate::state_components::current_inspector_tween() else {
        return false;
    };
    let bits = info.entity_bits;
    // ⚠️ **O índice é o da linha ABERTA, aparado à lista** — uma edição que carregasse um índice
    // fora dela escreveria no tween errado, e a lista pode ter encolhido entre dois quadros.
    let aberto = (*selected).min(info.rows.len().saturating_sub(1));
    let Ok(i) = u8::try_from(aberto) else {
        return false;
    };

    if let WidgetEvent::Click(id) = ev {
        // ⭐ A ESCOLHA da linha — o único evento desta secção que NÃO vai ao barramento.
        if let Some(n) = crate::ids::INSP_TWEEN_ROW.iter().position(|&r| r == id)
            && n < info.rows.len()
        {
            *selected = n;
            return true;
        }
        if id == crate::ids::INSP_TWEEN_ADD {
            push(host, bits, TweenFieldEdit::Add);
            return true;
        }
        if id == crate::ids::INSP_TWEEN_REMOVE {
            push(host, bits, TweenFieldEdit::Remove(i));
            return true;
        }
        // ⚠️ **Os quatro grupos de chips, cada um pela POSIÇÃO no array** — a mesma lei que o
        // `SignalVerb::from_tag` escreve: a posição **é** a tag, e reordenar o array faria um
        // clique escrever outro valor (e compila).
        // ⭐⭐⭐ Os PRESETS — antes dos outros grupos, porque um clique num deles reescreve o que
        // eles mostram.
        if let Some(n) = crate::ids::INSP_TWEEN_PRESET.iter().position(|&c| c == id)
            && let Ok(n) = u8::try_from(n)
        {
            push(host, bits, TweenFieldEdit::Preset(i, n));
            return true;
        }
        for (ids_, faz) in [
            (
                &crate::ids::INSP_TWEEN_CANAL[..],
                &(|i, n| TweenFieldEdit::Canal(i, n)) as &dyn Fn(u8, u8) -> TweenFieldEdit,
            ),
            (&crate::ids::INSP_TWEEN_FAMILIA[..], &|i, n| {
                TweenFieldEdit::Familia(i, n)
            }),
            (&crate::ids::INSP_TWEEN_MODO[..], &|i, n| {
                TweenFieldEdit::Modo(i, n)
            }),
            (&crate::ids::INSP_TWEEN_AO_ACABAR[..], &|i, n| {
                TweenFieldEdit::AoAcabar(i, n)
            }),
            (&crate::ids::INSP_TWEEN_CICLO[..], &|i, n| {
                TweenFieldEdit::Ciclo(i, n)
            }),
        ] {
            if let Some(n) = ids_.iter().position(|&c| c == id)
                && let Ok(n) = u8::try_from(n)
            {
                push(host, bits, faz(i, n));
                return true;
            }
        }
        return false;
    }

    // ⭐⭐⭐ **AS DUAS CAIXAS DO RELÓGIO** — elas escrevem no `Timers[i]`, pela porta que a secção
    // TIMERS já usa.
    //
    // ⚠️⚠️ **Uma caixa emite `Toggled`, NUNCA `Click`**, e é o gate de costura que o diz: a 1.ª
    // redacção pôs os dois braços dentro do ramo do clique e eles ficaram **inalcançáveis**, com o
    // widget vivo sob o dedo e a suíte inteira verde. *Um controlo que responde ao evento errado
    // lê-se exactamente como um controlo morto.*
    //
    // ⚠️ **O estado de partida vem do SNAPSHOT e não do store** — a lei desta secção inteira: ler o
    // store faria o primeiro clique depois de trocar de objecto mandar o valor do objecto ANTERIOR.
    if let WidgetEvent::Toggled(id) = ev
        && let Some(r) = info.rows.get(aberto)
    {
        if id == crate::ids::INSP_TWEEN_REPEAT {
            relogio(host, bits, TimerFieldEdit::Repeat(i, !r.repeat));
            return true;
        }
        if id == crate::ids::INSP_TWEEN_AUTOSTART {
            relogio(host, bits, TimerFieldEdit::Autostart(i, !r.autostart));
            return true;
        }
    }

    if let WidgetEvent::ValueChanged(id) = ev {
        let v = host.store().number_value(id).unwrap_or(0.0);
        #[allow(clippy::cast_possible_truncation)]
        let f = v as f32;
        for (ids_, faz) in [
            (
                &crate::ids::INSP_TWEEN_DE[..],
                &(|i, c, f| TweenFieldEdit::De(i, c, f)) as &dyn Fn(u8, u8, f32) -> TweenFieldEdit,
            ),
            (&crate::ids::INSP_TWEEN_PARA[..], &|i, c, f| {
                TweenFieldEdit::Para(i, c, f)
            }),
        ] {
            if let Some(c) = ids_.iter().position(|&d| d == id)
                && let Ok(c) = u8::try_from(c)
            {
                push(host, bits, faz(i, c, f));
                return true;
            }
        }
        // ⭐⭐⭐ **A DURAÇÃO do relógio deste tween** — em SEGUNDOS, como a da secção TIMERS, e pela
        // MESMA variante: quem satura no `TIMER_MAX_US` continua a ser a shell, num sítio só.
        if id == crate::ids::INSP_TWEEN_DURACAO {
            relogio(host, bits, TimerFieldEdit::DurationSecs(i, f));
            return true;
        }
    }
    false
}

/// ⭐⭐ **A segunda chamadora da porta do relógio** — a primeira é o [`crate::event_timer`].
///
/// ⛔ *Não* é uma segunda superfície sobre um valor: é a MESMA `TimerFieldEdit`, no MESMO índice, e
/// o dreno da shell é um só. É a forma que o teclado e o menu do `project_io` já têm para gravar.
fn relogio(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: TimerFieldEdit) {
    host.bus_mut()
        .push(EditorAction::InspectorTimerEdit { entity_bits, edit });
}

fn push(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: TweenFieldEdit) {
    host.bus_mut().push(EditorAction::InspectorComponentEdit {
        entity_bits,
        edit: ComponentEdit::Tween(edit),
    });
}
