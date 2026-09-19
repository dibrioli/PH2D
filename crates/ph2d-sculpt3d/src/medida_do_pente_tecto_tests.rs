//! ⭐⭐⭐⭐ **O TECTO da [`super::grade_da_faixa`]** — a calibração da régua que
//! toda esta cena usa, e que nunca tinha sido medida.
//!
//! ⚠️ Ele saiu do irmão por **TECTO DE LOC** (`861` contra `700`), e o corte é
//! por responsabilidade: ali fica a régua, aqui fica **de que é a fracção que
//! ela devolve**.

use ph2d_mesh::{Face, Mesh};

/// ⭐⭐⭐⭐ **O TECTO da [`super::grade_da_faixa`] é DOIS TERÇOS, e sem este
/// gate `64 %` lê-se como «há muito por ganhar».**
///
/// Uma grade quadrada **TRIANGULADA** tem três famílias de aresta: as duas
/// do quadrado — que a dobra de `90°` põe as duas em `0°` — e a
/// **DIAGONAL**, a `45°`, que é exactamente o balde mais afastado. ⇒ numa
/// grade **perfeita** um terço das arestas conta como desalinhado, **por
/// construção**, e o tecto é `2/3`.
///
/// ⚠️⚠️ **Nunca tinha sido medido**, e o produto lê `65,4 %` — **`98,1 %`
/// do tecto**. *Ler uma fracção sem saber de que é a forma de gastar uma
/// wave a perseguir os últimos dois pontos de uma coluna saturada*, e esta
/// linha esteve a um passo disso.
///
/// ⭐ A convergência é pelo lado de cima (`67,11` a `8×8`, `66,69` a
/// `32×32`): a orla da chapa tem menos diagonais do que o miolo.
#[test]
fn o_tecto_da_regua_da_grade_e_dois_tercos() {
    for (n, barra) in [(16usize, 67.0f64), (32, 66.8)] {
        let m = chapa(n);
        let percurso: Vec<[f32; 3]> = (0..9).map(|k| [k as f32 / 8.0 - 0.5, 0.0, 0.0]).collect();
        let (bins, total) = super::grade_da_faixa(&m, &percurso, 1.0);
        let pct = 100.0 * bins[0] as f64 / total.max(1) as f64;
        assert!(
            (66.0..barra).contains(&pct),
            "o tecto da grade a {n}x{n} mede {pct:.2} %, fora de [66, {barra}]"
        );
        // ⭐ E o balde do MEIO tem de estar VAZIO: numa grade perfeita nao
        // existe aresta a 15-30 graus. *Sem esta metade, uma malha
        // qualquer com um terco de arestas a 45 graus passaria.*
        assert_eq!(
            bins[1], 0,
            "uma grade perfeita nao tem arestas a 15-30 graus"
        );
        assert_eq!(
            bins[0] + bins[2],
            total,
            "os tres baldes tem de somar o total"
        );
    }
}

fn chapa(n: usize) -> Mesh {
    let mut pos = Vec::new();
    for j in 0..n {
        for i in 0..n {
            pos.push([
                i as f32 / (n - 1) as f32 - 0.5,
                j as f32 / (n - 1) as f32 - 0.5,
                0.0,
            ]);
        }
    }
    let mut faces = Vec::new();
    for j in 0..n - 1 {
        for i in 0..n - 1 {
            let a = (j * n + i) as u32;
            let (b, c) = (a + 1, a + n as u32);
            faces.push(Face::tri(a, b, c + 1));
            faces.push(Face::tri(a, c + 1, c));
        }
    }
    Mesh::from_parts(pos, faces).expect("chapa")
}

/// ⛔ SONDA — a mesma medida, impressa em tres tamanhos.
///
/// Nunca foi medido, e sem ele `64 %` lê-se como *«mais de metade, e ha'
/// muito por ganhar»*. Uma grade quadrada TRIANGULADA tem tres familias de
/// aresta: as duas do quadrado (que a dobra de `90°` poe as duas em `0°`) e
/// a DIAGONAL, a `45°` — que e' exactamente o balde mais afastado.
#[test]
#[ignore = "sonda: imprime o tecto, nao afirma nada"]
fn diag_o_tecto_da_grade() {
    for n in [8usize, 16, 32] {
        let mut pos = Vec::new();
        for j in 0..n {
            for i in 0..n {
                pos.push([
                    i as f32 / (n - 1) as f32 - 0.5,
                    j as f32 / (n - 1) as f32 - 0.5,
                    0.0,
                ]);
            }
        }
        let mut faces = Vec::new();
        for j in 0..n - 1 {
            for i in 0..n - 1 {
                let a = (j * n + i) as u32;
                let (b, c) = (a + 1, a + n as u32);
                faces.push(Face::tri(a, b, c + 1));
                faces.push(Face::tri(a, c + 1, c));
            }
        }
        let m = Mesh::from_parts(pos, faces).expect("chapa");
        let percurso: Vec<[f32; 3]> = (0..9).map(|k| [k as f32 / 8.0 - 0.5, 0.0, 0.0]).collect();
        let (bins, total) = super::grade_da_faixa(&m, &percurso, 1.0);
        println!(
            "grade PERFEITA {n}x{n}: alinhadas {:.2} % ({}/{})  baldes {bins:?}",
            100.0 * bins[0] as f64 / total.max(1) as f64,
            bins[0],
            total
        );
    }
}
