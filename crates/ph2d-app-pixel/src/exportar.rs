//! **File ▸ Open, Import, Export As, Export Sprite Sheet e Repeat** do lado da família: os diálogos do
//! sistema e o documento em Draw. A lei está em `ph2d_pixel_io`; a tela, em `ph2d-panel-pixel-export`.

use ph2d_editor_core::pixel_panel::{PixelFileAction as A, PixelRequest};
use ph2d_editor_core::screens::hero::{HeroScreen, PANEL_PIXEL_EXPORT};
use ph2d_i18n::tr;
use ph2d_panel_pixel_export::canal::{self, Pedido, Tela};
use ph2d_panel_pixel_export::textos;
use ph2d_pixel_io::abrir;
use ph2d_pixel_io::exporta::{self, Export};

use crate::PixelApp;

pub use ph2d_pixel_io::abrir::le_todos;

impl PixelApp {
    /// O menu File do Pixel; `false` = não é dele.
    pub(crate) fn aplica_ficheiro(&mut self, r: &PixelRequest) -> bool {
        let PixelRequest::File(a) = r else {
            return false;
        };
        let dialogo =
            || rfd::FileDialog::new().add_filter(tr("pixel.io.kind_images"), &abrir::LIDOS);
        match a {
            A::Open => {
                if let Some(c) = ph2d_app_host::modal::pick_files(dialogo()) {
                    self.abre_leva(&c);
                }
            }
            A::ImportSheet => {
                if let Some(c) = ph2d_app_host::modal::pick_file(dialogo())
                    && let Err(t) = self.exporta.prepara_folha(&c)
                {
                    self.avisos.push((true, t));
                }
            }
            A::ExportAs => self.abre_janela(Tela::ExportAs),
            A::ExportSheet => self.abre_janela(Tela::Folha),
            A::RepeatExport => {
                if let Some(d) = self.chave.and_then(|k| self.docs.get(&k))
                    && let Some(t) = self.exporta.gravacoes.repete(&d.sprite)
                {
                    self.avisos.push((true, t));
                }
            }
        }
        true
    }

    fn abre_janela(&mut self, t: Tela) {
        if self.chave.is_some() {
            canal::abrir(t);
            self.exporta.previa.esquece();
            self.exporta.abrir = true;
        }
    }

    /// **O Export da janela**: pergunta o caminho e grava. Cancelar o diálogo deixa a janela aberta.
    fn exporta_para_o_disco(&mut self, e: &Export) -> bool {
        let Some((key, doc)) = self.chave.and_then(|k| Some((k, self.docs.get(&k)?))) else {
            return false;
        };
        let (ext, tipo) = textos::tipo_do_export(e);
        let (nome, pasta) = textos::sugestao_de_nome(&doc.sprite, ext);
        let mut d = rfd::FileDialog::new()
            .add_filter(tipo, &[ext])
            .set_file_name(nome);
        if let Some(p) = pasta {
            d = d.set_directory(p);
        }
        let Some(mut c) = ph2d_app_host::modal::save_file(d) else {
            return false;
        };
        if c.extension().is_none() {
            c.set_extension(ext);
        }
        let s = doc.sprite.clone();
        self.exporta.gravacoes.comeca(s, e.clone(), c.clone());
        // Lembra já: o Repeat acende, e uma falha repete a mesma frase em vez de calar.
        if let Some(d) = self.docs.get_mut(&key) {
            exporta::lembrar(&mut d.sprite, e, &c);
        }
        true
    }

    /// Há exports a gravar ou a folha a montar? (Os testes esperam por isto.)
    #[must_use]
    pub fn exportando(&self) -> bool {
        !self.exporta.gravacoes.vazia() || self.exporta.previa.montando()
    }

    /// As barras dos exports que começaram, para a fila da casa.
    pub fn drena_barras(&mut self, q: &mut ph2d_editor_core::progress::JobQueue) {
        self.exporta.gravacoes.drena_barras(q);
    }

    fn abre_leva(&mut self, c: &[std::path::PathBuf]) {
        for r in abrir::abrir_leva(c) {
            self.abre(r);
        }
    }

    fn abre(&mut self, r: Result<abrir::Aberto, abrir::ErroAbrir>) -> bool {
        let (frases, doc) = textos::do_abrir(r);
        self.avisos.extend(frases);
        let ok = doc.is_some();
        self.nasce_doc.extend(doc);
        ok
    }
}

/// O quadro: o que o menu mostra, o que a janela lê, e o verbo que ela pediu.
pub(crate) fn frame(app: &mut PixelApp, hero: &mut HeroScreen) {
    let frases = app.exporta.gravacoes.colhe();
    app.avisos.extend(frases);
    if let Some(c) = hero.store.take_import_asked() {
        app.exporta.pergunta(c);
    }
    let doc = app.chave.and_then(|k| Some((k, app.docs.get(&k)?)));
    hero.store
        .set_pixel_file_menu(doc.map(|(_, d)| d.sprite.export_preset().is_some()));
    if std::mem::take(&mut app.exporta.abrir) {
        hero.panel_visibility.insert(PANEL_PIXEL_EXPORT, true);
    }
    let aberta = hero.panel_visibility.get(PANEL_PIXEL_EXPORT).copied() == Some(true);
    if !aberta || (doc.is_none() && canal::pergunta().is_none()) {
        canal::publish_pergunta(None);
        hero.panel_visibility.insert(PANEL_PIXEL_EXPORT, false);
        app.exporta.previa.esquece();
        return;
    }
    if let Some((k, d)) = doc {
        canal::publish_contexto_de(&d.sprite);
        app.exporta.previa.publica(k, d.stamp, &d.sprite);
    }
    let fim = match canal::take_pedido() {
        Some(Pedido::Exportar(e)) => app.exporta_para_o_disco(&e),
        Some(Pedido::Importar(g)) => {
            let de = app.exporta.imagem.clone();
            de.is_some_and(|(i, nome, dados)| {
                let r = abrir::importar_folha_com(&i.rgba, i.w, i.h, &g, dados.as_deref());
                app.abre(r.map(|s| (s, Vec::new(), nome)))
            })
        }
        Some(Pedido::ImportarComo { pixel }) => {
            let c = std::mem::take(&mut app.exporta.leva);
            if pixel {
                app.abre_leva(&c);
            } else {
                hero.store.import_as_image(c);
            }
            true
        }
        None => false,
    };
    if fim {
        hero.panel_visibility.insert(PANEL_PIXEL_EXPORT, false);
    }
}
