//! **File ▸ Open, Import, Export As, Export Sprite Sheet e Repeat** do lado da família: os diálogos do
//! sistema e o documento em Draw. A lei está em `ph2d_pixel_io`; a tela, em `ph2d-panel-pixel-export`.

use ph2d_editor_core::pixel_panel::{PixelFileAction as A, PixelRequest};
use ph2d_editor_core::screens::hero::{HeroScreen, PANEL_PIXEL_EXPORT};
use ph2d_i18n::tr;
use ph2d_panel_pixel_export::canal::{self, Gravacoes, Imagem, Pedido, PreviaDaFolha, Tela};
use ph2d_panel_pixel_export::textos;
use ph2d_pixel_io::abrir;
use ph2d_pixel_io::exporta::{self, Export};

use crate::PixelApp;

const LIDOS: [&str; 6] = ["ase", "aseprite", "png", "apng", "gif", "webp"];

/// O lado da família da janela de Export e Import.
#[derive(Default)]
pub struct Estado {
    previa: PreviaDaFolha,
    gravacoes: Gravacoes,
    /// A imagem do Import, o nome do objecto e o `.json` ao lado.
    imagem: Option<(Imagem, String, Option<String>)>,
    /// A janela a abrir no próximo quadro (lá há o `hero`).
    abrir: bool,
}

impl PixelApp {
    /// O menu File do Pixel; `false` = não é dele.
    pub(crate) fn aplica_ficheiro(&mut self, r: &PixelRequest) -> bool {
        let PixelRequest::File(a) = r else {
            return false;
        };
        let dialogo = || rfd::FileDialog::new().add_filter(tr("pixel.io.kind_images"), &LIDOS);
        match a {
            A::Open => {
                if let Some(c) = ph2d_app_host::modal::pick_files(dialogo()) {
                    self.abre(abrir::abrir_caminhos(&c));
                }
            }
            A::ImportSheet => {
                if let Some(c) = ph2d_app_host::modal::pick_file(dialogo()) {
                    match abrir::abrir_caminhos(std::slice::from_ref(&c)) {
                        Ok((s, _, nome)) => {
                            let dados = abrir::dados_ao_lado(&c);
                            let seq = self.exporta.previa.proximo_seq();
                            let i = canal::imagem_de_import(&s, seq, dados.is_some());
                            canal::publish_imagem(Some(i.clone()));
                            self.exporta.imagem = Some((i, nome, dados));
                            self.abre_janela(Tela::Import);
                        }
                        Err(e) => self.avisos.push((true, textos::erro_abrir(&e))),
                    }
                }
            }
            A::ExportAs => self.abre_janela(Tela::ExportAs),
            A::ExportSheet => self.abre_janela(Tela::Folha),
            A::RepeatExport => {
                if let Some((k, d)) = self.chave.and_then(|k| Some((k, self.docs.get(&k)?)))
                    && let Some(t) = self.exporta.gravacoes.repete(k, &d.sprite)
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
        self.exporta.gravacoes.comeca(key, s, e.clone(), c);
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

    fn abre(&mut self, r: Result<abrir::Aberto, abrir::ErroAbrir>) -> bool {
        match r {
            Ok((s, avisos, nome)) => {
                self.avisos.extend(textos::do_aberto(&nome, avisos));
                self.nasce_doc = Some((s, nome));
                true
            }
            Err(e) => {
                self.avisos.push((true, textos::erro_abrir(&e)));
                false
            }
        }
    }
}

/// O quadro: o que o menu mostra, o que a janela lê, e o verbo que ela pediu.
pub(crate) fn frame(app: &mut PixelApp, hero: &mut HeroScreen) {
    let docs = &mut app.docs;
    let frases = app.exporta.gravacoes.colhe_em(|k, e, c| {
        if let Some(d) = docs.get_mut(&k) {
            exporta::lembrar(&mut d.sprite, e, c);
        }
    });
    app.avisos.extend(frases);
    let doc = app.chave.and_then(|k| Some((k, app.docs.get(&k)?)));
    hero.store
        .set_pixel_file_menu(doc.map(|(_, d)| d.sprite.export_preset().is_some()));
    if std::mem::take(&mut app.exporta.abrir) {
        hero.panel_visibility.insert(PANEL_PIXEL_EXPORT, true);
    }
    let aberta = hero.panel_visibility.get(PANEL_PIXEL_EXPORT).copied() == Some(true);
    if !aberta || doc.is_none() {
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
        None => false,
    };
    if fim {
        hero.panel_visibility.insert(PANEL_PIXEL_EXPORT, false);
    }
}
