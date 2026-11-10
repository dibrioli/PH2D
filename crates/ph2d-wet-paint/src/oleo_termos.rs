//! O SOMBREADO do Óleo Real: os [`Termos`] de uma célula sobre o [`super::CampoAltura`] (a lâmpada, o ambiente, o
//! metal, o verniz e o veludo) e a [`LuzCeu`] que a Subsurface guarda. Partido de `oleo_real.rs` (B52.R27, 10/11:
//! montagem do campo × sombreado), sem mudar uma conta.

use super::*;

/// O alargamento máximo do lobo (α²) pelo antisserrilhado — o tecto de Kaplanyan 2016 (`0,18`).
const VARIANCIA_MAX_BRILHO: f64 = 0.18;

/// `F0` do ligante (dieléctrico).
const F0: f64 = 0.04;

/// A luz directa, a visibilidade BRUTA (antes da força da fenda) e o ganho do céu de uma célula.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LuzCeu {
    pub luz: f64,
    pub v: f64,
    pub dir: Option<[f64; 3]>,
}

/// O que a [`LuzCeu`] lê da óptica (além do campo): com outra chave a célula volta a ser calculada.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ChaveLuz {
    azimute: f64,
    altitude: f64,
    sombra: f64,
    penumbra: f64,
    ambiente: Ambiente,
    alcance_fenda: f64,
    qualidade: QualidadeDaLuz,
}

impl ChaveLuz {
    pub(super) fn de(o: &OleoReal) -> Self {
        Self {
            azimute: o.azimute,
            altitude: o.altitude,
            sombra: o.sombra,
            penumbra: o.af.penumbra,
            ambiente: o.ambiente,
            alcance_fenda: o.alcance_fenda,
            qualidade: o.qualidade,
        }
    }
}

/// A [`LuzCeu`] da célula `(x, y)` contada de raiz.
pub(crate) fn luz_ceu(o: &OleoReal, campo: &CampoAltura, x: isize, y: isize) -> LuzCeu {
    let n = normal(campo, x, y);
    let luz = luz_directa(o, campo, x, y, campo.at_sombra(x, y));
    let ceu = CeuDoMapa::de(o);
    let horizonte = (escala_do_horizonte(o), o.qualidade);
    let (v, dir) = visibilidade_e_ceu(campo, x, y, n, horizonte, ceu.as_ref());
    LuzCeu { luz, v, dir }
}

/// O fim da [`sombra_no_papel`]: `luz` directa, a visibilidade `vis` já com a força da fenda, e o `dir` do céu.
pub(super) fn sombra_de(o: &OleoReal, luz: f64, vis: f64, dir: Option<[f64; 3]>) -> f64 {
    let directa = (1.0 - o.forca_ambiente) * (1.0 - luz);
    let tapado = if o.sombra <= 0.0 {
        0.0
    } else {
        // Com HDRI o papel recebe o céu do SECTOR que vê: ao pé da parede que tapa a janela, a sombra macia
        // vem do lado dela (a média dos canais: o plano da sombra é um só).
        let recebe = dir.map_or(vis, |d| vis * (d[0] + d[1] + d[2]) / 3.0);
        o.sombra * o.forca_ambiente * (1.0 - recebe)
    };
    (directa + tapado).clamp(0.0, 1.0)
}

/// O ambiente roda-se em torno de z para o lado claro dele ficar no azimute da luz; `d` na moldura da
/// tela (y para BAIXO) vai para a do mapa (a do Blender, y para cima).
pub(super) fn para_o_mapa(o: &OleoReal, claro: f64, d: [f64; 3]) -> [f64; 3] {
    let l = para_a_luz(o);
    let rot = claro - libm::atan2(-l[1], l[0]);
    let (c, s) = (libm::cos(rot), libm::sin(rot));
    let b = [d[0], -d[1], d[2]];
    [b[0] * c - b[1] * s, b[0] * s + b[1] * c, b[2]]
}

/// O factor das distâncias de [`HORIZONTE`] pelo [`OleoReal::alcance_fenda`] (`1` no de fábrica).
pub(super) fn escala_do_horizonte(o: &OleoReal) -> f64 {
    o.alcance_fenda.clamp(1.0, ALCANCE_FENDA_MAX) / ALCANCE_FENDA_MAX
}

