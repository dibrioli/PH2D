//! **AS SONDAS do contacto no passo** — as que IMPRIMEM, pela porta do produto (o [`crate::step`]).
//!
//! Irmã do [`super::contact_tests`] pelo tecto de LOC (HR-18) e por RESPONSABILIDADE: ali os gates
//! AFIRMAM, aqui as sondas MEDEM e imprimem. ⚠️ As duas metades partilham o arnês (`bola_sobre_
//! obstaculo`, `col`, `DT`), que é do irmão — *uma segunda cópia dele seria a segunda resposta à
//! pergunta «que cena estou a medir?»*.

use super::contact_tests::{DT, col};
use crate::step;
use ph2d_nodegraph::attr::{COLLIDER_BOX_COLUMN, Column, Stream};

/// SONDA — AUDITORIA DO ATRITO. Uma caixa desliza sobre um obstaculo fixo, com gravidade.
/// A lei diz que ela trava a `μ·g`. Quanto e' que ela trava de facto?
#[test]
#[ignore = "sonda de medicao"]
fn probe_auditoria_do_atrito() {
    use ph2d_nodegraph::attr::FRICTION_COLUMN;
    const G: f32 = 4.0;
    const V0: f32 = 1.0;
    // `sub` = em quantos pedacos o tique e' partido (o que os `substeps` da zona fazem).
    let desliza = |mu: f32, sub: u32, tiques: u32| -> (f32, f32) {
        // 0 = o chao (obstaculo, meia altura 0,5, topo em y = 0); 1 = a caixa em cima.
        let mut p = vec![[0.0_f32, -0.5], [0.0, 0.11]];
        let mut v = vec![[0.0_f32, 0.0], [V0, 0.0]];
        #[expect(clippy::cast_precision_loss, reason = "uma contagem de sub-passos")]
        let dt = DT / sub as f32;
        for _ in 0..(tiques * sub) {
            let s = Stream::new(2)
                .with("P", Column::Vec2(p.clone()))
                .with("vel", Column::Vec2(v.clone()))
                .with("accel", Column::Vec2(vec![[0.0, -G], [0.0, -G]]))
                .with("sim_t", Column::Scalar(vec![0.0, 0.0]))
                .with("inv_mass", Column::Scalar(vec![0.0, 1.0]))
                .with(FRICTION_COLUMN, Column::Scalar(vec![mu, mu]))
                .with(
                    COLLIDER_BOX_COLUMN,
                    Column::Vec2(vec![[4.0, 0.5], [0.11, 0.11]]),
                );
            let out = step(&s, dt, 1.0, 0.0, 0.0, 1.0);
            p = col(&out, "P");
            v = col(&out, "vel");
        }
        (v[1][0], p[1][0])
    };
    eprintln!("\n  UMA CAIXA A DESLIZAR — v0 = {V0} u/s, g = {G}, 30 tiques (0,5 s)");
    eprintln!("  A lei de Coulomb: v(t) = v0 − μ·g·t  ⇒  com μ=1 ela PARAVA em 0,25 s");
    eprintln!("   μ   | sub |  v final | travou | v teorica | percorreu");
    eprintln!("  -----|-----|----------|--------|-----------|----------");
    for mu in [0.0_f32, 0.5, 1.0] {
        for sub in [1_u32, 8] {
            let (vf, x) = desliza(mu, sub, 30);
            let teor = (V0 - mu * G * 0.5).max(0.0);
            eprintln!(
                "  {mu:>4.1} | {sub:>3} | {vf:>8.4} | {:>6.4} | {teor:>9.4} | {x:>8.4}",
                V0 - vf
            );
        }
    }
}

/// SONDA — AUDITORIA DO ATRITO, parte 2: uma PILHA. A de baixo sente o peso das de cima?
#[test]
#[ignore = "sonda de medicao"]
fn probe_auditoria_do_atrito_na_pilha() {
    use ph2d_nodegraph::attr::FRICTION_COLUMN;
    const G: f32 = 4.0;
    const L: f32 = 0.11;
    // Uma torre de `alt` caixas sobre um chao fixo; a de BAIXO leva um empurrao horizontal.
    let torre = |mu: f32, alt: usize, tiques: u32| -> (f32, f32) {
        let n = alt + 1;
        let mut p = vec![[0.0_f32, -0.5]];
        let mut v = vec![[0.0_f32, 0.0]];
        for k in 0..alt {
            #[expect(clippy::cast_precision_loss, reason = "um indice de andar")]
            let y = L + 2.0 * L * k as f32;
            p.push([0.0, y]);
            v.push(if k == 0 { [1.0, 0.0] } else { [0.0, 0.0] });
        }
        let mut pesos = vec![0.0_f32];
        pesos.extend(std::iter::repeat_n(1.0_f32, alt));
        let mut caixas = vec![[4.0_f32, 0.5]];
        caixas.extend(std::iter::repeat_n([L, L], alt));
        for _ in 0..(tiques * 8) {
            let s = Stream::new(n)
                .with("P", Column::Vec2(p.clone()))
                .with("vel", Column::Vec2(v.clone()))
                .with("accel", Column::Vec2(vec![[0.0, -G]; n]))
                .with("sim_t", Column::Scalar(vec![0.0; n]))
                .with("inv_mass", Column::Scalar(pesos.clone()))
                .with(FRICTION_COLUMN, Column::Scalar(vec![mu; n]))
                .with(COLLIDER_BOX_COLUMN, Column::Vec2(caixas.clone()));
            let out = step(&s, DT / 8.0, 1.0, 0.0, 0.0, 1.0);
            p = col(&out, "P");
            v = col(&out, "vel");
        }
        (v[1][0], p[1][0])
    };
    eprintln!("\n  A CAIXA DE BAIXO DE UMA TORRE leva um empurrao de 1,0 u/s. Quanto percorre?");
    eprintln!("  (mais peso em cima ⇒ mais atrito ⇒ MENOS percurso, se o peso propagar)");
    eprintln!("   μ   | andares |  v final | percorreu");
    eprintln!("  -----|---------|----------|----------");
    for mu in [0.0_f32, 1.0] {
        for alt in [1_usize, 2, 4] {
            let (vf, x) = torre(mu, alt, 30);
            eprintln!("  {mu:>4.1} | {alt:>7} | {vf:>8.4} | {x:>8.4}");
        }
    }
}

