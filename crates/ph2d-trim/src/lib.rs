//! **O desenho vira VOLUME** — a lei do corte, sem câmara e sem janela.
//!
//! O artista desenha uma forma sobre a peça; essa forma é varrida ao longo de um
//! eixo e vira um prisma fechado, que a [`ph2d_mesh_bool`] depois subtrai.
//!
//! Clean-room sob `docs/3D/cleanroom/SPEC_trim_gesture.md`, §4–§8.
//!
//! # ⚠️ O que esta crate recebe, e porquê
//!
//! Um **anel de pontos de ecrã** e **um raio por ponto**. Ela não desprojecta
//! nada: quem sabe fazê-lo é a câmara, e recebê-lo já feito é o que mantém a lei
//! pura — e testável contra um anel escrito à mão.
//!
//! [`ph2d_mesh_bool`]: ../ph2d_mesh_bool/index.html

use ph2d_mesh::{Face, Mesh, Ray};

/// O plano da forma (espec §5): de onde o varrimento parte e para onde aponta.
///
/// ⚠️ **As duas orientações do alvo diferem SÓ na normal** — a direcção da vista
/// invertida, ou a normal da superfície no ponto onde o gesto começou. Quem
/// escolhe é o chamador; para esta lei é um vector e mais nada.
#[derive(Clone, Copy, Debug)]
pub struct Plano {
    /// O ponto em mundo onde o gesto começou.
    pub origem: [f32; 3],
    /// O eixo do varrimento. **Não precisa de vir normalizado.**
    pub normal: [f32; 3],
}

/// Onde o volume começa e acaba ao longo do eixo (espec §6).
///
/// ⚠️ **Os dois regimes não são variações um do outro**, e a diferença é de onde
/// vem a distância: da PEÇA ou do CURSOR.
#[derive(Clone, Copy, Debug)]
pub enum Profundidade {
    /// §6.1 — a extensão da própria peça ao longo do eixo, com enchimento.
    ///
    /// ⇒ **o volume ATRAVESSA sempre a peça, por construção.**
    DaPeca,
    /// §6.2 — uma fatia centrada em `medio`, de meia-espessura `raio`.
    ///
    /// ⚠️ **É aqui que o volume pode NÃO atravessar** — e a resposta é que nada
    /// de especial acontece: sai um **bolso** em vez de um corte passante. ⛔ Não
    /// há enchimento neste regime: ele alteraria a profundidade que o cursor
    /// acabou de definir.
    DoCursor {
        /// A distância com sinal, ao plano, do centro da fatia.
        medio: f32,
        /// Meia-espessura, em unidades de cena.
        ///
        /// ⚠️ **CONDIÇÃO DE FRONTEIRA (espec §6.2):** ele só está definido
        /// quando o gesto começa **sobre a superfície**. Começando fora, o
        /// chamador tem de o derivar dos ajustes do pincel — ⛔ lê-lo como `0`
        /// dá um volume sem espessura, isto é, um corte que não corta.
        raio: f32,
    },
}

/// Como as paredes do prisma se comportam com a perspectiva (espec §7.2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Paredes {
    /// O anel de trás é o da frente deslocado pela normal ⇒ paredes
    /// **paralelas**, e o prisma é **independente da vista** por construção.
    Fixas,
    /// Cada ponto de ecrã é desprojectado **outra vez** à profundidade de trás
    /// ⇒ o prisma segue o cone de visão e sai **cónico** em perspectiva.
    ///
    /// ⚠️ Em vista **ortográfica** os dois modos dão a mesma saída — *um gate
    /// que os compare em ortográfica não afirma nada.*
    Projectadas,
}

/// Porque é que não há prisma.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Recusa {
    /// O anel tem menos de três pontos, ou área nula no ecrã.
    GestoDegenerado,
    /// Veio um raio por ponto? (defeito de chamada, não do artista)
    RaiosNaoBatem,
    /// Um raio é paralelo ao plano da forma — não há onde o pousar.
    RaioParaleloAoPlano,
    /// A espessura do volume é nula ou negativa.
    ///
    /// ⚠️ No regime do cursor é o que acontece com raio `0` — ver a condição de
    /// fronteira em [`Profundidade::DoCursor`].
    EspessuraNula,
}

