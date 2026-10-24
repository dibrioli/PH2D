//! **Aplicar uma macro** (W18, `docs/Macros/16_W18_plano.md` §2.5–2.7; decisões do dono 23/10, 3–6) —
//! UMA porta para o painel MACROS (`MacroRequest::Aplica`) e para o painel de Assets (a receita largada
//! ou aberta com duplo-clique): [`aplica`].
//!
//! - **Novo**: nasce o objecto Macro (o `spawn_root` do Add) e, dentro dele, o que cada trilha com ficha
//!   criou — a imagem pela porta de produto da composição ([`Portas::imagem`]) com o documento do FIM
//!   da reprodução instalado no Painter; o objecto Motion com os nós da macro no documento vivo. Os
//!   alvos das trilhas reescrevem-se para os objectos novos e a `Macro` instala-se com o clipe.
//! - **Por cima**: o objecto Macro nasce na pose do alvo; a trilha do Painter passa a ter o alvo como
//!   tela (o tamanho gravado fica no inicial, etiqueta 6 — o encaixe vivo lê o gizmo a cada quadro), a
//!   do Motion junta os nós dela aos do alvo.
//!
//! E o ficheiro que sai ([`para_exportar`]): cada trilha provada a tocar até ao fim, a do Motion só com
//! o grafo do dono. Falhou ⇒ nada se escreve, e o `aviso` diz porquê.

use crate::{Cena, Familias};
use ph2d_ecs::{ChildOf, Entity, Macro, ModoDaMacro, Name, SimWorld, StableId, Transform};
use ph2d_editor_core::macro_edits::{self as m, ErroDaMacroView, FamiliaDaMacro, OndeAplicar};
use ph2d_editor_core::macro_etapas::erros::{erro, erro_com};
use ph2d_editor_core::macro_tutorial::TranspoeDaFamilia;
use ph2d_macro::ficheiro::Ficheiro;
use ph2d_macro::{MacroDoc, Track};
use ph2d_tool_painter::macro_track::{self as pm, fim, inicial};
use ph2d_tool_painter::{PaintedDocument, PainterTool};
use std::collections::BTreeMap;

#[path = "aplicar_continua.rs"]
mod continua;
use continua::{continuacao_do_motion, rebaseia_o_painter};

/// Insere componentes no objecto `bits` (se ele existe).
macro_rules! poe {
    ($sim:expr, $bits:expr, $c:expr) => {
        if let Ok(mut e) = $sim.world_mut().get_entity_mut(Entity::from_bits($bits)) {
            e.insert($c);
        }
    };
}

/// Uma imagem a nascer: os píxeis (RGBA directo) e o nome.
pub struct Imagem {
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
    pub nome: String,
}

/// Píxeis RGBA directos e o tamanho.
pub type Rgba = (Vec<u8>, u32, u32);

/// **O que só a composição alcança** (o renderizador, o `AssetDb`, a câmara, o nascimento do Add).
pub struct Portas<'a> {
    /// O vazio do Add (nome único, `StableId`, `Transform`); devolve os bits.
    pub raiz: &'a dyn Fn(&mut SimWorld, &str) -> u64,
    /// Uma imagem nova pela porta de produto (atlas + `AssetDb`: sobrevive ao gravar); devolve os bits.
    pub imagem: &'a mut dyn FnMut(&mut SimWorld, Imagem) -> Result<u64, String>,
    /// Os píxeis (RGBA directo) de uma imagem de que o Painter ainda não tem documento.
    pub pixeis: &'a mut dyn FnMut(&SimWorld, u64) -> Option<Rgba>,
    /// O objecto mostra píxeis (uma imagem): a pergunta barata do painel, a cada quadro.
    pub e_imagem: &'a dyn Fn(&SimWorld, u64) -> bool,
    /// O centro da vista, no mundo (onde «Novo» nasce sem ponto).
    pub centro: [f32; 2],
    /// O registo de componentes do produto: a família «mundo» nomeia e põe os componentes por ele.
    pub registo: &'a ph2d_ecs::scene::ComponentRegistry,
    /// Os bytes de um recurso pelo id de conteúdo (o `AssetDb`): o que a família «mundo» embute.
    pub recurso: &'a dyn Fn(&[u8; 32]) -> Option<Vec<u8>>,
    /// O remapeador das referências entre objectos (`StableId` antigo → novo) da composição — a tabela
    /// da casa (`ph2d_app_components::instance_refs::remap_object_refs`); devolve quantos mudou.
    pub remapeia: &'a Remapeia,
}

