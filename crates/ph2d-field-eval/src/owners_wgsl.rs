//! ⭐⭐⭐ **A LEI DO DONO ESCRITA EM WGSL** — o que leva o material por objecto ao dispositivo.
//!
//! # ⚠️ Porque ela é uma wave e não «mais um slot»
//!
//! O `{FIELD}` do [`crate::wgsl`] leva **uma** fita: a da peça inteira, que é a união de tudo. O
//! sombreamento não pergunta *«onde está a superfície?»* — pergunta ***«de quem é este ponto?»***, e
//! essa resposta precisa de **uma fita por FOLHA** ([`super::Owners`]), da bola de cada uma, e da
//! mesma aritmética de desempate que a CPU corre. ⇒ o que atravessa não é um número, é a lei.
//!
//! ⛔⛔ **E a alternativa — um *id-buffer* — continua RECUSADA** (`super`): ela arrastaria um
//! segundo canal por cada passo da marcha. Aqui o dono resolve-se **uma vez por ponto**, no passe
//! que pinta, exactamente como na CPU.
//!
//! # ⚠️ As constantes viajam TODAS no mesmo vector, e o `const_base` é o contrato
//!
//! O molde do traçado já tem um `k` ligado ([`crate::wgsl::CONSTS`]) com as constantes da fita da
//! peça. As folhas escrevem **a seguir**, e é o chamador que concatena os dois vectores — a origem
//! que ele passa aqui e a ordem com que ele concatena são a **mesma decisão**, e discordarem pinta
//! cada folha com os números da vizinha sem erro nenhum.
//!
//! *Um arrasto de slider continua a reescrever um buffer e a não recompilar nada.*

use super::Owners;

/// A lei do dono, em WGSL, com as constantes de fora — o irmão do [`crate::wgsl::TapeWgsl`].
#[derive(Clone, Debug, PartialEq)]
pub struct OwnersWgsl {
    /// As `N` funções de folha mais `dono_mix`, que devolve `Dono { a, b, t }`.
    ///
    /// ⭐ **É esta a chave do cache de pipelines** — ela não muda quando um número muda.
    pub source: String,
    /// Os valores que este texto indexa, a partir do `const_base` que lhe foi dado: as constantes
    /// de cada folha, na ordem, depois **uma bola por folha** (`centro.xyz`, `raio`) e a **margem**.
    pub consts: Vec<f32>,
}

/// Quantas constantes uma bola ocupa no vector — `centro.xyz` mais o raio.
const BOLA: usize = 4;

