//! Import de OBJ — o mínimo para haver o que esculpir.
//!
//! Adaptado de `reference/sculptgl/src/files/ImportOBJ.js`, MIT — ver
//! `LICENSES/sculptgl-MIT.txt`.
//!
//! **Escopo, dito na porta:** geometria e cor por vértice. UV, materiais,
//! grupos e suavização são **ignorados em silêncio** — não por descuido: a W1
//! existe para haver uma malha na tela, e um importador que finge entender
//! `usemtl` cria a expectativa de que o material sobreviveu à viagem. Quando
//! houver dono para UV (a doação, W3), ele entra aqui com gate próprio.
//!
//! ⚠️ **Quads são PRESERVADOS.** Triangular na porta de entrada jogaria fora
//! exatamente a topologia que a multiresolução (`docs/3D/04.3`) precisa — o
//! `Face` guarda tri e quad justamente para isso.
//!
//! ⚠️ **N-gons acima de 4 viram leque de TRIÂNGULOS, e o motivo que esta nota
//! dava era falso.** Ela dizia *"porque não há representação para eles"* — há: o
//! `Face` guarda quad, e o `ImportOBJ.js:65-98` **quadrangula** (`nbPrim =
//! ceil(nbVerts/2) - 1`, emitindo quads e no máximo um triângulo). A escolha do
//! leque fica, agora com a razão verdadeira: ela é a mais simples que preserva a
//! superfície, e o quad só paga por si quando a subdivisão existir para
//! consumi-lo. **O gatilho é a W6**, e portar a quadrangulação é lá.
//!
//! ⚠️ **Uma linha `v` malformada RECUSA o arquivo, e isto é uma DIVERGÊNCIA
//! deliberada da referência.** O `ImportOBJ.js:44` empurra `parseFloat` de cada
//! token e faz `++nbVertices` **incondicionalmente**: no original o índice nunca
//! desliza, mas o preço é um vértice `NaN` dentro da malha. `NaN` em posição
//! atravessa o `from_parts` (que valida ÍNDICE, não coordenada), envenena o
//! octree e a AABB, e some das comparações — o defeito reaparece a três sistemas
//! de distância, sem nada apontando para o arquivo. Recusar nomeia a linha.
//!
//! ⚠️ **E a nota anterior desta casa dizia que o original "descarta o arquivo".
//! É falso** — foi lido inteiro para escrever este parágrafo. É exatamente o
//! falso-positivo de 55% que o `docs/3D/03.7` mede: fidelidade não se afirma por
//! leitura.

use crate::face::Face;
use crate::mesh::{Mesh, MeshError};

/// O que pode dar errado lendo um OBJ.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObjError {
    /// `f` referenciou um vértice que o arquivo não declarou (índice 0, fora de
    /// alcance, ou negativo além do começo).
    BadFaceIndex { line: usize, token: String },
    /// Uma linha `v` que não descreve um vértice: menos de três números, um
    /// token que não é número, ou uma coordenada não-finita.
    BadVertex { line: usize, text: String },
    /// A geometria carregou, mas não forma uma malha (ver `MeshError`).
    Mesh(MeshError),
}

impl core::fmt::Display for ObjError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BadFaceIndex { line, token } => {
                write!(f, "linha {line}: índice de face inválido {token:?}")
            }
            Self::BadVertex { line, text } => {
                write!(f, "linha {line}: vértice malformado {text:?}")
            }
            Self::Mesh(e) => write!(f, "{e}"),
        }
    }
}

impl core::error::Error for ObjError {}

/// Lê um OBJ de texto.
///
/// **Uma peça que veio de um arquivo** — a malha e o nome que o `o` lhe deu.
///
/// ⚠️ O nome viaja porque um arquivo de personagem diz `o cabeca` / `o corpo`, e
/// jogá-lo fora aqui obrigaria o artista a redescobrir qual peça é qual olhando
/// a silhueta. Ele é `Option` porque um arquivo sem `o` não nomeia nada — e
/// inventar um nome seria pior que não ter (ninguém saberia que foi inventado).
pub struct ImportedPiece {
    pub name: Option<String>,
    pub mesh: Mesh,
}