/// O remapeador das referências entre objectos: o mundo, as entidades e o mapa de `StableId`.
pub type Remapeia = dyn Fn(&mut ph2d_ecs::World, &[Entity], &BTreeMap<u64, u64>) -> usize;

/// **A família de uma trilha**, pela chave: o Painter e o mundo são desta crate, as outras chegam pela
/// composição (`outras`). Uma chave que ninguém conhece recusa-se pelo nome — nunca se toca como outra.
///
/// # Errors
/// [`m::MACRO_FAMILIA_DESCONHECIDA`] com a chave.
pub fn familia(
    t: &Track,
    outras: crate::etapas::Injectadas<'_>,
) -> Result<FamiliaDaMacro, ErroDaMacroView> {
    if t.family == pm::FAMILIA {
        return Ok(FamiliaDaMacro::Painter);
    }
    if t.family == crate::mundo::formato::FAMILIA {
        return Ok(FamiliaDaMacro::Mundo);
    }
    outras
        .iter()
        .find(|o| o.chave() == t.family)
        .map(|o| o.familia())
        .ok_or_else(|| erro_com(m::MACRO_FAMILIA_DESCONHECIDA, t.family.clone()))
}

fn modo_de(i: u8) -> ModoDaMacro {
    match i {
        m::MODO_EXPLICATIVO => ModoDaMacro::Explicativo,
        m::MODO_FAZER => ModoDaMacro::Fazer,
        _ => ModoDaMacro::Fiel,
    }
}

fn indice_de(modo: ModoDaMacro) -> u8 {
    match modo {
        ModoDaMacro::Fiel => m::MODO_FIEL,
        ModoDaMacro::Explicativo => m::MODO_EXPLICATIVO,
        ModoDaMacro::Fazer => m::MODO_FAZER,
    }
}

/// A frase de um erro, com o detalhe dele (o que o `aviso` cita).
fn frase(e: &ErroDaMacroView) -> String {
    match &e.detalhe {
        Some(d) => ph2d_i18n::tr_with(e.chave, &[("detail", d)]),
        None => ph2d_i18n::tr(e.chave).to_owned(),
    }
}

/// «Não toca até ao fim»: a chave do aviso, citando onde parou.
fn nao_toca(e: &ErroDaMacroView) -> ErroDaMacroView {
    erro_com(m::MACRO_LIB_NAO_TOCA, frase(e))
}

/// **A gravação do objecto `bits` como ficheiro**: o nome é o do objecto, o modo o do componente.
///
/// # Errors
/// [`m::MACRO_LIB_SEM_GRAVACAO`] (sem `Macro` ou vazia) · o ficheiro não se lê.
pub fn ficheiro_de(sim: &SimWorld, bits: u64) -> Result<Ficheiro, ErroDaMacroView> {
    let e = Entity::try_from_bits(bits).ok_or_else(|| erro(m::MACRO_LIB_SEM_GRAVACAO))?;
    let w = sim.world();
    let mac = w
        .get::<Macro>(e)
        .filter(|x| !x.doc.is_empty())
        .ok_or_else(|| erro(m::MACRO_LIB_SEM_GRAVACAO))?;
    let doc = ph2d_macro::codec::read(&mac.doc).map_err(|c| crate::erros::do_ficheiro(&c))?;
    Ok(Ficheiro {
        doc,
        modo: indice_de(mac.modo),
        nome: w.get::<Name>(e).map(|n| n.0.clone()).unwrap_or_default(),
    })
}

