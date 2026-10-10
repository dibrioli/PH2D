//! ⭐⭐⭐ **A ESCADA da sombra nítida** (smoke do dono, 10/10, foto
//! `docs/Luz2D/handoffs/fotos_W4/cena4_low_medium_high.png`): a borda da sombra da lâmpada PEQUENA da
//! cena 4, junto ao canto da parede (onde a penumbra é mais estreita), guardada numa grade grossa sai
//! em degraus. A `medida_borda_reta` amostra a cada 10 cm e não os vê; esta régua amostra a CADA
//! PÍXEL de tela.
//!
//! O gabarito é a geometria: o ponto de 50 % de luz está EXATAMENTE na reta centro da lâmpada → canto
//! (metade do disco à vista). O desvio do ponto de 50 % da placa a essa reta, píxel a píxel, é a régua.

use bevy_ecs::world::World;
use ph2d_app_light2d::{Degrau, LuzDoQuadro, smoke};
use ph2d_ecs::Name;
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use vello::kurbo::Affine;

use crate::comum::visivel;

const FHD: (u32, u32) = (1920, 1080);

fn afim() -> Affine {
    Camera2d::default().world_to_screen_affine(WindowSize::new(FHD.0, FHD.1))
}

/// A escada de um contorno: o desvio dele ao da geometria, em píxeis de tela.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Escada {
    /// O desvio médio (um deslocamento da borda inteira, não degrau).
    pub media_px: f64,
    /// O rms depois de tirar a reta de mínimos quadrados (só os degraus).
    pub degrau_px: f64,
    pub pontos: usize,
}

fn tira(w: &mut World, nome: &str) {
    let e = w
        .query::<(bevy_ecs::entity::Entity, &Name)>()
        .iter(w)
        .find(|(_, n)| n.0 == nome)
        .map(|(e, _)| e)
        .expect("existe");
    w.despawn(e);
}

/// A cena 4 sem a ambiente (só as lâmpadas contam) — com e sem a parede da sala da PEQUENA.
pub(crate) fn cena4_com_e_sem() -> (World, World) {
    let (mut com, mut sem) = (World::new(), World::new());
    assert!(smoke::encenar(&mut com, 4) && smoke::encenar(&mut sem, 4));
    tira(&mut com, "Luz ambiente");
    tira(&mut sem, "Luz ambiente");
    tira(&mut sem, "Parede da pequena");
    (com, sem)
}

/// A luz recebida em `p` pelo caminho do PRODUTO (`LuzDoQuadro` com `degrau`), lida da placa.
pub(crate) fn luz_do_quadro(gpu: &GpuContext, degrau: Degrau, w: &World) -> crate::comum::Fluencia {
    let r = crate::comum::renderer(gpu);
    let mut l = LuzDoQuadro::com_degrau(degrau);
    assert!(l.preparar(w, afim(), FHD));
    l.gravar(gpu, &mut ph2d_ecs::PresentWorld::new(), &r);
    crate::comum::fluencia_lida(gpu, l.solver().expect("solver"))
}

/// Os contornos da penumbra que a régua segue. A escada que o dono viu vive nos de 10 % e 90 % (o de
/// 50 % de um degrau interpolado quase não ondula); as MANCHAS da interpolação junto ao preto vivem
/// abaixo de 10 % (a foto da W4b viu-as e a régua de três contornos não): por isso os de 2 % e 98 %.
pub(crate) const NIVEIS: [f64; 5] = [0.02, 0.1, 0.5, 0.9, 0.98];

/// Uma coluna por contorno de [`NIVEIS`], para as sondas.
pub(crate) fn por_contorno(e: &[Escada], f: impl Fn(&Escada) -> f64) -> String {
    e.iter()
        .map(|x| format!("{:.3}", f(x)))
        .collect::<Vec<_>>()
        .join(" · ")
}

fn cruza(ts: &[f64], f: &[f64], nivel: f64) -> Option<f64> {
    f.windows(2)
        .position(|w| (w[0] - nivel) * (w[1] - nivel) <= 0.0 && w[0] != w[1])
        .map(|i| ts[i] + (nivel - f[i]) / (f[i + 1] - f[i]) * (ts[i + 1] - ts[i]))
}

fn resumo(desvios: &[(f64, f64)]) -> Escada {
    let n = desvios.len().max(1) as f64;
    let (mx, my) = desvios
        .iter()
        .fold((0.0, 0.0), |a, p| (a.0 + p.0 / n, a.1 + p.1 / n));
    let (sxy, sxx) = desvios.iter().fold((0.0, 0.0), |a, p| {
        (a.0 + (p.0 - mx) * (p.1 - my), a.1 + (p.0 - mx).powi(2))
    });
    let inclinacao = sxy / sxx.max(1e-12);
    let resto = |p: &(f64, f64)| p.1 - (my + inclinacao * (p.0 - mx));
    Escada {
        media_px: my,
        degrau_px: (desvios.iter().map(|p| resto(p).powi(2)).sum::<f64>() / n).sqrt(),
        pontos: desvios.len(),
    }
}

