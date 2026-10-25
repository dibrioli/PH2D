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
//! A biblioteca abre com o Painter, em qualquer meio; o navegador lista os quatro por categoria, o meio de
//! agora primeiro dentro de cada uma.
//! Escolher um preset de outro meio TROCA o meio (`preset_aplicar`); um meio desconhecido é recusado com
//! aviso. O preset lembrado de cada meio aplica-se UMA vez por sessão, no 1.º quadro em que esse meio está
//! activo (o comportamento de sempre do Wet Paint, agora em cada meio). A ordem, apagar, exportar e
//! importar: [`biblioteca`].

use crate::painter_bridge_categorias::{categoria_da_pasta, rotulo_da_categoria};
use biblioteca::Seletor;
use ph2d_brush_presets::{
    Biblioteca, Entrada, EstadoDoUtilizador, Origem, Parametros, Politica, Preset, apagar,
    caminho_relativo, gravar, pasta_do_utilizador,
};
use ph2d_editor_core::toast::{Toast, ToastQueue};
use ph2d_i18n::{idioma, tr, tr_with};
use ph2d_panel_painter_layers::{
    VistaPreset, VistaPresets, set_current_mao_do_preset, set_current_wet_presets,
};
use ph2d_tool_painter::{
    Gaveta, MeioDesconhecido, PaintMedia, PainterTool, PedidoPreset, ferramenta_do_preset,
    miniatura_do_preset, motor_do_meio, nome_do_meio,
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
    /// O preset que está NO PINCEL de cada gaveta (ferramenta × meio, aplicado por esta ponte), pela chave dela.
    aplicado: BTreeMap<u8, String>,
    /// As gavetas que já abriram nesta sessão — o lembrado de cada uma aplica-se uma vez.
    abertos: BTreeSet<u8>,
    /// A gaveta que a vista publicada mostra (`None` = a ferramenta não tem presets).
    gaveta: Option<Gaveta>,
    vista: Arc<VistaPresets>,
    seletor: Seletor,
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
            gaveta: painter.gaveta_de_presets(),
            vista: Arc::default(),
            seletor: Seletor::default(),
        };
        p.dizer_ilegiveis(toasts);
        p.abrir_meio(painter, toasts);
        p.publicar();
        p
    }

    /// O 1.º quadro da gaveta de agora nesta sessão: o pincel abre com o preset que o artista tinha NELA.
    fn abrir_meio(&mut self, painter: &mut PainterTool, toasts: &mut ToastQueue) {
        let Some(g) = painter.gaveta_de_presets() else {
            return;
        };
        if !self.abertos.insert(g.chave()) {
            return;
        }
        let nome = nome_do_meio(g.meio);
        let lembrado = self
            .estado
            .ativo_por_meio
            .get(&g.nome())
            .cloned()
            .or_else(|| {
                // Um `estado.json` de antes da L1 só tem o activo de todos (era sempre do Wet Paint).
                self.estado.ativo.clone().filter(|r| {
                    self.bib.procurar(r).is_some_and(|e| {
                        e.preset.parametros.media == nome
                            && ferramenta_do_preset(&e.preset.parametros) == g.ferramenta
                    })
                })
            });
        // Só um preset DESTA ferramenta (um lembrado de antes das gavetas pode ser a borracha antiga no Pincel).
        if let Some(rel) = lembrado.filter(|r| {
            self.bib
                .procurar(r)
                .is_some_and(|e| ferramenta_do_preset(&e.preset.parametros) == g.ferramenta)
        }) {
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

    /// O preset no pincel da gaveta de agora do `painter`.
    fn ativo(&self, painter: &PainterTool) -> Option<String> {
        self.aplicado
            .get(&painter.gaveta_de_presets()?.chave())
            .cloned()
    }

    /// O preset `rel` passa a ser o da gaveta de agora (e o lembrado dela).
    fn usar(&mut self, painter: &PainterTool, rel: &str) {
        if let Some(g) = painter.gaveta_de_presets() {
            self.aplicado.insert(g.chave(), rel.to_owned());
            self.abertos.insert(g.chave());
            self.estado.usar_no_meio(&g.nome(), rel);
        }
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
        self.usar(painter, rel);
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
                    self.usar(painter, &rel);
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
            PedidoPreset::Importar => self.importar(painter, toasts),
            PedidoPreset::Exportar => self.exportar(painter, toasts),
            PedidoPreset::Apagar => self.apagar_ativo(painter, toasts),
            PedidoPreset::ReporPosicoes => self.estado.ordem.clear(),
            PedidoPreset::Mover {
                arrastado,
                alvo,
                depois,
            } => self.mover(&arrastado, &alvo, depois),
        }
        self.gravar_estado(toasts);
        self.gaveta = painter.gaveta_de_presets();
        self.publicar();
    }

    /// A vista do navegador, pela ordem de [`Self::ordem_do_navegador`], só com os presets da FERRAMENTA de agora
    /// (dono 24/10); o activo é o da gaveta de agora.
    fn publicar(&mut self) {
        let ferramenta = self.gaveta.map(|g| g.ferramenta);
        let entradas = self
            .ordem_do_navegador()
            .into_iter()
            .filter(|e| Some(ferramenta_do_preset(&e.preset.parametros)) == ferramenta)
            .map(|e| VistaPreset {
                relativo: e.relativo.clone(),
                nome: e.partes()[3].to_owned(),
                categoria: rotulo_da_categoria(idioma(), categoria_da_pasta(e.partes()[1]))
                    .to_owned(),
                favorito: self.estado.favoritos.contains(&e.relativo),
                apagavel: e.origem == Origem::Utilizador && !e.tem_fabrica,
            })
            .collect();
        let politica = Politica::TODAS
            .iter()
            .position(|p| *p == self.estado.politica)
            .unwrap_or(0) as u8;
        self.vista = Arc::new(VistaPresets {
            entradas,
            ativo: self
                .gaveta
                .and_then(|g| self.aplicado.get(&g.chave()).cloned()),
            politica,
            ilegiveis: self.bib.erros.len(),
            ordem_manual: !self.estado.ordem.is_empty(),
        });
    }
}

