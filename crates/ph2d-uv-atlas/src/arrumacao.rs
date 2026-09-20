//! ⭐⭐ **A ARRUMAÇÃO** — orientar cada peça, medir a caixa dela e pô-la no quadrado.
//!
//! ⚠️ Esta folha saiu do [`crate`] por **TECTO DE LOC** (`818` contra `700`), e o corte é
//! por RESPONSABILIDADE e não pelo tamanho: o que fica lá é o **pipeline** (juntar ·
//! assentar · cortar) e o que vem para aqui é **o que se faz com as peças depois de elas
//! existirem**. *Uma isenção teria deixado as duas coisas no mesmo ficheiro para sempre.*
//!
//! A lei do empacotador vive na [`crate::empacota`]; aqui estão a ponte com a malha, a
//! rede das prateleiras e a escolha do lado do quadrado.

use crate::{TEXTURA_DE_REFERENCIA, VAO_EM_TEXELS, empacota, orienta};
use ph2d_mesh::Mesh;

/// ⚠️ **A resolução da grelha de arrumação, em células por lado — MEDIDA, e o joelho é
/// nítido.**
///
/// | células | tinta/quadrado (CRUA) | relógio | tinta (F1) | relógio |
/// |---|---|---|---|---|
/// | `128` | `19,9 %` | `121 ms` | `31,8 %` | `22 ms` |
/// | **`256`** | **`29,6 %`** | `151 ms` | **`38,7 %`** | `46 ms` |
/// | `512` | `30,2 %` | `287 ms` | `39,7 %` | `125 ms` |
///
/// ⭐ `128 → 256` compra **`9,7` pontos**; `256 → 512` compra **`0,6`** por **`1,9×`** o
/// relógio. ⛔ O recurso tem nome: o empacotador custa `O(peças × lado × máscara)`, e a
/// grelha grossa perde de outra maneira — *uma peça mais fina que uma célula deixa de ter
/// forma nenhuma, e a folga de uma célula à volta dela passa a valer mais que ela*.
const CELULAS_DA_ARRUMACAO: usize = 256;

/// Quantas vezes a bissecção do lado do quadrado corre. Ver [`empacota_por_mascara`].
const PASSOS_DA_BISSECCAO: usize = 5;