/// ⭐ **A régua**: de 4 px depois do canto até a linha de medida da penumbra, a cada píxel de tela,
/// o perfil perpendicular à borda (passo de 1/20 de píxel); em cada contorno de [`NIVEIS`], o
/// desvio do cruzamento da placa (fração com/sem parede) ao da geometria (`comum::visivel`).
pub(crate) fn escada_de(fc: &crate::comum::Fluencia, fs: &crate::comum::Fluencia) -> [Escada; 5] {
    let [a, b, ..] = afim().as_coeffs();
    let ppm = a.hypot(b);
    let (lampada, canto, linha) = smoke::geometria_da_sala(smoke::SALAS_X[0]);
    let (l, c) = (lampada.map(f64::from), canto.map(f64::from));
    let raio = f64::from(smoke::LAMPADA_PEQUENA_M);
    let d = [c[0] - l[0], c[1] - l[1]];
    let dist = d[0].hypot(d[1]);
    let u = [d[0] / dist, d[1] / dist];
    let nrm = [-u[1], u[0]];
    // Até o perfil tocar 2 px antes da face do muro do meio (x = −0,1 m): a foto da W4b viu um salto
    // junto a ele, depois da linha de medida. Dentro do muro a fração é 0 ÷ 0 (cruzamento falso).
    let _ = linha;
    let muro = -0.1 - 2.0 / ppm;
    let s_fim = (muro - c[0]) / u[0];
    let mut desvios: [Vec<(f64, f64)>; 5] = Default::default();
    let mut k = 4.0;
    while k / ppm <= s_fim {
        let s = k / ppm;
        let p0 = [c[0] + s * u[0], c[1] + s * u[1]];
        // Meia penumbra (o cone do disco a `s` do canto) e 6 px de folga de cada lado.
        let h = s * raio / dist + 6.0 / ppm;
        if p0[0] + h * nrm[0].abs() >= muro {
            break;
        }
        let n = (2.0 * h * ppm * 20.0).ceil() as usize;
        let ts: Vec<f64> = (0..=n)
            .map(|i| -h + 2.0 * h * i as f64 / n as f64)
            .collect();
        let ponto = |t: f64| [p0[0] + t * nrm[0], p0[1] + t * nrm[1]];
        let f: Vec<f64> = ts
            .iter()
            .map(|&t| fc.amostra(ponto(t)) / fs.amostra(ponto(t)).max(1e-9))
            .collect();
        let g: Vec<f64> = ts.iter().map(|&t| visivel(ponto(t), l, raio, c)).collect();
        for (j, nivel) in NIVEIS.into_iter().enumerate() {
            if let (Some(tp), Some(tg)) = (cruza(&ts, &f, nivel), cruza(&ts, &g, nivel)) {
                desvios[j].push((k, (tp - tg) * ppm));
            }
        }
        k += 1.0;
    }
    desvios.each_ref().map(|d| resumo(d))
}

/// A régua sobre o caminho do PRODUTO com `degrau`.
pub(crate) fn escada(gpu: &GpuContext, degrau: Degrau) -> [Escada; 5] {
    let (com, sem) = cena4_com_e_sem();
    escada_de(
        &luz_do_quadro(gpu, degrau, &com),
        &luz_do_quadro(gpu, degrau, &sem),
    )
}

