//! As edições das faixas de som (a linha do tempo do Pixel): cada uma é UM `Op` = um passo do
//! desfazer (`DocOp::SetAudio`, cujo inverso é a lista de antes).

use ph2d_pixel_core::{DocOp, FaixaDeAudio, Op, Sprite};

/// O que a linha de uma faixa pede.
#[derive(Debug, Clone, PartialEq)]
pub enum Edicao {
    /// **Add Audio…**: uma faixa nova no fim, a começar no quadro 0, ganho 1, a tocar.
    Adiciona(String),
    Mudo(usize, bool),
    /// Arrastar a linha: o novo `inicio_ms`.
    Move(usize, i32),
    Ganho(usize, f32),
    Remove(usize),
}

fn com(s: &Sprite, f: impl FnOnce(&mut Vec<FaixaDeAudio>) -> bool) -> Option<Op> {
    let mut v = s.audio().to_vec();
    (f(&mut v) && v != s.audio()).then(|| Op::Doc(Box::new(DocOp::SetAudio(v))))
}

/// O `Op` de `e` sobre o sprite; `None` = nada muda (índice fora, valor igual, asset vazio).
#[must_use]
pub fn op(s: &Sprite, e: &Edicao) -> Option<Op> {
    com(s, |v| match e {
        Edicao::Adiciona(asset) if !asset.is_empty() => {
            v.push(FaixaDeAudio {
                asset: asset.clone(),
                inicio_ms: 0,
                ganho: 1.0,
                mudo: false,
            });
            true
        }
        Edicao::Adiciona(_) => false,
        Edicao::Mudo(i, m) => v.get_mut(*i).map(|f| f.mudo = *m).is_some(),
        Edicao::Move(i, ms) => v.get_mut(*i).map(|f| f.inicio_ms = *ms).is_some(),
        Edicao::Ganho(i, g) => {
            g.is_finite() && *g >= 0.0 && v.get_mut(*i).map(|f| f.ganho = *g).is_some()
        }
        Edicao::Remove(i) => (*i < v.len()).then(|| v.remove(*i)).is_some(),
    })
}

/// O asset de um ficheiro escolhido: relativo à pasta do projeto quando está dentro dela (o
/// projeto leva a referência e muda de máquina com ela); fora dela, ou sem projeto, o caminho.
#[must_use]
pub fn asset_de(ficheiro: &std::path::Path, projeto: Option<&std::path::Path>) -> String {
    projeto
        .and_then(|p| ficheiro.strip_prefix(p).ok())
        .unwrap_or(ficheiro)
        .to_string_lossy()
        .replace('\\', "/")
}
