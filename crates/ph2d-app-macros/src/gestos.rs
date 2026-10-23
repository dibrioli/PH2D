//! **Os gestos da biblioteca** (W18, `docs/Macros/16_W18_plano.md` §2.5–2.7): guardar, aplicar,
//! renomear, apagar, exportar, importar, guardar como asset — e o que o painel mostra dela.
//! A pasta e o disco são de [`crate::biblioteca`]; aplicar é de [`crate::aplicar`].

use crate::aplicar::{self, Portas};
use crate::biblioteca::{self as bib, Biblioteca};
use crate::{Cena, Familias, Macros};
use ph2d_ecs::{Entity, Macro, MasterRoot, Name, SimWorld, Transform};
use ph2d_editor_core::macro_edits::{
    self as m, ErroDaMacroView, FamiliaDaMacro, MacroDaBibliotecaView, MacroPanelInfo,
    MacroRequest, OndeAplicar,
};
use ph2d_editor_core::macro_etapas::erros::{erro, erro_com};
use ph2d_editor_core::macro_tutorial::TranspoeDaFamilia;
use ph2d_macro::ficheiro::{EXTENSAO, Ficheiro};
use std::path::{Path, PathBuf};

/// A casa da biblioteca: a pasta (os testes injectam-na), a leitura em cache e o último aviso.
#[derive(Default)]
pub(crate) struct Casa {
    pasta: Option<PathBuf>,
    /// Sem pasta injectada, a do utilizador — só no `Macros` do app ([`Macros::do_utilizador`]): o
    /// `Default` não tem disco, e um teste que não injecta nunca lista a biblioteca do dono.
    do_utilizador: bool,
    lida: Option<(Biblioteca, Vec<MacroDaBibliotecaView>)>,
    aviso: Option<ErroDaMacroView>,
    /// As receitas a aplicar no próximo quadro: (bits da receita, onde nasce, sobre que objecto).
    receitas: Vec<(u64, Option<[f32; 2]>, Option<u64>)>,
}

fn do_disco(e: &bib::Erro) -> ErroDaMacroView {
    match e {
        bib::Erro::Disco(d) => erro_com(m::MACRO_LIB_DISCO, d.clone()),
        bib::Erro::Ficheiro(_) => erro(m::MACRO_LIB_ILEGIVEL),
        bib::Erro::NomeVazio => erro(m::MACRO_LIB_NOME_VAZIO),
        bib::Erro::JaExiste(n) => erro_com(m::MACRO_LIB_JA_EXISTE, n.clone()),
    }
}

/// Como o painel lista uma macro.
fn vista(f: &Ficheiro, nome: &str) -> MacroDaBibliotecaView {
    let painter = f
        .doc
        .tracks
        .iter()
        .find(|t| aplicar::familia(t) == FamiliaDaMacro::Painter);
    let tela = painter.and_then(|t| {
        ph2d_tool_painter::macro_track::inicial::tamanho_gravado(&t.initial).or_else(|| {
            ph2d_tool_painter::macro_track::inicial::tool_no_inicial(&t.initial)
                .ok()
                .map(|p| p.source_size())
        })
    });
    let fim = f
        .doc
        .tracks
        .iter()
        .filter_map(|t| t.events.last().map(|e| e.t.0))
        .max()
        .unwrap_or(0);
    #[expect(clippy::cast_precision_loss, reason = "µs de uma macro cabem num f64")]
    let duracao_s = fim as f64 / 1e6;
    MacroDaBibliotecaView {
        nome: nome.to_owned(),
        familias: f.doc.tracks.iter().map(aplicar::familia).collect(),
        duracao_s,
        tela,
    }
}

/// **A receita de um objecto Macro nos assets do projecto** (ADR-0164, plano §2.6): uma entidade com
/// `Name` + `Macro` + `MasterRoot` — o mestre de que o painel de Assets instancia. Devolve os bits.
///
/// # Errors
/// [`m::MACRO_LIB_SEM_GRAVACAO`].
pub fn guarda_como_asset(sim: &mut SimWorld, bits: u64) -> Result<u64, ErroDaMacroView> {
    let w = sim.world();
    let e = Entity::try_from_bits(bits).ok_or_else(|| erro(m::MACRO_LIB_SEM_GRAVACAO))?;
    let mac = w
        .get::<Macro>(e)
        .filter(|x| !x.doc.is_empty())
        .cloned()
        .ok_or_else(|| erro(m::MACRO_LIB_SEM_GRAVACAO))?;
    let nome = w
        .get::<Name>(e)
        .cloned()
        .unwrap_or_else(|| Name::new("Macro"));
    let r = sim
        .world_mut()
        .spawn((nome, mac, MasterRoot, Transform::IDENTITY))
        .id()
        .to_bits();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    Ok(r)
}

