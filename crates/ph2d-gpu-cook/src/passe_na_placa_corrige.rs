//! **O `corrige` do passe na placa** (doc 132 §3) — a fusão dos nove baldes à volta de uma peça,
//! por índice CRESCENTE (a ordem de soma da CPU), em formas que o compilador do dispositivo trata de
//! maneira diferente. Todas somam os MESMOS parceiros na MESMA ordem; o que muda é onde os nove
//! cursores vivem:
//!
//! - [`Corrige::Matriz`] — quatro matrizes privadas de 9 indexadas dinamicamente (`cab[k] < j`,
//!   `cur[kk]`): o padrão que os compiladores põem na memória LOCAL;
//! - [`Corrige::Ramos`] — os nove desenrolados em variáveis, e o cursor que avança escolhido por uma
//!   cadeia de `if`s (nove cópias da busca da cabeça);
//! - [`Corrige::Escolha`] — os nove desenrolados, o cursor escolhido por `select` e UMA busca.

use std::fmt::Write as _;

/// A forma do `corrige` — só a sonda e o gate escolhem (`MEDIR_VELOCIDADE.md`: as variantes no
/// MESMO processo).
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Corrige {
    #[default]
    Matriz,
    Ramos,
    Escolha,
}

impl Corrige {
    pub const TODAS: [Self; 3] = [Self::Matriz, Self::Ramos, Self::Escolha];

    pub(crate) fn indice(self) -> usize {
        self as usize
    }

    /// O `pnp_corrige` desta forma, concatenado depois de `passe_na_placa_varredura.wgsl`.
    pub(crate) fn fonte(self) -> String {
        match self {
            Self::Matriz => include_str!("passe_na_placa_corrige_matriz.wgsl").to_owned(),
            Self::Ramos => em_registos(false),
            Self::Escolha => em_registos(true),
        }
    }
}

/// O `pnp_corrige` com os nove cursores desenrolados em variáveis.
fn em_registos(escolha: bool) -> String {
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
    if escolha {
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
    } else {
        for k in 0..9 {
            let senao = if k == 0 { "        " } else { " else " };
            let _ = write!(
                s,
                "{senao}if (cab{k} == j) {{\n\
                 \x20           let h = pnp_cabeca(cur{k} + 1u, fim{k}, cel{k}, lado, bi);\n\
                 \x20           cur{k} = h.x;\n\
                 \x20           cab{k} = h.y;\n\
                 \x20       }}"
            );
        }
        s += "\n";
    }
    s += "        if (j == i) { continue; }\n\
          \x20       candidatos = candidatos + 1u;\n\
          \x20       let c = pnp_contribui(i, j, bi, pk, mk, col_k, ligados, acc);\n\
          \x20       acc = c.xyz;\n\
          \x20       if (c.w > 0.0) { contatos = contatos + 1u; }\n\
          \x20   }\n\
          \x20   if (atomicLoad(&ctrl[3]) == 0u) { atomicAdd(&ctrl[4], candidatos); }\n\
          \x20   return pnp_aplica(i, bi, bo, acc, contatos, alc_max);\n\
          }\n";
    s
}
