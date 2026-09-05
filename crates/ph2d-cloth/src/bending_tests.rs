//! ⭐⭐⭐ **A DERIVADA DA DOBRA, conferida contra a ENERGIA — e o repouso CURVO.**
//!
//! ⚠️ **A fixtura é dobrada de propósito.** Numa dobradiça plana o modelo
//! quadrático (refutado — ver [`crate::bending`]) e o do ângulo diedro concordam,
//! então uma fixtura plana **aprovaria o modelo errado**. Aqui o repouso já tem
//! ângulo, que é o caso de toda escultura.

use crate::{ClothMaterial, ClothRest, ClothTopology, V3, bending, energy, fixtures};

fn mat() -> ClothMaterial {
    ClothMaterial {
        young: 0.0,
        bending: 0.5,
        ..ClothMaterial::default()
    }
}

fn e_of(topo: &ClothTopology, rest: &ClothRest, x: &[V3]) -> f64 {
    energy(topo, rest, &mat(), x)
}

fn armar() -> (Vec<V3>, ClothTopology, ClothRest) {
    let (x, t) = fixtures::hinge_pair();
    let topo = fixtures::region(&x, &t);
    assert_eq!(topo.hinge_count(), 1, "a fixtura tem UMA dobradica");
    let rest = ClothRest::measure(&topo, &x, &mat());
    (x, topo, rest)
}

/// ⭐⭐⭐ **GATE — um repouso CURVO não tem energia de dobra.**
///
/// ⛔⛔ **É o gate que refuta o modelo quadrático**, e por isso ele é o mais
/// importante deste arquivo: aquele modelo mede o desvio do PLANO, então nesta
/// mesma fixtura (que está dobrada no repouso) ele daria energia e força. Aqui a
/// medida é o desvio do **próprio repouso**, e ela é zero ao bit.
#[test]
fn um_repouso_curvo_nao_tem_energia_de_dobra() {
    let (x, topo, rest) = armar();
    let ang = bending::dihedral(&x, topo.hinges[0]);
    assert!(
        ang.abs() > 0.2,
        "a fixtura nao contem o fenomeno: ela esta' quase plana ({ang:.4} rad)"
    );
    assert_eq!(e_of(&topo, &rest, &x), 0.0);
    for slot in 0..4 {
        let (g, _) = bending::accumulate(&x, topo.hinges[0], &rest.hinge[0], mat().bending, slot);
        assert_eq!(g, [0.0; 3], "slot {slot} empurra no repouso curvo");
    }
}

/// ⭐⭐⭐ **GATE — as quatro derivadas do ângulo SOMAM ZERO.**
///
/// ⚠️ **É a própria construção**, e não uma verificação extra: as duas derivadas
/// da aresta são derivadas das dos ápices exatamente por esta invariância
/// (transladar os quatro não muda o ângulo). Um sinal trocado a quebra.
#[test]
fn as_quatro_derivadas_do_angulo_somam_zero() {
    let (x, topo, _) = armar();
    let g = bending::grads(&x, topo.hinges[0]);
    for c in 0..3 {
        let s: f64 = g.iter().map(|v| v[c]).sum();
        assert!(s.abs() < 1e-12, "eixo {c} soma {s:.3e}");
    }
}

/// ⭐⭐⭐ **GATE — o gradiente do ÂNGULO é a derivada do ângulo.**
///
/// ⚠️ Mede-se o **ângulo**, não a energia: no repouso a energia tem derivada nula
/// e um sinal trocado no ângulo passaria despercebido ali. *Uma régua avaliada no
/// ponto em que a grandeza é estacionária não vê o sinal dela.*
#[test]
fn o_gradiente_do_angulo_bate_com_a_diferenca_finita() {
    let (x, topo, _) = armar();
    let hg = topo.hinges[0];
    let g = bending::grads(&x, hg);
    let step = 1e-6;
    let mut pior = 0.0f64;
    for (slot, v) in hg.verts().into_iter().enumerate() {
        for c in 0..3 {
            let (mut a, mut b) = (x.clone(), x.clone());
            a[v as usize][c] += step;
            b[v as usize][c] -= step;
            let fd = (bending::dihedral(&a, hg) - bending::dihedral(&b, hg)) / (2.0 * step);
            pior = pior.max((fd - g[slot][c]).abs() / fd.abs().max(1.0));
        }
    }
    assert!(pior < 1e-6, "gradiente do angulo: {pior:.3e}");
}

