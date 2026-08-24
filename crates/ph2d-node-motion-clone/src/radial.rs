//! **O LEQUE** — o modo `Radial` do `motion.clone` (doc 89 folha 04, a célula *Setor* do
//! `motion.kaleidoscope`).
//!
//! ⚠️ **Esta wave nasceu de uma RECUSA que apontava para um dono inexistente.** A célula do
//! setor dizia, com razão, que um leque é *"trabalho do Cloner/duplicator, não do
//! caleidoscópio"* — e a medição de 2026-08-24 mostrou que **nenhum nó do catálogo o fazia**:
//! o `motion.clone` dispunha as cópias numa RETA (`rank · distance · (cos θ, sin θ)`), o
//! `motion.duplicator` espalha por pontos e o `motion.mirror` reflecte. *Uma recusa que
//! delega para um dono que não entrega é um adiamento com cara de decisão.* ⇒ a capacidade
//! foi construída onde a própria célula disse que ela pertence, e só então a célula fechou.
//!
//! ## A lei
//!
//! A cópia `c` é o layout **rodado em torno de um pivô** por `angle + rank_c · (arc / k)`, e
//! depois empurrado para fora por `distance` na direcção desse mesmo ângulo:
//!
//! ```text
//! p' = pivot + R(θ_c) · (p − pivot) + distance · (cos θ_c, sin θ_c)
//! ```
//!
//! ⚠️ **E as duas parcelas são UMA**, o que a escrita acima esconde: `d·(cos θ, sin θ)` é
//! `R(θ)·(d, 0)`, logo o todo é `pivot + R(θ)·((p − pivot) + (d, 0))` — *empurra em +X,
//! depois gira*. Daí sai o invariante que os gates medem: **todas as cópias partilham um raio**
//! (uma rotação preserva comprimento), e esse raio é `|(p − pivot) + (d, 0)|`, que só vale `d`
//! quando a peça está POUSADA no pivô. Um gate que exigisse `d` estaria a medir a fixtura.
//!
//! ⚠️ **O passo é `arc / k` e NÃO `arc / (k−1)`** — o `arc` é o setor que as cópias
//! REPARTEM, uma por fatia. É a lei que a célula já tinha derivado (*"`start = 0, end = 360`
//! ⇒ `s · (1/k)` exato"*), e a única que não precisa de um caso especial: com `arc/(k−1)` um
//! giro completo poria a última cópia **em cima da primeira**, e um `if arc == 360` para o
//! evitar seria uma descontinuidade dentro do curso do próprio knob.
//!
//! ⚠️ **`distance` é o RAIO, e é o que impede o modo de ter um buraco.** Com a lei só de
//! rotação, um elemento POUSADO no pivô tem raio zero e as `k` cópias coincidem — e *"pegar
//! nesta coisa e pôr oito à volta de um círculo"* é precisamente o gesto canónico de um
//! cloner radial. Reusar o `distance` (que já é um `Length` e já se chama distância) dá o
//! raio de graça, e mantém o knob VIVO nos dois modos em vez de o deixar morto num deles.
//!
//! ⚠️ **`angle` continua a dizer *para onde o padrão aponta*, nos dois modos** — em `Linear` é
//! a direcção da fila, em `Radial` é onde a primeira cópia começa. Não é o mesmo número a
//! significar duas coisas: é a mesma PERGUNTA respondida na geometria de cada modo, e é o que
//! deixa o `center` continuar ortogonal (ele decide se a fila/leque sai do original ou o
//! ladeia, em ambos).
//!
//! ⚠️ **A cópia NÃO gira sobre si mesma, e o precedente é o irmão:** o
//! `motion.kaleidoscope` — o replicador rotacional que já existia — escreve **só `P`**
//! (medido: `out.set` toca `P`, `Index` e `Count`, nunca `rot`). Um carrossel de peças
//! direitas é o que sai, e a roseta em que cada peça olha para fora é
//! `motion.look_at(pivot)` a jusante — um nó que já existe e cuja razão de ser é exactamente
//! essa. ⛔ Escrever `rot` aqui daria à casa **dois** replicadores rotacionais que discordam
//! sobre o que replicar significa.
//!
//! ⚠️ **O taper é ortogonal ao modo**: `scale_taper`/`rot_taper` correm sobre a ORDINAL da
//! cópia (`taper_t`), que não sabe nem quer saber por onde as cópias andam.