impl Recusa {
    /// A frase que o artista lê — o facto, e depois a cura.
    #[must_use]
    pub fn porque(self) -> &'static str {
        match self {
            Self::GestoDegenerado => {
                "o desenho nao delimita area nenhuma -- repita o gesto abrindo a forma"
            }
            Self::RaiosNaoBatem | Self::RaioParaleloAoPlano => {
                "a camara e o desenho nao concordam -- rode a vista e repita"
            }
            Self::EspessuraNula => {
                "o volume do corte ficou sem espessura -- aumente o pincel, ou \
                 desligue a profundidade pelo cursor"
            }
        }
    }
}

/// **A porta:** o anel de ecrã varrido num prisma fechado.
///
/// # ⭐ O enrolamento do desenho é IRRELEVANTE, e é por construção
///
/// A espec (§8) descreve o alvo a reorientar as faces do prisma no fim. Aqui a
/// mesma propriedade sai mais barata: monta-se o prisma e, se o **volume com
/// sinal** der negativo, invertem-se todas as faces. ⇒ *um laço desenhado no
/// sentido horário e o mesmo laço no anti-horário dão a MESMA saída.*
///
/// ⚠️⚠️ **É a armadilha mais cara desta lei.** Sem isto, metade dos gestos
/// entrega ao solucionador um volume com o dentro e o fora trocados — e uma
/// diferença com o operando invertido **não falha**: ela devolve o
/// **complemento**, isto é, apaga tudo *menos* o que se queria apagar. O defeito
/// não é geometria visivelmente errada: é a ferramenta a fazer o contrário do
/// pedido, **de forma intermitente**, conforme o sentido do gesto.
pub fn prisma(
    anel: &[[f32; 2]],
    raios: &[Ray],
    plano: &Plano,
    peca: &Mesh,
    profundidade: Profundidade,
    paredes: Paredes,
) -> Result<Mesh, Recusa> {
    if anel.len() < 3 {
        return Err(Recusa::GestoDegenerado);
    }
    if raios.len() != anel.len() {
        return Err(Recusa::RaiosNaoBatem);
    }
    let n = anel.len();
    let eixo = normaliza(plano.normal).ok_or(Recusa::GestoDegenerado)?;

    // ⭐⭐⭐ **O ANEL é ordenado AQUI, ANTES de qualquer coisa ser varrida** —
    // e o «antes» é a lei inteira.
    //
    // ⛔⛔ **A 1.ª redação ordenava só a TRIANGULAÇÃO da tampa**, e o gate
    // `o_mesmo_c_desenhado_nos_dois_sentidos_da_a_mesma_saida_ao_bit` apanhou-a:
    // os vértices saíam **iguais** e os volumes **não** (`3,43` contra `1,14`).
    // A causa: as PAREDES são construídas do anel como ele veio (`i → i+1`), logo
    // num desenho ao contrário elas dão a volta no sentido oposto ao das tampas
    // ⇒ a malha fica **internamente incoerente**, e ⛔ *um sinal de volume
    // global não repara isso* — ele só vira uma malha que já é coerente.
    //
    // ⇒ ordenar o ANEL (e os raios em passo com ele) faz tampas e paredes
    // nascerem do mesmo sentido, que é a rota barata que a espec §8 **N** nomeia.
    let (anel, raios) = if area_com_sinal(anel) < 0.0 {
        let mut a = anel.to_vec();
        a.reverse();
        let mut r: Vec<Ray> = raios.to_vec();
        r.reverse();
        (std::borrow::Cow::Owned(a), std::borrow::Cow::Owned(r))
    } else {
        (
            std::borrow::Cow::Borrowed(anel),
            std::borrow::Cow::Borrowed(raios),
        )
    };
    let (anel, raios) = (anel.as_ref(), raios.as_ref());

    let tris_2d = triangula(anel).ok_or(Recusa::GestoDegenerado)?;

    let (frente, tras) = faixa(plano.origem, eixo, peca, profundidade)?;

    let mut pos = Vec::with_capacity(2 * n);
    for r in raios {
        pos.push(pousa(r, plano.origem, eixo, frente)?);
    }
    for (i, r) in raios.iter().enumerate() {
        pos.push(match paredes {
            Paredes::Projectadas => pousa(r, plano.origem, eixo, tras)?,
            // ⚠️ O vértice DA FRENTE deslocado pela normal — é isto que torna
            // este modo independente da vista.
            Paredes::Fixas => {
                let d = tras - frente;
                let f = pos[i];
                [f[0] + eixo[0] * d, f[1] + eixo[1] * d, f[2] + eixo[2] * d]
            }
        });
    }

    let mut faces = Vec::with_capacity(2 * (n - 2) + 2 * n);
    // As duas tampas partilham a MESMA lista de triângulos (espec §7.3), com
    // enrolamento oposto.
    let nn = u32::try_from(n).map_err(|_| Recusa::GestoDegenerado)?;
    for t in &tris_2d {
        faces.push(Face::tri(t[0], t[2], t[1]));
        faces.push(Face::tri(t[0] + nn, t[1] + nn, t[2] + nn));
    }
    for i in 0..nn {
        let j = (i + 1) % nn;
        faces.push(Face::tri(i, j, j + nn));
        faces.push(Face::tri(i, j + nn, i + nn));
    }

    let mut m = Mesh::from_parts(pos, faces).map_err(|_| Recusa::GestoDegenerado)?;
    if volume_com_sinal(&m) < 0.0 {
        inverte(&mut m);
    }
    Ok(m)
}