/// **O ficheiro que sai do PH2D**: cada trilha provada a tocar até ao fim num tocador limpo (o Painter
/// pelo [`fim::documento_no_fim`], o Motion pela porta dele, que também a reduz ao grafo do dono).
///
/// # Errors
/// [`m::MACRO_LIB_NAO_TOCA`] com o porquê — nada se escreve.
pub fn para_exportar(
    f: &Ficheiro,
    motion: &dyn TranspoeDaFamilia,
    outras: crate::etapas::Injectadas<'_>,
) -> Result<Ficheiro, ErroDaMacroView> {
    let mut out = f.clone();
    for t in &mut out.doc.tracks {
        match familia(t, outras)? {
            FamiliaDaMacro::Painter => {
                fim::documento_no_fim(t, None)
                    .map_err(|e| nao_toca(&crate::erros::do_painter(&e)))?;
            }
            FamiliaDaMacro::Motion => *t = motion.exporta(t).map_err(|e| nao_toca(&e))?,
            FamiliaDaMacro::Mundo => crate::mundo::aplica::confere(t)?,
        }
    }
    Ok(out)
}

/// Os píxeis compostos de `doc` — pelo MESMO caminho do load de um projecto (`baked_document_pixels`).
fn composto(doc: PaintedDocument) -> Option<(Vec<u8>, u32, u32)> {
    let mut t = PainterTool::default();
    t.install_document(1, doc);
    t.baked_document_pixels(1)
}

/// O documento que o Painter tem (ou teria) para a imagem `bits`.
fn documento_da_imagem(
    painter: Option<&PainterTool>,
    portas: &mut Portas<'_>,
    sim: &SimWorld,
    bits: u64,
) -> Option<PaintedDocument> {
    if let Some(p) = painter
        && p.document_entities().contains(&bits)
    {
        return p
            .collect_documents(&BTreeMap::from([(bits, 0)]))
            .into_iter()
            .next();
    }
    let (px, w, h) = (portas.pixeis)(sim, bits).filter(|(_, w, h)| *w > 0 && *h > 0)?;
    let mut t = PainterTool::default();
    t.bind_document(1, px, w, h);
    t.collect_documents(&BTreeMap::from([(1, 0)]))
        .into_iter()
        .next()
}

fn sid_de(sim: &mut SimWorld, bits: u64) -> u64 {
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    sim.world()
        .get::<StableId>(Entity::from_bits(bits))
        .map_or(0, |s| s.0)
}

fn transform_local(l: [f32; 7]) -> Transform {
    let mut t = Transform::IDENTITY;
    t.translation.x = l[0];
    t.translation.y = l[1];
    t.rotation = l[2];
    t.scale.x = l[3];
    t.scale.y = l[4];
    t.with_skew(l[5], l[6])
}

/// Desfaz um nascimento que não chegou ao fim (nada fica meio aplicado).
fn desfaz(sim: &mut SimWorld, nascidos: &[u64]) {
    for b in nascidos.iter().rev() {
        let _ = sim.world_mut().despawn(Entity::from_bits(*b));
    }
}

/// **As imagens que as trilhas do Painter do objecto Macro `raiz` pintam** (bits, cada uma uma vez: a
/// continuação pinta a mesma). O documento do Painter é de 8 bits: a composição avisa a precisão de
/// cada uma que seja de 16.
#[must_use]
pub fn imagens_que_pinta(sim: &SimWorld, raiz: u64) -> Vec<u64> {
    let Ok(fi) = ficheiro_de(sim, raiz) else {
        return Vec::new();
    };
    let imagens: std::collections::BTreeSet<u64> = fi
        .doc
        .tracks
        .iter()
        .filter(|t| t.family == pm::FAMILIA)
        .filter_map(|t| ph2d_ecs::entity_with_stable_id(sim.world(), StableId(t.target)))
        .map(Entity::to_bits)
        .collect();
    imagens.into_iter().collect()
}

