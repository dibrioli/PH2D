//! ⭐⭐⭐⭐ **A GRADE DE LONGE — o raio atravessa o VAZIO por uma grade e toca a peça pela árvore.**
//!
//! # A pergunta que isto responde
//!
//! O dispositivo avalia a fita INTEIRA em todo passo de todo raio, e numa peça complexa esse é o
//! custo do quadro: medido (`docs/Render3d/03_o_plano.md`, *«a grade assada contra a árvore»*), o
//! nó de toro de `721` linhas custa `59,46 ms` pela árvore e `20,09` marchado numa grade — o passo
//! na grade custa `~7×` menos. ⛔ **Mas a grade SOZINHA estraga a forma** (normal a `10°` na
//! mediana a `128³`, e o nó desfeito a `64³`, que é o `res` máximo do MagicaCSG).
//!
//! ⇒ **as duas coisas, cada uma onde é certa**: longe da superfície o raio anda pelo LIMITE
//! INFERIOR que a grade garante, e perto dela avalia a fita EXACTA. A silhueta, o ponto de paragem
//! e a normal continuam a ser os da árvore — *a grade só decide quanto se pode andar sem olhar*.
//!
//! # ⭐ Porque o salto é SEGURO, e não aproximado
//!
//! A grade guarda `f` (o campo do documento) nos nós. O passo seguro da marcha é `s · f`, com
//! `s = 1/L` e `L` o tecto de `‖∇f‖` ([`ph2d_field_eval::safe_march_step`]). A interpolação
//! trilinear é uma média PESADA dos oito cantos, e a média pesada das distâncias de `p` aos cantos
//! é no máximo `(√3/2)·h` (`Σ tᵢ(1−tᵢ)h² ≤ 3h²/4`, pela desigualdade de Jensen) ⇒
//!
//! ```text
//! f(p) ≥ f̃(p) − L·(√3/2)·h      ⇒      s·f(p) ≥ s·f̃(p) − (√3/2)·h
//! ```
//!
//! ⇒ **andar `s·f̃(p) − 0,866·h` nunca passa da superfície**, qualquer que seja a peça. *Não é uma
//! folga medida: é uma desigualdade.* Fora da caixa da grade o salto é a distância à caixa, que é
//! menor que a distância à peça porque a caixa a contém.
//!
//! # ⭐ E a caixa RECORTA o raio antes de ele começar
//!
//! Um raio que não toca a caixa da peça não tem superfície nenhuma a achar, e a marcha de sempre
//! avaliava a fita até `t_max` para o descobrir. ⇒ o raio entra na caixa e sai dela — o `clip` que
//! a CPU já tem ([`ph2d_field_render`]), aqui com o mesmo porquê.
//!
//! # ⚠️ A grade é ASSADA NA PLACA, pelo mesmo texto de campo
//!
//! Assá-la na CPU custa `19,6 ms` a `64³` no nó de toro, **por edição** — e arrastar uma forma é
//! uma edição por quadro. Na placa ela é um despacho com o MESMO `field()` que a marcha avalia,
//! logo *a grade e a árvore são a mesma função em `f32`, avaliada nos mesmos bits*.
//!
//! ⚠️ **Ela mora no armazém das esculturas, depois delas.** Um armazém a mais no grupo `0` poria o
//! matcap em `9` contra o piso de `8` do WebGPU ([`crate::matcap::PISO_DO_WEBGPU`]).

/// Quantos `f32` o cabeçalho ocupa no vector das constantes — ver [`cabecalho`].
pub const CABECALHO: usize = 16;

/// Quantas células de folga a grade tem à volta da caixa da peça.
///
/// ⚠️ **Uma basta, e é para os arredondamentos:** o salto dentro da grade é um limite inferior em
/// todo o lado, e a folga só serve para que um ponto na parede da caixa da peça tenha cantos dos
/// dois lados. A distância à caixa trata do resto.
pub const FOLGA: u32 = 1;

/// Quantos saltos um raio pode dar antes de desistir — o tecto de uma travessia.
///
/// ⚠️ **Não é um orçamento de qualidade:** cada salto anda pelo menos [`Longe::perto`], logo uma
/// travessia da caixa inteira cabe em `√3 · lado/perto` saltos. Este número é a rede contra um
/// campo com `NaN`, que nenhuma desigualdade cobre.
pub const SALTOS_MAX: u32 = 4096;

