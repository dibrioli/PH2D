//! ⭐⭐⭐⭐ **A TINTA FINA SAI NO FICHEIRO** — filho (`#[path]`) do
//! [`super::export`], cortado dele pelo ASSUNTO: lá *que formato e que nome*,
//! aqui *como milhões de amostras viram um `.png` que outro programa abre*.
//!
//! ⚠️ **A metade que faltava desde 22/09.** A wave do aviso ensinou a saída a
//! DIZER que a tinta fina não era carregada; esta ensina-a a **carregá-la**.
//! ⇒ o `Lost:` de um `.obj` deixa de a nomear, e isso não é uma frase nova: é a
//! mesma tabela ([`MeshFormat::keeps_fine_paint`]) a responder outra coisa.
//!
//! # ⛔ Três ficheiros e não um, e é o formato que obriga
//!
//! Um `.obj` não embute imagem: ele aponta para um `.mtl`, que aponta para o
//! `.png`. Os três saem **lado a lado**, com o nome derivado do que o artista
//! escreveu — e o `map_Kd` leva o nome **sem pasta**, senão o material quebra
//! no computador de quem o abrir.
//!
//! # ⚠️ Uma textura POR PEÇA, e não uma para a cena
//!
//! Cada peça tem o plano dela, com a topologia dela — um assado único obrigaria
//! a re-endereçar as amostras de todas as peças num espaço comum, que é
//! exactamente o trabalho que a família do atlas faz e que esta não precisa de
//! fazer. *Duas peças são dois materiais, que é como todo DCC as escreve.*

use ph2d_mesh::{MeshFormat, UvDaPeca};
use ph2d_mesh_colors::{Assado, Recusa};

use super::Sculpt3dScene;

/// ⭐ **O maior lado de textura que esta saída aceita.**
///
/// ⚠️ **O recurso é a PLACA de quem abrir o ficheiro:** `8192` é o
/// `max_texture_dimension_2d` que esta casa já usa como tecto noutro
/// exportador (`MAX_SHEET_EDGE_PX`, na folha de sprites), e uma textura maior
/// que isso é um ficheiro que o destino recusa a carregar. ⛔ Ele **não** é um
/// palpite sobre o peso do ficheiro — esse é outra grandeza, e não é esta que
/// a limita.
pub(crate) const TECTO_DE_TEXELS: u32 = 8192;

/// O que a cena entregou ao escritor.
#[derive(Default)]
pub(crate) struct Assados {
    /// Paralelo a `scene.objects` — `None` é uma peça sem tinta fina.
    pub(crate) por_peca: Vec<Option<Assado>>,
    /// As peças que **tinham** plano e não couberam, com o motivo.
    pub(crate) recusas: Vec<(usize, Recusa)>,
}

impl Assados {
    /// Alguma peça assou de verdade?
    pub(crate) fn alguma(&self) -> bool {
        self.por_peca.iter().any(Option::is_some)
    }
}

/// ⭐⭐ **Assa o plano de cada peça que o tem.**
///
/// ⚠️ **Ele pergunta ao DONO do empréstimo** ([`crate::tinta_da_peca::plano_da_peca`]):
/// durante um traço o plano está EMPRESTADO ao gesto e o `Option` da peça está
/// vazio — perguntar à peça devolveria *«não há tinta fina»* a meio de uma
/// pincelada. *A mesma porta que o aviso de 22/09 teve de aprender.*
pub(crate) fn assa(scene: &Sculpt3dScene) -> Assados {
    let mut out = Assados::default();
    for i in 0..scene.objects.len() {
        let Some(tinta) = scene.plano_de(i) else {
            out.por_peca.push(None);
            continue;
        };
        let mesh = scene.objects[i].stack.mesh();
        match ph2d_mesh_colors::assar(
            tinta,
            mesh.faces().iter().map(ph2d_mesh::Face::verts),
            TECTO_DE_TEXELS,
        ) {
            Ok(a) => out.por_peca.push(Some(a)),
            Err(e) => {
                out.recusas.push((i, e));
                out.por_peca.push(None);
            }
        }
    }
    out
}

/// Os nomes dos três ficheiros, derivados do que o artista escreveu.
///
/// ⚠️ **Separado de quem grava, porque ESTA metade é testável** — a outra toca
/// no disco. É o corte que o `sheet_export` da shell já fazia pela mesma razão.
pub(crate) fn nomes(path: &std::path::Path, pecas: &[Option<Assado>]) -> (String, Vec<String>) {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("sculpt")
        .to_owned();
    let pngs = pecas
        .iter()
        .enumerate()
        .map(|(i, a)| {
            if a.is_some() {
                format!("{stem}_{i}.png")
            } else {
                String::new()
            }
        })
        .collect();
    (format!("{stem}.mtl"), pngs)
}

/// O nome do material da peça `i`.
pub(crate) fn material(i: usize) -> String {
    format!("ph2d_{i}")
}

