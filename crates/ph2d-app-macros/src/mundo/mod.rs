//! **A família «mundo»** (W19, `docs/Macros/17_W19_plano.md`): «tudo pode ser macro» — os componentes, os
//! objectos e «Simular» dentro do objecto Macro, como diferença de linhas do undo e corridas do rapier.
//!
//! - [`grava`]: o cru, enquanto se grava.
//! - [`fecha`]: ao Parar, as ids locais, o dono de cada objecto e as [`portas`] de cada componente.
//! - [`formato`]: os bytes da trilha.

pub mod aplica;
/// A sprite que não traz os seus píxeis (uma célula do atlas) grava-se com eles embutidos.
mod embute;
pub mod estado;
pub mod formato;
pub mod grava;
/// A reprodução: `S(t)` conduzido pelo preview, e «Simular» recozido.
pub mod palco;
pub mod palco_fisica;
pub mod portas;

pub use embute::Leitores;

use formato::{Componente, Evento, Inicial, Linha, Mudanca, Objeto, Quem, Recurso};
use grava::{Corrida, Cru, Gravacao};
use ph2d_ecs::scene::{ComponentRegistry, EntitySnapshotRow, LinhaMudada};
use ph2d_editor_core::macro_edits::FamiliaDaMacro;
use ph2d_macro::{MacroDoc, Track};
use portas::Porta;
use std::collections::{BTreeMap, BTreeSet};

/// O nome canónico do componente dos píxeis de uma imagem importada (o recurso que se embute).
const PIXEIS: &str = "ph2d::ecs::SpritePixels";

/// **Os píxeis em alfa pré-multiplicado** do mundo (pelo id de conteúdo): as sprites com
/// `SpritePixels` e `Sprite::premultiplied` — a única cópia viva da bandeira (o `AssetDb` não a tem).
#[must_use]
pub fn premultiplicados(sim: &mut ph2d_ecs::SimWorld) -> BTreeSet<[u8; 32]> {
    let w = sim.world_mut();
    let mut q = w.query::<(&ph2d_ecs::SpritePixels, &ph2d_render::Sprite)>();
    q.iter(w)
        .filter(|(_, s)| s.premultiplied)
        .map(|(p, _)| *p.0.as_bytes())
        .collect()
}

/// O nome canónico do `Name`.
const NOME: &str = "ph2d::ecs::Name";

/// O que o painel diz ao Parar.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Avisos {
    /// Os componentes que não se gravaram (pelo nome canónico).
    pub recusados: BTreeSet<String>,
    /// Os objectos que nasceram ou estavam na macro e foram apagados a gravar (o `Name` que tinham):
    /// no fim não existem, a reprodução não tem onde os mostrar.
    pub apagados: Vec<String>,
}

/// A trilha do mundo, pronta a abrir: o inicial e os passos com o instante do chamador.
pub struct Fecho {
    pub inicial: Vec<u8>,
    pub eventos: Vec<(u64, Vec<u8>)>,
    pub avisos: Avisos,
}

/// Os objectos desta família que não estão vivos no fim, pelo nome (`#id` sem `Name`).
fn apagados(inicial: &Inicial, eventos: &[(u64, Evento)]) -> Vec<String> {
    let lida = estado::Lida {
        inicial: inicial.clone(),
        eventos: eventos
            .iter()
            .map(|(us, e)| (ph2d_macro::MacroTime(*us), e.clone()))
            .collect(),
    };
    lida.no_fim()
        .into_iter()
        .filter(|(_, o)| o.quem == Quem::Mundo && !o.vivo)
        .map(|(id, o)| {
            o.componentes
                .get(NOME)
                .and_then(|b| postcard::from_bytes::<ph2d_ecs::Name>(b).ok())
                .map(|n| n.0)
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| format!("#{id}"))
        })
        .collect()
}