/// ⭐ **A grade de longe de UM quadro** — a caixa da peça e a resolução pedida.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Longe {
    /// O canto mínimo da caixa que contém a peça.
    pub lo: [f32; 3],
    /// O canto máximo.
    pub hi: [f32; 3],
    /// Células no lado MAIOR da caixa. `0` = só o recorte da caixa, sem grade.
    pub res: u32,
    /// Abaixo deste limite (em células) o raio deixa a grade e avalia a árvore.
    ///
    /// ⚠️ É em CÉLULAS porque o erro da grade é em células: um limite em unidades de mundo seria
    /// grosso numa peça pequena e fino numa grande.
    pub perto: f32,
}

/// A geometria da grade que a [`Longe`] pede — derivada, nunca guardada.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grade {
    /// Quantos nós em cada eixo.
    pub dims: [u32; 3],
    /// O nó `(0, 0, 0)`, no mundo.
    pub origem: [f32; 3],
    /// A aresta da célula.
    pub celula: f32,
}

impl Longe {
    /// A grade que esta caixa pede — `None` quando a resolução é `0` ou a caixa é degenerada.
    #[must_use]
    pub fn grade(&self) -> Option<Grade> {
        if self.res == 0 {
            return None;
        }
        let ext = [
            self.hi[0] - self.lo[0],
            self.hi[1] - self.lo[1],
            self.hi[2] - self.lo[2],
        ];
        let maior = ext[0].max(ext[1]).max(ext[2]);
        if maior.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) || !maior.is_finite() {
            return None;
        }
        #[allow(clippy::cast_precision_loss)]
        let celula = maior / self.res as f32;
        #[allow(clippy::cast_precision_loss)]
        let folga = celula * FOLGA as f32;
        let mut dims = [0u32; 3];
        let mut origem = [0.0f32; 3];
        for a in 0..3 {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let n = (ext[a] / celula).ceil() as u32 + 1;
            dims[a] = n + 2 * FOLGA;
            origem[a] = self.lo[a] - folga;
        }
        Some(Grade {
            dims,
            origem,
            celula,
        })
    }

    /// Quantos `f32` a grade ocupa no armazém — `0` sem grade.
    #[must_use]
    pub fn valores(&self) -> u64 {
        self.grade().map_or(0, |g| {
            u64::from(g.dims[0]) * u64::from(g.dims[1]) * u64::from(g.dims[2])
        })
    }
}

/// ⭐ **O cabeçalho no `k`** — a ordem que o [`LEI`] lê, campo a campo.
///
/// | índice | conteúdo |
/// |---|---|
/// | `0..3` | canto mínimo da caixa |
/// | `3` | aresta da célula (`0` = sem grade) |
/// | `4..7` | canto máximo da caixa |
/// | `7` | onde a grade começa no `grades` (nos BITS do `f32`) |
/// | `8..11` | nós por eixo |
/// | `11` | o limite de perto, em unidades de mundo |
/// | `12..15` | o nó `(0,0,0)` |
///
/// ⚠️ **O deslocamento viaja nos bits e não no valor** — a razão escrita no
/// [`crate::sculpt::emit`]: acima de `2²⁴` um `f32` deixa de representar um inteiro.
#[must_use]
pub fn cabecalho(l: &Longe, deslocamento: u32) -> [f32; CABECALHO] {
    let mut h = [0.0f32; CABECALHO];
    h[0..3].copy_from_slice(&l.lo);
    h[4..7].copy_from_slice(&l.hi);
    h[7] = f32::from_bits(deslocamento);
    if let Some(g) = l.grade() {
        h[3] = g.celula;
        #[allow(clippy::cast_precision_loss)]
        {
            h[8] = g.dims[0] as f32;
            h[9] = g.dims[1] as f32;
            h[10] = g.dims[2] as f32;
        }
        h[11] = l.perto * g.celula;
        h[12..15].copy_from_slice(&g.origem);
    }
    h
}

/// ⭐⭐⭐ **A lei, em WGSL** — o recorte da caixa e o limite inferior de um salto.
///
/// ⚠️ `s.longe` é o índice do cabeçalho no `k` **mais um**; `0` = sem grade de longe, e aí nenhuma
/// destas funções é chamada.
pub(crate) const LEI: &str = r"
// ⭐ `vec2(entrada, saída)` do raio na caixa da peça; `entrada > saída` quando falha.
// ⚠️ Um eixo com a direcção a ZERO não restringe se a origem está dentro da laje dele e mata o raio
// se está fora — sem dividir por zero, que em WGSL não tem resultado garantido.
fn longe_caixa(r: Raio) -> vec2<f32> {
    let h = s.longe - 1u;
    let lo = vec3<f32>(k[h], k[h + 1u], k[h + 2u]);
    let hi = vec3<f32>(k[h + 4u], k[h + 5u], k[h + 6u]);
    var a = -3.0e38;
    var b = 3.0e38;
    for (var e: u32 = 0u; e < 3u; e = e + 1u) {
        let o = r.o[e];
        let d = r.d[e];
        if (abs(d) < 1.0e-30) {
            if (o < lo[e] || o > hi[e]) { return vec2<f32>(1.0, -1.0); }
        } else {
            let t1 = (lo[e] - o) / d;
            let t2 = (hi[e] - o) / d;
            a = max(a, min(t1, t2));
            b = min(b, max(t1, t2));
        }
    }
    return vec2<f32>(a, b);
}

