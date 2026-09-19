//! ⭐⭐⭐ **A PERGUNTA DE ANTES DE QUALQUER LINHA do suplente #23 (`PathFollow`)**: *a composição de
//! hoje — `Timer` + `Tween` + a cena vectorial — já exprime «este objecto ANDA sobre a curva que o
//! artista desenhou»?*
//!
//! `CLAUDE.md` §5.0: **antes de construir um item de lista aberta, MEÇA se a composição já o
//! exprime.** Nesta linha a pergunta já REESCREVEU três entregas: o **#3 `SensorZone`** estava
//! fechado por composição, a **W5 do #21** descobriu que o pintor já existia, e o **#24 `Health`**
//! deixou de ser um componente e virou *o sinal saber quem*.
//!
//! ⚠️ **Ela mora AQUI, na crate de FAMÍLIA, porque esta é a única que vê os dois lados**: o
//! `ph2d-ecs` (o `Timer`, o `Transform`, o mundo), o `ph2d-tween` (a lei que a W8 deu o ciclo) e o
//! `ph2d-vec-scene` (a curva desenhada, com as cúbicas de verdade). ⛔ O `ph2d-ecs` **não** vê a
//! cena vectorial, e isso não é um acaso — é a doutrina do `VecPathRef`, *«não põe geometria no
//! ECS»*. *Uma sonda que só visse um lado mediria o espantalho.*
//!
//! ⚠️ **Sonda, não gate.** Corre com `--ignored` e IMPRIME; o que ela decide é *o quê* do
//! componente, não se os números batem uma barra.
//!
//! ```text
//! cargo test -p ph2d-app-components --test it \
//!     mede_o_que_a_composicao_ja_da_ao_caminho -- --ignored --nocapture
//! ```
//!
//! # As perguntas, uma por bloco
//!
//! A) **Um TWEEN em X e Y exprime uma curva desenhada?** — é o concorrente mais forte, e ele
//!    acabou de shipar nesta mesma linha (suplente #22, dez waves).
//! B) **Quem consegue LER a geometria de um caminho a partir de um componente?**
//! C) **A lei geométrica está paga?** — *«onde fica o arco `s`, e para onde ele aponta ali»*.
//! D) **A curva sabe dizer para onde VIRA?** — sem isso não há *«a nave aponta para onde voa»*.
//! E) **O documento guarda LOCAL; o artista vê MUNDO.** Quanto custa ignorar isso?
//! F) **A dobra do ciclo (o vai-e-volta) já existe?**

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_tween::{Canal, Ciclo, Relogio, Tween};
use ph2d_vec_scene::Xform;
use ph2d_vec_scene::arc_path::ArcPath;
use ph2d_vec_scene::{VecVertex, VertexKind};

/// O manifesto da fundação — lido em tempo de COMPILAÇÃO, logo não pode envelhecer sem o ficheiro
/// mudar.
const MANIFESTO_ECS: &str = include_str!("../../../ph2d-ecs/Cargo.toml");
/// Idem para esta família, que é onde as pontes das waves #11..#22 moram.
const MANIFESTO_FAMILIA: &str = include_str!("../../Cargo.toml");

/// ⛔ **A régua lê só a secção `[dependencies]` e SEM comentários** — a sonda irmã (a do tween)
/// pagou exactamente este defeito: `manifesto.contains("ph2d-anim")` leu `true` sobre uma linha de
/// **prosa** que descrevia de que é feita outra crate. *Um censo textual que não separa prosa de
/// código mede a prosa.*
fn declara_dependencia(manifesto: &str, crate_: &str) -> bool {
    let mut dentro = false;
    for linha in manifesto.lines() {
        let l = linha.trim();
        if l.starts_with('[') {
            dentro = l == "[dependencies]";
            continue;
        }
        if !dentro || l.starts_with('#') {
            continue;
        }
        let Some((nome, _)) = l.split_once('=') else {
            continue;
        };
        if nome.trim().trim_matches('"').split('.').next() == Some(crate_) {
            return true;
        }
    }
    false
}

