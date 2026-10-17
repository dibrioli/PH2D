// O censo dos controlos (`censo_dos_controlos.rs`, que o inclui): o ruído do Wet Paint e os controlos
// esmaecidos — esmaecido ⇔ não age.

/// O RUÍDO do Wet Paint: a mesma base, oito vezes em paralelo, sem e com tinta já na tela. A água anda
/// ao relógio (`wetpaint/offthread.rs`), e um censo cujo «vivo» é ruído esconde um morto.
#[test]
#[ignore = "sonda: o piso de ruído do Wet Paint ao relógio real"]
fn sonda_o_ruido_do_wet_paint() {
    for armar in [Vec::new(), vec![Gesto::Traco]] {
        let corridas: Vec<Vec<u8>> = em_paralelo(&[0u8; 8], |_| {
            mede(PaintMedia::WetPaint, &armar, &Gesto::Nenhum).0
        });
        let piores: Vec<usize> = corridas[1..]
            .iter()
            .map(|c| diferenca(&corridas[0], c).0)
            .collect();
        eprintln!("RUIDO-WET armar={armar:?} texels diferentes da 1.ª: {piores:?}");
    }
}

/// Os ids que a ferramenta pode declarar inertes ([`ph2d_tool_painter::Dependente`] e os parâmetros
/// dos padrões) — a população do gate do esmaecido.
fn ids_dependentes() -> Vec<NodeId> {
    let mut v: Vec<NodeId> = ph2d_tool_painter::Dependente::TODOS
        .iter()
        .flat_map(|d| d.ids().iter().copied())
        .collect();
    v.extend(ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_PARAMS);
    v.extend(ph2d_tool_painter::ids::PAINTER_SHAPE_PARAMS);
    v
}

/// ⭐⭐⭐ **A LINHA ESMAECIDA É A QUE NÃO AGE — e só ela** (decisão do dono, 2026-10-04; doc 46 §2-3).
///
/// Em cada estado da tabela das pré-condições (e na fábrica do meio), cada controlo dependente que a
/// tela oferece é conduzido pela porta do ponteiro: a linha tem de estar esmaecida (a dica no id) SE E
/// SÓ SE a tinta não mudou. Esmaecida e viva = a tela mente que não age; viva e inerte sem esmaecer =
/// o controlo morto que esta decisão existe para mostrar.
///
/// **Mutações que sangram:** inverter a lei do Space Attenuation (`space_attenuation_reaches`); tirar
/// o `talvez_esmaecido` da linha de número; o Taper esmaecer com comprimento.
#[test]
fn a_linha_esmaecida_e_a_que_nao_age() {
    let nomes = nomes();
    let dependentes = ids_dependentes();
    let mut estados: Vec<(PaintMedia, Vec<Gesto>)> = Vec::new();
    for (media, _, armar, _, depois) in pre_condicoes() {
        if depois {
            continue;
        }
        for e in [Vec::new(), armar] {
            if !estados
                .iter()
                .any(|(m, a)| *m == media && format!("{a:?}") == format!("{e:?}"))
            {
                estados.push((media, e));
            }
        }
    }
    let falhas: Vec<Vec<String>> = em_paralelo(&estados, |(media, armar)| {
        let media = *media;
        let Ok(bancada) =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| Bancada::nova(media, armar)))
        else {
            return Vec::new();
        };
        let esmaecida = |id: NodeId| bancada.host.store().tooltip_for(id).is_some();
        let (base, _) = ensaio(media, armar, &Gesto::Nenhum);
        let mut out = Vec::new();
        let mut vistos: Vec<NodeId> = Vec::new();
        for g in gestos(media, armar) {
            let alvo = match &g {
                Gesto::Opcao(menu, _, k, _) if *k > 0 => *menu,
                Gesto::Opcao(..) => continue,
                _ => match g.id() {
                    Some(id) => id,
                    None => continue,
                },
            };
            if !dependentes.contains(&alvo) || vistos.contains(&alvo) {
                continue;
            }
            let (px, entrega) = ensaio(media, armar, &g);
            if !entrega.ajuste_mudou {
                continue;
            }
            vistos.push(alvo);
            let inerte = diferenca(&base, &px).0 == 0;
            if esmaecida(alvo) != inerte {
                out.push(format!(
                    "{media:?} [{}] {}: esmaecida={} mas a tinta {}",
                    armar
                        .iter()
                        .map(|a| format!("{a:?}"))
                        .collect::<Vec<_>>()
                        .join(" + "),
                    nome(&nomes, alvo),
                    esmaecida(alvo),
                    if inerte { "NÃO mudou" } else { "mudou" }
                ));
            }
        }
        out
    });
    let falhas: Vec<String> = falhas.into_iter().flatten().collect();
    assert!(
        falhas.is_empty(),
        "{} linha(s) dependente(s) mentem sobre agir:\n{}",
        falhas.len(),
        falhas.join("\n")
    );
}

