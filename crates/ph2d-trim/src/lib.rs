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

/// ⭐⭐⭐ **Quão fina é a malha do prisma — que é a malha que a FACE CORTADA vai
/// ter.**
///
/// # Porque isto existe (report do dono, 2026-09-15, com foto)
///
/// *«o remesh da face que você cortou fica ruim demais»*. Medido: a tampa de um
/// corte numa peça de `10 000` triângulos saía com **DOIS** triângulos —
/// `1 385×` menos do que a densidade da própria peça — e os cantos dela herdam
/// a normal da esfera, logo uma face **plana** era sombreada como se fosse
/// curva. *Não era a triangulação que estava torta: era a face a não ter malha
/// nenhuma.*
///
/// ⭐ **A cura não é um pós-passe sobre o resultado, é a lâmina:** o que o motor
/// de booleana devolve na superfície de corte é a tesselação da **PAREDE DO
/// PRISMA** recortada pela peça. ⇒ *um prisma tesselado dá um corte tesselado*,
/// e a propriedade que decide a arquitectura — longe do corte, nem um bit — fica
/// intacta **por construção**, porque nada toca a malha da peça.
///
/// # ⚠️ O preço, MEDIDO (release, esfera, `Op::Subtrair`)
///
/// | peça | lâmina mínima | lâmina à aresta da peça | tampa |
/// |---|---|---|---|
/// | `10 000` T | `15,9 ms` | **`23,5 ms`** | `2` → `2 450` |
/// | `50 176` T | `85,3 ms` | **`128,7 ms`** | `2` → `12 482` |
/// | `199 809` T | `413,8 ms` | **`545,0 ms`** | `2` → `49 928` |
///
/// ⇒ **o custo é da PEÇA, não da lâmina** (`+32 %` a `+51 %`), e por isso não há
/// aqui tecto de qualidade nenhum a inventar: a contagem da lâmina é
/// `perímetro/alvo × profundidade/alvo`, isto é, ela escala com a peça sozinha.
/// O único tecto que existe ([`TECTO_DE_TRIANGULOS`]) é contra um `alvo`
/// absurdo vindo de quem chama, e é renormalização — nunca uma recusa.
#[derive(Clone, Copy, Debug)]
pub enum Resolucao {
    /// Duas faces por parede: o prisma mínimo que encerra volume.
    ///
    /// ⚠️ É a saída desta lei até 2026-09-15, e fica por ser a **rota de
    /// bissecção** — e porque os gates de contagem da espec §7.1 falam dela.
    Minima,
    /// Nenhuma aresta das **PAREDES** acima deste comprimento, em unidades de
    /// cena.
    ///
    /// ⚠️ **Subdividir não move a superfície um bit:** as paredes são regradas e
    /// as tampas planas, então os pontos novos saem de interpolação **na própria
    /// superfície**. Há gate a comparar o volume das duas resoluções.
    ///
    /// # ⛔ As TAMPAS: a fronteira sim, o interior não — e porquê
    ///
    /// O anel adensado é **obrigatório** na tampa (senão nasce uma junta em T e
    /// o motor lê a lâmina como **ABERTA**), mas o **interior** dela fica com a
    /// triangulação grossa mais um leque, logo pode ter aresta bem acima do
    /// alvo — medido, a diagonal de uma tampa de `1 × 1` com alvo `0,4` mede
    /// `1,414`.
    ///
    /// ⭐ **E isso não toca o produto**, porque no regime que shipa
    /// ([`Profundidade::DaPeca`]) as tampas ficam **FORA da peça** por
    /// construção — o enchimento da [`faixa`] afasta-as —, logo elas nunca
    /// aparecem na superfície cortada. ⚠️ **Dívida NOMEADA para o dia em que a
    /// [`Profundidade::DoCursor`] chegar à interface:** ali a tampa **é** a face
    /// do corte, e um bolso sairia com o interior grosso. A cura é triangular a
    /// tampa com pontos interiores, que é trabalho próprio.
    Ate(f32),
}

