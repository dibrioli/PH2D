//! **A biblioteca do navegador** (23/10): a ORDEM (a por omissão é estável; a do artista arrasta-se e
//! persiste no `estado.json`), APAGAR um pincel do artista, EXPORTAR um preset para um ficheiro portátil e
//! IMPORTAR de volta. Os seletores de ficheiro são da shell (`rfd`) e entram pelo [`Seletor`], que os
//! testes trocam.

use super::{GRUPO_SEM_ORIGEM, Presets};
use crate::painter_bridge_categorias::categoria_da_pasta;
use ph2d_brush_presets::{
    EXTENSAO, Entrada, Origem, apagar, caminho_relativo, escrever, gravar, ler, mesclar_ordem,
    mover_na_ordem,
};
use ph2d_editor_core::toast::{Toast, ToastQueue};
use ph2d_i18n::{tr, tr_with};
use ph2d_tool_painter::{PainterTool, meio_do_preset, motor_do_meio};
use std::path::{Path, PathBuf};

/// Onde gravar um exportado, a partir do nome sugerido (`None` = o artista cancelou).
type OndeExportar = Box<dyn Fn(&str) -> Option<PathBuf>>;

/// Os dois diálogos de ficheiro: onde gravar um exportado e que ficheiros importar.
pub(super) struct Seletor {
    pub(super) exportar: OndeExportar,
    pub(super) importar: Box<dyn Fn() -> Vec<PathBuf>>,
}

impl Default for Seletor {
    fn default() -> Self {
        let filtro =
            || rfd::FileDialog::new().add_filter(tr("app.painter.presets.file_filter"), &["png"]);
        Self {
            exportar: Box::new(move |nome| filtro().set_file_name(nome).save_file()),
            importar: Box::new(move || filtro().pick_files().unwrap_or_default()),
        }
    }
}

/// Um nome de pasta ou de ficheiro vindo de FORA (o JSON de um importado): sem separadores nem `..`.
fn nome_seguro(s: &str, omissao: &str) -> String {
    let limpo: String = s
        .chars()
        .map(|c| {
            if matches!(c, '/' | '\\') || c.is_control() {
                '-'
            } else {
                c
            }
        })
        .collect();
    let limpo = limpo.trim().trim_start_matches('.').to_owned();
    if limpo.is_empty() {
        omissao.to_owned()
    } else {
        limpo
    }
}

impl Presets {
    /// ⭐ **A ordem do navegador** — a ÚNICA porta: por categoria (a ordem fixa da tabela) e, dentro dela,
    /// a ordem que o artista arrastou sobre a da biblioteca. Não depende do meio de agora (era o «meio de
    /// agora primeiro» que trocava a lista ao escolher um pincel de outro meio — smoke do dono 23/10).
    pub(super) fn ordem_do_navegador(&self) -> Vec<&Entrada> {
        let cat = |e: &&Entrada| categoria_da_pasta(e.partes()[1]);
        let mut padrao: Vec<&Entrada> = self.bib.entradas.iter().collect();
        padrao.sort_by_key(cat);
        let rels: Vec<String> = padrao.iter().map(|e| e.relativo.clone()).collect();
        let mut vista: Vec<&Entrada> = mesclar_ordem(&rels, &self.estado.ordem)
            .iter()
            .filter_map(|r| self.bib.procurar(r))
            .collect();
        vista.sort_by_key(cat);
        vista
    }

    /// A secção do navegador onde `rel` aparece (Favoritos, ou a categoria): o arrasto não sai dela.
    fn secao(&self, rel: &str) -> Option<(bool, usize)> {
        let e = self.bib.procurar(rel)?;
        Some((
            self.estado.favoritos.contains(rel),
            categoria_da_pasta(e.partes()[1]),
        ))
    }

    /// O artista largou `arrastado` antes (ou `depois`) de `alvo`. Fora da mesma secção, nada muda.
    pub(super) fn mover(&mut self, arrastado: &str, alvo: &str, depois: bool) {
        if self.secao(arrastado).is_none() || self.secao(arrastado) != self.secao(alvo) {
            return;
        }
        let lista: Vec<String> = self
            .ordem_do_navegador()
            .iter()
            .map(|e| e.relativo.clone())
            .collect();
        if let Some(nova) = mover_na_ordem(&lista, arrastado, alvo, depois) {
            self.estado.ordem = nova;
        }
    }

    /// Apaga o preset activo, se for só do artista. O pincel fica na mão até o artista escolher outro.
    pub(super) fn apagar_ativo(&mut self, painter: &PainterTool, toasts: &mut ToastQueue) {
        let Some(rel) = self.ativo(painter) else {
            return;
        };
        let Some(e) = self.bib.procurar(&rel) else {
            return;
        };
        if e.origem != Origem::Utilizador || e.tem_fabrica {
            return;
        }
        let nome = e.preset.parametros.name.clone();
        let Some(raiz) = self.utilizador.clone() else {
            return;
        };
        if let Err(err) = apagar(&raiz, &rel) {
            toasts.push(Toast::error(tr_with(
                "app.painter.presets.delete_failed",
                &[("e", &err)],
            )));
            return;
        }
        self.estado.apagado(&rel);
        self.aplicado.retain(|_, r| *r != rel);
        self.recarregar(toasts);
        toasts.push(Toast::success(tr_with(
            "app.painter.presets.deleted",
            &[("name", &nome)],
        )));
    }

