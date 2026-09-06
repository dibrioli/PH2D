//! ⭐ **A FORÇA POR PASSO DAS ÂNCORAS É ZERADA EM TODA A MALHA** (espec §4.3).
//!
//! ⚠️ **Este ficheiro existe porque a lei sobreviveu a uma mutação.** Apagar o
//! zeramento não mexeu em NENHUM dos 50 traços do oráculo, e a razão é
//! estrutural: a relaxação não filtra por vértice activo — quem apaga a
//! contribuição de um vértice longe é o `φ`, que já é zero fora da banda. As
//! duas leis tapam-se uma à outra em todo o corpus.
//!
//! ⚠️ **Mas elas medem coisas DIFERENTES, e o caso que as separa existe:** a
//! área é decidida sobre as posições **ACTUAIS** e o `φ` sobre as de
//! **REPOUSO**. Um vértice bastante deformado pode sair da área (posição actual
//! longe do cursor) e continuar com `φ > 0` (repouso perto) — e aí a marca
//! velha dele seria aplicada outra vez, contra a espec. *Um corpus que nunca
//! deforma o bastante para separar duas leis não testa nenhuma das duas.*

use crate::V3;
use crate::verlet_gesto::{Area, Modo, Passo, Pincel, PincelTecido};

/// Uma grelha `n × n` no plano `z = 0`, com passo `h`.
fn grelha(n: usize, h: f64) -> (Vec<V3>, Vec<Vec<u32>>) {
    let meio = (n - 1) as f64 * h * 0.5;
    let mut p = Vec::with_capacity(n * n);
    for j in 0..n {
        for i in 0..n {
            p.push([i as f64 * h - meio, j as f64 * h - meio, 0.0]);
        }
    }
    let idx = |i: usize, j: usize| u32::try_from(j * n + i).unwrap_or(u32::MAX);
    let mut faces = Vec::new();
    for j in 0..n - 1 {
        for i in 0..n - 1 {
            faces.push(vec![
                idx(i, j),
                idx(i + 1, j),
                idx(i + 1, j + 1),
                idx(i, j + 1),
            ]);
        }
    }
    (p, faces)
}

fn aneis(n: usize, faces: &[Vec<u32>]) -> Vec<Vec<u32>> {
    let mut a = vec![Vec::new(); n];
    for f in faces {
        for k in 0..f.len() {
            let (p, q) = (f[k] as usize, f[(k + 1) % f.len()] as usize);
            a[p].push(u32::try_from(q).unwrap_or(u32::MAX));
            a[q].push(u32::try_from(p).unwrap_or(u32::MAX));
        }
    }
    for l in &mut a {
        l.sort_unstable();
        l.dedup();
    }
    a
}

/// **GATE — nenhum vértice guarda a força por passo de um passo anterior.**
///
/// O arnês monta o caso que o corpus não tem: área *Dynamic* (o disco anda com
/// o cursor), modo de âncora, e um traço longo o bastante para que vértices
/// puxados no início fiquem para trás. Depois do último passo, todo vértice
/// fora do disco DESTE passo tem de ter `σ = 0`.
///
/// ⚠️ **A régua é a espec, não a nossa aritmética:** `σ` é *«a força por passo,
/// zerada em todo o objecto antes de ser reescrita»*, então o que se afirma é
/// uma propriedade do estado no fim do passo, e não um número.
#[test]
fn nenhum_vertice_guarda_a_forca_por_passo_de_um_passo_anterior() {
    for modo in [Modo::Agarrar, Modo::Gancho] {
        let (rest, faces) = grelha(41, 0.05);
        let an = aneis(rest.len(), &faces);
        let anel = |v: u32| an[v as usize].clone();
        let pincel = Pincel {
            modo,
            area: Area::Dinamica,
            raio: 0.20,
            ..Pincel::default()
        };
        let r = pincel.raio;
        let mut pos = rest.clone();
        let inicio = [-0.5, 0.0, 0.0];
        let mut tecido = PincelTecido::pen_down(pincel, &pos, inicio);
        let mut cursor = inicio;
        let normais = vec![[0.0, 0.0, 1.0]; rest.len()];
        for k in 0..24 {
            let delta = if k == 0 { [0.0; 3] } else { [0.05, 0.0, 0.10] };
            cursor = [
                cursor[0] + delta[0],
                cursor[1] + delta[1],
                cursor[2] + delta[2],
            ];
            let passo = Passo {
                cursor,
                delta,
                // A grelha do arnês vive em `z = 0` e a vista é ao longo de `z`
                // ⇒ a projecção é um no-op e os dois deltas coincidem ao bit.
                delta_3d: delta,
                parado: k == 0,
                vista: [0.0, 0.0, 1.0],
                normais: &normais,
                pressao: 1.0,
            };
            if tecido.passo(&pos, &anel, &passo) {
                for (v, act) in tecido.sim.activo.iter().enumerate() {
                    if *act {
                        pos[v] = tecido.sim.x[v];
                    }
                }
            }
        }
        // Controlo anti-vácuo: o traço tem de ter DEFORMADO, senão «σ = 0 em
        // toda a parte» é verdade num pincel morto.
        let movidos = pos
            .iter()
            .zip(&rest)
            .filter(|(a, b)| crate::verlet::norm([a[0] - b[0], a[1] - b[1], a[2] - b[2]]) > 1e-9)
            .count();
        assert!(movidos > 100, "{modo:?}: só {movidos} movidos — vácuo");
        // E tem de haver quem esteja FORA do disco deste passo com σ escrito
        // num passo anterior, senão o gate não olha para nada.
        let mut fora = 0usize;
        for (v, p) in pos.iter().enumerate() {
            let d = crate::verlet::norm([p[0] - cursor[0], p[1] - cursor[1], p[2] - cursor[2]]);
            if d >= r {
                fora += 1;
                assert!(
                    tecido.sim.sigma[v] == 0.0,
                    "{modo:?}: vertice {v} esta a {d:.4} do cursor (raio {r}) e guarda \
                     forca por passo {} -- a espec §4.3 manda zerar em TODO o objecto",
                    tecido.sim.sigma[v]
                );
            }
        }
        assert!(
            fora > 100,
            "{modo:?}: só {fora} vértices fora do disco — vácuo"
        );
    }
}

