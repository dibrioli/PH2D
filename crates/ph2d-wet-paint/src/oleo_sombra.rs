//! A SOMBRA no papel, o rebote e a TRANSMISSÃO/Transparency de quem tapa (partido de `oleo_real.rs`, 29/10).

use super::*;

/// **A sombra no PAPEL** (M6: cai também fora da tinta): a fracção `0..1` de escurecimento de uma
/// célula SEM tinta — a luz directa que a tinta à volta lhe tira, `(1 − céu)·(1 − luz)`, e o céu que ela
/// lhe tapa ao pé, `céu·(1 − V)` (o Cycles: visibilidade 0,68 no p5 do papel). O difuso, a fenda e o
/// brilho são da tinta. O PAPEL é plano também sob a orla parcial de uma célula pintada (§BUG-ressalto: a
/// normal inclinada da orla dava-lhe o céu inteiro — um fio claro entre a tinta e a sombra dela).
pub fn sombra_no_papel(o: &OleoReal, campo: &CampoAltura, cx: usize, cy: usize) -> f64 {
    let (x, y) = (cx as isize, cy as isize);
    // A luz do céu guardada é a da normal da célula: só serve ao papel onde ela é a do plano (sem relevo).
    if o.sombra > 0.0
        && campo.at_fina(x, y) == 0.0
        && let Some(lc) = campo.luz_ceu_em(o, x, y)
    {
        return menos_rebote(
            o,
            campo,
            cx,
            cy,
            sombra_de(o, lc.luz, forca_da_fenda(lc.v, o.fenda), lc.dir),
        );
    }
    let luz = luz_directa(o, campo, x, y, campo.at_sombra(x, y));
    let (vis, dir) = if o.sombra <= 0.0 {
        (1.0, None)
    } else {
        let ceu = CeuDoMapa::de(o);
        let (v, dir) = visibilidade_e_ceu(
            campo,
            x,
            y,
            PAPEL_PLANO,
            escala_do_horizonte(o),
            ceu.as_ref(),
        );
        (forca_da_fenda(v, o.fenda), dir)
    };
    menos_rebote(o, campo, cx, cy, sombra_de(o, luz, vis, dir))
}

/// A sombra `s` do papel menos o [`rebote_no_papel`] (sem rebote, `s` tal e qual).
pub(super) fn menos_rebote(o: &OleoReal, campo: &CampoAltura, cx: usize, cy: usize, s: f64) -> f64 {
    if campo.albedo.is_empty() || s <= 0.0 {
        return s;
    }
    (s - rebote_no_papel(o, campo, cx, cy)).max(0.0)
}

