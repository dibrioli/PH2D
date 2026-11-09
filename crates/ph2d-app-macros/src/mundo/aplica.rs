//! **Aplicar a trilha do mundo** (W19, `docs/Macros/17_W19_plano.md` §2.4): o FIM da gravação nasce no
//! projecto de destino, num passo só com as outras famílias ([`crate::aplicar`]).
//!
//! - Quem faz nascer cada objecto local: `Macro` = o objecto Macro nascido; `Trilha(k)` = o objecto que o
//!   aplicar fez nascer para a trilha `k` (por cima: o alvo); `Mundo` = nasce aqui, dentro do pai local.
//! - Uma imagem importada (`SpritePixels` + `Sprite`) nasce pela porta de produto com os píxeis do
//!   recurso embutido (PNG); o `Sprite` gravado guarda tudo menos a fonte (o atlas é do destino).
//! - As referências entre objectos (`RefKind::Object`) reescrevem-se `original → StableId novo` pelo
//!   remapeador da composição, no blob ANTES de entrar no mundo — e na trilha instalada também (a
//!   reprodução escreve esses blobs), com cada `Objeto.original` já no `StableId` novo.

use super::estado::{Lida, ObjetoEm};
use super::formato::{Evento, Linha, Mudanca, Objeto, Quem, Recurso};
use crate::aplicar::{Imagem, Portas};
use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::{ChildOf, Entity, SimWorld, StableId, Transform, World};
use ph2d_editor_core::macro_edits::{self as m, ErroDaMacroView, FamiliaDaMacro};
use ph2d_editor_core::macro_etapas::erros::{erro, erro_com};
use ph2d_macro::{MacroDoc, Track};
use std::collections::{BTreeMap, BTreeSet};

const SPRITE: &str = "ph2d::render::Sprite";
const NOME: &str = "ph2d::ecs::Name";
const TRANSFORM: &str = "ph2d::ecs::Transform";

/// Onde a trilha do mundo cai.
pub(crate) struct Destino<'a> {
    /// O objecto Macro nascido (bits).
    pub raiz: u64,
    /// O objecto de cada trilha das outras famílias, pelo índice (bits).
    pub trilhas: &'a BTreeMap<usize, u64>,
    /// Por cima: o alvo guarda o nome e a pose dele.
    pub por_cima: bool,
}

/// **Os bytes de um recurso embutido**: a imagem em PNG (o que a composição dá à porta `recurso`).
#[must_use]
pub fn png(w: u32, h: u32, rgba: &[u8]) -> Option<Vec<u8>> {
    let img = image::RgbaImage::from_raw(w, h, rgba.to_vec())?;
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).ok()?;
    Some(out.into_inner())
}

fn de_png(b: &[u8]) -> Option<crate::aplicar::Rgba> {
    let i = image::load_from_memory_with_format(b, image::ImageFormat::Png)
        .ok()?
        .to_rgba8();
    let (w, h) = i.dimensions();
    Some((i.into_raw(), w, h))
}

fn curto(nome: &str) -> String {
    nome.rsplit("::").next().unwrap_or(nome).to_owned()
}

fn desconhecido(nome: &str) -> ErroDaMacroView {
    erro_com(m::MACRO_MUNDO_COMPONENTE_DESCONHECIDO, curto(nome))
}

fn nao_nasce(nome: &str, e: &dyn std::fmt::Display) -> ErroDaMacroView {
    erro_com(m::MACRO_APLICAR_NAO_NASCE, format!("{}: {e}", curto(nome)))
}

fn ilegivel() -> ErroDaMacroView {
    erro(m::MACRO_LIB_ILEGIVEL)
}

/// **Exportar**: o inicial e cada evento leem-se.
///
/// # Errors
/// [`m::MACRO_LIB_ILEGIVEL`].
pub fn confere(t: &Track) -> Result<(), ErroDaMacroView> {
    Lida::de(t).map(|_| ()).map_err(|_| ilegivel())
}

/// Um mundo de rascunho onde os blobs entram para o remapeador os reescrever.
struct Rascunho<'a> {
    w: World,
    registo: &'a ComponentRegistry,
    remapeia: &'a crate::aplicar::Remapeia,
    mapa: BTreeMap<u64, u64>,
}