/// De quem é cada objecto que outra família faz nascer: o `StableId` → (índice da trilha, família).
fn donos(
    doc: &MacroDoc,
    dono: u64,
    familia: &dyn Fn(&Track) -> Option<FamiliaDaMacro>,
) -> BTreeMap<u64, (Quem, Option<FamiliaDaMacro>)> {
    let mut m = BTreeMap::new();
    m.insert(dono, (Quem::Macro, None));
    for (k, t) in doc.tracks.iter().enumerate() {
        let Some(fam) = familia(t) else { continue };
        if t.continua.is_some() {
            continue;
        }
        let k = u32::try_from(k).expect("as trilhas cabem num u32");
        match t.nasce {
            Some(_) => {
                m.insert(t.target, (Quem::Trilha(k), Some(fam)));
            }
            // Sem ficha: o objecto Macro É o objecto da trilha (uma sprite com `Macro`).
            None => {
                m.insert(dono, (Quem::Macro, Some(fam)));
            }
        }
    }
    m
}

struct Tradutor<'a> {
    dono: u64,
    registo: &'a ComponentRegistry,
    donos: BTreeMap<u64, (Quem, Option<FamiliaDaMacro>)>,
    locais: BTreeMap<u64, u32>,
    novos: Vec<Objeto>,
    recursos_vistos: BTreeSet<[u8; 32]>,
    recursos: Vec<Recurso>,
    l: Leitores<'a>,
    /// Os objectos (`StableId`) cujos píxeis já são um `SpritePixels` gravado.
    com_pixeis: BTreeSet<u64>,
    /// O id embutido dos píxeis da sprite de cada objecto (`None` = não se leram).
    sprites: BTreeMap<u64, Option<[u8; 32]>>,
    recusados: BTreeSet<String>,
}

impl Tradutor<'_> {
    fn local(&mut self, sid: u64) -> u32 {
        if let Some(&i) = self.locais.get(&sid) {
            return i;
        }
        let i = u32::try_from(self.locais.len()).expect("os objectos cabem num u32");
        self.locais.insert(sid, i);
        let quem = self.donos.get(&sid).map_or(Quem::Mundo, |d| d.0);
        self.novos.push(Objeto {
            id: i,
            original: sid,
            quem,
        });
        i
    }

    fn pai(&mut self, p: Option<ph2d_ecs::StableId>) -> Option<u32> {
        p.filter(|p| p.0 != self.dono).map(|p| self.local(p.0))
    }

    /// O componente `tipo` do objecto `sid` grava-se? O nome dele, se sim.
    fn grava(&mut self, sid: u64, tipo: u64) -> Option<&'static str> {
        let Some(e) = self.registo.get_by_id(tipo) else {
            self.recusados.insert(format!("#{tipo:016x}"));
            return None;
        };
        let nome = e.canonical_name;
        let familia = self.donos.get(&sid).and_then(|d| d.1);
        match portas::porta(nome) {
            Some(Porta::Gravado) => {}
            Some(Porta::DaFamilia(f)) if familia == Some(f) => return None,
            Some(Porta::DaFamilia(_)) => {}
            Some(Porta::SoDaFamilia(f)) if familia == Some(f) => return None,
            Some(Porta::SoDaFamilia(_)) => {
                self.recusados.insert(nome.to_owned());
                return None;
            }
            Some(Porta::DoAplicar) => return None,
            Some(Porta::ForaDaRaiz) if sid == self.dono => return None,
            Some(Porta::ForaDaRaiz) => {}
            Some(Porta::Recusado(_)) | None => {
                self.recusados.insert(nome.to_owned());
                return None;
            }
        }
        Some(nome)
    }

    /// Os bytes de um componente gravado; um `SpritePixels` leva o recurso (uma vez por id).
    fn bytes(&mut self, nome: &str, b: &[u8]) -> bool {
        if nome != PIXEIS {
            return true;
        }
        let Ok(px) = postcard::from_bytes::<ph2d_ecs::SpritePixels>(b) else {
            self.recusados.insert(nome.to_owned());
            return false;
        };
        let id = *px.0.as_bytes();
        if self.recursos_vistos.contains(&id) {
            return true;
        }
        match (self.l.recurso)(&id) {
            Some(bytes) => {
                self.recursos_vistos.insert(id);
                self.recursos.push(Recurso {
                    id,
                    bytes,
                    premultiplicado: (self.l.premultiplicado)(&id),
                });
                true
            }
            None => {
                self.recusados.insert(nome.to_owned());
                false
            }
        }
    }

    fn linha(&mut self, r: &EntitySnapshotRow) -> Linha {
        let id = self.local(r.id.0);
        let pai = self.pai(r.parent);
        let mut componentes = Vec::new();
        for c in &r.components {
            if let Some(nome) = self.grava(r.id.0, c.type_id)
                && !self.pixeis_velhos(r.id.0, nome)
                && self.bytes(nome, &c.data)
            {
                componentes.push(Componente {
                    nome: nome.to_owned(),
                    bytes: c.data.clone(),
                });
            }
        }
        self.sprite_da_linha(r.id.0, &mut componentes);
        Linha {
            id,
            pai,
            componentes,
        }
    }

    fn mudanca(&mut self, l: &LinhaMudada) -> Option<Mudanca> {
        match l {
            LinhaMudada::Nasce(r) => Some(Mudanca::Nasce(self.linha(r))),
            LinhaMudada::Morre(s) => Some(Mudanca::Morre(self.local(s.0))),
            LinhaMudada::Muda {
                id,
                pai,
                componentes,
            } => {
                let i = self.local(id.0);
                let pai = pai.map(|p| self.pai(p));
                let mut cs = Vec::new();
                for (tipo, b) in componentes {
                    let Some(nome) = self
                        .grava(id.0, *tipo)
                        .filter(|n| !self.pixeis_velhos(id.0, n))
                    else {
                        continue;
                    };
                    if b.as_deref().is_none_or(|b| self.bytes(nome, b)) {
                        cs.push((nome.to_owned(), b.clone()));
                    }
                }
                self.sprite_da_mudanca(id.0, &mut cs);
                (pai.is_some() || !cs.is_empty()).then_some(Mudanca::Muda {
                    id: i,
                    pai,
                    componentes: cs,
                })
            }
        }
    }
}

