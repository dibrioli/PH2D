//! **A família «mundo»** (W19, `docs/Macros/17_W19_plano.md`): «tudo pode ser macro» — os componentes, os
//! objectos e «Simular» dentro do objecto Macro, como diferença de linhas do undo e corridas do rapier.
//!
//! - [`grava`]: o cru, enquanto se grava.
//! - [`fecha`]: ao Parar, as ids locais, o dono de cada objecto e as [`portas`] de cada componente.
//! - [`formato`]: os bytes da trilha.

pub mod aplica;
pub mod estado;
pub mod formato;
pub mod grava;
/// A reprodução: `S(t)` conduzido pelo preview, e «Simular» recozido.
pub mod palco;
pub mod palco_fisica;
pub mod portas;

use formato::{Componente, Evento, Inicial, Linha, Mudanca, Objeto, Quem, Recurso};
use grava::{Cru, Gravacao};
use ph2d_ecs::scene::{ComponentRegistry, EntitySnapshotRow, LinhaMudada};
use ph2d_editor_core::macro_edits::FamiliaDaMacro;
use ph2d_macro::{MacroDoc, Track};
use portas::Porta;
use std::collections::{BTreeMap, BTreeSet};

/// O nome canónico do componente dos píxeis de uma imagem importada (o recurso que se embute).
const PIXEIS: &str = "ph2d::ecs::SpritePixels";

/// A trilha do mundo, pronta a abrir: o inicial e os passos com o instante do chamador.
pub struct Fecho {
    pub inicial: Vec<u8>,
    pub eventos: Vec<(u64, Vec<u8>)>,
    /// Os componentes que não se gravaram (pelo nome canónico), para o painel dizer quais.
    pub recusados: BTreeSet<String>,
}

/// De quem é cada objecto que outra família faz nascer: o `StableId` → (índice da trilha, família).
fn donos(doc: &MacroDoc, dono: u64, familia: &dyn Fn(&Track) -> Option<FamiliaDaMacro>) -> BTreeMap<u64, (Quem, Option<FamiliaDaMacro>)> {
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
    recurso: &'a dyn Fn(&[u8; 32]) -> Option<Vec<u8>>,
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
        match (self.recurso)(&id) {
            Some(bytes) => {
                self.recursos_vistos.insert(id);
                self.recursos.push(Recurso { id, bytes });
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
                && self.bytes(nome, &c.data)
            {
                componentes.push(Componente {
                    nome: nome.to_owned(),
                    bytes: c.data.clone(),
                });
            }
        }
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
                    let Some(nome) = self.grava(id.0, *tipo) else {
                        continue;
                    };
                    if b.as_deref().is_none_or(|b| self.bytes(nome, b)) {
                        cs.push((nome.to_owned(), b.clone()));
                    }
                }
                (pai.is_some() || !cs.is_empty()).then_some(Mudanca::Muda {
                    id: i,
                    pai,
                    componentes: cs,
                })
            }
        }
    }
}

/// **Ao Parar**: a gravação `g` passa a trilha, com as outras trilhas de `doc` já abertas (elas dizem
/// quem faz nascer cada objecto). `None` = nada do mundo mudou.
#[must_use]
pub fn fecha(
    g: Gravacao,
    doc: &MacroDoc,
    familia: &dyn Fn(&Track) -> Option<FamiliaDaMacro>,
    registo: &ComponentRegistry,
    recurso: &dyn Fn(&[u8; 32]) -> Option<Vec<u8>>,
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
        recurso,
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
    for (us, cru) in &g.passos {
        let ev = match cru {
            Cru::Passo(d) => {
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
                let locais = t.locais.clone();
                Evento::Simula(c.simula(&|s| locais.get(&s).copied()))
            }
        };
        eventos.push((*us, ev.escreve()));
    }
    if eventos.is_empty() && inicial.linhas.iter().all(|l| l.componentes.is_empty()) {
        return None;
    }
    Some(Fecho {
        inicial: inicial.escreve(),
        eventos,
        recusados: t.recusados,
    })
}

/// **O aviso do que não se gravou**: a chave e os nomes curtos (`Collider`, não o caminho canónico).
#[must_use]
pub fn aviso(recusados: &BTreeSet<String>) -> Option<ph2d_editor_core::macro_edits::ErroDaMacroView> {
    if recusados.is_empty() {
        return None;
    }
    let nomes: Vec<&str> = recusados
        .iter()
        .map(|n| n.rsplit("::").next().unwrap_or(n))
        .collect();
    Some(ph2d_editor_core::macro_etapas::erros::erro_com(
        ph2d_editor_core::macro_edits::MACRO_MUNDO_NAO_GRAVOU,
        nomes.join(", "),
    ))
}
