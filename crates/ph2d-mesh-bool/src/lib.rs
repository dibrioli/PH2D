//! **CORTAR uma peça com outra** — a fronteira entre a nossa malha e o motor de
//! booleana.
//!
//! # ⛔⛔⛔ A razão nº 1 de esta porta existir: o motor falha em SILÊNCIO
//!
//! Reproduzido nesta casa, fora do repo, com o motor cru:
//!
//! ```text
//! FECHADA   entrada status=NoError      → corte OK (16 V, 28 T)
//! ABERTA    entrada status=NotManifold
//! ABERTA    RESULTADO status=NoError  V=0  T=0   ← a armadilha
//! ```
//!
//! ⇒ com uma peça **aberta** o motor devolve malha **VAZIA** e o estado do
//! **resultado** diz *«sem erro»*. **Quem verificar o resultado lê sucesso e
//! entrega uma escultura APAGADA.** O sinal verdadeiro está no estado da
//! **ENTRADA**, e é por isso que [`corta`] o pergunta **antes** de operar.
//!
//! ⚠️ **E o motor «robusto» não resgata isto:** ele aceita *soup fechada*
//! (não-manifold, desconexa, com vazios) — não superfície com **bordo**. *É a
//! distinção entre «suja» e «aberta», e ela separa a promessa do que se mede.*
//!
//! # A superfície, e porque ela é pequena de propósito
//!
//! Cinco coisas: converter para lá, importar, **ler o estado da entrada**,
//! operar, converter de volta. Trocar o motor é reescrever este ficheiro.

use ph2d_mesh::{Face, Mesh};

/// ⭐⭐⭐ **A LIMPEZA DA COSTURA** — ver [`costura`].
#[path = "costura.rs"]
pub mod costura;
pub use costura::limpa_a_costura;

/// O que o corte faz com o volume da lâmina.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Op {
    /// Tirar da peça o volume da lâmina — o corte.
    Subtrair,
    /// Somar a lâmina à peça.
    Juntar,
    /// Ficar só com o que as duas partilham.
    Intersectar,
}

/// Porque é que o corte não aconteceu — **em voz alta, e nomeando o lado**.
///
/// ⚠️ **Cada variante é um facto sobre a ENTRADA, menos a última.** Um corte que
/// não pode acontecer tem de dizer **o que falta**, senão o artista lê a
/// ferramenta como partida — é a lei que a família de recusas desta casa já
/// aplica aos pincéis (`recusa::Entradas`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Recusa {
    /// A **peça** não encerra volume (tem bordo, ou não é orientável).
    PecaAberta,
    /// A **lâmina** não encerra volume.
    LaminaAberta,
    /// A peça ficou sem nada. ⚠️ É a única que é facto sobre a SAÍDA, e é
    /// recusa e não resultado: *um corte que apaga a peça inteira não é um
    /// corte*, e devolvê-lo como sucesso é o mesmo defeito que esta porta
    /// existe para impedir, um nível acima.
    ResultadoVazio,
}

impl Recusa {
    /// A frase que o artista lê — ela diz o que falta, nunca só que falhou.
    #[must_use]
    pub fn porque(self) -> &'static str {
        match self {
            Self::PecaAberta => {
                "a peca tem BORDO ABERTO e um corte precisa de volume fechado \
                 -- feche-a (o remesh tapa buracos) e repita"
            }
            Self::LaminaAberta => {
                "a forma do corte nao fechou -- o gesto foi degenerado (um \
                 risco, ou area zero); repita o desenho"
            }
            Self::ResultadoVazio => {
                "este corte apagaria a peca INTEIRA -- desfaca-o ou corte menos"
            }
        }
    }
}

