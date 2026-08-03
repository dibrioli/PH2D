//! **A PORTA DE SAÍDA no app** — Ctrl+Shift+E escreve a cena num arquivo.
//!
//! Irmão do [`super::sculpt3d_import`], e o par dele: a W8.4 deu a entrada, a
//! W8.3 deu o documento, e sem esta a escultura **entra, salva e não sai**. Um
//! `.ph2d` só abre aqui; levar a malha ao Blender ou a uma impressora é o que
//! torna o módulo parte de um fluxo.
//!
//! ⚠️ **A EXTENSÃO decide o formato**, e o diálogo não tem um segundo seletor —
//! ver [`MeshFormat::from_extension`]. Duas portas para *"que formato é este?"*
//! divergem no primeiro `retrato.obj` salvo com "STL" escolhido ao lado.
//!
//! ⚠️ **O que o formato NÃO carrega é DITO no toast**, e a frase sai da mesma
//! tabela que o escritor consulta ([`MeshFormat::keeps_colour`] /
//! [`keeps_pieces`](MeshFormat::keeps_pieces)). Exportar em STL e perder a
//! pintura em silêncio seria a resposta errada com a confiança da certa — e a
//! MÁSCARA não sobrevive a nenhum dos três, então ela é dita sempre.

use ph2d_mesh::{ExportPiece, MeshFormat};

use super::Sculpt3dScene;

impl Sculpt3dScene {
    /// A cena inteira pronta para escrever — o nível **VIVO** de cada peça.
    ///
    /// ⚠️ **O nível vivo, não a base nem o topo:** o que está na tela é o que o
    /// artista acabou de julgar. Exportar a base entregaria um bloco liso a quem
    /// esculpiu detalhe; exportar o topo entregaria milhões de triângulos a quem
    /// desceu de propósito para trabalhar grosso.
    pub(crate) fn export_pieces(&self) -> Vec<ExportPiece<'_>> {
        self.objects
            .iter()
            .map(|o| ExportPiece {
                name: None,
                mesh: o.stack.mesh(),
                pose: o.pose,
            })
            .collect()
    }
}

impl crate::app_state::App {
    /// **Escreve a cena num arquivo escolhido pelo artista.**
    ///
    /// ⚠️ Sem cena não há o que exportar, e o silêncio seria indistinguível de
    /// um diálogo que falhou — o toast diz.
    pub(crate) fn sculpt3d_export(&mut self) {
        let Some(gfx) = self.gfx.as_ref() else {
            return;
        };
        let Some(scene) = gfx.sculpt3d.as_ref() else {
            self.sculpt3d_toast("Nothing to export: no sculpture open".into());
            return;
        };
        let n = scene.objects.len();

        // ⚠️ **UM FILTRO POR FORMATO, e o report do Enio é o motivo:** com um
        // filtro único listando as três extensões, o diálogo nativo (o portal
        // XDG, o GTK, o Windows) **completa o nome com a PRIMEIRA delas** — o
        // artista digitava `volta.ply` e o arquivo saía `volta.ply.obj`. Não é
        // uma segunda porta para *"que formato é este?"*: quem decide continua
        // sendo a extensão do caminho FINAL, e o filtro é só como o diálogo
        // ajuda a escrevê-la.
        let mut dialog = rfd::FileDialog::new();
        for f in MeshFormat::ALL {
            dialog = dialog.add_filter(f.extension().to_uppercase(), &[f.extension()]);
        }
        let Some(path) = dialog
            .set_file_name(format!("sculpt.{}", MeshFormat::Obj.extension()))
            .save_file()
        else {
            return;
        };
        // ⚠️ **Uma extensão que não reconhecemos NÃO vira OBJ em silêncio.** Um
        // default calado escreveria um arquivo OBJ com o nome `.fbx`, e o
        // primeiro programa a abri-lo diria que o ARQUIVO está corrompido —
        // apontando para o lugar errado.
        let Some(fmt) = path
            .extension()
            .and_then(|e| e.to_str())
            .and_then(MeshFormat::from_extension)
        else {
            self.sculpt3d_toast(format!(
                "Unknown extension: use {}",
                MeshFormat::ALL
                    .map(|f| format!(".{}", f.extension()))
                    .join(", ")
            ));
            return;
        };

        // O empréstimo das malhas termina aqui: `write` já produziu os bytes.
        let bytes = {
            let scene = self.gfx.as_ref().and_then(|g| g.sculpt3d.as_ref());
            let Some(scene) = scene else { return };
            fmt.write(&scene.export_pieces())
        };
        let size = bytes.len();
        match std::fs::write(&path, bytes) {
            Ok(()) => {
                let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("?");
                self.sculpt3d_toast(format!(
                    "Exported {n} piece(s), {} KB -- {name} ({})",
                    size / 1024,
                    lost_by(fmt)
                ));
            }
            Err(e) => self.sculpt3d_toast(format!("Export failed: {e}")),
        }
    }
}

/// O que este formato deixa para trás, em palavras.
///
/// ⚠️ **Deriva da MESMA tabela que o escritor consulta.** Uma segunda lista aqui
/// diria *"cor preservada"* sobre um STL no dia em que alguém trocasse o
/// escritor — e um aviso errado é pior que aviso nenhum, porque o artista
/// confia nele e só descobre no outro programa.
///
/// ⚠️ A **máscara** é dita sempre: nenhum dos três formatos tem campo para ela, e
/// isso não é uma pergunta — é uma constante. Quem a preserva é o documento.
fn lost_by(fmt: MeshFormat) -> String {
    let mut lost = vec!["mask"];
    if !fmt.keeps_colour() {
        lost.push("colour");
    }
    if !fmt.keeps_pieces() {
        lost.push("pieces merged");
    }
    format!("not carried: {}", lost.join(", "))
}