/// **O alvo recebe esta macro por cima?** — `Ok(família)`, ou a chave do porquê não. As trilhas do
/// mundo não contam para o «misto»: vão para dentro do objecto Macro; uma macro só do mundo serve
/// qualquer objecto (`Ok(Mundo)`).
///
/// # Errors
/// [`m::MACRO_APLICAR_MISTO`] · [`m::MACRO_APLICAR_INCOMPATIVEL`] · [`m::MACRO_APLICAR_SEM_ALVO`] ·
/// [`m::MACRO_FAMILIA_DESCONHECIDA`].
pub fn compativel(
    sim: &SimWorld,
    e_imagem: &dyn Fn(&SimWorld, u64) -> bool,
    painter: Option<&PainterTool>,
    doc: &MacroDoc,
    alvo: u64,
    outras: crate::etapas::Injectadas<'_>,
) -> Result<FamiliaDaMacro, &'static str> {
    let w = sim.world();
    let e = Entity::try_from_bits(alvo)
        .filter(|e| w.get_entity(*e).is_ok())
        .ok_or(m::MACRO_APLICAR_SEM_ALVO)?;
    if doc.tracks.is_empty() {
        return Err(m::MACRO_LIB_SEM_GRAVACAO);
    }
    let fams: Vec<FamiliaDaMacro> = doc
        .tracks
        .iter()
        .map(|t| familia(t, outras))
        .collect::<Result<_, _>>()
        .map_err(|e| e.chave)?;
    let mut fora = fams.into_iter().filter(|x| *x != FamiliaDaMacro::Mundo);
    let Some(fam) = fora.next() else {
        return Ok(FamiliaDaMacro::Mundo);
    };
    if fora.any(|x| x != fam) {
        return Err(m::MACRO_APLICAR_MISTO);
    }
    let serve = match fam {
        FamiliaDaMacro::Painter => {
            e_imagem(sim, alvo) || painter.is_some_and(|p| p.document_entities().contains(&alvo))
        }
        FamiliaDaMacro::Motion => w.get::<ph2d_ecs::MotionGenerator>(e).is_some(),
        FamiliaDaMacro::Mundo => true,
    };
    if serve {
        Ok(fam)
    } else {
        Err(m::MACRO_APLICAR_INCOMPATIVEL)
    }
}

/// **A porta única de aplicar**: devolve os bits do objecto Macro nascido.
///
/// # Errors
/// A chave da frase (e o detalhe) — o mundo fica como estava.
pub fn aplica(
    fi: &Ficheiro,
    onde: OndeAplicar,
    f: &mut Familias<'_>,
    c: &mut Cena<'_>,
    portas: &mut Portas<'_>,
) -> Result<u64, ErroDaMacroView> {
    if fi.doc.tracks.is_empty() {
        return Err(erro(m::MACRO_LIB_SEM_GRAVACAO));
    }
    // Uma família que ninguém conhece recusa-se antes de nascer o que quer que seja.
    let fams: Vec<FamiliaDaMacro> = fi
        .doc
        .tracks
        .iter()
        .map(|t| familia(t, f.etapas))
        .collect::<Result<_, _>>()?;
    let r = match onde {
        OndeAplicar::Novo { em } => novo(fi, &fams, em.unwrap_or(portas.centro), f, c, portas),
        OndeAplicar::PorCima { alvo } => por_cima(fi, &fams, alvo, f, c, portas),
    }?;
    let (raiz, doc) = r;
    crate::instala::instala_nascido(c, f.etapas, raiz, &doc, modo_de(fi.modo)).map_err(erro)?;
    Ok(raiz)
}

