//! Live snapshot published by the shell's mixer bridge each frame, read by
//! the panel painter. Thread-local (UI + shell run on the main thread), mirrors
//! the other panels' publish channels (e.g. `panel-painter-layers`). Split out
//! of `lib.rs` to keep it under the panel LOC cap.
//!
//! ⭐ **A mesa é DOCUMENTO** (F30 §B2): o painel guarda dois retratos da `ph2d_audio::Mesa` — o
//! AUTORADO (o que a mão escreve e o projecto grava) e o VIVO (o autorado com a curva da Cena por
//! cima: o que se pinta e o que o motor ouve). A edição escreve nos dois e marca-se; a ponte
//! (`ph2d_app_audio::mesa_doc`) toma-a para o documento e devolve os dois retratos a cada quadro.
//! O toggle lê o VIVO (o que o artista vê), como o painel da física.

use crate::SUB_BUS_COUNT;
use ph2d_audio::Mesa;
use std::cell::Cell;

/// Per-frame peak-hold decay factor (UI ballistics). The held marker falls
/// by this each paint; a new peak snaps it back up.
const PEAK_HOLD_DECAY: f32 = 0.92; // LITERAL-PX-OK: meter peak-hold decay per frame (ballistics, not a UI dimension)
/// Peak above this (full scale) latches the clip indicator.
const CLIP_THRESHOLD: f32 = 1.0; // LITERAL-PX-OK: clip = exceeds full scale (audio domain)

thread_local! {
    static LEVELS_PEAK: Cell<[f32; 2]> = const { Cell::new([0.0, 0.0]) };
    static LEVELS_RMS: Cell<[f32; 2]> = const { Cell::new([0.0, 0.0]) };
    static LOUDNESS_LUFS: Cell<f32> = const { Cell::new(-120.0) }; // LITERAL-PX-OK: LUFS silence-floor default (audio domain)
    static PEAK_HOLD: Cell<[f32; 2]> = const { Cell::new([0.0, 0.0]) };
    static CLIP: Cell<[bool; 2]> = const { Cell::new([false, false]) };
    static SUB_CLIP: Cell<[[bool; 2]; SUB_BUS_COUNT]> = const { Cell::new([[false, false]; SUB_BUS_COUNT]) };
    // Play Test é ferramenta de AUDIÇÃO da sessão: fica fora da mesa.
    static PLAY_TEST: Cell<bool> = const { Cell::new(false) };
    static SUB_LEVELS_PEAK: Cell<[[f32; 2]; SUB_BUS_COUNT]> = const { Cell::new([[0.0, 0.0]; SUB_BUS_COUNT]) };
    static SUB_LEVELS_RMS: Cell<[[f32; 2]; SUB_BUS_COUNT]> = const { Cell::new([[0.0, 0.0]; SUB_BUS_COUNT]) };
    static SUB_PEAK_HOLD: Cell<[[f32; 2]; SUB_BUS_COUNT]> = const { Cell::new([[0.0, 0.0]; SUB_BUS_COUNT]) };
    static AUTORADO: Cell<Mesa> = Cell::new(Mesa::default());
    static VIVO: Cell<Mesa> = Cell::new(Mesa::default());
    static EDITADO: Cell<bool> = const { Cell::new(false) };
}

/// O que a mesa CORRE agora (o que se pinta e o que o motor ouve).
#[must_use]
pub fn mesa_viva() -> Mesa {
    VIVO.with(Cell::get)
}

/// O que a mão deixou na mesa (o que o projecto grava).
#[must_use]
pub fn mesa_autorada() -> Mesa {
    AUTORADO.with(Cell::get)
}

/// Ponte → painel: os dois retratos do documento, a cada quadro.
pub fn publicar_mesa(autorado: Mesa, vivo: Mesa) {
    AUTORADO.with(|c| c.set(autorado));
    VIVO.with(|c| c.set(vivo));
}

/// Painel → ponte: o autorado, se a mão o mudou desde a última tomada.
#[must_use]
pub fn tomar_edicao() -> Option<Mesa> {
    EDITADO.with(|e| e.replace(false)).then(mesa_autorada)
}

/// A mão escreve `f` no autorado E no vivo (a linha mostra-a já).
fn editar(f: impl Fn(&mut Mesa)) {
    for c in [&AUTORADO, &VIVO] {
        c.with(|c| {
            let mut m = c.get();
            f(&mut m);
            c.set(m);
        });
    }
    EDITADO.with(|e| e.set(true));
}