use super::trig::cos_sin_cycles;

/// A fila numa recta — o modo que sempre shipou.
pub(super) const MODE_LINEAR: i32 = 0;
/// O leque em torno de um pivô.
pub(super) const MODE_RADIAL: i32 = 1;
/// As palavras da referência (C4D Cloner ▸ Mode), na ordem dos números acima.
pub(super) const MODE_LABELS: &[&str] = &["Linear", "Radial"];

/// Graus por volta — o divisor exacto do ângulo autorado para a unidade da trig em ciclos.
const DEG_PER_TURN: f32 = 360.0;

/// **Onde a cópia `c` põe cada elemento.** Uma pergunta, duas geometrias.
#[derive(Clone, Copy)]
pub(super) enum Placement {
    /// `p + rank · (sx, sy)` — a expressão que sempre shipou, intocada.
    Linear { dx: f32, dy: f32 },
    /// `pivot + R(θ)·(p − pivot) + distance·(cos θ, sin θ)`.
    Radial {
        c: f32,
        s: f32,
        pivot: [f32; 2],
        distance: f32,
    },
}

impl Placement {
    /// A colocação da cópia de posto `rank` (ordinal `copy` de `k`).
    ///
    /// ⚠️ **O `rank` assinado entra nos DOIS modos pelo mesmo sítio**, que é o que mantém o
    /// `center` a significar uma coisa só: com ele desligado o posto `0` é o próprio original
    /// (deslocamento nulo / rotação nula), e com ele ligado o padrão ladeia o original.
    pub(super) fn of(
        radial: bool,
        rank: f32,
        step_deg: f32,
        angle_deg: f32,
        distance: f32,
        pivot: [f32; 2],
    ) -> Self {
        if radial {
            let (c, s) = cos_sin_cycles((angle_deg + rank * step_deg) / DEG_PER_TURN);
            Self::Radial {
                c,
                s,
                pivot,
                distance,
            }
        } else {
            let (c, s) = cos_sin_cycles(angle_deg / DEG_PER_TURN);
            Self::Linear {
                dx: rank * (distance * c),
                dy: rank * (distance * s),
            }
        }
    }

    /// Onde este elemento vai parar.
    pub(super) fn apply(self, p: [f32; 2]) -> [f32; 2] {
        match self {
            Self::Linear { dx, dy } => [p[0] + dx, p[1] + dy],
            Self::Radial {
                c,
                s,
                pivot,
                distance,
            } => {
                let (dx, dy) = (p[0] - pivot[0], p[1] - pivot[1]);
                [
                    pivot[0] + (dx * c - dy * s) + distance * c,
                    pivot[1] + (dx * s + dy * c) + distance * s,
                ]
            }
        }
    }
}