/// O degrau de cada nível do Settings (Low · Medium · High).
pub(crate) fn degraus() -> [(&'static str, Degrau); 3] {
    use ph2d_editor_core::luz_edits::NivelDaLuz;
    [
        ("Low", NivelDaLuz::Baixa),
        ("Medium", NivelDaLuz::Media),
        ("High", NivelDaLuz::Alta),
    ]
    .map(|(nome, n)| (nome, ph2d_app_light2d::qualidade::degrau_do(n)))
}

/// O teto da escada — o MEDIDO + 10 % (W4b, `medida_luz2d_w4b` B, do canto até o muro), degrau dos
/// contornos 2 · 10 · 50 · 90 · 98 % com a grade fina de 1 px: Low 0,163 · 0,105 · 0,085 · 0,139 ·
/// 0,182 px, Medium 0,176 · 0,123 · 0,079 · 0,122 · 0,170, High 0,179 · 0,113 · 0,061 · 0,121 · 0,179;
/// sem a grade fina Low 0,666 · 0,739 · 0,474 · 0,664 · 1,121 e Medium 0,435 · 0,341 · 0,204 · 0,309 ·
/// 0,420; sem o joelho, o de 2 % no Low 0,352 (as manchas da foto); com a visibilidade das sondas no
/// passo grosso, Medium 0,258 · 0,307 · 0,221.
const TETO_DO_DEGRAU_PX: f64 = 0.21;
/// ⭐ F1 (W5a) — o deslocamento da borda inteira, em PÍXEIS DE TELA: o MEDIDO + 10 % (rodada
/// `medida_luz2d_w5a`, a captura fina dos opacos de ½ px): o pior é o contorno de 2 % no High, −0,949
/// px (Low −0,938 · −0,821 · −0,785 · −0,657 · −0,514). Na W4b a captura das cascatas arredondava o
/// canto do oclusor: Low −4,2 · Medium −2,1 · High −1,2 px no contorno de 50 % (teto 0,95 texel GROSSO).
const TETO_DA_MEDIA_PX: f64 = 1.05;

/// ⭐⭐⭐ **GATE — a sombra nítida não tem ESCADA** em nenhum nível (os três contornos da penumbra,
/// a cada píxel de tela), e a borda não sai do lugar mais do que a captura explica.
#[test]
fn a_sombra_nitida_nao_tem_escada() {
    let Some(gpu) = crate::comum::gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let [a, b, ..] = afim().as_coeffs();
    let ppm = a.hypot(b);
    for (nome, degrau) in degraus() {
        let g = ph2d_app_light2d::grade_do_quadro(afim(), FHD, degrau).expect("grade");
        let texel_px = g.texel_m * ppm;
        for (nivel, e) in NIVEIS.into_iter().zip(escada(&gpu, degrau)) {
            eprintln!(
                "escada · {nome} · {:>2.0} %: degrau {:.3} px · média {:.3} px ({:.2} texel) · {} pontos",
                nivel * 100.0,
                e.degrau_px,
                e.media_px,
                e.media_px / texel_px,
                e.pontos
            );
            assert!(
                e.pontos >= 150,
                "a régua achou só {} pontos da borda",
                e.pontos
            );
            assert!(
                e.degrau_px <= TETO_DO_DEGRAU_PX,
                "{nome}, contorno de {nivel}: a borda da sombra sai em ESCADA — {:.3} px (teto {TETO_DO_DEGRAU_PX})",
                e.degrau_px
            );
            assert!(
                e.media_px.abs() <= TETO_DA_MEDIA_PX,
                "{nome}, contorno de {nivel}: a borda saiu do lugar {:.2} px (teto {TETO_DA_MEDIA_PX} px)",
                e.media_px
            );
        }
    }
}

/// Sonda da CAUSA do deslocamento da borda (a média): a marcha com passo de ¼ de texel perto do
/// conteúdo. Se a média cai, é a amostragem da marcha; se fica, é a captura (o canto arredondado
/// pela cobertura interpolada). À mão: `… --ignored --nocapture medida_causa_do_desvio`.
#[test]
#[ignore = "sonda: corre-se à mão"]
fn medida_causa_do_desvio() {
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    let (com, sem) = cena4_com_e_sem();
    let (cc, cs) = (
        ph2d_light2d_ecs::coleta::cena_de_luz(&com),
        ph2d_light2d_ecs::coleta::cena_de_luz(&sem),
    );
    for (nome, degrau) in degraus().into_iter().take(2) {
        let g = ph2d_app_light2d::grade_do_quadro(afim(), FHD, degrau).expect("grade");
        let fina = ph2d_app_light2d::fina_do_quadro(afim(), degrau);
        // (passo perto, refino, captura por texel fino)
        for (passo, refino, por) in [
            (1.0_f32, ph2d_light2d_gpu::REFINO_PADRAO, 1),
            (0.5, ph2d_light2d_gpu::REFINO_PADRAO, 1),
            (1.0, ph2d_light2d_gpu::Refino::Tudo, 1),
            (0.5, ph2d_light2d_gpu::Refino::Tudo, 1),
            (1.0, ph2d_light2d_gpu::REFINO_PADRAO, 2),
            (2.0, ph2d_light2d_gpu::REFINO_PADRAO, 2),
            (2.0, ph2d_light2d_gpu::Refino::Tudo, 2),
        ] {
            let cfg = ph2d_light2d_gpu::Config {
                passo_perto: passo,
                fina,
                captura: fina * por,
                refino,
                ..ph2d_light2d_gpu::Config::padrao_para(&gpu.device, g)
            };
            let flu = |cena: &ph2d_light2d_ecs::CenaDeLuz| {
                let mut s = ph2d_light2d_gpu::Solver::novo(&gpu, cfg).expect("solver");
                s.quadro(&gpu, cena);
                crate::comum::fluencia_lida(&gpu, &s)
            };
            let e = escada_de(&flu(&cc), &flu(&cs));
            eprintln!(
                "causa · {nome} · captura ×{} · passo {passo} · {}: média {} · degrau {} px (2 · 10 · 50 · 90 · 98 %)",
                cfg.captura,
                if matches!(refino, ph2d_light2d_gpu::Refino::Tudo) { "tudo" } else { "bordas" },
                por_contorno(&e, |x| x.media_px),
                por_contorno(&e, |x| x.degrau_px)
            );
        }
    }
}
