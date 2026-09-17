//! ⭐⭐⭐ **A BANCADA DO PRODUTO do pincel afiado** — o TRAÇO arrastado, pela
//! porta que o artista usa, contra as fixturas de `produto/` e `detector/`
//! (`SPEC_pincel_afiado.md` §12: G-3, G-4, G-5, G-8, G-10, G-11).
//!
//! ⚠️⚠️ **É a metade que a obra do pincel de plano teve de aprender à custa de
//! um report do dono:** a lei por dab batia a `1e-8` e o produto que ele usava
//! saía pior que o do alvo, porque o corpus fixava os valores à mão e dava os
//! dabs por script. Aqui o pincel vem com os valores de FÁBRICA, o traço é
//! **arrastado** com o passo e a atenuação, e o cursor é picado na superfície
//! VIVA a cada dab — as três coisas que o artista faz sem saber que as faz.
//!
//! ⚠️ **A régua é a do §7.1 da espec** e mede o que o nome do pincel promete: a
//! profundidade do vinco, a largura a meia profundidade e a nitidez. ⛔ Uma
//! comparação vértice a vértice sozinha não a substitui — ela diz *quão perto*,
//! e não *o que o artista vê*.

use super::oraculo_do_pincel_afiado::{
    Fixtura, ler, maior_distancia, malha, na_superficie, olho, pincel,
};
use ph2d_mesh::Mesh;
use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry};

/// G-3a (§12): o traço contínuo, na FAIXA do vinco.
const BARRA_CONTINUO: f32 = 2e-3;
/// G-3b: os traços separados, na faixa do vinco.
const BARRA_SEPARADOS: f32 = 5e-3;
/// ⛔ **O TECTO DECLARADO das PONTAS do traço** — e ele **DESCEU** em 2026-09-16,
/// porque a cura da silhueta pagou a divergência D-1 **deste verbo**.
///
/// ⚠️ **A D-1 era do [`ph2d_sculpt3d::walk`]:** o alvo deposita um dab num salto
/// de EXACTAMENTE um passo e aquele `walk` recusa-o, o que a meio do traço dá a
/// mesma lista e **na INVERSÃO** perde o dab adiado (medido então: `4,04e-2` no
/// vértice da ponta). ⭐ O afiado já **não passa por ali** — ele percorre o
/// [`ph2d_sculpt3d::CaminhoNoMundo`], cuja fronteira (`acumulado < passo` ⇒ zero)
/// é a do alvo ⇒ a sombra da ponta cai para **`1,91e-2`**.
///
/// ⛔⛔ **Descer este número não é cosmética: mantê-lo em `5e-2` transformava-o
/// em LICENÇA** — passava a caber ali uma regressão do dobro do que hoje se mede.
/// *Uma catraca que a cura fez descer e ninguém desceu é uma catraca que virou
/// tolerância.* Medido, o pior das duas suítes: contínuo `1,17e-2` · separado
/// `2,18e-2` (a cura) contra `4,06e-2` (a régua de ecrã do alvo).
const TECTO_DAS_PONTAS: f32 = 2.5e-2;
/// ⛔⛔⛔ **A DIVERGÊNCIA DECLARADA DA CURA** — o tecto da faixa do vinco na
/// ÚLTIMA passagem de um vaivém de traços SEPARADOS.
///
/// ⚠️ **Ela é o preço, medido e atribuído, do centro do dab que segue o barro**
/// ([`ph2d_sculpt3d::levado_pela_deformacao`]) — e **não** do passo medido sobre
/// a superfície, que é a outra metade da cura. A tabela das três leis do cursor
/// (a sonda `diag_a_tabela_das_tres_leis`), na faixa do vinco:
///
/// | lei | p1 | p2 | p4 | p8 | pontas p8 |
/// |---|---|---|---|---|---|
/// | ecrã (o alvo) | `4,0e-4` | `2,0e-4` | `4,7e-4` | `1,1e-3` | `4,04e-2` |
/// | só o PASSO | `4,1e-4` | `2,7e-4` | `5,9e-4` | `1,2e-3` | `2,20e-2` |
/// | passo + BARRO | `4,0e-4` | `7,3e-4` | `1,9e-3` | **`1,71e-2`** | `1,91e-2` |
///
/// ⭐⭐ **O mecanismo lê-se na própria tabela: a divergência CRESCE com o número
/// de passagens, que é o mesmo que dizer que ela cresce com o quanto a superfície
/// já está ESCULPIDA.** Na 1.ª passagem a peça é um plano de frente para a vista
/// e as três leis dão o mesmo número; da 2.ª em diante o cursor anda dentro de um
/// vinco, que é superfície CURVA — e é exactamente aí que as duas leis são
/// desenhadas para discordar: *o nosso cursor é um facto do BARRO e o do alvo é
/// um facto do RAIO*. ⛔ Ali o lado do alvo **é** o defeito que o dono
/// fotografou: o mesmo pontilhado, reproduzido dentro do próprio vinco dele.
///
/// ⚠️ **Ao nível do PRODUTO a troca é pequena e está medida pela régua do §7.1**
/// (`a_regua_do_vinco_concorda_com_a_do_alvo`): na 8.ª passagem a profundidade
/// fica `2,4 %`–`3,6 %` mais rasa e o vinco `1,4 %`–`4,9 %` mais largo. As
/// passagens `1`–`4` ficam dentro da barra apertada de sempre.
const TECTO_DO_ULTIMO_SEPARADO: f32 = 2.0e-2;