/// **O rebote de UM salto da tinta vizinha no papel** (K10, `oraculo/README.md` §K10), em fracção da luz do plano
/// aceso. Em cada lado de cada fatia do GTAO (a mesma marcha da visibilidade), pelo cosseno da normal do papel:
/// o arco da horizontal até ao horizonte é a PAREDE, que devolve a cor dela × a luz da FACE virada para o papel
/// (o céu de uma face vertical, `½`, e o sol que lhe bate, `0` do lado escuro). A sombra projectada atrás de um
/// empaste alto continua escura: a face que ela vê é a de costas para a luz. O papel é plano (§BUG-ressalto): o
/// «chão inclinado» da K10 era a saia da forma desfocada e saiu com ela.
#[must_use]
pub fn rebote_no_papel(o: &OleoReal, campo: &CampoAltura, cx: usize, cy: usize) -> f64 {
    use core::f64::consts::FRAC_PI_2;
    if campo.albedo.is_empty() || o.sombra <= 0.0 {
        return 0.0;
    }
    let (x, y) = (cx as isize, cy as isize);
    let h0 = campo.at_fina(x, y);
    // O papel é PLANO (§BUG-ressalto): o relevo não sai do pigmento, e sob a orla parcial ele é o chão.
    let n = PAPEL_PLANO;
    let l = para_a_luz(o);
    let forca = o.forca_ambiente;
    let escala = escala_do_horizonte(o);
    let lum = |q: (isize, isize)| {
        let k = campo.idx(q.0, q.1);
        let a = campo.albedo[k];
        let l = 0.2126 * f64::from(a[0]) + 0.7152 * f64::from(a[1]) + 0.0722 * f64::from(a[2]);
        // A parede TRANSPARENTE (a Transparency) mostra o papel branco por trás: `1 − (1 − t)·(1 − A)`.
        let t = crate::material::peso(campo.material.get(k).map_or(0.0, |m| m.transparencia));
        if t > 0.0 {
            1.0 - (1.0 - t) * (1.0 - l)
        } else {
            l
        }
    };
    // `∫₀^a cos(θ − g)·sin θ dθ`: o peso pelo cosseno de um arco da fatia (o da visibilidade).
    let int =
        |a: f64, g: f64| 0.25 * (-libm::cos(2.0 * a - g) + libm::cos(g) + 2.0 * a * libm::sin(g));
    Amostras::com(escala, |am| {
        let mut soma = 0.0;
        for k in 0..FATIAS {
            let (dx, dy) = am.dir[k];
            let no = -n[0] * dy + n[1] * dx;
            let np = [n[0] + no * dy, n[1] - no * dx, n[2]];
            let len = exato::sqrt(np[0] * np[0] + np[1] * np[1] + np[2] * np[2]);
            if len < 1e-9 {
                continue;
            }
            let g = libm::acos((np[2] / len).clamp(-1.0, 1.0)).copysign(np[0] * dx + np[1] * dy);
            for (sgn, lado) in [1.0, -1.0].into_iter().zip(&am.desloc[k]) {
                let gs = sgn * g;
                // O limite da meia-esfera da normal deste lado (abaixo da horizontal se ela pende para cá).
                let fim = FRAC_PI_2 + gs;
                let (mut e, mut parede) = (0.0f64, (x, y));
                for &(ox, oy, d) in lado {
                    if d == 0.0 {
                        continue;
                    }
                    let q = (x + ox, y + oy);
                    let dh = campo.at_fina(q.0, q.1) - h0;
                    let sen = dh / exato::sqrt(dh * dh + d * d);
                    if sen > e {
                        (e, parede) = (sen, q);
                    }
                }
                let mut luz = 0.0;
                if e > 0.0 {
                    let (a, b) = (FRAC_PI_2 - libm::asin(e), fim.min(FRAC_PI_2));
                    if b > a {
                        // A face olha de volta para o papel: normal horizontal `−(dx, dy)·sgn`.
                        let ndl = -(sgn * dx * l[0] + sgn * dy * l[1]);
                        let face = 0.5 * forca + (1.0 - forca) * ndl.max(0.0) / l[2];
                        luz += (int(b, gs) - int(a, gs)) * lum(parede) * face;
                    }
                }
                soma += len * luz;
            }
        }
        o.sombra * o.af.rebote_papel * soma / FATIAS as f64
    })
}

/// A [`sombra_no_papel`] de uma célula PINTADA a partir dos [`Termos`] que a cor dela já calculou (a luz
/// directa, a visibilidade e o ganho do céu são as MESMAS peças): bit a bit a conta por célula, a metade do
/// custo (gate `a_sombra_dos_termos_e_bit_identica_a_do_papel`).
#[must_use]
pub fn sombra_dos_termos(o: &OleoReal, t: &Termos) -> f64 {
    sombra_de(o, t.luz, t.vis, t.dir)
}

/// ⭐ **A sombra no papel sob a tinta TRANSPARENTE** (Transparency `t`, dono 27/10; medida contra o Cycles com o
/// Alpha do Principled, `oraculo/README.md` §Transparency): a fracção `s` que a tinta OPACA tiraria ao papel da
/// célula de cobertura geométrica `a` (`0..1`, a orla parcial). A luz que a tinta projectaria ATRAVESSA-A por duas
/// faces (entra pela parede, sai pelo topo): passa `t²` — `s·(1 − t²)`; o Cycles com o Alpha na mesma malha dá a
/// sombra a T 0,5 em `0,77` da de T 0 (`1 − t²` = `0,75`; `1 − t` seria `0,5`). O papel SOB a tinta recebe a luz
/// que atravessa o topo (`× t`) e é visto através dele: a parte coberta pesa `a·t²` em vez de `a·t` no composite
/// `a·(1 − t)` por cima. `t = 0` devolve `s` ao bit; `t = 1`, `0` (o Alpha 0: a tinta não projecta nada).
#[must_use]
pub fn sombra_transparente(s: f64, a: f64, t: f64) -> f64 {
    sombra_transparente_a_luz(s, a, t, t)
}

