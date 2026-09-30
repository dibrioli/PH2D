//! Os gates da [`crate::tela_na_malha::BaseDaCadeia`] — a base de uma cadeia de
//! traços molhados (report do dono, 30/09: *«melhorou mas não curou
//! perfeitamente»*, sobre a marca clara que ficava depois de rodar a vista).
//!
//! ⚠️ **A fixtura tem de conter o fenómeno:** uma frente de água FINA (uma
//! coluna de um píxel) pintada numa vista, e a vista seguinte deslocada MEIO
//! píxel — é aí que o retrato da peça amostrado nos píxeis novos deixa de
//! descrever a amostra. Com a vista de antes, ou com uma frente larga, a lei de
//! 29/09 e a de hoje coincidem e o gate não mediria nada. O CONTROLO de cada
//! gate é a lei de 29/09 sobre a mesma peça.

use ph2d_mesh::{Face, Mesh};
use ph2d_mesh_colors::Tinta;

use crate::SculptStroke;
use crate::tela_na_malha::{BaseDaCadeia, Tela, TelaNaMalha, Vista};
use crate::tela_na_malha_tests::{ANTES, LADO, malha, vista};
use crate::tela_origem::origem;
use crate::tela_semente::{semente, semente_antes_da_cadeia};
use crate::tinta_fina::TintaDoTraco;

/// Ortográfica deslocada `dx` em NDC: um ponto cai `dx · LADO / 2` píxeis à direita.
fn deslocada(dx: f32) -> Vista {
    let m = [
        1.0, 0.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        dx, 0.0, 0.0, 1.0,
    ];
    Vista::nova(m, (LADO, LADO), [0.0, 0.0, 10.0])
}

/// Meio píxel para a direita.
fn rodada() -> Vista {
    deslocada(1.0 / LADO as f32)
}

/// A frente da água: a coluna `x` escurecida por cima de `base`.
fn com_frente(base: &[u8], x: u32) -> Vec<u8> {
    let mut t = base.to_vec();
    for y in 0..LADO {
        let o = ((y * LADO + x) * 4) as usize;
        t[o..o + 3].copy_from_slice(&[20, 20, 20]);
    }
    t
}

/// A faixa da tela que a água mudou — as colunas `40..66`, onde vivem as duas
/// frentes. ⚠️ Pousar a tela inteira em cada traço custa `~25 s` em debug no
/// plano fino (medido); o que o produto pousa é o rectângulo que o Painter
/// mudou, e é isto.
const FAIXA: [u32; 4] = [40, 0, 26, LADO];

/// A água passou e deixou a tela chapada: `F` em todo píxel.
const F: [u8; 3] = [100, 150, 200];

fn chapada() -> Vec<u8> {
    [F[0], F[1], F[2], 255].repeat((LADO * LADO) as usize)
}

/// A peça: com plano de tinta fina ou só a cor por vértice.
#[derive(Clone)]
struct Peca {
    m: Mesh,
    tinta: Option<Tinta>,
    /// O que o ÚLTIMO traço pôs na janela do desfazer (vértices ou amostras).
    tocou: Vec<u32>,
}

impl Peca {
    fn por_vertice() -> Self {
        // (o plano, abaixo: `20` quads de `5` píxeis a `8` amostras por lado.)
        // `200` quads em `100` píxeis: dois vértices por píxel — o detalhe fino
        // da peça é mais fino que o ecrã, que é o que a tinta fina é.
        Self {
            m: malha(200),
            tinta: None,
            tocou: Vec::new(),
        }
    }

    fn com_plano() -> Self {
        let m = malha(20);
        let faces = || m.faces().iter().map(Face::verts);
        let tinta = Tinta::semeada(m.colors().expect("pintada"), faces(), 8);
        Self {
            m,
            tinta: Some(tinta),
            tocou: Vec::new(),
        }
    }

    fn retrato(&self, v: &Vista) -> Vec<u8> {
        semente(&self.m, self.tinta.as_ref(), v)
    }

