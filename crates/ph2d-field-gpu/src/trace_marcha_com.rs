//! ⭐⭐⭐ **O FLUXO DE UM QUADRO: marchar, e opcionalmente PINTAR o Matcap** — o despacho que o
//! [`super::trace::Tracer`] serve.
//!
//! ⚠️ **Ele saiu do [`super::trace`] por um TECTO DE LOC** (`763` contra `700`, 2026-09-22) — e a
//! fronteira que o tecto forçou é a certa: *os TIPOS de um passe e o FLUXO dele são duas
//! responsabilidades*, e o `trace.rs` fica com os tipos e com o dono do dispositivo.

use super::*;

// O dispositivo, a fila, o cache, a fita, o pedido, a tela e a pintura — sete coisas
// independentes, e uma struct só as renomearia. E o corpo é longo porque são seis bindings,
// três despachos e duas travessias do barramento.
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
    let bgl = bgl_marcha(device);
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("campo"),
        bind_group_layouts: &[Some(&bgl)],
        immediate_size: 0,
    });

    // ⭐⭐⭐ **A ORDEM DO `k` É O CONTRATO**, e ela é uma só: a fita da peça, depois os cabeçalhos
    // das esculturas, depois o da grade de longe. Cada emissor recebe a origem dele **desta**
    // aritmética, e é por isso que ela vive aqui e não em vários sítios.
    let escultura = crate::sculpt::emit(sculpts, fita.consts.len());
    let esculturas = escultura.as_ref().map_or("", |e| e.source.as_str());
    let molde_com_esculturas = molde().replace("{ESCULTURAS}", esculturas);
    // ⚠️ As leis saem da MESMA substituição para a assadura da grade de longe: uma segunda chamada
    // ao `crate::sculpt::emit` daria outra aritmética de origens para o mesmo `k`.
    let leis_com_esculturas = crate::trace_wgsl::leis().replace("{ESCULTURAS}", esculturas);
    use wgpu::util::DeviceExt;
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
    // ⭐⭐⭐⭐ **Os kernels deste quadro compilam-se JUNTOS** — ver
    // [`crate::FieldPipelines::precompila`]. A lista é a dos pedidos que se seguem, com as MESMAS
    // condições; um pedido que ela esqueça compila-se sozinho mais abaixo, que é o caminho de antes.
    {
        let mut lista = vec!["centro_so"];
        if setup.antialias {
            lista.extend(["bordas", "bordas_marcha"]);
        }
        let pedidos: Vec<crate::PedidoDeLote<'_>> = lista
            .iter()
            .map(|e| (molde_com_esculturas.as_str(), fita, *e, Some(&layout)))
            .collect();
        cache.precompila_lote(device, &pedidos);
    }
    let p_centro = cache
        .entry_with_layout(
            device,
            &molde_com_esculturas,
            fita,
            "centro_so",
            Some(&layout),
        )
        .clone();
    // ⚠️ **Compilar é o caro** — o pipeline da borda só nasce quando ela vai de facto correr.
    let p_bordas = setup.antialias.then(|| {
        let mut e = |nome| {
            cache
                .entry_with_layout(device, &molde_com_esculturas, fita, nome, Some(&layout))
                .clone()
        };
        (e("bordas"), e("bordas_marcha"))
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
            device, &bgl, &ub, &kb, &b_centro, &b_conta, &b_borda, &b_grades,
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
    let inteira = (width.div_ceil(8), height.div_ceil(8));
    let mut despachos: Vec<(
        &wgpu::ComputePipeline,
        &wgpu::BindGroup,
        (u32, u32),
        &'static str,
    )> = vec![(&p_centro, &bg_centro, inteira, "centro")];
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
    // ⭐⭐⭐ **QUANDO O MATCAP PINTA, O G-BUFFER NÃO ATRAVESSA O BARRAMENTO.** Ele fica no
    // dispositivo, que é onde o passe seguinte o lê — e o que volta é a IMAGEM.
    let pinta = !matches!(pintura, Pintura::Nenhuma);
    let r_centro = (!pinta).then(|| ler(&mut enc, &b_centro, n * 16));
    let r_conta = ler(&mut enc, &b_conta, 16);
    if let Some(c) = cache.cronometro.as_mut() {
        c.resolve(&mut enc);
        relogio_cpu = c.cpu("cpu-marcha", relogio_cpu);
    }
    queue.submit([enc.finish()]);

    for b in r_centro.iter().chain([&r_conta]) {
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
    if let Pintura::Matcap(mc) = pintura {
        // ⚠️ **A contagem de bordas tinha de voltar primeiro** — o Matcap pinta num segundo envio:
        // quantos workgroups o passe da borda precisa é um número que o dispositivo escreveu.
        let usadas = if setup.antialias {
            quantas.min(max_bordas)
        } else {
            0
        };
        drop(d_conta);
        #[allow(clippy::cast_possible_truncation)]
        let edges = usadas as usize;
        let alvos = Alvos {
            fita,
            bgl: &bgl,
            grades: &b_grades,
            setup: &ub,
            k: &kb,
            centro: &b_centro,
            conta: &b_conta,
            borda: &b_borda,
        };
        let rgba = crate::matcap::pinta(device, queue, cache, mc, &alvos, width, height, usadas);
        return Saida::Imagem(Pintado {
            edges,
            rgba,
            compilado_ms: 0.0,
        });
    }
    let d_centro = r_centro.as_ref().expect("sem pintura o centro volta");
    let d_centro = d_centro.slice(..).get_mapped_range();

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

    let (t, normal, edges) = lida(&d_centro, usadas, d_borda.as_deref());

    drop(d_centro);
    drop(d_conta);
    drop(d_borda);

    Saida::Gbuffer(DeviceGbuffer {
        width,
        height,
        t,
        normal,
        edges,
    })
}