/// A faixa `(frente, trás)` ao longo do eixo (espec §6).
fn faixa(
    origem: [f32; 3],
    eixo: [f32; 3],
    peca: &Mesh,
    p: Profundidade,
) -> Result<(f32, f32), Recusa> {
    let (frente, tras) = match p {
        Profundidade::DoCursor { medio, raio } => (medio - raio, medio + raio),
        Profundidade::DaPeca => {
            let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
            for v in peca.positions() {
                let d = distancia_com_sinal(*v, origem, eixo);
                lo = lo.min(d);
                hi = hi.max(d);
            }
            if !lo.is_finite() || !hi.is_finite() {
                return Err(Recusa::EspessuraNula);
            }
            // ⚠️ **Os DOIS termos são necessários** (espec §6.1): o relativo
            // escala com a peça, e o absoluto cobre a peça **degenerada** cuja
            // extensão ao longo do eixo é zero. A razão do enchimento é
            // numérica e não estética: afastar as tampas das faces da peça evita
            // faces **coplanares**, que é onde um solucionador exacto é mais
            // frágil.
            let pad = (hi - lo) * 0.01 + 0.001;
            (lo - pad, hi + pad)
        }
    };
    if tras - frente <= 0.0 {
        return Err(Recusa::EspessuraNula);
    }
    Ok((frente, tras))
}

/// Onde o raio encontra o plano a `d` do plano da forma.
fn pousa(r: &Ray, origem: [f32; 3], eixo: [f32; 3], d: f32) -> Result<[f32; 3], Recusa> {
    let denom = ponto(r.dir(), eixo);
    if denom.abs() < 1e-12 {
        return Err(Recusa::RaioParaleloAoPlano);
    }
    let alvo = [
        origem[0] + eixo[0] * d,
        origem[1] + eixo[1] * d,
        origem[2] + eixo[2] * d,
    ];
    let o = r.origin();
    let t = ponto([alvo[0] - o[0], alvo[1] - o[1], alvo[2] - o[2]], eixo) / denom;
    Ok(r.at(t))
}

fn distancia_com_sinal(v: [f32; 3], origem: [f32; 3], eixo: [f32; 3]) -> f32 {
    ponto([v[0] - origem[0], v[1] - origem[1], v[2] - origem[2]], eixo)
}

