//! Gates da ARRUMAÇÃO — hoje, a igualação de densidade.
//!
//! ⚠️ **A fixtura é DUAS peças de áreas conhecidas e cartas do MESMO tamanho**, e é ela
//! que contém o fenómeno: a peça grande recebe a mesma carta que a pequena, logo nasce
//! com **metade** dos texels por unidade de superfície. *A fita de três cartas do
//! [`super::lib_tests`] varia `5,7 %` — ela mede a lei e não a distingue do ruído.*

use super::arrumacao::iguala_a_densidade;
use ph2d_mesh::{Face, Mesh};

/// Duas chapas soltas: a peça `0` mede `1×1` no mundo e a peça `1` mede `2×2`.
/// As duas recebem a MESMA carta unitária no plano.
fn duas_chapas() -> (Mesh, Vec<u32>, Vec<[f32; 2]>, Vec<u32>) {
    let pos = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [2.0, 0.0, 0.0],
        [4.0, 0.0, 0.0],
        [4.0, 2.0, 0.0],
        [2.0, 2.0, 0.0],
    ];
    let faces = vec![Face::quad(0, 1, 2, 3), Face::quad(4, 5, 6, 7)];
    let mesh = Mesh::from_parts(pos, faces).expect("a fixtura e' valida");
    let base = vec![0u32, 4];
    // ⚠️ A 2.ª carta nasce deslocada de propósito: a densidade é invariante a onde a
    // peça está, e uma fixtura com as duas na origem não distinguiria isso de uma lei
    // que dependesse da posição.
    let plano = vec![
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
        [7.0, 5.0],
        [8.0, 5.0],
        [8.0, 6.0],
        [7.0, 6.0],
    ];
    (mesh, base, plano, vec![0, 1])
}

/// A área somada de uma peça no plano, pelo mesmo leque que a lei usa.
fn area_do_plano(mesh: &Mesh, base: &[u32], plano: &[[f32; 2]], peca: &[u32], i: u32) -> f64 {
    let mut soma = 0.0;
    for (f, face) in mesh.faces().iter().enumerate() {
        if peca[f] != i {
            continue;
        }
        let b = base[f] as usize;
        for k in 1..face.verts().len() - 1 {
            let (a, c, d) = (plano[b], plano[b + k], plano[b + k + 1]);
            let (ux, uy) = (f64::from(c[0] - a[0]), f64::from(c[1] - a[1]));
            let (vx, vy) = (f64::from(d[0] - a[0]), f64::from(d[1] - a[1]));
            soma += (ux.mul_add(vy, -(uy * vx)) * 0.5).abs();
        }
    }
    soma
}

#[test]
fn as_duas_pecas_passam_a_receber_os_mesmos_texels_por_unidade_de_superficie() {
    let (mesh, base, plano0, peca) = duas_chapas();

    // ⭐ O CONTROLO: antes da lei, a peça grande tem METADE da densidade da pequena.
    let d = |p: &[[f32; 2]], i: u32, mundo: f64| {
        (area_do_plano(&mesh, &base, p, &peca, i) / mundo).sqrt()
    };
    let antes = [d(&plano0, 0, 1.0), d(&plano0, 1, 4.0)];
    assert!(
        (antes[0] / antes[1] - 2.0).abs() < 1.0e-6,
        "a fixtura tinha de conter o fenomeno: densidades {antes:?}"
    );

    let mut plano = plano0.clone();
    let faixa = iguala_a_densidade(&mesh, &base, &mut plano, &peca, 2);
    let depois = [d(&plano, 0, 1.0), d(&plano, 1, 4.0)];
    assert!(
        (depois[0] / depois[1] - 1.0).abs() < 1.0e-5,
        "as duas pecas tinham de ficar com a mesma densidade: {depois:?}"
    );

    // ⭐⭐ E o alvo PRESERVA a tinta total, exactamente — a propriedade que o doc da lei
    // demonstra. ⚠️ Sem esta metade, uma lei que encolhesse TUDO passaria na de cima.
    let soma = |p: &[[f32; 2]]| {
        area_do_plano(&mesh, &base, p, &peca, 0) + area_do_plano(&mesh, &base, p, &peca, 1)
    };
    assert!(
        (soma(&plano) / soma(&plano0) - 1.0).abs() < 1.0e-6,
        "a tinta total mudou: {} contra {}",
        soma(&plano),
        soma(&plano0)
    );

    // ⭐ A faixa devolvida é o TAMANHO do defeito: `√0,4` e `2 · √0,4`.
    assert!(
        (faixa[1] / faixa[0] - 2.0).abs() < 1.0e-5,
        "a faixa tinha de dizer o dobro: {faixa:?}"
    );
}