/// Devolve **uma peça por `o`** — as malhas já construídas (normais, adjacência
/// e octree derivados), e a cor por vértice só materializada se o arquivo de
/// fato trouxer cor (a preguiça dos planos, honrada desde a porta de entrada).
///
/// ⚠️ **Um arquivo multi-objeto vira VÁRIAS peças, e isto é a dívida que a W8.1
/// nomeou.** Até ela existir, "a cena" e "a malha" eram a mesma coisa e um
/// arquivo com cabeça, corpo e olhos entrava como um bloco só — impossível de
/// posar, esconder ou apagar em separado. Agora a cena é uma LISTA, e a
/// tradução honesta de `o` é *uma peça*.
///
/// ⚠️ **O pool de vértices é do ARQUIVO, não do objeto** — os índices de `f` são
/// globais em OBJ e podem apontar para vértices declarados antes do `o`. Cada
/// peça é COMPACTADA no fim (só os vértices que ela usa, com os índices
/// remapeados); ler os índices como se fossem locais é o defeito clássico deste
/// formato, e ele produz geometria embaralhada em vez de um erro.
///
/// ⚠️ **Um arquivo SEM `o` devolve exatamente uma peça** — é o comportamento de
/// antes desta wave, e há gate pinando que ele não se moveu.
///
/// # Errors
/// Linha `v` malformada, índice de `f` fora de alcance, ou geometria que não
/// forma malha.
pub fn import_obj(text: &str) -> Result<Vec<ImportedPiece>, ObjError> {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 3]> = Vec::new();
    let mut has_colors = false;
    let mut idx: Vec<u32> = Vec::new();
    // A peça em construção: o nome do `o` mais recente e as faces desde ele.
    let mut name: Option<String> = None;
    let mut faces: Vec<Face> = Vec::new();
    let mut pieces: Vec<(Option<String>, Vec<Face>)> = Vec::new();

    // ⚠️ **O BOM é comido na porta.** Ele é marcador de codificação — todo editor
    // de Windows o escreve —, não geometria: sem isto o primeiro token vira
    // `"\u{feff}v"`, a linha cai no braço desconhecido, e o arquivo INTEIRO
    // carrega com todo índice deslocado de um.
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);

    for (lineno, raw) in text.lines().enumerate() {
        let line = raw.trim();
        let mut it = line.split_ascii_whitespace();
        match it.next() {
            Some("v") => {
                let (n, count) = parse_vertex(it).ok_or_else(|| ObjError::BadVertex {
                    line: lineno + 1,
                    text: line.to_string(),
                })?;
                positions.push([n[0], n[1], n[2]]);
                if count >= 6 {
                    has_colors = true;
                    colors.push([n[3], n[4], n[5]]);
                } else {
                    colors.push(crate::mesh::DEFAULT_COLOR);
                }
            }
            Some("f") => {
                idx.clear();
                for token in it {
                    // `a`, `a/b`, `a//c`, `a/b/c` — só o primeiro campo é a
                    // posição, e é o único que esta wave consome.
                    let first = token.split('/').next().unwrap_or("");
                    let Ok(raw_i) = first.parse::<i64>() else {
                        return Err(ObjError::BadFaceIndex {
                            line: lineno + 1,
                            token: token.to_string(),
                        });
                    };
                    // OBJ é 1-based, e negativo conta de trás para a frente a
                    // partir do que já foi declarado.
                    let resolved = if raw_i > 0 {
                        raw_i - 1
                    } else if raw_i < 0 {
                        positions.len() as i64 + raw_i
                    } else {
                        -1
                    };
                    if resolved < 0 || resolved >= positions.len() as i64 {
                        return Err(ObjError::BadFaceIndex {
                            line: lineno + 1,
                            token: token.to_string(),
                        });
                    }
                    idx.push(resolved as u32);
                }
                push_face(&idx, &mut faces);
            }
            Some("o") => {
                // ⚠️ **Um `o` fecha a peça anterior SÓ se ela tiver faces.** Um
                // arquivo que abre com `o corpo` (o caso normal) não pode
                // produzir uma peça vazia antes dele — e uma malha sem face é
                // recusada pelo `from_parts`, então o arquivo inteiro morreria
                // por causa de um cabeçalho.
                if !faces.is_empty() {
                    pieces.push((name.take(), core::mem::take(&mut faces)));
                }
                name = Some(it.collect::<Vec<_>>().join(" "));
            }
            _ => {}
        }
    }
    if !faces.is_empty() || pieces.is_empty() {
        pieces.push((name, faces));
    }

    let mut out = Vec::with_capacity(pieces.len());
    for (name, faces) in pieces {
        let (verts, cols, faces) = compact(&positions, &colors, &faces);
        let mut mesh = Mesh::from_parts(verts, faces).map_err(ObjError::Mesh)?;
        if has_colors {
            mesh.colors_mut().copy_from_slice(&cols);
        }
        out.push(ImportedPiece { name, mesh });
    }
    Ok(out)
}