/// A [`sombra_transparente`] com a luz que ATRAVESSA a tinta à parte (`t_luz`, a [`transparencia_a_luz`]: o Alpha e
/// a Transmission) do que se VÊ do papel sob ela (`t`, só o Alpha — o papel visto pelo filme transmissivo já está na
/// cor da tinta, [`crate::material::transmite`]). `t_luz = t` é a lei de antes ao bit.
#[must_use]
pub fn sombra_transparente_a_luz(s: f64, a: f64, t: f64, t_luz: f64) -> f64 {
    if t_luz <= 0.0 {
        return s;
    }
    let atravessa = s * (1.0 - t_luz * t_luz);
    let deixa = 1.0 - a * (1.0 - t);
    if deixa <= 0.0 {
        return atravessa;
    }
    (1.0 - (1.0 - atravessa) * (a * t * t + (1.0 - a)) / deixa).clamp(0.0, 1.0)
}

/// ⭐ **A transparência À LUZ de uma tinta** de Transparency `t` e Transmission `w` com albedo linear `a` no modo
/// `claro` — o `t` equivalente que a [`sombra_transparente`] recebe, a MESMA conta sob a tinta (`render`) e no papel
/// nu ([`transparencia_de_quem_tapa`]). Clear glass (`claro = 1`): a luz atravessa o Alpha (`t²`) ou passa tingida
/// pelo [`crate::material::vidro`] (`w·L(vidro(a))`) — `t² = 1 − (1 − t²)(1 − w·L(vidro(a)))`. Blender (`claro = 0`):
/// a Transmission não passa luz para o papel (o Cycles: a sombra da Transmission 1 = a da opaca, `oraculo/README.md`
/// §Transmission «Ronda 4») — só o Alpha; entre os dois, `claro·w`. `w·claro = 0` devolve `t` ao bit.
#[must_use]
pub fn transparencia_a_luz(t: f64, w: f64, a: [f64; 3], claro: f64) -> f64 {
    let w = w * claro.clamp(0.0, 1.0);
    if w <= 0.0 {
        return t;
    }
    let v = a.map(crate::material::vidro);
    let passa = w * (0.2126 * v[0] + 0.7152 * v[1] + 0.0722 * v[2]);
    exato::sqrt(1.0 - (1.0 - t * t) * (1.0 - passa))
}

/// ⭐ **A transparência À LUZ da tinta que tapa o papel NU `(cx, cy)`** (W3b, `oraculo/README.md` §Transmission):
/// marcha para a luz como [`luz_directa`] e lê a tinta da célula mais alta no caminho — a Transparency e a
/// Transmission DELA (só da célula: o cartão não as tem) — pela [`transparencia_a_luz`]. Sem o campo do material:
/// `0` (a tinta opaca de sempre, ao bit).
#[must_use]
pub fn transparencia_de_quem_tapa(o: &OleoReal, campo: &CampoAltura, cx: usize, cy: usize) -> f64 {
    if campo.material.is_empty() {
        return 0.0;
    }
    let (x, y) = (cx as f64, cy as f64);
    let l = para_a_luz(o);
    let lxy = libm::hypot(l[0], l[1]).max(1e-9);
    let (dx, dy) = (l[0] / lxy, l[1] / lxy);
    // O CORPO da tinta que tapa: a célula mais alta no caminho da luz (a orla parcial à frente dela mistura o
    // material com o papel pela cobertura).
    let (mut alto, mut quem) = (f64::NEG_INFINITY, (cx as isize, cy as isize));
    for t in 1..=PASSOS_SOMBRA {
        let t = t as f64;
        let q = (
            exato::round(x + dx * t) as isize,
            exato::round(y + dy * t) as isize,
        );
        let h = campo.at_sombra(q.0, q.1);
        if h > alto {
            alto = h;
            quem = q;
        }
    }
    let m = campo.material_em(quem.0.max(0) as usize, quem.1.max(0) as usize);
    transparencia_a_luz(
        crate::material::peso(m.transparencia),
        crate::material::peso(m.transmissao),
        albedo_em(campo, quem.0, quem.1),
        crate::material::peso(m.vidro_claro),
    )
}