/// Uma linha da tabela: `(passagem, desvio na faixa, desvio nas pontas)`.
type LinhaDaCelula = (usize, f32, f32);
/// Uma coluna da tabela: a lei do cursor e o que ela deu em cada passagem.
type ColunaDaLei = (LeiDoCursor, Vec<LinhaDaCelula>);

/// Meia largura da FAIXA do vinco (em unidades do mundo) — a mesma da espec.
const FAIXA: f32 = 0.25;

/// O pior desvio dentro da faixa do vinco, e o pior FORA dela.
fn desvio(f: &Fixtura, nossa: &[[f32; 3]], alvo: &[[f32; 3]]) -> (f32, f32) {
    let (mut faixa, mut pontas) = (0.0f32, 0.0f32);
    for (i, r) in f.repouso.iter().enumerate() {
        let d = (0..3)
            .map(|k| (nossa[i][k] - alvo[i][k]).abs())
            .fold(0.0f32, f32::max);
        if r[0].abs() <= FAIXA {
            faixa = faixa.max(d);
        } else {
            pontas = pontas.max(d);
        }
    }
    (faixa, pontas)
}

/// **O ARRASTO como a app o faz**, e uma foto depois de cada passagem.
///
/// ⭐ **Uma passagem é UMA travessia** (a mesma convenção da bancada do pincel
/// de plano): o cabeçalho das fixturas diz *«8 passagens em vaivém»*, e o
/// `p<k>` é a foto no fim da `k`-ésima.
///
/// ⚠️ **O percurso sai do CABEÇALHO, nunca desta prosa:** o píxel do pen-down no
/// mundo, o vector de um píxel para a direita e os píxeis por unidade. Com eles
/// o passo em píxeis é o do produto (`passo_do_traco`), e o `walk` da casa
/// decide onde cada dab cai.
///
/// ⚠️ **`separados` vem do cabeçalho também** — a caneta levanta no fim de cada
/// passagem, e cada pen-down **refotografa** as posições, que é o que faz o
/// limite do vinco recomeçar (espec §4.2).
fn arrastar(f: &Fixtura, b: &Brush, passagens: usize) -> Vec<Vec<[f32; 3]>> {
    arrastar_com(f, b, passagens, LeiDoCursor::PassoEBarro)
}

/// ⭐⭐⭐ **AS TRÊS LEIS DO CURSOR, como PARÂMETRO e nunca como variável de
/// ambiente** — é isto que deixa um gate medir a cura **e** o controlo dela na
/// mesma corrida.
///
/// ⚠️ **Uma bissecção por `PH2D_*` não é um controlo:** ela mora fora do teste,
/// ninguém a corre no portão, e `env VAR= cargo …` **define** a variável vazia —
/// uma leitura por `is_err()` lê isso como armado e as duas colunas saem iguais,
/// que foi exactamente o que aconteceu ao medir esta tabela pela primeira vez.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LeiDoCursor {
    /// A régua do ALVO: passo contado em píxeis de ecrã, cursor picado na
    /// superfície VIVA. É o lado de ANTES da cura.
    Ecra,
    /// Metade da cura: o passo medido sobre a superfície do pen-down, com o
    /// cursor ainda picado no vivo.
    Passo,
    /// A cura inteira: o passo sobre a superfície **e** o centro do dab levado
    /// pela deformação. É o que o produto corre.
    PassoEBarro,
}