/// **Aplicar sem placa** (os gates que só a composição alcança): a imagem nasce como a porta de produto
/// a deixa no mundo, sem o atlas — os píxeis são do documento do Painter.
#[cfg(feature = "test-support")]
pub fn sem_placa(
    fi: &Ficheiro,
    onde: OndeAplicar,
    sim: &mut SimWorld,
    painter: &mut PainterTool,
    motion: &mut dyn ph2d_editor_core::macro_tutorial::GravadoraQueEnsina,
    raiz: &dyn Fn(&mut SimWorld, &str) -> u64,
) -> Result<u64, ErroDaMacroView> {
    let mut imagem = |s: &mut SimWorld, i: Imagem| {
        let b = s
            .world_mut()
            .spawn((Name::new(i.nome), Transform::IDENTITY))
            .id();
        ph2d_ecs::assign_missing_stable_ids(s.world_mut());
        Ok(b.to_bits())
    };
    let (mut pixeis, e_imagem) = (|_: &SimWorld, _: u64| None, |_: &SimWorld, _: u64| false);
    let mut registo = ph2d_ecs::scene::ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut registo);
    ph2d_physics_ecs::register_physics_components(&mut registo);
    // A `Sprite` de uma imagem importada que a macro faz nascer.
    ph2d_render::register_render_components(&mut registo);
    let remapeia = |w: &mut ph2d_ecs::World, es: &[Entity], m: &BTreeMap<u64, u64>| {
        ph2d_ecs::remap_instance_of(w, es, m)
            + ph2d_physics_ecs::remap_joint_refs(w, es, m)
            + ph2d_physics_ecs::remap_wheel_refs(w, es, m)
    };
    let mut portas = Portas {
        raiz,
        imagem: &mut imagem,
        pixeis: &mut pixeis,
        e_imagem: &e_imagem,
        centro: [0.0, 0.0],
        registo: &registo,
        recurso: &|_| None,
        remapeia: &remapeia,
    };
    let mut f = Familias {
        painter: Some(painter),
        painter_activo: false,
        motion,
        motion_em_edicao: None,
        motion_na_mao: false,
        etapas: &[],
        escolhido: None,
        portas: None,
    };
    let mut tl = ph2d_timeline::TimelineState::default();
    let mut c = Cena {
        sim,
        timeline: &mut tl,
        intents: &mut Vec::new(),
        partidas: None,
    };
    aplica(fi, onde, &mut f, &mut c, &mut portas)
}

/// O objecto Macro de nome `nome` com a pose `t`; devolve os bits.
fn nasce_a_raiz(sim: &mut SimWorld, portas: &Portas<'_>, nome: &str, t: Transform) -> u64 {
    let b = (portas.raiz)(sim, nome);
    poe!(sim, b, (t, Macro::default()));
    b
}

/// A trilha que acaba a imagem da trilha `k`: a ÚLTIMA que a continua (o artista voltou a ela), ou ela.
fn ultima_da_imagem(doc: &MacroDoc, k: usize) -> &Track {
    let ku = u32::try_from(k).ok();
    doc.tracks
        .iter()
        .rev()
        .find(|t| t.family == pm::FAMILIA && t.continua.is_some() && t.continua == ku)
        .unwrap_or(&doc.tracks[k])
}

/// O que a trilha `k` do Painter cria: a imagem do FIM da reprodução (o da última continuação), com o
/// documento no Painter.
fn nasce_a_imagem(
    doc: &MacroDoc,
    k: usize,
    nome: &str,
    f: &mut Familias<'_>,
    sim: &mut SimWorld,
    portas: &mut Portas<'_>,
) -> Result<u64, ErroDaMacroView> {
    let doc = fim::documento_no_fim(ultima_da_imagem(doc, k), None)
        .map_err(|e| nao_toca(&crate::erros::do_painter(&e)))?;
    let (rgba, w, h) = composto(doc.clone())
        .ok_or_else(|| erro_com(m::MACRO_APLICAR_NAO_NASCE, nome.to_owned()))?;
    let bits = (portas.imagem)(
        sim,
        Imagem {
            w,
            h,
            rgba,
            nome: nome.to_owned(),
        },
    )
    .map_err(|e| erro_com(m::MACRO_APLICAR_NAO_NASCE, e))?;
    if let Some(p) = f.painter.as_deref_mut() {
        p.install_document(bits, doc);
    }
    Ok(bits)
}

/// Um objecto Motion vazio pelo Add.
fn nasce_o_motion(sim: &mut SimWorld, portas: &Portas<'_>, nome: &str) -> u64 {
    let b = (portas.raiz)(sim, nome);
    poe!(sim, b, ph2d_ecs::MotionGenerator);
    b
}