/// A área somada dos triângulos, no plano — a TINTA.
pub fn tinta_do_plano(mesh: &Mesh, base: &[u32], plano: &[[f32; 2]], peca_da_face: &[u32]) -> f32 {
    let mut soma = 0.0f64;
    for (f, face) in mesh.faces().iter().enumerate() {
        if peca_da_face[f] == u32::MAX {
            continue;
        }
        let (n, b) = (face.verts().len(), base[f] as usize);
        for k in 1..n.saturating_sub(1) {
            let (a, c, d) = (plano[b], plano[b + k], plano[b + k + 1]);
            let (ux, uy) = (f64::from(c[0] - a[0]), f64::from(c[1] - a[1]));
            let (vx, vy) = (f64::from(d[0] - a[0]), f64::from(d[1] - a[1]));
            soma += (ux.mul_add(vy, -(uy * vx)) * 0.5).abs();
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    {
        soma as f32
    }
}

/// ⭐⭐⭐ **Arruma pela FORMA.** Ver [`empacota`]. Devolve o mesmo que [`por_prateleiras`].
///
/// ⚠️ **O lado do quadrado acha-se em duas fases, e a segunda não é acabamento:** a
/// primeira cresce `8 %` de cada vez até caber, logo ela pára até `8 %` acima do
/// necessário — e o lado entra na conta ao QUADRADO, o que são `16 %` de área.
/// A segunda **bissecta** entre o último que não coube e o primeiro que coube.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn por_mascara(
    mesh: &Mesh,
    base: &[u32],
    plano: &[[f32; 2]],
    peca_da_face: &[u32],
    lo: &[[f32; 2]],
    hi: &[[f32; 2]],
    tinta: f32,
) -> Option<(Vec<[f32; 2]>, f32, f32)> {
    let n = lo.len();
    let medida = |i: usize| {
        if lo[i][0] <= hi[i][0] {
            (hi[i][0] - lo[i][0], hi[i][1] - lo[i][1])
        } else {
            (0.0, 0.0)
        }
    };
    let caixas: f32 = (0..n).map(|i| medida(i).0 * medida(i).1).sum();
    let fraccao = VAO_EM_TEXELS / TEXTURA_DE_REFERENCIA;

    let tenta = |lado: f32| -> Option<Vec<[f32; 2]>> {
        let celula = lado / CELULAS_DA_ARRUMACAO as f32;
        if celula <= 0.0 || !celula.is_finite() {
            return None;
        }
        let folga = ((lado * fraccao) / celula).ceil().max(1.0) as usize;
        let mut masc: Vec<empacota::Mascara> = (0..n)
            .map(|i| {
                let (w, h) = medida(i);
                let larg = ((w / celula).ceil() as usize + 1).max(1);
                let alt = ((h / celula).ceil() as usize + 1).max(1);
                empacota::Mascara {
                    larg,
                    alt,
                    celulas: vec![false; larg * alt],
                }
            })
            .collect();
        for (f, face) in mesh.faces().iter().enumerate() {
            let pi = peca_da_face[f];
            if pi == u32::MAX {
                continue;
            }
            let (pi, b, k) = (pi as usize, base[f] as usize, face.verts().len());
            let o = lo[pi];
            let p = |c: usize| [plano[c][0] - o[0], plano[c][1] - o[1]];
            for j in 1..k.saturating_sub(1) {
                empacota::marca_triangulo(&mut masc[pi], [p(b), p(b + j), p(b + j + 1)], celula);
            }
        }
        // ⛔ **O piso de população:** uma peça sem uma célula marcada é RECUSADA pelo
        // empacotador (ele não sabe arrumar o que não ocupa nada). Ela existe — uma lasca
        // mais fina que uma célula — e a resposta é marcar-lhe UMA célula.
        for m in &mut masc {
            if m.ocupadas() == 0 {
                m.celulas[0] = true;
            }
        }
        let cel = empacota::arruma(&masc, CELULAS_DA_ARRUMACAO, folga)?;
        Some(
            cel.iter()
                .map(|&(cx, cy)| [cx as f32 * celula, cy as f32 * celula])
                .collect(),
        )
    };

    // Fase 1: crescer até caber. O piso é o quadrado CHEIO, que é impossível por
    // construção (há folga de costura), logo ele serve de limite inferior da bissecção.
    let mut baixo = tinta.sqrt().max(1.0e-6);
    let mut lado = baixo;
    let mut melhor = None;
    for _ in 0..40 {
        if let Some(pos) = tenta(lado) {
            melhor = Some((lado, pos));
            break;
        }
        baixo = lado;
        lado *= 1.08;
    }
    let (mut alto, mut pos) = melhor?;
    // Fase 2: bissectar. ⚠️ `PASSOS_DA_BISSECCAO` é um tecto de RELÓGIO — cada passo é
    // uma arrumação inteira —, e `5` fecha a folga de `8 %` a menos de `0,3 %`.
    for _ in 0..PASSOS_DA_BISSECCAO {
        let meio = 0.5 * (baixo + alto);
        if let Some(p) = tenta(meio) {
            alto = meio;
            pos = p;
        } else {
            baixo = meio;
        }
    }
    Some((pos, alto, caixas))
}

/// Roda cada peça para o eixo da caixa mínima dela, no sítio.
pub fn orienta_as_pecas(
    mesh: &Mesh,
    base: &[u32],
    plano: &mut [[f32; 2]],
    peca_da_face: &[u32],
    pecas: usize,
) {
    let mut nuvem: Vec<Vec<[f32; 2]>> = vec![Vec::new(); pecas];
    for (f, face) in mesh.faces().iter().enumerate() {
        let pi = peca_da_face[f];
        if pi == u32::MAX {
            continue;
        }
        let b = base[f] as usize;
        for k in 0..face.verts().len() {
            nuvem[pi as usize].push(plano[b + k]);
        }
    }
    let eixos: Vec<[f32; 2]> = nuvem
        .iter()
        .map(|n| orienta::eixo_da_caixa_minima(n))
        .collect();
    for (f, face) in mesh.faces().iter().enumerate() {
        let pi = peca_da_face[f];
        if pi == u32::MAX {
            continue;
        }
        let e = eixos[pi as usize];
        let b = base[f] as usize;
        for k in 0..face.verts().len() {
            plano[b + k] = orienta::roda(plano[b + k], e);
        }
    }
}

/// A caixa de cada peça no plano da ilha dela.
pub fn caixas(
    mesh: &Mesh,
    base: &[u32],
    plano: &[[f32; 2]],
    peca_da_face: &[u32],
    pecas: usize,
) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let mut lo = vec![[f32::MAX; 2]; pecas];
    let mut hi = vec![[f32::MIN; 2]; pecas];
    for (f, face) in mesh.faces().iter().enumerate() {
        let pi = peca_da_face[f];
        if pi == u32::MAX {
            continue;
        }
        let (pi, b) = (pi as usize, base[f] as usize);
        for k in 0..face.verts().len() {
            let q = plano[b + k];
            lo[pi][0] = lo[pi][0].min(q[0]);
            lo[pi][1] = lo[pi][1].min(q[1]);
            hi[pi][0] = hi[pi][0].max(q[0]);
            hi[pi][1] = hi[pi][1].max(q[1]);
        }
    }
    (lo, hi)
}