/// O tecto de triângulos da lâmina — e **de que recurso ele é**: o relógio do
/// motor de booleana, medido nesta casa.
///
/// | triângulos da lâmina | corte numa peça de `10 000` T |
/// |---|---|
/// | `14 700` | `20,1 ms` |
/// | `58 800` | `41,3 ms` |
/// | `231 852` | `132,4 ms` |
/// | `927 408` | `523,6 ms` |
///
/// ⇒ acima daqui a lâmina passa a ser o custo em vez da peça. ⚠️ **O caminho do
/// produto nunca lá chega** (o `alvo` vem da peça, logo a lâmina escala com
/// ela): isto é a rede contra um `alvo` degenerado de quem chama, e ela
/// **renormaliza o alvo para cima**, nunca recusa o gesto.
pub const TECTO_DE_TRIANGULOS: usize = 200_000;

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
    resolucao: Resolucao,
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

    // ── OS DOIS ANÉIS, como a espec §7.1 os descreve ───────────────────────
    let mut frente_anel = Vec::with_capacity(n);
    for r in raios {
        frente_anel.push(pousa(r, plano.origem, eixo, frente)?);
    }
    let mut tras_anel = Vec::with_capacity(n);
    for (i, r) in raios.iter().enumerate() {
        tras_anel.push(match paredes {
            Paredes::Projectadas => pousa(r, plano.origem, eixo, tras)?,
            // ⚠️ O vértice DA FRENTE deslocado pela normal — é isto que torna
            // este modo independente da vista.
            Paredes::Fixas => {
                let d = tras - frente;
                let f = frente_anel[i];
                [f[0] + eixo[0] * d, f[1] + eixo[1] * d, f[2] + eixo[2] * d]
            }
        });
    }

    // ── A RESOLUÇÃO (ver [`Resolucao`]) ────────────────────────────────────
    let alvo = alvo_que_cabe(&frente_anel, &tras_anel, resolucao);
    let (frente_d, tras_d, denso_de) = adensa_o_anel(&frente_anel, &tras_anel, alvo);
    let filas = filas_em_profundidade(&frente_anel, &tras_anel, alvo);
    let mm = u32::try_from(frente_d.len()).map_err(|_| Recusa::GestoDegenerado)?;

    // A grelha: uma fila de anel por degrau de profundidade. ⚠️ Os pontos novos
    // são interpolações **na própria superfície** — a parede é regrada e a tampa
    // é plana —, então adensar não move a forma.
    let mut pos = Vec::with_capacity(frente_d.len() * (filas + 1));
    for r in 0..=filas {
        let t = r as f32 / filas as f32;
        for k in 0..frente_d.len() {
            pos.push(lerp(frente_d[k], tras_d[k], t));
        }
    }
    let ultima = u32::try_from(filas).map_err(|_| Recusa::GestoDegenerado)? * mm;

    let mut faces = Vec::with_capacity(2 * (n - 2) + 2 * frente_d.len() * filas);
    // As duas tampas partilham a MESMA lista de triângulos (espec §7.3), com
    // enrolamento oposto.
    for t in &tris_2d {
        tampa(&mut faces, &mut pos, t, &denso_de, n, 0, false);
        tampa(&mut faces, &mut pos, t, &denso_de, n, ultima, true);
    }
    for r in 0..filas {
        let (base, topo) = (
            u32::try_from(r).map_err(|_| Recusa::GestoDegenerado)? * mm,
            u32::try_from(r + 1).map_err(|_| Recusa::GestoDegenerado)? * mm,
        );
        for k in 0..mm {
            let l = (k + 1) % mm;
            faces.push(Face::tri(base + k, base + l, topo + l));
            faces.push(Face::tri(base + k, topo + l, topo + k));
        }
    }

    let mut m = Mesh::from_parts(pos, faces).map_err(|_| Recusa::GestoDegenerado)?;
    if volume_com_sinal(&m) < 0.0 {
        inverte(&mut m);
    }
    Ok(m)
}

/// O `alvo` de aresta que **cabe** — ver [`TECTO_DE_TRIANGULOS`].
///
/// ⚠️ Ele sobe, nunca desce: renormalizar para cima entrega uma lâmina mais
/// grossa; recusar entregaria um gesto perdido.
fn alvo_que_cabe(frente: &[[f32; 3]], tras: &[[f32; 3]], r: Resolucao) -> Option<f32> {
    let mut a = match r {
        Resolucao::Minima => return None,
        Resolucao::Ate(a) if a > 0.0 && a.is_finite() => a,
        // ⚠️ Um alvo não-positivo ou não-finito **não é uma recusa**: é o pedido
        // de nada, e a lâmina mínima é exactamente isso.
        Resolucao::Ate(_) => return None,
    };
    for _ in 0..16 {
        let t = triangulos_previstos(frente, tras, a);
        if t <= TECTO_DE_TRIANGULOS as f64 {
            break;
        }
        // A contagem é quadrática em `1/a`, logo a raiz é o passo exacto; o piso
        // de `1,05` é o que garante que o laço termina se a previsão saturar.
        a *= ((t / TECTO_DE_TRIANGULOS as f64).sqrt() as f32).max(1.05);
    }
    Some(a)
}

fn triangulos_previstos(frente: &[[f32; 3]], tras: &[[f32; 3]], a: f32) -> f64 {
    let n = frente.len();
    let m: f64 = (0..n)
        .map(|i| f64::from(pedacos(frente, tras, i, Some(a)) as u32))
        .sum();
    m * f64::from(filas_em_profundidade(frente, tras, Some(a)) as u32) * 2.0
}

