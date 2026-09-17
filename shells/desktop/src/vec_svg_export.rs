//! ⭐ **A PONTE do `File > Export SVG…`** — o gesto, e só ele.
//!
//! A **lei** (o documento inteiro serializado: tintas, gradientes, tracejado, mistura, a aproximação
//! de cada arco) é pura e mudou-se para [`ph2d_app_vec::svg_export`] na Fase C — 360 linhas que
//! nunca precisaram de um `AppGfx`.
//!
//! O que fica é o método de `App` que abre o diálogo de ficheiro e lê a cena do `gfx`. ⚠️ **As duas
//! leis de produto vivem aqui de propósito**, porque são sobre a SESSÃO e não sobre o documento:
//! *pergunta SEMPRE o caminho* (um export não tem *"o ficheiro da sessão"*) e *sem forma visível
//! ele DIZ*, em vez de escrever um ficheiro vazio que abre em branco.
//!
//! ⚠️ A re-exportação mantém `crate::vec_svg_export::svg(…)` byte a byte igual para quem já a
//! escreve (HOWTO §1.4 — *a reescrita uniforme é o que evita um mapa de excepções*).

pub(crate) use ph2d_app_vec::svg_export::*;
use ph2d_i18n::{tr, tr_with};

impl crate::App {
    /// ⭐⭐⭐ **O GESTO de exportar** — *File > Export SVG…*.
    ///
    /// ⚠️ **Pergunta SEMPRE o caminho**, ao contrário do `Save`: um export não tem *"o ficheiro da
    /// sessão"* — o projecto tem, e não é o mesmo ficheiro. Gravar por cima do último SVG sem
    /// perguntar seria o app a decidir onde o trabalho de outra pessoa vive.
    ///
    /// ⚠️ **Sem forma visível ele DIZ**, e não escreve um ficheiro vazio: um SVG de zero formas
    /// abre em branco, e o artista conclui que a exportação se partiu.
    pub(crate) fn export_svg_gesture(&mut self) {
        let Some(gfx) = self.gfx.as_ref() else {
            return;
        };
        let xf = ph2d_vec_entities::transform::build(&gfx.sim, &self.vec.entities);
        let vista = ph2d_vec_entities::entities::view_state(&gfx.sim, &self.vec.entities);
        // ⚠️ As DUAS perguntas são as mesmas que o balde faz: o que não se vê não sai, e o que é
        // área de balde sai MARCADO (`VecViewState::is_derived` é populado do `VecBucketFill`).
        let out = svg(&gfx.vec_scene, &xf, &|id| vista.is_hidden(id), &|id| {
            vista.is_derived(id)
        });
        if out.formas == 0 {
            self.toast(tr("shell.vec_svg_export.export_svg_the_drawing").to_string());
            return;
        }
        let sugerido = self
            .project_path
            .as_deref()
            .and_then(|p| std::path::Path::new(p).file_stem()?.to_str())
            .map_or_else(|| "drawing.svg".to_string(), |s| format!("{s}.svg"));
        let Some(path) = ph2d_app_host::modal::save_file(
            rfd::FileDialog::new()
                .set_file_name(&sugerido)
                .add_filter("SVG (.svg)", &["svg"]),
        ) else {
            return; // o artista desistiu — e desistir não é um erro
        };
        match std::fs::write(&path, out.texto.as_bytes()) {
            Ok(()) => {
                let extra = if out.aproximadas.is_empty() {
                    String::new()
                } else {
                    tr_with(
                        "shell.vec_svg_export.approximated",
                        &[("aproximadas", &(out.aproximadas.len()))],
                    )
                };
                eprintln!(
                    "[ph2d-vec] SVG: {} ({} forma[s], {} bytes){extra}",
                    path.display(),
                    out.formas,
                    out.texto.len()
                );
                self.toast(tr_with(
                    "shell.vec_svg_export.exported_shape_s_to",
                    &[
                        ("formas", &(out.formas)),
                        (
                            "to_string_lossy",
                            &(path.file_name().unwrap_or_default().to_string_lossy()),
                        ),
                        ("extra", &extra),
                    ],
                ));
            }
            Err(e) => {
                eprintln!("[ph2d-vec] SVG: erro ao gravar {}: {e}", path.display());
                self.toast(tr_with(
                    "shell.vec_svg_export.export_svg_failed",
                    &[("e", &e)],
                ));
            }
        }
    }
}