/// O interruptor `campo` passa ao contrário do que se VÊ; devolve o novo.
fn virar(campo: impl Fn(&mut Mesa) -> &mut bool) -> bool {
    let next = !*campo(&mut mesa_viva());
    editar(|m| *campo(m) = next);
    next
}

fn no_bus(i: usize, f: impl Fn(&mut ph2d_audio::BusMesa)) {
    if i < SUB_BUS_COUNT {
        editar(|m| f(&mut m.bus[i]));
    }
}

fn por_bus<T>(f: impl Fn(&ph2d_audio::BusMesa) -> T) -> [T; SUB_BUS_COUNT] {
    let m = mesa_viva();
    std::array::from_fn(|i| f(&m.bus[i]))
}

/// Decay the held peak toward silence, snapping up to any new peak.
fn hold_step(prev: [f32; 2], peak: [f32; 2]) -> [f32; 2] {
    [
        (prev[0] * PEAK_HOLD_DECAY).max(peak[0]),
        (prev[1] * PEAK_HOLD_DECAY).max(peak[1]),
    ]
}

/// Latch a clip flag pair against this frame's peak (`true` sticks until cleared).
fn latch_clip(prev: [bool; 2], peak: [f32; 2]) -> [bool; 2] {
    [
        prev[0] || peak[0] > CLIP_THRESHOLD,
        prev[1] || peak[1] > CLIP_THRESHOLD,
    ]
}

/// Shell → panel: current master output peak + RMS levels for the meter.
/// Latches the clip indicator when the peak exceeds full scale.
pub fn set_levels(peak: [f32; 2], rms: [f32; 2]) {
    LEVELS_PEAK.with(|c| c.set(peak));
    LEVELS_RMS.with(|c| c.set(rms));
    CLIP.with(|c| c.set(latch_clip(c.get(), peak)));
}

/// The master latched clip flags (cleared by clicking the meter). Public so
/// the clip seam is observable.
pub fn master_clipped() -> [bool; 2] {
    CLIP.with(Cell::get)
}

pub(crate) fn clear_master_clip() {
    CLIP.with(|c| c.set([false, false]));
}

/// The master RMS fill for the meter bar.
pub(crate) fn master_rms() -> [f32; 2] {
    LEVELS_RMS.with(Cell::get)
}

/// Shell → panel: the master momentary loudness in LUFS (for the readout).
pub fn set_loudness(lufs: f32) {
    LOUDNESS_LUFS.with(|c| c.set(lufs));
}

pub(crate) fn loudness() -> f32 {
    LOUDNESS_LUFS.with(Cell::get)
}

/// Advance + return the master peak-hold marker (call once per paint frame).
pub(crate) fn tick_master_hold() -> [f32; 2] {
    let peak = LEVELS_PEAK.with(Cell::get);
    PEAK_HOLD.with(|c| {
        let held = hold_step(c.get(), peak);
        c.set(held);
        held
    })
}

/// Panel → shell: the master gain the fader drives (0..1).
pub(crate) fn set_master_gain(gain: f32) {
    editar(|m| m.ganho = gain);
}

pub fn master_gain() -> f32 {
    mesa_viva().ganho
}

/// Panel → shell: the master low-pass cutoff in Hz (from the Cutoff slider).
pub(crate) fn set_cutoff(hz: f32) {
    editar(|m| m.corte_alto_hz = hz);
}

pub fn cutoff() -> f32 {
    mesa_viva().corte_alto_hz
}

/// Panel → shell: the master high-pass (low-cut) cutoff in Hz (20 Hz = off).
pub(crate) fn set_lowcut(hz: f32) {
    editar(|m| m.corte_baixo_hz = hz);
}

pub fn lowcut() -> f32 {
    mesa_viva().corte_baixo_hz
}

pub fn muted() -> bool {
    mesa_viva().mudo
}

pub(crate) fn toggle_muted() -> bool {
    virar(|m| &mut m.mudo)
}

/// Panel → shell: the master stereo balance (`-1.0`..`1.0`, `0.0` = center).
pub(crate) fn set_master_pan(pan: f32) {
    editar(|m| m.pan = pan);
}

pub fn master_pan() -> f32 {
    mesa_viva().pan
}