/// ⭐ **O ARCO que o artista desenha por cima de um obstáculo** — meia circunferência de raio 2, de
/// `(−2, 0)` a `(2, 0)`, a passar por `(0, 2)`, em duas cúbicas.
///
/// ⚠️ **A fixtura é escolhida para ter a resposta FECHADA**: numa circunferência de raio `r`
/// centrada na origem, todo ponto da corda `y = 0` está a **exactamente** `r` do arco — logo o
/// desvio que o bloco (A) mede não é um número de amostragem, é o raio.
///
/// `4/3·tan(π/8) = 0,552_284_749_830_793_4` é a constante clássica do quarto de círculo por cúbica.
fn arco() -> Vec<VecVertex> {
    const K: f64 = 0.552_284_749_830_793_4 * 2.0;
    vec![
        VecVertex {
            anchor: [-2.0, 0.0],
            in_handle: [-2.0, 0.0],
            out_handle: [-2.0, K],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        },
        VecVertex {
            anchor: [0.0, 2.0],
            in_handle: [-K, 2.0],
            out_handle: [K, 2.0],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        },
        VecVertex {
            anchor: [2.0, 0.0],
            in_handle: [2.0, K],
            out_handle: [2.0, 0.0],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        },
    ]
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
    dx.hypot(dy)
}