/// Os [`Termos`] da célula `(cx, cy)` com a célula de material NEUTRO (o do cartão vale) e a tinta cinza
/// `0,5` (a que o metal e o tint reflectem).
pub fn termos(o: &OleoReal, campo: &CampoAltura, cx: usize, cy: usize) -> Termos {
    termos_com(o, campo, cx, cy, &MaterialCelula::NEUTRO, [0.5; 3])
}

/// As peças da óptica que variam DENTRO da célula (A.1, [`crate::oleo_vista`]): a normal, a hessiana da altura fina
/// (`hxx`, `hyy`, `hxy`, em unidades da célula), a luz e o céu, e o Gloss. A célula inteira dá-as
/// [`Amostra::da_celula`]; a vista dá-as do campo ampliado.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Amostra {
    pub n: [f64; 3],
    pub hxx: f64,
    pub hyy: f64,
    pub hxy: f64,
    pub lc: LuzCeu,
    pub brilho: f64,
}

impl Amostra {
    /// As peças da célula `(x, y)`, as de sempre.
    pub(crate) fn da_celula(o: &OleoReal, campo: &CampoAltura, x: isize, y: isize) -> Self {
        let hf = |dx: isize, dy: isize| campo.at_fina(x + dx, y + dy);
        Self {
            n: normal(campo, x, y),
            hxx: hf(1, 0) - 2.0 * hf(0, 0) + hf(-1, 0),
            hyy: hf(0, 1) - 2.0 * hf(0, 0) + hf(0, -1),
            hxy: (hf(1, 1) - hf(1, -1) - hf(-1, 1) + hf(-1, -1)) / 4.0,
            lc: luz_ceu_da_celula(o, campo, x, y),
            brilho: campo.brilho_em(x as usize, y as usize),
        }
    }
}

/// A [`LuzCeu`] da célula `(x, y)`: a guardada no campo ou contada de raiz.
pub(crate) fn luz_ceu_da_celula(o: &OleoReal, campo: &CampoAltura, x: isize, y: isize) -> LuzCeu {
    campo
        .luz_ceu_em(o, x, y)
        .unwrap_or_else(|| luz_ceu(o, campo, x, y))
}

/// A cor sRGB `0..255` da tinta `rgb` com a [`Amostra`] `am`; o material, a Subsurface e o reflexo livre são os da
/// célula `(cx, cy)`. Com `Amostra::da_celula` é [`ilumina`] bit a bit.
pub(crate) fn cor_da_amostra(
    o: &OleoReal,
    campo: &CampoAltura,
    cx: usize,
    cy: usize,
    rgb: [f64; 3],
    cobertura: f64,
    am: &Amostra,
) -> [f64; 3] {
    let a = rgb.map(srgb255_to_linear);
    let t = termos_da_amostra(o, campo, cx, cy, &campo.material_em(cx, cy), a, am);
    cor_dos_termos(o, &t, a, cobertura)
}

/// Os [`Termos`] da célula `(cx, cy)` com o material `mat` dela e a cor LINEAR `a` (o F0 do metal e o tint).
pub fn termos_com(
    o: &OleoReal,
    campo: &CampoAltura,
    cx: usize,
    cy: usize,
    mat: &MaterialCelula,
    a: [f64; 3],
) -> Termos {
    let am = Amostra::da_celula(o, campo, cx as isize, cy as isize);
    termos_da_amostra(o, campo, cx, cy, mat, a, &am)
}

