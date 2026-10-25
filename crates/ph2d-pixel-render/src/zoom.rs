//! **Zoom inteiro**: no modo Edit uma célula ocupa um número INTEIRO de píxeis de ecrã, e a roda
//! anda por estes degraus (os de 100 % para cima do Aseprite: 100, 200, …, 6400 %).

/// Os degraus, em píxeis de ecrã por célula.
pub const LEVELS: [u32; 18] = [
    1, 2, 3, 4, 5, 6, 8, 10, 12, 16, 20, 24, 32, 40, 48, 56, 64, 96,
];

/// O degrau mais próximo de `px` (empate → o menor).
#[must_use]
pub fn nearest(px: f64) -> u32 {
    let mut best = LEVELS[0];
    for &l in &LEVELS {
        if (f64::from(l) - px).abs() < (f64::from(best) - px).abs() {
            best = l;
        }
    }
    best
}

/// O degrau seguinte (`up`) ou anterior a partir de `px` — de um zoom fora da escada, o primeiro
/// degrau na direcção pedida.
#[must_use]
pub fn step(px: f64, up: bool) -> u32 {
    const EPS: f64 = 1e-6;
    if up {
        LEVELS
            .iter()
            .copied()
            .find(|&l| f64::from(l) > px + EPS)
            .unwrap_or(LEVELS[LEVELS.len() - 1])
    } else {
        LEVELS
            .iter()
            .rev()
            .copied()
            .find(|&l| f64::from(l) < px - EPS)
            .unwrap_or(LEVELS[0])
    }
}

/// O maior degrau com que um sprite `w × h` cabe numa área `area_w × area_h` de ecrã (o *Fit
/// Screen* do Aseprite, preso à escada).
#[must_use]
pub fn fit(w: u32, h: u32, area_w: f64, area_h: f64) -> u32 {
    let k = (area_w / f64::from(w.max(1))).min(area_h / f64::from(h.max(1)));
    LEVELS
        .iter()
        .rev()
        .copied()
        .find(|&l| f64::from(l) <= k)
        .unwrap_or(LEVELS[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ladder_is_strictly_increasing_integers_from_one() {
        assert_eq!(LEVELS[0], 1);
        assert!(LEVELS.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn step_walks_the_ladder_and_clamps_at_the_ends() {
        assert_eq!(step(8.0, true), 10);
        assert_eq!(step(8.0, false), 6);
        assert_eq!(step(7.3, true), 8);
        assert_eq!(step(7.3, false), 6);
        assert_eq!(step(96.0, true), 96);
        assert_eq!(step(1.0, false), 1);
    }

    #[test]
    fn fit_takes_the_biggest_level_that_fits() {
        assert_eq!(fit(32, 32, 800.0, 600.0), 16);
        assert_eq!(fit(32, 16, 330.0, 1000.0), 10);
        assert_eq!(fit(4096, 4096, 800.0, 600.0), 1);
    }
}