impl Rascunho<'_> {
    /// O blob `b` do componente `nome` com as referências no projecto de destino.
    fn blob(&mut self, nome: &str, b: &[u8]) -> Result<Vec<u8>, ErroDaMacroView> {
        let e = self
            .registo
            .get_by_name(nome)
            .ok_or_else(|| desconhecido(nome))?;
        let (insere, serializa) = (e.insert_from_bytes, e.serialize);
        let x = self.w.spawn_empty().id();
        insere(&mut self.w, x, b).map_err(|er| nao_nasce(nome, &er))?;
        let out = if (self.remapeia)(&mut self.w, &[x], &self.mapa) > 0 {
            serializa(&self.w, x)
                .map_err(|er| nao_nasce(nome, &er))?
                .unwrap_or_else(|| b.to_vec())
        } else {
            b.to_vec()
        };
        self.w.despawn(x);
        Ok(out)
    }

    fn linha(&mut self, l: &mut Linha) -> Result<(), ErroDaMacroView> {
        for c in &mut l.componentes {
            c.bytes = self.blob(&c.nome, &c.bytes)?;
        }
        Ok(())
    }
}

/// **A trilha do mundo de `doc` nasce no destino** (cada trilha `Mundo`): os objectos, os componentes,
/// as referências; a trilha fica transposta e com o `target` no objecto Macro. Os nascidos entram em
/// `nascidos` (o chamador desfá-los se algo falhar).
///
/// # Errors
/// [`m::MACRO_LIB_ILEGIVEL`] · [`m::MACRO_MUNDO_COMPONENTE_DESCONHECIDO`] · [`m::MACRO_APLICAR_NAO_NASCE`].
pub(crate) fn aplica(
    doc: &mut MacroDoc,
    fams: &[FamiliaDaMacro],
    d: &Destino<'_>,
    sim: &mut SimWorld,
    portas: &mut Portas<'_>,
    nascidos: &mut Vec<u64>,
) -> Result<(), ErroDaMacroView> {
    for (k, &fam) in fams.iter().enumerate() {
        if fam != FamiliaDaMacro::Mundo {
            continue;
        }
        let lida = Lida::de(&doc.tracks[k]).map_err(|_| ilegivel())?;
        let mut r = Rascunho {
            w: World::new(),
            registo: portas.registo,
            remapeia: portas.remapeia,
            mapa: BTreeMap::new(),
        };
        let raiz = nasce(&lida, d, sim, portas, nascidos, &mut r)?;
        let mut t = transpoe(&doc.tracks[k], lida, &mut r)?;
        t.target = raiz;
        super::palco_fisica::fim_no_destino(&t, sim, portas.registo, Entity::from_bits(d.raiz));
        doc.tracks[k] = t;
    }
    Ok(())
}

fn sid(w: &World, e: Entity) -> u64 {
    w.get::<StableId>(e).map_or(0, |s| s.0)
}

/// A imagem que um objecto do mundo é (os píxeis do recurso embutido), se é uma. A porta de produto
/// faz nascer alfa RECTO (o atlas): um recurso pré-multiplicado volta a recto antes de nascer.
fn imagem_de(
    o: &ObjetoEm,
    recursos: &BTreeMap<[u8; 32], &Recurso>,
) -> Result<Option<Imagem>, ErroDaMacroView> {
    let (Some(px), true) = (
        o.componentes.get(super::PIXEIS),
        o.componentes.contains_key(SPRITE),
    ) else {
        return Ok(None);
    };
    let nome = o
        .componentes
        .get(NOME)
        .and_then(|b| postcard::from_bytes::<ph2d_ecs::Name>(b).ok())
        .map(|n| n.0)
        .unwrap_or_default();
    let falta = || erro_com(m::MACRO_APLICAR_NAO_NASCE, nome.clone());
    let id = postcard::from_bytes::<ph2d_ecs::SpritePixels>(px).map_err(|_| falta())?;
    let r = recursos.get(id.0.as_bytes()).ok_or_else(falta)?;
    let (mut rgba, w, h) = de_png(&r.bytes).ok_or_else(falta)?;
    if r.premultiplicado {
        ph2d_render::unpremultiply_rgba8(&mut rgba);
    }
    Ok(Some(Imagem { w, h, rgba, nome }))
}