// ⭐⭐⭐ **Quanto se pode andar sem olhar** — a desigualdade do topo do módulo.
fn longe_limite(p: vec3<f32>) -> f32 {
    let h = s.longe - 1u;
    let cel = k[h + 3u];
    if (cel <= 0.0) { return 0.0; }
    let dims = vec3<u32>(u32(k[h + 8u]), u32(k[h + 9u]), u32(k[h + 10u]));
    let origem = vec3<f32>(k[h + 12u], k[h + 13u], k[h + 14u]);
    let topo = origem + vec3<f32>(f32(dims.x - 1u), f32(dims.y - 1u), f32(dims.z - 1u)) * cel;
    let fora = length(max(max(origem - p, p - topo), vec3<f32>(0.0)));
    if (fora > 0.0) { return fora; }
    let off = bitcast<u32>(k[h + 7u]);
    return escultura_trilinear(dims, origem, cel, off, p) * s.step - 0.8660254 * cel;
}

// Há grade, ou só o recorte da caixa? (`res = 0` escreve a célula a `0`.)
fn longe_tem_grade() -> bool { return k[s.longe - 1u + 3u] > 0.0; }

// O limite abaixo do qual o raio deixa a grade — ver `Longe::perto`.
fn longe_perto() -> f32 { return k[s.longe - 1u + 11u]; }
";

/// ⭐⭐ **O despacho que ASSA a grade** — um nó por invocação, pelo `field()` da marcha.
///
/// ⚠️ **O `grades` é `read_write` só neste módulo**: ele escreve a região da grade de longe e lê a
/// das esculturas (quando a peça as tem), e as duas regiões são disjuntas por construção.
pub(crate) const ASSA: &str = r"
@compute @workgroup_size(4, 4, 4)
fn assa_longe(@builtin(global_invocation_id) g: vec3<u32>) {
    let h = s.longe - 1u;
    let dims = vec3<u32>(u32(k[h + 8u]), u32(k[h + 9u]), u32(k[h + 10u]));
    if (g.x >= dims.x || g.y >= dims.y || g.z >= dims.z) { return; }
    let cel = k[h + 3u];
    let origem = vec3<f32>(k[h + 12u], k[h + 13u], k[h + 14u]);
    let off = bitcast<u32>(k[h + 7u]);
    let p = origem + vec3<f32>(f32(g.x), f32(g.y), f32(g.z)) * cel;
    grades[off + g.x + g.y * dims.x + g.z * dims.x * dims.y] = field(p);
}
";

/// O [`crate::trace_wgsl::comum`] com o armazém das grades aberto à ESCRITA — só para o [`ASSA`].
///
/// # Panics
/// Se o texto do grupo `0` deixar de declarar o armazém como ele é declarado hoje — e é esse o
/// ponto: uma substituição que não casa é um no-op silencioso que desenharia sem grade nenhuma.
#[must_use]
pub(crate) fn comum_para_assar() -> String {
    let comum = crate::trace_wgsl::comum();
    let de = "var<storage, read> grades";
    assert_eq!(
        comum.matches(de).count(),
        1,
        "o grupo 0 tem de declarar o armazém das grades exactamente uma vez"
    );
    comum.replace(de, "var<storage, read_write> grades")
}

/// As entradas do grupo do despacho que assa: o uniforme, o `k` e o armazém aberto à escrita.
///
/// ⚠️ **Os números dos bindings são os do grupo `0`** — o texto é o mesmo [`comum_para_assar`], e
/// um número diferente aqui ligaria o `k` onde o shader lê o armazém.
#[must_use]
pub(crate) fn entradas() -> [wgpu::BindGroupLayoutEntry; 3] {
    use crate::trace::{armazem, uniforme};
    [uniforme(0), armazem(1, true), armazem(6, false)]
}

#[cfg(test)]
#[path = "longe_tests.rs"]
mod longe_tests;
