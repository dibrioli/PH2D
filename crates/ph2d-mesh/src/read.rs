//! **Ler um ficheiro de malha do disco** — a porta única sobre os três importadores.
//!
//! # Por que ela vive aqui e não num módulo de app
//!
//! Esta função é `std::fs::read` mais um `match` sobre [`MeshFormat`], e **todo** o trabalho real
//! está em [`import_obj`], [`import_ply`] e [`import_stl`], que já são desta crate. Ela viveu
//! dentro da shell (`sculpt3d_import`) por inércia, e em 2026-09-11 um segundo consumidor apareceu:
//! o módulo de modelagem 3D, ao sair da shell para a crate dele, precisava de ler uma escultura.
//!
//! ⛔ **A alternativa era a segunda cópia**, e este repo tem lei escrita sobre isso: *uma lei
//! escrita em dois sítios ainda não é uma lei — só uma PORTA é.* Duas dispatches de extensão
//! divergem no dia em que um quarto formato entrar por uma delas.

use std::path::Path;

use crate::{ImportedPiece, MeshFormat, import_obj, import_ply, import_stl};

/// **Lê as peças de um ficheiro de malha** (`.obj`, `.ply`, `.stl`), pela extensão.
///
/// ⚠️ Um OBJ pode trazer **várias** peças nomeadas; PLY e STL trazem uma malha só — daí o `Vec`
/// com um elemento, e não duas assinaturas.
///
/// # Errors
/// Devolve a mensagem do erro quando o ficheiro não se lê, a extensão é desconhecida, ou o
/// conteúdo não faz sentido para o formato.
pub fn read_pieces(path: &Path) -> Result<Vec<ImportedPiece>, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let fmt = path
        .extension()
        .and_then(|e| e.to_str())
        .and_then(MeshFormat::from_extension)
        .ok_or_else(|| "unknown extension".to_string())?;
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(str::to_string);
    let mesh = match fmt {
        MeshFormat::Obj => {
            // ⚠️ `from_utf8_lossy` e não `from_utf8`: um OBJ é texto por definição, e um byte
            // estranho num comentário não é razão para recusar a geometria inteira. O parser
            // ignora o que não entende.
            return import_obj(&String::from_utf8_lossy(&bytes)).map_err(|e| e.to_string());
        }
        MeshFormat::Ply => import_ply(&bytes).map_err(|e| e.to_string())?,
        MeshFormat::Stl => import_stl(&bytes).map_err(|e| e.to_string())?,
    };
    Ok(vec![ImportedPiece { name, mesh }])
}

/// **O que este formato NÃO leva** — a frase que o app mostra antes de exportar.
///
/// ⚠️ A máscara nunca viaja em formato nenhum dos três, então ela está sempre na lista; a cor e a
/// separação em peças dependem do formato, e a resposta é do próprio [`MeshFormat`].
///
/// ⭐⭐⭐⭐ **E a TINTA FINA é a terceira espécie, com a pergunta partida em DUAS:**
/// *o formato leva?* (nunca, hoje — [`MeshFormat::keeps_fine_paint`]) **e** *esta
/// cena TEM alguma?*. A máscara não precisa da segunda metade porque é um canal
/// que toda peça tem; a tinta fina é **opcional** e a esmagadora maioria das
/// peças não a tem.
///
/// ⚠️⚠️ **Sem essa segunda metade o aviso soava SEMPRE**, e um aviso que soa
/// sempre é ruído que o artista aprende a ignorar — exactamente quando ele
/// passar a ser verdade. *É a mesma lei que a recusa do pen-down desta família
/// já paga: as metades NEGATIVAS são metade do valor.*
#[must_use]
pub fn lost_by(fmt: MeshFormat, has_fine_paint: bool) -> String {
    let mut lost = vec!["mask"];
    if !fmt.keeps_colour() {
        lost.push("colour");
    }
    if !fmt.keeps_pieces() {
        lost.push("pieces merged");
    }
    if has_fine_paint && !fmt.keeps_fine_paint() {
        lost.push("fine paint (mesh resolution only)");
    }
    format!("not carried: {}", lost.join(", "))
}