fn novo(
    fi: &Ficheiro,
    fams: &[FamiliaDaMacro],
    em: [f32; 2],
    f: &mut Familias<'_>,
    c: &mut Cena<'_>,
    portas: &mut Portas<'_>,
) -> Result<(u64, MacroDoc), ErroDaMacroView> {
    use FamiliaDaMacro::{Motion, Mundo, Painter};
    let sim = &mut *c.sim;
    let mut pose = Transform::IDENTITY;
    (pose.translation.x, pose.translation.y) = (em[0], em[1]);
    let mut doc = fi.doc.clone();
    let mut nascidos = Vec::new();
    // O objecto que cada trilha (pelo índice) faz nascer ou continua — o que a família «mundo» lê.
    let mut objs: BTreeMap<usize, u64> = BTreeMap::new();
    let r = (|| {
        let legado = doc
            .tracks
            .iter()
            .zip(fams)
            .all(|(t, x)| *x == Mundo || (t.nasce.is_none() && t.continua.is_none()));
        let primeira = fams.iter().position(|x| *x != Mundo);
        let raiz = if legado && let Some(p) = primeira {
            // Uma gravação de antes da W18 (sem fichas): o objecto Macro É o objecto da trilha — e só
            // uma por família diz qual.
            if [Painter, Motion]
                .iter()
                .any(|x| fams.iter().filter(|y| *y == x).count() > 1)
            {
                return Err(erro(m::MACRO_APLICAR_SEM_FICHAS_VARIAS));
            }
            let b = match fams[p] {
                Painter => nasce_a_imagem(&doc, p, &fi.nome, f, sim, portas)?,
                Motion | Mundo => nasce_o_motion(sim, portas, &fi.nome),
            };
            nascidos.push(b);
            poe!(sim, b, (pose, Macro::default()));
            let sid = sid_de(sim, b);
            for (k, t) in doc.tracks.iter_mut().enumerate() {
                if fams[k] != Mundo {
                    *t = muda_o_alvo(t, fams[k], sid, sid, f)?;
                    objs.insert(k, b);
                }
            }
            b
        } else {
            let raiz = nasce_a_raiz(sim, portas, &fi.nome, pose);
            nascidos.push(raiz);
            let pai = Entity::from_bits(raiz);
            let vida = sid_de(sim, raiz);
            for (k, &fam) in fams.iter().enumerate() {
                if fam == Mundo {
                    continue;
                }
                if let Some(o) = doc.tracks[k].continua {
                    // A continuação: o alvo é o objecto da trilha que ela continua; não nasce nada.
                    let b = usize::try_from(o)
                        .ok()
                        .and_then(|o| objs.get(&o).copied())
                        .ok_or_else(|| erro_com(m::MACRO_APLICAR_NAO_NASCE, format!("#{o}")))?;
                    let sid = sid_de(sim, b);
                    doc.tracks[k] = if fam == Motion {
                        continuacao_do_motion(&fi.doc, &doc, k, sid, vida, f)?
                    } else {
                        muda_o_alvo(&doc.tracks[k], fam, sid, vida, f)?
                    };
                    objs.insert(k, b);
                    continue;
                }
                let Some(n) = doc.tracks[k].nasce.clone() else {
                    // Uma trilha sem ficha numa macro com fichas: o próprio objecto Macro.
                    doc.tracks[k] = muda_o_alvo(&doc.tracks[k], fam, vida, vida, f)?;
                    objs.insert(k, raiz);
                    continue;
                };
                let b = match fam {
                    Painter => nasce_a_imagem(&doc, k, &n.nome, f, sim, portas)?,
                    Motion | Mundo => nasce_o_motion(sim, portas, &n.nome),
                };
                nascidos.push(b);
                poe!(sim, b, (transform_local(n.local), ChildOf(pai)));
                doc.tracks[k] = muda_o_alvo(&doc.tracks[k], fam, sid_de(sim, b), vida, f)?;
                objs.insert(k, b);
            }
            raiz
        };
        let destino = crate::mundo::aplica::Destino {
            raiz,
            trilhas: &objs,
            por_cima: false,
        };
        crate::mundo::aplica::aplica(&mut doc, fams, &destino, sim, portas, &mut nascidos)?;
        Ok(raiz)
    })();
    match r {
        Ok(raiz) => Ok((raiz, doc)),
        Err(e) => {
            desfaz(sim, &nascidos);
            Err(e)
        }
    }
}

