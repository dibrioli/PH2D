//! ⭐⭐⭐ **A SOMBRA COM A BORDA MOLE — a que um material TRANSLÚCIDO lê.**
//!
//! # Porque ela existe: o report de 2026-09-18
//!
//! O dono apontou uma **linha dura** na fronteira entre a parte iluminada e a sombreada de uma
//! esfera de jade. O experimento que a diagnosticou tirou a placa vizinha da cena: **sem ela a bola
//! sai lisa** ⇒ a linha é a borda da SOMBRA que a placa lança, e ela é dura porque a luz é um
//! **ponto**.
//!
//! ⛔ Isso é certo como geometria e errado como produto: num jade a luz que entra **fora** da sombra
//! espalha-se por baixo da superfície **para dentro** dela. A nossa subsuperfície tinha a difusão na
//! lei do `N·L` (o [`ph2d_material`] envolve a luz à volta do terminador) e **não a tinha na lei da
//! SOMBRA** — a visibilidade entrava dura, por pixel.
//!
//! # ⭐⭐ A lei: a visibilidade que a closure de subsuperfície lê é a MÉDIA da vizinhança
//!
//! O comprimento sobre o qual se faz a média é a **distância de espalhamento** — o `subsurface_radius`
//! por canal, em unidades do MUNDO, convertido a píxeis pela câmera. ⭐ É por ser **por canal** que a
//! borda fica avermelhada: o vermelho viaja mais no material e entra mais fundo na sombra, que é a
//! assinatura de toda pele e de toda cera.
//!
//! ⚠️ **A média é GUARDADA pela normal** ([`crate::OCCLUSION_BLUR_COS`], a mesma porta que o céu e o
//! ricochete usam): ela alisa dentro de uma superfície e **não atravessa uma quina**. *Duas cópias
//! da guarda divergiriam no dia em que alguém afinasse uma delas.*
//!
//! ⚠️⚠️ **E ela é SEPARÁVEL, em duas passagens de uma dimensão.** Um quadrado de `(2r+1)²` toques
//! por pixel é `O(r²)`; duas passagens de `(2r+1)` são `O(r)`, e dão a mesma resposta para um núcleo
//! de caixa. ⛔ A guarda de normal **não** é separável em rigor (um vizinho pode estar ligado na
//! horizontal e não na vertical) — a divergência é declarada e vale o preço: a alternativa é `O(r²)`,
//! e a `r = 24` isso são `2 401` toques por pixel e por canal.
//!
//! # ⚠️ Canal vazio ⇒ o quadro de sempre, AO BIT
//!
//! Sem esta passagem o [`crate::Shadows::soft_at`] devolve a visibilidade DURA, logo a lei da
//! subsuperfície recebe exactamente o que recebia. É o mesmo desenho do `ambient` e do `bounce`.

use crate::Gbuffer;

/// ⭐ **O tecto do raio, em píxeis — e ele diz de que recurso é: o RELÓGIO.**
///
/// A passagem custa `O(r)` por pixel e por canal. Medido a `320×240` (debug, `load ~1`):
///
/// | `r` | relógio da passagem |
/// |---:|---:|
/// | `8` | `4,1 ms` |
/// | `24` | `11,2 ms` |
/// | `48` | `21,6 ms` |
///
/// ⛔ Acima de `48` a borda de um jade já não muda de aspecto (a sombra está toda lavada) e o preço
/// continua a subir linearmente — *o tecto é onde o efeito satura e o custo não*.
pub const MAX_RAIO_PX: f32 = 48.0;