/// SONDA — AUDITORIA parte 3: um DISCO que desliza chega a ROLAR sem derrapar?
/// A condição de rolamento puro é `v = ω·R`. Medida pela porta do produto.
#[test]
#[ignore = "sonda de medicao"]
fn probe_auditoria_do_rolamento() {
    use crate::SPIN;
    use ph2d_nodegraph::attr::{COLLIDER_COLUMN, FRICTION_COLUMN};
    const G: f32 = 4.0;
    const R: f32 = 0.2;
    let rola = |mu: f32, tiques: u32| -> (f32, f32) {
        let mut p = vec![[0.0_f32, -0.5], [0.0, R]];
        let mut v = vec![[0.0_f32, 0.0], [1.0, 0.0]];
        let mut spin = vec![0.0_f32, 0.0];
        let mut rot = vec![0.0_f32, 0.0];
        let mut rot_antes = 0.0_f32;
        for _ in 0..(tiques * 8) {
            rot_antes = rot[1];
            let s = Stream::new(2)
                .with("P", Column::Vec2(p.clone()))
                .with("vel", Column::Vec2(v.clone()))
                .with(SPIN, Column::Scalar(spin.clone()))
                .with("accel", Column::Vec2(vec![[0.0, -G], [0.0, -G]]))
                .with("sim_t", Column::Scalar(vec![0.0, 0.0]))
                .with("inv_mass", Column::Scalar(vec![0.0, 1.0]))
                .with(FRICTION_COLUMN, Column::Scalar(vec![mu, mu]))
                .with(COLLIDER_COLUMN, Column::Scalar(vec![0.0, R]))
                .with(
                    COLLIDER_BOX_COLUMN,
                    Column::Vec2(vec![[4.0, 0.5], [0.0, 0.0]]),
                );
            let out = step(&s, DT / 8.0, 1.0, 0.0, 0.0, 1.0);
            p = col(&out, "P");
            v = col(&out, "vel");
            if let Some(Column::Scalar(sp)) = out.get(SPIN) {
                spin = sp.clone();
            }
            // ⚠️ O contacto NAO tem velocidade angular: ele escreve a rotacao em `rot`, em GRAUS
            // (doc 109 §6). Ler `spin` mede a rotacao AUTORADA, que aqui e' sempre zero.
            if let Some(Column::Scalar(r)) = out.get("rot") {
                rot = r.clone();
            }
        }
        // A velocidade angular efectiva: quanto `rot` andou no ULTIMO sub-passo, em rad/s.
        let w = (rot[1] - rot_antes).to_radians() / (DT / 8.0);
        (v[1][0], -w * R)
    };
    eprintln!("\n  UM DISCO (R = {R}) largado a 1,0 u/s sobre um chao fixo, g = {G}");
    eprintln!("  Rolamento PURO e' `v = ω·R`. Se ω ficar em 0, a bola DERRAPA para sempre.");
    eprintln!("   μ   |     v    |   ω·R    | rola? (v ≈ ω·R)");
    eprintln!("  -----|----------|----------|----------------");
    for mu in [0.0_f32, 0.25, 0.5, 1.0] {
        let (v, wr) = rola(mu, 30);
        let rola_bem = if (v - wr).abs() < 0.1 * v.abs().max(1e-3) {
            "SIM"
        } else {
            "nao"
        };
        eprintln!("  {mu:>4.2} | {v:>8.4} | {wr:>8.4} | {rola_bem:>15}");
    }
}