fn arrastar_com(f: &Fixtura, b: &Brush, passagens: usize, lei: LeiDoCursor) -> Vec<Vec<[f32; 3]>> {
    let px = f.num("vista_px_por_unidade");
    let pd: Vec<f32> = f
        .chave("pixel_do_pen_down_no_mundo")
        .split_whitespace()
        .take(2)
        .map(|s| s.parse().expect("pixel do pen-down"))
        .collect();
    let um: Vec<f32> = f
        .chave("um_pixel_para_a_direita")
        .split_whitespace()
        .take(2)
        .map(|s| s.parse().expect("um pixel"))
        .collect();
    let separados = f.chave("caminho_do_traco").contains("SEPARADOS");
    let passo = ph2d_sculpt3d::passo_do_traco(b.verb, b.radius * px);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let eventos = (1.0 / um[0]).round() as usize;
    let mut m = malha(f);
    let e = olho(f);
    let mut s = SculptStroke::default();
    s.begin(&m);
    // ⚠️⚠️ **A fotografia é RE-TIRADA em cada pen-down** — o `space.rs` arma-a
    // no pen-down, não uma vez por sessão. ⛔ Uma bancada que a tirasse uma só
    // vez mediria o 8.º traço contra a superfície do 1.º, que é outro programa.
    let mut congelada = m.clone();
    let mundo = |t: f32| (pd[0] + um[0] * t, pd[1] + um[1] * t);
    let carimbar = |m: &mut Mesh, s: &mut SculptStroke, t: f32| {
        let (x, y) = mundo(t);
        if let Some(c) = na_superficie(m, x, y, e) {
            s.dab(m, b, &Dab::at(c, b.radius, e), Symmetry::default());
        }
    };
    let mut t = 0.0f32;
    carimbar(&mut m, &mut s, t);
    let mut ancora = [t, 0.0];
    let mut fotos = Vec::with_capacity(passagens);
    // ⭐⭐⭐ **A bancada corre a LEI DO PRODUTO, e desde 2026-09-16 ela é a do
    // passo medido sobre a superfície** ([`ph2d_sculpt3d::CaminhoNoMundo`]).
    // ⚠️ **É por isso que estes gates continuam a afirmar paridade:** num plano
    // de frente para a vista as duas leis COINCIDEM por construção (`cos θ = 1`),
    // e todo este corpus é plano ou um cilindro percorrido ao longo do eixo.
    // *Uma bancada que corresse a lei antiga mediria um programa que já não
    // existe.*
    let no_mundo = b.verb.mede_o_passo_no_mundo() && lei != LeiDoCursor::Ecra;
    let passo_mundo = ph2d_sculpt3d::passo_no_mundo(b.verb, b.radius);
    let mut caminho = ph2d_sculpt3d::CaminhoNoMundo::novo();
    for k in 0..passagens {
        let dir = if k % 2 == 0 { 1.0 } else { -1.0 };
        for _ in 0..eventos {
            let de = t;
            t += dir;
            match (no_mundo, passo_mundo) {
                (true, Some(pm)) => {
                    let mut alvo = CarimboDoProduto {
                        malha: &mut m,
                        traco: &mut s,
                        congelada: &congelada,
                        pincel: b,
                        mundo: &mundo,
                        olho: e,
                        leva: b.verb.o_dab_segue_o_barro() && lei == LeiDoCursor::PassoEBarro,
                    };
                    caminho.percorre(de, t, pm, &mut alvo);
                    ancora = [t, 0.0];
                }
                _ => {
                    if let Some(passos) = ph2d_sculpt3d::walk(ancora, [t, 0.0], passo) {
                        for q in passos {
                            carimbar(&mut m, &mut s, q[0]);
                        }
                        ancora = passos.anchor();
                    }
                }
            }
        }
        fotos.push(m.positions().to_vec());
        if separados && k + 1 < passagens {
            // A caneta levanta e volta a descer na ponta onde esta acabou.
            s.begin(&m);
            caminho.esquece();
            congelada = m.clone();
            carimbar(&mut m, &mut s, t);
            ancora = [t, 0.0];
        }
    }
    fotos
}

/// **A BANCADA a responder às duas perguntas da lei do caminho** — a gémea do
/// `CarimboDaCena` do app.
struct CarimboDoProduto<'a> {
    malha: &'a mut Mesh,
    traco: &'a mut SculptStroke,
    congelada: &'a Mesh,
    pincel: &'a Brush,
    mundo: &'a dyn Fn(f32) -> (f32, f32),
    olho: [f32; 3],
    leva: bool,
}

impl ph2d_sculpt3d::CarimboDoCaminho for CarimboDoProduto<'_> {
    fn congelado(&mut self, t: f32) -> Option<[f32; 3]> {
        let (x, y) = (self.mundo)(t);
        na_superficie(self.congelada, x, y, self.olho)
    }

    fn carimba(&mut self, t: f32) -> bool {
        let (x, y) = (self.mundo)(t);
        let origem = [
            x - self.olho[0] * 10.0,
            y - self.olho[1] * 10.0,
            -self.olho[2] * 10.0,
        ];
        let centro = if self.leva {
            self.congelada
                .raycast(&ph2d_mesh::Ray::new(origem, self.olho))
                .and_then(|h| ph2d_sculpt3d::levado_pela_deformacao(self.congelada, self.malha, &h))
        } else {
            na_superficie(self.malha, x, y, self.olho)
        };
        if let Some(c) = centro {
            self.traco.dab(
                self.malha,
                self.pincel,
                &Dab::at(c, self.pincel.radius, self.olho),
                Symmetry::default(),
            );
        }
        true
    }
}

/// O pincel de uma fixtura de produto, com o raio do cabeçalho.
fn pincel_produto(f: &Fixtura) -> Brush {
    let b = pincel(f);
    assert!(
        b.traco_arrastado,
        "{}: uma fixtura de produto tem de vir do caminho ARRASTADO",
        f.nome
    );
    b
}

/// Quantas passagens o cabeçalho pede.
fn passagens(f: &Fixtura) -> usize {
    let c = f.chave("caminho_do_traco");
    let i = c.find("passagens").expect("passagens no cabecalho");
    c[..i]
        .split_whitespace()
        .next_back()
        .and_then(|p| p.parse().ok())
        .expect("numero de passagens")
}

