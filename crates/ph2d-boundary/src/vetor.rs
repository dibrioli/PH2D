//! A aritmética de vectores desta crate.
//!
//! ⚠️ **Ela repete a da [`ph2d_pose`] de propósito** — ver o cabeçalho do
//! `Cargo.toml`: as duas são clean-room de **especs diferentes**, e o que lá
//! parece igual traz guardas que são **lei daquela espec**. *Somar três `f32` é
//! vocabulário; a guarda em cima deles é que é a lei.*

/// Um ponto ou um vector em espaço de objecto.
pub type V3 = [f32; 3];

pub fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn escalar(a: V3, k: f32) -> V3 {
    [a[0] * k, a[1] * k, a[2] * k]
}

pub fn ponto(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn cruz(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub fn comprimento(a: V3) -> f32 {
    ponto(a, a).sqrt()
}

pub fn distancia(a: V3, b: V3) -> f32 {
    comprimento(sub(a, b))
}

/// A distância AO QUADRADO — ⚠️ e ela tem um consumidor com nome: o raio que
/// limita a busca da âncora compara **quadrados** (§5.1), e tirar a raiz ali
/// seria somar um arredondamento a uma comparação que decide quem entra na
/// fronteira.
pub fn distancia2(a: V3, b: V3) -> f32 {
    let d = sub(a, b);
    ponto(d, d)
}

/// Normaliza, ou `None` quando o vector não descreve direcção nenhuma.
///
/// ⚠️ **O piso é sobre o comprimento ao QUADRADO e é generoso de propósito:** a
/// pergunta não é *«quanto erro há?»* e sim *«existe direcção aqui?»*.
pub fn normalizar(v: V3) -> Option<V3> {
    let sq = ponto(v, v);
    if !sq.is_finite() || sq <= 1e-20 {
        return None;
    }
    Some(escalar(v, sq.sqrt().recip()))
}

/// ⭐ **Rodar `p` em torno do eixo `eixo` (unitário) que passa por `centro`**, a
/// fórmula de Rodrigues.
///
/// ⚠️ **Os dois modos que rodam pedem exactamente isto e nada mais** (§10.1 e
/// §10.5): o `BEND` com um centro **por coluna** e o `TWIST` com um centro
/// partilhado. *Escrever duas rotações, uma «à volta de um ponto» e outra «à
/// volta de um eixo», seria a mesma lei em dois sítios.*
pub fn rodar(p: V3, centro: V3, eixo: V3, angulo: f32) -> V3 {
    let r = sub(p, centro);
    let (s, c) = angulo.sin_cos();
    // r·cos + (k × r)·sin + k(k·r)(1 − cos)
    let termo = add(
        add(escalar(r, c), escalar(cruz(eixo, r), s)),
        escalar(eixo, ponto(eixo, r) * (1.0 - c)),
    );
    add(centro, termo)
}
