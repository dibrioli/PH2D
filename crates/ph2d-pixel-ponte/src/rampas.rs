//! **As rampas e o ciclo da paleta** (P13 `P13_plano.md` §4, §6) do lado da família: o instantâneo
//! do painel, o gesto → `Op` e o *New Palette from Sprite* que LEVA as rampas (a rampa é dado
//! autorado: a paleta nova = as cores usadas ∪ as das rampas, remapeadas no mesmo passo).

use std::collections::{BTreeMap, BTreeSet};

use ph2d_editor_core::pixel_panel::{
    PixelCycle, PixelCycleDirection, PixelRampAction, PixelRampConfig, PixelRampInfo,
};
use ph2d_pixel_core::palette::MAX_COLORS;
use ph2d_pixel_core::{
    Cel, ColorMode, Cycle, CycleDirection, DocOp, Op, PaletteRamps, RampId, Sprite, Whole,
};
use ph2d_pixel_rampa::{ConfigRampa, gera_rampa, op_rampa_nova};

const fn dir(d: CycleDirection) -> PixelCycleDirection {
    match d {
        CycleDirection::Forward => PixelCycleDirection::Forward,
        CycleDirection::Reverse => PixelCycleDirection::Reverse,
        CycleDirection::PingPong => PixelCycleDirection::PingPong,
    }
}

const fn direcao(d: PixelCycleDirection) -> CycleDirection {
    match d {
        PixelCycleDirection::Forward => CycleDirection::Forward,
        PixelCycleDirection::Reverse => CycleDirection::Reverse,
        PixelCycleDirection::PingPong => CycleDirection::PingPong,
    }
}

/// As rampas do sprite como o painel as mostra.
#[must_use]
pub fn info(s: &Sprite) -> Vec<PixelRampInfo> {
    let p = s.ramps();
    p.ramps
        .iter()
        .map(|r| PixelRampInfo {
            id: r.id.0,
            indices: r.indices.clone(),
            cycle: p
                .cycles
                .iter()
                .find(|c| c.ramp == r.id)
                .map(|c| PixelCycle {
                    period_ms: c.periodo_ms,
                    direction: dir(c.direcao),
                }),
        })
        .collect()
}

/// Os campos inteiros do diálogo → a configuração do gerador.
#[must_use]
pub fn config(c: PixelRampConfig) -> ConfigRampa {
    let pct = |v: u8| f32::from(v) / 100.0;
    ConfigRampa {
        passos: usize::from(c.steps.max(1)),
        desvio_matiz_graus: f32::from(c.hue_shift_deg),
        curva_croma: pct(c.chroma_pct),
        faixa_l: c.light_pct.map(pct),
    }
}

fn doc(d: DocOp) -> Op {
    Op::Doc(Box::new(d))
}

/// O gesto do painel → UM `Op` (a base do New Ramp é a cor primária `fg`). O ciclo só se liga em
/// Indexed (em RGB o pixel guarda a cor, não o índice).
#[must_use]
pub fn op(s: &Sprite, fg: [u8; 4], a: &PixelRampAction) -> Option<Op> {
    match a {
        PixelRampAction::New(c) => op_rampa_nova(s, "", &gera_rampa(fg, config(*c))),
        PixelRampAction::Make(ix) => (!ix.is_empty()).then(|| {
            doc(DocOp::AddRamp {
                name: String::new(),
                indices: ix.clone(),
            })
        }),
        PixelRampAction::Remove(id) => Some(doc(DocOp::RemoveRamp(RampId(*id)))),
        PixelRampAction::Cycle { ramp, cycle } => {
            let ramp = RampId(*ramp);
            let cycle = cycle.map(|c| Cycle {
                ramp,
                periodo_ms: c.period_ms.max(1),
                direcao: direcao(c.direction),
            });
            (cycle.is_none() || s.mode() == ColorMode::Indexed)
                .then(|| doc(DocOp::SetCycle { ramp, cycle }))
        }
    }
}