/// O blob de `nome` no projeto de destino: as folhas extra trazem as imagens delas; o resto passa como veio.
fn no_destino(
    nome: &str,
    b: Vec<u8>,
    recursos: &BTreeMap<[u8; 32], &Recurso>,
    poe: &dyn Fn(crate::aplicar::Rgba) -> [u8; 32],
) -> Result<Vec<u8>, ErroDaMacroView> {
    if nome == super::FOLHAS {
        folhas_no_destino(&b, recursos, poe)
    } else {
        Ok(b)
    }
}

/// ⭐ **As folhas extra noutro projeto** (plano GEF 26 F12): a imagem de cada uma entra na biblioteca do destino pela
/// porta, e a folha passa ao id que ela lá tem. Sem o recurso embutido, a macro não nasce (o mesmo erro da imagem).
fn folhas_no_destino(
    b: &[u8],
    recursos: &BTreeMap<[u8; 32], &Recurso>,
    poe: &dyn Fn(crate::aplicar::Rgba) -> [u8; 32],
) -> Result<Vec<u8>, ErroDaMacroView> {
    let falha = |e: &dyn std::fmt::Display| nao_nasce(super::FOLHAS, e);
    let mut s: ph2d_ecs::SpriteSheets = postcard::from_bytes(b).map_err(|e| falha(&e))?;
    for x in &mut s.0 {
        let r = recursos
            .get(x.asset.as_bytes())
            .ok_or_else(|| falha(&"∅"))?;
        // Sem desfazer a pré-multiplicação: a folha extra desenha-se em alfa recto, e o `premultiplicado` do recurso é
        // o da sprite que o partilha pelo id (auditoria 08/11). Os bytes de uma folha são sempre rectos: o Add sheet
        // de um recurso mostrado pré-multiplicado já guarda uma cópia recta (`ph2d_anim_sheets::apply::straight_id`).
        let px = de_png(&r.bytes).ok_or_else(|| falha(&"png"))?;
        x.asset = ph2d_asset_id::AssetId::from_digest(poe(px));
    }
    postcard::to_allocvec(&s).map_err(|e| falha(&e))
}

/// O `Sprite` gravado sobre o que a porta de produto pôs: tudo menos a fonte (o atlas do destino).
fn funde_o_sprite(
    w: &mut World,
    e: Entity,
    bytes: &[u8],
    r: &mut Rascunho<'_>,
) -> Result<(), ErroDaMacroView> {
    let entrada = r
        .registo
        .get_by_name(SPRITE)
        .ok_or_else(|| desconhecido(SPRITE))?;
    let (valor, poe, insere) = (
        entrada.value_of,
        entrada.set_value,
        entrada.insert_from_bytes,
    );
    let fonte = valor(w, e)
        .ok()
        .flatten()
        .and_then(|v| v.get("source").cloned());
    let Some(fonte) = fonte else {
        return insere(w, e, bytes).map_err(|er| nao_nasce(SPRITE, &er));
    };
    let x = r.w.spawn_empty().id();
    insere(&mut r.w, x, bytes).map_err(|er| nao_nasce(SPRITE, &er))?;
    let gravado = valor(&r.w, x).map_err(|er| nao_nasce(SPRITE, &er))?;
    r.w.despawn(x);
    let mut v = gravado.ok_or_else(|| nao_nasce(SPRITE, &"∅"))?;
    if let Some(o) = v.as_object_mut() {
        o.insert("source".to_owned(), fonte);
    }
    poe(w, e, v).map_err(|er| nao_nasce(SPRITE, &er))
}