/// Panel → shell: whether the built-in test signal should be playing.
pub fn play_test() -> bool {
    PLAY_TEST.with(Cell::get)
}

pub(crate) fn toggle_play_test() -> bool {
    PLAY_TEST.with(|c| {
        let next = !c.get();
        c.set(next);
        next
    })
}

/// Panel → shell: whether the master soft-clip limiter is engaged.
pub fn limiter() -> bool {
    mesa_viva().limitador
}

pub(crate) fn toggle_limiter() -> bool {
    virar(|m| &mut m.limitador)
}

/// Panel → shell: master reverb enable + room size (0..1) + wet/dry mix (0..1).
pub fn reverb_on() -> bool {
    mesa_viva().reverb
}

pub fn reverb_size() -> f32 {
    mesa_viva().reverb_tamanho
}

pub fn reverb_mix() -> f32 {
    mesa_viva().reverb_retorno
}

pub(crate) fn toggle_reverb() -> bool {
    virar(|m| &mut m.reverb)
}

pub(crate) fn set_reverb_size(v: f32) {
    editar(|m| m.reverb_tamanho = v);
}

pub(crate) fn set_reverb_mix(v: f32) {
    editar(|m| m.reverb_retorno = v);
}

/// Panel → shell: master delay enable + time (s) + feedback + return mix.
pub fn delay_on() -> bool {
    mesa_viva().delay
}

pub fn delay_time() -> f32 {
    mesa_viva().delay_tempo
}

pub fn delay_feedback() -> f32 {
    mesa_viva().delay_feedback
}

pub fn delay_mix() -> f32 {
    mesa_viva().delay_retorno
}

pub(crate) fn toggle_delay() -> bool {
    virar(|m| &mut m.delay)
}

pub(crate) fn set_delay_time(v: f32) {
    editar(|m| m.delay_tempo = v);
}

pub(crate) fn set_delay_feedback(v: f32) {
    editar(|m| m.delay_feedback = v);
}

pub(crate) fn set_delay_mix(v: f32) {
    editar(|m| m.delay_retorno = v);
}

/// Panel → shell: ducking enable + depth (every bus ducks under the key bus).
pub fn ducking() -> bool {
    mesa_viva().ducking
}

pub fn duck_depth() -> f32 {
    mesa_viva().ducking_profundidade
}

/// Panel → shell: the sidechain key sub-bus index (everything else ducks under
/// it). Defaults to the last sub-bus (Voice).
pub fn ducking_key() -> usize {
    usize::from(mesa_viva().ducking_chave).min(SUB_BUS_COUNT - 1)
}

pub(crate) fn toggle_ducking() -> bool {
    virar(|m| &mut m.ducking)
}

pub(crate) fn set_duck_depth(v: f32) {
    editar(|m| m.ducking_profundidade = v);
}

/// Choose the sidechain key sub-bus. An index past the last bus is ignored — the choice has
/// exactly [`SUB_BUS_COUNT`] segments, so it cannot happen from the panel.
pub(crate) fn set_duck_key(i: usize) {
    if let Ok(k) = u8::try_from(i)
        && i < SUB_BUS_COUNT
    {
        editar(|m| m.ducking_chave = k);
    }
}

/// Shell → panel: current post-fader peak + RMS per sub-bus. Latches each
/// sub-bus's clip indicator when its peak exceeds full scale.
pub fn set_sub_levels(peak: [[f32; 2]; SUB_BUS_COUNT], rms: [[f32; 2]; SUB_BUS_COUNT]) {
    SUB_LEVELS_PEAK.with(|c| c.set(peak));
    SUB_LEVELS_RMS.with(|c| c.set(rms));
    SUB_CLIP.with(|c| {
        let mut v = c.get();
        for i in 0..SUB_BUS_COUNT {
            v[i] = latch_clip(v[i], peak[i]);
        }
        c.set(v);
    });
}

/// The per-sub-bus latched clip flags.
pub(crate) fn sub_clipped() -> [[bool; 2]; SUB_BUS_COUNT] {
    SUB_CLIP.with(Cell::get)
}

pub(crate) fn clear_sub_clip(i: usize) {
    SUB_CLIP.with(|c| {
        let mut v = c.get();
        if let Some(slot) = v.get_mut(i) {
            *slot = [false, false];
        }
        c.set(v);
    });
}