/// ⭐⭐⭐ **GATE — o gradiente da ENERGIA é a derivada da energia, fora do repouso.**
#[test]
fn o_gradiente_da_energia_bate_com_a_diferenca_finita() {
    let (x0, topo, rest) = armar();
    let mut x = x0.clone();
    x[2] = [0.4, 0.9, 0.75];
    x[3] = [0.6, -0.8, -0.15];
    let step = 1e-6;
    let mut pior = 0.0f64;
    for (slot, v) in topo.hinges[0].verts().into_iter().enumerate() {
        let (g, _) = bending::accumulate(&x, topo.hinges[0], &rest.hinge[0], mat().bending, slot);
        for c in 0..3 {
            let (mut a, mut b) = (x.clone(), x.clone());
            a[v as usize][c] += step;
            b[v as usize][c] -= step;
            let fd = (e_of(&topo, &rest, &a) - e_of(&topo, &rest, &b)) / (2.0 * step);
            pior = pior.max((fd - g[c]).abs() / fd.abs().max(1.0));
        }
    }
    assert!(pior < 1e-6, "gradiente da energia de dobra: {pior:.3e}");
}

/// ⭐⭐⭐ **GATE — o ângulo tem SINAL.**
///
/// ⛔ Sem sinal, dobrar para um lado e para o outro leriam igual e o pano não
/// teria como voltar: a força de restauração apontaria para o mesmo lado nos dois
/// casos, e uma prega uma vez feita nunca desfaria.
#[test]
fn o_angulo_tem_sinal() {
    let (x0, topo, _) = armar();
    let hg = topo.hinges[0];
    let (mut up, mut down) = (x0.clone(), x0.clone());
    up[2][2] += 0.5;
    down[2][2] -= 0.5;
    let (a, b) = (bending::dihedral(&up, hg), bending::dihedral(&down, hg));
    assert!(
        (a - bending::dihedral(&x0, hg)) * (b - bending::dihedral(&x0, hg)) < 0.0,
        "dobrar para os dois lados deu o mesmo sinal: {a:.4} e {b:.4}"
    );
}

/// ⭐⭐⭐ **GATE — a Hessiana de Gauss-Newton é semi-definida POSITIVA.**
///
/// ⚠️ É a propriedade pela qual ela foi escolhida (o termo com `∂²θ` foi
/// descartado de propósito): com o gradiente exato e uma métrica PSD, o passo
/// local é **garantidamente de descida**. Um produto externo é PSD por
/// construção — e este gate é o que impede alguém de "melhorar" a Hessiana
/// acrescentando o termo que falta sem medir o que isso custa na estabilidade.
#[test]
fn a_hessiana_da_dobra_e_semi_definida_positiva() {
    let (x0, topo, rest) = armar();
    let mut x = x0.clone();
    x[2] = [0.4, 0.9, 0.9];
    for slot in 0..4 {
        let (_, h) = bending::accumulate(&x, topo.hinges[0], &rest.hinge[0], mat().bending, slot);
        for d in [
            [1.0, 0.0, 0.0],
            [0.3, -0.7, 0.5],
            [-0.2, 0.1, 0.9],
            [1.0, 1.0, 1.0],
        ] {
            let q: f64 = (0..3)
                .map(|r| d[r] * (0..3).map(|c| h[r][c] * d[c]).sum::<f64>())
                .sum();
            assert!(q >= -1e-12, "slot {slot} deu forma quadratica {q:.3e}");
        }
    }
}