/// ⭐ **O empacotador: prateleiras, as mais altas primeiro, com o vão do mip.**
///
/// Devolve `(canto de cada peça, lado do quadrado, área somada das caixas)`.
pub fn por_prateleiras(lo: &[[f32; 2]], hi: &[[f32; 2]]) -> (Vec<[f32; 2]>, f32, f32) {
    let n = lo.len();
    let fraccao = VAO_EM_TEXELS / TEXTURA_DE_REFERENCIA;
    let mut tam: Vec<(usize, f32, f32)> = (0..n)
        .map(|i| {
            let (w, h) = if lo[i][0] <= hi[i][0] {
                (hi[i][0] - lo[i][0], hi[i][1] - lo[i][1])
            } else {
                (0.0, 0.0)
            };
            (i, w, h)
        })
        .collect();
    tam.sort_by(|a, b| b.2.total_cmp(&a.2));
    let area: f32 = tam.iter().map(|t| t.1 * t.2).sum();
    let mut lado = area.sqrt().max(1.0e-6);
    let mut pos = vec![[0.0f32, 0.0]; n];
    // ⚠️ O laço CRESCE o quadrado até caber. *Um empacotador que devolve «não coube» a
    // quem lhe deu rectângulos não resolveu nada* — e o preço de crescer é medido pelo
    // aproveitamento, que é a coluna que o relatório publica.
    for _ in 0..200 {
        let vao = lado * fraccao;
        let (mut x, mut y, mut alt) = (vao, vao, 0.0f32);
        let mut coube = true;
        for &(i, w, h) in &tam {
            // ⛔⛔ **A 1.ª redacção não tinha esta linha, e o gate do quadrado apanhou-a:**
            // sem ela uma ilha mais LARGA que o quadrado era «colocada» na primeira
            // prateleira e o laço declarava que coube — a fixtura de uma ilha só (`2 × 1`
            // num quadrado de `1,41`) saía com `u = 1,42`. *Um empacotador que só verifica
            // a altura mede metade do problema.*
            if w + 2.0 * vao > lado || h + 2.0 * vao > lado {
                coube = false;
                break;
            }
            if x + w + vao > lado {
                x = vao;
                y += alt + vao;
                alt = 0.0;
            }
            if y + h + vao > lado {
                coube = false;
                break;
            }
            pos[i] = [x, y];
            x += w + vao;
            alt = alt.max(h);
        }
        if coube {
            break;
        }
        lado *= 1.05;
    }
    (pos, lado, area)
}