    fn destino(&self) -> usize {
        self.tinta
            .as_ref()
            .map_or(self.m.vert_count(), |t| t.amostras().len())
    }

    /// Um traço: abre, semeia, continua a cadeia, pousa `tela`, larga.
    fn traco(
        &mut self,
        v: Vista,
        semente: Vec<u8>,
        cadeia: Option<BaseDaCadeia>,
        tela: &[u8],
    ) -> BaseDaCadeia {
        self.traco_de(v, semente, cadeia, &[tela])
    }

    /// Um traço de VÁRIOS quadros: cada tela é pousada por ordem.
    fn traco_de(
        &mut self,
        v: Vista,
        semente: Vec<u8>,
        cadeia: Option<BaseDaCadeia>,
        telas: &[&[u8]],
    ) -> BaseDaCadeia {
        let mut s = SculptStroke::default();
        s.begin(&self.m);
        s.tinta_fina = self.tinta.take().map(|t| TintaDoTraco::nova(t, 0));
        let mut sessao = TelaNaMalha::nova(&self.m, v, self.destino_com(&s));
        sessao.com_semente(semente);
        if let Some(c) = cadeia {
            assert!(sessao.com_cadeia(c), "a cadeia é deste destino");
        }
        for tela in telas {
            let t = Tela {
                rgba: tela,
                largura: LADO,
                altura: LADO,
            };
            s.pousa_a_tela(&mut self.m, &mut sessao, &t, FAIXA);
        }
        self.tocou = s
            .tinta_fina
            .as_ref()
            .map_or_else(|| s.touched().to_vec(), |f| f.tocadas().to_vec());
        self.tinta = s.tinta_fina.take().map(TintaDoTraco::entregar);
        sessao.larga().1
    }

    fn destino_com(&self, s: &SculptStroke) -> usize {
        s.tinta_fina
            .as_ref()
            .map_or(self.m.vert_count(), |t| t.tinta().amostras().len())
    }

    /// A semente do traço que continua a cadeia na vista `nova` — a mesma porta
    /// do produto (`painter_semeia`).
    fn semente_na(&mut self, nova: &Vista, velha: &Vista, cadeia: &mut BaseDaCadeia) -> Vec<u8> {
        let mapa = origem(&self.m, nova, velha);
        cadeia.so_o_que_a_vista_levou(nova, &mapa);
        let mut fina = self.tinta.take().map(|t| TintaDoTraco::nova(t, 0));
        let r = semente_antes_da_cadeia(&mut self.m, fina.as_mut(), nova, cadeia);
        self.tinta = fina.map(TintaDoTraco::entregar);
        r
    }

    /// As cores do destino cujo ponto de ecrã na vista `v` cai no MIOLO da
    /// faixa (a orla dela é onde a bilinear lê de fora do que foi pousado).
    fn miolo(&self, v: &Vista) -> Vec<[f32; 3]> {
        let pos = self.m.positions();
        let dentro = |p: [f32; 3]| {
            v.ecra(p)
                .is_some_and(|[x, y]| (42.0..64.0).contains(&x) && (5.0..95.0).contains(&y))
        };
        match &self.tinta {
            None => {
                let cores = self.m.colors().expect("pintada");
                pos.iter()
                    .zip(cores)
                    .filter(|(p, _)| dentro(**p))
                    .map(|(_, c)| *c)
                    .collect()
            }
            Some(t) => {
                // O plano de uma grelha de quads: a amostra `(i, j)` da face
                // `f` está no ponto bilinear dos cantos dela.
                let mut out = Vec::new();
                for (fi, face) in self.m.faces().iter().enumerate() {
                    let c = face.verts();
                    let lado = t.lado_da_face(fi);
                    for j in 0..=lado {
                        for i in 0..=lado {
                            let (u, w) = (i as f32 / lado as f32, j as f32 / lado as f32);
                            let pk = |k: usize| pos[c[k] as usize];
                            let p: [f32; 3] = std::array::from_fn(|e| {
                                pk(0)[e] * (1.0 - u) * (1.0 - w)
                                    + pk(1)[e] * u * (1.0 - w)
                                    + pk(2)[e] * u * w
                                    + pk(3)[e] * (1.0 - u) * w
                            });
                            if dentro(p) {
                                out.push(t.amostras()[t.indice_quad(fi, c, i, j) as usize]);
                            }
                        }
                    }
                }
                out
            }
        }
    }
}

