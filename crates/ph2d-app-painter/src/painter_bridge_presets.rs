//! **A ponte dos presets** (W1, `docs/Painter/NewFeaturesWetPaint/02 §5`; L1 — os quatro meios,
//! `docs/Painter/Pinceis/03_MOTORES.md` §2): o disco e a biblioteca são DAQUI. O tool só regista o pedido do
//! painel (`take_wet_preset_request`) e o painel só pinta a vista que esta ponte publica
//! (`set_current_wet_presets`) — nenhum dos dois lê ficheiros.
//!
//! Fábrica em `assets/brushes/` (do repositório), do artista em `~/.local/share/ph2d/brushes/`
//! (`pasta_do_utilizador`); a pasta-raiz de cada meio é `motor_do_meio` (`WetPainter`, `Digital`,
//! `Watercolor`, `Impasto`). Um ficheiro do artista no mesmo caminho SOBREPÕE o de fábrica («Save as
//! Default»); o «Reset» apaga-o. Favoritos, a política, os volumes lembrados e o activo de cada meio vivem em
//! `estado.json` na raiz do artista.
//!
//! A biblioteca abre com o Painter, em qualquer meio; o navegador lista os quatro, o meio de agora primeiro.
//! Escolher um preset de outro meio TROCA o meio (`preset_aplicar`); um meio desconhecido é recusado com
//! aviso. O preset lembrado de cada meio aplica-se UMA vez por sessão, no 1.º quadro em que esse meio está
//! activo (o comportamento de sempre do Wet Paint, agora em cada meio).

use ph2d_brush_presets::{
    Biblioteca, Entrada, EstadoDoUtilizador, Origem, Parametros, Politica, Preset, apagar,
    caminho_relativo, gravar, pasta_do_utilizador,
};
use ph2d_editor_core::toast::{Toast, ToastQueue};
use ph2d_i18n::{Idioma, idioma, tr, tr_em, tr_with};
use ph2d_panel_painter_layers::{VistaPreset, VistaPresets, set_current_wet_presets};
use ph2d_tool_painter::{
    MeioDesconhecido, PaintMedia, PainterTool, PedidoPreset, meio_do_preset, miniatura_do_preset,
    motor_do_meio, nome_do_meio,
};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;

/// A pasta do meio (o 2.º nível) de um preset guardado sem nenhum activo de onde herdar o lugar: a do Wet
/// Paint de sempre; nos outros meios a do artista.
fn meio_sem_origem(m: PaintMedia) -> &'static str {
    if m == PaintMedia::WetPaint {
        "Watercolor"
    } else {
        GRUPO_SEM_ORIGEM
    }
}
const GRUPO_SEM_ORIGEM: &str = "User";

fn pasta_de_fabrica() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes")
}