/// **Os passos que a corrida deixa ao parar** (a pose do solver vira documento, lei do
/// `ph2d-preview-drive`; a captura do undo pode vir um tique atrasada e acertar no passo seguinte) não
/// são autoria: o `Transform` de um corpo da última corrida igual a uma pose que ela percorreu sai do
/// passo; os outros componentes e objectos ficam.
fn sem_a_pose_da_corrida(
    d: &[LinhaMudada],
    c: Option<&Corrida>,
    registo: &ComponentRegistry,
) -> Vec<LinhaMudada> {
    let Some(c) = c else {
        return d.to_vec();
    };
    let e_o_fim = |sid: u64, tipo: u64, b: &Option<Vec<u8>>| {
        let transform = registo
            .get_by_id(tipo)
            .is_some_and(|e| e.canonical_name == palco::TRANSFORM);
        let t = b
            .as_deref()
            .and_then(|b| postcard::from_bytes::<ph2d_ecs::Transform>(b).ok());
        transform
            && t.is_some_and(|t| {
                let bits = [t.translation.x, t.translation.y, t.rotation].map(f32::to_bits);
                c.percorridas.contains(&(sid, bits))
            })
    };
    d.iter()
        .filter_map(|l| match l {
            LinhaMudada::Muda {
                id,
                pai,
                componentes,
            } => {
                let cs: Vec<_> = componentes
                    .iter()
                    .filter(|(tipo, b)| !e_o_fim(id.0, *tipo, b))
                    .cloned()
                    .collect();
                (pai.is_some() || !cs.is_empty()).then_some(LinhaMudada::Muda {
                    id: *id,
                    pai: *pai,
                    componentes: cs,
                })
            }
            outra => Some(outra.clone()),
        })
        .collect()
}