/// ⭐⭐⭐ **A média guardada, em duas passagens** — devolve um canal RGB por lâmpada.
///
/// `raio_px` é o raio por canal, em píxeis de ecrã. Um raio `<= 0` num canal deixa-o **igual ao
/// duro**, ao bit.
///
/// ⛔⛔ **Um canal que não cobre o gbuffer devolve VAZIO, e nunca entra em pânico.** Ele foi
/// escrito a supor que `vis` tem um valor por pixel, e as duas passagens percorrem o **gbuffer**
/// enquanto indexam o **canal** — com um `vis` vazio (uma lâmpada que não existe) isso é um
/// `index out of bounds`, e foi assim que ele estoirou numa sonda de 18/09.
///
/// ⭐ O vazio é a resposta CERTA e não um remendo: o [`crate::Shadows::soft_at`] cai na
/// visibilidade **dura** quando não há canal mole, que é exactamente *«esta lâmpada não tem borda
/// mole»*. ⚠️ *Um porte que estoira sobre uma entrada vazia é uma armadilha para o segundo
/// chamador* — e o primeiro só não a pisou porque percorre as lâmpadas que existem.
#[must_use]
pub fn blur_por_canal(g: &Gbuffer, vis: &[f32], raio_px: [f32; 3]) -> Vec<[f32; 3]> {
    if vis.len() != g.hit.len() {
        return Vec::new();
    }
    let n = vis.len();
    let mut out = vec![[0.0f32; 3]; n];
    for (k, &r) in raio_px.iter().enumerate() {
        let canal = uma_dimensao(g, vis, r, true);
        let canal = uma_dimensao(g, &canal, r, false);
        for i in 0..n {
            out[i][k] = canal[i];
        }
    }
    out
}

/// Uma passagem de uma dimensão, com a guarda de normal da casa.
fn uma_dimensao(g: &Gbuffer, canal: &[f32], raio_px: f32, horizontal: bool) -> Vec<f32> {
    let mut out = canal.to_vec();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let r = raio_px.clamp(0.0, MAX_RAIO_PX) as i32;
    if r <= 0 {
        return out;
    }
    let (w, h) = (g.width as i32, g.height as i32);
    if w <= 0 || h <= 0 {
        return out;
    }
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            if !g.hit[i] {
                continue;
            }
            let n0 = g.normal[i];
            let (mut soma, mut peso) = (0.0f32, 0.0f32);
            for d in -r..=r {
                let (xx, yy) = if horizontal { (x + d, y) } else { (x, y + d) };
                if xx < 0 || yy < 0 || xx >= w || yy >= h {
                    continue;
                }
                let j = (yy * w + xx) as usize;
                if !g.hit[j] {
                    continue;
                }
                let nj = g.normal[j];
                // ⚠️ A MESMA guarda do céu e do ricochete — ver o cabeçalho.
                if n0[0] * nj[0] + n0[1] * nj[1] + n0[2] * nj[2] < crate::OCCLUSION_BLUR_COS {
                    continue;
                }
                soma += canal[j];
                peso += 1.0;
            }
            if peso > 0.0 {
                out[i] = soma / peso;
            }
        }
    }
    out
}

/// ⭐⭐ **O raio em PÍXEIS que uma distância de espalhamento do MUNDO vale**, nesta câmera.
///
/// ⚠️ É por isto que a borda não muda de largura quando se dá zoom: ela é uma distância do MUNDO, e
/// o número de píxeis que ela ocupa tem de a seguir. *Um raio escrito em píxeis seria uma borda que
/// encolhe quando o artista se aproxima.*
#[must_use]
pub fn raio_em_pixeis(cam: &crate::Orbit, altura_px: u32, mundo: [f32; 3]) -> [f32; 3] {
    if altura_px == 0 {
        return [0.0; 3];
    }
    #[allow(clippy::cast_precision_loss)]
    // A vista cobre `2 · half_extent` de mundo na altura da imagem.
    let px_por_mundo = altura_px as f32 / (2.0 * cam.half_extent.max(f32::EPSILON));
    mundo.map(|m| (m.max(0.0) * px_por_mundo).min(MAX_RAIO_PX))
}