/// ⭐⭐⭐ **G-3a — o traço CONTÍNUO reproduz o oráculo, vértice a vértice**, nas
/// seis superfícies que a espec conta, em cada foto publicada.
#[test]
fn o_traco_continuo_reproduz_o_oraculo() {
    const CELULAS: [&str; 6] = [
        "afiado_valores_de_fabrica_continuo",
        "afiado_valores_de_fabrica_ctrl",
        "densidade_48_afiado_continuo",
        "densidade_192_afiado_continuo",
        "cilindro_afiado_continuo",
        "triangulos_afiado_continuo",
    ];
    let (mut pior, mut pior_pontas) = (0.0f32, 0.0f32);
    for nome in CELULAS {
        let f = ler("produto", nome);
        let b = pincel_produto(&f);
        let n = passagens(&f);
        let fotos = arrastar(&f, &b, n);
        for (k, foto) in fotos.iter().enumerate() {
            let passagem = k + 1;
            let alvo = if passagem == n {
                Some(f.saida())
            } else {
                f.blocos.get(&format!("p{passagem}")).map(Vec::as_slice)
            };
            let Some(alvo) = alvo else { continue };
            let (d, pontas) = desvio(&f, foto, alvo);
            pior_pontas = pior_pontas.max(pontas);
            assert!(
                d <= BARRA_CONTINUO,
                "{nome}, passagem {passagem}: na faixa do vinco max|Δ| {d:.3e} passa a barra \
                 {BARRA_CONTINUO:.0e}"
            );
            assert!(
                pontas <= TECTO_DAS_PONTAS,
                "{nome}, passagem {passagem}: nas pontas {pontas:.3e} passa o TECTO DECLARADO \
                 {TECTO_DAS_PONTAS:.0e} — a sombra da D-1 cresceu"
            );
            pior = pior.max(d);
        }
    }
    println!(
        "G-3a: pior de {} celulas = {pior:.3e} na faixa · {pior_pontas:.3e} nas pontas (D-1)",
        CELULAS.len()
    );
}

/// ⭐⭐⭐ **G-3b — os traços SEPARADOS contra o oráculo, pelas TRÊS leis do
/// cursor**, na faixa central e fora dela.
///
/// ⚠️ **Eles são o caso DURO**, e por uma razão de produto: cada pen-down
/// refotografa as posições, logo o limite do vinco recomeça e ele fica cada vez
/// mais fundo e mais estreito — ao 8.º traço o cursor cai num vinco já estreito,
/// e **onde** ele cai muda a profundidade.
///
/// ⭐⭐ **O CONTROLO vive DENTRO do gate, e é ele que ATRIBUI a divergência.**
/// Sem a coluna do meio ([`LeiDoCursor::Passo`]) este gate diria *«a cura custa
/// paridade»* sem dizer QUAL das duas metades a custa — e a resposta muda a
/// decisão: o passo medido sobre a superfície não custa nada e melhora a ponta;
/// quem paga é o centro levado pelo barro, que é o que cura o report do dono.
///
/// ⛔⛔ **A última asserção é a que impede o tecto declarado de virar licença:**
/// ela exige que a divergência **EXISTA**. No dia em que alguém a fizer
/// desaparecer, este gate reprova e obriga a apagar o
/// [`TECTO_DO_ULTIMO_SEPARADO`] em vez de o deixar a cobrir uma regressão nova.
#[test]
fn os_tracos_separados_reproduzem_o_oraculo() {
    const CELULAS: [&str; 5] = [
        "afiado_valores_de_fabrica_separados",
        "densidade_48_afiado_separados",
        "densidade_192_afiado_separados",
        "cilindro_afiado_separados",
        "triangulos_afiado_separados",
    ];
    let (mut pior_faixa, mut pior_pontas) = (0.0f32, 0.0f32);
    let mut divergiu = false;
    for nome in CELULAS {
        let f = ler("produto", nome);
        let b = pincel_produto(&f);
        let n = passagens(&f);
        // Uma coluna por lei, para as três serem comparáveis entre si — e não só
        // cada uma contra uma barra.
        let colunas: Vec<ColunaDaLei> = [
            LeiDoCursor::Ecra,
            LeiDoCursor::Passo,
            LeiDoCursor::PassoEBarro,
        ]
        .into_iter()
        .map(|lei| {
            let linhas = arrastar_com(&f, &b, n, lei)
                .iter()
                .enumerate()
                .filter_map(|(k, foto)| {
                    let passagem = k + 1;
                    let alvo = if passagem == n {
                        Some(f.saida())
                    } else {
                        f.blocos.get(&format!("p{passagem}")).map(Vec::as_slice)
                    }?;
                    let (faixa, pontas) = desvio(&f, foto, alvo);
                    Some((passagem, faixa, pontas))
                })
                .collect();
            (lei, linhas)
        })
        .collect();
        for (lei, linhas) in &colunas {
            for &(passagem, faixa, _) in linhas {
                // ⭐ A barra APERTADA vale para tudo menos a última passagem da
                // cura — e valer para as outras DUAS leis NA última é o que
                // prova que o passo sozinho não custa paridade nenhuma.
                let ultima_da_cura = passagem == n && *lei == LeiDoCursor::PassoEBarro;
                let barra = if ultima_da_cura {
                    TECTO_DO_ULTIMO_SEPARADO
                } else {
                    BARRA_SEPARADOS
                };
                assert!(
                    faixa <= barra,
                    "{nome} · {lei:?} · traço {passagem}: na faixa do vinco {faixa:.3e} passa \
                     a barra {barra:.0e}"
                );
                divergiu |= ultima_da_cura && faixa > BARRA_SEPARADOS;
            }
        }
        // ⛔⛔ **A sombra da ponta é uma CATRACA, não uma tolerância:** a lei que
        // shipa fica debaixo do tecto **e** o PIOR dela na célula é melhor que o
        // pior da régua de ecrã do alvo. *Sem a segunda metade, o dia em que a
        // cura regredisse para o valor do alvo passaria calado.*
        //
        // ⚠️⚠️ **É o PIOR da célula e NÃO cada passagem, e a diferença foi
        // medida:** a cura ganha nos extremos (`5,25e-3` contra `1,04e-2` na 1.ª,
        // `1,91e-2` contra `4,04e-2` na 8.ª) e na 2.ª passagem fica um cabelo
        // ATRÁS (`1,069e-2` contra `1,021e-2`). ⛔ Uma catraca escrita passagem a
        // passagem reprovaria sobre uma cura que corta a sombra ao meio — *uma
        // barra afirmada antes de medida mede a redacção dela, não o produto.*
        let pior_de =
            |linhas: &[LinhaDaCelula]| linhas.iter().fold(0.0f32, |a, &(_, _, p)| a.max(p));
        let (p_ecra, p_cura) = (pior_de(&colunas[0].1), pior_de(&colunas[2].1));
        assert!(
            p_cura <= TECTO_DAS_PONTAS,
            "{nome}: nas pontas {p_cura:.3e} passa o TECTO DECLARADO {TECTO_DAS_PONTAS:.0e}"
        );
        assert!(
            p_cura < p_ecra,
            "{nome}: a cura ({p_cura:.3e}) deixou de ser melhor na ponta que a régua de ecrã do \
             alvo ({p_ecra:.3e})"
        );
        pior_faixa = colunas[2]
            .1
            .iter()
            .fold(pior_faixa, |a, &(_, d, _)| a.max(d));
        pior_pontas = pior_pontas.max(p_cura);
    }
    assert!(
        divergiu,
        "a divergência declarada DESAPARECEU — apague o TECTO_DO_ULTIMO_SEPARADO em vez de o \
         deixar a cobrir uma regressão futura"
    );
    println!("G-3b (a cura): faixa {pior_faixa:.3e} · pontas {pior_pontas:.3e}");
}