/// ⭐ **Grava as texturas e a biblioteca de materiais, ao lado do `.obj`.**
///
/// Devolve o nome do `.mtl` se **tudo** foi gravado. ⛔ Em erro devolve `None`
/// e diz QUAL ficheiro falhou — e quem chama volta a escrever um `.obj` sem
/// textura, que é um ficheiro **coerente**. *Um `.obj` a apontar para um
/// material que não existe é pior que um `.obj` sem tinta: o destino abre-o
/// preto e ninguém sabe porquê.*
pub(crate) fn grava(
    path: &std::path::Path,
    assados: &Assados,
    toasts: &mut ph2d_editor_core::ToastQueue,
) -> Option<String> {
    let (mtl_nome, pngs) = nomes(path, &assados.por_peca);
    let dir = path.parent().unwrap_or(std::path::Path::new("."));
    let mut materiais = Vec::new();
    for (i, a) in assados.por_peca.iter().enumerate() {
        let Some(a) = a else { continue };
        let alvo = dir.join(&pngs[i]);
        // ⚠️ **TRÊS canais, e a razão está no doc da [`Assado::rgb`]:** o alfa
        //    do assado é a COBERTURA, e entregá-lo a um `.png` faz o destino
        //    ler a peça como translúcida (medido no Blender 5.2.2).
        if let Err(e) = image::save_buffer(
            &alvo,
            &a.rgb(),
            a.lado_px,
            a.lado_px,
            image::ColorType::Rgb8,
        ) {
            crate::import::toast(
                toasts,
                ph2d_i18n::tr_with(
                    "app.sculpt3d.export.texture_failed",
                    &[("name", &pngs[i]), ("e", &e)],
                ),
            );
            return None;
        }
        materiais.push((material(i), pngs[i].clone()));
    }
    if materiais.is_empty() {
        return None;
    }
    let alvo = dir.join(&mtl_nome);
    if let Err(e) = std::fs::write(&alvo, ph2d_mesh::write_mtl(&materiais)) {
        crate::import::toast(
            toasts,
            ph2d_i18n::tr_with(
                "app.sculpt3d.export.texture_failed",
                &[("name", &mtl_nome), ("e", &e)],
            ),
        );
        return None;
    }
    Some(mtl_nome)
}

/// As coordenadas que o escritor do `.obj` recebe, uma entrada por peça.
pub(crate) fn uvs<'a>(assados: &'a Assados, materiais: &'a [String]) -> Vec<Option<UvDaPeca<'a>>> {
    assados
        .por_peca
        .iter()
        .enumerate()
        .map(|(i, a)| {
            a.as_ref().map(|a| UvDaPeca {
                uv: &a.uv,
                off: &a.off_uv,
                material: &materiais[i],
            })
        })
        .collect()
}

/// ⚠️ **A frase de uma recusa nomeia a PEÇA e a CURA** — sem ela o artista vê
/// um `.obj` sem tinta e não sabe se o pincel falhou, se o formato não a leva,
/// ou se a textura não coube. *As três curas são diferentes.*
pub(crate) fn frase_da_recusa(recusas: &[(usize, Recusa)]) -> Option<String> {
    let (i, e) = recusas.first()?;
    Some(ph2d_i18n::tr_with(
        "app.sculpt3d.export.texture_too_big",
        &[("n", &(i + 1)), ("why", &e.to_string())],
    ))
}

/// A tabela do que um formato carrega, respondida para ESTA exportação.
///
/// ⚠️⚠️ **Ter tinta fina e PERDÊ-LA são perguntas diferentes, e o aviso fala da
/// segunda.** O `.obj` leva-a agora (é o que o [`MeshFormat::keeps_fine_paint`]
/// declara) — mas só se ela de facto assou: uma peça recusada por tamanho tem
/// tinta fina e ela **fica para trás**, e um aviso que se calasse ali seria a
/// resposta errada com a confiança da certa.
pub(crate) fn perdeu_tinta_fina(fmt: MeshFormat, tem: bool, assados: &Assados) -> bool {
    tem && (!fmt.keeps_fine_paint() || !assados.alguma())
}

#[cfg(test)]
#[path = "export_assado_tests.rs"]
mod export_assado_tests;

#[cfg(test)]
#[path = "export_assado_sonda.rs"]
mod export_assado_sonda;

#[cfg(test)]
#[path = "export_assado_relogio.rs"]
mod export_assado_relogio;

/// ⭐⭐⭐ **A SAÍDA DIZ, NO TERMINAL, o que escreveu e onde.**
///
/// ⚠️ **Ela nasceu de um report** (dono, 22/09: *«o Blender não consegue
/// importar e não dá nenhuma mensagem»*), e a razão de ser TERMINAL é medida: o
/// balão tem `48` caracteres ([§22](../../../docs/3D/handoffs/HANDOFF_line_sculpt3d_A_TINTA_FINA_SOBREVIVE_2026-09-21.md))
/// e não cabe um caminho — *o terminal é a única superfície onde o artista e eu
/// olhamos para os MESMOS números*.
///
/// ⛔ Ela corre **depois** de o `.obj` estar em disco: escrita antes, ela mede
/// o ficheiro que lá estava de uma exportação anterior, e *um instrumento que
/// mede o ficheiro errado é pior que nenhum — ele CONFIRMA*.
pub(crate) fn diz_o_que_escreveu(path: &std::path::Path, assados: &Assados, bytes: usize) {
    let lado = assados
        .por_peca
        .iter()
        .flatten()
        .map(|a| a.lado_px)
        .max()
        .unwrap_or(0);
    let texturas = assados.por_peca.iter().flatten().count();
    eprintln!(
        "[sculpt3d] exportado: {} ({bytes} bytes) + {texturas} textura(s) de \
         {lado}x{lado} px + material",
        path.display()
    );
}