/// ⭐⭐⭐ **QUANTAS PASSAGENS DE BORRÃO esta cena pede** — os espalhamentos DISTINTOS dela.
///
/// # ⚠️ Ela é o PREÇO da cura, e por isso é uma porta
///
/// O borrão é uma passagem sobre a imagem inteira: o custo é o número de **valores distintos**, e
/// não o de materiais. ⭐ Dois materiais com o mesmo número pedem a **mesma** passagem, e é isso
/// que faz toda cena de hoje — um valor só — continuar a pagar o que pagava.
///
/// ⛔⛔ **Ela existe porque o gate do preço tinha uma SEGUNDA CÓPIA desta dedução**, e uma mutação
/// que apagava a de cá **sobreviveu**: a saída não muda (todas as passagens dão a mesma imagem), só
/// o **custo** dobra — *e um gate que mede a imagem é cego a um custo que dobra em silêncio*.
///
/// ⚠️ **Os distintos comparam-se pelos BITS**, e é o que se quer: a pergunta não é *«são
/// parecidos»*, é *«é a mesma passagem»*.
///
/// ⛔ **Um material que não lê curvatura não pede passagem nenhuma** — ele não tem termo de
/// subsuperfície que a leia, por mais gordo que o raio dele esteja.
#[must_use]
pub fn espalhamentos_distintos(surfaces: &crate::Surfaces<'_>) -> Vec<[f32; 3]> {
    let mut distintos: Vec<[f32; 3]> = Vec::new();
    for s in surfaces.all {
        if !s.reads_curvature() {
            continue;
        }
        let e = s.scatter_distance();
        if !distintos
            .iter()
            .any(|d| d.map(f32::to_bits) == e.map(f32::to_bits))
        {
            distintos.push(e);
        }
    }
    distintos
}

