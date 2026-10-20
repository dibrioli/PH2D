//! **A ponte dos presets do Wet Painter** (W1, `docs/Painter/NewFeaturesWetPaint/02 §5`): o disco e a
//! biblioteca são DAQUI. O tool só regista o pedido do painel (`take_wet_preset_request`) e o painel
//! só pinta a vista que esta ponte publica (`set_current_wet_presets`) — nenhum dos dois lê ficheiros.
//!
//! Fábrica em `assets/brushes/` (do repositório), do artista em `~/.local/share/ph2d/brushes/`
//! (`pasta_do_utilizador`). Um ficheiro do artista no mesmo caminho SOBREPÕE o de fábrica («Save as
//! Default»); o «Reset» apaga-o. Favoritos, a política e os volumes lembrados vivem em
//! `estado.json` na raiz do artista.

use ph2d_brush_presets::{
    Biblioteca, Entrada, EstadoDoUtilizador, Origem, Parametros, Politica, Preset, apagar,
    caminho_relativo, gravar, pasta_do_utilizador,
};
use ph2d_editor_core::toast::{Toast, ToastQueue};
use ph2d_i18n::{Idioma, idioma, tr, tr_em, tr_with};
use ph2d_panel_painter_layers::{VistaPreset, VistaPresets, set_current_wet_presets};
use ph2d_tool_painter::{
    MEIO_DO_PRESET, MOTOR_DOS_PRESETS, PainterTool, PedidoPreset, miniatura_do_preset,
};
use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::Arc;

/// A pasta do meio e o grupo de um preset guardado sem nenhum activo de onde herdar o lugar.
const MEIO_SEM_ORIGEM: &str = "Watercolor";
const GRUPO_SEM_ORIGEM: &str = "User";

fn pasta_de_fabrica() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes")
}

struct Presets {
    fabrica: PathBuf,
    utilizador: Option<PathBuf>,
    bib: Biblioteca,
    estado: EstadoDoUtilizador,
    /// O preset que está NO PINCEL (aplicado por esta ponte). `estado.ativo` é o que se lembra.
    aplicado: Option<String>,
    vista: Arc<VistaPresets>,
}

thread_local! {
    static PRESETS: RefCell<Option<Presets>> = const { RefCell::new(None) };
}

impl Presets {
    fn abrir(painter: &mut PainterTool, toasts: &mut ToastQueue) -> Self {
        Self::abrir_em(pasta_de_fabrica(), pasta_do_utilizador(), painter, toasts)
    }

    fn abrir_em(
        fabrica: PathBuf,
        utilizador: Option<PathBuf>,
        painter: &mut PainterTool,
        toasts: &mut ToastQueue,
    ) -> Self {
        let estado = utilizador
            .as_deref()
            .map(EstadoDoUtilizador::ler)
            .transpose()
            .unwrap_or_else(|e| {
                toasts.push(Toast::error(tr_with(
                    "app.painter.presets.state_unreadable",
                    &[("e", &e)],
                )));
                None
            })
            .unwrap_or_default();
        let mut p = Self {
            bib: Biblioteca::carregar(&fabrica, utilizador.as_deref()),
            fabrica,
            utilizador,
            estado,
            aplicado: None,
            vista: Arc::default(),
        };
        p.dizer_ilegiveis(toasts);
        // O pincel abre com o preset que o artista tinha (se ainda existe).
        if let Some(rel) = p.estado.ativo.clone()
            && p.bib.procurar(&rel).is_some()
        {
            p.aplicar(painter, &rel, toasts);
        }
        p.publicar();
        p
    }

    fn dizer_ilegiveis(&self, toasts: &mut ToastQueue) {
        if self.bib.erros.is_empty() {
            return;
        }
        for (caminho, e) in &self.bib.erros {
            eprintln!("[presets] {}: {e}", caminho.display());
        }
        toasts.push(Toast::error(tr_with(
            "app.painter.presets.unreadable",
            &[("n", &self.bib.erros.len())],
        )));
    }

    fn recarregar(&mut self, toasts: &mut ToastQueue) {
        self.bib = Biblioteca::carregar(&self.fabrica, self.utilizador.as_deref());
        self.dizer_ilegiveis(toasts);
    }

    fn gravar_estado(&self, toasts: &mut ToastQueue) {
        if let Some(u) = &self.utilizador
            && let Err(e) = self.estado.gravar(u)
        {
            toasts.push(Toast::error(tr_with(
                "app.painter.presets.state_save_failed",
                &[("e", &e)],
            )));
        }
    }

