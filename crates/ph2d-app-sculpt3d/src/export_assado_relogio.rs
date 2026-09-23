//! ⭐⭐⭐ **O RELÓGIO DA SAÍDA** — quanto tempo o app fica PARADO a assar.
//!
//! ⚠️ **Ela existe por um report do dono** (22/09: *«o Blender não consegue
//! importar e não dá nenhuma mensagem»*), e a pergunta que ela faz não é sobre
//! o formato: o ficheiro que o escritor produz **importa** no Blender 5.2.2
//! (medido, com o alvo a correr sobre um ficheiro nosso). O que falta medir é a
//! outra ponta — *quanto custa produzi-lo, na thread que DESENHA*.
//!
//! ⛔⛔ **Esta casa já pagou este defeito uma vez, e está escrito:** a
//! exportação do modelador 3D custava `8 min 17 s` e **saiu da thread que
//! desenha**, porque *declarar o congelamento curava a mensagem e não o
//! congelamento* — e a `12 s` o KDE dá a janela por morta e oferece forçar o
//! encerramento. *Uma janela dada por morta é indistinguível, para quem a usa,
//! de um ficheiro que não saiu.*
//!
//! Corre-se com `PH2D_RELOGIO_SAIDA=1` e `-- --nocapture`. ⛔ Sem a variável
//! ela não corre: é uma medição de relógio, e o §5.0 proíbe que uma suíte
//! inteira pague por ela.

use ph2d_mesh_colors::Tinta;

/// As densidades que o artista alcança com um clique.
///
/// ⚠️ **A peça de FÁBRICA do módulo está aqui de propósito:** a cena `=52` abre
/// grossa, e o `K` do roteiro da `=14` leva o artista à de fábrica em duas
/// teclas. *Medir só a cena mede o caso em que o defeito não aparece.*
fn pecas() -> Vec<(&'static str, ph2d_mesh::Mesh)> {
    vec![
        ("a cena =52", crate::scenes::tinta_fina::peca()),
        ("media", ph2d_mesh::shapes::uv_sphere(96, 128, 1.0)),
        ("de fabrica", ph2d_mesh::shapes::uv_sphere(222, 296, 1.0)),
    ]
}

#[test]
#[ignore]
fn diag_quanto_custa_assar() {
    if std::env::var("PH2D_RELOGIO_SAIDA").is_err() {
        return;
    }
    eprintln!("\n-- O RELOGIO DA SAIDA (o app fica PARADO isto) --------------");
    eprintln!(
        "{:<14} {:>8} {:>8} {:>10} {:>9} {:>9} {:>9}",
        "peca", "faces", "nivel", "textura", "assar", "png", "TOTAL"
    );
    for (nome, mesh) in pecas() {
        for nivel in [1u8, 3] {
            let tinta = Tinta::nova(
                mesh.vert_count(),
                mesh.faces().iter().map(ph2d_mesh::Face::verts),
                nivel,
            );
            let t0 = std::time::Instant::now();
            let assado = match ph2d_mesh_colors::assar(
                &tinta,
                mesh.faces().iter().map(ph2d_mesh::Face::verts),
                super::TECTO_DE_TEXELS,
            ) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!(
                        "{nome:<14} {:>8} {:>8} {:>10}",
                        mesh.faces().len(),
                        1u32 << nivel,
                        format!("RECUSA {e}")
                    );
                    continue;
                }
            };
            let t_assar = t0.elapsed();

            // ⚠️ O PNG é medido a CODIFICAR para memória e não para disco: o
            //    disco mede o SSD de quem corre, e o que o app paga sempre é a
            //    codificação.
            let t1 = std::time::Instant::now();
            let mut bytes: Vec<u8> = Vec::new();
            image::codecs::png::PngEncoder::new(std::io::Cursor::new(&mut bytes))
                .write_image(
                    &assado.rgba,
                    assado.lado_px,
                    assado.lado_px,
                    image::ExtendedColorType::Rgba8,
                )
                .expect("png");
            let t_png = t1.elapsed();

            eprintln!(
                "{nome:<14} {:>8} {:>8} {:>10} {:>8.2}s {:>8.2}s {:>8.2}s",
                mesh.faces().len(),
                1u32 << nivel,
                format!("{}px", assado.lado_px),
                t_assar.as_secs_f32(),
                t_png.as_secs_f32(),
                (t_assar + t_png).as_secs_f32()
            );
        }
    }
    eprintln!("------------------------------------------------------------\n");
}

use image::ImageEncoder;
