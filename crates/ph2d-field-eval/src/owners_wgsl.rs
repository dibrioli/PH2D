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
//!
//! # ⭐⭐⭐⭐ As folhas viajam como DADOS, e o texto é UM só (2026-10-01)
//!
//! Report do dono: *«ao acrescentar um box ele demora uns 5 segundos para aparecer … tb 5 seg para
//! mudar de cor»*. A lei COMPILADA ([`Owners::compilada`]) escrevia uma fita por folha no texto do
//! pintor ⇒ toda forma nova, e a primeira cor diferente (a passagem do stub de uma folha para a lei
//! inteira), era um texto novo e uma compilação do driver: `1`–`3,8 s` por entrada do pintor,
//! medido pelo `PH2D_PIPELINE_LOG`. ⇒ o caminho de omissão é a lei INTERPRETADA
//! ([`Owners::interpretada`]): cada folha vai em bytecode para o `k`, e o texto
//! ([`texto_interpretado`]) não depende da peça — nem de ter donos ([`sem_donos`]).
//!
//! ⭐ **O preço por quadro está medido e é ruído:** `1,4`–`4,5 ms` a `1280×720` com e sem a lei,
//! de 2 a 8 folhas (`diag_o_preco_do_dono_por_quadro`) — a bola à frente deixa `~1` folha por
//! pixel, e uma folha do catálogo é `26`–`525` operações interpretadas, uma vez por ponto e nunca
//! por passo de marcha. ⚠️ A lei compilada FICA como rede para uma folha que não cabe no tecto de
//! registos — certa, e lenta de compilar.

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
        self.interpretada(const_base)
            .or_else(|| self.compilada(const_base))
    }

    /// ⛔ **A lei com as fitas COMPILADAS no texto** — o caminho até 2026-10-01, e hoje só a rede
    /// do [`Self::to_wgsl`] para uma folha que não cabe no interpretador
    /// ([`REGISTOS_DO_INTERPRETADOR`]). ⚠️ O texto dela muda com cada folha, e é isso que custava
    /// os segundos — ver o cabeçalho deste módulo.
    #[must_use]
    pub fn compilada(&self, const_base: usize) -> Option<OwnersWgsl> {
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

    /// ⭐ **Quantas constantes o [`Self::to_wgsl`] acrescenta ao vector**, sem o devolver.
    ///
    /// ⛔ Não é uma segunda resposta: ela CHAMA a porta. A redacção anterior repetia a aritmética
    /// do emissor compilado e dizia *«há gate a ligar os dois»* — o gate não existia, e no dia em
    /// que o emissor mudou de formato a conta teria ficado para trás em silêncio.
    #[must_use]
    pub fn wgsl_const_count(&self) -> Option<usize> {
        self.to_wgsl(0).map(|w| w.consts.len())
    }

    /// ⭐⭐⭐⭐ **A LEI COM AS FOLHAS INTERPRETADAS** — o texto é [`texto_interpretado`], o MESMO para
    /// toda peça, e as folhas viajam como DADOS no `k`. `None` quando alguma folha não tem fita ou
    /// precisa de mais do que [`REGISTOS_DO_INTERPRETADOR`] registos.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn interpretada(&self, const_base: usize) -> Option<OwnersWgsl> {
        let n = self.leaves.len();
        let cabeca = 2 + n * FOLHA;
        let mut codigo: Vec<f32> = Vec::new();
        let mut folhas: Vec<f32> = Vec::with_capacity(n * FOLHA);
        for (b, campo) in &self.leaves {
            let bc = campo.tape_bytecode()?;
            if bc.registos > REGISTOS_DO_INTERPRETADOR {
                return None;
            }
            let (&raiz, palavras) = bc.palavras.split_last()?;
            let ini = cabeca + codigo.len();
            codigo.extend(crate::interp::em_floats(palavras));
            let fim = cabeca + codigo.len();
            folhas.extend_from_slice(&[
                b.center[0],
                b.center[1],
                b.center[2],
                b.radius,
                ini as f32,
                fim as f32,
                raiz as f32,
            ]);
        }
        let mut consts = Vec::with_capacity(cabeca + codigo.len() + 1);
        consts.push(n as f32);
        consts.push(self.margin);
        consts.extend(folhas);
        consts.extend(codigo);
        fecha_o_bloco(consts, const_base)
    }
}

/// ⭐⭐⭐ **A LEI DE UMA PEÇA SEM DONOS** — o mesmo texto, com ZERO folhas no bloco.
///
/// ⚠️⚠️ **Ela existe para o texto não mudar quando a peça GANHA donos** (report do dono,
/// 2026-10-01: *«5 segundos para mudar de cor»*): a primeira cor diferente era a passagem do
/// stub de uma folha para a lei inteira, e um texto novo é uma compilação nova do pintor. Com zero
/// folhas o `dono_mix` devolve `(0, 0, 0)` — a mesma resposta do stub, logo a mesma imagem.
#[must_use]
pub fn sem_donos(const_base: usize) -> Option<OwnersWgsl> {
    fecha_o_bloco(vec![0.0, 0.0], const_base)
}