/// **Cada controlo com o Shape em Bristles** (dono 13/10: *«veja com cuidado cada parâmetro da seção»*): para
/// cada meio, cada gesto que o painel oferece é medido com o Shape **None** e com o Shape **Bristles**; sai a
/// linha de quem age num e não no outro (no Digital todos os gestos; nos meios caros só a seção Shape, a pegada
/// e o Jitter). `CENSO_MEIO=<Meio>` restringe.
#[test]
#[ignore = "sonda: os controlos com o Shape em Bristles (plano 27)"]
fn sonda_cada_controlo_com_as_cerdas_no_shape() {
    use ph2d_tool_painter::ids::*;
    let nomes = nomes();
    let so: Option<String> = std::env::var("CENSO_MEIO").ok();
    let cerdas = opcao(
        PAINTER_SHAPE_KIND,
        painter_shape_kind_option_id(ph2d_tool_painter::TextureKind::Bristles.to_u8()),
    );
    for media in [
        PaintMedia::Digital,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ] {
        if so.as_ref().is_some_and(|m| *m != format!("{media:?}")) {
            continue;
        }
        let armar = vec![cerdas.clone()];
        let todos = media == PaintMedia::Digital;
        let deste: Vec<Gesto> = gestos(media, &armar)
            .into_iter()
            .filter(|g| {
                let id = g.id().unwrap_or(NodeId(0));
                let n = nome(&nomes, id);
                todos
                    || n.contains("SHAPE")
                    || n.contains("DAB")
                    || n.contains("JITTER")
                    || PAINTER_SHAPE_PARAMS.contains(&id)
            })
            .collect();
        let (base_c, _) = mede(media, &armar, &Gesto::Nenhum);
        let (base_n, _) = mede(media, &[], &Gesto::Nenhum);
        let medidas = em_paralelo(&deste, |g| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let (pc, ec) = mede(media, &armar, g);
                let (pn, _) = mede(media, &[], g);
                (
                    diferenca(&base_c, &pc).0,
                    ec.ajuste_mudou,
                    diferenca(&base_n, &pn).0,
                )
            }))
            .ok()
        });
        for (g, m) in deste.iter().zip(medidas) {
            let Some((nc, mudou, nn)) = m else {
                eprintln!("CERDAS\t{media:?}\tPÂNICO\t{g:?}");
                continue;
            };
            let n = nome(&nomes, g.id().unwrap_or(NodeId(0)));
            let eixo = match g {
                Gesto::Opcao(_, _, k, t) => format!("[{k}/{t}]"),
                _ => String::new(),
            };
            let veredito = if nc == 0 && mudou {
                "MORTO-COM-CERDAS"
            } else {
                ""
            };
            if todos && nc > 0 && veredito.is_empty() {
                continue;
            }
            eprintln!(
                "CERDAS\t{media:?}\t{n}{eixo}\tcerdas={nc}\tnone={nn}\tajuste={mudou}\t{veredito}"
            );
        }
    }
}