/// **Ao Parar**: a gravação `g` passa a trilha, com as outras trilhas de `doc` já abertas (elas dizem
/// quem faz nascer cada objecto), com os [`Leitores`] da composição. `None` = nada do mundo mudou.
#[must_use]
pub fn fecha<'a>(
    g: Gravacao,
    doc: &MacroDoc,
    familia: &dyn Fn(&Track) -> Option<FamiliaDaMacro>,
    registo: &'a ComponentRegistry,
    l: Leitores<'a>,
) -> Option<Fecho> {
    let g = g.acaba();
    if g.passos.is_empty() {
        return None;
    }
    let mut t = Tradutor {
        dono: g.dono,
        registo,
        donos: donos(doc, g.dono, familia),
        locais: BTreeMap::new(),
        novos: Vec::new(),
        recursos_vistos: BTreeSet::new(),
        recursos: Vec::new(),
        l,
        com_pixeis: BTreeSet::new(),
        sprites: BTreeMap::new(),
        recusados: BTreeSet::new(),
    };
    t.local(g.dono);
    let mut inicial = Inicial::default();
    for r in g.inicio.iter().flat_map(|i| i.values()) {
        let l = t.linha(r);
        inicial.linhas.push(l);
    }
    inicial.objetos = std::mem::take(&mut t.novos);
    inicial.recursos = std::mem::take(&mut t.recursos);
    let mut eventos = Vec::new();
    let mut corrida: Option<&Corrida> = None;
    for (us, cru) in &g.passos {
        let ev = match cru {
            Cru::Passo(d) => {
                let d = sem_a_pose_da_corrida(d, corrida, registo);
                let mudancas: Vec<_> = d.iter().filter_map(|l| t.mudanca(l)).collect();
                if mudancas.is_empty() {
                    continue; // os objectos novos vão com o passo seguinte
                }
                Evento::Passo {
                    novos: std::mem::take(&mut t.novos),
                    mudancas,
                    recursos: std::mem::take(&mut t.recursos),
                }
            }
            Cru::Corrida(c) => {
                corrida = Some(c);
                let locais = t.locais.clone();
                Evento::Simula(c.simula(&|s| locais.get(&s).copied()))
            }
        };
        eventos.push((*us, ev));
    }
    if eventos.is_empty() && inicial.linhas.iter().all(|l| l.componentes.is_empty()) {
        return None;
    }
    Some(Fecho {
        avisos: Avisos {
            recusados: t.recusados,
            apagados: apagados(&inicial, &eventos),
        },
        inicial: inicial.escreve(),
        eventos: eventos
            .into_iter()
            .map(|(us, e)| (us, e.escreve()))
            .collect(),
    })
}

/// **O aviso do Parar**: os componentes que não se gravaram (nomes curtos: `Collider`, não o caminho
/// canónico) e os objectos apagados a gravar — cada lista com a sua frase; as duas, uma após a outra.
#[must_use]
pub fn aviso(a: &Avisos) -> Option<ph2d_editor_core::macro_edits::ErroDaMacroView> {
    use ph2d_editor_core::macro_edits as m;
    use ph2d_editor_core::macro_etapas::erros::erro_com;
    let nomes: Vec<&str> = a
        .recusados
        .iter()
        .map(|n| n.rsplit("::").next().unwrap_or(n))
        .collect();
    let recusados =
        (!nomes.is_empty()).then(|| erro_com(m::MACRO_MUNDO_NAO_GRAVOU, nomes.join(", ")));
    let apagados =
        (!a.apagados.is_empty()).then(|| erro_com(m::MACRO_MUNDO_APAGADOS, a.apagados.join(", ")));
    match (recusados, apagados) {
        (Some(r), Some(p)) => {
            let frase = |e: &m::ErroDaMacroView| {
                let d = e.detalhe.clone().unwrap_or_default();
                ph2d_i18n::tr_with(e.chave, &[("detail", &d)])
            };
            Some(erro_com(
                m::MACRO_MUNDO_DOIS_AVISOS,
                format!("{} {}", frase(&r), frase(&p)),
            ))
        }
        (r, p) => r.or(p),
    }
}