/// O pior desvio da cor chapada `F` sobre o miolo.
fn pior_desvio_de_f(cores: &[[f32; 3]]) -> f32 {
    let f = F.map(|b| f32::from(b) / 255.0);
    cores
        .iter()
        .flat_map(|c| (0..3).map(move |e| (c[e] - f[e]).abs()))
        .fold(0.0, f32::max)
}

/// ⭐⭐⭐ **Depois de rodar, a frente de antes não fica na peça.** A água passou
/// e a tela ficou chapada: a peça tem de ficar chapada. O CONTROLO é a lei de
/// 29/09 (semente = o retrato da peça COMO ESTÁ, base = a amostra), que deixa a
/// frente de antes desenhada — é a marca do report.
fn a_frente_de_antes_nao_fica(inicio: Peca) {
    let va = vista();
    let vb = rodada();
    let mut p = inicio;
    let sa = p.retrato(&va);
    let cadeia = p.traco(va, sa.clone(), None, &com_frente(&sa, 50));
    assert!(!cadeia.is_empty(), "a fixtura: a cadeia tocou a peça");

    // O CONTROLO: a lei de 29/09.
    let mut controlo = p.clone();
    let s_como_esta = controlo.retrato(&vb);
    controlo.traco(vb, s_como_esta, None, &chapada());
    let marca = pior_desvio_de_f(&controlo.miolo(&vb));
    assert!(
        marca > 0.1,
        "a fixtura não contém o fenómeno: a lei de 29/09 só desvia {marca}"
    );

    // A lei da cadeia.
    let mut cadeia = cadeia;
    let s0 = p.semente_na(&vb, &va, &mut cadeia);
    p.traco(vb, s0, Some(cadeia), &chapada());
    let resto = pior_desvio_de_f(&p.miolo(&vb));
    assert!(
        resto <= 1.0 / 255.0,
        "a frente de antes ficou na peça: {resto} (a lei de 29/09 deixava {marca})"
    );
}

#[test]
fn depois_de_rodar_a_frente_de_antes_nao_fica_na_cor_por_vertice() {
    a_frente_de_antes_nao_fica(Peca::por_vertice());
}

#[test]
fn depois_de_rodar_a_frente_de_antes_nao_fica_na_tinta_fina() {
    a_frente_de_antes_nao_fica(Peca::com_plano());
}

