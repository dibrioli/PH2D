//! ⭐⭐⭐ **O FLUXO DE UM QUADRO: marchar, e opcionalmente PINTAR** — o despacho que o
//! [`super::trace::Tracer`] serve.
//!
//! ⚠️ **Ele saiu do [`super::trace`] por um TECTO DE LOC** (`763` contra `700`, 2026-09-22) — e a
//! fronteira que o tecto forçou é a certa: *os TIPOS de um passe e o FLUXO dele são duas
//! responsabilidades*, e o `trace.rs` fica com os tipos e com o dono do dispositivo.

use super::*;

// O dispositivo, a fila, o cache, a fita, o pedido, a tela e o pintor — sete coisas
// independentes, e uma struct só as renomearia. E o corpo é longo porque são seis bindings,
// dois despachos e duas travessias do barramento.
#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
pub(super) fn marcha_com(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    cache: &mut crate::FieldPipelines,
    fita: &TapeWgsl,
    sculpts: &[ph2d_field_eval::device::DeviceSculpt],
    setup: MarchSetup,
    width: u32,
    height: u32,
    pintura: Pintura<'_>,
) -> Saida {
    let mut relogio_cpu = std::time::Instant::now();
    // ⭐ O pintor de MATERIAL, quando é ele — as leis do dono e a fita são só dele.
    let pintor = match &pintura {
        Pintura::Material(p) => Some(*p),
        Pintura::Nenhuma | Pintura::Matcap(_) => None,
    };
    let bgl = bgl_marcha(device);
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("campo"),
        bind_group_layouts: &[Some(&bgl)],
        immediate_size: 0,
    });

    // ⭐⭐⭐ **A ORDEM DO `k` É O CONTRATO**, e ela é uma só: a fita da peça, depois os cabeçalhos
    // das esculturas, depois a lei do dono. Cada emissor recebe a origem dele **desta** aritmética,
    // e é por isso que ela vive aqui e não em três sítios.
    let escultura = crate::sculpt::emit(sculpts, fita.consts.len());
    let esculturas = escultura.as_ref().map_or("", |e| e.source.as_str());
    let molde_com_esculturas = molde().replace("{ESCULTURAS}", esculturas);
    // ⭐ **As mesmas leis, para o pintor** (`docs/Render3d/08` §12) — ele marcha o ricochete, e
    // marchar é isto. ⚠️ Elas saem da MESMA substituição: uma segunda chamada ao
    // `crate::sculpt::emit` daria outra aritmética de origens para o mesmo `k`.
    let leis_com_esculturas = crate::trace_wgsl::leis().replace("{ESCULTURAS}", esculturas);
    // ⭐⭐⭐⭐ **O MATCAP MARCHA NUM KERNEL MAGRO** (`docs/Render3d/03` §W9, «o kernel que hospeda a
    // marcha»). Ele só lê a NORMAL, e o `centro_e_luz` traz o chão, as lâmpadas, a visibilidade e o
    // ricochete — código que o matcap nunca corre e que o compilador da placa paga em REGISTOS, logo
    // em raios a correr ao mesmo tempo. Medido a `1920×1080`: a mesma marcha num kernel que só marcha
    // custa `18×`–`31×` menos do que o quadro que a hospedava (o nó, `2,76` contra `86,34 ms`).
    let so_o_centro = matches!(pintura, Pintura::Matcap(_));
    // ⭐⭐⭐⭐ **A OCLUSÃO NO TEMPO** — ver [`crate::ceu_tempo`]. Sem chave (uma escultura) ou sem
    // oclusão, o quadro é o de sempre: o modo desce a `Nao` ANTES de o uniforme ser escrito.
    let mut setup = setup;
    let ceu_tempo =
        (!so_o_centro && setup.ao_rays > 0 && setup.ceu_tempo != crate::ceu_tempo::CeuTempo::Nao)
            .then(|| {
                let l = device.limits();
                // ⭐⭐⭐⭐ **A tabela é dimensionada pela ÁREA, não pelo traçado** (2026-10-01): com a
                // resolução dinâmica o traçado muda de tamanho de um quadro para o outro, o tamanho
                // da tabela está na chave, e cada mudança RECOMEÇAVA o céu do zero (`40`–`50 ms` em
                // quadros alternados, medido no laço do produto). A área é a da entrega.
                let tracado = u64::from(width) * u64::from(height);
                let area = match &pintura {
                    Pintura::Material(p) => p
                        .entrega
                        .map_or(tracado, |(w, h)| u64::from(w) * u64::from(h)),
                    _ => tracado,
                };
                crate::ceu_tempo::ChaveDoCeu::de(
                    fita,
                    sculpts,
                    &setup,
                    tracado.max(area),
                    l.max_storage_buffer_binding_size.min(l.max_buffer_size),
                )
            })
            .flatten();
    if ceu_tempo.is_none() {
        setup.ceu_tempo = crate::ceu_tempo::CeuTempo::Nao;
    }
    // ⚠️ Um quadro de movimento cuja PEÇA mudou (a mão a arrastar um parâmetro) não tem histórico a
    // herdar — a tabela vai recomeçar do zero, e acumular ali daria a cada pixel UMA fatia de cones.
    // Esse quadro faz a oclusão de sempre e GRAVA-a, e o seguinte (se a peça parar) já herda.
    if setup.ceu_tempo == crate::ceu_tempo::CeuTempo::Acumula
        && !ceu_tempo
            .as_ref()
            .is_some_and(|c| crate::ceu_tempo::herda(cache, c))
    {
        setup.ceu_tempo = crate::ceu_tempo::CeuTempo::Grava;
    }
    let luz_a_parte = !so_o_centro && crate::luz_separada();
    let p_centro = cache
        .entry_with_layout(
            device,
            &molde_com_esculturas,
            fita,
            if so_o_centro || luz_a_parte {
                "centro_so"
            } else {
                "centro_e_luz"
            },
            Some(&layout),
        )
        .clone();
    let p_luz = luz_a_parte.then(|| {
        cache
            .entry_with_layout(device, &molde_com_esculturas, fita, "luz_so", Some(&layout))
            .clone()
    });
    // ⭐⭐⭐⭐ **A oclusão a passo** reconstrói-se DEPOIS de a luz estar escrita em toda a imagem: o
    // `ceu_sobe` lê os representantes vizinhos.
    // ⚠️ Com a oclusão NO TEMPO a mexer, quem a escreve é o `ceu_tempo_acumula` — o passo não corre.
    let p_ceu = (!so_o_centro
        && setup.ao_rays > 0
        && setup.ceu_passo > 1
        && setup.ceu_tempo != crate::ceu_tempo::CeuTempo::Acumula)
        .then(|| {
            let mut e = |nome| {
                cache
                    .entry_with_layout(device, &molde_com_esculturas, fita, nome, Some(&layout))
                    .clone()
            };
            (e("ceu_meia"), e("ceu_sobe"))
        });
    // ⚠️ **Compilar é o caro** — o pipeline da borda só nasce quando ela vai de facto correr.
    let p_bordas = setup.antialias.then(|| {
        let mut e = |nome| {
            cache
                .entry_with_layout(device, &molde_com_esculturas, fita, nome, Some(&layout))
                .clone()
        };
        (e("bordas"), e("bordas_marcha"))
    });

    use wgpu::util::DeviceExt;
    // ⭐⭐⭐ **UM vector de constantes para os DOIS passes.** A fita da peça ocupa o princípio; a lei
    // do dono escreve a seguir, e a origem dela é **exactamente** `fita.consts.len()`.
    //
    // ⚠️⚠️ **É por isso que quem a EMITE é este sítio e não o chamador:** a origem que o texto
    // indexa e a ordem com que os vectores se concatenam são a MESMA decisão, e duas respostas
    // pintam cada folha com os números da vizinha **sem erro nenhum**.
    let mut consts = fita.consts.clone();
    if let Some(e) = &escultura {
        consts.extend_from_slice(&e.consts);
    }
    // ⭐⭐⭐⭐ **O CABEÇALHO DA GRADE DE LONGE** — ver [`crate::longe`]. Ele cai a seguir às
    // esculturas, e a grade dele mora no armazém a seguir às grades delas: as duas origens saem
    // DESTA aritmética. ⚠️ `longe_k` é o índice MAIS UM, porque `0` quer dizer *«sem grade»*.
    let regiao_das_esculturas = crate::sculpt::grid_len(sculpts).unwrap_or(0);
    let longe = setup
        .longe
        .filter(|_| regiao_das_esculturas <= u32::MAX as usize);
    let longe_k = match &longe {
        Some(l) => {
            let indice = consts.len();
            #[allow(clippy::cast_possible_truncation)]
            consts.extend_from_slice(&crate::longe::cabecalho(l, regiao_das_esculturas as u32));
            #[allow(clippy::cast_possible_truncation)]
            {
                indice as u32 + 1
            }
        }
        None => 0,
    };
    let folga = longe.as_ref().map_or(0, crate::longe::Longe::valores);
    let assa = longe.as_ref().and_then(crate::longe::Longe::grade);
    let ub = uniforme_do_pedido(device, setup, width, height, longe_k);
    // ⭐⭐⭐⭐ **O MESMO pedido com a precisão do MUNDO** — só a assadura das SONDAS o lê (ver
    // [`crate::paint::Alvos::setup_mundo`]). ⛔ Sem ele as sondas guardadas entre quadros levavam a
    // precisão do zoom em que foram assadas, e a mesma vista saía diferente conforme o caminho do zoom.
    let ub_mundo = matches!(pintura, Pintura::Material(_)).then(|| {
        let w = ph2d_field_render::Sharpness::do_mundo();
        uniforme_do_pedido(
            device,
            MarchSetup {
                hit_eps: w.hit,
                normal_eps: w.normal,
                ..setup
            },
            width,
            height,
            longe_k,
        )
    });
    let lei_do_dono = pintor.and_then(|p| p.owners?.to_wgsl(consts.len()));
    if let Some(l) = &lei_do_dono {
        consts.extend_from_slice(&l.consts);
    }
    if consts.is_empty() {
        consts.push(0.0);
    }
    let mut kb_bytes = Vec::with_capacity(consts.len() * 4);
    for c in &consts {
        kb_bytes.extend_from_slice(&c.to_le_bytes());
    }
    let kb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("k"),
        contents: &kb_bytes,
        usage: wgpu::BufferUsages::STORAGE,
    });
    let n = u64::from(width) * u64::from(height);
    let storage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC;
    let cria = |nome: &str, bytes: u64| {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(nome),
            size: bytes.max(16),
            usage: storage,
            mapped_at_creation: false,
        })
    };
    // ⭐⭐⭐ **AS GRADES SOBEM UMA VEZ** — ver [`crate::FieldPipelines::grades`]. Sem escultura é um
    // buffer mínimo, que o layout exige e o shader nunca lê.
    //
    // ⚠️ **Clonado e não emprestado:** um `wgpu::Buffer` é um punho com contagem, e segurar o
    // empréstimo do cache impediria a compilação do pipeline mais abaixo de lhe tocar.
    let b_grades = cache.grades(device, sculpts, folga).clone();
    let b_centro = cria("centro", n * 16);
    // ⭐ O passo é `1 + n_lamps + 6`: o céu, uma visibilidade por lâmpada e o RICOCHETE
    // (`docs/Render3d/08`) — mais **SEIS por lâmpada** quando há BORDA MOLE (o intermediário da
    // passagem horizontal e o resultado, `docs/Render3d/10` §25). ⚠️ Ele é a mesma conta do
    // `passo_da_luz()` do WGSL, e as duas têm de andar juntas: um buffer curto faz o shader
    // escrever fora e a `wgpu` recusa o despacho.
    let passo_luz = u64::from(setup.n_lamps) * (1 + 6 * u64::from(setup.mole.is_some())) + 1 + 6;
    // ⚠️ O `centro_so` não escreve a luz — o buffer fica no mínimo que o layout aceita.
    let b_luz = cria("luz", if so_o_centro { 16 } else { n * passo_luz * 4 });
    // ⛔⛔ **O TECTO da lista de bordas era `6 %` e ESTOUROU** — o gate da paridade apanhou-o: na
    // ROSCA a GPU devolveu exactamente `1 296` bordas, que **é** o tecto, contra `1 745` da CPU, e
    // a sobreposição das listas caiu para `72,6 %`.
    //
    // ⚠️ **O `0,5`–`1,2 %` que eu citei é da SILHUETA** (`docs/3DModeling/05`), e a borda deste
    // passe é silhueta **mais VINCO**: uma peça de ranhuras finas é quase toda vinco. Medido, a
    // rosca dá `8,4 %`. ⇒ `25 %`, que é três vezes o pior medido — e o custo é `20 B` por pixel
    // (`41 MB` a `1920×1080`), que a leitura já paga em `~2 ms`.
    //
    // ⚠️ *Um tecto derivado da grandeza ERRADA lê-se como generoso.*
    let max_bordas = (n / 4).max(1024);
    let b_borda = cria("bordas", max_bordas * 5 * 16);
    let b_conta = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("conta"),
        // ⚠️ Quatro palavras e não uma: a cópia de leitura alinha a `16 B`, e um buffer de `4`
        // seria lido fora dos limites.
        contents: &[0u8; 16],
        usage: storage,
    });

    let bind = |_p: &wgpu::ComputePipeline| {
        crate::trace_grupo::grupo_da_marcha(
            device, &bgl, &ub, &kb, &b_centro, &b_luz, &b_conta, &b_borda, &b_grades,
        )
    };
    let bg_centro = bind(&p_centro);
    let bg_bordas = p_bordas.as_ref().map(|(p, _)| bind(p));

    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    // ⭐⭐⭐⭐ **A GRADE ASSA-SE ANTES DA MARCHA, no mesmo encoder** — ver [`crate::longe`]. ⚠️ Dois
    // passes de computação no mesmo encoder correm em ordem, com a escrita do primeiro visível ao
    // segundo; é isso que dispensa um `submit` a mais por quadro.
    if let Some(g) = assa {
        let entradas = crate::longe::entradas();
        let bgl_assa = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("assa-longe"),
            entries: &entradas,
        });
        let layout_assa = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("assa-longe"),
            bind_group_layouts: &[Some(&bgl_assa)],
            immediate_size: 0,
        });
        let molde_assa = format!(
            "{}{leis_com_esculturas}{}",
            crate::longe::comum_para_assar(),
            crate::longe::ASSA
        );
        let p_assa = cache
            .entry_with_layout(device, &molde_assa, fita, "assa_longe", Some(&layout_assa))
            .clone();
        let bg_assa = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("assa-longe"),
            layout: &bgl_assa,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ub.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: kb.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: b_grades.as_entire_binding(),
                },
            ],
        });
        let mut crono = cache.cronometro.take();
        let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("assa-longe"),
            timestamp_writes: crono.as_mut().and_then(|c| c.marca("assa-longe")),
        });
        cache.cronometro = crono;
        cp.set_pipeline(&p_assa);
        cp.set_bind_group(0, &bg_assa, &[]);
        cp.dispatch_workgroups(
            g.dims[0].div_ceil(4),
            g.dims[1].div_ceil(4),
            g.dims[2].div_ceil(4),
        );
    }
    // ⚠️ **DOIS despachos, e a ordem é a lei**: a borda pergunta pelos VIZINHOS, logo o centro tem
    // de estar escrito para toda a imagem antes de ela correr.
    // ⭐ E a re-amostragem é um TERCEIRO, depois de a lista estar escrita — ver o `bordas_marcha`.
    // ⚠️ Cada despacho leva o SEU tamanho: o `ceu_meia` corre na grelha grossa dos representantes.
    let inteira = (width.div_ceil(8), height.div_ceil(8));
    let mut despachos: Vec<(
        &wgpu::ComputePipeline,
        &wgpu::BindGroup,
        (u32, u32),
        &'static str,
    )> = vec![(&p_centro, &bg_centro, inteira, "centro")];
    if let Some(p) = &p_luz {
        despachos.push((p, &bg_centro, inteira, "luz"));
    }
    if let Some((meia, sobe)) = &p_ceu {
        let passo = setup.ceu_passo.max(1);
        let grossa = (
            width.div_ceil(passo).div_ceil(8),
            height.div_ceil(passo).div_ceil(8),
        );
        despachos.push((meia, &bg_centro, grossa, "ceu-meia"));
        despachos.push((sobe, &bg_centro, inteira, "ceu-sobe"));
    }
    if let (Some((p, pm)), Some(bg)) = (p_bordas.as_ref(), bg_bordas.as_ref()) {
        despachos.push((p, bg, inteira, "bordas-lista"));
        despachos.push((pm, bg, inteira, "bordas-marcha"));
    }
    let mut crono = cache.cronometro.take();
    for (p, bg, (gx, gy), rotulo) in despachos {
        let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: crono.as_mut().and_then(|c| c.marca(rotulo)),
        });
        cp.set_pipeline(p);
        cp.set_bind_group(0, bg, &[]);
        cp.dispatch_workgroups(gx, gy, 1);
    }
    cache.cronometro = crono;
    // ⭐⭐⭐⭐ O histórico da oclusão — DEPOIS da luz, que é quem deixa o canal do céu de cada pixel.
    if let Some(chave) = ceu_tempo {
        crate::ceu_tempo::despacha(
            device,
            &mut enc,
            cache,
            &molde_com_esculturas,
            fita,
            &bgl,
            &bg_centro,
            chave,
            &setup,
            width,
            height,
        );
    }

    let ler = |enc: &mut wgpu::CommandEncoder, b: &wgpu::Buffer, bytes: u64| {
        let r = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("leitura"),
            size: bytes.max(16),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        enc.copy_buffer_to_buffer(b, 0, &r, 0, bytes.max(16));
        r
    };
    // ⭐⭐⭐ **QUANDO O PINTOR CORRE, O G-BUFFER NÃO ATRAVESSA O BARRAMENTO.** Ele fica no
    // dispositivo, que é onde o passe seguinte o lê — e o que volta é a IMAGEM.
    //
    // Medido a `1920×1080`: o centro e a luz são `49,8 MB` por quadro e a imagem são `8,3`.
    // ⚠️ **As DUAS leis de pintura deixam o G-buffer no dispositivo** — o que volta é a imagem.
    // *É essa a propriedade que o matcap partilha com o material, e a única que este ponto lê.*
    let pinta = !matches!(pintura, Pintura::Nenhuma);
    let (r_centro, r_luz) = if pinta {
        (None, None)
    } else {
        (
            Some(ler(&mut enc, &b_centro, n * 16)),
            Some(ler(&mut enc, &b_luz, n * passo_luz * 4)),
        )
    };
    let r_conta = ler(&mut enc, &b_conta, 16);
    // ⭐⭐⭐⭐ **O MATERIAL pinta no MESMO encoder** — ver [`crate::paint::pinta`]: um envio só.
    if let Pintura::Material(pintor) = pintura {
        if let Some(c) = cache.cronometro.as_mut() {
            c.cpu("cpu-marcha", relogio_cpu);
        }
        let alvos = crate::paint::Alvos {
            leis: &leis_com_esculturas,
            fita,
            bgl: &bgl,
            grades: &b_grades,
            setup: &ub,
            setup_mundo: ub_mundo.as_ref().unwrap_or(&ub),
            k: &kb,
            centro: &b_centro,
            luz: &b_luz,
            conta: &b_conta,
            borda: &b_borda,
        };
        let tecto = if setup.antialias { max_bordas } else { 0 };
        let rgba = crate::paint::pinta(
            device,
            queue,
            cache,
            pintor,
            lei_do_dono.as_ref(),
            &alvos,
            width,
            height,
            tecto,
            // ⭐⭐⭐⭐ **A chave das sondas** — ver [`crate::sondas_na_placa`].
            crate::sondas_na_placa::ChaveDasSondas::de(
                fita,
                sculpts,
                &setup,
                pintor,
                lei_do_dono.as_ref(),
            ),
            enc,
        );
        // A contagem volta DEPOIS da imagem, no mesmo envio — ela só diz quantas bordas houve.
        r_conta.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).ok();
        let d = r_conta.slice(..).get_mapped_range();
        let quantas = u64::from(u32::from_le_bytes([d[0], d[1], d[2], d[3]]));
        #[allow(clippy::cast_possible_truncation)]
        let edges = quantas.min(tecto) as usize;
        return Saida::Imagem(Pintado { edges, rgba });
    }
    if let Some(c) = cache.cronometro.as_mut() {
        c.resolve(&mut enc);
        relogio_cpu = c.cpu("cpu-marcha", relogio_cpu);
    }
    queue.submit([enc.finish()]);

    for b in r_centro.iter().chain(r_luz.iter()).chain([&r_conta]) {
        b.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    }
    device.poll(wgpu::PollType::wait_indefinitely()).ok();
    if let Some(c) = cache.cronometro.as_mut() {
        relogio_cpu = c.cpu("espera-marcha", relogio_cpu);
        c.colhe(device);
    }
    let _ = relogio_cpu;

    let d_conta = r_conta.slice(..).get_mapped_range();
    let quantas = u32::from_le_bytes([d_conta[0], d_conta[1], d_conta[2], d_conta[3]]) as u64;
    if pinta {
        // ⚠️ **A contagem de bordas tinha de voltar primeiro** — no MATCAP, que ainda pinta num
        // segundo envio: quantos workgroups o passe da borda precisa é um número que o dispositivo
        // escreveu.
        let usadas = if setup.antialias {
            quantas.min(max_bordas)
        } else {
            0
        };
        drop(d_conta);
        #[allow(clippy::cast_possible_truncation)]
        let edges = usadas as usize;
        let alvos = crate::paint::Alvos {
            leis: &leis_com_esculturas,
            fita,
            bgl: &bgl,
            grades: &b_grades,
            setup: &ub,
            setup_mundo: ub_mundo.as_ref().unwrap_or(&ub),
            k: &kb,
            centro: &b_centro,
            luz: &b_luz,
            conta: &b_conta,
            borda: &b_borda,
        };
        let rgba = match pintura {
            Pintura::Matcap(mc) => {
                crate::matcap::pinta(device, queue, cache, mc, &alvos, width, height, usadas)
            }
            Pintura::Material(_) | Pintura::Nenhuma => {
                unreachable!("o material pinta acima e o `pinta` exclui o nenhum")
            }
        };
        return Saida::Imagem(Pintado { edges, rgba });
    }
    let d_centro = r_centro.as_ref().expect("sem pintor o centro volta");
    let d_centro = d_centro.slice(..).get_mapped_range();
    let d_luz = r_luz.as_ref().expect("sem pintor a luz volta");
    let d_luz = d_luz.slice(..).get_mapped_range();

    // ⛔⛔ **A LISTA DE BORDAS LÊ-SE PELO QUE FOI ESCRITO, e não pelo tecto** — e é a diferença
    // entre `35 ms` e o que a máquina de facto faz. O tecto é `25 %` dos pixels (`41 MB` a
    // `1920×1080`) e a ocupação real é `1`–`8 %`: copiar o tecto inteiro a cada quadro era
    // **quase metade** dos `90 MB` de leitura. ⇒ um segundo `submit`, que custa um ida-e-volta e
    // poupa dezenas de megabytes. *Um buffer dimensionado para o pior caso não se lê no pior caso.*
    // ⚠️ **Sem anti-serrilhado não há segunda travessia nenhuma** — nem o `submit`, nem o
    // `poll`, que é um ida-e-volta completo ao dispositivo por quadro.
    let usadas = if setup.antialias {
        quantas.min(max_bordas)
    } else {
        0
    };
    let r_borda = {
        let mut enc2 =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let r = ler(&mut enc2, &b_borda, usadas * 5 * 16);
        if usadas > 0 {
            queue.submit([enc2.finish()]);
            r.slice(..).map_async(wgpu::MapMode::Read, |_| {});
            device.poll(wgpu::PollType::wait_indefinitely()).ok();
        }
        r
    };
    let d_borda = if usadas > 0 {
        Some(r_borda.slice(..).get_mapped_range())
    } else {
        None
    };

    let (t, normal, shadow, ambient, bounce, edges) =
        lida(&d_centro, &d_luz, passo_luz, usadas, d_borda.as_deref());

    drop(d_centro);
    drop(d_luz);
    drop(d_conta);
    drop(d_borda);

    Saida::Gbuffer(DeviceGbuffer {
        width,
        height,
        t,
        normal,
        shadow,
        // ⚠️ **Menos os TRÊS do ricochete** — o passo deixou de ser `1 + n_lamps`
        // (`docs/Render3d/08`), e sem este desconto os canais dele leriam-se como lâmpadas
        // fantasma. *O modo de falha foi o bom: um índice fora do `shadow`, alto e no primeiro
        // quadro.*
        #[allow(clippy::cast_possible_truncation)]
        lamps: (passo_luz - 1 - 6) as usize,
        ambient,
        bounce,
        ground: setup.ground,
        edges,
    })
}