/// A trilha a instalar com o alvo `sid`: o Painter só troca o `target`; o Motion junta os nós ao
/// documento vivo (que passa ao fim dela, num passo preso à vida do objecto Macro `vida`) e devolve
/// a trilha transposta. O mundo transpõe-se depois de tudo nascer ([`crate::mundo::aplica`]).
fn muda_o_alvo(
    t: &Track,
    fam: FamiliaDaMacro,
    sid: u64,
    vida: u64,
    f: &mut Familias<'_>,
) -> Result<Track, ErroDaMacroView> {
    match fam {
        FamiliaDaMacro::Painter | FamiliaDaMacro::Mundo => {
            let mut t = t.clone();
            t.target = sid;
            Ok(t)
        }
        FamiliaDaMacro::Motion => {
            let mut n = f.motion.aplica(t, sid, vida)?;
            n.nasce.clone_from(&t.nasce);
            n.continua = t.continua;
            Ok(n)
        }
    }
}

fn por_cima(
    fi: &Ficheiro,
    fams: &[FamiliaDaMacro],
    alvo: u64,
    f: &mut Familias<'_>,
    c: &mut Cena<'_>,
    portas: &mut Portas<'_>,
) -> Result<(u64, MacroDoc), ErroDaMacroView> {
    let sim = &mut *c.sim;
    let fam = compativel(
        sim,
        portas.e_imagem,
        f.painter.as_deref(),
        &fi.doc,
        alvo,
        f.etapas,
    )
    .map_err(erro)?;
    let sid = sid_de(sim, alvo);
    let mut doc = fi.doc.clone();
    let base = if fam == FamiliaDaMacro::Painter {
        Some(
            documento_da_imagem(f.painter.as_deref(), portas, sim, alvo)
                .ok_or_else(|| erro(m::MACRO_APLICAR_INCOMPATIVEL))?,
        )
    } else {
        None
    };
    // A pose do alvo no mundo: o gizmo do objecto Macro começa por cima dele.
    let pose = ph2d_ecs::world_transform(sim.world(), Entity::from_bits(alvo)).unwrap_or_default();
    let raiz = nasce_a_raiz(sim, portas, &fi.nome, pose);
    let mut nascidos = vec![raiz];
    let r = (|| {
        if let Some(base) = &base {
            rebaseia_o_painter(&mut doc, fams, base, sid, sim, (raiz, alvo))?;
        }
        if fam == FamiliaDaMacro::Motion {
            let vida = sid_de(sim, raiz);
            for (k, &fam_k) in fams.iter().enumerate() {
                if fam_k != FamiliaDaMacro::Motion {
                    continue;
                }
                let mut n = if doc.tracks[k].continua.is_some() {
                    continuacao_do_motion(&fi.doc, &doc, k, sid, vida, f)?
                } else {
                    f.motion.aplica(&doc.tracks[k], sid, vida)?
                };
                n.nasce = None;
                n.continua = doc.tracks[k].continua;
                doc.tracks[k] = n;
            }
        }
        // Todas as trilhas das outras famílias vão para o alvo.
        let objs: BTreeMap<usize, u64> = (0..doc.tracks.len())
            .filter(|k| fams[*k] != FamiliaDaMacro::Mundo)
            .map(|k| (k, alvo))
            .collect();
        let destino = crate::mundo::aplica::Destino {
            raiz,
            trilhas: &objs,
            por_cima: true,
        };
        crate::mundo::aplica::aplica(&mut doc, fams, &destino, sim, portas, &mut nascidos)
    })();
    match r {
        Ok(()) => Ok((raiz, doc)),
        Err(e) => {
            desfaz(sim, &nascidos);
            Err(e)
        }
    }
}