/// ⚠️⚠️ **O tecto do factor de igualação — VARRIDO, e o recurso é a ARRUMAÇÃO.**
///
/// Uma peça pode vir do parametrizador `20×` mais grossa do que devia, e ela é quase
/// sempre uma **lasca**: esticá-la até ao alvo multiplica a área dela por `400` e obriga
/// o quadrado inteiro a crescer. ⇒ o tecto não protege a lei, protege a TINTA.
///
/// Varrido pela porta do produto sobre a peça do dono (`sculpt_antes`), com a coluna que
/// decide a ser o **pior sítio em texels por unidade de mundo** (`p05 × p50`, a `2048²`)
/// — *é o sítio mais borrado que o artista encontra, e é ele que se lê como «o pincel
/// mudou de tamanho»*:
///
/// | tecto | `F1` tinta | `F1` pior | `F1` ENTRE | `CRUA` tinta | `CRUA` pior |
/// |---|---|---|---|---|---|
/// | desligado | `47,9 %` | `164` | `0,58×` | `42,6 %` | `254` |
/// | `1,5` | `48,5 %` | `217` | `0,89×` | `40,3 %` | `262` |
/// | `2,0` | `47,9 %` | `235` | `0,89×` | `40,5 %` | `263` |
/// | **`3,0`** | **`47,0 %`** | **`249`** | **`0,95×`** | **`40,5 %`** | **`263`** |
/// | `6,0` | `45,4 %` | `257` | `0,97×` | `40,5 %` | `263` |
/// | sem tecto | `41,6 %` | `252` | `0,97×` | `40,5 %` | `263` |
///
/// ⛔⛔ **A linha de baixo é DOMINADA, e é ela que faz o tecto existir:** sem tecto o pior
/// sítio fica **PIOR** que a `6,0` (`252` contra `257`) e paga `3,8` pontos de tinta a
/// mais — *esticar a lasca coarsa a peça inteira, incluindo o sítio que a lei estava a
/// tentar salvar*. Um tecto aqui não é conforto: é a única configuração em que mais lei
/// não deixa de comprar nada e passa a custar.
///
/// ⭐ `1,0 → 1,5` é **de graça** na malha que o botão produz (`+0,6` pontos de tinta) e
/// compra `+32 %` de pior sítio. O `3,0` leva `95 %` da cura entre peças e `97 %` do
/// melhor pior sítio; `6,0` compra os `3 %` que faltam por **o dobro** do preço.
///
/// ⚠️ **A malha CRUA mal discrimina** — dali para cima todas as linhas leem igual —, e é
/// por isso que a coluna que manda é a do `F1`: *é nela que o produto pinta*.
pub(crate) const TECTO_DA_IGUALACAO: f32 = 3.0;