// ── A RÉGUA do produto (espec §7.1) ─────────────────────────────────────────

/// O perfil do vinco: a média do deslocamento ao longo da normal de repouso,
/// por linha transversal, sobre as colunas com `|x| ≤ 0,1`.
///
/// ⚠️ **A malha é uma grelha com `x` a variar primeiro** — o README do corpus
/// declara-o, e é o que torna a linha transversal um bloco contíguo de índices.
fn perfil(f: &Fixtura, pos: &[[f32; 3]]) -> Vec<(f32, f32)> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let lado = (f.repouso.len() as f64).sqrt().round() as usize;
    let cilindro = f.chave("superficie").contains("cilindro");
    let mut saida = Vec::with_capacity(lado);
    for j in 0..lado {
        let (mut soma, mut n, mut coord) = (0.0f64, 0usize, 0.0f64);
        for i in 0..lado {
            let k = j * lado + i;
            let r = f.repouso[k];
            if r[0].abs() > 0.1 {
                continue;
            }
            // A normal de REPOUSO, analítica: o plano olha para `+z`; o cilindro
            // tem o eixo ao longo de `x` e raio `1,2`.
            let nrm = if cilindro {
                let (y, z) = (f64::from(r[1]), f64::from(r[2]) + 1.2);
                let l = (y * y + z * z).sqrt();
                [0.0, y / l, z / l]
            } else {
                [0.0, 0.0, 1.0]
            };
            let d: f64 = (0..3).map(|c| f64::from(pos[k][c] - r[c]) * nrm[c]).sum();
            soma += d;
            coord += if cilindro {
                1.2 * f64::from(r[1]).atan2(f64::from(r[2]) + 1.2)
            } else {
                f64::from(r[1])
            };
            n += 1;
        }
        if n > 0 {
            #[allow(clippy::cast_possible_truncation)]
            saida.push(((coord / n as f64) as f32, (soma / n as f64) as f32));
        }
    }
    saida
}