/// ⭐⭐⭐ **A NORMAL DA ÁREA só vê METADE do raio** (espec §4.2-bis (3)).
///
/// ⛔ **É a diferença que deixou o `plano_empurrar_plano_local` a errar `0,944`**:
/// a nossa lei somava as normais do disco INTEIRO, e o alvo amostra num disco de
/// `R · 0,5`. Num plano em repouso as duas leituras coincidem — *e é por isso
/// que a fixtura de um passo saía ao bit e o traço inteiro não*.
#[test]
fn a_normal_da_area_so_ve_metade_do_raio() {
    let raio = 1.0;
    let cursor = [0.0, 0.0, 0.0];
    let vista = [0.0, 0.0, 1.0];
    // Dois anéis: um DENTRO de `R/2` e outro fora, com normais bem diferentes.
    let pos = vec![[0.1, 0.0, 0.0], [0.9, 0.0, 0.0]];
    let dentro_do_raio = [0.0, 0.0, 1.0];
    // ⚠️ **O de fora tem de estar no MESMO balde**, senão a regra dos baldes
    // mascara a do raio e o gate sobrevive a trocar `R/2` por `R` — foi o que
    // aconteceu na primeira redacção, e a mutação passou.
    let fora = [0.6, 0.0, 0.8];
    let normais = vec![dentro_do_raio, fora];
    let n = crate::verlet_gesto::normal_da_area(&pos, &normais, &[0, 1], cursor, raio, vista);
    // O de fora nao pode ter entrado: a resposta e' a normal do de dentro.
    for c in 0..3 {
        assert!(
            (n[c] - dentro_do_raio[c]).abs() < 1e-12,
            "a normal da area leu o vertice a {:.2}R do cursor: {n:?}",
            0.9
        );
    }
    // Controlo: com o de dentro FORA da lista, a resposta muda -- senao este
    // gate estaria verde por o segundo vertice nunca contar para nada.
    let so_o_de_fora =
        crate::verlet_gesto::normal_da_area(&pos, &normais, &[1], cursor, raio * 4.0, vista);
    assert!(
        so_o_de_fora != n,
        "o vertice de fora nao conta nem quando o raio o alcanca -- gate vacuo"
    );
}

