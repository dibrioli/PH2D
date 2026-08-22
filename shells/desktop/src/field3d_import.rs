//! ⭐ **A PORTA DE ENTRADA da escultura no modelo implícito** — um arquivo de malha vira um nó da
//! peça, e a booleana passa a poder cortá-lo ([ADR-0161], plano W5).
//!
//! # ⚠️ Ela fecha o par com a saída, e o par é o produto
//!
//! A W19 escreveu a **saída** (`field3d_export`) e a W21 o **motor** da entrada. O que faltava era o
//! corredor: até aqui o único sítio do app onde uma escultura existia como campo era a cena 6 do
//! smoke, que fabrica a malha sozinha. *Uma porta sem corredor é código morto com a suíte verde.*
//!
//! # ⚠️ Nada de segundo leitor de malha
//!
//! Os três formatos e a leitura vêm do [`crate::sculpt3d::import::read_pieces`], que a escultura já
//! tinha, com os gates dela. Uma cópia local diria uma coisa e a original outra no dia em que
//! qualquer um dos três parsers mudasse.
//!
//! # ⚠️ As duas metades da pose, e por que a geometria NÃO é reescrita
//!
//! O arquivo pode vir em qualquer escala — 300 unidades ou 0,01. O campo é construído da malha
//! **como ela está** (é isso que faz a célula da grade ser a resolução real do arquivo), e o
//! tamanho de convivência vai para a **pose do nó**: `SampledLeaf::at` desfaz a pose na entrada e
//! multiplica o valor pela escala na saída, então o campo continua a ser uma distância.
//!
//! ⚠️ **O CENTRO, esse, é reescrito** ([`ph2d_mesh::Mesh::recenter`]), e por um mecanismo: a caixa
//! da grade nasce da caixa da malha, então uma peça longe da origem paga uma grade que é quase toda
//! vazio. Recentrar é o que faz a resolução ir para onde há geometria.
//!
//! [ADR-0161]: ../../../docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md

use ph2d_mesh::{MeshFormat, Pose};

/// ⭐ **Que fração do enquadramento uma escultura importada ocupa.**
///
/// ⚠️ **É o mesmo problema que o import da escultura resolveu**, e a resposta é a mesma: um arquivo
/// de 300 unidades ao lado de uma caixa de 1 torna a segunda invisível, e o artista conclui que a
/// importação falhou. Meia extensão do enquadramento deixa a peça inteira à vista com margem para o
/// gizmo, e o número original do autor **não é reescrito** — ele vive na pose, onde um clique o
/// desfaz.
const FRAMING_FRACTION: f32 = 0.5;

/// Abre o diálogo, lê o arquivo, constrói o campo e **anota** a escultura para o próximo quadro.
///
/// ⚠️ Ela devolve o nome (o caminho) por um canal e não cria o nó: quem tem o `&mut World` é a ponte
/// com a cena, e quem pode abrir um diálogo é o app. É a mesma divisão da exportação.
pub(crate) fn field3d_import(toasts: &mut ph2d_editor::ToastQueue) {
    let say =
        |toasts: &mut ph2d_editor::ToastQueue, m: String| toasts.push(ph2d_editor::Toast::info(m));

    // ⚠️ **UM FILTRO POR FORMATO**, a mesma lição do export: com um filtro único o diálogo nativo
    // completa o nome com a primeira extensão da lista.
    let mut dialog = rfd::FileDialog::new();
    for f in MeshFormat::ALL {
        dialog = dialog.add_filter(f.extension().to_uppercase(), &[f.extension()]);
    }
    let Some(path) = dialog.pick_file() else {
        return;
    };

    let pieces = match crate::sculpt3d::import::read_pieces(&path) {
        Ok(p) if !p.is_empty() => p,
        Ok(_) => {
            say(toasts, "That file has no mesh in it".into());
            return;
        }
        Err(e) => {
            say(toasts, format!("Could not read the mesh: {e}"));
            return;
        }
    };

    // ⚠️ **As peças de um OBJ viram UM corpo.** Uma escultura entra na booleana como uma coisa só —
    // um `Difference` contra "três peças" precisaria de escolher qual, e a resposta certa é que o
    // arquivo é a escultura.
    let refs: Vec<(&ph2d_mesh::Mesh, Pose)> =
        pieces.iter().map(|p| (&p.mesh, Pose::IDENTITY)).collect();
    let mut mesh = match ph2d_mesh::merge(&refs) {
        Ok(m) => m,
        Err(e) => {
            say(toasts, format!("Could not merge the pieces: {e:?}"));
            return;
        }
    };
    let tris = mesh.faces().len();
    mesh.recenter();
    let extent = {
        let b = mesh.bounds();
        (b.max[0] - b.min[0])
            .max(b.max[1] - b.min[1])
            .max(b.max[2] - b.min[2])
            .max(f32::MIN_POSITIVE)
    };

    let t0 = std::time::Instant::now();
    let Some(field) =
        ph2d_field_mesh::SampledField::from_mesh(&mesh, ph2d_field_mesh::DEFAULT_RESOLUTION)
    else {
        say(toasts, "That mesh is empty".into());
        return;
    };
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    let cell = field.cell();

    // ⭐ **A chave é o CAMINHO**, e é isso que torna a persistência possível sem guardar a grade.
    let key = path.to_string_lossy().to_string();
    crate::field3d_smoke::register_sampled(&key, std::sync::Arc::new(field));
    crate::field3d_smoke::ask_spawn_sculpt(key);
    crate::field3d_smoke::ask_sculpt_extent(extent);

    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("?");
    say(
        toasts,
        format!("Imported {name}: {tris} tris -> field in {ms:.0} ms (detail {cell:.4})"),
    );
}

/// A escala que põe uma peça de extensão `extent` no enquadramento.
///
/// ⚠️ **Função pura, e é ela que o gate dirige**: a decisão que pode estar errada — *a peça cabe no
/// quadro? o número do autor sobreviveu?* — não tem nada a ver com um diálogo nem com um device.
pub(crate) fn framing_scale(extent: f32, half_extent: f32) -> f32 {
    if !extent.is_finite() || extent <= 0.0 {
        return 1.0;
    }
    (half_extent * 2.0 * FRAMING_FRACTION / extent).max(f32::MIN_POSITIVE)
}

#[cfg(test)]
#[path = "field3d_import_tests.rs"]
mod tests;