/// ⭐⭐ **Na MESMA vista a cadeia é a lei de antes, somada** — reaproveitar a
/// tela semeada com a última tela (a lei antiga) e continuar a cadeia semeada
/// com o retrato de antes dela (a de hoje) dão a mesma peça. E quando a água SAI
/// de um sítio (a tela volta à semente ali, exactamente), a amostra volta ao que
/// era: a lei antiga fazia-o com `c − c₁ ≠ 0`, a de hoje com a amostra a ser da
/// cadeia mesmo sem diferença.
fn na_mesma_vista_e_a_lei_de_antes(inicio: Peca) {
    let va = vista();
    let mut p = inicio;
    let sa = p.retrato(&va);
    let c1 = com_frente(&sa, 50);
    let cadeia = p.traco(va, sa.clone(), None, &c1);
    let depois_do_1 = p.clone();

    for (nome, c2) in [
        ("a água espalhou", com_frente(&c1, 60)),
        ("a água saiu", sa.clone()),
    ] {
        let mut antiga = depois_do_1.clone();
        antiga.traco(va, c1.clone(), None, &c2);
        let mut hoje = depois_do_1.clone();
        hoje.traco(va, sa.clone(), Some(cadeia.clone()), &c2);
        let (a, h) = (antiga.miolo(&va), hoje.miolo(&va));
        let pior = a
            .iter()
            .zip(&h)
            .flat_map(|(x, y)| (0..3).map(move |e| (x[e] - y[e]).abs()))
            .fold(0.0f32, f32::max);
        assert!(pior <= 1e-5, "{nome}: as duas leis discordam {pior}");
    }
    // O CONTROLO: «a água saiu» devolve a peça de antes — sem isto as duas
    // leis podiam concordar em não fazer nada.
    let mut hoje = depois_do_1;
    hoje.traco(va, sa.clone(), Some(cadeia), &sa);
    let antes = ANTES;
    let pior = hoje
        .miolo(&va)
        .iter()
        .flat_map(|c| (0..3).map(move |e| (c[e] - antes[e]).abs()))
        .fold(0.0f32, f32::max);
    assert!(
        pior <= 1.0 / 255.0,
        "a água saiu e a peça não voltou: {pior}"
    );
}

#[test]
fn na_mesma_vista_a_cadeia_e_a_lei_de_antes_por_vertice() {
    na_mesma_vista_e_a_lei_de_antes(Peca::por_vertice());
}

#[test]
fn na_mesma_vista_a_cadeia_e_a_lei_de_antes_na_tinta_fina() {
    na_mesma_vista_e_a_lei_de_antes(Peca::com_plano());
}

/// **Sem cor a comparar, a amostra da cadeia FICA** — um píxel apagado não
/// devolve a amostra à cor de antes da cadeia (seria apagar a tinta por falta
/// de informação), nem num quadro seguinte do mesmo traço: ali ela volta à cor
/// de antes do TRAÇO, que é a lei de sempre — e o traço apagado NÃO TOCA nada
/// (a cerca de `na_cadeia` pergunta pela mistura, e sem ela a amostra entrava
/// na janela do desfazer com a cor de sempre). O CONTROLO é a mesma tela sem o
/// apagão, onde a água sai e ela volta.
#[test]
fn sem_cor_a_comparar_a_amostra_da_cadeia_fica() {
    let va = vista();
    let mut p = Peca::por_vertice();
    let sa = p.retrato(&va);
    let cadeia = p.traco(va, sa.clone(), None, &com_frente(&sa, 50));
    let escura = |p: &Peca| {
        p.miolo(&va)
            .iter()
            .filter(|c| c[0] < ANTES[0] - 0.05)
            .count()
    };
    let antes = escura(&p);
    assert!(antes > 0, "a fixtura: a frente está na peça");

    let mut apagada = sa.clone();
    for y in 0..LADO {
        for x in 45..56 {
            apagada[((y * LADO + x) * 4 + 3) as usize] = 0;
        }
    }
    let mut q = p.clone();
    q.traco(va, sa.clone(), Some(cadeia.clone()), &apagada);
    assert_eq!(
        escura(&q),
        antes,
        "o apagão devolveu a tinta à cor de antes"
    );
    // ⚠️ E não TOCOU a cadeia: sem cor a comparar a amostra dela nem entra na
    // janela do desfazer (a cor não mudaria — o que se pagaria é a captura, o
    // upload e um passo de undo vazio). ⚠️ Só a CADEIA: na orla do apagão a
    // bilinear mistura píxeis apagados e vivos, e ali a diferença é real.
    let da_cadeia = |q: &Peca| q.tocou.iter().filter(|&&i| cadeia.contem(i)).count();
    assert_eq!(
        da_cadeia(&q),
        0,
        "o apagão pôs amostras da cadeia na janela do desfazer"
    );
    // O mesmo apagão no 2.º quadro de um traço que já tocou a frente.
    let mut q = p.clone();
    let espalhou = com_frente(&com_frente(&sa, 50), 51);
    q.traco_de(va, sa.clone(), Some(cadeia.clone()), &[&espalhou, &apagada]);
    assert_eq!(
        escura(&q),
        antes,
        "o apagão a meio do traço devolveu a tinta à cor de antes da cadeia"
    );

    p.traco(va, sa.clone(), Some(cadeia.clone()), &sa);
    assert_eq!(
        escura(&p),
        0,
        "o CONTROLO: sem o apagão a água sai e a peça volta"
    );
    assert!(
        da_cadeia(&p) > 0,
        "o CONTROLO da régua: aí a cadeia é tocada"
    );
}