    /// Aplica o preset `rel` ao pincel e, por cima, o que a política lembrou dele.
    fn aplicar(&mut self, painter: &mut PainterTool, rel: &str, toasts: &mut ToastQueue) {
        let Some(e) = self.bib.procurar(rel) else {
            return;
        };
        let falhas = painter.wet_preset_aplicar(&e.preset);
        for id in falhas {
            toasts.push(Toast::error(tr_with(
                "app.painter.presets.texture_unreadable",
                &[("name", &e.preset.parametros.name), ("id", &id)],
            )));
        }
        if let Some(l) = self.estado.lembrados.get(rel) {
            painter.wet_preset_aplicar_volumes(&l.volumes);
            if let Some(m) = &l.paint_mode {
                painter.wet_preset_aplicar_modo(m);
            }
        }
        self.aplicado = Some(rel.to_owned());
        self.estado.usar(rel);
    }

    /// Antes de deixar o preset aplicado: guarda o que a política manda.
    fn lembrar_o_que_se_deixa(&mut self, painter: &PainterTool) {
        if let Some(prev) = self.aplicado.clone() {
            self.estado.lembrar(
                &prev,
                painter.wet_preset_volumes(),
                Some(painter.wet_preset_modo()),
            );
        }
    }

    /// Captura o pincel sobre `base` e grava-o em `rel` na raiz do artista.
    fn guardar(
        &mut self,
        painter: &PainterTool,
        base: &Parametros,
        rel: &str,
        toasts: &mut ToastQueue,
    ) -> bool {
        let Some(raiz) = self.utilizador.clone() else {
            toasts.push(Toast::error(tr("app.painter.presets.no_user_folder")));
            return false;
        };
        // A tabela raio → largura (W2) mede-se pela sonda da fábrica: o pincel do artista leva a do
        // preset de onde veio (`wet_preset_capturar`), a promessa que ele estava a ver ao pintar.
        let (parametros, texturas) = painter.wet_preset_capturar(base);
        let mut preset = Preset {
            parametros,
            origem: Origem::Utilizador,
            miniatura: ph2d_brush_presets::Miniatura {
                largura: 1,
                altura: 1,
                rgba: vec![0; 4],
            },
            texturas,
        };
        preset.miniatura = miniatura_do_preset(&preset);
        match gravar(&raiz, rel, &preset) {
            Ok(_) => {
                toasts.push(Toast::success(tr_with(
                    "app.painter.presets.saved",
                    &[("name", &preset.parametros.name)],
                )));
                true
            }
            Err(e) => {
                toasts.push(Toast::error(tr_with(
                    "app.painter.presets.save_failed",
                    &[("e", &e)],
                )));
                false
            }
        }
    }

    fn atender(
        &mut self,
        painter: &mut PainterTool,
        pedido: PedidoPreset,
        toasts: &mut ToastQueue,
    ) {
        match pedido {
            PedidoPreset::Escolher(rel) => {
                self.lembrar_o_que_se_deixa(painter);
                self.aplicar(painter, &rel, toasts);
            }
            PedidoPreset::AlternarFavorito => {
                if let Some(rel) = self.aplicado.clone() {
                    self.estado.alternar_favorito(&rel);
                }
            }
            PedidoPreset::GuardarComoNovo => {
                let ativo = self.aplicado.as_deref().and_then(|r| self.bib.procurar(r));
                let (meio, grupo, base) = ativo.map_or_else(
                    || {
                        (
                            MEIO_SEM_ORIGEM.to_owned(),
                            GRUPO_SEM_ORIGEM.to_owned(),
                            Parametros::novo(
                                MEIO_DO_PRESET,
                                GRUPO_SEM_ORIGEM,
                                tr("app.painter.presets.new_name"),
                            ),
                        )
                    },
                    |e: &Entrada| {
                        let [_, m, g, _] = e.partes();
                        (m.to_owned(), g.to_owned(), e.preset.parametros.clone())
                    },
                );
                let nome = self
                    .bib
                    .nome_livre(MOTOR_DOS_PRESETS, &meio, &grupo, &base.name);
                let mut base = base;
                base.name.clone_from(&nome);
                base.group.clone_from(&grupo);
                let rel = caminho_relativo(MOTOR_DOS_PRESETS, &meio, &grupo, &nome);
                if self.guardar(painter, &base, &rel, toasts) {
                    self.recarregar(toasts);
                    self.lembrar_o_que_se_deixa(painter);
                    self.aplicado = Some(rel.clone());
                    self.estado.usar(&rel);
                }
            }
            PedidoPreset::GuardarComoPadrao => {
                let Some(rel) = self.aplicado.clone() else {
                    return;
                };
                let Some(base) = self.bib.procurar(&rel).map(|e| e.preset.parametros.clone())
                else {
                    return;
                };
                if self.guardar(painter, &base, &rel, toasts) {
                    self.estado.esquecer(&rel);
                    self.recarregar(toasts);
                }
            }
            PedidoPreset::Repor => {
                let Some(rel) = self.aplicado.clone() else {
                    return;
                };
                if let Some(e) = self.bib.procurar(&rel)
                    && e.origem == Origem::Utilizador
                    && e.tem_fabrica
                    && let Some(u) = &self.utilizador
                {
                    if let Err(err) = apagar(u, &rel) {
                        toasts.push(Toast::error(tr_with(
                            "app.painter.presets.save_failed",
                            &[("e", &err)],
                        )));
                        return;
                    }
                    self.recarregar(toasts);
                }
                self.estado.esquecer(&rel);
                self.aplicar(painter, &rel, toasts);
            }
            PedidoPreset::Politica(i) => {
                if let Some(&p) = Politica::TODAS.get(usize::from(i)) {
                    self.estado.politica = p;
                }
            }
        }
        self.gravar_estado(toasts);
        self.publicar();
    }