struct Presets {
    fabrica: PathBuf,
    utilizador: Option<PathBuf>,
    bib: Biblioteca,
    estado: EstadoDoUtilizador,
    /// O preset que está NO PINCEL de cada meio (aplicado por esta ponte), pelo fio do meio.
    aplicado: BTreeMap<u8, String>,
    /// Os meios que já abriram nesta sessão — o lembrado de cada um aplica-se uma vez.
    abertos: BTreeSet<u8>,
    /// O meio que a vista publicada mostra.
    meio: PaintMedia,
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
            aplicado: BTreeMap::new(),
            abertos: BTreeSet::new(),
            meio: painter.paint_media(),
            vista: Arc::default(),
        };
        p.dizer_ilegiveis(toasts);
        p.abrir_meio(painter, toasts);
        p.publicar();
        p
    }

    /// O 1.º quadro do meio de agora nesta sessão: o pincel abre com o preset que o artista tinha NELE.
    fn abrir_meio(&mut self, painter: &mut PainterTool, toasts: &mut ToastQueue) {
        let m = painter.paint_media();
        if !self.abertos.insert(m.to_u8()) {
            return;
        }
        let nome = nome_do_meio(m);
        let lembrado = self.estado.ativo_por_meio.get(nome).cloned().or_else(|| {
            // Um `estado.json` de antes da L1 só tem o activo de todos (era sempre do Wet Paint).
            self.estado.ativo.clone().filter(|r| {
                self.bib
                    .procurar(r)
                    .is_some_and(|e| e.preset.parametros.media == nome)
            })
        });
        if let Some(rel) = lembrado.filter(|r| self.bib.procurar(r).is_some()) {
            self.aplicar(painter, &rel, toasts);
        }
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

    /// O preset no pincel do meio de agora do `painter`.
    fn ativo(&self, painter: &PainterTool) -> Option<String> {
        self.aplicado.get(&painter.paint_media().to_u8()).cloned()
    }

    /// Aplica o preset `rel` ao pincel (no meio DELE) e, por cima, o que a política lembrou dele. Um meio
    /// desconhecido é recusado com aviso e nada muda.
    fn aplicar(&mut self, painter: &mut PainterTool, rel: &str, toasts: &mut ToastQueue) {
        let Some(e) = self.bib.procurar(rel) else {
            return;
        };
        let nome = &e.preset.parametros.name;
        let falhas = match painter.preset_aplicar(&e.preset) {
            Ok(f) => f,
            Err(MeioDesconhecido(media)) => {
                toasts.push(Toast::error(tr_with(
                    "app.painter.presets.unknown_media",
                    &[("name", nome), ("media", &media)],
                )));
                return;
            }
        };
        for f in falhas {
            let toast = match f.strip_prefix("paint_mode:") {
                Some(modo) => tr_with(
                    "app.painter.presets.mode_not_applied",
                    &[("name", nome), ("mode", &modo)],
                ),
                None => tr_with(
                    "app.painter.presets.texture_unreadable",
                    &[("name", nome), ("id", &f)],
                ),
            };
            toasts.push(Toast::error(toast));
        }
        if let Some(l) = self.estado.lembrados.get(rel) {
            painter.preset_aplicar_volumes(&l.volumes);
            if let Some(m) = &l.paint_mode {
                painter.preset_aplicar_modo(m);
            }
        }
        let m = painter.paint_media();
        self.aplicado.insert(m.to_u8(), rel.to_owned());
        self.abertos.insert(m.to_u8());
        self.estado.usar_no_meio(nome_do_meio(m), rel);
    }

    /// Antes de deixar o preset aplicado: guarda o que a política manda.
    fn lembrar_o_que_se_deixa(&mut self, painter: &PainterTool) {
        if let Some(prev) = self.ativo(painter) {
            self.estado
                .lembrar(&prev, painter.preset_volumes(), painter.preset_modo());
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
        let (parametros, texturas) = painter.preset_capturar(base);
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
                if let Some(rel) = self.ativo(painter) {
                    self.estado.alternar_favorito(&rel);
                }
            }
            PedidoPreset::GuardarComoNovo => {
                let m = painter.paint_media();
                let ativo = self.ativo(painter);
                let ativo = ativo.as_deref().and_then(|r| self.bib.procurar(r));
                let (meio, grupo, base) = ativo.map_or_else(
                    || {
                        (
                            meio_sem_origem(m).to_owned(),
                            GRUPO_SEM_ORIGEM.to_owned(),
                            Parametros::novo(
                                nome_do_meio(m),
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
                let motor = motor_do_meio(m);
                let nome = self.bib.nome_livre(motor, &meio, &grupo, &base.name);
                let mut base = base;
                base.name.clone_from(&nome);
                base.group.clone_from(&grupo);
                let rel = caminho_relativo(motor, &meio, &grupo, &nome);
                if self.guardar(painter, &base, &rel, toasts) {
                    self.recarregar(toasts);
                    self.lembrar_o_que_se_deixa(painter);
                    self.aplicado.insert(m.to_u8(), rel.clone());
                    self.estado.usar_no_meio(nome_do_meio(m), &rel);
                }
            }
            PedidoPreset::GuardarComoPadrao => {
                let Some(rel) = self.ativo(painter) else {
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
                let Some(rel) = self.ativo(painter) else {
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
        self.meio = painter.paint_media();
        self.publicar();
    }

    /// A vista do navegador: os quatro meios agrupados (`meio · pasta · grupo`), o meio de agora primeiro; o
    /// activo é o do meio de agora.
    fn publicar(&mut self) {
        let agora = nome_do_meio(self.meio);
        let mut entradas: Vec<(bool, VistaPreset)> = self
            .bib
            .entradas
            .iter()
            .map(|e| {
                let [_, meio, grupo, nome] = e.partes();
                let media = &e.preset.parametros.media;
                let motor = meio_do_preset(media).map_or(media.as_str(), |m| tr(m.name_key()));
                let (meio, grupo) = (
                    rotulo_da_pasta(idioma(), meio),
                    rotulo_da_pasta(idioma(), grupo),
                );
                let vista = VistaPreset {
                    relativo: e.relativo.clone(),
                    nome: nome.to_owned(),
                    grupo: tr_with(
                        "app.painter.presets.group_in_media",
                        &[("engine", &motor), ("media", &meio), ("group", &grupo)],
                    ),
                    favorito: self.estado.favoritos.contains(&e.relativo),
                };
                (media.as_str() != agora, vista)
            })
            .collect();
        // Estável: dentro de cada metade, a ordem da biblioteca.
        entradas.sort_by_key(|(outro, _)| *outro);
        let politica = Politica::TODAS
            .iter()
            .position(|p| *p == self.estado.politica)
            .unwrap_or(0) as u8;
        self.vista = Arc::new(VistaPresets {
            entradas: entradas.into_iter().map(|(_, v)| v).collect(),
            ativo: self.aplicado.get(&self.meio.to_u8()).cloned(),
            politica,
            ilegiveis: self.bib.erros.len(),
        });
    }
}

/// A cada quadro com o Painter activo: abre a biblioteca na primeira vez, atende o pedido do painel, abre o
/// meio de agora (o preset lembrado dele) quando o artista troca de meio, e publica a vista.
pub(crate) fn drive(painter: &mut PainterTool, toasts: &mut ToastQueue) {
    PRESETS.with(|c| {
        let mut c = c.borrow_mut();
        let p = c.get_or_insert_with(|| Presets::abrir(painter, toasts));
        if let Some(pedido) = painter.take_wet_preset_request() {
            p.atender(painter, pedido, toasts);
        }
        if painter.paint_media() != p.meio {
            p.abrir_meio(painter, toasts);
            p.meio = painter.paint_media();
            p.gravar_estado(toasts);
            p.publicar();
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
