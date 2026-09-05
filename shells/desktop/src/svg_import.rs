//! ⭐⭐⭐ **UM `.svg` LARGADO NA JANELA VIRA FORMAS EDITÁVEIS** (estudo 42, item 3).
//!
//! A tradução vive na crate-folha [`ph2d_vec_svg`]; aqui mora o que é da SHELL — onde o desenho
//! aterra, que nome cada forma leva na Hierarquia, e como os `<g>` do ficheiro viram grupos.
//!
//! # Três leis, e as três já existiam nesta casa
//!
//! 1. **Um px é um px** — o mesmo divisor (`pixels_per_meter`) que dimensiona uma sprite
//!    importada. Um `.svg` de 512 unidades entra do tamanho de um `.png` de 512 px.
//! 2. **Um path ⟺ uma entidade** — quem as cria é o [`crate::vec_entities::sync`], e não este
//!    módulo: pôr aqui um segundo criador de entidades seria a segunda porta pela qual um path
//!    órfão nasce.
//! 3. **Agrupar é o verbo que já existe** ([`crate::vec_entities::group_entities`]) — ele
//!    põe o grupo entre os filhos, compensa a pose de cada um e ordena a lista. ⚠️ Ele exige
//!    **dois** membros, então um `<g>` com um filho só é achatado; é a mesma regra que o artista
//!    lê no menu (*"Select at least 2 objects to group"*).
//!
//! ⚠️ **O ficheiro inteiro vira UM objecto** quando traz mais de uma coisa de topo: sem isso um
//! logótipo de 40 formas aterraria como 40 raízes na Hierarquia, e mover o desenho seria
//! impossível sem antes o seleccionar todo.

use ph2d_ecs::{Name, SimWorld};
use ph2d_vec_scene::{VecPathId, VecScene, Xform, bake_xform};
use std::path::Path;

/// As extensões que este importador reclama. ⚠️ O `.svgz` é um `.svg` comprimido, e o usvg
/// descomprime-o sozinho (a feature `svgz` da crate) — recusá-lo aqui seria uma recusa inventada.
pub(crate) const SVG_EXTENSIONS: &[&str] = &["svg", "svgz"];

/// É um desenho vectorial? ⛔ **Não** passa pelo `is_supported_image_extension`: um `.svg` não é
/// uma imagem neste app — se fosse, entraria como sprite e o artista receberia pixels.
pub(crate) fn is_svg_file(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| SVG_EXTENSIONS.iter().any(|k| e.eq_ignore_ascii_case(k)))
}

/// O que a leitura de UM ficheiro produziu.
pub(crate) enum SvgImportResult {
    Ok {
        name: String,
        shapes: usize,
        /// A entidade que representa o desenho: o grupo, ou a única forma.
        bits: u64,
        /// Quanto o desenho ocupa em mundo — é o passo da fila do próximo.
        size: [f64; 2],
        /// ⛔ O que o ficheiro carrega e o documento não exprime, já com a contagem.
        notes: Vec<String>,
    },
    Err {
        name: String,
        error: String,
    },
}

fn nome_do_ficheiro(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Drawing")
        .to_owned()
}

/// **Lê um `.svg` e põe-no na cena**, centrado em `centro`.
pub(crate) fn import_svg(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &mut crate::vec_entities::VecEntityMap,
    path: &Path,
    centro: [f32; 2],
    pixels_per_meter: f32,
) -> SvgImportResult {
    let ficheiro = nome_do_ficheiro(path);
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            return SvgImportResult::Err {
                name: ficheiro,
                error: format!("read: {e}"),
            };
        }
    };
    let desenho = match ph2d_vec_svg::import(
        &bytes,
        &ph2d_vec_svg::Options {
            pixels_per_meter: f64::from(pixels_per_meter),
        },
    ) {
        Ok(d) => d,
        Err(e) => {
            return SvgImportResult::Err {
                name: ficheiro,
                error: e.to_string(),
            };
        }
    };
    if desenho.shapes.is_empty() {
        // ⚠️ Recusa em voz alta, e não um documento vazio: um `.svg` só com `<text>` (ou só com uma
        // imagem embutida) entra por aqui, e sem esta linha o artista veria "Imported" e nada no
        // ecrã. A nota diz-lhe o que o ficheiro tinha.
        let porque = if desenho.notes.is_empty() {
            "no drawable shape".to_owned()
        } else {
            desenho.notes.join("; ")
        };
        return SvgImportResult::Err {
            name: ficheiro,
            error: porque,
        };
    }
    // O desenho vem centrado na origem; a shell é que sabe ONDE ele aterra.
    let ao_ponto = Xform([
        1.0,
        0.0,
        0.0,
        1.0,
        f64::from(centro[0]),
        f64::from(centro[1]),
    ]);
    let mut ids: Vec<VecPathId> = Vec::with_capacity(desenho.shapes.len());
    for s in &desenho.shapes {
        let mut p = s.path.clone();
        bake_xform(&mut p, &ao_ponto);
        ids.push(scene.push_path(p));
    }
    // ⚠️ **A porta ÚNICA path→entidade.** Chamá-la aqui (e não esperar pelo prólogo do frame
    // seguinte) é o que permite nomear e agrupar no MESMO gesto — e um gesto que só se completa no
    // frame seguinte é um gesto que o undo parte ao meio.
    crate::vec_entities::sync(sim, scene, map);
    // ⚠️⚠️ **ANTES de agrupar, e a ordem é load-bearing.** O `settle_origins` só toca em formas
    // **sem pai** e na identidade; agrupar primeiro punha um `ChildOf` em cada uma e elas ficavam
    // para sempre com o pivô na origem do mundo — e o grupo, cuja pose é a média das poses dos
    // membros, nascia lá também, com o gizmo longe do desenho. É o mesmo defeito que o report do
    // Enio de 30/08 curou para o verbo *Group*, por outra porta.
    crate::vec_transform::settle_origins(sim, scene, map, &[]);

    let entidades: Vec<u64> = ids.iter().filter_map(|id| map.get(id).copied()).collect();
    baptiza(sim, &desenho, &entidades, &ficheiro);
    let bits = agrupa(sim, &desenho, &entidades, &ficheiro);

    SvgImportResult::Ok {
        name: ficheiro,
        shapes: desenho.shapes.len(),
        bits: bits.unwrap_or_else(|| entidades.first().copied().unwrap_or(0)),
        size: desenho.size,
        notes: desenho.notes,
    }
}

