//! ⭐⭐⭐ **A LEI DA COR — porque a matiz de um jade muda com a profundidade, e a nossa não.**
//!
//! Ordem do dono, 2026-09-18, depois do veredito a quatro colunas da
//! [`§16`](../../../docs/Render3d/10_a_luz_que_atravessa_a_peca.md): *«atacamos agora a cor»*.
//!
//! # ⛔⛔ O mecanismo da nossa surdez, e ele está numa DIVISÃO
//!
//! O [`ph2d_material::subsurface::integrate_burley`] devolve `Σ(R·w) / Σ R` — um quociente em que
//! o perfil `R` aparece **em cima e em baixo**. Tudo o que o perfil sabe sobre a profundidade (a
//! forma `1/mfp`, as duas exponenciais, a escala) **cancela-se na divisão**, e o que sobra é uma
//! média direccional pura. A cor sai depois por `sss = subsurface_color × isso`.
//!
//! ⇒ *a informação da profundidade não se perde por aproximação: ela é DIVIDIDA FORA por
//! construção.* Com o `mfp` partilhado pelos três canais o quociente é o mesmo nos três, e a matiz
//! que sai **é** a que o artista escreveu no painel — a qualquer profundidade.
//!
//! # ⭐⭐⭐ E o mecanismo da VERDADE é um expoente, não um multiplicador
//!
//! Num passeio aleatório, cada evento de espalhamento multiplica a luz pelo albedo do canal. Um
//! `mfp` curto ⇒ **muitos** eventos antes de sair ⇒ a cor é multiplicada muitas vezes ⇒ **saturada**.
//! Um `mfp` da ordem da peça ⇒ **poucos** eventos ⇒ a cor mal é multiplicada ⇒ **lava para o
//! branco**. ⇒ a matiz que volta é `albedo^p`, com `p` a **cair** com a profundidade.
//!
//! Esta sonda mede esse `p` nas DUAS verdades, e mede o nosso pela porta do material.
//! ⛔ **Ela não propõe lei nenhuma** — a lei escreve-se depois de as duas verdades concordarem
//! sobre a curva, e não antes.
//!
//! # ⚠️ A cerca que as réguas de hoje NÃO têm: a ORIENTAÇÃO
//!
//! As linhas de um PFM vêm de **baixo para cima**. A [`super::razao_rb`] e a
//! [`super::casa_a_populacao`] somam sobre um limiar, logo são **invariantes a um espelho
//! vertical** — a janela E notou isso por conta própria e passou a reportar a posição do píxel mais
//! brilhante. Uma **máscara geométrica fixa**, que é o que esta sonda usa, *não* é invariante: um
//! ficheiro ao contrário mediria o fundo. ⇒ a cerca é medida, não assumida.

use super::{Quadro, W, H, le_pfm, quadro};
use ph2d_field_render::Orbit;

/// O raio da esfera da cena `=33`, em unidades do mundo — ⛔ leia-o da cena, não daqui, se ela mudar.
const RAIO_DA_PECA: f32 = 0.42;

/// A cor autorada das duas fixturas (base e subsuperfície).
const COR: [f32; 3] = [0.75, 0.35, 0.35];

/// As profundidades onde **as duas** verdades existem, na família de raios IGUAIS.
///
/// ⭐ É a família de raios iguais porque ela **isola a pergunta**: com os três canais a viajar a
/// mesma distância, toda mudança de matiz é da PROFUNDIDADE e nenhuma é de os canais viajarem
/// distâncias diferentes.
const PROFUNDIDADES: [(&str, f32); 4] = [
    ("g003", 0.03),
    ("g010", 0.10),
    ("g030", 0.30),
    ("g100", 1.00),
];