/// O objecto `bits` é uma receita de macro (um mestre com gravação) — o duplo-clique e o arrasto do
/// painel de Assets aplicam-na em vez de instanciar.
#[must_use]
pub fn e_receita(sim: &SimWorld, bits: u64) -> bool {
    Entity::try_from_bits(bits)
        .and_then(|e| sim.world().get_entity(e).ok())
        .is_some_and(|e| {
            e.contains::<MasterRoot>() && e.get::<Macro>().is_some_and(|x| !x.doc.is_empty())
        })
}

/// **O verbo de USAR do painel de Assets sobre uma receita de macro** (duplo-clique, arrasto): aplica-a
/// no próximo quadro — por cima de `sobre` se ele a recebe, senão nova em `em`. `false` = o `StableId`
/// `sid` não é uma receita de macro (o verbo segue para o instanciar de sempre).
pub fn aplica_se_receita(
    sim: &mut SimWorld,
    sid: u64,
    em: Option<[f32; 2]>,
    sobre: Option<u64>,
) -> bool {
    let Some(e) = ph2d_ecs::entity_of_stable_id(sim.world_mut(), ph2d_ecs::StableId(sid)) else {
        return false;
    };
    if !e_receita(sim, e.to_bits()) {
        return false;
    }
    crate::com(|m| m.aplica_receita(e.to_bits(), em, sobre));
    true
}

impl Macros {
    /// O `Macros` do processo do app: a biblioteca é a pasta do utilizador, resolvida no 1.º gesto.
    #[must_use]
    pub fn do_utilizador() -> Self {
        let mut m = Self::default();
        m.biblioteca.do_utilizador = true;
        m
    }

    /// A pasta da biblioteca (os testes e a cena de smoke apontam outra); relê no próximo pedido.
    pub fn com_pasta(&mut self, p: PathBuf) {
        self.biblioteca.pasta = Some(p);
        self.biblioteca.lida = None;
    }

    /// A pasta que a biblioteca lê e escreve (`None` = nenhuma: os gestos recusam, a lista é vazia).
    #[must_use]
    pub fn pasta_da_biblioteca(&self) -> Option<PathBuf> {
        self.pasta().ok()
    }

    fn pasta(&self) -> Result<PathBuf, ErroDaMacroView> {
        self.biblioteca
            .pasta
            .clone()
            .or_else(|| {
                self.biblioteca
                    .do_utilizador
                    .then(bib::pasta_do_utilizador)
                    .flatten()
            })
            .ok_or_else(|| erro(m::MACRO_LIB_SEM_PASTA))
    }

    /// A biblioteca lida (da cache; o disco só depois de um gesto).
    fn lida(&mut self) -> &(Biblioteca, Vec<MacroDaBibliotecaView>) {
        if self.biblioteca.lida.is_none() {
            let b = self
                .pasta()
                .map(|p| Biblioteca::carregar(&p))
                .unwrap_or_default();
            let v = b
                .entradas
                .iter()
                .map(|e| vista(&e.ficheiro, &e.nome))
                .collect();
            self.biblioteca.lida = Some((b, v));
        }
        self.biblioteca.lida.as_ref().expect("acabou de ler")
    }

    /// A frase do último gesto da biblioteca.
    #[must_use]
    pub fn aviso(&self) -> Option<&ErroDaMacroView> {
        self.biblioteca.aviso.as_ref()
    }

    fn ficheiro_da_biblioteca(&mut self, nome: &str) -> Result<Ficheiro, ErroDaMacroView> {
        self.lida()
            .0
            .procura(nome)
            .map(|e| e.ficheiro.clone())
            .ok_or_else(|| erro_com(m::MACRO_LIB_NAO_HA, nome.to_owned()))
    }