/// Cada forma leva o `id` que o ficheiro lhe deu — e o nome passa pela porta do nome ÚNICO, porque
/// nesta casa **o nome é identidade** (a animação reencontra o objecto pelo hash dele).
fn baptiza(sim: &mut SimWorld, desenho: &ph2d_vec_svg::Drawing, entidades: &[u64], ficheiro: &str) {
    for (s, bits) in desenho.shapes.iter().zip(entidades) {
        let base = if s.name.is_empty() { ficheiro } else { &s.name };
        let nome = crate::name_unique::unique_name(sim, base);
        if let Some(mut n) = sim
            .world_mut()
            .get_mut::<Name>(ph2d_ecs::Entity::from_bits(*bits))
        {
            *n = Name::new(nome);
        }
    }
}

/// Reconstrói a árvore de `<g>` do ficheiro, **de dentro para fora**.
///
/// ⚠️ A ordem é load-bearing: um grupo interior tem de existir (e ser raiz) antes de o exterior o
/// reclamar como membro. A lista de grupos vem em ordem de descida, então percorrê-la ao contrário
/// dá exactamente os filhos primeiro.
///
/// Devolve o objecto de topo do desenho, quando há um só.
fn agrupa(
    sim: &mut SimWorld,
    desenho: &ph2d_vec_svg::Drawing,
    entidades: &[u64],
    ficheiro: &str,
) -> Option<u64> {
    let mut feitos: Vec<Option<u64>> = vec![None; desenho.groups.len()];
    for g in (0..desenho.groups.len()).rev() {
        let mut membros: Vec<u64> = desenho
            .shapes
            .iter()
            .zip(entidades)
            .filter(|(s, _)| s.group == Some(g))
            .map(|(_, b)| *b)
            .collect();
        membros.extend(
            desenho
                .groups
                .iter()
                .enumerate()
                .filter(|(i, sub)| sub.parent == Some(g) && feitos[*i].is_some())
                .filter_map(|(i, _)| feitos[i]),
        );
        let nome = crate::name_unique::unique_name(sim, &desenho.groups[g].name);
        feitos[g] = crate::vec_entities::group_entities(sim, &membros, nome);
    }
    // O que ficou na raiz: as formas sem grupo e os grupos sem pai.
    let mut topo: Vec<u64> = desenho
        .shapes
        .iter()
        .zip(entidades)
        .filter(|(s, _)| s.group.is_none())
        .map(|(_, b)| *b)
        .collect();
    topo.extend(
        desenho
            .groups
            .iter()
            .enumerate()
            .filter(|(i, g)| g.parent.is_none() && feitos[*i].is_some())
            .filter_map(|(i, _)| feitos[i]),
    );
    if topo.len() == 1 {
        return topo.first().copied();
    }
    // ⚠️ **O ficheiro inteiro vira um objecto.** Sem isto um logótipo de 40 formas aterra como 40
    // raízes, e não há gesto que o mova inteiro sem o artista o seleccionar todo primeiro.
    let nome = crate::name_unique::unique_name(sim, ficheiro);
    crate::vec_entities::group_entities(sim, &topo, nome)
}

#[cfg(test)]
#[path = "svg_import_tests.rs"]
mod tests;
