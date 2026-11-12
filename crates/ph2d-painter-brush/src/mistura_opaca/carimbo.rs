//! ⭐ **O carimbo do Wet blend** — a lei de [`super`] num carimbo: a amostra, o reservatório, a tinta e a película.

use super::*;
use rayon::prelude::*;

/// Uma faixa de `ALTO` linhas da tela: o índice, os bytes dela e os ladrilhos da sombra que a cobrem.
type Faixa<'a> = (u32, &'a mut [u8], &'a mut [Option<Box<[f32]>>]);

/// Um carimbo do Wet blend. `antes(i)` = o RGBA do byte `i` (múltiplo de 4) da tela de ANTES do traço, sobre a qual
/// a película se compõe; `amostra_de(i)` = o fundo de onde se amostra (a mesma tela, ou todas as camadas visíveis —
/// [`AmostraDaMistura`]). Chamados só dentro da caixa da pegada. Devolve `None` com a lei `Off`, raio nulo ou fora da tela —
/// e nesses casos a tela e o reservatório ficam intactos.
pub fn mistura_opaca_carimbo(
    tela: &mut Tela<'_>,
    antes: impl Fn(usize) -> [u8; 4] + Sync,
    amostra_de: impl Fn(usize) -> [u8; 4] + Sync,
    spec: &BrushSpec,
    c: Carimbo,
    pegada: Pegada<'_>,
    res: &mut ReservatorioOpaco,
) -> Option<DirtyRect> {
    let m = spec.mistura_opaca;
    if !m.ligada() || c.radius_px <= 0.0 || c.coverage <= 0.0 {
        return None;
    }
    let (w, h) = (tela.width, tela.height);
    let (x0, y0, x1, y1) = match pegada {
        Pegada::Mascara { caixa, .. } => caixa,
        Pegada::Disco { centro, raio } => caixa_do_dab(w, h, centro, raio)?,
    };
    if x1 <= x0 || y1 <= y0 || x1 > w || y1 > h {
        return None;
    }
    // O texel `(x, y)` da tela na máscara da pegada.
    let na_mascara = |x: u32, y: u32| -> usize {
        match pegada {
            Pegada::Mascara { org, largura, .. } => {
                (((y - org[1]) * largura + (x - org[0])) * 4) as usize
            }
            Pegada::Disco { .. } => 0,
        }
    };
    let peso = |x: u32, y: u32| -> f32 {
        match pegada {
            Pegada::Mascara { mascara, .. } => f32::from(mascara[na_mascara(x, y) + 3]) / 255.0,
            Pegada::Disco { centro, raio } => {
                let dx = x as f32 + 0.5 - centro[0];
                let dy = y as f32 + 0.5 - centro[1];
                let t = (dx * dx + dy * dy).sqrt() / raio.max(1e-6);
                if t >= 1.0 {
                    0.0
                } else {
                    spec.falloff_weight(t)
                }
            }
        }
    };
    let dist = if res.cheio {
        let (dx, dy) = (c.center[0] - res.ultimo[0], c.center[1] - res.ultimo[1]);
        (dx * dx + dy * dy).sqrt()
    } else {
        0.0
    };
    let arco = res.arco + dist;
    let velha_desde = arco - DESFASAGEM * c.radius_px;
    // A película desta PASSADA (a metade de trás foi coberta há no máximo dois raios) tinge a amostra mas não
    // RECARREGA o pincel: a tinta que ele acabou de pôr saiu dele (o dedo esgota-se no vazio, dono 10/11). A volta de
    // um zigue-zague, mais velha, recarrega.
    let carrega_desde = arco - (DESFASAGEM + 2.0) * c.radius_px;
    let sombra_valida = tela.sombra.mesma_tela(w, h);
    let dir = if dist > 1e-4 {
        Some([
            (c.center[0] - res.ultimo[0]) / dist,
            (c.center[1] - res.ultimo[1]) / dist,
        ])
    } else {
        res.dir
    };
    // Quanto o texel ficou ATRÁS do centro, ao longo do movimento (`≤ 0` = à frente ou ao lado).
    let atras = |x: u32, y: u32| -> f32 {
        dir.map_or(0.0, |d| {
            -((x as f32 + 0.5 - c.center[0]) * d[0] + (y as f32 + 0.5 - c.center[1]) * d[1])
        })
    };
    // 1. A amostra: a média sob a pegada da tela de antes com a película VELHA deste traço por cima, pesada pelo
    // alfa; a carga = quanto ali é tinta. Em movimento, só a metade de TRÁS: o pincel leva o que já atravessou.
    // As FAIXAS de `ALTO` linhas (as linhas de ladrilhos da película): a amostra soma-se por faixa e as faixas em
    // ordem (o mesmo byte com qualquer número de núcleos); a película pinta-se por faixa (cada texel lê e escreve só o
    // seu). Um carimbo pequeno corre as mesmas faixas num núcleo.
    let (ty0, ty1) = (y0 / ALTO, (y1 - 1) / ALTO);
    let paralelo = ((x1 - x0) * (y1 - y0)) as usize >= crate::PARALLEL_MIN_AREA;
    let sombra_le: &Sombra = tela.sombra;
    // Marca larga: a amostra só nos texels pares da TELA (a grelha não depende da caixa).
    let salto = if c.radius_px >= RAIO_DA_AMOSTRA_ESPARSA
        && AMOSTRA_ESPARSA.load(std::sync::atomic::Ordering::Relaxed)
    {
        CARIMBOS_ESPARSOS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        2
    } else {
        1
    };
    let par = |v: u32| v.next_multiple_of(salto);
    let soma_da_faixa = |ty: u32| -> [f32; 6] {
        // `[r, g, b]` pesados pelo alfa · o alfa · a carga · o peso.
        let mut s = [0.0f32; 6];
        for y in (par((ty * ALTO).max(y0))..((ty + 1) * ALTO).min(y1)).step_by(salto as usize) {
            for x in (par(x0)..x1).step_by(salto as usize) {
                let wt = peso(x, y);
                if wt <= 0.0 || atras(x, y) < 0.0 {
                    continue;
                }
                let i = ((y * w + x) * 4) as usize;
                let fundo = amostra_de(i).map(|v| f32::from(v) / 255.0);
                let (px, carga_px) = match sombra_valida.then(|| sombra_le.le(x, y)).flatten() {
                    Some(f) if f[4] <= velha_desde && f[3] > 0.0 => {
                        let px = sobre([f[0], f[1], f[2], f[3]], fundo);
                        (
                            px,
                            if f[4] <= carrega_desde {
                                px[3]
                            } else {
                                fundo[3]
                            },
                        )
                    }
                    _ => (fundo, fundo[3]),
                };
                let t = wt * px[3];
                for k in 0..3 {
                    s[k] += t * px[k];
                }
                s[3] += t;
                s[4] += wt * carga_px;
                s[5] += wt;
            }
        }
        s
    };
    let parciais: Vec<[f32; 6]> = if paralelo {
        (ty0..=ty1).into_par_iter().map(soma_da_faixa).collect()
    } else {
        (ty0..=ty1).map(soma_da_faixa).collect()
    };
    let mut total = [0.0f32; 6];
    for p in &parciais {
        for (t, v) in total.iter_mut().zip(p) {
            *t += v;
        }
    }
    let (soma, soma_a, soma_carga, soma_w) =
        ([total[0], total[1], total[2]], total[3], total[4], total[5]);
    if soma_w <= 0.0 {
        return None;
    }
    let pincel = c.cor.map(|v| v.clamp(0.0, 1.0));
    let carga_aqui = soma_carga / soma_w;
    let amostra = if soma_a > 0.0 {
        soma.map(|v| v / soma_a)
    } else {
        pincel
    };
    // 2. O reservatório, por distância (em diâmetros).
    let primeiro = !res.cheio;
    let passo = if res.cheio {
        (dist / (2.0 * c.radius_px)).min(1.0)
    } else {
        1.0
    };
    let retem = if res.cheio {
        potencia(m.retem_por_diametro(), passo)
    } else {
        0.0
    };
    let velha = retem * res.carga;
    let nova = (1.0 - retem) * carga_aqui;
    let carga = velha + nova;
    if carga > 0.0 {
        let f = nova / carga;
        let peso = if res.cheio && res.carga > 0.0 { f } else { 1.0 };
        for (cor, a) in res.cor.iter_mut().zip(amostra.iter()) {
            *cor += (a - *cor) * peso;
        }
    }
    res.carga = carga;
    res.cheio = true;
    res.ultimo = c.center;
    res.arco = arco;
    res.dir = dir;
    // 3. A tinta do carimbo: o papel transparente não suja (o pincel fica com a cor dele onde não apanhou nada).
    let amount = MisturaOpaca::pela_pressao(m.paint_amount, m.amount_pressure, c.pressao);
    let a_eff = peso_do_pincel(amount, res.carga);
    // O «Pigment» é UM controlo do pincel (`BrushSpec::effective_pigment_mix`, cartão Paint Mixing, dono 10/11): o mesmo
    // número mistura o pincel com o que ele apanhou e a tinta com a tela.
    let pig = spec.effective_pigment_mix();
    let tinta_de = |res_cor: [f32; 3], a_eff: f32, pincel: [f32; 3]| -> [f32; 3] {
        let rgb: [f32; 3] = std::array::from_fn(|k| res_cor[k] + (pincel[k] - res_cor[k]) * a_eff);
        if pig > 0.0 {
            let e = crate::espectral::mistura(res_cor, pincel, a_eff);
            std::array::from_fn(|k| rgb[k] + (e[k] - rgb[k]) * pig)
        } else {
            rgb
        }
    };
    let agora = Marco {
        arco,
        cor: res.cor,
        a_eff,
        tinta: tinta_de(res.cor, a_eff, pincel),
    };
    // O texel atrás do centro recebe o estado de quando o centro passou por ele: uma tabela por fração do raio.
    let tabela: [Marco; MARCOS] = std::array::from_fn(|k| {
        res.historico
            .em(arco - c.radius_px * k as f32 / (MARCOS - 1) as f32, agora)
    });
    if res
        .historico
        .novo()
        .is_none_or(|m| arco - m.arco >= c.radius_px / MARCOS_POR_RAIO)
    {
        res.historico.guarda(agora);
    }
    let estado_em = |x: u32, y: u32| -> Marco {
        let b = atras(x, y);
        if b <= 0.0 {
            return agora;
        }
        let f = (b / c.radius_px.max(1e-6)).min(1.0) * (MARCOS - 1) as f32;
        let k = (f as usize).min(MARCOS - 2);
        Marco::lerp(tabela[k], tabela[k + 1], f - k as f32)
    };
    let cor_da_mascara = match pegada {
        Pegada::Mascara {
            mascara,
            com_cor: true,
            ..
        } => Some(mascara),
        _ => None,
    };
    // 4. A película do traço: cada carimbo põe `tinta` nela à taxa da pegada × Density × Flow (normalizada pelo
    // passo do pincel), e a cobertura dela sobe só até ao TECTO do pincel sem a mistura (Accumulate desligado: a
    // coverage do dab × o grão; ligado: 1). A tela é `antes ⊕ película`: a de baixo só aparece
    // por `1 − cobertura`, nunca como degrau dentro da cor (a costura da foto do dono, 08/11).
    // O pincel só cobre o que não apanhou na medida em que TEM tinta: Amount 0 (um esbatedor, o Gouache blender)
    // não pinta o vazio; a partir de [`AMOUNT_QUE_COBRE`] cobre inteiro (o vazio não lava a cor).
    let corpo = res.carga + (1.0 - res.carga) * (amount / AMOUNT_QUE_COBRE).min(1.0);
    let densidade = m.densidade(c.pressao);
    let cobertura = c.coverage.clamp(0.0, 1.0);
    // O CSP medido (08/11, `wetblend_vs_csp_medidas.txt`): a Density é a TAXA por carimbo (Density 45–50 ainda
    // cobre 88–98 % numa passada) e o tecto é a opacidade do pincel; como tecto ela dava um véu chapado.
    // O `corpo` é a opacidade que a tinta ATINGE (a película tende a ele), não uma taxa: com a taxa, o dedo de carga
    // pequena empilhava carimbos até ao opaco e nunca esmaecia (dono 10/11). Com `corpo = 1` é a lei de antes.
    let taxa = densidade * spec.flow.clamp(0.0, 1.0);
    let tecto = if spec.accumulate { 1.0 } else { cobertura };
    let taxa = if spec.accumulate {
        taxa * cobertura
    } else {
        taxa
    };
    if tecto <= 0.0 || taxa <= 0.0 {
        return None;
    }
    // A opacidade acumula como o pincel SEM a mistura: cada carimbo vale `passo / passo do pincel` de um dab dele
    // (a mistura adensa os carimbos a 3 %, não a cobertura); o 1.º vale um passo da mistura (sem disco no início).
    let nominal = spec.spacing.max(PASSO_DA_MISTURA);
    // O pincel que sorteia a pegada (Pointillism, Pastel, Chalk) carimba também a 3 % e enche: o look aprovado pelo
    // dono (08/11, revoga o Bug #78).
    let k_passo = if primeiro {
        spec.spacing_do_traco() / nominal
    } else {
        passo / nominal
    };
    let k_passo = k_passo.clamp(PASSO_DA_MISTURA * 0.25, 1.0);
    // No plano do traço a tela É a película (relê-se dela); na tela directa a película nasce vazia no pen-down.
    tela.sombra
        .garante(tela.buf, w, h, (x0, y0, x1, y1), tela.plano);
    let alfa_de = |wt: f32| 1.0 - potencia(1.0 - (wt * taxa).min(1.0), k_passo);
    // A pegada da máscara é um BYTE: a taxa de cada texel é uma de 256 por carimbo (a potência por texel era o maior
    // custo do carimbo de véu).
    let tabela_alfa: Option<[f32; 256]> = matches!(pegada, Pegada::Mascara { .. })
        .then(|| std::array::from_fn(|b| alfa_de(f32::from(b as u8) / 255.0)));
    let (plano, trava_alfa) = (tela.plano, tela.trava_alfa);
    // O TEXEL da película em duas metades — `quanto` (o alfa e o tecto, só lê) e `aplica` (escreve a película e a
    // tela) — para as duas partições pintarem a MESMA conta: as faixas de ladrilhos e as linhas (missão 4K onda 3).
    let quanto = |x: u32, y: u32| -> Option<(f32, f32)> {
        let (wt, byte) = match pegada {
            Pegada::Mascara { mascara, .. } => {
                let b = mascara[na_mascara(x, y) + 3];
                (f32::from(b) / 255.0, Some(b))
            }
            Pegada::Disco { .. } => (peso(x, y), None),
        };
        if wt <= 0.0 {
            return None;
        }
        // O grão: a razão entre a pegada e a pegada sem ele vai ao tecto; a pegada sem ele é a taxa.
        let (wt, grao, byte) = match pegada {
            Pegada::Mascara {
                sem_grao: Some(sg), ..
            } => {
                let b = sg[na_mascara(x, y) + 3];
                let liso = f32::from(b) / 255.0;
                if liso > 0.0 {
                    (liso, (wt / liso).min(1.0), Some(b))
                } else {
                    (wt, 1.0, byte)
                }
            }
            _ => (wt, 1.0, byte),
        };
        let tecto = tecto * grao;
        let alfa = match (&tabela_alfa, byte) {
            (Some(t), Some(b)) => t[usize::from(b)],
            _ => alfa_de(wt),
        };
        (alfa > 0.0).then_some((alfa, tecto))
    };
    let aplica = |x: u32, y: u32, (alfa, tecto): (f32, f32), px: &mut [f32], saida: &mut [u8]| {
        let i = ((y * w + x) * 4) as usize;
        if px[4] == SEM_TINTA {
            px[4] = arco;
        }
        let estado = estado_em(x, y);
        let tinta = match cor_da_mascara {
            Some(mc) => tinta_de(estado.cor, estado.a_eff, {
                let mi = na_mascara(x, y);
                [mc[mi], mc[mi + 1], mc[mi + 2]].map(|v| f32::from(v) / 255.0)
            }),
            None => estado.tinta,
        };
        let s0 = px[3];
        let s_livre = s0 * (1.0 - alfa) + corpo * alfa;
        if s_livre > 0.0 {
            for k in 0..3 {
                px[k] = (px[k] * s0 * (1.0 - alfa) + tinta[k] * corpo * alfa) / s_livre;
            }
        }
        px[3] = s_livre.max(s0).min(tecto.max(s0));
        let pel = [px[0], px[1], px[2], px[3]];
        let out = if plano {
            pel
        } else {
            let ant = antes(i).map(|v| f32::from(v) / 255.0);
            let sc = pel[3];
            if trava_alfa {
                let sc = if ant[3] > 0.0 { sc } else { 0.0 };
                [
                    ant[0] + (pel[0] - ant[0]) * sc,
                    ant[1] + (pel[1] - ant[1]) * sc,
                    ant[2] + (pel[2] - ant[2]) * sc,
                    ant[3],
                ]
            } else {
                let a = sc + ant[3] * (1.0 - sc);
                if a > 0.0 {
                    let f = ant[3] * (1.0 - sc);
                    [
                        (pel[0] * sc + ant[0] * f) / a,
                        (pel[1] * sc + ant[1] * f) / a,
                        (pel[2] * sc + ant[2] * f) / a,
                        a,
                    ]
                } else {
                    [0.0; 4]
                }
            }
        };
        for (dst, v) in saida.iter_mut().zip(out.iter()) {
            *dst = (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        }
    };
    let colunas = tela.sombra.colunas as usize;
    if PELICULA_POR_LINHAS.load(std::sync::atomic::Ordering::Relaxed) {
        // Uma tarefa por LINHA: as linhas da tela e, de cada ladrilho da caixa, a linha dele.
        let (tx0, tx1) = (x0 / LADO, (x1 - 1) / LADO);
        let nt = (tx1 - tx0 + 1) as usize;
        let mut filas: Vec<Option<&mut [f32]>> =
            (0..(y1 - y0) as usize * nt).map(|_| None).collect();
        for (ty, fila) in tela
            .sombra
            .ladrilhos
            .chunks_mut(colunas)
            .enumerate()
            .skip(ty0 as usize)
            .take((ty1 - ty0 + 1) as usize)
        {
            for (k, l) in fila[tx0 as usize..=tx1 as usize].iter_mut().enumerate() {
                let tx = tx0 + k as u32;
                let lw = LADO.min(w - tx * LADO) as usize;
                let l = l
                    .as_deref_mut()
                    .expect("ladrilho da sombra garantido antes do toque");
                for (r, linha) in l.chunks_mut(lw * POR_TEXEL).enumerate() {
                    let y = ty as u32 * ALTO + r as u32;
                    if (y0..y1).contains(&y) {
                        filas[(y - y0) as usize * nt + k] = Some(linha);
                    }
                }
            }
        }
        let stride = (w * 4) as usize;
        let pinta_linha = |(r, (linha, lads)): (usize, (&mut [u8], &mut [Option<&mut [f32]>]))| {
            let y = y0 + r as u32;
            for x in x0..x1 {
                let Some(q) = quanto(x, y) else { continue };
                let tx = x / LADO;
                let l = lads[(tx - tx0) as usize]
                    .as_deref_mut()
                    .expect("a linha do ladrilho");
                let j = (x - tx * LADO) as usize * POR_TEXEL;
                let xi = (x * 4) as usize;
                aplica(x, y, q, &mut l[j..j + POR_TEXEL], &mut linha[xi..xi + 4]);
            }
        };
        let linhas = &mut tela.buf[y0 as usize * stride..y1 as usize * stride];
        if paralelo {
            linhas
                .par_chunks_mut(stride)
                .zip(filas.par_chunks_mut(nt))
                .enumerate()
                .for_each(pinta_linha);
        } else {
            linhas
                .chunks_mut(stride)
                .zip(filas.chunks_mut(nt))
                .enumerate()
                .for_each(pinta_linha);
        }
    } else {
        let pinta_faixa = |(ty, linhas, ladrilhos): Faixa<'_>| {
            let base = ty * ALTO;
            for y in base.max(y0)..(base + ALTO).min(y1) {
                for x in x0..x1 {
                    let Some(q) = quanto(x, y) else { continue };
                    let tx = x / LADO;
                    let lw = LADO.min(w - tx * LADO);
                    let ladrilho = ladrilhos[tx as usize]
                        .as_deref_mut()
                        .expect("ladrilho da sombra garantido antes do toque");
                    let j = ((y - base) * lw + (x - tx * LADO)) as usize * POR_TEXEL;
                    let li = (((y - base) * w + x) * 4) as usize;
                    aplica(
                        x,
                        y,
                        q,
                        &mut ladrilho[j..j + POR_TEXEL],
                        &mut linhas[li..li + 4],
                    );
                }
            }
        };
        let faixas: Vec<_> = tela
            .buf
            .chunks_mut((ALTO * w * 4) as usize)
            .zip(tela.sombra.ladrilhos.chunks_mut(colunas))
            .enumerate()
            .skip(ty0 as usize)
            .take((ty1 - ty0 + 1) as usize)
            .map(|(ty, (linhas, ladrilhos))| (ty as u32, linhas, ladrilhos))
            .collect();
        if paralelo {
            faixas.into_par_iter().for_each(pinta_faixa);
        } else {
            faixas.into_iter().for_each(pinta_faixa);
        }
    }
    Some(DirtyRect {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    })
}