/// [`termos_com`] com as peças de dentro da célula dadas por `am`.
pub(crate) fn termos_da_amostra(
    o: &OleoReal,
    campo: &CampoAltura,
    cx: usize,
    cy: usize,
    mat: &MaterialCelula,
    a: [f64; 3],
    am: &Amostra,
) -> Termos {
    let af = &o.af;
    let m = crate::material::efectivo(mat, &o.material);
    // O Gloss: o da tinta da célula (dono 28/10) sobre o ganho do motor (`1` no produto).
    let brilho = o.brilho * am.brilho;
    let (x, y) = (cx as isize, cy as isize);
    let n = am.n;
    let l = para_a_luz(o);
    let escala = escala_do_horizonte(o);
    // W2 E2: com HDRI o DIFUSO do céu é a irradiância do SECTOR que cada fatia vê (nunca a do HDRI × a
    // visibilidade média: a janela do estúdio entrava nas fendas que as paredes tapam, 1,09 × 0,55).
    let LuzCeu {
        luz,
        v: v_bruta,
        dir,
    } = am.lc;
    let vis = forca_da_fenda(v_bruta, o.fenda);
    let forca = o.forca_ambiente;

    // Difuso da lâmpada relativo: plano iluminado = 1 − força do ambiente.
    let ndl = (n[0] * l[0] + n[1] * l[1] + n[2] * l[2]).max(0.0);
    let sol = (1.0 - forca) * (ndl / l[2]) * luz;

    // Brilho da lâmpada: GGX + Schlick, vista ortográfica (0,0,1), só onde a luz directa chega.
    let hv = [l[0], l[1], l[2] + 1.0];
    let hn = exato::sqrt(hv[0] * hv[0] + hv[1] * hv[1] + hv[2] * hv[2]);
    let ndh = ((n[0] * hv[0] + n[1] * hv[1] + n[2] * hv[2]) / hn).max(0.0);
    // Antisserrilhado do brilho (Kaplanyan 2016): o lobo alarga-se pela variação da normal dentro da célula
    // (a hessiana da altura fina) — numa encosta estreita o pico de GGX saturava em pontos brancos.
    let (hxx, hyy, hxy) = (am.hxx, am.hyy, am.hxy);
    let var = 0.25 * (hxx * hxx + hyy * hyy + 2.0 * hxy * hxy) * n[2] * n[2];
    // A Roughness ABSOLUTA do material (célula ∘ cartão; no neutro a `RUGOSIDADE_PADRAO`, bit a bit).
    let aa = (2.0 * var * af.antisserrilhado).min(VARIANCIA_MAX_BRILHO);
    let aa_amb = (2.0 * var * af.antisserrilhado_ambiente).min(VARIANCIA_MAX_BRILHO);
    let rugo = m.rugosidade;
    let a2 = rugo * rugo * rugo * rugo + aa;
    let ggx = |a2: f64| {
        let den = ndh * ndh * (a2 - 1.0) + 1.0;
        a2 / (core::f64::consts::PI * den * den)
    };
    let d_ggx = ggx(a2);
    let vdh = hv[2] / hn;
    let p5 = libm::pow(1.0 - vdh, 5.0);
    // O F0 dieléctrico: IOR Level `0,5` = IOR 1,5 (`F0 = 0,08·s`; o F90 é branco).
    let f0d = F0 * (m.nivel_ior / 0.5);
    // IOR Level abaixo de `0,5` apaga também a BORDA (o Principled a `0` não reflecte nada: IOR 1).
    let borda = (m.nivel_ior / 0.5).min(1.0);
    // O lobo da lâmpada é o microfacet do Principled (Cook–Torrance, Smith GGX correlacionado em altura, o do
    // Cycles): `D·F·G / (4·n·l·n·v)` × a irradiância da lâmpada `π·(1 − força)/l_z` (o plano de albedo 1 = 1) ×
    // `n·l`. Cura 02/11c: o lobo de antes (`brilho_max 1,6 · F·D·n·l`, sem o `G/(4·n·v)` nem a irradiância) dava
    // no miolo do Dabber 3,2× o físico (0,0332 × 0,0103; Cycles 0,0093) — `cura-cycles.md`.
    let lambda = |c: f64, a2: f64| {
        let c2 = (c * c).max(1e-12);
        0.5 * (exato::sqrt(1.0 + a2 * (1.0 - c2) / c2) - 1.0)
    };
    let nv = n[2].max(1e-4);
    let irradiancia = core::f64::consts::PI * (1.0 - forca) / l[2].max(1e-4);
    let lampada_de = |f0: f64, borda: f64, d: f64, a2: f64| {
        if ndl <= 0.0 {
            return 0.0;
        }
        let fres = f0 + (1.0 - f0) * p5 * borda;
        let g = 1.0 / (1.0 + lambda(ndl, a2) + lambda(nv, a2));
        brilho * af.brilho_max * fres * d * g / (4.0 * nv) * luz * irradiancia
    };
    let lampada = [lampada_de(f0d, borda, d_ggx, a2); 3];

    // Brilho do ambiente (split-sum: pré-filtrado × F0·A + B, Karis 2014), tapado pela visibilidade. Sem mapa
    // (Clássico) o metal e o verniz reflectem o céu uniforme.
    let mapa = o.ambiente.mapa();
    // `(pré-filtrado por canal, A, B, fracção livre)` do lobo de rugosidade `a2` com a normal `nr`.
    let ambiente_de = |a2: f64, nr: [f64; 3]| {
        let r = [
            2.0 * nr[2] * nr[0],
            2.0 * nr[2] * nr[1],
            2.0 * nr[2] * nr[2] - 1.0,
        ];
        let rug = exato::sqrt(exato::sqrt(a2));
        let nv = nr[2].max(1e-4);
        let r0 = -rug + 1.0;
        let a004 = (r0 * r0).min(libm::exp2(-9.28 * nv)) * r0 + (-0.0275 * rug + 0.0425);
        let xa = a004 * -1.04 + (-0.572 * rug + 1.04);
        let yb = a004 * 1.04 + (0.022 * rug - 0.04);
        let pre = mapa.map_or([1.0; 3], |mp| mp.brilho(para_o_mapa(o, mp.claro, r), rug));
        let livre = reflexo_livre(campo, x, y, r, rug, (escala, o.qualidade));
        (pre, xa, yb, livre)
    };
    let a2_amb = rugo * rugo * rugo * rugo + aa_amb;
    let base = (mapa.is_some() || m.metal > 0.0).then(|| ambiente_de(a2_amb, n));
    let mut espec = match (mapa, base) {
        (Some(_), Some((pre, xa, yb, livre))) => [0, 1, 2].map(|c| {
            let ab = f0d * xa + yb * borda;
            lampada[c] + brilho * forca * pre[c] * ab * livre
        }),
        _ => lampada,
    };
    // METAL: F0 = a cor da tinta, sem difuso. O que a parede vizinha TAPA no reflexo não é
    // preto: é a tinta dela (a mesma tinta: o difuso dela, ou o que ela reflecte com o F0 dela se é metal,
    // `a·difuso·mix(1, a, metal)`). O lobo da lâmpada do metal alarga-se como o do AMBIENTE (o alargamento
    // pela curvatura do dieléctrico punha o sol em toda a chapa lisa: R 0,18 × 0,05 do Cycles) — §E3.
    if let (true, Some((pre, xa, yb, livre))) = (m.metal > 0.0, base) {
        let sem_brilho = Termos {
            vis,
            luz,
            sol,
            espec: [0.0; 3],
            dir,
            metal: 0.0,
            sob_verniz: 1.0,
            sss: [0.0; 3],
            transmissao: 0.0,
            vidro_claro: 0.0,
        };
        let d_metal = ggx(a2_amb);
        for c in 0..3 {
            let f0m = a[c].clamp(0.0, 1.0);
            let ab = f0m * xa + yb;
            let viz = a[c] * difuso(o, &sem_brilho, a[c], c) * (1.0 + (a[c] - 1.0) * m.metal);
            let em = lampada_de(f0m, 1.0, d_metal, a2_amb)
                + brilho * (forca * pre[c] * livre + (1.0 - livre) * viz) * ab;
            espec[c] = (1.0 - m.metal) * espec[c] + m.metal * em;
        }
    }
    // VERNIZ: um dieléctrico liso por cima (IOR 1,5), de rugosidade própria; a camada de baixo recebe o que
    // o Fresnel dele deixa passar.
    let mut sob_verniz = 1.0;
    if m.verniz > 0.0 {
        let rc = m.verniz_rugosidade.clamp(0.0, 1.0);
        let a2c = rc * rc * rc * rc + aa;
        let (pre, xa, yb, livre) = ambiente_de(rc * rc * rc * rc + aa_amb, n);
        let fv = F0 + (1.0 - F0) * libm::pow(1.0 - n[2].clamp(0.0, 1.0), 5.0);
        sob_verniz = 1.0 - m.verniz * fv;
        let lv = lampada_de(F0, 1.0, ggx(a2c), a2c);
        for c in 0..3 {
            let ev = lv + brilho * forca * pre[c] * (F0 * xa + yb) * livre;
            espec[c] = espec[c] * sob_verniz + m.verniz * ev;
        }
    }
    let mut t = Termos {
        vis,
        luz,
        sol,
        espec,
        dir,
        metal: m.metal,
        sob_verniz,
        sss: if m.subsuperficie > 0.0 {
            campo
                .subsuperficie_em(cx, cy)
                .map(|s| m.subsuperficie.min(1.0) * s)
        } else {
            [0.0; 3]
        },
        transmissao: m.transmissao,
        vidro_claro: m.vidro_claro,
    };
    // VELUDO: o sheen branco, mais forte onde a normal foge da vista, à luz que a célula recebe.
    if m.veludo > 0.0 {
        for c in 0..3 {
            let e = difuso(o, &t, 1.0, c);
            t.espec[c] += oleo_material::veludo(m.veludo, n[2], e) * sob_verniz;
        }
    }
    t
}