/// Faz nascer o FIM e põe-lhe os componentes; devolve o `StableId` do objecto Macro.
fn nasce(
    lida: &Lida,
    d: &Destino<'_>,
    sim: &mut SimWorld,
    portas: &mut Portas<'_>,
    nascidos: &mut Vec<u64>,
    r: &mut Rascunho<'_>,
) -> Result<u64, ErroDaMacroView> {
    let fim = lida.no_fim();
    let recursos: BTreeMap<[u8; 32], &Recurso> = lida.recursos().map(|x| (x.id, x)).collect();
    let raiz = Entity::from_bits(d.raiz);
    let (mut ents, mut imagens) = (BTreeMap::new(), BTreeSet::new());
    // Os objectos que outra família faz nascer (com ou sem linhas na trilha).
    for o in lida.objetos() {
        let e = match o.quem {
            Quem::Macro => raiz,
            Quem::Trilha(j) => usize::try_from(j)
                .ok()
                .and_then(|j| d.trilhas.get(&j))
                .map(|b| Entity::from_bits(*b))
                .ok_or_else(|| erro_com(m::MACRO_APLICAR_NAO_NASCE, format!("#{j}")))?,
            Quem::Mundo => continue,
        };
        ents.insert(o.id, e);
    }
    for (id, o) in fim.iter().filter(|(_, o)| o.vivo) {
        let e = match o.quem {
            Quem::Macro | Quem::Trilha(_) => continue,
            Quem::Mundo => {
                let b = match imagem_de(o, &recursos)? {
                    Some(i) => {
                        imagens.insert(*id);
                        (portas.imagem)(sim, i)
                            .map_err(|e| erro_com(m::MACRO_APLICAR_NAO_NASCE, e))?
                    }
                    None => sim.world_mut().spawn(Transform::IDENTITY).id().to_bits(),
                };
                nascidos.push(b);
                Entity::from_bits(b)
            }
        };
        ents.insert(*id, e);
    }
    let w = sim.world_mut();
    // Os pais depois de todos existirem (a ordem das ids locais não é a da árvore).
    for (id, o) in fim.iter().filter(|(_, o)| o.quem == Quem::Mundo) {
        if let Some(&e) = ents.get(id) {
            let pai = o.pai.and_then(|p| ents.get(&p).copied()).unwrap_or(raiz);
            w.entity_mut(e).insert(ChildOf(pai));
        }
    }
    ph2d_ecs::assign_missing_stable_ids(w);
    // Um objecto que não chega vivo ao fim não nasceu aqui: `0` (nenhum) — nunca o `StableId` da
    // gravação, que no destino pode ser de um objecto alheio.
    r.mapa = lida
        .objetos()
        .map(|o| (o.original, ents.get(&o.id).map_or(0, |e| sid(w, *e))))
        .collect();
    for (id, o) in &fim {
        let Some(&e) = ents.get(id) else {
            continue;
        };
        let do_alvo = d.por_cima && matches!(o.quem, Quem::Trilha(_));
        let imagem = imagens.contains(id);
        for (nome, bytes) in &o.componentes {
            if (do_alvo && (nome == NOME || nome == TRANSFORM))
                || (imagem && (nome == super::PIXEIS || nome == NOME))
            {
                continue;
            }
            let b = no_destino(nome, r.blob(nome, bytes)?, &recursos, portas.poe_recurso)?;
            if imagem && nome == SPRITE {
                funde_o_sprite(w, e, &b, r)?;
                continue;
            }
            let insere = r
                .registo
                .get_by_name(nome)
                .ok_or_else(|| desconhecido(nome))?;
            (insere.insert_from_bytes)(w, e, &b).map_err(|er| nao_nasce(nome, &er))?;
        }
        // ADR-0199: uma gravação antiga traz a forma no corpo — ela nasce na filha.
        ph2d_physics_ecs::move_forma_para_filha(w, e);
    }
    Ok(sid(w, raiz))
}