/// Em quantos pedaços o segmento `i → i+1` do anel se parte.
///
/// ⚠️ **Mede-se nos DOIS anéis e fica o maior:** em [`Paredes::Projectadas`] a
/// parede é um trapézio, e medir só a frente deixaria a aresta de trás acima do
/// alvo.
fn pedacos(frente: &[[f32; 3]], tras: &[[f32; 3]], i: usize, alvo: Option<f32>) -> usize {
    let Some(a) = alvo else { return 1 };
    let j = (i + 1) % frente.len();
    let d = dist3(frente[i], frente[j]).max(dist3(tras[i], tras[j]));
    ((d / a).ceil()).clamp(1.0, 1e9) as usize
}

/// Quantos degraus em profundidade — o mesmo número para todas as paredes, que é
/// o que impede uma junta em T entre paredes vizinhas.
fn filas_em_profundidade(frente: &[[f32; 3]], tras: &[[f32; 3]], alvo: Option<f32>) -> usize {
    let Some(a) = alvo else { return 1 };
    let d = frente
        .iter()
        .zip(tras)
        .map(|(f, t)| dist3(*f, *t))
        .fold(0.0f32, f32::max);
    ((d / a).ceil()).clamp(1.0, 1e9) as usize
}

/// O anel grosso adensado. Devolve `(frente, trás, onde cada canto GROSSO ficou)`
/// — a terceira lista tem `n + 1` entradas, e a última é o comprimento, para que
/// os pontos do último segmento se leiam sem caso especial de dar a volta.
fn adensa_o_anel(
    frente: &[[f32; 3]],
    tras: &[[f32; 3]],
    alvo: Option<f32>,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let n = frente.len();
    let mut fd = Vec::with_capacity(n);
    let mut td = Vec::with_capacity(n);
    let mut onde = Vec::with_capacity(n + 1);
    for i in 0..n {
        let j = (i + 1) % n;
        onde.push(fd.len() as u32);
        fd.push(frente[i]);
        td.push(tras[i]);
        let k = pedacos(frente, tras, i, alvo);
        for s in 1..k {
            let t = s as f32 / k as f32;
            fd.push(lerp(frente[i], frente[j], t));
            td.push(lerp(tras[i], tras[j], t));
        }
    }
    onde.push(fd.len() as u32);
    (fd, td, onde)
}

/// Uma tampa, a partir de um triângulo do anel GROSSO.
///
/// ⛔⛔ **Um ponto que o adensamento pôs numa aresta do anel NÃO é opcional para
/// a tampa.** Ele pertence às paredes; se a tampa continuasse a ir de canto a
/// canto, a malha ficava com uma **junta em T** — e o motor lê uma junta em T
/// como superfície **ABERTA**. Medido nesta casa com a primeira sonda deste
/// trabalho: seis grelhas sem vértices partilhados devolveram `LaminaAberta`,
/// e uma lâmina aberta não corta nada.
///
/// ⚠️ **O leque parte do CENTRO do laço, e não de um canto:** os pontos de uma
/// aresta subdividida são colineares com os cantos dela, logo um leque a partir
/// de um canto emitiria triângulos de área **zero**. O centro de um laço de
/// fronteira de um triângulo está estritamente dentro dele.
fn tampa(
    faces: &mut Vec<Face>,
    pos: &mut Vec<[f32; 3]>,
    tri: &[u32; 3],
    denso_de: &[u32],
    n: usize,
    fila: u32,
    invertida: bool,
) {
    let mut laco: Vec<u32> = Vec::with_capacity(3);
    for e in 0..3 {
        let u = tri[e] as usize;
        let v = tri[(e + 1) % 3] as usize;
        laco.push(fila + denso_de[u]);
        // ⚠️ Só uma aresta DO ANEL tem pontos; uma diagonal interior da
        // triangulação não foi tocada pelo adensamento.
        if v == (u + 1) % n {
            for x in (denso_de[u] + 1)..denso_de[u + 1] {
                laco.push(fila + x);
            }
        }
    }
    let vira = |a: u32, b: u32, c: u32| {
        if invertida {
            Face::tri(a, b, c)
        } else {
            Face::tri(a, c, b)
        }
    };
    if laco.len() == 3 {
        faces.push(vira(laco[0], laco[1], laco[2]));
        return;
    }
    let centro = {
        let mut c = [0.0f32; 3];
        for &i in &laco {
            let p = pos[i as usize];
            c = [c[0] + p[0], c[1] + p[1], c[2] + p[2]];
        }
        let k = laco.len() as f32;
        [c[0] / k, c[1] / k, c[2] / k]
    };
    pos.push(centro);
    let ci = (pos.len() - 1) as u32;
    for i in 0..laco.len() {
        faces.push(vira(ci, laco[i], laco[(i + 1) % laco.len()]));
    }
}

fn lerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
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