/// A cada quadro com o Painter activo: abre a biblioteca na primeira vez, atende o pedido do painel, abre o
/// meio de agora (o preset lembrado dele) quando o artista troca de meio, e publica a vista.
pub(crate) fn drive(painter: &mut PainterTool, toasts: &mut ToastQueue) {
    PRESETS.with(|c| {
        let mut c = c.borrow_mut();
        let p = c.get_or_insert_with(|| Presets::abrir(painter, toasts));
        let mut mudou = false;
        if let Some(pedido) = painter.take_wet_preset_request() {
            p.atender(painter, pedido, toasts);
            mudou = true;
        }
        if painter.gaveta_de_presets() != p.gaveta {
            p.abrir_meio(painter, toasts);
            p.gaveta = painter.gaveta_de_presets();
            p.gravar_estado(toasts);
            p.publicar();
            mudou = true;
        }
        if mudou {
            p.lembrar_a_mao_do_ativo(painter);
        }
        set_current_wet_presets(Some(p.vista.clone()));
        set_current_mao_do_preset(painter.mao_do_preset_disponivel());
    });
}

impl Presets {
    /// **A mão do preset activo vai para o tool** — o alvo do «Reset to brush preset» do cartão Pen. Aplicar já a
    /// lembra; isto cobre o activo que muda SEM ser aplicado (gravado por cima, apagado, outra gaveta).
    fn lembrar_a_mao_do_ativo(&self, painter: &mut PainterTool) {
        let ativo = self.ativo(painter);
        let e = ativo.as_deref().and_then(|r| self.bib.procurar(r));
        painter.set_mao_do_preset(e.map(|e| &e.preset.parametros));
    }
}

#[path = "painter_bridge_presets_biblioteca.rs"]
mod biblioteca;
#[cfg(test)]
#[path = "painter_bridge_presets_biblioteca_tests.rs"]
mod biblioteca_tests;
#[cfg(test)]
#[path = "painter_bridge_presets_foto_tests.rs"]
mod foto_tests;
#[cfg(test)]
#[path = "painter_bridge_presets_tests.rs"]
mod tests;