/// A área de um triângulo no MUNDO.
fn area_no_mundo(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f64 {
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

/// ⭐⭐⭐ **IGUALA A DENSIDADE: cada peça passa a receber os mesmos texels por unidade de
/// SUPERFÍCIE.**
///
/// ⛔⛔ **O defeito que ela cura não tinha régua nenhuma até 2026-09-20.** O atlas era
/// medido por `tinta / quadrado`, por sobreposição e pelo vão — três colunas que dizem
/// quanto do quadrado é usado e nenhuma que diga se ele é usado **por igual**. Medido na
/// malha que o botão produz (`sculpt_antes` F1), `5 %` da superfície recebia **menos de
/// metade** dos texels da mediana: o artista pinta com o mesmo pincel e a marca sai
/// nítida num sítio e borrada noutro.
///
/// ⚠️ **A arrumação está ILIBADA e mesmo assim a cura mora aqui.** O empacotador só
/// TRANSLADA e a orientação só RODA — nenhum dos dois escala —, logo o espalhamento é
/// todo do parametrizador (o G3), que resolve cada carta contra o mesmo passo alvo e
/// acerta em cada uma com um factor diferente. *A montante seria um termo novo na
/// energia; aqui é um escalar por peça, e uma semelhança não pode desfazer o que o corte
/// garantiu.*
///
/// ⭐⭐ **O alvo PRESERVA a tinta total, exactamente — ENQUANTO o tecto não morder:**
/// com `d* = √(Σ_uv / Σ_mundo)`, a área nova de cada peça é `d*² × mundo_i` e a soma
/// delas é `d*² × Σ_mundo = Σ_uv`. *Qualquer diferença que a coluna da tinta mostre a
/// seguir é da ARRUMAÇÃO a lidar com formas de outro tamanho, nunca desta conta.*
///
/// ⚠️ **E a cláusula não é um detalhe: o [`TECTO_DA_IGUALACAO`] quebra-a de propósito.**
/// Uma peça travada no tecto fica com menos área do que o alvo lhe daria, logo a soma
/// encolhe — e é exactamente por isso que ele existe. *As duas metades têm gate, e a que
/// mede a preservação corre num regime onde o tecto NÃO morde: medir a promessa no
/// regime que a viola seria escrevê-la ao contrário.*
///
/// ⚠️⚠️ **A escala é em torno da ORIGEM do plano, e o ponto é INOBSERVÁVEL** — a
/// arrumação refaz as caixas a seguir e coloca cada peça pelo canto dela, logo onde a
/// peça está antes de ser arrumada não chega a ninguém. ⭐ A 1.ª redacção escalava em
/// torno do canto (`lo + (q − lo)·f`) e pagava **duas** coisas por isso: uma passagem a
/// mais para achar o canto, e uma cerca para o factor exactamente `1` — porque aquela
/// forma **não** devolve `q` ao bit. *Uma linha que a mutação não consegue matar é um
/// comentário com sintaxe de código*, e `q · 1,0` é `q` por definição.
///
/// Devolve `[menor factor, maior factor]` — o TAMANHO do defeito que ela acabou de curar.
pub fn iguala_a_densidade(
    mesh: &Mesh,
    base: &[u32],
    plano: &mut [[f32; 2]],
    peca_da_face: &[u32],
    pecas: usize,
) -> [f32; 2] {
    let pos = mesh.positions();
    let mut uv = vec![0.0f64; pecas];
    let mut mundo = vec![0.0f64; pecas];
    for (f, face) in mesh.faces().iter().enumerate() {
        let pi = peca_da_face[f];
        if pi == u32::MAX {
            continue;
        }
        let (pi, b, vs) = (pi as usize, base[f] as usize, face.verts());
        for k in 1..vs.len().saturating_sub(1) {
            let (a, c, d) = (plano[b], plano[b + k], plano[b + k + 1]);
            let (ux, uy) = (f64::from(c[0] - a[0]), f64::from(c[1] - a[1]));
            let (vx, vy) = (f64::from(d[0] - a[0]), f64::from(d[1] - a[1]));
            uv[pi] += (ux.mul_add(vy, -(uy * vx)) * 0.5).abs();
            mundo[pi] += area_no_mundo(
                pos[vs[0] as usize],
                pos[vs[k] as usize],
                pos[vs[k + 1] as usize],
            );
        }
    }
    let (su, sm): (f64, f64) = (uv.iter().sum(), mundo.iter().sum());
    if su <= 0.0 || sm <= 0.0 {
        return [1.0, 1.0];
    }
    let alvo = (su / sm).sqrt();
    let mut faixa = [f32::MAX, f32::MIN];
    #[allow(clippy::cast_possible_truncation)]
    let factor: Vec<f32> = (0..pecas)
        .map(|i| {
            // ⛔ **Uma peça sem área não tem cerca PRÓPRIA, e isso foi medido:** a
            // 1.ª redacção tinha um `if uv <= 0 || mundo <= 0` aqui, e a mutação que o
            // apagava **SOBREVIVEU** — os três casos degenerados (`0/x`, `x/0`, `0/0`)
            // saem `+∞`, `0` e `NaN`, e a cerca de baixo apanha os três. *Uma linha que
            // a mutação não consegue matar é um comentário com sintaxe de código.*
            let f = ((alvo / (uv[i] / mundo[i]).sqrt()) as f32)
                .clamp(1.0 / TECTO_DA_IGUALACAO, TECTO_DA_IGUALACAO);
            if f.is_finite() && f > 0.0 {
                faixa[0] = faixa[0].min(f);
                faixa[1] = faixa[1].max(f);
                f
            } else {
                1.0
            }
        })
        .collect();
    for (f, face) in mesh.faces().iter().enumerate() {
        let pi = peca_da_face[f];
        if pi == u32::MAX {
            continue;
        }
        let (pi, b) = (pi as usize, base[f] as usize);
        let e = factor[pi];
        for k in 0..face.verts().len() {
            let q = &mut plano[b + k];
            q[0] *= e;
            q[1] *= e;
        }
    }
    if faixa[0] > faixa[1] {
        return [1.0, 1.0];
    }
    faixa
}