    /// O preset activo como um ficheiro portátil em `destino`: o ficheiro da biblioteca (o pincel com
    /// as imagens de todos os slots) mais a pasta do meio, que o caminho dizia e o ficheiro solto não diz.
    pub(super) fn exportar_para(
        &self,
        painter: &PainterTool,
        destino: &Path,
    ) -> Result<(), String> {
        let sem_ativo = || tr("panel.painter_layers.wetpaint.preset_none").to_owned();
        let rel = self.ativo(painter).ok_or_else(sem_ativo)?;
        let e = self.bib.procurar(&rel).ok_or_else(sem_ativo)?;
        let mut p = e.preset.clone();
        p.parametros.library_folder = Some(e.partes()[1].to_owned());
        let bytes = escrever(&p).map_err(|e| e.to_string())?;
        std::fs::write(destino, bytes).map_err(|e| format!("{}: {e}", destino.display()))
    }

    pub(super) fn exportar(&self, painter: &PainterTool, toasts: &mut ToastQueue) {
        let Some(nome) = self
            .ativo(painter)
            .and_then(|r| self.bib.procurar(&r).map(|e| e.partes()[3].to_owned()))
        else {
            return;
        };
        let Some(destino) = (self.seletor.exportar)(&format!("{nome}{EXTENSAO}")) else {
            return;
        };
        let toast = match self.exportar_para(painter, &destino) {
            Ok(()) => Toast::success(tr_with(
                "app.painter.presets.exported",
                &[("path", &destino.display())],
            )),
            Err(e) => Toast::error(tr_with("app.painter.presets.export_failed", &[("e", &e)])),
        };
        toasts.push(toast);
    }

    /// Copia cada ficheiro para a pasta do artista, no lugar que ele diz (`<motor>/<pasta do meio>/<grupo>`,
    /// um nome livre), e devolve os caminhos relativos que entraram. Os que não servem dizem porquê.
    pub(super) fn importar_de(
        &mut self,
        ficheiros: &[PathBuf],
        toasts: &mut ToastQueue,
    ) -> Vec<String> {
        let Some(raiz) = self.utilizador.clone() else {
            toasts.push(Toast::error(tr("app.painter.presets.no_user_folder")));
            return Vec::new();
        };
        let mut entrou = Vec::new();
        for f in ficheiros {
            let nome_f = f
                .file_name()
                .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
            let falha = |toasts: &mut ToastQueue, e: &dyn std::fmt::Display| {
                toasts.push(Toast::error(tr_with(
                    "app.painter.presets.import_failed",
                    &[("file", &nome_f), ("e", e)],
                )));
            };
            let mut p = match std::fs::read(f)
                .map_err(|e| e.to_string())
                .and_then(|b| ler(&b).map_err(|e| e.to_string()))
            {
                Ok(p) => p,
                Err(e) => {
                    falha(toasts, &e);
                    continue;
                }
            };
            let Some(m) = meio_do_preset(&p.parametros.media) else {
                toasts.push(Toast::error(tr_with(
                    "app.painter.presets.import_unknown_media",
                    &[("file", &nome_f), ("media", &p.parametros.media)],
                )));
                continue;
            };
            let motor = motor_do_meio(m);
            let meio = nome_seguro(
                p.parametros.library_folder.as_deref().unwrap_or(""),
                GRUPO_SEM_ORIGEM,
            );
            let grupo = nome_seguro(&p.parametros.group, GRUPO_SEM_ORIGEM);
            let base = nome_seguro(&p.parametros.name, tr("app.painter.presets.new_name"));
            let nome = self.bib.nome_livre(motor, &meio, &grupo, &base);
            let rel = caminho_relativo(motor, &meio, &grupo, &nome);
            p.parametros.name.clone_from(&nome);
            p.parametros.group.clone_from(&grupo);
            p.parametros.library_folder = None;
            p.origem = Origem::Importado;
            match gravar(&raiz, &rel, &p) {
                Ok(_) => {
                    entrou.push(rel);
                    self.recarregar(toasts);
                }
                Err(e) => falha(toasts, &e),
            }
        }
        if !entrou.is_empty() {
            toasts.push(Toast::success(tr_with(
                "app.painter.presets.imported",
                &[("n", &entrou.len())],
            )));
        }
        entrou
    }

    /// O pedido Import: os ficheiros que o artista escolheu entram, e o último fica no pincel.
    pub(super) fn importar(&mut self, painter: &mut PainterTool, toasts: &mut ToastQueue) {
        let ficheiros = (self.seletor.importar)();
        if let Some(ultimo) = self.importar_de(&ficheiros, toasts).pop() {
            self.lembrar_o_que_se_deixa(painter);
            self.aplicar(painter, &ultimo, toasts);
        }
    }
}