#[test]
#[ignore = "sonda: imprime a medição do §5.0, não afirma uma barra"]
fn mede_o_que_a_composicao_ja_da_ao_caminho() {
    let verts = arco();
    let curva = ArcPath::from_contour(&verts, false).expect("o arco tem dois segmentos");
    let total = curva.total();

    eprintln!("\n════ §5.0 — O QUE A COMPOSIÇÃO JÁ DÁ AO CAMINHO (suplente #23) ════\n");
    eprintln!("Fixtura: meia circunferência de raio 2, aberta, duas cúbicas.");
    eprintln!(
        "  comprimento de arco medido = {total:.6}   (π·r = {:.6})",
        std::f64::consts::PI * 2.0
    );
    eprintln!(
        "  distância em linha recta    = {:.6}\n",
        dist([-2.0, 0.0], [2.0, 0.0])
    );

    // ── (A) Um TWEEN em X e Y exprime a curva? ────────────────────────────────────────────────
    //
    // O concorrente a sério: dois tweens de pose no mesmo objecto, no mesmo relógio, um por eixo.
    // É EXACTAMENTE o que o suplente #22 entrega hoje, e é o primeiro sítio onde um artista tenta.
    let tx = Tween::linear(Canal::PositionX, -2.0, 2.0);
    let ty = Tween::linear(Canal::PositionY, 0.0, 0.0);
    let mut fora_da_pista: f64 = 0.0;
    let mut no_mesmo_instante: f64 = 0.0;
    let mut andado = 0.0;
    let mut anterior: Option<[f64; 2]> = None;
    for i in 0..=100 {
        let u = f64::from(i) / 100.0;
        let relogio = Relogio {
            a_correr: true,
            acabou: false,
            #[expect(
                clippy::cast_possible_truncation,
                reason = "u ∈ [0,1] cabe em f32 sem perda"
            )]
            progresso: u as f32,
        };
        let (Some(vx), Some(vy)) = (
            ph2d_tween::valor(&tx, relogio),
            ph2d_tween::valor(&ty, relogio),
        ) else {
            continue;
        };
        let p = [f64::from(vx[0]), f64::from(vy[0])];
        // Onde o tween põe o objecto, contra o ponto da CURVA mais próximo dele.
        let (perto, _) = curva.frame_at(curva.closest_arc(p));
        fora_da_pista = fora_da_pista.max(dist(p, perto));
        // …e contra o ponto da curva no MESMO instante do relógio.
        let (mesmo, _) = curva.frame_at(u * total);
        no_mesmo_instante = no_mesmo_instante.max(dist(p, mesmo));
        if let Some(q) = anterior {
            andado += dist(p, q);
        }
        anterior = Some(p);
    }
    eprintln!("(A) DOIS TWEENS DE POSE (o concorrente que esta linha acabou de shipar)");
    eprintln!(
        "    o objecto sai da pista em ..... {fora_da_pista:.6}  (= o raio, e a conta fecha)"
    );
    eprintln!("    desvio no mesmo instante ...... {no_mesmo_instante:.6}");
    eprintln!("    caminho andado ................ {andado:.6}  contra {total:.6} de curva");
    eprintln!("    ⇒ um tween é uma RECTA entre dois valores: ele não sabe que há curva.\n");

    // ── (B) Quem consegue LER a geometria? ────────────────────────────────────────────────────
    let ecs_ve = declara_dependencia(MANIFESTO_ECS, "ph2d-vec-scene");
    let familia_ve = declara_dependencia(MANIFESTO_FAMILIA, "ph2d-vec-scene");
    let familia_ve_mapa = declara_dependencia(MANIFESTO_FAMILIA, "ph2d-vec-entities");
    // Controlo positivo: as duas declaram o `ph2d-core`. Sem ele, uma régua partida devolveria
    // `false` a tudo e a sonda leria-se como confirmada.
    assert!(
        declara_dependencia(MANIFESTO_ECS, "ph2d-core")
            && declara_dependencia(MANIFESTO_FAMILIA, "ph2d-core"),
        "controlo positivo da régua de manifesto"
    );
    eprintln!("(B) QUEM VÊ A CURVA");
    eprintln!("    ph2d-ecs           → ph2d-vec-scene : {ecs_ve}");
    eprintln!("    ph2d-app-components→ ph2d-vec-scene : {familia_ve}");
    eprintln!("    ph2d-app-components→ ph2d-vec-entities : {familia_ve_mapa}");
    eprintln!("    ⇒ o componente NÃO pode carregar geometria; a PONTE é que lê a curva.\n");

    // ── (C) A lei geométrica está paga? ───────────────────────────────────────────────────────
    //
    // Duas varreduras de 40 passos sobre a MESMA curva: uma em parâmetro uniforme (o que um
    // seguidor escrito à mão faz), outra em ARCO uniforme. A régua é a razão entre o maior e o
    // menor passo de facto andado — `1,0` é velocidade constante.
    let passo_arco = |n: usize| -> (f64, f64) {
        let pts: Vec<[f64; 2]> = (0..=n)
            .map(|i| curva.frame_at(i as f64 / n as f64 * total).0)
            .collect();
        espalhamento(&pts)
    };
    let passo_param = |n: usize| -> (f64, f64) {
        // Parâmetro uniforme sobre os DOIS segmentos, que é o atalho que dispensa o `inv_arclen`.
        let segs = 2usize;
        let pts: Vec<[f64; 2]> = (0..=n)
            .map(|i| {
                let g = i as f64 / n as f64 * segs as f64;
                let s = (g.floor() as usize).min(segs - 1);
                let t = g - s as f64;
                let c = segmento(&verts, s);
                ph2d_vec_scene::arclen::point_at(&c, t)
            })
            .collect();
        espalhamento(&pts)
    };
    let (a_min, a_max) = passo_arco(40);
    let (p_min, p_max) = passo_param(40);
    eprintln!("(C) VELOCIDADE CONSTANTE — 40 passos, razão maior/menor");
    eprintln!(
        "    por PARÂMETRO (o atalho) ...... {:.4}   (passo {p_min:.4} .. {p_max:.4})",
        p_max / p_min
    );
    eprintln!(
        "    por ARCO (`ArcPath::frame_at`)  {:.4}   (passo {a_min:.4} .. {a_max:.4})",
        a_max / a_min
    );
    eprintln!(
        "    ⇒ a lei do arco já está escrita e é EXACTA: esta wave não escreve matemática.\n"
    );

    // ── (D) A curva sabe para onde VIRA? ──────────────────────────────────────────────────────
    let mut pior_unidade: f64 = 0.0;
    let mut pior_angulo: f64 = 0.0;
    for i in 0..=20 {
        let s = f64::from(i) / 20.0 * total;
        let (p, t) = curva.frame_at(s);
        pior_unidade = pior_unidade.max((t[0].hypot(t[1]) - 1.0).abs());
        // Numa circunferência a tangente é perpendicular ao raio — o produto interno é o erro.
        let r = [p[0] / 2.0, p[1] / 2.0];
        pior_angulo = pior_angulo.max(r[0].mul_add(t[0], r[1] * t[1]).abs());
    }
    eprintln!("(D) A TANGENTE");
    eprintln!("    desvio de |t| = 1 ............. {pior_unidade:.3e}");
    eprintln!("    desvio de t ⟂ raio ............ {pior_angulo:.3e}");
    eprintln!("    ⇒ «apontar para onde voa» é um `atan2` sobre o que a porta já devolve.\n");

    // ── (E) LOCAL contra MUNDO ────────────────────────────────────────────────────────────────
    //
    // O caminho é uma entidade com pose, e pode ser FILHO de outra. A geometria que o documento
    // guarda é LOCAL — a lei-mãe do editor vectorial. Quanto custa saltar essa conversão?
    let mut sim = SimWorld::new();
    let pai = sim
        .world_mut()
        .spawn((
            Transform {
                translation: Vec2::new(5.0, -3.0),
                ..Transform::IDENTITY
            },
            Name::new("Palco"),
        ))
        .id();
    let caminho = sim
        .world_mut()
        .spawn((
            Transform {
                rotation: std::f32::consts::FRAC_PI_2,
                scale: Vec2::new(2.0, 2.0),
                ..Transform::IDENTITY
            },
            Name::new("Trilho"),
            ph2d_ecs::ChildOf(pai),
        ))
        .id();
    let mundo = ph2d_vec_entities::transform::world_transform(&sim, caminho);
    let afim: Xform = ph2d_vec_entities::transform::xform_of_transform(mundo);
    let meio_local = curva.frame_at(total * 0.5).0;
    let meio_mundo = afim.apply(meio_local);
    eprintln!("(E) O DOCUMENTO GUARDA LOCAL; O ARTISTA VÊ MUNDO");
    eprintln!(
        "    o topo do arco em LOCAL ....... [{:.4}, {:.4}]",
        meio_local[0], meio_local[1]
    );
    eprintln!(
        "    …e em MUNDO (pai + pose) ...... [{:.4}, {:.4}]",
        meio_mundo[0], meio_mundo[1]
    );
    eprintln!(
        "    distância entre os dois ....... {:.4}",
        dist(meio_local, meio_mundo)
    );
    eprintln!("    ⇒ a ponte compõe a cadeia de pais; a porta já existe e é a dos sprites.\n");

    // ── (F) A dobra do ciclo ──────────────────────────────────────────────────────────────────
    let ida_e_volta: Vec<f64> = [0.0, 0.25, 0.5, 0.75, 1.0]
        .iter()
        .map(|&u| Ciclo::PingPong.dobra(u))
        .collect();
    eprintln!("(F) O VAI-E-VOLTA");
    eprintln!("    `Ciclo::PingPong::dobra` em 0..1: {ida_e_volta:?}");
    eprintln!("    ⇒ a dobra é a da W8 desta mesma linha — segundo consumidor, zero lei nova.\n");

    eprintln!("════ FIM DA MEDIÇÃO ════\n");
}

/// O menor e o maior passo de facto andado entre amostras consecutivas.
fn espalhamento(pts: &[[f64; 2]]) -> (f64, f64) {
    let mut min = f64::INFINITY;
    let mut max: f64 = 0.0;
    for w in pts.windows(2) {
        let d = dist(w[0], w[1]);
        min = min.min(d);
        max = max.max(d);
    }
    (min, max)
}

/// A cúbica do segmento `i` de um contorno ABERTO — a mesma aritmética que o `ArcPath` usa por
/// dentro, escrita aqui porque o `corner_live::segment` é `pub(crate)` da outra crate.
fn segmento(verts: &[VecVertex], i: usize) -> [[f64; 2]; 4] {
    [
        verts[i].anchor,
        verts[i].out_handle,
        verts[i + 1].in_handle,
        verts[i + 1].anchor,
    ]
}