/// ⭐⭐ **UM vértice virado para a vista apaga todos os virados ao contrário**
/// (espec §4.2-bis (4)) — e a resposta NÃO é uma média nem «o balde com mais».
#[test]
fn um_vertice_virado_para_a_vista_apaga_os_de_costas() {
    let (raio, cursor, vista) = (1.0, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
    // Três de costas e UM de frente, todos dentro de `R/2`.
    let pos = vec![
        [0.05, 0.0, 0.0],
        [0.0, 0.05, 0.0],
        [-0.05, 0.0, 0.0],
        [0.0, -0.05, 0.0],
    ];
    let costas = [0.0, 0.0, -1.0];
    let frente = [0.0, 0.0, 1.0];
    let normais = vec![costas, costas, costas, frente];
    let n = crate::verlet_gesto::normal_da_area(&pos, &normais, &[0, 1, 2, 3], cursor, raio, vista);
    assert!(
        n[2] > 0.9,
        "tres de costas ganharam a UM de frente: {n:?} -- a regra nao e' a maioria"
    );
    // Controlo: sem o de frente, a resposta e' a dos de costas.
    let so_costas =
        crate::verlet_gesto::normal_da_area(&pos, &normais, &[0, 1, 2], cursor, raio, vista);
    assert!(so_costas[2] < -0.9, "sem o de frente: {so_costas:?}");
}

/// ⭐⭐ **Sem balde válido a normal é NULA**, e o teste é *não-vazio E soma
/// não-nula*, balde a balde (espec §4.2-bis (4) e (5)).
///
/// ⚠️⚠️ **E há uma coisa que a redacção da espec não deixa ver, e que escrever
/// este gate revelou: o balde da FRENTE, se não estiver vazio, NUNCA tem soma
/// nula.** A soma dele é `Σ wᵢ n̂ᵢ` com todos os `wᵢ > 0` e todos os `n̂ᵢ · v̂ > 0`,
/// logo `(Σ wᵢ n̂ᵢ) · v̂ > 0` e o vector não pode ser zero. ⇒ *a cláusula «e soma
/// não-nula» só é observável no balde de TRÁS*, onde `n̂ · v̂ ≤ 0` admite o zero e
/// duas normais perpendiculares à vista podem cancelar-se. É por isso que a
/// primeira redacção deste gate era IMPOSSÍVEL de satisfazer.
#[test]
fn sem_balde_valido_a_normal_da_area_e_nula() {
    let (raio, cursor, vista) = (1.0, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
    // (a) Ninguém dentro do alcance ⇒ nulo.
    let pos = vec![[0.9, 0.0, 0.0]];
    let normais = vec![[0.0, 0.0, 1.0]];
    let n = crate::verlet_gesto::normal_da_area(&pos, &normais, &[0], cursor, raio, vista);
    assert_eq!(
        n, [0.0; 3],
        "sem vertice no alcance a normal tem de ser nula"
    );

    // (b) Balde da frente VAZIO e o de trás com soma não-nula ⇒ ganha o de trás.
    let pos = vec![[0.05, 0.0, 0.0], [0.0, 0.05, 0.0]];
    let atras = [0.0, 0.0, -1.0];
    let normais = vec![atras, atras];
    let n = crate::verlet_gesto::normal_da_area(&pos, &normais, &[0, 1], cursor, raio, vista);
    assert!(n[2] < -0.9, "o balde de tras devia ter respondido: {n:?}");

    // (c) ⭐ O caso que a cláusula «soma não-nula» existe para apanhar: os dois
    // vértices são PERPENDICULARES à vista (`n̂ · v̂ = 0` ⇒ balde de trás) e
    // cancelam-se ⇒ nenhum balde passa, e a resposta é o vector NULO.
    let normais = vec![[1.0, 0.0, 0.0], [-1.0, 0.0, 0.0]];
    let n = crate::verlet_gesto::normal_da_area(&pos, &normais, &[0, 1], cursor, raio, vista);
    assert_eq!(
        n, [0.0; 3],
        "o balde de tras cancelou-se e a resposta nao foi nula: {n:?} -- sem esta \
         clausula o `unit` de um vector nulo devolveria NaN"
    );
}

/// ⭐ **A normal da área PESA cada vértice por `3p² − 2p³`**, com
/// `p = 1 − d/(R·0,5)` — o de perto do cursor conta mais (espec §4.2-bis (3)).
///
/// ⛔⛔ **Este gate existe porque a mutação que apaga o peso SOBREVIVEU à
/// paridade inteira**, e não por ser inofensiva: sem peso, cinco traços do
/// oráculo mudam (`plano_empurrar_radial_local` `0,214 → 0,194`), e todos ficam
/// **ligeiramente melhores**. ⚠️ *Isso não absolve a soma crua — absolve o peso
/// de ser a causa do resíduo do Push, e diz que há outra coisa a compensá-lo.*
/// A lei que shipa é a da espec, que foi lida no fonte e atestada; a medição
/// fica registada no plano do que falta.
///
/// A fixtura é a mais simples que os separa: duas normais diferentes a
/// distâncias diferentes, dentro do meio-raio e no MESMO balde.
#[test]
fn a_normal_da_area_pesa_o_vertice_pela_distancia() {
    let (raio, cursor, vista) = (1.0, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
    // `d = 0,05` ⇒ `p = 0,9` ⇒ peso `0,972`; `d = 0,45` ⇒ `p = 0,1` ⇒ `0,028`.
    let pos = vec![[0.05, 0.0, 0.0], [0.45, 0.0, 0.0]];
    let perto = [0.0, 0.0, 1.0];
    let longe = [1.0, 0.0, 0.2];
    let normais = vec![perto, longe];
    let n = crate::verlet_gesto::normal_da_area(&pos, &normais, &[0, 1], cursor, raio, vista);
    // Com o peso, o de longe quase não conta: a resposta fica muito perto do
    // de perto. Sem peso ela cairia a meio caminho entre os dois.
    let sem_peso = {
        let s = [
            perto[0] + longe[0],
            perto[1] + longe[1],
            perto[2] + longe[2],
        ];
        crate::verlet::unit(s)
    };
    assert!(
        n[0] < 0.15,
        "o vertice a 0,45 pesou como o de 0,05: {n:?} -- a soma e' crua"
    );
    assert!(
        (n[0] - sem_peso[0]).abs() > 0.3,
        "a resposta com peso ({n:?}) nao se distingue da soma crua ({sem_peso:?}) \
         -- a fixtura nao separa as duas leis"
    );
}