/// The per-sub-bus RMS fills for the meter bars.
pub(crate) fn sub_rms() -> [[f32; 2]; SUB_BUS_COUNT] {
    SUB_LEVELS_RMS.with(Cell::get)
}

/// Advance + return sub-bus `i`'s peak-hold marker (call once per paint frame).
pub(crate) fn tick_sub_hold(i: usize) -> [f32; 2] {
    let peak = SUB_LEVELS_PEAK.with(Cell::get);
    SUB_PEAK_HOLD.with(|c| {
        let mut held = c.get();
        if let Some(slot) = held.get_mut(i) {
            *slot = hold_step(*slot, peak[i]);
        }
        c.set(held);
        held.get(i).copied().unwrap_or([0.0, 0.0])
    })
}

/// Panel → shell: each sub-bus fader gain (0..1).
pub fn sub_gain() -> [f32; SUB_BUS_COUNT] {
    por_bus(|b| b.ganho)
}

pub(crate) fn set_sub_gain(i: usize, gain: f32) {
    no_bus(i, |b| b.ganho = gain);
}

/// Panel → shell: each sub-bus mute flag (bridge sends gain 0 when engaged).
pub fn sub_muted() -> [bool; SUB_BUS_COUNT] {
    por_bus(|b| b.mudo)
}

pub(crate) fn toggle_sub_muted(i: usize) {
    if i < SUB_BUS_COUNT {
        virar(|m| &mut m.bus[i].mudo);
    }
}

/// Panel → shell: each sub-bus stereo balance (`-1.0`..`1.0`).
pub fn sub_pan() -> [f32; SUB_BUS_COUNT] {
    por_bus(|b| b.pan)
}

pub(crate) fn set_sub_pan(i: usize, pan: f32) {
    no_bus(i, |b| b.pan = pan);
}

/// Panel → shell: each sub-bus low-pass cutoff in Hz (from its Tone slider).
pub fn sub_tone() -> [f32; SUB_BUS_COUNT] {
    por_bus(|b| b.tom_hz)
}

pub(crate) fn set_sub_tone(i: usize, hz: f32) {
    no_bus(i, |b| b.tom_hz = hz);
}

/// Panel → shell: each sub-bus high-pass (low-cut) cutoff in Hz (20 Hz = off).
pub fn sub_lowcut() -> [f32; SUB_BUS_COUNT] {
    por_bus(|b| b.corte_baixo_hz)
}

pub(crate) fn set_sub_lowcut(i: usize, hz: f32) {
    no_bus(i, |b| b.corte_baixo_hz = hz);
}

/// Panel → shell: each sub-bus reverb aux-send amount (0..1; 0 = dry).
pub fn sub_send() -> [f32; SUB_BUS_COUNT] {
    por_bus(|b| b.envio_reverb)
}

pub(crate) fn set_sub_send(i: usize, amount: f32) {
    no_bus(i, |b| b.envio_reverb = amount);
}

/// Panel → shell: each sub-bus delay aux-send amount (0..1; 0 = dry).
pub fn sub_delay_send() -> [f32; SUB_BUS_COUNT] {
    por_bus(|b| b.envio_delay)
}

pub(crate) fn set_sub_delay_send(i: usize, amount: f32) {
    no_bus(i, |b| b.envio_delay = amount);
}

/// Panel → shell: each sub-bus compressor amount (0..1; 0 = off).
pub fn sub_comp() -> [f32; SUB_BUS_COUNT] {
    por_bus(|b| b.comp)
}

pub(crate) fn set_sub_comp(i: usize, amount: f32) {
    no_bus(i, |b| b.comp = amount);
}

/// Panel → shell: the master 3-band EQ slider positions (0..1; 0.5 = flat),
/// `[low, mid, high]`. The shell maps each to ±12 dB.
pub fn eq() -> [f32; 3] {
    mesa_viva().eq
}

pub(crate) fn set_eq(band: usize, pos: f32) {
    if band < 3 {
        editar(|m| m.eq[band] = pos);
    }
}

/// Panel → shell: each sub-bus solo flag. When *any* is set, the shell mutes
/// every non-soloed sub-bus (so you hear only the soloed ones).
pub fn sub_soloed() -> [bool; SUB_BUS_COUNT] {
    por_bus(|b| b.solo)
}

pub(crate) fn toggle_sub_soloed(i: usize) {
    if i < SUB_BUS_COUNT {
        virar(|m| &mut m.bus[i].solo);
    }
}