    /// **Exportar** a macro `nome` para `destino` (o caminho que o diálogo deu): provada a tocar até ao
    /// fim, a do Motion só com o grafo do dono. Devolve o caminho escrito.
    ///
    /// # Errors
    /// O porquê — nada se escreve.
    pub fn exporta_para(
        &mut self,
        nome: &str,
        destino: &Path,
        motion: &dyn TranspoeDaFamilia,
    ) -> Result<PathBuf, ErroDaMacroView> {
        let f = self.ficheiro_da_biblioteca(nome)?;
        let f = aplicar::para_exportar(&f, motion)?;
        bib::exporta(destino, &f).map_err(|e| do_disco(&e))
    }

    /// **Importar** o ficheiro `origem` para a biblioteca; devolve o nome com que ficou.
    ///
    /// # Errors
    /// O porquê.
    pub fn importa_de(&mut self, origem: &Path) -> Result<String, ErroDaMacroView> {
        let raiz = self.pasta()?;
        self.biblioteca.lida = None;
        bib::importa(&raiz, origem).map_err(|e| do_disco(&e))
    }

    /// **Guardar** a gravação do objecto Macro `bits` na biblioteca, com o nome dele (livre).
    ///
    /// # Errors
    /// O porquê.
    pub fn guarda(&mut self, sim: &SimWorld, bits: u64) -> Result<String, ErroDaMacroView> {
        let f = aplicar::ficheiro_de(sim, bits)?;
        let raiz = self.pasta()?;
        let nome = self.lida().0.nome_livre(&f.nome);
        self.biblioteca.lida = None;
        bib::grava(&raiz, &nome, &f).map_err(|e| do_disco(&e))?;
        Ok(nome)
    }

    /// **A receita `bits` do painel de Assets** (duplo-clique ou arrasto): aplica-se no próximo quadro,
    /// por cima de `sobre` se ele a recebe, senão nova em `em`.
    pub fn aplica_receita(&mut self, bits: u64, em: Option<[f32; 2]>, sobre: Option<u64>) {
        self.biblioteca.receitas.push((bits, em, sobre));
    }

    pub(crate) fn receitas(&mut self, f: &mut Familias<'_>, c: &mut Cena<'_>) {
        for (bits, em, sobre) in std::mem::take(&mut self.biblioteca.receitas) {
            let r = aplicar::ficheiro_de(c.sim, bits).and_then(|fi| {
                let por_cima = sobre.filter(|&alvo| {
                    f.portas.as_ref().is_some_and(|p| {
                        aplicar::compativel(c.sim, p.e_imagem, f.painter.as_deref(), &fi.doc, alvo)
                            .is_ok()
                    })
                });
                let onde = match por_cima {
                    Some(alvo) => OndeAplicar::PorCima { alvo },
                    None => OndeAplicar::Novo { em },
                };
                self.aplica_ficheiro(&fi, onde, f, c)
            });
            self.biblioteca.aviso = Some(r.err().unwrap_or_else(|| erro(m::MACRO_APLICADA)));
        }
    }

