//! **O `corrige` do passe na placa** (doc 132 §3) — a fusão dos nove baldes à volta de uma peça, por
//! índice CRESCENTE (a ordem de soma da CPU), com os nove cursores DESENROLADOS em variáveis: a menor
//! cabeça por `min`, o cursor que avança escolhido por `select` e UMA busca da cabeça seguinte.
//!
//! ⛔ Recusas MEDIDAS (doc 132 §7; o código está em `38c9767ea5`), a mesma soma ao bit nas três:
//!
//! | forma | nuvem 2 048 · 64, um grupo | cadeia 16 · 1 024 | porquê sai |
//! |---|---:|---:|---|
//! | quatro matrizes de 9 indexadas dinamicamente (a do doc 130) | `25,66 ms` | `11,13` | o padrão que o compilador do dispositivo põe na memória LOCAL |
//! | os nove desenrolados, o cursor por cadeia de `if`s | `19,60` | `11,78` | nove cópias da busca da cabeça |
//! | **os nove desenrolados, o cursor por `select`** | **`13,26`** | **`11,13`** | — |

use std::fmt::Write as _;

/// O `pnp_corrige`, concatenado depois de `passe_na_placa_varredura.wgsl`.
pub(crate) fn fonte() -> String {
    let mut s = String::from(
        "fn pnp_corrige(i: u32, bi: u32, bo: u32, alc_max: f32) -> f32 {\n\
         \x20   let s = est[bi + i];\n\
         \x20   if (u32(dec[4u * i].x) == 0u) {\n\
         \x20       est[bo + i] = s;\n\
         \x20       return -1.0;\n\
         \x20   }\n\
         \x20   let lado = 2.0 * alc_max;\n\
         \x20   let ci = pnp_celula(s.xy, lado);\n",
    );
    // Os 9 baldes à volta e a cabeça de cada um (a peça que vive DE FACTO na célula).
    for k in 0..9i32 {
        let (dx, dy) = (k % 3 - 1, k / 3 - 1);
        let _ = write!(
            s,
            "    let cel{k} = ci + vec2<i32>({dx}, {dy});\n\
             \x20   let b{k} = pnp_balde(cel{k}, u.num_buckets);\n\
             \x20   let fim{k} = atomicLoad(&starts[b{k} + 1u]);\n\
             \x20   let h{k} = pnp_cabeca(atomicLoad(&starts[b{k}]), fim{k}, cel{k}, lado, bi);\n\
             \x20   var cur{k} = h{k}.x;\n\
             \x20   var cab{k} = h{k}.y;\n"
        );
    }
    s += "    let pk = s.xy;\n\
          \x20   let mk = dec[4u * i + 2u];\n\
          \x20   let col_k = pnp_agora(i);\n\
          \x20   let ligados = (u.tem & 256u) != 0u;\n\
          \x20   var acc = vec3<f32>(0.0);\n\
          \x20   var contatos = 0u;\n\
          \x20   var candidatos = 0u;\n\
          \x20   loop {\n";
    // A menor cabeça: uma peça vive numa só célula, logo é a cabeça de UM cursor.
    let mut j = String::from("cab0");
    for k in 1..9 {
        j = format!("min({j}, cab{k})");
    }
    let _ = write!(
        s,
        "        let j = {j};\n        if (j == PNP_NINGUEM) {{ break; }}\n"
    );
    for k in 0..9 {
        let _ = writeln!(s, "        let m{k} = cab{k} == j;");
    }
    s += "        var pc = cur0;\n        var pf = fim0;\n        var pe = cel0;\n";
    for k in 1..9 {
        let _ = write!(
            s,
            "        pc = select(pc, cur{k}, m{k});\n\
             \x20       pf = select(pf, fim{k}, m{k});\n\
             \x20       pe = select(pe, cel{k}, m{k});\n"
        );
    }
    s += "        let h = pnp_cabeca(pc + 1u, pf, pe, lado, bi);\n";
    for k in 0..9 {
        let _ = write!(
            s,
            "        cur{k} = select(cur{k}, h.x, m{k});\n\
             \x20       cab{k} = select(cab{k}, h.y, m{k});\n"
        );
    }
    s += "        if (j == i) { continue; }\n\
          \x20       candidatos = candidatos + 1u;\n\
          \x20       let c = pnp_contribui(i, j, bi, pk, mk, col_k, ligados, acc);\n\
          \x20       acc = c.xyz;\n\
          \x20       if (c.w > 0.0) { contatos = contatos + 1u; }\n\
          \x20   }\n\
          \x20   // O `Relatorio` (doc 130 §5): os candidatos da 1.ª varredura, somados sobre as peças.\n\
          \x20   if (atomicLoad(&ctrl[3]) == 0u) { atomicAdd(&ctrl[4], candidatos); }\n\
          \x20   return pnp_aplica(i, bi, bo, acc, contatos, alc_max);\n\
          }\n";
    s
}
