//! ⭐ **O que esta sonda MEDE** — tudo o que produz um NUMERO.
//!
//! ⚠️ O corte do `main.rs` (`1 256` linhas) e' por RESPONSABILIDADE: aqui as reguas,
//! no [`super::desenhos`] as imagens, e no `main.rs` as corridas. ⛔ A rasterizacao
//! ([`varre`]) mora AQUI e nao la': ela e' o percurso que a regua dos texels CONTA, e
//! o desenho usa-a de proposito — *se elas divergissem, a imagem e o numero seriam
//! dois factos diferentes.*

use ph2d_mesh::Mesh;

/// O comprimento de uma polilinha de vértices **globais**, em unidades de mundo.
pub(crate) fn chain_len(pos: &[[f32; 3]], chain: &[u32]) -> f64 {
    chain
        .windows(2)
        .map(|w| {
            let (a, b) = (pos[w[0] as usize], pos[w[1] as usize]);
            let d = [
                f64::from(a[0] - b[0]),
                f64::from(a[1] - b[1]),
                f64::from(a[2] - b[2]),
            ];
            d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
        })
        .sum()
}

/// A área **com sinal** de um triângulo no plano `(u, v)`.
pub(crate) fn uv_area2(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f64 {
    let (ux, uy) = (f64::from(b[0] - a[0]), f64::from(b[1] - a[1]));
    let (vx, vy) = (f64::from(c[0] - a[0]), f64::from(c[1] - a[1]));
    ux.mul_add(vy, -(uy * vx)) * 0.5
}

/// ⭐⭐⭐ **QUANTOS TEXELS O ATLAS PINTA DUAS VEZES** — a pergunta de CORRECÇÃO, na
/// unidade em que o artista a sente.
///
/// ⛔ Uma ilha assentada ao longo de uma árvore não tem holonomia **e pode dobrar-se sobre
/// si mesma** — nada no assentamento o impede. Um texel coberto por dois sítios da
/// superfície é tinta que aparece onde ninguém a pôs.
///
/// ⚠️ **Ela e a [`ph2d_uv_atlas::sobreposicao`] medem a MESMA coisa em unidades
/// diferentes, e as duas ficam:** esta conta TEXELS (que é o que se vê) e aquela conta
/// ÁREA exacta com a ATRIBUIÇÃO ao mecanismo (que é o que se corta). *Uma régua agregada
/// diz que há defeito e não diz o que se corta.*
///
/// Devolve `(texels cobertos, texels cobertos MAIS DE UMA VEZ)`.
pub(crate) fn sobreposicao(
    atlas: &ph2d_uv_atlas::Atlas,
    mesh: &Mesh,
    lado: usize,
) -> (usize, usize) {
    let mut n = vec![0u16; lado * lado];
    for t in ph2d_uv_atlas::topo::triangulos(mesh) {
        conta(
            &mut n,
            lado,
            [
                atlas.uv[t[0] as usize],
                atlas.uv[t[1] as usize],
                atlas.uv[t[2] as usize],
            ],
        );
    }
    let cobertos = n.iter().filter(|&&c| c > 0).count();
    let dobrados = n.iter().filter(|&&c| c > 1).count();
    (cobertos, dobrados)
}

/// A mesma varredura do [`preenche`], a CONTAR em vez de pintar.
///
/// ⚠️ Ela é uma segunda travessia do mesmo rectângulo de propósito: o pintor escreve a
/// última cor e a contagem soma, e juntar as duas num só percurso faria a imagem depender
/// de quem se sobrepõe. *Duas perguntas, duas varreduras.*
pub(crate) fn conta(n: &mut [u16], lado: usize, t: [[f32; 2]; 3]) {
    varre(lado, t, &mut |i| n[i] = n[i].saturating_add(1));
}

/// ⭐ **A VARREDURA, uma só** — quem pinta e quem conta percorrem exactamente os mesmos
/// texels, senão a imagem e o número descrevem atlas diferentes.
pub(crate) fn varre(lado: usize, t: [[f32; 2]; 3], f: &mut dyn FnMut(usize)) {
    let n = lado as f32;
    let p: Vec<[f32; 2]> = t.iter().map(|z| [z[0] * n, (1.0 - z[1]) * n]).collect();
    let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
    for q in &p {
        lo[0] = lo[0].min(q[0]);
        lo[1] = lo[1].min(q[1]);
        hi[0] = hi[0].max(q[0]);
        hi[1] = hi[1].max(q[1]);
    }
    let y0 = lo[1].floor().max(0.0) as usize;
    let y1 = (hi[1].ceil().max(0.0) as usize).min(lado);
    let x0 = lo[0].floor().max(0.0) as usize;
    let x1 = (hi[0].ceil().max(0.0) as usize).min(lado);
    for y in y0..y1 {
        for x in x0..x1 {
            let q = [x as f32 + 0.5, y as f32 + 0.5];
            let w = |a: [f32; 2], b: [f32; 2]| {
                (b[0] - a[0]).mul_add(q[1] - a[1], -((b[1] - a[1]) * (q[0] - a[0])))
            };
            let (u, v, s) = (w(p[0], p[1]), w(p[1], p[2]), w(p[2], p[0]));
            let dentro = (u >= 0.0 && v >= 0.0 && s >= 0.0) || (u <= 0.0 && v <= 0.0 && s <= 0.0);
            if dentro {
                f(y * lado + x);
            }
        }
    }
}

/// Uma contagem como `f64`, sem o `as` solto que o clippy da casa recusa.
pub(crate) fn as_f64(n: usize) -> f64 {
    u32::try_from(n).map_or(f64::from(u32::MAX), f64::from)
}

/// ⭐⭐⭐ **A fronteira somada das peças, em unidades de MUNDO.**
///
/// Uma aresta da malha conta quando os dois lados dela caem em peças diferentes — ou
/// quando ela é bordo da peça. ⚠️ **É medida na MALHA e não no atlas**, e é por isso que
/// ela é comparável entre duas arrumações: *o que o pintor sente é o comprimento do corte
/// na escultura, não no quadrado*.
pub(crate) fn fronteira_das_pecas(atlas: &ph2d_uv_atlas::Atlas, mesh: &Mesh) -> f64 {
    let (base, _) = ph2d_uv_atlas::bases_dos_cantos(mesh);
    let pos = mesh.positions();
    let mut por_aresta: std::collections::BTreeMap<(u32, u32), Vec<u32>> =
        std::collections::BTreeMap::new();
    for (f, face) in mesh.faces().iter().enumerate() {
        let v = face.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            por_aresta
                .entry((a.min(b), a.max(b)))
                .or_default()
                .push(u32::try_from(f).unwrap_or(0));
        }
    }
    let peca = |f: u32| {
        atlas
            .ilha
            .get(base[f as usize] as usize)
            .copied()
            .unwrap_or(u32::MAX)
    };
    let mut soma = 0.0;
    for ((a, b), faces) in &por_aresta {
        let fronteira = faces.len() != 2 || peca(faces[0]) != peca(faces[1]);
        if !fronteira {
            continue;
        }
        let (p, q) = (pos[*a as usize], pos[*b as usize]);
        let d = [
            f64::from(p[0] - q[0]),
            f64::from(p[1] - q[1]),
            f64::from(p[2] - q[2]),
        ];
        soma += d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
    }
    soma
}