/// O bloco acaba com a ORIGEM dele — ver [`texto_interpretado`]. `None` se ela não for exacta em
/// `f32` (um `k` de mais de `2²⁴` floats).
#[allow(clippy::cast_precision_loss)]
fn fecha_o_bloco(mut consts: Vec<f32>, const_base: usize) -> Option<OwnersWgsl> {
    if const_base + consts.len() >= 1 << 24 {
        return None;
    }
    consts.push(const_base as f32);
    Some(OwnersWgsl {
        source: texto_interpretado(),
        consts,
    })
}

/// ⭐⭐⭐ **O TECTO DE REGISTOS do interpretador** — o tamanho do vector privado de cada thread.
///
/// ⚠️ **Um recurso, e medido:** é o rascunho POR THREAD do pintor (`80 × 4 = 320` bytes). O
/// catálogo inteiro cabe — o pior é a engrenagem, com `75` (censo
/// `toda_forma_do_catalogo_cabe_no_interpretador`, no `ph2d-app-field3d`). Uma folha acima dele
/// (um contorno puxado muito comprido) não é recusada: a peça cai na lei COMPILADA, que é certa e
/// lenta de compilar.
pub const REGISTOS_DO_INTERPRETADOR: usize = 80;

/// Quantos floats a cabeça de uma folha ocupa: a bola (4), o início, o fim e a raiz.
const FOLHA: usize = 7;

/// ⭐⭐⭐⭐ **O TEXTO DA LEI INTERPRETADA — um só, para toda peça.**
///
/// # O bloco no `k`, e porque a ORIGEM vai no FIM
///
/// ```text
/// [ n, margem, (cx, cy, cz, raio, ini, fim, raiz) × n, código das folhas… , origem ]
///                                                                             ↑ k[len − 1]
/// ```
///
/// ⚠️ A origem do bloco é o tamanho do vector da peça, que muda com cada forma — escrita no TEXTO
/// faria o texto mudar outra vez. ⇒ ela é o ÚLTIMO elemento do `k`, e o `arrayLength` encontra-a.
/// ⛔ **O contrato é de quem concatena:** o bloco tem de ser o último a entrar no vector
/// (`ph2d_field_gpu::trace_marcha_com` escreve-o assim, e o arnês de paridade também).
#[must_use]
pub fn texto_interpretado() -> String {
    let k = crate::wgsl::CONSTS;
    let interp =
        crate::interp::interpretador_em_k_wgsl("dono_interpreta", REGISTOS_DO_INTERPRETADOR);
    format!(
        r"
{interp}
fn dono_base() -> u32 {{ return u32({k}[arrayLength(&{k}) - 1u]); }}

fn dono_campo(i: u32, p: vec3<f32>) -> f32 {{
  let b = dono_base();
  let o = b + 2u + i * {FOLHA}u + 4u;
  return dono_interpreta(p, b + u32({k}[o]), b + u32({k}[o + 1u]), u32({k}[o + 2u]));
}}

fn dono_bola(i: u32) -> vec4<f32> {{
  let o = dono_base() + 2u + i * {FOLHA}u;
  return vec4<f32>({k}[o], {k}[o + 1u], {k}[o + 2u], {k}[o + 3u]);
}}

fn dono_dentro(i: u32, p: vec3<f32>, margem: f32) -> bool {{
  let b = dono_bola(i);
  let d = p - b.xyz;
  let r = b.w + margem;
  return dot(d, d) <= r * r;
}}

struct Dono {{ a: u32, b: u32, t: f32 }};

// ⭐⭐⭐ A `Owners::mix_at` da CPU, ramo a ramo — incluindo o DESEMPATE (ganha o índice MENOR).
fn dono_mix(p: vec3<f32>, width: f32) -> Dono {{
  let base = dono_base();
  let folhas = u32({k}[base]);
  if (folhas == 0u) {{ return Dono(0u, 0u, 0.0); }}
  let margem = max({k}[base + 1u], width);
  var filtra = false;
  if (folhas > 1u) {{
    for (var i = 0u; i < folhas; i = i + 1u) {{
      if (dono_dentro(i, p, margem)) {{ filtra = true; break; }}
    }}
  }}
  var tem_b = false; var bv = 0.0; var bi = 0u;
  var tem_s = false; var sv = 0.0; var si = 0u;
  for (var i = 0u; i < folhas; i = i + 1u) {{
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
  if (width <= 0.0) {{ return Dono(bi, si, 0.0); }}
  return Dono(bi, si, max(0.5 - (sv - bv) / (2.0 * width), 0.0));
}}
"
    )
}