/// O albedo LINEAR da tinta da célula `(x, y)` guardado no campo (`1` sem o plano — sem rebote não se guarda).
#[must_use]
pub fn albedo_em(campo: &CampoAltura, x: isize, y: isize) -> [f64; 3] {
    if campo.albedo.is_empty() {
        [1.0; 3]
    } else {
        campo.albedo[campo.idx(x, y)].map(f64::from)
    }
}

/// W3b: quanto do DENTE do papel se vê ATRAVÉS da tinta da célula `(cx, cy)` (`0..1`): a Transmission `w` da CÉLULA
/// (o papel iluminado sob o filme). `0` na tinta de sempre — o dente fica enterrado (§BUG-papel). O Relevo do papel
/// não passa por aqui: a SUPERFÍCIE da tinta segue o dente na altura fina do campo (luz e brilho do óleo).
#[must_use]
pub fn grao_do_papel(_o: &OleoReal, campo: &CampoAltura, cx: usize, cy: usize) -> f64 {
    crate::material::peso(campo.material_em(cx, cy).transmissao)
}

/// Quantos níveis (`u8`) uma célula do plano da sombra tem de ficar ACIMA da mediana das 3×3 para ser um furo
/// (`oraculo/README.md` §BUG-bordas-W3b).
pub const FURO_DA_SOMBRA: u8 = 32;

#[cfg(test)]
thread_local! {
    /// (Testes) o [`FURO_DA_SOMBRA`] de uma variante medida no MESMO processo; `255` desliga a lei.
    pub(crate) static FURO_DA_MEDIDA: core::cell::Cell<Option<u8>> = const { core::cell::Cell::new(None) };
}

/// ⭐ §BUG-bordas-W3b: **a sombra no papel não tem furos de uma célula.** O plano da sombra (`gw × gh`, `0..255` de
/// escurecimento) lido na célula `(x, y)`: a célula [`FURO_DA_SOMBRA`] níveis (ou mais) mais escura que a MEDIANA
/// das 3×3 lê a mediana. O papel num vão de uma ou duas células ao pé da tinta (a baía da orla rasgada, a orla
/// parcial cortada) via o céu tapado em três lados e a luz tapada: 0,8–0,88 de sombra entre vizinhas a 0,1–0,4 — o
/// pontinho escuro solto. A faixa da sombra projectada (contínua) e a sombra larga não mudam: a mediana de um campo
/// liso é ele próprio.
#[must_use]
pub fn sombra_sem_furo(plano: &[u8], (gw, gh): (usize, usize), (x, y): (usize, usize)) -> u8 {
    let s = plano[x + y * gw];
    #[cfg(test)]
    let furo = FURO_DA_MEDIDA
        .with(core::cell::Cell::get)
        .unwrap_or(FURO_DA_SOMBRA);
    #[cfg(not(test))]
    let furo = FURO_DA_SOMBRA;
    if s < furo {
        return s;
    }
    let mut v = [0u8; 9];
    for (k, e) in v.iter_mut().enumerate() {
        let (xx, yy) = (
            (x + k % 3).saturating_sub(1).min(gw - 1),
            (y + k / 3).saturating_sub(1).min(gh - 1),
        );
        *e = plano[xx + yy * gw];
    }
    v.sort_unstable();
    // Só a célula MAIS ESCURA que a mediana (C1 da auditoria W3b: `s − mediana` com `s` mais clara estourava em dev e,
    // em smoke/release, dava a volta e ESCURECIA a célula clara até à mediana).
    if s.saturating_sub(v[4]) >= furo {
        v[4]
    } else {
        s
    }
}

/// A sombra no papel em `u8` (o plano que o composite guarda): a MESMA conversão nos dois caminhos. `s` é uma
/// fracção de LUZ (linear); o composite escurece o byte sRGB por ela, por isso guarda-se a fracção do byte que
/// tira essa luz: `1 − sRGB(1 − s)` (exacta no papel branco). Guardar `s` no byte era tirar ~2,2× a luz — o halo
/// cinza à volta de todo traço (K4, `oraculo/README.md` §K4).
#[must_use]
pub fn sombra_em_u8(s: f64) -> u8 {
    (255.0 - srgb255_of_linear(1.0 - s.clamp(0.0, 1.0))).round() as u8
}