/// `D` (a profundidade) e `W50` (a largura INTEIRA a meia profundidade), em
/// unidades do raio — a régua da espec §7.1.
///
/// ⚠️ **`W` é a largura TOTAL entre os dois cruzamentos** (errata do R-pré), e
/// eles acham-se andando **para fora** a partir do máximo, um de cada lado.
fn regua(f: &Fixtura, pos: &[[f32; 3]], raio: f32) -> (f32, f32) {
    let p = perfil(f, pos);
    // ⭐ **O SINAL sai do próprio perfil e não de uma suposição:** o pincel de
    // fábrica afunda, mas o corpus tem fixturas a SOMAR (a ablação, o `Ctrl`) e
    // o controlo do desenho comum, que levanta. *Uma régua que presume o lado
    // devolve `NaN` na primeira fixtura do outro.*
    let lado = {
        let extremo = p
            .iter()
            .map(|v| v.1)
            .fold(0.0f32, |a, b| if b.abs() > a.abs() { b } else { a });
        if extremo < 0.0 { -1.0f32 } else { 1.0 }
    };
    let p: Vec<(f32, f32)> = p.into_iter().map(|(c, v)| (c, v * lado)).collect();
    let (topo, d) = p
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.1.partial_cmp(&b.1.1).expect("perfil sem NaN"))
        .map(|(i, v)| (i, v.1))
        .expect("perfil vazio");
    let meio = d * 0.5;
    let cruza = |de: usize, passo: isize| -> Option<f32> {
        let mut i = de;
        loop {
            let j = i.checked_add_signed(passo)?;
            let (a, b) = (*p.get(i)?, *p.get(j)?);
            if b.1 <= meio {
                let t = (a.1 - meio) / (a.1 - b.1);
                return Some(a.0 + (b.0 - a.0) * t);
            }
            i = j;
        }
    };
    let (esq, dir) = (cruza(topo, -1), cruza(topo, 1));
    let w = match (esq, dir) {
        (Some(a), Some(b)) => (b - a).abs(),
        _ => f32::NAN,
    };
    (d / raio, w / raio)
}

/// ⭐⭐⭐ **G-4 — a RÉGUA do produto**: a profundidade, a largura e a nitidez do
/// nosso vinco contra as do alvo, em cada superfície e em cada passagem
/// publicada.
///
/// ⚠️ **É este o gate que responde ao dono**, e não o G-3: ele mede o que o
/// nome do pincel promete. As barras são as da espec §12 (`6 %` · `2 %` · `6 %`
/// no contínuo; mais folgadas no 8.º traço separado, que é sensível por
/// construção).
#[test]
fn a_regua_do_vinco_concorda_com_a_do_alvo() {
    for (nome, barra_d, barra_w) in [
        ("afiado_valores_de_fabrica_continuo", 0.06, 0.02),
        ("densidade_48_afiado_continuo", 0.06, 0.02),
        ("densidade_192_afiado_continuo", 0.06, 0.02),
        ("cilindro_afiado_continuo", 0.06, 0.02),
        ("afiado_valores_de_fabrica_separados", 0.08, 0.15),
        ("densidade_48_afiado_separados", 0.08, 0.15),
        ("densidade_192_afiado_separados", 0.08, 0.15),
        ("cilindro_afiado_separados", 0.08, 0.15),
    ] {
        let f = ler("produto", nome);
        let b = pincel_produto(&f);
        let n = passagens(&f);
        let fotos = arrastar(&f, &b, n);
        for (k, foto) in fotos.iter().enumerate() {
            let passagem = k + 1;
            let alvo = if passagem == n {
                Some(f.saida())
            } else {
                f.blocos.get(&format!("p{passagem}")).map(Vec::as_slice)
            };
            let Some(alvo) = alvo else { continue };
            let (nd, nw) = regua(&f, foto, b.radius);
            let (ad, aw) = regua(&f, alvo, b.radius);
            let dd = ((nd - ad) / ad).abs();
            let dw = ((nw - aw) / aw).abs();
            let nitidez = ((nd / nw - ad / aw) / (ad / aw)).abs();
            println!(
                "{nome} p{passagem}: D {nd:.4} contra {ad:.4} ({:.1} %) · W50 {nw:.3} contra \
                 {aw:.3} ({:.1} %)",
                dd * 100.0,
                dw * 100.0
            );
            assert!(
                dd <= barra_d,
                "{nome} p{passagem}: a profundidade desvia {:.1} % (barra {:.0} %)",
                dd * 100.0,
                barra_d * 100.0
            );
            assert!(
                dw <= barra_w,
                "{nome} p{passagem}: a largura desvia {:.1} % (barra {:.0} %)",
                dw * 100.0,
                barra_w * 100.0
            );
            assert!(
                nitidez <= barra_d + barra_w,
                "{nome} p{passagem}: a nitidez desvia {:.1} %",
                nitidez * 100.0
            );
        }
    }
}