/// SONDA — uma caixa atingida FORA DO CENTRO continua a rodar depois de o contacto acabar?
#[test]
#[ignore = "sonda de medicao"]
fn probe_a_caixa_atingida_continua_a_rodar() {
    use ph2d_nodegraph::attr::FRICTION_COLUMN;
    const L: f32 = 0.11;
    // 0 = o projéctil, a vir da esquerda; 1 = o alvo, parado e DESALINHADO em y (o embate e' fora
    // do centro dele, logo tem de o fazer girar).
    let mut p = vec![[-0.30_f32, 0.0], [0.0, 1.5 * L]];
    let mut v = vec![[1.0_f32, 0.0], [0.0, 0.0]];
    let mut rot = vec![0.0_f32, 0.0];
    // ⚠️ O `spin` TEM de voltar ao tique seguinte — sem ele a sonda mede um programa em que a
    // velocidade angular é deitada fora a cada quadro, que é exactamente o defeito a testar.
    let mut spin = vec![0.0_f32, 0.0];
    eprintln!("\n  tique |  rot do alvo | Δrot   |  spin do alvo | dist | em contacto?");
    eprintln!("  ------|--------------|--------|--------------------|-------------");
    for k in 0..40 {
        let antes = rot[1];
        let s = Stream::new(2)
            .with("P", Column::Vec2(p.clone()))
            .with("vel", Column::Vec2(v.clone()))
            .with("rot", Column::Scalar(rot.clone()))
            .with(crate::SPIN, Column::Scalar(spin.clone()))
            .with("sim_t", Column::Scalar(vec![0.0, 0.0]))
            .with(FRICTION_COLUMN, Column::Scalar(vec![1.0, 1.0]))
            .with(COLLIDER_BOX_COLUMN, Column::Vec2(vec![[L, L], [L, L]]));
        let out = step(&s, DT, 1.0, 0.0, 0.0, 1.0);
        p = col(&out, "P");
        v = col(&out, "vel");
        if let Some(Column::Scalar(r)) = out.get("rot") {
            rot = r.clone();
        }
        if let Some(Column::Scalar(sp)) = out.get(crate::SPIN) {
            spin = sp.clone();
        }
        let d = (p[1][0] - p[0][0]).hypot(p[1][1] - p[0][1]);
        let toca = if d < 2.0 * L * 1.45 { "SIM" } else { "-" };
        if k % 4 == 0 || (rot[1] - antes).abs() > 1e-4 {
            eprintln!(
                "  {k:>5} | {:>11.4}° | {:>6.4} | {:>12.4} | {d:>4.2} | {toca:>11}",
                rot[1],
                rot[1] - antes,
                spin[1]
            );
        }
    }
    eprintln!(
        "\n  ⚠️ se o Δrot voltar a ZERO assim que elas se separam, a peca NAO TEM velocidade"
    );
    eprintln!("     angular: ela roda enquanto toca e para no ar (doc 109 §6).");
}

/// SONDA — uma caixa a GIRAR pousada num chao fixo: o atrito trava-lhe o giro?
#[test]
#[ignore = "sonda de medicao"]
fn probe_o_atrito_trava_o_giro() {
    use crate::SPIN;
    use ph2d_nodegraph::attr::FRICTION_COLUMN;
    const L: f32 = 0.11;
    for mu in [0.0_f32, 0.5, 1.0] {
        let mut p = vec![[0.0_f32, -0.5], [0.0, L]];
        let mut v = vec![[0.0_f32, 0.0], [0.0, 0.0]];
        let mut rot = vec![0.0_f32, 0.0];
        let mut spin = vec![0.0_f32, 180.0]; // meia volta por segundo
        let s0 = spin[1];
        for _ in 0..(30 * 8) {
            let s = Stream::new(2)
                .with("P", Column::Vec2(p.clone()))
                .with("vel", Column::Vec2(v.clone()))
                .with("rot", Column::Scalar(rot.clone()))
                .with(SPIN, Column::Scalar(spin.clone()))
                .with("accel", Column::Vec2(vec![[0.0, -4.0], [0.0, -4.0]]))
                .with("sim_t", Column::Scalar(vec![0.0, 0.0]))
                .with("inv_mass", Column::Scalar(vec![0.0, 1.0]))
                .with(FRICTION_COLUMN, Column::Scalar(vec![mu, mu]))
                .with(COLLIDER_BOX_COLUMN, Column::Vec2(vec![[4.0, 0.5], [L, L]]));
            let out = step(&s, DT / 8.0, 1.0, 0.0, 0.0, 1.0);
            p = col(&out, "P");
            v = col(&out, "vel");
            if let Some(Column::Scalar(r)) = out.get("rot") {
                rot = r.clone();
            }
            if let Some(Column::Scalar(sp)) = out.get(SPIN) {
                spin = sp.clone();
            }
        }
        eprintln!(
            "  μ = {mu:>4.2} : spin {s0:>6.1} °/s -> {:>8.2} °/s apos 0,5 s  (deslocou-se {:.3})",
            spin[1], p[1][0]
        );
    }
    eprintln!(
        "\n  ⚠️ se o spin nao descer com μ, o atrito nao ve' a rotacao e a pilha gira para sempre."
    );
}
