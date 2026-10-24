//! `PH2D_MACRO_SMOKE=8` — **o exemplo do dono** (W19, `docs/Macros/17_W19_plano.md` §7): «começar com uma
//! pintura de um player, acrescentar um collider e simular a física». Um chão (Static + Collider) fora do
//! objecto Macro; Add ▸ Macro, Gravar, uma imagem nova, três pinceladas (o boneco), Add Physics Body,
//! Physics + Play (cai), Pause, Parar, Guardar na biblioteca, apagar; o chão sai e nasce outro mais
//! abaixo (não há «Ficheiro ▸ Novo projecto» que a cena conduza); Aplicar ▸ Criar novo e Play: a imagem
//! nasce, pinta-se, ganha o collider e cai sobre o chão novo.
//!
//! Corre no condutor da cena 7 ([`crate::cena_da_biblioteca`]): os mesmos [`Passo`], portas e sonda
//! (`tests/it/cena_do_mundo.rs`, o gate `a_cena_8_percorre_o_caminho_inteiro`).

use crate::cena_da_biblioteca::{PASSO_MS, Passo, Qual};

/// O lado da imagem do player (px).
pub const LADO: u32 = 128;
/// O chão: uma imagem comprida (px).
pub const CHAO: (u32, u32) = (900, 24);
/// A cor do chão (sRGB).
pub const COR_DO_CHAO: [u8; 3] = [30, 34, 42];
/// Onde fica cada chão, abaixo do sítio onde nasce (px do mundo à escala do projecto).
pub const CHAO_EM_PX: [[f32; 2]; 2] = [[0.0, -250.0], [0.0, -400.0]];
/// As cores das três pinceladas: a cabeça, o corpo e as pernas, os braços (sRGB).
pub const CORES: [[u8; 3]; 3] = [[210, 40, 40], [30, 70, 210], [40, 150, 60]];

/// A cabeça: um círculo de raio `0,12` em `(0,5; 0,22)`, em fracções da imagem.
fn cabeca() -> Vec<[f32; 2]> {
    (0..=24)
        .map(|i| {
            #[expect(clippy::cast_precision_loss, reason = "poucos pontos")]
            let a = std::f32::consts::TAU * i as f32 / 24.0;
            [0.5 + 0.12 * a.cos(), 0.22 + 0.12 * a.sin()]
        })
        .collect()
}

/// O corpo e as duas pernas num só traço (a 1.ª perna percorre-se de volta).
const CORPO: [[f32; 2]; 5] = [
    [0.5, 0.34],
    [0.5, 0.64],
    [0.3, 0.92],
    [0.5, 0.64],
    [0.7, 0.92],
];
const BRACOS: [[f32; 2]; 2] = [[0.25, 0.46], [0.75, 0.46]];

/// Um traço por `pontos` (polilinha) em `ms`, um `Move` a cada [`PASSO_MS`], a velocidade constante.
fn traco(r: &mut Vec<(u64, Passo)>, t0: u64, ms: u64, pontos: &[[f32; 2]]) {
    let seg = |a: [f32; 2], b: [f32; 2]| (b[0] - a[0]).hypot(b[1] - a[1]);
    let total: f32 = pontos.windows(2).map(|w| seg(w[0], w[1])).sum();
    let n = ms / PASSO_MS;
    for i in 0..=n {
        #[expect(clippy::cast_precision_loss, reason = "poucos passos")]
        let mut d = total * i as f32 / n as f32;
        let mut uv = pontos[pontos.len() - 1];
        for w in pontos.windows(2) {
            let l = seg(w[0], w[1]);
            if d <= l && l > 0.0 {
                let f = d / l;
                uv = [
                    w[0][0] + f * (w[1][0] - w[0][0]),
                    w[0][1] + f * (w[1][1] - w[0][1]),
                ];
                break;
            }
            d -= l;
        }
        let p = if i == 0 {
            Passo::Desce(uv)
        } else {
            Passo::Move(uv)
        };
        r.push((t0 + i * PASSO_MS, p));
    }
    r.push((t0 + ms + PASSO_MS, Passo::Sobe));
}

/// O chão `k` nasce, desce para [`CHAO_EM_PX`], ganha o corpo e passa a *Static*.
fn poe_o_chao(r: &mut Vec<(u64, Passo)>, t0: u64, k: u8) {
    r.extend([
        (t0, Passo::Chao(k)),
        (
            t0 + 400,
            Passo::Pose {
                qual: Qual::Chao,
                graus: 0.0,
                px: CHAO_EM_PX[usize::from(k)],
            },
        ),
        (t0 + 700, Passo::Corpo(Qual::Chao)),
        (t0 + 1_000, Passo::Estatico),
    ]);
}

/// **O roteiro da `=8`**: cada passo e o instante (ms do relógio da cena), por ordem.
#[must_use]
pub fn roteiro() -> Vec<(u64, Passo)> {
    use Passo as P;
    let mut r = Vec::new();
    poe_o_chao(&mut r, 300, 0);
    r.extend([
        (1_600, P::Enquadra),
        (2_000, P::Diz(1)),
        (3_000, P::AbreAdd),
        (4_000, P::EscolheMacro),
        (6_000, P::Diz(2)),
        (7_000, P::Grava),
        (7_300, P::Diz(3)),
        (7_800, P::NovaImagem(LADO)),
        (8_300, P::Pinta),
        (8_600, P::Diz(4)),
        (8_800, P::Cor(CORES[0])),
        (9_300, P::FechaCor),
    ]);
    traco(&mut r, 9_500, 900, &cabeca());
    r.extend([(10_600, P::Cor(CORES[1])), (11_100, P::FechaCor)]);
    traco(&mut r, 11_300, 1_200, &CORPO);
    r.extend([(12_700, P::Cor(CORES[2])), (13_200, P::FechaCor)]);
    traco(&mut r, 13_400, 700, &BRACOS);
    r.extend([
        (15_500, P::Diz(5)),
        (16_500, P::Corpo(Qual::Escolhido)),
        (18_500, P::Diz(6)),
        (19_500, P::Fisica),
        (20_000, P::Toca),
        (22_500, P::Pausa),
        (23_500, P::Diz(7)),
        (24_500, P::Para),
        // Fora da gravação: sair do modo Paint a gravar grava recusas do Painter (o aplicar recusa).
        (25_000, P::Objeto),
        (26_500, P::Diz(8)),
        (27_500, P::Guarda),
        (30_000, P::Diz(9)),
        (31_000, P::Apaga),
        (33_000, P::Diz(10)),
        (34_000, P::ApagaChao),
        (34_500, P::Rebobina),
    ]);
    poe_o_chao(&mut r, 35_000, 1);
    r.extend([
        (38_500, P::Diz(11)),
        (39_500, P::AplicaNovo),
        (40_000, P::Enquadra),
        (40_500, P::Toca),
        (60_000, P::Diz(0)),
    ]);
    r
}

/// A frase do passo `k` no ecrã (`0` = o fim).
#[must_use]
pub fn frase(k: u8) -> String {
    if k == 0 {
        return ph2d_i18n::tr("app.macros.smoke7.done").to_owned();
    }
    ph2d_i18n::tr(&format!("app.macros.smoke8.step.{k}")).to_owned()
}

/// Os píxeis do chão.
#[must_use]
pub fn chao_px() -> Option<(u32, u32, Vec<u8>)> {
    let (w, h) = CHAO;
    let [r, g, b] = COR_DO_CHAO;
    let n = usize::try_from(w * h).ok()?;
    Some((w, h, [r, g, b, 255].repeat(n)))
}