/// O passo angular entre cópias vizinhas, em graus — ver o cabeçalho.
///
/// ⚠️ **`k = 0` não acontece** (o `copies_within_budget` garante ≥ 1), e `k = 1` dá o `arc`
/// inteiro para uma cópia só, que nunca é usado porque o único posto é `0`.
pub(super) fn step_deg(arc_deg: f32, k: usize) -> f32 {
    #[expect(clippy::cast_precision_loss, reason = "contagem de cópias, ≤ 2^24")]
    let n = k.max(1) as f32;
    arc_deg / n
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **O modo Linear é a expressão de sempre, AO BIT** — o caminho por onde passa todo
    /// grafo já autorado.
    #[test]
    fn the_linear_placement_is_the_offset_that_shipped_bit_for_bit() {
        let (c, s) = cos_sin_cycles(37.0 / DEG_PER_TURN);
        for rank in [0.0_f32, 1.0, -1.5, 4.0] {
            let (dx, dy) = (rank * (2.5 * c), rank * (2.5 * s));
            let pl = Placement::of(false, rank, 999.0, 37.0, 2.5, [9.0, -9.0]);
            for p in [[0.0_f32, 0.0], [1.25, -3.5]] {
                let now = pl.apply(p);
                let then = [p[0] + dx, p[1] + dy];
                assert_eq!(
                    (now[0].to_bits(), now[1].to_bits()),
                    (then[0].to_bits(), then[1].to_bits()),
                    "posto {rank}, ponto {p:?}"
                );
            }
        }
        // ⚠️ E o `arc`/`pivot` que passei acima são LIXO de propósito: em Linear eles não
        // podem tocar no resultado, e este gate morre se alguém os ligar ao caminho antigo.
    }

    /// **Posto `0` sem raio é a IDENTIDADE, também no leque** — o original continua a estar
    /// onde estava, que é o invariante que o `center` desligado promete nos dois modos.
    #[test]
    fn rank_zero_with_no_radius_is_the_input_itself() {
        let pl = Placement::of(true, 0.0, 45.0, 0.0, 0.0, [1.0, 2.0]);
        let q = pl.apply([3.0, -4.0]);
        assert!(
            (q[0] - 3.0).abs() < 1e-5 && (q[1] + 4.0).abs() < 1e-5,
            "o original moveu-se: {q:?}"
        );
    }

    /// ⭐ **O RAIO é o que faz o gesto canónico funcionar:** uma peça POUSADA no pivô vira
    /// `k` peças num círculo, em vez de `k` peças coincidentes.
    #[test]
    fn a_piece_sitting_on_the_pivot_still_fans_out_because_distance_is_the_radius() {
        let k = 4;
        let step = step_deg(360.0, k);
        let places: Vec<[f32; 2]> = (0..k)
            .map(|c| Placement::of(true, c as f32, step, 0.0, 2.0, [0.0, 0.0]).apply([0.0, 0.0]))
            .collect();
        for (i, q) in places.iter().enumerate() {
            let r = q[0].hypot(q[1]);
            assert!(
                (r - 2.0).abs() < 0.05,
                "copia {i} fora do raio: {q:?} (r={r:.3})"
            );
        }
        // E elas são QUATRO pontos distintos, não quatro cópias do mesmo.
        for i in 0..k {
            for j in (i + 1)..k {
                let d = (places[i][0] - places[j][0]).hypot(places[i][1] - places[j][1]);
                assert!(d > 1.0, "copias {i} e {j} coincidiram ({d:.4})");
            }
        }
    }

    /// **O passo reparte o setor, então uma volta inteira NÃO repete a primeira cópia.**
    #[test]
    fn a_full_turn_never_lands_the_last_copy_on_the_first() {
        for k in [2_usize, 3, 5, 8] {
            let step = step_deg(360.0, k);
            assert!(
                (step * k as f32 - 360.0).abs() < 1e-3,
                "as {k} fatias tinham de fechar a volta"
            );
            // A última cópia está a UM passo de fechar, nunca em cima da primeira.
            let last = step * (k - 1) as f32;
            assert!(
                last < 360.0 - step * 0.5,
                "k={k}: a ultima fechou o circulo"
            );
        }
    }

    /// **Um setor parcial reparte-se pelo mesmo divisor** — sem caso especial em `360`.
    #[test]
    fn a_partial_sector_uses_the_same_divisor() {
        assert!((step_deg(90.0, 3) - 30.0).abs() < 1e-4);
        assert!(
            (step_deg(-180.0, 4) + 45.0).abs() < 1e-4,
            "e o sinal inverte o leque"
        );
    }
}