/// `R/B` **linear** sobre uma máscara fixa.
///
/// ⭐ Em linear o quociente é **invariante à exposição** (os dois canais escalam juntos), logo aqui
/// não há exposição a casar — e é por isso que este é o espaço certo para a pergunta da matiz. Ver
/// a [`§16.3`](../../../docs/Render3d/10_a_luz_que_atravessa_a_peca.md), onde a régua de bytes
/// chega a **inverter a ordem** de duas colunas.
fn rb_linear(px: &[[f32; 3]], mascara: &[bool]) -> f32 {
    let (mut r, mut b) = (0.0f64, 0.0f64);
    for (i, &m) in mascara.iter().enumerate() {
        if m {
            r += f64::from(px[i][0]);
            b += f64::from(px[i][2]);
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    ((r / b.max(1e-12)) as f32)
}

/// Onde está o píxel mais brilhante — a **cerca de orientação**.
fn mais_brilhante(px: &[[f32; 3]]) -> (usize, usize) {
    let w = W as usize;
    let i = px
        .iter()
        .enumerate()
        .max_by(|a, b| {
            let s = |c: &[f32; 3]| c[0] + c[1] + c[2];
            s(a.1).total_cmp(&s(b.1))
        })
        .map_or(0, |p| p.0);
    (i % w, i / w)
}

/// O expoente da matiz: `R/B = (0,75/0,35)^p`.
///
/// ⭐ `p = 1` quer dizer *«a cor que sai é a cor autorada»*; `p = 0` quer dizer *«branco»*.
fn expoente(rb: f32) -> f32 {
    rb.max(1e-6).ln() / (COR[0] / COR[2]).ln()
}

/// ⏱️⭐⭐⭐ **A CURVA DA MATIZ CONTRA A PROFUNDIDADE, nas duas verdades e na nossa lei.**
///
/// ```text
/// PH2D_UNREAL=/var/tmp/ph2d-oraculo-unreal/out \
/// PH2D_VERDADE2=/var/tmp/ph2d-verdade-cycles \
///   cargo test -p ph2d-app-field3d --lib sonda_a_lei_da_cor -- --ignored --nocapture
/// ```
///
/// ⛔ **Ela mede e não cura.** Uma lei escrita antes de as duas verdades concordarem sobre a curva
/// seria um ajuste a três pontos com cara de mecanismo — que é exactamente o que este repo chama de
/// *«uma recusa medida responde UMA pergunta»* ao contrário.
#[test]
#[ignore = "sonda: precisa dos dois oráculos em $PH2D_UNREAL e $PH2D_VERDADE2"]
fn sonda_a_lei_da_cor_da_profundidade() {
    let (Ok(dir_u), Ok(dir_v)) = (
        std::env::var("PH2D_UNREAL"),
        std::env::var("PH2D_VERDADE2"),
    ) else {
        println!("sem $PH2D_UNREAL e/ou $PH2D_VERDADE2 — saltado");
        return;
    };

    // ── A MÁSCARA: a silhueta iluminada da bola OPACA, a mesma para todas as células ────────────
    // ⚠️ Fixa de propósito. A janela E mediu que uma máscara derivada da própria imagem (um limiar
    // sobre um percentil) **anda com o ruído**: o traçado dela lia `1,2170 → 1,3122 → 1,3516` ao
    // subir amostras, e o que se movia era a régua. Com máscara fixa, `0,117 %`.
    let mut cam = Orbit::default();
    cam.half_extent *= 0.42;
    cam.target = [0.55, 0.0, 0.0];
    let doc = crate::smoke::scenes::edge::cena_33().expect("a cena");
    let (onde, luz) = crate::lights::opening_light(&cam);
    let (_, _, opaco) = quadro(&Quadro {
        doc: &doc,
        m: ph2d_material::OpenPbr {
            subsurface_weight: 0.0,
            base_color: COR,
            specular_weight: 0.0,
            ..ph2d_material::OpenPbr::default()
        },
        cam: &cam,
        onde,
        luz,
        com_sombra: true,
        chao: None,
        sem_ceu: true,
    });
    let n_px = (W as usize) * (H as usize);
    let mascara: Vec<bool> = (0..n_px)
        .map(|i| {
            let q = i * 4;
            u32::from(opaco[q]) + u32::from(opaco[q + 1]) + u32::from(opaco[q + 2]) > 30
        })
        .collect();
    let vivos = mascara.iter().filter(|m| **m).count();
    println!("\n  máscara fixa (silhueta iluminada do opaco): {vivos} px de {n_px}");

    // ── A CERCA DE ORIENTAÇÃO ───────────────────────────────────────────────────────────────────
    let nosso_pico = {
        let w = W as usize;
        let i = (0..n_px)
            .max_by_key(|&i| {
                u32::from(opaco[i * 4]) + u32::from(opaco[i * 4 + 1]) + u32::from(opaco[i * 4 + 2])
            })
            .unwrap_or(0);
        (i % w, i / w)
    };
    println!("  pico do NOSSO opaco: {nosso_pico:?}");

    // ── A CURVA ─────────────────────────────────────────────────────────────────────────────────
    println!(
        "\n  ── A MATIZ CONTRA A PROFUNDIDADE (raios IGUAIS, LINEAR, máscara fixa) ──\n    \
         mfp   · mfp/raio ·   CYCLES R/B ·   p ·  UNREAL-PT R/B ·   p"
    );
    let mut anterior: Option<f32> = None;
    for (tag, mfp) in PROFUNDIDADES {
        let ler = |caminho: String| -> Option<f32> {
            let (_, _, px) = le_pfm(&caminho)?;
            let pico = mais_brilhante(&px);
            // ⛔ Um ficheiro ao contrário põe o pico a ~`H − y` daqui, e a máscara mediria o fundo.
            let dy = pico.1.abs_diff(nosso_pico.1);
            assert!(
                dy < 40,
                "ORIENTAÇÃO: o pico de {caminho} está em {pico:?} e o nosso em {nosso_pico:?} \
                 (Δy = {dy}) — o ficheiro pode estar espelhado na vertical"
            );
            Some(rb_linear(&px, &mascara))
        };
        let cy = ler(format!("{dir_v}/ref_jade_{tag}_e5.pfm"));
        let un = ler(format!("{dir_u}/jade_{tag}_pathtracer.pfm"));
        let sat = un.is_some_and(|u| {
            anterior.is_some_and(|a| (u - a).abs() <= super::unreal_contendor::INDISTINGUIVEL * a)
        });
        anterior = un;
        let f = |o: Option<f32>| {
            o.map_or_else(
                || "         —      —".to_string(),
                |v| format!("{v:>9.4} · {:>5.3}", expoente(v)),
            )
        };
        println!(
            "    {mfp:>5.2} ·  {:>6.3}  · {} · {}{}",
            mfp / RAIO_DA_PECA,
            f(cy),
            f(un),
            if sat { "  ⛔ SATURADO" } else { "" }
        );
    }

    // ── A NOSSA LEI, medida pela PORTA DO MATERIAL e não pela imagem ─────────────────────────────
    //
    // ⭐⭐ Aqui a resposta é EXACTA e não precisa de um quadro linear: `direct` devolve radiância
    // linear, e com luz branca o quociente é a matiz da lei. *A surdez deixa de ser um argumento e
    // passa a ser uma corrida.*
    println!("\n  ── A NOSSA LEI, pela porta do material (luz branca, radiância linear) ──");
    println!("    mfp   · N·L=0,9 · N·L=0,4 · N·L=0,0 · N·L=−0,3");
    let mut todos: Vec<f32> = Vec::new();
    for (_, mfp) in PROFUNDIDADES {
        let s = ph2d_material::OpenPbr {
            subsurface_weight: 1.0,
            geometry_thin_walled: false,
            subsurface_color: COR,
            base_color: COR,
            specular_weight: 0.0,
            subsurface_radius: mfp,
            subsurface_radius_scale: [1.0, 1.0, 1.0],
            ..ph2d_material::OpenPbr::default()
        }
        .prepare()
        .at_curvature(1.0 / RAIO_DA_PECA);
        let n = [0.0, 0.0, 1.0];
        let v = [0.0, 0.0, 1.0];
        let mut linha = format!("    {mfp:>5.2} ·");
        for ndl in [0.9f32, 0.4, 0.0, -0.3] {
            let l = [(1.0 - ndl * ndl).max(0.0).sqrt(), 0.0, ndl];
            let c = s.direct(n, v, l, [1.0, 1.0, 1.0]);
            let rb = c[0] / c[2].max(1e-12);
            todos.push(rb);
            linha += &format!(" {rb:>8.5} ·");
        }
        println!("{linha}");
    }
    let (lo, hi) = todos
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), &x| (a.min(x), b.max(x)));
    println!(
        "\n    ⇒ a nossa matiz varre [{lo:.5} .. {hi:.5}] · balanço {:.4}× · p = {:.4}\n       (a cor \
         autorada é {:.5}; o OPACO lê o mesmo)",
        hi / lo.max(1e-12),
        expoente(lo),
        COR[0] / COR[2]
    );
}