#[test]
fn uma_peca_que_ja_esta_no_alvo_nao_e_tocada_ao_bit() {
    // Duas chapas do MESMO tamanho com a MESMA carta: todo factor é exactamente `1`.
    let pos = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [2.0, 0.0, 0.0],
        [3.0, 0.0, 0.0],
        [3.0, 1.0, 0.0],
        [2.0, 1.0, 0.0],
    ];
    let faces = vec![Face::quad(0, 1, 2, 3), Face::quad(4, 5, 6, 7)];
    let mesh = Mesh::from_parts(pos, faces).expect("a fixtura e' valida");
    let base = vec![0u32, 4];
    let plano0 = vec![
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
        [7.25, 5.5],
        [8.25, 5.5],
        [8.25, 6.5],
        [7.25, 6.5],
    ];
    let mut plano = plano0.clone();
    let faixa = iguala_a_densidade(&mesh, &base, &mut plano, &[0, 1], 2);
    assert_eq!(faixa, [1.0, 1.0], "nada havia a corrigir");
    assert_eq!(
        plano, plano0,
        "uma peca ja' no alvo tem de sair AO BIT — e' `q * 1,0` que o garante, sem cerca \
         nenhuma: a metade que esta asserção defende e' o ALVO, que tem de ser o que ja' \
         la' esta"
    );
}

#[test]
fn uma_peca_sem_area_nao_derruba_a_lei() {
    // ⛔ O piso de população: uma lasca degenerada (área ZERO no mundo) não tem
    // densidade, e a resposta certa é deixá-la como está — nunca dividir por zero.
    let pos = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [2.0, 0.0, 0.0],
        [3.0, 0.0, 0.0],
        [4.0, 0.0, 0.0],
    ];
    let faces = vec![Face::quad(0, 1, 2, 3), Face::tri(4, 5, 6)];
    let mesh = Mesh::from_parts(pos, faces).expect("a fixtura e' valida");
    let base = vec![0u32, 4];
    let plano0 = vec![
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
        [7.0, 5.0],
        [8.0, 5.0],
        [9.0, 5.0],
    ];
    let mut plano = plano0.clone();
    let faixa = iguala_a_densidade(&mesh, &base, &mut plano, &[0, 1], 2);
    assert!(
        faixa.iter().all(|f| f.is_finite() && *f > 0.0),
        "a faixa tem de continuar finita com uma lasca na cena: {faixa:?}"
    );
    assert_eq!(
        plano[4..],
        plano0[4..],
        "a lasca sem area nao se escala — nao ha' densidade para igualar"
    );
}

#[test]
fn o_tecto_trava_a_lasca_e_deixa_de_igualar_de_proposito() {
    // Duas chapas `1×1` no mundo: a peça `1` recebe uma carta `20×` mais pequena, logo
    // ela precisaria de `~14×` para chegar ao alvo. ⛔ O tecto dá-lhe `3` e para.
    let pos = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [2.0, 0.0, 0.0],
        [3.0, 0.0, 0.0],
        [3.0, 1.0, 0.0],
        [2.0, 1.0, 0.0],
    ];
    let faces = vec![Face::quad(0, 1, 2, 3), Face::quad(4, 5, 6, 7)];
    let mesh = Mesh::from_parts(pos, faces).expect("a fixtura e' valida");
    let base = vec![0u32, 4];
    let mut plano = vec![
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
        [7.0, 5.0],
        [7.05, 5.0],
        [7.05, 5.05],
        [7.0, 5.05],
    ];
    let antes = area_do_plano(&mesh, &base, &plano, &[0, 1], 0)
        + area_do_plano(&mesh, &base, &plano, &[0, 1], 1);
    let faixa = iguala_a_densidade(&mesh, &base, &mut plano, &[0, 1], 2);
    assert!(
        (faixa[1] - super::arrumacao::TECTO_DA_IGUALACAO).abs() < 1.0e-6,
        "a lasca tinha de travar no tecto: {faixa:?}"
    );

    // ⭐ E a metade que torna isto uma DECISÃO e não um acidente: travada, ela continua
    // longe do alvo — a lei DESISTE de igualar esta peça, e é isso que compra a tinta.
    let d = |i: u32, mundo: f64| (area_do_plano(&mesh, &base, &plano, &[0, 1], i) / mundo).sqrt();
    assert!(
        d(0, 1.0) / d(1, 1.0) > 2.0,
        "travada, a peca tem de continuar por igualar: {} contra {}",
        d(0, 1.0),
        d(1, 1.0)
    );

    // ⚠️ **E a promessa da tinta total NÃO vale aqui**, e o doc da lei diz porquê. Sem
    // esta linha alguém leria a asserção do gate irmão como uma lei sem cláusula.
    let depois = area_do_plano(&mesh, &base, &plano, &[0, 1], 0)
        + area_do_plano(&mesh, &base, &plano, &[0, 1], 1);
    assert!(
        depois < antes * 0.999,
        "com o tecto a morder a tinta TEM de encolher: {depois} contra {antes}"
    );
}