fn ponto(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normaliza(v: [f32; 3]) -> Option<[f32; 3]> {
    let n = ponto(v, v).sqrt();
    (n > 1e-20).then(|| [v[0] / n, v[1] / n, v[2] / n])
}

/// O volume com sinal de uma malha fechada de triângulos.
///
/// ⚠️ É a régua que decide o enrolamento global — e ela é a mesma grandeza que a
/// espec usa para comparar cortes.
fn volume_com_sinal(m: &Mesh) -> f32 {
    let p = m.positions();
    let mut tris = Vec::new();
    let mut v = 0.0f64;
    for f in m.faces() {
        tris.clear();
        f.triangles(&mut tris);
        for t in &tris {
            let (a, b, c) = (p[t[0] as usize], p[t[1] as usize], p[t[2] as usize]);
            let cr = [
                f64::from(b[1]) * f64::from(c[2]) - f64::from(b[2]) * f64::from(c[1]),
                f64::from(b[2]) * f64::from(c[0]) - f64::from(b[0]) * f64::from(c[2]),
                f64::from(b[0]) * f64::from(c[1]) - f64::from(b[1]) * f64::from(c[0]),
            ];
            v += f64::from(a[0]) * cr[0] + f64::from(a[1]) * cr[1] + f64::from(a[2]) * cr[2];
        }
    }
    (v / 6.0) as f32
}

fn inverte(m: &mut Mesh) {
    let faces: Vec<Face> = m
        .faces()
        .iter()
        .map(|f| {
            let v = f.verts();
            Face::tri(v[0], v[2], v[1])
        })
        .collect();
    *m = Mesh::from_parts(m.positions().to_vec(), faces).expect("inverter não muda os índices");
}

/// **A tampa é triangulada no ECRÃ, não em 3D** (espec §7.3).
///
/// ⚠️ A razão: no ecrã o anel é, por construção, um polígono simples e **plano**;
/// em 3D ele pode não ser plano nenhum — uma vista em perspectiva com
/// [`Paredes::Projectadas`] põe os pontos a profundidades diferentes.
/// ⇒ um laço **CÔNCAVO** (um C) é tampado correctamente, e a mesma lista serve
/// às duas tampas.
fn triangula(anel: &[[f32; 2]]) -> Option<Vec<[u32; 3]>> {
    let n = anel.len();
    let area = area_com_sinal(anel);
    if area.abs() < 1e-12 {
        return None;
    }
    // ⚠️ Trabalha-se sempre no sentido positivo; o enrolamento do desenho é
    // resolvido aqui e o global pelo volume com sinal.
    let mut idx: Vec<u32> = (0..n as u32).collect();
    if area < 0.0 {
        idx.reverse();
    }
    let mut out = Vec::with_capacity(n - 2);
    let mut guarda = 0;
    while idx.len() > 3 {
        let antes = idx.len();
        for k in 0..idx.len() {
            let (i0, i1, i2) = (
                idx[(k + idx.len() - 1) % idx.len()],
                idx[k],
                idx[(k + 1) % idx.len()],
            );
            if orelha(anel, &idx, i0, i1, i2) {
                out.push([i0, i1, i2]);
                idx.remove(k);
                break;
            }
        }
        if idx.len() == antes {
            // ⛔ Nenhuma orelha: o anel não é um polígono simples (auto-cruza).
            return None;
        }
        guarda += 1;
        if guarda > n + 2 {
            return None;
        }
    }
    out.push([idx[0], idx[1], idx[2]]);
    Some(out)
}

fn orelha(anel: &[[f32; 2]], idx: &[u32], a: u32, b: u32, c: u32) -> bool {
    let (pa, pb, pc) = (anel[a as usize], anel[b as usize], anel[c as usize]);
    if cruz(pa, pb, pc) <= 0.0 {
        return false;
    }
    !idx.iter()
        .any(|&i| i != a && i != b && i != c && dentro(anel[i as usize], pa, pb, pc))
}

fn cruz(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

fn dentro(p: [f32; 2], a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> bool {
    cruz(a, b, p) >= 0.0 && cruz(b, c, p) >= 0.0 && cruz(c, a, p) >= 0.0
}

fn area_com_sinal(anel: &[[f32; 2]]) -> f32 {
    let n = anel.len();
    let mut s = 0.0;
    for i in 0..n {
        let (a, b) = (anel[i], anel[(i + 1) % n]);
        s += a[0] * b[1] - b[0] * a[1];
    }
    s * 0.5
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
