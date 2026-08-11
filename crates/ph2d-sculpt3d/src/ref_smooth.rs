//! **O SMOOTH DA REFERÊNCIA** — o laplaciano com as duas regras de borda, e o
//! kernel que o consome.
//!
//! Filho de [`super`] (`ref_kernels`) e não um módulo solto: ele é a MESMA
//! tradução do mesmo original, e a superfície pública fica plana (`rk::smooth`,
//! `rk::laplacian`) porque quem chama está a falar com **um** porte.
//!
//! ⚠️ **Ele mora num arquivo próprio pela razão que o próprio SculptGL usa:**
//! `Smooth.js` é o único tool de geometria que **não tem falloff**. Os nove
//! irmãos multiplicam pela quártica de [`super::falloff`]; este aplica a MESMA
//! intensidade a toda a pegada, e o `dist` nem é computado
//! (`Smooth.js:39-60` — não há uma linha de distância no laço).
//!
//! ⚠️ **Isso é uma DIVERGÊNCIA do nosso motor de hoje, e ela é visível:** o
//! nosso Smooth pesa pelo falloff escolhido, então a borda da pegada mal se
//! move; o da referência move a borda tanto quanto o miolo, e o resultado tem
//! **degrau na fronteira do pincel**. Portar é reproduzir o degrau. Quem decide
//! se ele fica é o smoke — mas a decisão tem de ser tomada sabendo que ela
//! existe, em vez de descoberta como *"o smooth ficou duro"*.
//!
//! ⚠️ **E ele NÃO é o [`ph2d_mesh::ring_average`].** Os dois computam a média do
//! anel e discordam num caso: quando um vértice de borda tem MENOS de dois
//! vizinhos de borda, o original **cai para a média de TODOS os vizinhos**
//! (`SculptBase.js:344` zera os acumuladores e segue para o laço geral) e o
//! nosso **devolve o próprio ponto**. Eles coexistem porque respondem a
//! chamadores diferentes: aquele é NOSSO (o relax da costura do extract, com
//! gates próprios), este é o PORTE. Colapsá-los seria escolher, em silêncio,
//! qual dos dois consumidores fica errado.

/// A média do anel de cada vértice de `verts`, **empacotada** em `out`.
///
/// ⚠️ **EMPACOTADA — `out[i * 3 + k]` é a média do `verts[i]`**, não do vértice
/// de id `i`. É o `smoothVerts` do original (`Smooth.js:45`, indexado por `i3 =
/// i * 3` enquanto o vértice é `iVerts[i]`), e é a mesma armadilha que o proxy
/// do [`super::r#move`] carrega: com `verts == [0..n)` as duas indexações
/// coincidem e um gate cuja fixture não embaralha a lista **não as distingue**.
///
/// ⚠️ **O armazenamento intermediário é `f32` de propósito:** no original o
/// `smoothVerts` é um `Float32Array`, então a média é arredondada UMA vez ao ser
/// guardada e lida de volta como `f64` pelo kernel. Manter tudo em `f64` até o
/// fim daria outro número — e seria *mais preciso*, que aqui é o mesmo que
/// errado.
///
/// As três fatias do anel são as de [`ph2d_mesh::Csr::parts`]; `on_edge[v] != 0`
/// é o `vertOnEdge` do original, cuja identidade o
/// [`ph2d_mesh::Adjacency::is_border`] já reproduz.
///
/// # As três regras, na ordem em que o original as pergunta
///
/// 1. **anel de dois ou menos** ⇒ o vértice não se move (a média é ele mesmo);
/// 2. **vértice de borda** ⇒ média só dos vizinhos que também são borda, **se
///    houver dois ou mais** — é o que impede a beira de uma malha aberta de ser
///    sugada para dentro;
/// 3. caso contrário ⇒ média de todo o anel.
pub fn laplacian(
    pos: &[f32],
    verts: &[u32],
    ring_starts: &[u32],
    ring_lens: &[u32],
    ring_values: &[u32],
    on_edge: &[u8],
    out: &mut Vec<f32>,
) {
    out.clear();
    out.reserve(verts.len() * 3);
    for &id in verts {
        let vi = id as usize;
        let start = ring_starts[vi] as usize;
        let count = ring_lens[vi] as usize;
        let ring = &ring_values[start..start + count];

        if count <= 2 {
            let ind = vi * 3;
            out.push(pos[ind]);
            out.push(pos[ind + 1]);
            out.push(pos[ind + 2]);
            continue;
        }

        let mut acc = [0.0f64; 3];
        if on_edge[vi] != 0 {
            let mut n = 0u32;
            for &nb in ring {
                if on_edge[nb as usize] == 0 {
                    continue;
                }
                let ind = nb as usize * 3;
                for k in 0..3 {
                    acc[k] += f64::from(pos[ind + k]);
                }
                n += 1;
            }
            if n >= 2 {
                let inv = f64::from(n);
                for a in acc {
                    out.push((a / inv) as f32);
                }
                continue;
            }
            // ⚠️ **Cai para o laço geral, e é aqui que divergimos do
            // `ring_average`.** O original zera os acumuladores e média TODO o
            // anel; o nosso deixa o vértice quieto.
            //
            // ⚠️ **E este ramo é INALCANÇÁVEL em malha manifold — medido, não
            // suposto:** a mutação que o troca pelo comportamento do
            // `ring_average` **sobreviveu ao gate**, porque a curva de borda de
            // uma superfície manifold é um LOOP FECHADO e todo vértice nela tem
            // exatamente dois vizinhos de borda. É a MESMA medição que o
            // `ph2d_mesh::smooth` já carrega, do outro lado da divergência — e é
            // por isso que ela custa zero na prática e mesmo assim as duas
            // funções não podem ser colapsadas: elas discordam num ponto que
            // nenhuma entrada de produto alcança, e a que ficasse teria de ser
            // escolhida sem dado.
            acc = [0.0; 3];
        }

        for &nb in ring {
            let ind = nb as usize * 3;
            for k in 0..3 {
                acc[k] += f64::from(pos[ind + k]);
            }
        }
        let inv = count as f64;
        for a in acc {
            out.push((a / inv) as f32);
        }
    }
}

/// **`Smooth.smooth`** — cada vértice caminha para a média do próprio anel.
///
/// `smoothed` é a saída EMPACOTADA de [`laplacian`] sobre os MESMOS `verts`, na
/// mesma ordem: `smoothed[i * 3 + k]` pertence a `verts[i]`.
///
/// ⚠️ **Sem falloff, sem `dist`, sem raio** — ver o cabeçalho do módulo. O único
/// peso por-vértice é `intensity × free`, e é ele que o `1 − mIntensity`
/// complementa.
pub fn smooth(pos: &mut [f32], free: &[f32], verts: &[u32], smoothed: &[f32], intensity: f64) {
    for (i, &v) in verts.iter().enumerate() {
        let ind = v as usize * 3;
        let i3 = i * 3;
        let m = intensity * f64::from(free[v as usize]);
        let comp = 1.0 - m;
        for k in 0..3 {
            pos[ind + k] =
                (f64::from(pos[ind + k]) * comp + f64::from(smoothed[i3 + k]) * m) as f32;
        }
    }
}