/// ⭐⭐⭐ **A régua é o PRODUTO.** Os gates de cima chamam a lei; este percorre o
/// [`super::build`], que é o que qualquer consumidor corre.
///
/// ⛔⛔ Sem ele, **três** mutações sobrevivem sem sangrar uma linha: a porta a nascer
/// desligada, a chamada a desaparecer do pipeline, e o tecto a descer a `1`. *Um gate
/// que chama a função em vez de percorrer a rota afirma que a lei existe, nunca que o
/// produto a corre* — a lei que esta casa paga em cada wave que a esquece.
#[test]
fn o_atlas_de_fabrica_entrega_a_mesma_densidade_em_todas_as_pecas() {
    let (mesh, cut, map, jumps) = super::lib_tests::fita(0, false);
    // As três cartas do prisma nascem com densidades parecidas; aqui elas são esticadas
    // de propósito, que é o que faz esta fixtura conter o fenómeno.
    //
    // ⚠️⚠️ **O esticão está DENTRO da janela do [`super::arrumacao::TECTO_DA_IGUALACAO`],
    // de propósito, e a 1.ª redacção não estava:** com `3×`/`0,5×` a terceira carta pedia
    // `3,76×`, travava no tecto e o gate reprovava sobre uma lei CERTA. *Um gate da
    // igualação medido num regime onde o tecto morde mede o tecto* — e o tecto tem o
    // gate dele, com a fixtura própria.
    let mut esticado = map.clone();
    for q in &mut esticado.uv[1] {
        q[0] *= 2.0;
        q[1] *= 2.0;
    }
    for q in &mut esticado.uv[2] {
        q[0] *= 0.6;
        q[1] *= 0.6;
    }

    let densidades = |atlas: &super::Atlas| -> Vec<f64> {
        let (base, _) = super::bases_dos_cantos(&mesh);
        let pos = mesh.positions();
        let mut uv = std::collections::BTreeMap::<u32, (f64, f64)>::new();
        for (f, face) in mesh.faces().iter().enumerate() {
            let (b, vs) = (base[f] as usize, face.verts());
            let e = uv.entry(atlas.ilha[b]).or_insert((0.0, 0.0));
            for k in 1..vs.len() - 1 {
                let (a, c, d) = (atlas.uv[b], atlas.uv[b + k], atlas.uv[b + k + 1]);
                let (ux, uy) = (f64::from(c[0] - a[0]), f64::from(c[1] - a[1]));
                let (vx, vy) = (f64::from(d[0] - a[0]), f64::from(d[1] - a[1]));
                e.0 += (ux.mul_add(vy, -(uy * vx)) * 0.5).abs();
                let (p, q, r) = (
                    pos[vs[0] as usize],
                    pos[vs[k] as usize],
                    pos[vs[k + 1] as usize],
                );
                let (u, v) = (
                    [
                        f64::from(q[0] - p[0]),
                        f64::from(q[1] - p[1]),
                        f64::from(q[2] - p[2]),
                    ],
                    [
                        f64::from(r[0] - p[0]),
                        f64::from(r[1] - p[1]),
                        f64::from(r[2] - p[2]),
                    ],
                );
                let n = [
                    u[1].mul_add(v[2], -(u[2] * v[1])),
                    u[2].mul_add(v[0], -(u[0] * v[2])),
                    u[0].mul_add(v[1], -(u[1] * v[0])),
                ];
                e.1 += 0.5 * n[0].mul_add(n[0], n[1].mul_add(n[1], n[2] * n[2])).sqrt();
            }
        }
        uv.values().map(|(a, m)| (a / m).sqrt()).collect()
    };
    let faixa = |d: &[f64]| {
        let (lo, hi) = d
            .iter()
            .fold((f64::MAX, f64::MIN), |(l, h), &x| (l.min(x), h.max(x)));
        hi / lo
    };

    // ⭐ O CONTROLO: com a porta fechada a fixtura mostra o defeito — e sem esta metade
    // o gate abaixo passaria sobre um atlas que nunca teve o fenómeno.
    let sem = super::build_com(
        &mesh,
        &cut,
        &map_esticado(&esticado),
        &jumps,
        super::Opcoes {
            densidade_igual: false,
            ..super::Opcoes::default()
        },
    );
    let d_sem = densidades(&sem);
    assert!(
        d_sem.len() >= 3,
        "a fixtura tinha de dar pelo menos tres pecas: {}",
        d_sem.len()
    );
    assert!(
        faixa(&d_sem) > 2.0,
        "a fixtura tinha de conter o fenomeno: {d_sem:?}"
    );

    let com = super::build(&mesh, &cut, &map_esticado(&esticado), &jumps);
    let d_com = densidades(&com);
    assert!(
        faixa(&d_com) < 1.01,
        "de fabrica as pecas tinham de sair com a mesma densidade: {d_com:?}"
    );
    assert_ne!(
        com.relatorio.escala_das_pecas,
        [0.0, 0.0],
        "a porta de fabrica tem de ter CORRIDO — `[0, 0]` e' o byte de «nao medido»"
    );
}

/// O `GridMap` não é `Clone` em todas as versões desta cadeia; esta cópia rasa é o que
/// o gate acima precisa e mais nada.
fn map_esticado(m: &ph2d_gridmap::GridMap) -> ph2d_gridmap::GridMap {
    ph2d_gridmap::GridMap {
        uv: m.uv.clone(),
        shift: m.shift.clone(),
    }
}