/// A trilha no projecto de destino: os `original` e os blobs com referências passam aos novos.
fn transpoe(t: &Track, lida: Lida, r: &mut Rascunho<'_>) -> Result<Track, ErroDaMacroView> {
    let Lida {
        mut inicial,
        mut eventos,
    } = lida;
    let mapa = r.mapa.clone();
    let novo = |o: &mut Objeto| o.original = mapa.get(&o.original).copied().unwrap_or(0);
    inicial.objetos.iter_mut().for_each(novo);
    for l in &mut inicial.linhas {
        r.linha(l)?;
    }
    for (_, ev) in &mut eventos {
        let Evento::Passo {
            novos, mudancas, ..
        } = ev
        else {
            continue;
        };
        novos.iter_mut().for_each(novo);
        for mu in mudancas {
            match mu {
                Mudanca::Nasce(l) => r.linha(l)?,
                Mudanca::Morre(_) => {}
                Mudanca::Muda { componentes, .. } => {
                    for (n, b) in componentes {
                        if let Some(b) = b {
                            *b = r.blob(n, b)?;
                        }
                    }
                }
            }
        }
    }
    let mut out = t.clone();
    out.initial = inicial.escreve();
    for (e, (_, ev)) in out.events.iter_mut().zip(&eventos) {
        e.payload = ev.escreve();
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(rgba: &[u8], w: u32, h: u32) -> Vec<u8> {
        let img = image::RgbaImage::from_raw(w, h, rgba.to_vec()).unwrap();
        let mut out = std::io::Cursor::new(Vec::new());
        img.write_to(&mut out, image::ImageFormat::Png).unwrap();
        out.into_inner()
    }

    /// ⭐ **As folhas extra chegam ao outro projeto** (plano GEF 26 F12): a imagem de cada uma entra pela porta e a
    /// folha passa ao id que ela lá tem; sem o recurso, a macro não nasce. *Mutações: não chamar a porta ⇒ o id fica o
    /// da gravação e `postos` vazio; aceitar sem recurso ⇒ o `Err` some.*
    #[test]
    fn as_folhas_extra_chegam_ao_outro_projeto_com_a_imagem() {
        let rgba = [9u8, 8, 7, 255, 1, 2, 3, 255];
        let gravado = [5u8; 32];
        let r = Recurso {
            id: gravado,
            bytes: png(&rgba, 2, 1),
            premultiplicado: false,
        };
        let recursos: BTreeMap<[u8; 32], &Recurso> = [(gravado, &r)].into();
        let folha = ph2d_ecs::ExtraSheet {
            asset: ph2d_asset_id::AssetId::from_digest(gravado),
            size: [1.0, 1.0],
            anchor: [0.0, 0.0],
            hframes: 2,
            vframes: 1,
        };
        let b =
            postcard::to_allocvec(&ph2d_ecs::SpriteSheets([folha].into_iter().collect())).unwrap();
        let postos = std::cell::RefCell::new(Vec::new());
        let poe = |px: crate::aplicar::Rgba| {
            postos.borrow_mut().push(px);
            [6u8; 32]
        };
        let f = no_destino(super::super::FOLHAS, b.clone(), &recursos, &poe).expect("nasce");
        let s: ph2d_ecs::SpriteSheets = postcard::from_bytes(&f).unwrap();
        assert_eq!(*s.0[0].asset.as_bytes(), [6; 32]);
        assert_eq!(*postos.borrow(), [(rgba.to_vec(), 2, 1)]);
        assert!(
            folhas_no_destino(&b, &BTreeMap::new(), &poe).is_err(),
            "sem recurso não nasce"
        );
        // CONTROLO: outro componente passa como veio, sem tocar na porta.
        assert_eq!(
            no_destino("ph2d::ecs::Name", vec![1, 2], &recursos, &poe).unwrap(),
            [1, 2]
        );
        assert_eq!(postos.borrow().len(), 1);
    }

    /// ⭐ Achado da auditoria 08/11: uma folha extra desenha-se SEMPRE em alfa recto (`ph2d_render::SheetTextures`), mas
    /// o recurso dela é partilhado pelo id com a sprite pré-multiplicada que mostre a mesma imagem (o BG removal), e a
    /// chegada desfazia-lhe a pré-multiplicação: a cor da folha mudava no outro projeto. Os bytes chegam como foram.
    /// *Mutação: voltar a desfazer quando `premultiplicado` ⇒ os bytes da meia-transparência mudam.*
    #[test]
    fn a_folha_extra_chega_com_a_mesma_cor_mesmo_partilhada_com_uma_premultiplicada() {
        let rgba = [200u8, 100, 50, 128, 1, 2, 3, 255];
        let gravado = [5u8; 32];
        let r = Recurso {
            id: gravado,
            bytes: png(&rgba, 2, 1),
            premultiplicado: true,
        };
        let recursos: BTreeMap<[u8; 32], &Recurso> = [(gravado, &r)].into();
        let folha = ph2d_ecs::ExtraSheet {
            asset: ph2d_asset_id::AssetId::from_digest(gravado),
            size: [1.0, 1.0],
            anchor: [0.0, 0.0],
            hframes: 2,
            vframes: 1,
        };
        let b =
            postcard::to_allocvec(&ph2d_ecs::SpriteSheets([folha].into_iter().collect())).unwrap();
        let postos = std::cell::RefCell::new(Vec::new());
        let poe = |px: crate::aplicar::Rgba| {
            postos.borrow_mut().push(px);
            [6u8; 32]
        };
        folhas_no_destino(&b, &recursos, &poe).expect("nasce");
        assert_eq!(*postos.borrow(), [(rgba.to_vec(), 2, 1)], "ao bit");
    }
}