/// ⭐ **Rodar deixa na cadeia só o que a vista nova herdou** — uma amostra cujo
/// píxel novo não tem origem, ou que a vista nova não mostra, sai: a água dela
/// não viajou, e a tinta que pousou fica na peça.
#[test]
fn rodar_deixa_na_cadeia_so_o_que_a_vista_nova_herdou() {
    let va = vista();
    let mut p = Peca::por_vertice();
    let sa = p.retrato(&va);
    let cadeia = p.traco(va, sa.clone(), None, &com_frente(&sa, 50));
    let n = cadeia.len();
    assert!(n > 0);

    let tudo_visto = vec![Some([0.0, 0.0]); (LADO * LADO) as usize];
    let nada_visto = vec![None; (LADO * LADO) as usize];

    let mut c = cadeia.clone();
    c.so_o_que_a_vista_levou(&va, &tudo_visto);
    assert_eq!(c.len(), n, "com origem em todo píxel, nada sai");

    let mut c = cadeia.clone();
    c.so_o_que_a_vista_levou(&va, &nada_visto);
    assert!(c.is_empty(), "sem origem nenhuma, tudo sai");

    let mut c = cadeia;
    c.so_o_que_a_vista_levou(&deslocada(10.0), &tudo_visto);
    assert!(c.is_empty(), "o que a vista nova não mostra sai");
}

/// **O retrato sem a cadeia devolve a peça AO BIT** — a troca desfaz-se antes
/// de voltar. O CONTROLO: o retrato sem a cadeia não é o da peça como está.
#[test]
fn o_retrato_sem_a_cadeia_nao_mexe_na_peca() {
    let va = vista();
    for inicio in [Peca::por_vertice(), Peca::com_plano()] {
        let mut p = inicio;
        let sa = p.retrato(&va);
        let mut cadeia = p.traco(va, sa.clone(), None, &com_frente(&sa, 50));
        let antes = p.clone();
        let sem = p.semente_na(&va, &va, &mut cadeia);
        assert_eq!(p.m.colors(), antes.m.colors(), "a cor por vértice mudou");
        assert_eq!(
            p.tinta.as_ref().map(Tinta::amostras),
            antes.tinta.as_ref().map(Tinta::amostras),
            "o plano mudou"
        );
        assert_eq!(sem, sa, "o retrato sem a cadeia é o de antes dela");
        assert_ne!(
            p.retrato(&va),
            sa,
            "o CONTROLO: a peça como está tem a frente"
        );
        assert_eq!(cadeia.destino(), p.destino());
    }
}

/// **Uma cadeia de outro destino é recusada** — o plano mudou de degrau entre
/// dois traços, e os índices dela já não são destas amostras.
#[test]
fn uma_cadeia_de_outro_destino_e_recusada() {
    let m = malha(4);
    let mut sessao = TelaNaMalha::nova(&m, vista(), m.vert_count());
    assert!(
        !sessao.com_cadeia(BaseDaCadeia::vazia(m.vert_count() + 1)),
        "uma cadeia de outro destino entrou"
    );
    assert_eq!(sessao.cadeia().destino(), m.vert_count());
    assert!(
        sessao.com_cadeia(BaseDaCadeia::vazia(m.vert_count())),
        "o CONTROLO: a do mesmo destino entra"
    );
}