impl Owners {
    /// ⭐⭐⭐ **A lei do dono desta peça, escrita em WGSL.**
    ///
    /// `None` quando **alguma** folha não tem fita — a mesma resposta que o [`crate::Field::at`] dá
    /// com `NaN`, e a mesma cerca que o [`crate::Field::tape_wgsl`] tem. ⚠️ É tudo ou nada de
    /// propósito: uma folha em falta não é «uma folha a menos», é **a peça toda com o dono errado**
    /// a partir dali.
    ///
    /// ⚠️ `const_base` é o índice em que estas constantes começam no vector partilhado — ver a nota
    /// do módulo.
    #[must_use]
    pub fn to_wgsl(&self, const_base: usize) -> Option<OwnersWgsl> {
        if self.leaves.is_empty() {
            return None;
        }
        let mut consts: Vec<f32> = Vec::new();
        let mut s = String::with_capacity(self.leaves.len() * 4096);
        for (i, (_, campo)) in self.leaves.iter().enumerate() {
            let t = campo
                .tape
                .to_wgsl_named(&format!("dono_folha_{i}"), const_base + consts.len())?;
            s.push_str(&t.source);
            consts.extend_from_slice(&t.consts);
        }
        let bolas = const_base + consts.len();
        for (b, _) in &self.leaves {
            consts.extend_from_slice(&[b.center[0], b.center[1], b.center[2], b.radius]);
        }
        let margem = const_base + consts.len();
        consts.push(self.margin);

        let n = self.leaves.len();
        let k = crate::wgsl::CONSTS;
        s.push_str(&format!(
            "\nconst DONO_FOLHAS: u32 = {n}u;\n\
             const DONO_BOLAS: u32 = {bolas}u;\n\
             const DONO_MARGEM: u32 = {margem}u;\n"
        ));
        // ⚠️ **O `switch` é sobre um índice de RUNTIME**, logo o compilador insere as `N` fitas e
        // escolhe uma. Não há forma de o evitar: a pergunta é *«quanto vale a folha `i` AQUI?»*, e
        // `i` só se sabe depois de o filtro correr.
        s.push_str("fn dono_campo(i: u32, p: vec3<f32>) -> f32 {\n  switch i {\n");
        for i in 0..n {
            s.push_str(&format!("    case {i}u: {{ return dono_folha_{i}(p); }}\n"));
        }
        s.push_str("    default: { return 0.0; }\n  }\n}\n");
        s.push_str(&format!(
            r"
fn dono_bola(i: u32) -> vec4<f32> {{
  let o = DONO_BOLAS + i * 4u;
  return vec4<f32>({k}[o], {k}[o + 1u], {k}[o + 2u], {k}[o + 3u]);
}}

// ⚠️⚠️ **A MARGEM não é folga, é obrigatória** — ver `ph2d_field_eval::owners`. A marcha pára
// LIGEIRAMENTE FORA da superfície, e um filtro exacto rejeita a resposta certa em todo o lado.
fn dono_dentro(i: u32, p: vec3<f32>, margem: f32) -> bool {{
  let b = dono_bola(i);
  let d = p - b.xyz;
  let r = b.w + margem;
  return dot(d, d) <= r * r;
}}

struct Dono {{ a: u32, b: u32, t: f32 }};

// ⭐⭐⭐ A `Owners::mix_at` da CPU, ramo a ramo — incluindo o DESEMPATE, que é observável: com dois
// valores iguais ganha o de índice MENOR, e trocá-lo pinta metade de uma união com a cor errada.
fn dono_mix(p: vec3<f32>, width: f32) -> Dono {{
  let margem = max({k}[DONO_MARGEM], width);
  // ⚠️ **A rede é uma pergunta, não um `if` escondido**: com uma mistura suave nenhuma bola contém
  // o ponto, e aí o filtro não filtra — ele responderia «ninguém» sobre uma peça visível.
  var filtra = false;
  if (DONO_FOLHAS > 1u) {{
    for (var i = 0u; i < DONO_FOLHAS; i = i + 1u) {{
      if (dono_dentro(i, p, margem)) {{ filtra = true; break; }}
    }}
  }}
  var tem_b = false; var bv = 0.0; var bi = 0u;
  var tem_s = false; var sv = 0.0; var si = 0u;
  for (var i = 0u; i < DONO_FOLHAS; i = i + 1u) {{
    if (filtra && !dono_dentro(i, p, margem)) {{ continue; }}
    let v = abs(dono_campo(i, p));
    if (tem_b && v >= bv) {{
      if (!tem_s || v < sv) {{ sv = v; si = i; tem_s = true; }}
    }} else {{
      tem_s = tem_b; sv = bv; si = bi;
      bv = v; bi = i; tem_b = true;
    }}
  }}
  if (!tem_b) {{ return Dono(0u, 0u, 0.0); }}
  if (!tem_s) {{ return Dono(bi, bi, 0.0); }}
  // ⚠️ `width <= 0` devolve o degrau a pique — um chamador sem escala de pixel não pode receber
  // uma mistura inventada.
  if (width <= 0.0) {{ return Dono(bi, si, 0.0); }}
  return Dono(bi, si, max(0.5 - (sv - bv) / (2.0 * width), 0.0));
}}
"
        ));
        Some(OwnersWgsl { source: s, consts })
    }

    /// ⭐ **Quantas constantes o [`Self::to_wgsl`] acrescenta ao vector**, sem o escrever.
    ///
    /// ⚠️ Ela existe para quem precisa de dimensionar o buffer **antes** de compor o texto. ⛔ Não
    /// é uma segunda resposta: o número sai da mesma aritmética, e há gate a ligar os dois.
    #[must_use]
    pub fn wgsl_const_count(&self) -> Option<usize> {
        let mut n = 0usize;
        for (_, campo) in &self.leaves {
            n += campo.tape.to_wgsl_named("x", 0)?.consts.len();
        }
        Some(n + self.leaves.len() * BOLA + 1)
    }
}