    fn publicar(&mut self) {
        let entradas = self
            .bib
            .entradas
            .iter()
            .map(|e| {
                let [_, meio, grupo, nome] = e.partes();
                let (meio, grupo) = (
                    rotulo_da_pasta(idioma(), meio),
                    rotulo_da_pasta(idioma(), grupo),
                );
                VistaPreset {
                    relativo: e.relativo.clone(),
                    nome: nome.to_owned(),
                    grupo: tr_with(
                        "app.painter.presets.group",
                        &[("media", &meio), ("group", &grupo)],
                    ),
                    favorito: self.estado.favoritos.contains(&e.relativo),
                }
            })
            .collect();
        let politica = Politica::TODAS
            .iter()
            .position(|p| *p == self.estado.politica)
            .unwrap_or(0) as u8;
        self.vista = Arc::new(VistaPresets {
            entradas,
            ativo: self.aplicado.clone(),
            politica,
            ilegiveis: self.bib.erros.len(),
        });
    }
}

/// A cada quadro com o Painter activo: abre a biblioteca na primeira vez que o Wet Paint está armado,
/// atende o pedido do painel e publica a vista.
pub(crate) fn drive(painter: &mut PainterTool, wet_armado: bool, toasts: &mut ToastQueue) {
    PRESETS.with(|c| {
        let mut c = c.borrow_mut();
        if c.is_none() {
            if !wet_armado {
                return;
            }
            *c = Some(Presets::abrir(painter, toasts));
        }
        let Some(p) = c.as_mut() else {
            return;
        };
        if let Some(pedido) = painter.take_wet_preset_request() {
            p.atender(painter, pedido, toasts);
        }
        set_current_wet_presets(Some(p.vista.clone()));
    });
}

/// O nome de uma pasta da FÁBRICA como rótulo de interface (HR-15); a pasta do artista é texto DELE e
/// mostra-se como está. Pasta de fábrica nova = um braço aqui (`cada_pasta_de_fabrica_tem_rotulo`).
fn rotulo_da_pasta(idioma: Idioma, nome: &str) -> &str {
    match nome {
        "Watercolor" => tr_em(idioma, "app.painter.presets.folder.watercolor"),
        "Fluid" => tr_em(idioma, "app.painter.presets.folder.fluid"),
        "Oil" => tr_em(idioma, "app.painter.presets.folder.oil"),
        "ExpressOil" => tr_em(idioma, "app.painter.presets.folder.express_oil"),
        "Pencil" => tr_em(idioma, "app.painter.presets.folder.pencil"),
        "Pastel" => tr_em(idioma, "app.painter.presets.folder.pastel"),
        "Bristle" => tr_em(idioma, "app.painter.presets.folder.bristle"),
        "Round" => tr_em(idioma, "app.painter.presets.folder.round"),
        "Graphite" => tr_em(idioma, "app.painter.presets.folder.graphite"),
        "Charcoal" => tr_em(idioma, "app.painter.presets.folder.charcoal"),
        "Dry" => tr_em(idioma, "app.painter.presets.folder.dry"),
        "Ink" => tr_em(idioma, "app.painter.presets.folder.ink"),
        "Pen" => tr_em(idioma, "app.painter.presets.folder.pen"),
        "Brush" => tr_em(idioma, "app.painter.presets.folder.brush"),
        "Marker" => tr_em(idioma, "app.painter.presets.folder.marker"),
        "Airbrush" => tr_em(idioma, "app.painter.presets.folder.airbrush"),
        "Spray" => tr_em(idioma, "app.painter.presets.folder.spray"),
        "Gouache" => tr_em(idioma, "app.painter.presets.folder.gouache"),
        "Colored" => tr_em(idioma, "app.painter.presets.folder.colored"),
        "Scratch" => tr_em(idioma, "app.painter.presets.folder.scratch"),
        "Scrape" => tr_em(idioma, "app.painter.presets.folder.scrape"),
        "Dabber" => tr_em(idioma, "app.painter.presets.folder.dabber"),
        _ => nome,
    }
}

#[cfg(test)]
#[path = "painter_bridge_presets_tests.rs"]
mod tests;