/// ⭐⭐ **G-8 — a régua SEPARA o afiado do desenho comum** (o controlo).
///
/// ⛔ **Sem ele, um G-4 verde não afirma nada:** se a régua não distinguisse os
/// dois pincéis, ela estaria a medir uma grandeza em que eles coincidem.
#[test]
fn a_regua_separa_o_afiado_do_desenho_comum() {
    let a = ler("produto", "afiado_valores_de_fabrica_continuo");
    let d = ler("produto", "desenho_valores_de_fabrica_continuo");
    // ⚠️ **As duas saídas são do ALVO** — este gate mede a RÉGUA, não o nosso
    // motor: ele pergunta se ela vê a diferença entre os dois pincéis DELE.
    let (_, wa) = regua(&a, a.saida(), a.num("raio_efectivo_objecto"));
    let (_, wd) = regua(&d, d.saida(), d.num("raio_efectivo_objecto"));
    let razao = wd / wa;
    println!("G-8: W50 do desenho comum / do afiado = {razao:.2}");
    assert!(
        razao >= 1.8,
        "a régua deixou de ver a diferença: {razao:.2} (barra 1,8)"
    );
}

/// ⭐⭐ **G-5 — o PASSO do traço**: a contagem de dabs e as posições.
///
/// A lei é `⌊N/5⌋ + 1` dabs para um salto de `N` píxeis — o do pen-down mais um
/// por passo inteiro.
///
/// ⛔ **A fixtura de `5` px fica de FORA, e é uma divergência DECLARADA** (§15,
/// D-1): o `walk` da casa recusa um salto de **exactamente** um passo, com gate
/// próprio; num arrasto real as duas listas coincidem (o dab cai no mesmo sítio,
/// um evento depois).
#[test]
fn o_passo_do_traco_e_o_do_alvo() {
    const SALTOS: [u32; 13] = [0, 1, 4, 9, 10, 14, 15, 19, 20, 24, 25, 29, 30];
    let mut pior = 0.0f32;
    for n in SALTOS {
        let nome = format!("salto_{n:02}px");
        let f = ler("detector", &nome);
        let b = pincel_produto(&f);
        let px = f.num("vista_px_por_unidade");
        let passo = ph2d_sculpt3d::passo_do_traco(b.verb, b.radius * px);
        assert!(
            (passo - 5.0).abs() < 1e-4,
            "{nome}: o passo devia ser 5 px, e' {passo}"
        );
        // A contagem, pela porta do produto.
        let dabs = 1 + ph2d_sculpt3d::walk(
            [0.0, 0.0],
            [f32::from(u16::try_from(n).unwrap()), 0.0],
            passo,
        )
        .map_or(0, |p| p.into_iter().count());
        let esperado = 1 + (n / 5) as usize;
        assert_eq!(dabs, esperado, "{nome}: {dabs} dabs contra {esperado}");
        // E as posições.
        let fotos = arrastar_salto(&f, &b, n);
        let d = maior_distancia(&fotos, f.saida());
        assert!(d <= 7e-3, "{nome}: max|Δ| {d:.3e} passa a barra 7e-3");
        pior = pior.max(d);
    }
    println!("G-5: pior de {} saltos = {pior:.3e}", SALTOS.len());
}

/// O arrasto de UM salto: o pen-down e um evento de `n` píxeis.
fn arrastar_salto(f: &Fixtura, b: &Brush, n: u32) -> Vec<[f32; 3]> {
    let px = f.num("vista_px_por_unidade");
    let pd: Vec<f32> = f
        .chave("pixel_do_pen_down_no_mundo")
        .split_whitespace()
        .take(2)
        .map(|s| s.parse().expect("pixel do pen-down"))
        .collect();
    let um: Vec<f32> = f
        .chave("um_pixel_para_a_direita")
        .split_whitespace()
        .take(2)
        .map(|s| s.parse().expect("um pixel"))
        .collect();
    let passo = ph2d_sculpt3d::passo_do_traco(b.verb, b.radius * px);
    let mut m = malha(f);
    let e = olho(f);
    let mut s = SculptStroke::default();
    s.begin(&m);
    let carimbar = |m: &mut Mesh, s: &mut SculptStroke, t: f32| {
        let (x, y) = (pd[0] + um[0] * t, pd[1] + um[1] * t);
        if let Some(c) = na_superficie(m, x, y, e) {
            s.dab(m, b, &Dab::at(c, b.radius, e), Symmetry::default());
        }
    };
    carimbar(&mut m, &mut s, 0.0);
    if let Some(passos) = ph2d_sculpt3d::walk(
        [0.0, 0.0],
        [f32::from(u16::try_from(n).unwrap()), 0.0],
        passo,
    ) {
        for q in passos {
            carimbar(&mut m, &mut s, q[0]);
        }
    }
    m.positions().to_vec()
}