/// As rampas levadas para a paleta nova `colors`: cada índice de rampa fica com `novo[i]` (o índice
/// novo da sua cor) se nenhuma outra entrada de rampa o tomou; senão a cor dele ganha um lugar no
/// fim. Entrada que já não cabe nas 256 sai da rampa; rampa vazia sai com o ciclo.
fn leva(s: &Sprite, colors: &mut Vec<[u8; 4]>, novo: &[Option<u8>]) -> PaletteRamps {
    let mut out = s.ramps().clone();
    let mut tomado = [false; MAX_COLORS];
    for r in &mut out.ramps {
        r.indices.retain_mut(|i| {
            let livre = novo[usize::from(*i)].filter(|n| !tomado[usize::from(*n)]);
            let n = livre.or_else(|| {
                let c = *s.palette().get(usize::from(*i))?;
                (colors.len() < MAX_COLORS).then(|| colors.push(c))?;
                u8::try_from(colors.len() - 1).ok()
            });
            let Some(n) = n else { return false };
            tomado[usize::from(n)] = true;
            *i = n;
            true
        });
    }
    let vivas: BTreeSet<RampId> = out
        .ramps
        .iter()
        .filter(|r| !r.indices.is_empty())
        .map(|r| r.id)
        .collect();
    out.ramps.retain(|r| vivas.contains(&r.id));
    out.cycles.retain(|c| vivas.contains(&c.ramp));
    out
}

/// A paleta muda e as rampas seguem no MESMO passo: tira-as, troca a paleta, põe as remapeadas
/// (o `Op::Whole`/`Op::Palette` recusam a paleta que parte uma rampa).
fn embrulha(s: &Sprite, op: Op, rampas: PaletteRamps) -> Op {
    if s.ramps().ramps.is_empty() {
        return op;
    }
    let vazias = PaletteRamps {
        ramps: Vec::new(),
        cycles: Vec::new(),
        ..s.ramps().clone()
    };
    Op::Batch(vec![
        doc(DocOp::SetRamps(vazias)),
        op,
        doc(DocOp::SetRamps(rampas)),
    ])
}

/// **New Palette from Sprite** em Indexed: a transparente primeiro, as usadas (`usados`, sem
/// repetir cor) e as cores das rampas; as células seguem as cores. Devolve o `Op` e o tamanho novo.
#[must_use]
pub fn paleta_do_sprite(s: &Sprite, usados: &[usize]) -> (Op, usize) {
    let t = usize::from(s.transparent_index());
    let mut colors = vec![s.palette().get(t).copied().unwrap_or([0; 4])];
    let mut por_cor: BTreeMap<[u8; 4], u8> = BTreeMap::new();
    let mut novo = [None; MAX_COLORS];
    novo[t] = Some(0);
    let mut mapa = [0u8; MAX_COLORS];
    for &i in usados.iter().filter(|&&i| i != t) {
        let c = s.palette().get(i).copied().unwrap_or([0; 4]);
        let n = *por_cor.entry(c).or_insert_with(|| {
            colors.push(c);
            u8::try_from(colors.len() - 1).unwrap_or(u8::MAX)
        });
        (mapa[i], novo[i]) = (n, Some(n));
    }
    let rampas = leva(s, &mut colors, &novo);
    let cels = s
        .entries()
        .map(|(k, c)| {
            let data = c.data.iter().map(|&v| mapa[usize::from(v)]).collect();
            (*k, Cel { data, ..c.clone() })
        })
        .collect();
    let mut tilesets = s.tilesets().to_vec();
    for v in tilesets.iter_mut().flat_map(|t| &mut t.tiles).flatten() {
        *v = mapa[usize::from(*v)];
    }
    let n = colors.len();
    let w = Whole {
        mode: ColorMode::Indexed,
        palette: colors,
        transparent: 0,
        cels,
        tilesets,
    };
    (embrulha(s, Op::Whole(Box::new(w)), rampas), n)
}

/// **New Palette from Sprite** em RGB: as cores vistas (`colors`) e as das rampas, as rampas
/// remapeadas pela cor. Devolve o `Op` e o tamanho novo.
#[must_use]
pub fn paleta_rgb(s: &Sprite, mut colors: Vec<[u8; 4]>) -> (Op, usize) {
    let mut onde: BTreeMap<[u8; 4], u8> = BTreeMap::new();
    for (i, c) in colors.iter().enumerate().rev() {
        onde.insert(*c, u8::try_from(i).unwrap_or(u8::MAX));
    }
    let novo: Vec<Option<u8>> = (0..MAX_COLORS)
        .map(|i| onde.get(s.palette().get(i)?).copied())
        .collect();
    let rampas = leva(s, &mut colors, &novo);
    let n = colors.len();
    let op = Op::Palette {
        colors,
        transparent: 0,
    };
    (embrulha(s, op, rampas), n)
}