/// ⭐⭐⭐ **O menor vão entre duas ilhas, em texels.**
///
/// Uma travessia em largura a partir de TODOS os texels pintados, de oito vizinhos: onde
/// duas frentes de ilhas diferentes se encontram, a soma das distâncias delas é o vão.
///
/// ⚠️ **Oito vizinhos e não quatro, e isso é o que a torna honesta para um gate:** a
/// distância de Chebyshev é `≤` à euclidiana, logo o número que sai daqui é um limite
/// INFERIOR do vão verdadeiro. *Com quatro vizinhos ela seria Manhattan, que está acima —
/// e uma régua que sobrestima um vão aprova um atlas que sangra.*
pub(crate) fn vao_entre_ilhas(
    atlas: &ph2d_uv_atlas::Atlas,
    mesh: &Mesh,
    lado: usize,
) -> Option<(usize, u32, u32)> {
    let mut dono = vec![u32::MAX; lado * lado];
    for t in ph2d_uv_atlas::topo::triangulos(mesh) {
        let ilha = atlas.ilha[t[0] as usize];
        if ilha == u32::MAX {
            continue;
        }
        let z = [
            atlas.uv[t[0] as usize],
            atlas.uv[t[1] as usize],
            atlas.uv[t[2] as usize],
        ];
        varre(lado, z, &mut |i| {
            if dono[i] == u32::MAX {
                dono[i] = ilha;
            }
        });
    }
    let mut dist = vec![u16::MAX; lado * lado];
    let mut fila: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
    for (i, &d) in dono.iter().enumerate() {
        if d != u32::MAX {
            dist[i] = 0;
            fila.push_back(i);
        }
    }
    let mut melhor: Option<(usize, u32, u32)> = None;
    while let Some(c) = fila.pop_front() {
        let (cx, cy) = (c % lado, c / lado);
        // ⭐ O tecto: um vão maior que o dobro do pedido não interessa a ninguém, e sem
        // ele a travessia varre o quadrado inteiro.
        if usize::from(dist[c]) > 2 * (ph2d_uv_atlas::VAO_EM_TEXELS as usize) {
            break;
        }
        for dy in -1i64..=1 {
            for dx in -1i64..=1 {
                let (nx, ny) = (cx as i64 + dx, cy as i64 + dy);
                if nx < 0 || ny < 0 || nx >= lado as i64 || ny >= lado as i64 {
                    continue;
                }
                let n = ny as usize * lado + nx as usize;
                if dono[n] == u32::MAX {
                    dono[n] = dono[c];
                    dist[n] = dist[c] + 1;
                    fila.push_back(n);
                } else if dono[n] != dono[c] {
                    let sep = usize::from(dist[n]) + usize::from(dist[c]);
                    if melhor.is_none_or(|(m, _, _)| sep < m) {
                        melhor = Some((sep, dono[c].min(dono[n]), dono[c].max(dono[n])));
                    }
                }
            }
        }
    }
    melhor
}