/// ⭐⭐⭐ **A BORDA MOLE COM O RAIO DE CADA MATERIAL** — a cura do vazamento entre peças (ordem do
/// dono, 2026-09-19).
///
/// # ⛔⛔ O que ela cura
///
/// Até 18/09 o raio era o **MÁXIMO da cena**: uma esfera de espalhamento `0,05` ao lado de uma
/// chapa de `0,90` desenhava a borda dela com `0,90` — **`18×`**, e a razão declarada no código
/// dizia que *«só se via onde as duas peças se tocam»*, o que era falso (o raio é a **largura** com
/// que toda borda de sombra é amaciada). *A chapa escolhia o espalhamento da esfera.*
///
/// # ⭐⭐ Porque ela é por RAIO DISTINTO e não por material
///
/// O borrão é uma passagem sobre a imagem inteira, logo o preço é o número de passagens. ⚠️ Mas dois
/// materiais com o **mesmo** espalhamento pedem a **mesma** passagem ⇒ o custo é o número de
/// **valores distintos**, que numa cena real é `1` ou `2` e não o número de peças.
///
/// ⭐⭐⭐ **E com UM valor distinto ela é BYTE-IDÊNTICA ao que ship**, sem sequer olhar a quem é cada
/// pixel: *paga-se a correcção exactamente quando se usa a capacidade*, e uma cena de um material
/// só — que é toda cena de hoje — não paga nada.
///
/// ⚠️ **Um pixel cujo material não espalha leva a visibilidade DURA**, e não a de um vizinho: ele
/// não tem termo de subsuperfície para a ler, e emprestar-lhe um raio seria inventar espalhamento
/// onde o artista pôs zero.
#[must_use]
pub fn blur_por_material(
    g: &Gbuffer,
    vis: &[f32],
    surfaces: &crate::Surfaces<'_>,
    cam: &crate::Orbit,
    altura_px: u32,
) -> Vec<[f32; 3]> {
    if vis.len() != g.hit.len() {
        return Vec::new();
    }
    // ⚠️ **Os valores DISTINTOS, pelos bits** — comparar `f32` por igualdade é exactamente o que se
    // quer aqui: dois materiais com o mesmo número escrito pedem a mesma passagem, e dois que
    // diferem no último bit pedem duas. *A pergunta não é «são parecidos», é «é a mesma passagem».*
    let distintos = espalhamentos_distintos(surfaces);
    if distintos.is_empty() {
        return Vec::new();
    }
    let passagens: Vec<Vec<[f32; 3]>> = distintos
        .iter()
        .map(|e| blur_por_canal(g, vis, raio_em_pixeis(cam, altura_px, *e)))
        .collect();
    // ⭐⭐⭐ **O CAMINHO DE UM VALOR SÓ devolve a passagem CRUA** — byte-idêntico ao que ship, e sem
    // perguntar a quem é cada pixel. *Uma cena de um material não paga a pergunta que só uma cena de
    // dois precisa de fazer.*
    if passagens.len() == 1 {
        return passagens.into_iter().next().unwrap_or_default();
    }
    let mut out = vec![[0.0f32; 3]; vis.len()];
    for (i, o) in out.iter_mut().enumerate() {
        let s = surfaces.of(g.point[i]);
        if !s.reads_curvature() {
            *o = [vis[i]; 3];
            continue;
        }
        let e = s.scatter_distance().map(f32::to_bits);
        let k = distintos
            .iter()
            .position(|d| d.map(f32::to_bits) == e)
            .unwrap_or(0);
        *o = passagens[k][i];
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Uma tira de `n × 1` píxeis, com a normal a VIRAR ao meio — uma quina.
    fn tira_com_quina(n: usize) -> Gbuffer {
        #[allow(clippy::cast_possible_truncation)]
        Gbuffer {
            width: n as u32,
            height: 1,
            hit: vec![true; n],
            // Metade a apontar para `+z`, metade para `+x`: `cos = 0`, muito abaixo da guarda.
            normal: (0..n)
                .map(|i| {
                    if i < n / 2 {
                        [0.0, 0.0, 1.0]
                    } else {
                        [1.0, 0.0, 0.0]
                    }
                })
                .collect(),
            point: vec![[0.0; 3]; n],
            curvature: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// TRÊS bolas lado a lado, cada uma a sua FOLHA — para o `Owners` ter o que responder.
    ///
    /// ⚠️⚠️ **TRÊS e não duas, e foi uma mutação que o exigiu:** com duas, um caso *«translúcida +
    /// opaca»* tem **um** espalhamento distinto e cai no caminho rápido, onde a pergunta *«de quem é
    /// este pixel?»* nem chega a ser feita. *Para ver a regra do opaco é preciso que o caminho
    /// por-pixel corra*, e para isso a cena precisa de dois raios translúcidos **mais** o opaco.
    fn tres_pecas() -> (Gbuffer, ph2d_field_eval::owners::Owners) {
        const N: usize = 33;
        let reg = ph2d_field_eval::hybrid::Registry::new();
        let bola = |x: f32| {
            ph2d_field::FieldDoc::new(
                vec![ph2d_field_eval::leaf(
                    ph2d_field::Primitive::Sphere { radius: 0.5 },
                    ph2d_field::Xform {
                        translation: [x, 0.0, 0.0],
                        ..ph2d_field::Xform::IDENTITY
                    },
                )],
                ph2d_field::NodeId(0),
            )
            .expect("uma bola")
        };
        let owners =
            ph2d_field_eval::owners::Owners::new(&[bola(-2.0), bola(0.0), bola(2.0)], &reg, 1e-3);
        #[allow(clippy::cast_precision_loss)]
        let g = Gbuffer {
            // ⚠️ Um TERÇO dos píxeis DENTRO de cada bola — é o `point` que o `Owners` lê, e sem ele
            // este gate mediria um material só.
            point: (0..N)
                .map(|i| [((i * 3 / N) as f32 - 1.0) * 2.0, 0.0, 0.0])
                .collect(),
            // ⭐ Normais TODAS iguais: sem isto a guarda da quina corta a média a meio e o gate
            // passaria a medir a quina em vez do raio.
            normal: vec![[0.0, 0.0, 1.0]; N],
            ..tira_com_quina(N)
        };
        (g, owners)
    }

    /// ⭐⭐⭐ **CADA PEÇA É AMACIADA COM O RAIO DELA** — a cura do vazamento entre peças (ordem do
    /// dono, 2026-09-19).
    ///
    /// # ⛔⛔ O defeito que ela fecha
    ///
    /// Até 18/09 o raio era o **MÁXIMO da cena**: uma esfera de espalhamento `0,05` ao lado de uma
    /// chapa de `0,90` desenhava a borda dela com `0,90` — **`18×`**. *A vizinha escolhia o
    /// espalhamento da peça.*
    ///
    /// ⭐ **A régua é a IGUALDADE AO BIT com o que cada peça teria SOZINHA**, e não uma barra: o que
    /// se afirma é que a vizinha deixou de entrar na conta, e isso ou é exacto ou não aconteceu.
    #[test]
    fn cada_peca_e_amaciada_com_o_raio_dela_e_nao_com_o_da_vizinha() {
        let (g, owners) = tres_pecas();
        let n = g.hit.len();
        let vis: Vec<f32> = (0..n).map(|i| f32::from(i % 2 == 0)).collect();
        let jade = |raio: f32| {
            ph2d_material::OpenPbr {
                subsurface_weight: 1.0,
                subsurface_radius: raio,
                ..ph2d_material::OpenPbr::default()
            }
            .prepare()
        };
        // ⚠️ **O opaco tem RAIO GORDO**: o que o deixa de fora é o PESO — *uma fixtura cujo valor
        // «mau» é zero não testa o filtro que o deita fora.*
        let opaco = ph2d_material::OpenPbr {
            subsurface_weight: 0.0,
            subsurface_radius: 0.90,
            ..ph2d_material::OpenPbr::default()
        }
        .prepare();
        let (magra, gorda) = (jade(0.05), jade(0.90));
        let cam = crate::Orbit::default();
        let mats = [magra, gorda, opaco];
        let out = blur_por_material(
            &g,
            &vis,
            &crate::Surfaces {
                all: &mats,
                owners: Some(&owners),
            },
            &cam,
            64,
        );
        // O que cada uma teria SOZINHA, sobre a mesma imagem.
        let so = |s: &ph2d_material::Surface| {
            blur_por_canal(&g, &vis, raio_em_pixeis(&cam, 64, s.scatter_distance()))
        };
        let (so_magra, so_gorda) = (so(&magra), so(&gorda));
        for i in 0..n {
            let peca = i * 3 / n;
            // ⭐⭐ **O terço OPACO leva a visibilidade DURA**, e não o borrão de um vizinho: ele não
            // tem termo de subsuperfície que a leia, e emprestar-lhe um raio seria inventar
            // espalhamento onde o artista pôs zero.
            let esperado = match peca {
                0 => so_magra[i],
                1 => so_gorda[i],
                _ => [vis[i]; 3],
            };
            assert_eq!(
                out[i].map(f32::to_bits),
                esperado.map(f32::to_bits),
                "o pixel {i} (peça {peca}) não levou o que a peça dele pede"
            );
        }
        // ⭐ **O CONTROLO**: as duas passagens translúcidas têm mesmo de ser DIFERENTES, senão o
        // teste acima passa por vácuo — *um gate que compara duas coisas iguais não afirma nada*.
        assert_ne!(
            so_magra[n / 6].map(f32::to_bits),
            so_gorda[n / 6].map(f32::to_bits),
            "os dois raios deram a mesma imagem — a fixtura deixou de conter o fenómeno"
        );
    }

    /// ⭐⭐⭐ **COM UM RAIO SÓ, A SAÍDA É BYTE-IDÊNTICA À QUE JÁ SHIP** — e é isso que faz a cura
    /// não tocar em nenhuma cena de hoje.
    ///
    /// ⚠️ **Ela nem sequer pergunta a quem é cada pixel** nesse caso: *paga-se a correcção
    /// exactamente quando se usa a capacidade*, e uma cena de um material não paga a pergunta que
    /// só uma cena de dois precisa de fazer.
    #[test]
    fn com_um_raio_so_a_saida_e_a_de_sempre_ao_bit() {
        let (g, owners) = tres_pecas();
        let n = g.hit.len();
        let vis: Vec<f32> = (0..n).map(|i| f32::from(i % 2 == 0)).collect();
        let m = ph2d_material::OpenPbr {
            subsurface_weight: 1.0,
            subsurface_radius: 0.30,
            ..ph2d_material::OpenPbr::default()
        }
        .prepare();
        let cam = crate::Orbit::default();
        let mats = [m, m];
        let out = blur_por_material(
            &g,
            &vis,
            &crate::Surfaces {
                all: &mats,
                owners: Some(&owners),
            },
            &cam,
            64,
        );
        let sempre = blur_por_canal(&g, &vis, raio_em_pixeis(&cam, 64, m.scatter_distance()));
        assert_eq!(
            out.iter().map(|v| v.map(f32::to_bits)).collect::<Vec<_>>(),
            sempre
                .iter()
                .map(|v| v.map(f32::to_bits))
                .collect::<Vec<_>>(),
            "com um raio só a saída deixou de ser a de sempre — a cura passou a mexer no que ship"
        );
    }

    /// ⭐⭐ **UMA CENA SÓ DE OPACOS NÃO ASSA CANAL NENHUM** — o `soft_at` cai na visibilidade DURA e
    /// o quadro é byte a byte o de sempre.
    #[test]
    fn uma_cena_so_de_opacos_nao_assa_canal_nenhum() {
        let (g, owners) = tres_pecas();
        let vis = vec![0.5f32; g.hit.len()];
        // ⚠️ **Opaco com RAIO GORDO**: o que o deixa de fora é o PESO, e não o raio dele estar a
        // zero — uma fixtura cujo valor «mau» é zero não testa o filtro que o deita fora.
        let opaco = ph2d_material::OpenPbr {
            subsurface_weight: 0.0,
            subsurface_radius: 0.90,
            ..ph2d_material::OpenPbr::default()
        }
        .prepare();
        let mats = [opaco, opaco];
        assert!(
            blur_por_material(
                &g,
                &vis,
                &crate::Surfaces {
                    all: &mats,
                    owners: Some(&owners),
                },
                &crate::Orbit::default(),
                64,
            )
            .is_empty(),
            "uma cena só de opacos assou um canal — o quadro deixa de ser byte a byte o de sempre"
        );
    }

    /// ⭐⭐⭐ **A MÉDIA NÃO ATRAVESSA UMA QUINA** — o gémeo do gate que o céu já tem.
    ///
    /// ⚠️ Ele existe porque uma **mutação SOBREVIVEU**: a cena da bola de jade é lisa, e apagar a
    /// guarda de normal não movia um byte lá. *Uma cena sem quina nenhuma não testa a guarda da
    /// quina*, e a régua tem de trazer o fenómeno consigo.
    #[test]
    fn a_media_da_borda_mole_nao_atravessa_uma_quina() {
        const N: usize = 32;
        let g = tira_com_quina(N);
        // Um degrau de visibilidade que coincide com a quina.
        let vis: Vec<f32> = (0..N).map(|i| if i < N / 2 { 0.0 } else { 1.0 }).collect();
        let out = blur_por_canal(&g, &vis, [8.0; 3]);
        // ⭐⭐ **A asserção mora COLADA à quina**, e não nas pontas: a `r = 8` as pontas ficam fora
        // do alcance dela e leem `0` e `1` na mesma **com a guarda apagada** — *uma mutação
        // SOBREVIVEU por eu ter medido onde a guarda não decide nada*.
        assert!(
            (out[N / 2 - 1][0] - 0.0).abs() < 1e-6 && (out[N / 2][0] - 1.0).abs() < 1e-6,
            "os dois píxeis colados à quina leem {:.4} e {:.4} — a média atravessou-a",
            out[N / 2 - 1][0],
            out[N / 2][0]
        );
        // ⭐ E o CONTROLO: sem quina, a mesma média ESBORRATA o mesmo degrau. Sem esta metade, uma
        // «média» que não fizesse nada passaria na de cima.
        let liso = Gbuffer {
            normal: vec![[0.0, 0.0, 1.0]; N],
            ..tira_com_quina(N)
        };
        let out = blur_por_canal(&liso, &vis, [8.0; 3]);
        assert!(
            out[N / 2 - 1][0] > 0.05 && out[N / 2][0] < 0.95,
            "sem quina o degrau ficou em {:.4}/{:.4} — a média não está a fazer nada",
            out[N / 2 - 1][0],
            out[N / 2][0]
        );
    }

    /// ⭐⭐⭐ **UM CANAL QUE NÃO COBRE O GBUFFER DEVOLVE VAZIO — e nunca entra em pânico.**
    ///
    /// ⛔⛔ **Ele estoirava**, e foi uma sonda de 2026-09-18 que o pisou:
    /// `index out of bounds: the len is 0 but the index is 9983`. As duas passagens percorrem o
    /// **gbuffer** enquanto indexam o **canal**, e com um `vis` vazio — uma lâmpada que o passe de
    /// sombra não escreveu — o índice sai da faixa.
    ///
    /// ⭐ **O vazio é a resposta CERTA:** o [`crate::Shadows::soft_at`] cai na visibilidade DURA
    /// quando não há canal mole, que é exactamente *«esta lâmpada não tem borda mole»*.
    ///
    /// ⚠️ **O primeiro chamador só não o pisou porque percorre as lâmpadas que EXISTEM** — *um
    /// porte que estoira sobre uma entrada vazia é uma armadilha para o segundo chamador*, e o
    /// segundo chegou onze dias depois.
    #[test]
    fn um_canal_que_nao_cobre_o_gbuffer_devolve_vazio_em_vez_de_estoirar() {
        const N: usize = 16;
        let g = tira_com_quina(N);
        // (1) O caso que estoirava: nenhuma lâmpada, logo nenhum canal.
        assert!(
            blur_por_canal(&g, &[], [8.0; 3]).is_empty(),
            "um canal VAZIO devolveu alguma coisa — ou estoirou"
        );
        // (2) ⚠️ E um canal CURTO conta como o mesmo defeito: ele indexaria igual.
        assert!(
            blur_por_canal(&g, &[1.0; N - 1], [8.0; 3]).is_empty(),
            "um canal mais curto que o gbuffer passou — é o mesmo índice fora da faixa"
        );
        // (3) ⭐ O CONTROLO: o canal do tamanho certo continua a ser borrado, senão esta guarda
        // teria apagado a passagem inteira e as duas metades acima passariam por vácuo.
        assert_eq!(
            blur_por_canal(&g, &[1.0; N], [8.0; 3]).len(),
            N,
            "o canal do tamanho certo deixou de ser borrado — a guarda comeu o caminho bom"
        );
    }

    /// ⭐⭐ **Raio zero num canal ⇒ esse canal é a visibilidade DURA, ao bit** — é o que faz um
    /// material sem espalhamento não pagar nada e o quadro sair o de sempre.
    #[test]
    fn raio_zero_devolve_a_visibilidade_dura_ao_bit() {
        const N: usize = 16;
        let g = tira_com_quina(N);
        #[allow(clippy::cast_precision_loss)]
        let vis: Vec<f32> = (0..N).map(|i| i as f32 / N as f32).collect();
        let out = blur_por_canal(&g, &vis, [0.0; 3]);
        for i in 0..N {
            assert!(
                (out[i][0] - vis[i]).to_bits() == 0.0f32.to_bits(),
                "o pixel {i} lê {:.6} contra {:.6} — o raio zero mexeu no canal",
                out[i][0],
                vis[i]
            );
        }
    }
}
