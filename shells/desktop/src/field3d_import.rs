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

/// O que um arquivo de malha dá: o campo, mais os números que o artista lê.
pub(crate) struct Loaded {
    pub field: ph2d_field_mesh::SampledField,
    /// A maior aresta da caixa da malha, **nas unidades do arquivo** — a entrada do enquadramento.
    pub extent: f32,
    pub tris: usize,
    pub millis: f64,
}

/// ⭐ **UMA resposta a "que campo este arquivo dá"** — e é por isso que ela é uma função e não o
/// corpo do diálogo.
///
/// ⚠️ **O recarregamento (W23) chama exactamente esta**, e é o que impede a divergência mais cara
/// deste desenho: o documento guarda o **caminho**, não a grade, então o que volta do arquivo tem de
/// ser byte-a-byte o que entrou por ele. Uma segunda cópia — com outra resolução, ou sem o
/// `recenter` — daria uma peça que muda de forma ao reabrir o projeto, e **sem nada na tela a
/// dizê-lo**.
///
/// # Errors
/// A mensagem é a que o artista vê no aviso; ela nomeia o que falhou, nunca o mecanismo.
pub(crate) fn field_from_file(path: &std::path::Path) -> Result<Loaded, String> {
    let pieces = match crate::sculpt3d::import::read_pieces(path) {
        Ok(p) if !p.is_empty() => p,
        Ok(_) => return Err("that file has no mesh in it".into()),
        Err(e) => return Err(format!("could not read it ({e})")),
    };

    // ⚠️ **As peças de um OBJ viram UM corpo.** Uma escultura entra na booleana como uma coisa só —
    // um `Difference` contra "três peças" precisaria de escolher qual, e a resposta certa é que o
    // arquivo é a escultura.
    let refs: Vec<(&ph2d_mesh::Mesh, Pose)> =
        pieces.iter().map(|p| (&p.mesh, Pose::IDENTITY)).collect();
    let mesh =
        ph2d_mesh::merge(&refs).map_err(|e| format!("could not merge its pieces ({e:?})"))?;
    field_from_mesh(mesh)
}

/// ⭐ **UMA resposta a "que campo esta MALHA dá"** — e ela serve as duas portas.
///
/// ⚠️ **Factorizada na W39**, quando a escultura passou a poder vir da **cena** em vez do disco. As
/// duas portas têm de produzir o **mesmo** campo: o documento guarda uma **chave**, não a grade, e
/// duas voxelizações diferentes dariam uma peça que muda de forma conforme por onde entrou — sem
/// nada na tela a dizê-lo.
///
/// ⚠️ **O `recenter` é load-bearing** (e não arrumação): a caixa da grade nasce da caixa da malha,
/// então uma peça longe da origem paga uma grade que é quase toda vazio. O tamanho de convivência
/// vai para a **pose** do nó, não para a geometria.
///
/// ⚠️ **Custa 229–389 ms a 128³** (medido, `measure_sculpt_to_field_bridge`), e é por isso que este
/// caminho é um **gesto** e não um vínculo contínuo: uma re-voxelização por edição de escultura são
/// 14 a 23 quadros de congelamento. O doc da `DEFAULT_RESOLUTION` já dizia *"o custo é pago uma
/// vez, na importação, não por quadro"* — esta linha é essa decisão a valer também para a cena.
///
/// # Errors
/// A mensagem é a que o artista vê no aviso; ela nomeia o que falhou, nunca o mecanismo.
pub(crate) fn field_from_mesh(mut mesh: ph2d_mesh::Mesh) -> Result<Loaded, String> {
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
    let field =
        ph2d_field_mesh::SampledField::from_mesh(&mesh, ph2d_field_mesh::DEFAULT_RESOLUTION)
            .ok_or_else(|| "that mesh is empty".to_string())?;
    Ok(Loaded {
        field,
        extent,
        tris,
        millis: t0.elapsed().as_secs_f64() * 1000.0,
    })
}

/// ⭐ **A CHAVE da escultura da cena** (W39).
///
/// ⚠️ **Não é um caminho de arquivo, e o resolvedor tem de o saber**: um `scene:` não se lê do
/// disco. Quem o re-registra ao reabrir o projeto é o shell, a partir da escultura que o
/// `ProjectFile` já guarda — ver `field3d_reload::missing_keys`.
pub(crate) const SCENE_PREFIX: &str = "scene:";

/// A chave da escultura viva, derivada do prefixo — nunca dois literais que possam divergir.
pub(crate) const SCENE_KEY: &str = "scene:sculpt";

/// ⭐ **Traz a escultura VIVA da cena para dentro da peça** — sem disco no meio.
///
/// Devolve a mensagem que o artista lê.
pub(crate) fn field3d_scene_sculpt(mesh: ph2d_mesh::Mesh) -> String {
    let loaded = match field_from_mesh(mesh) {
        Ok(l) => l,
        Err(e) => return format!("Could not use the scene sculpture: {e}"),
    };
    let (tris, ms, cell) = (loaded.tris, loaded.millis, loaded.field.cell());
    crate::field3d_smoke::register_sampled(SCENE_KEY, std::sync::Arc::new(loaded.field));
    crate::field3d_smoke::ask_spawn_sculpt(SCENE_KEY.to_string());
    crate::field3d_smoke::ask_sculpt_extent(loaded.extent);
    format!("Scene sculpture in: {tris} tris -> field in {ms:.0} ms (detail {cell:.4})")
}

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
    // ⚠️ **Pela PORTA** (`crate::modal`), pela razão escrita no irmão do export: o diálogo congela
    // o loop, e sem declarar isso a mensagem escrita a seguir vive um quadro só.
    let Some(path) = crate::modal::pick_file(dialog) else {
        return;
    };

    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("?")
        .to_string();
    let loaded = match field_from_file(&path) {
        Ok(l) => l,
        Err(e) => {
            say(toasts, format!("Could not import {name}: {e}"));
            return;
        }
    };
    let (tris, ms, cell) = (loaded.tris, loaded.millis, loaded.field.cell());

    // ⭐ **A chave é o CAMINHO**, e é isso que torna a persistência possível sem guardar a grade.
    let key = path.to_string_lossy().to_string();
    crate::field3d_smoke::register_sampled(&key, std::sync::Arc::new(loaded.field));
    crate::field3d_smoke::ask_spawn_sculpt(key);
    crate::field3d_smoke::ask_sculpt_extent(loaded.extent);

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