    fn aplica_ficheiro(
        &mut self,
        fi: &Ficheiro,
        onde: OndeAplicar,
        f: &mut Familias<'_>,
        c: &mut Cena<'_>,
    ) -> Result<u64, ErroDaMacroView> {
        if self.sessao.is_some() {
            return Err(erro(m::MACRO_RECUSA_JA_GRAVA));
        }
        let Some(mut p): Option<Portas<'_>> = f.portas.take() else {
            return Err(erro(m::MACRO_APLICAR_NAO_NASCE));
        };
        let r = aplicar::aplica(fi, onde, f, c, &mut p);
        f.portas = Some(p);
        r
    }

    /// Os pedidos da biblioteca (o resto do [`MacroRequest`] é de [`Macros::quadro`]).
    pub(crate) fn gesto(&mut self, r: MacroRequest, f: &mut Familias<'_>, c: &mut Cena<'_>) {
        let feito = |k: &'static str, d: String| erro_com(k, d);
        let aviso = match r {
            MacroRequest::Guarda { entity_bits } => self
                .guarda(c.sim, entity_bits)
                .map(|n| feito(m::MACRO_LIB_GUARDADA, n)),
            MacroRequest::Aplica { nome, onde } => self
                .ficheiro_da_biblioteca(&nome)
                .and_then(|fi| self.aplica_ficheiro(&fi, onde, f, c))
                .map(|_| feito(m::MACRO_APLICADA, nome)),
            MacroRequest::Renomeia { de, para } => self.pasta().and_then(|raiz| {
                self.biblioteca.lida = None;
                bib::renomeia(&raiz, &de, &para)
                    .map_err(|e| do_disco(&e))
                    .map(|_| feito(m::MACRO_LIB_RENOMEADA, bib::limpa(&para)))
            }),
            MacroRequest::ApagaDaBiblioteca { nome } => self.pasta().and_then(|raiz| {
                self.biblioteca.lida = None;
                bib::apaga(&raiz, &nome)
                    .map_err(|e| do_disco(&e))
                    .map(|()| feito(m::MACRO_LIB_APAGADA, nome))
            }),
            MacroRequest::Exporta { nome } => {
                let dialogo = rfd::FileDialog::new()
                    .add_filter(ph2d_i18n::tr("app.macros.lib.file_kind"), &[&EXTENSAO[1..]])
                    .set_file_name(format!("{nome}{EXTENSAO}"));
                let Some(destino) = ph2d_app_host::modal::save_file(dialogo) else {
                    return;
                };
                self.exporta_para(&nome, &destino, &*f.motion)
                    .map(|p| feito(m::MACRO_LIB_EXPORTADA, p.display().to_string()))
            }
            MacroRequest::Importa => {
                let dialogo = rfd::FileDialog::new()
                    .add_filter(ph2d_i18n::tr("app.macros.lib.file_kind"), &[&EXTENSAO[1..]]);
                let Some(origem) = ph2d_app_host::modal::pick_file(dialogo) else {
                    return;
                };
                self.importa_de(&origem)
                    .map(|n| feito(m::MACRO_LIB_IMPORTADA, n))
            }
            MacroRequest::GuardaComoAsset { entity_bits } => guarda_como_asset(c.sim, entity_bits)
                .map(|b| {
                    let n = c
                        .sim
                        .world()
                        .get::<Name>(Entity::from_bits(b))
                        .map(|n| n.0.clone())
                        .unwrap_or_default();
                    feito(m::MACRO_ASSET_GUARDADO, n)
                }),
            // Os outros são do quadro.
            _ => return,
        };
        self.biblioteca.aviso = Some(aviso.unwrap_or_else(|e| e));
    }

    /// A parte da biblioteca do instantâneo do painel.
    pub(crate) fn info_da_biblioteca(
        &mut self,
        f: &Familias<'_>,
        sim: &SimWorld,
        escolhido: Option<u64>,
        macro_do_escolhido: Option<u64>,
    ) -> MacroPanelInfo {
        let (b, v) = self.lida();
        let (biblioteca, biblioteca_ilegiveis) = (v.clone(), b.erros.len());
        let pode_guardar = macro_do_escolhido.is_some_and(|x| {
            sim.world()
                .get::<Macro>(Entity::from_bits(x))
                .is_some_and(|m| !m.doc.is_empty())
        });
        // «Por cima de»: uma imagem ou um objecto Motion escolhido (a família confere-se ao aplicar).
        let alvo = escolhido.filter(|&b| {
            let w = sim.world();
            Entity::try_from_bits(b)
                .and_then(|e| w.get_entity(e).ok())
                .is_some_and(|e| {
                    e.contains::<ph2d_ecs::MotionGenerator>()
                        || f.portas.as_ref().is_some_and(|p| (p.e_imagem)(sim, b))
                        || f.painter
                            .as_deref()
                            .is_some_and(|p| p.document_entities().contains(&b))
                })
        });
        MacroPanelInfo {
            biblioteca,
            biblioteca_ilegiveis,
            pode_guardar,
            por_cima_de: alvo.map(|b| {
                sim.world()
                    .get::<Name>(Entity::from_bits(b))
                    .map(|n| n.0.clone())
                    .unwrap_or_default()
            }),
            por_cima_alvo: alvo,
            por_cima_recusa: alvo.is_none().then_some(m::MACRO_APLICAR_INCOMPATIVEL),
            aviso: self.biblioteca.aviso.clone(),
            ..MacroPanelInfo::default()
        }
    }
}