/// **A porta.** Corta `peca` com `lamina`, ou recusa dizendo porquê.
///
/// ⚠️ **A ordem é a lei:** os dois estados de entrada são lidos **antes** de o
/// motor correr. Ver o cabeçalho — invertê-la devolve `Ok` sobre uma malha
/// vazia.
pub fn corta(peca: &Mesh, lamina: &Mesh, op: Op) -> Result<Mesh, Recusa> {
    use manifold_rust::manifold::Manifold;
    use manifold_rust::types::OpType;

    let a = Manifold::from_mesh_gl64(&para_o_motor(peca));
    if !fechada(&a) {
        return Err(Recusa::PecaAberta);
    }
    let b = Manifold::from_mesh_gl64(&para_o_motor(lamina));
    if !fechada(&b) {
        return Err(Recusa::LaminaAberta);
    }

    let saida = a.boolean(
        &b,
        match op {
            Op::Subtrair => OpType::Subtract,
            Op::Juntar => OpType::Add,
            Op::Intersectar => OpType::Intersect,
        },
    );
    let gl = saida.get_mesh_gl64(-1);
    if gl.tri_verts.is_empty() {
        return Err(Recusa::ResultadoVazio);
    }
    // ⭐⭐⭐ **A COSTURA é limpa AQUI, dentro da porta** (report do dono,
    // 2026-09-15: *«melhore a topologia das bordas do corte»*). Ver
    // [`costura`] — o motor emite vértices duplicados na curva de interseção, e
    // um triângulo de aspecto `2 573 809` não tem normal utilizável.
    //
    // ⚠️ **Ela precisa da PEÇA e é por isso que vive dentro do `corta`:** é da
    // entrada que sai *qual vértice é antigo* — a cerca que mantém intacta a
    // propriedade que decide a arquitectura desta linha (*longe do corte, nem
    // um bit*).
    Ok(limpa_a_costura(&do_motor(&gl), peca))
}

/// **O corte SEM a limpeza da costura** — o lado *antes* das réguas dela.
///
/// ⛔ **Só existe para os gates**, e é a única maneira honesta de eles serem uma
/// AFIRMAÇÃO em vez de um número solto: *uma régua sobre a saída curada não diz
/// que a cura fez alguma coisa.* ⚠️ Ela duplica quatro linhas do [`corta`] de
/// propósito — chamá-lo e «des-limpar» é impossível, e um parâmetro no caminho
/// do produto seria um interruptor que alguém pode deixar no sítio errado.
#[cfg(test)]
pub(crate) fn corta_cru(peca: &Mesh, lamina: &Mesh, op: Op) -> Result<Mesh, Recusa> {
    use manifold_rust::manifold::Manifold;
    use manifold_rust::types::OpType;

    let a = Manifold::from_mesh_gl64(&para_o_motor(peca));
    if !fechada(&a) {
        return Err(Recusa::PecaAberta);
    }
    let b = Manifold::from_mesh_gl64(&para_o_motor(lamina));
    if !fechada(&b) {
        return Err(Recusa::LaminaAberta);
    }
    let gl = a
        .boolean(
            &b,
            match op {
                Op::Subtrair => OpType::Subtract,
                Op::Juntar => OpType::Add,
                Op::Intersectar => OpType::Intersect,
            },
        )
        .get_mesh_gl64(-1);
    if gl.tri_verts.is_empty() {
        return Err(Recusa::ResultadoVazio);
    }
    Ok(do_motor(&gl))
}

/// O estado da ENTRADA — a pergunta que o cabeçalho manda fazer.
fn fechada(m: &manifold_rust::manifold::Manifold) -> bool {
    m.status() == manifold_rust::types::Error::NoError
}

/// A nossa malha na representação do motor.
///
/// ⚠️ **Triangula pela porta da `Face`** ([`Face::triangles`]) e não por uma
/// segunda aritmética aqui: um quad partido de duas maneiras dá duas superfícies
/// diferentes, e a casa já tem UMA resposta a essa pergunta.
fn para_o_motor(m: &Mesh) -> manifold_rust::types::MeshGL64 {
    let mut tris = Vec::new();
    for f in m.faces() {
        f.triangles(&mut tris);
    }
    manifold_rust::types::MeshGL64 {
        num_prop: 3,
        vert_properties: m
            .positions()
            .iter()
            .flat_map(|p| [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])])
            .collect(),
        tri_verts: tris
            .iter()
            .flat_map(|t| [u64::from(t[0]), u64::from(t[1]), u64::from(t[2])])
            .collect(),
        ..Default::default()
    }
}

/// De volta à nossa malha.
///
/// ⚠️ **A viagem `f32 → f64 → f32` é exacta para quem não foi tocado** — é isso
/// que faz a preservação bit-a-bit longe do corte ser observável, e há gate.
fn do_motor(gl: &manifold_rust::types::MeshGL64) -> Mesh {
    let pos: Vec<[f32; 3]> = gl
        .vert_properties
        .as_chunks::<3>()
        .0
        .iter()
        .map(|c| [c[0] as f32, c[1] as f32, c[2] as f32])
        .collect();
    let faces: Vec<Face> = gl
        .tri_verts
        .as_chunks::<3>()
        .0
        .iter()
        .map(|t| Face::tri(t[0] as u32, t[1] as u32, t[2] as u32))
        .collect();
    Mesh::from_parts(pos, faces).expect("o motor devolve triangulos indexados")
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
