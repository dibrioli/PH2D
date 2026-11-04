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
fn a_linha_esmaecida_e_a_que_nao_age_no_digital() {
    a_linha_esmaecida_e_a_que_nao_age(PaintMedia::Digital);
}

/// O [`a_linha_esmaecida_e_a_que_nao_age_no_digital`] na Aquarela.
#[test]
fn a_linha_esmaecida_e_a_que_nao_age_na_aquarela() {
    a_linha_esmaecida_e_a_que_nao_age(PaintMedia::Watercolor);
}

/// O [`a_linha_esmaecida_e_a_que_nao_age_no_digital`] no Impasto.
#[test]
fn a_linha_esmaecida_e_a_que_nao_age_no_impasto() {
    a_linha_esmaecida_e_a_que_nao_age(PaintMedia::Impasto);
}

/// O [`a_linha_esmaecida_e_a_que_nao_age_no_digital`] no Wet Paint.
#[test]
fn a_linha_esmaecida_e_a_que_nao_age_no_wet_paint() {
    a_linha_esmaecida_e_a_que_nao_age(PaintMedia::WetPaint);
}

/// Os meios dos irmãos acima — e o controlo: a tabela das pré-condições não tem estado de OUTRO meio (um meio
/// novo na tabela, sem irmão, ficaria fora do gate em silêncio).
const MEIOS_DO_ESMAECIDO: [PaintMedia; 4] = [
    PaintMedia::Digital,
    PaintMedia::Watercolor,
    PaintMedia::Impasto,
    PaintMedia::WetPaint,
];

/// O corpo dos quatro irmãos (partido por meio em 02/11: os 77 estados num só teste passavam o prazo sozinhos).
fn a_linha_esmaecida_e_a_que_nao_age(so: PaintMedia) {
    let nomes = nomes();
    let dependentes = ids_dependentes();
    let mut estados: Vec<(PaintMedia, Vec<Gesto>)> = Vec::new();
    for (media, _, armar, _, depois) in pre_condicoes() {
        assert!(
            MEIOS_DO_ESMAECIDO.contains(&media),
            "{media:?}: um meio das pré-condições sem irmão no gate do esmaecido"
        );
        if depois || media != so {
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
    // ⚠️ O trabalho é ACHATADO em (estado, gesto) numa só fila da largura da máquina (o laço em série por estado
    // passou o prazo de 600 s sozinho, 02/11: 77 estados, o painel do E10 a ~0,25 s por pintura). A resposta é a do
    // laço em série: por estado, cada alvo é o 1.º dos seus gestos (na ordem da tela) cujo ajuste mudou — e por
    // isso vai por VAGAS: a vaga `n` mede o `n`-ésimo gesto de cada alvo ainda sem ajuste mudado; nenhum ensaio a
    // mais que o laço em série. As falhas saem na ordem do laço (estado, posição do gesto).
    struct Estado {
        base: Vec<u8>,
        esmaecidos: Vec<NodeId>,
        /// Por alvo (na ordem da 1.ª aparição): os gestos dele, `(posição na tela, gesto)`.
        alvos: Vec<(NodeId, Vec<(usize, Gesto)>)>,
    }
    let preparados: Vec<Option<Estado>> = em_paralelo(&estados, |(media, armar)| {
        let media = *media;
        let bancada =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| Bancada::nova(media, armar)))
                .ok()?;
        let esmaecida = |id: NodeId| {
            bancada
                .host
                .store()
                .tooltip_for(id)
                .is_some_and(ph2d_panel_painter_layers::e_dica_de_inercia)
        };
        let (base, _) = ensaio(media, armar, &Gesto::Nenhum);
        let mut alvos: Vec<(NodeId, Vec<(usize, Gesto)>)> = Vec::new();
        for (k, g) in gestos(media, armar).into_iter().enumerate() {
            let alvo = match &g {
                Gesto::Opcao(menu, _, k, _) if *k > 0 => *menu,
                Gesto::Opcao(..) => continue,
                _ => match g.id() {
                    Some(id) => id,
                    None => continue,
                },
            };
            if !dependentes.contains(&alvo) {
                continue;
            }
            match alvos.iter_mut().find(|(a, _)| *a == alvo) {
                Some((_, v)) => v.push((k, g)),
                None => alvos.push((alvo, vec![(k, g)])),
            }
        }
        let esmaecidos = alvos
            .iter()
            .map(|(a, _)| *a)
            .filter(|a| esmaecida(*a))
            .collect();
        Some(Estado {
            base,
            esmaecidos,
            alvos,
        })
    });
    // (estado, alvo) → `Some((posição, inerte))` do gesto escolhido, ou `None` se nenhum mudou o ajuste.
    let mut escolhido: BTreeMap<(usize, usize), Option<(usize, bool)>> = BTreeMap::new();
    let mut vaga = 0;
    loop {
        let fila: Vec<(usize, usize, usize, &Gesto)> = preparados
            .iter()
            .enumerate()
            .filter_map(|(e, p)| p.as_ref().map(|p| (e, p)))
            .flat_map(|(e, p)| {
                p.alvos
                    .iter()
                    .enumerate()
                    .filter_map(move |(a, (_, gs))| gs.get(vaga).map(|(k, g)| (e, a, *k, g)))
            })
            .filter(|(e, a, ..)| escolhido.get(&(*e, *a)).is_none_or(Option::is_none))
            .collect();
        if fila.is_empty() {
            break;
        }
        let medidas = em_paralelo(&fila, |(e, _, _, g)| {
            let (media, armar) = &estados[*e];
            let (px, entrega) = ensaio(*media, armar, g);
            let base = &preparados[*e].as_ref().expect("preparado").base;
            (entrega.ajuste_mudou, diferenca(base, &px).0 == 0)
        });
        for ((e, a, k, _), (mudou, inerte)) in fila.into_iter().zip(medidas) {
            let s = escolhido.entry((e, a)).or_insert(None);
            if mudou {
                *s = Some((k, inerte));
            }
        }
        vaga += 1;
    }
    let mut falhas: Vec<String> = Vec::new();
    for (e, p) in preparados.iter().enumerate() {
        let Some(p) = p else { continue };
        let (media, armar) = &estados[e];
        let mut deste: Vec<(usize, NodeId, bool)> = p
            .alvos
            .iter()
            .enumerate()
            .filter_map(|(a, (alvo, _))| {
                escolhido
                    .get(&(e, a))
                    .copied()
                    .flatten()
                    .map(|(k, inerte)| (k, *alvo, inerte))
            })
            .collect();
        deste.sort_by_key(|(k, ..)| *k);
        for (_, alvo, inerte) in deste {
            let esmaecida = p.esmaecidos.contains(&alvo);
            if esmaecida != inerte {
                falhas.push(format!(
                    "{media:?} [{}] {}: esmaecida={} mas a tinta {}",
                    armar
                        .iter()
                        .map(|a| format!("{a:?}"))
                        .collect::<Vec<_>>()
                        .join(" + "),
                    nome(&nomes, alvo),
                    esmaecida,
                    if inerte { "NÃO mudou" } else { "mudou" }
                ));
            }
        }
    }
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