/// **Só os vértices que estas faces usam**, com os índices remapeados.
///
/// ⚠️ Ela existe porque o pool é do ARQUIVO: sem a compactação, cada peça de um
/// arquivo de dez objetos carregaria os vértices dos outros nove — malha
/// inteira em memória por peça, com uma nuvem de vértices órfãos que o
/// `from_parts` aceita, o octree indexa e a caixa da câmera enxerga (a peça
/// nasceria enquadrada no arquivo inteiro em vez de nela mesma).
fn compact(
    positions: &[[f32; 3]],
    colors: &[[f32; 3]],
    faces: &[Face],
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<Face>) {
    let mut remap = vec![u32::MAX; positions.len()];
    let (mut verts, mut cols) = (Vec::new(), Vec::new());
    let mut out = Vec::with_capacity(faces.len());
    for f in faces {
        let n = f.verts().len();
        let mut mapped = f.0;
        // ⚠️ **Só os `n` primeiros slots** — o 4º de um triângulo é a sentinela
        // `TRI` (`u32::MAX`), e remapeá-la a transformaria num índice de vértice
        // que a malha não tem. É o mesmo motivo por que o `verts()` existe.
        for slot in &mut mapped[..n] {
            let old = *slot as usize;
            if remap[old] == u32::MAX {
                remap[old] = u32::try_from(verts.len()).unwrap_or(u32::MAX);
                verts.push(positions[old]);
                cols.push(colors[old]);
            }
            *slot = remap[old];
        }
        out.push(Face(mapped));
    }
    (verts, cols, out)
}

/// Os números de uma linha `v`, ou `None` se ela não descreve um vértice.
///
/// Devolve os SEIS primeiros (posição + cor) e quantos havia — sem alocar, que
/// é o que importa numa malha de milhões de vértices.
///
/// ⚠️ **Um token que não é número RECUSA a linha; ele não é pulado.** O
/// `filter_map(...ok())` anterior descartava o inconversível em silêncio, e as
/// duas consequências eram diferentes conforme quantos números sobravam:
/// sobrando menos de três, a linha era descartada e **todo índice seguinte
/// deslizava**; sobrando três ou mais, o vértice FICAVA com as coordenadas
/// erradas (`v 1,0 0 0 1.0` virava `(0, 0, 1)`). Um arquivo assim carregava sem
/// um aviso, com a geometria embaralhada.
///
/// ⚠️ **O `#` corta a linha, e é o que separa isto de uma cura ingênua.** Um
/// comentário de fim de linha (`v 1 0 0 1 0 0 # vermelho`) é OBJ legal, e uma
/// regra do tipo *"todo token tem de ser número"* o recusaria — trocando um
/// defeito por outro, em arquivos que hoje funcionam.
///
/// ⚠️ **Não-finito também recusa.** `inf` sobrevive ao `parse` e é **visível à
/// AABB** (ao contrário do `NaN`, que some nas comparações): um só deles
/// envenena o `Aabb::longest_edge`, que é de onde saem os raios de pincel
/// default — o pincel inteiro passa a medir infinito, e a causa está a três
/// sistemas de distância.
fn parse_vertex<'a>(tokens: impl Iterator<Item = &'a str>) -> Option<([f32; 6], usize)> {
    let mut out = [0.0f32; 6];
    let mut count = 0usize;
    for t in tokens {
        if t.starts_with('#') {
            break;
        }
        let v: f32 = t.parse().ok()?;
        if !v.is_finite() {
            return None;
        }
        if count < out.len() {
            out[count] = v;
        }
        count += 1;
    }
    (count >= 3).then_some((out, count))
}

/// Um polígono do arquivo vira 1 face (tri/quad) ou um leque de triângulos.
///
/// ⚠️ **Índice repetido descarta a face** (`ImportOBJ.js:88-92`, que faz o
/// mesmo). `f 1 1 2` tem área zero **por construção** — não é o caso limite de
/// uma malha achatada, é uma face que o arquivo declarou impossível —, e aceitá-la
/// era criar de graça exatamente a face degenerada cujo voto o `normals.rs`
/// precisou aprender a recusar.
fn push_face(idx: &[u32], faces: &mut Vec<Face>) {
    if has_repeat(idx) {
        return;
    }
    match idx.len() {
        0..=2 => {} // linha ou ponto: não é superfície
        3 => faces.push(Face::tri(idx[0], idx[1], idx[2])),
        4 => faces.push(Face::quad(idx[0], idx[1], idx[2], idx[3])),
        _ => {
            for k in 1..idx.len() - 1 {
                faces.push(Face::tri(idx[0], idx[k], idx[k + 1]));
            }
        }
    }
}

/// Algum índice se repete? Um polígono com um vértice citado duas vezes não é
/// uma superfície, seja tri, quad ou n-gon — a checagem é sobre a LISTA, e não
/// um caso especial por tamanho.
fn has_repeat(idx: &[u32]) -> bool {
    idx.iter()
        .enumerate()
        .any(|(i, a)| idx[i + 1..].contains(a))
}

#[cfg(test)]
#[path = "obj_tests.rs"]
mod tests;