/// ⭐⭐ **G-5c — o dab do PEN-DOWN também é atenuado** (espec §5.3).
///
/// ⚠️ **A fixtura de controlo é a MESMA corrida sem a atenuação**, e é ela que
/// torna isto uma afirmação: sem o factor o mesmo dab desvia `4,5e-2`.
#[test]
fn o_dab_do_pen_down_e_atenuado() {
    let f = ler("detector", "salto_00px");
    let b = pincel_produto(&f);
    let nossa = arrastar_salto(&f, &b, 0);
    let d = maior_distancia(&nossa, f.saida());
    assert!(d <= 1e-5, "o dab do pen-down desvia {d:.3e}");
    // O controlo: a mesma corrida do alvo SEM a atenuação.
    let sem = ler("detector", "salto_00px_sem_atenuacao");
    let longe = maior_distancia(&nossa, sem.saida());
    assert!(
        longe >= 1e-2,
        "sem atenuação o alvo devia estar LONGE de nós: {longe:.3e}"
    );
    println!("G-5c: com atenuação {d:.3e} · sem ela {longe:.3e}");
}

/// ⭐⭐ **G-10 — a direcção é simétrica**: o traço com `Ctrl` é a imagem
/// espelhada do normal.
#[test]
fn a_direccao_e_simetrica() {
    let base = ler("produto", "afiado_valores_de_fabrica_continuo");
    let ctrl = ler("produto", "afiado_valores_de_fabrica_ctrl");
    let n = passagens(&ctrl);
    let nossa_base = arrastar(&base, &pincel_produto(&base), n);
    let nossa_ctrl = arrastar(&ctrl, &pincel_produto(&ctrl), n);
    let (a, b) = (&nossa_base[n - 1], &nossa_ctrl[n - 1]);
    // ⚠️ **Compara DESLOCAMENTOS e não posições** (errata do R-pré): os dois
    // vértices de canto das malhas de caixa dupla estão em `z = ±1`, e espelhar
    // a POSIÇÃO deles leria `2,0` sobre uma lei certa.
    let pior = base
        .repouso
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let da = a[i][2] - r[2];
            let db = b[i][2] - r[2];
            (da + db).abs()
        })
        .fold(0.0f32, f32::max);
    assert!(
        pior <= 2e-6,
        "o traço com Ctrl devia ser o espelho do normal: {pior:.3e}"
    );
}

/// ⭐⭐⭐ **G-11 — o recorte do cursor pela CAIXA ENVOLVENTE não se copia**
/// (espec §9.1).
///
/// No alvo, um acerto vivo que caia fora da caixa envolvente da peça é
/// **descartado**, e com uma caixa de espessura zero os dabs seguintes ao
/// primeiro **perdem-se**. ⛔ É um artefacto do programa e não uma lei: a nossa
/// corrida na MESMA malha tem de ficar perto da corrida dele na malha de caixa
/// grossa e **longe** da dele na fina.
#[test]
fn o_recorte_pela_caixa_nao_se_copia() {
    let fina = ler("artefacto_caixa", "caixa_fina_salto_14px");
    let grossa = ler("detector", "salto_14px");
    let nossa = arrastar_salto(&fina, &pincel_produto(&fina), 14);
    // ⚠️ Os dois vértices de canto são a ÚNICA diferença entre as duas malhas —
    // e é por isso que eles se saltam aqui.
    let sem_cantos = |a: &[[f32; 3]], b: &[[f32; 3]]| {
        a.iter()
            .zip(b)
            .zip(&grossa.repouso)
            .filter(|((_, _), r)| r[2].abs() < 0.5)
            .map(|((p, q), _)| (0..3).map(|k| (p[k] - q[k]).abs()).fold(0.0f32, f32::max))
            .fold(0.0f32, f32::max)
    };
    let perto = sem_cantos(&nossa, grossa.saida());
    let longe = sem_cantos(&nossa, fina.saida());
    println!("G-11: da corrida sem recorte {perto:.3e} · da recortada {longe:.3e}");
    assert!(
        perto <= 7e-3,
        "devíamos reproduzir a corrida SEM recorte: {perto:.3e}"
    );
    assert!(
        longe >= 1e-2,
        "não podemos reproduzir o artefacto do recorte: {longe:.3e}"
    );
}

/// **SONDA da tabela do §54** — as três leis do cursor sobre o corpus separado.
#[test]
#[ignore = "sonda: imprime a tabela do §54"]
fn diag_a_tabela_das_tres_leis() {
    for nome in [
        "afiado_valores_de_fabrica_separados",
        "densidade_48_afiado_separados",
        "densidade_192_afiado_separados",
        "cilindro_afiado_separados",
        "triangulos_afiado_separados",
    ] {
        let f = ler("produto", nome);
        let b = pincel_produto(&f);
        let n = passagens(&f);
        for lei in [
            LeiDoCursor::Ecra,
            LeiDoCursor::Passo,
            LeiDoCursor::PassoEBarro,
        ] {
            let fotos = arrastar_com(&f, &b, n, lei);
            for (k, foto) in fotos.iter().enumerate() {
                let passagem = k + 1;
                let alvo = if passagem == n {
                    Some(f.saida())
                } else {
                    f.blocos.get(&format!("p{passagem}")).map(Vec::as_slice)
                };
                let Some(alvo) = alvo else { continue };
                let (faixa, pontas) = desvio(&f, foto, alvo);
                println!("TAB {nome} {lei:?} p{passagem} faixa {faixa:.3e} pontas {pontas:.3e}");
            }
        }
    }
}