/// A área de um triângulo no MUNDO.
pub(crate) fn area3(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f64 {
    let u = [
        f64::from(b[0] - a[0]),
        f64::from(b[1] - a[1]),
        f64::from(b[2] - a[2]),
    ];
    let v = [
        f64::from(c[0] - a[0]),
        f64::from(c[1] - a[1]),
        f64::from(c[2] - a[2]),
    ];
    let n = [
        u[1].mul_add(v[2], -(u[2] * v[1])),
        u[2].mul_add(v[0], -(u[0] * v[2])),
        u[0].mul_add(v[1], -(u[1] * v[0])),
    ];
    0.5 * n[0].mul_add(n[0], n[1].mul_add(n[1], n[2] * n[2])).sqrt()
}

/// Uma amostra da régua da densidade: quantas unidades de `(u, v)` por unidade de mundo,
/// e com que peso (a área da superfície que ela cobre).
pub(crate) struct Amostra {
    pub(crate) densidade: f64,
    pub(crate) area: f64,
    pub(crate) ilha: u32,
}

/// O percentil `q` de uma lista ORDENADA por densidade, pesado pela ÁREA.
///
/// ⚠️ **O peso é a área da SUPERFÍCIE e não a contagem de triângulos**, e a escolha é a
/// régua: o que o artista vê é quanta PEÇA está borrada, e uma malha adensada numa ponta
/// tem ali mil triângulos minúsculos que uma mediana por contagem deixaria mandar.
pub(crate) fn percentil(ordenado: &[Amostra], q: f64) -> f64 {
    let total: f64 = ordenado.iter().map(|a| a.area).sum();
    if total <= 0.0 {
        return 0.0;
    }
    let alvo = q * total;
    let mut acc = 0.0;
    for a in ordenado {
        acc += a.area;
        if acc >= alvo {
            return a.densidade;
        }
    }
    ordenado.last().map_or(0.0, |a| a.densidade)
}

/// ⭐⭐⭐ **A DENSIDADE DE TEXELS, e o ESPALHAMENTO dela.**
///
/// ⛔⛔ **Todas as réguas desta sonda até aqui mediram QUANTO do quadrado é usado; nenhuma
/// mede se ele é usado POR IGUAL.** Um atlas com `0,00 %` de sobreposição, `8` texels de
/// vão e `42 %` de tinta pode entregar a peça **nítida de um lado e borrada do outro** —
/// e o artista lê isso como o pincel a mudar de tamanho ao andar sobre a escultura.
///
/// A grandeza é `√(área em uv / área no mundo)`: unidades de `(u, v)` por unidade de
/// mundo, que a `N` texels por lado é `N ×` texels por unidade de mundo.
///
/// ⭐ **E ela vem com a ATRIBUIÇÃO**, porque as duas causas têm curas opostas: se o
/// espalhamento estiver **DENTRO** de cada ilha, quem estica é o parametrizador (a
/// montante, no G3); se estiver **ENTRE** ilhas, quem estica é a arrumação (aqui).
/// O que a régua da densidade devolve. Cada faixa é `[p05, p95]` em múltiplos da
/// mediana a que ela se compara.
pub(crate) struct Densidade {
    /// A mediana, em unidades de `(u, v)` por unidade de mundo.
    pub(crate) p50: f64,
    /// O espalhamento sobre a peça INTEIRA.
    pub(crate) global: [f64; 2],
    /// O espalhamento das MEDIANAS de cada ilha — a parte que a arrumação explica.
    pub(crate) entre: [f64; 2],
    /// O espalhamento de cada triângulo contra a mediana da PRÓPRIA ilha — a parte que
    /// o parametrizador explica.
    pub(crate) dentro: [f64; 2],
}

pub(crate) fn densidade(atlas: &ph2d_uv_atlas::Atlas, mesh: &Mesh) -> Option<Densidade> {
    let (base, _) = ph2d_uv_atlas::bases_dos_cantos(mesh);
    let pos = mesh.positions();
    let mut am: Vec<Amostra> = Vec::new();
    for (f, face) in mesh.faces().iter().enumerate() {
        let vs = face.verts();
        let b = base[f] as usize;
        for k in 1..vs.len().saturating_sub(1) {
            let mundo = area3(
                pos[vs[0] as usize],
                pos[vs[k] as usize],
                pos[vs[k + 1] as usize],
            );
            let plano = uv_area2(atlas.uv[b], atlas.uv[b + k], atlas.uv[b + k + 1]).abs();
            if mundo <= 0.0 || plano <= 0.0 {
                continue;
            }
            am.push(Amostra {
                densidade: (plano / mundo).sqrt(),
                area: mundo,
                ilha: atlas.ilha[b],
            });
        }
    }
    if am.is_empty() {
        return None;
    }
    am.sort_by(|x, y| x.densidade.total_cmp(&y.densidade));
    let p50 = percentil(&am, 0.5);
    let global = [
        percentil(&am, 0.05) / p50.max(1.0e-12),
        percentil(&am, 0.95) / p50.max(1.0e-12),
    ];

    // ⭐ A ATRIBUIÇÃO. A mediana de cada ilha contra a mediana global dá o espalhamento
    // ENTRE ilhas; cada amostra contra a mediana da PRÓPRIA ilha dá o de DENTRO.
    let mut por_ilha: std::collections::BTreeMap<u32, Vec<Amostra>> =
        std::collections::BTreeMap::new();
    for a in am {
        por_ilha.entry(a.ilha).or_default().push(Amostra {
            densidade: a.densidade,
            area: a.area,
            ilha: a.ilha,
        });
    }
    let mut medianas: Vec<Amostra> = Vec::new();
    let mut dentro: Vec<Amostra> = Vec::new();
    for (i, lista) in &por_ilha {
        let m = percentil(lista, 0.5);
        medianas.push(Amostra {
            densidade: m,
            area: lista.iter().map(|a| a.area).sum(),
            ilha: *i,
        });
        for a in lista {
            dentro.push(Amostra {
                densidade: a.densidade / m.max(1.0e-12),
                area: a.area,
                ilha: *i,
            });
        }
    }
    medianas.sort_by(|x, y| x.densidade.total_cmp(&y.densidade));
    dentro.sort_by(|x, y| x.densidade.total_cmp(&y.densidade));
    let m50 = percentil(&medianas, 0.5);
    let entre = [
        percentil(&medianas, 0.05) / m50.max(1.0e-12),
        percentil(&medianas, 0.95) / m50.max(1.0e-12),
    ];
    Some(Densidade {
        p50,
        global,
        entre,
        dentro: [percentil(&dentro, 0.05), percentil(&dentro, 0.95)],
    })
}

/// ⭐⭐ **Quantos vértices a PLACA teria de duplicar.**
///
/// ⚠️ A UV é por CANTO e um buffer de vértices é por VÉRTICE: um vértice sobre um corte
/// tem uma `(u, v)` de cada lado, e desenhar isso obriga a parti-lo em cópias. Esta é a
/// costura contada em VÉRTICES em vez de em comprimento — a grandeza que decide se o
/// canal cabe na `Mesh` como está ou se a malha de desenho é uma segunda malha.
///
/// ⚠️ **A comparação é AO BIT, de propósito.** Dentro de uma peça todo canto do mesmo
/// vértice atravessa o mesmo deslocamento e a mesma rotação, logo sai com os mesmos bits;
/// uma tolerância aqui fundiria cópias que o atlas separou de propósito.
pub(crate) fn cantos_duplicados(atlas: &ph2d_uv_atlas::Atlas, mesh: &Mesh) -> (usize, usize) {
    let (base, _) = ph2d_uv_atlas::bases_dos_cantos(mesh);
    let mut por_vertice: Vec<Vec<[u32; 2]>> = vec![Vec::new(); mesh.positions().len()];
    for (f, face) in mesh.faces().iter().enumerate() {
        let b = base[f] as usize;
        for (k, &v) in face.verts().iter().enumerate() {
            let uv = atlas.uv[b + k];
            let chave = [uv[0].to_bits(), uv[1].to_bits()];
            let slot = &mut por_vertice[v as usize];
            if !slot.contains(&chave) {
                slot.push(chave);
            }
        }
    }
    (por_vertice.iter().map(Vec::len).sum(), por_vertice.len())
}
