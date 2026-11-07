//! **A exploração na tela** (`docs/SfxLab/05_W4_plano.md` §4): a barra, a faixa Heard e a grade do Explore. Só pinta e diz o que
//! se clica ([`AlvoE`]); quem decide é a família `ph2d-app-sfxlab`.

#![forbid(unsafe_code)]

use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_text_elided, rect_to_vello, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_sfx_explore::{Exploracao, FILHOS, Filho, Grade, TAMANHO_ROLL};
use ph2d_sfx_lab::job::Onda;
use ph2d_sfx_recipe::Como;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::{Affine, BezPath, Brush, Color, Stroke, VectorScene};

pub const BARRA_H: f32 = 40.0;
const CABECA_H: f32 = 28.0;
const LINHA_H: f32 = 24.0;
const LINHAS: f32 = 5.0;
pub const HEARD_H: f32 = CABECA_H + LINHAS * LINHA_H;
const PE_H: f32 = 36.0;
const PAD: f32 = 12.0;
const GAP: f32 = 8.0;
const RAIO: f32 = 6.0;
const BOTAO_H: f32 = 28.0;
const BOTAO_PAD: f32 = 12.0;
const TEXTO_PX: f32 = 13.0;
const PEQUENO_PX: f32 = 11.0;
const NUMERO_W: f32 = 20.0;
const ESTRELA: f32 = 14.0;
const FAIXA_H: f32 = 6.0;
const FAIXA_W: f32 = 180.0;
const ROTULO_W: f32 = 104.0;
const TRACO: f32 = 1.0;
const TRACO_ATUAL: f32 = 2.0;
const ONDA_PAD: f32 = 3.0;
/// As pontas de uma estrela, numa caixa de 1 × 1.
const PONTAS: [(f64, f64); 10] = [
    (0.5, 0.0),
    (0.62, 0.35),
    (1.0, 0.38),
    (0.7, 0.6),
    (0.81, 0.97),
    (0.5, 0.75),
    (0.19, 0.97),
    (0.3, 0.6),
    (0.0, 0.38),
    (0.38, 0.35),
];
/// Onde cada filho mora na grade 3 × 3: o som atual fica na célula do meio.
const CELULAS: [(usize, usize); FILHOS] = [
    (0, 0),
    (1, 0),
    (2, 0),
    (0, 1),
    (2, 1),
    (0, 2),
    (1, 2),
    (2, 2),
];

/// O que se clica.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlvoE {
    Roll,
    Mutate,
    Next,
    Keep,
    Lock,
    Explore,
    Atual,
    Filho(u8),
    /// O aviso da variação que estoura. Registrado antes da célula, que fica por cima: não tira o clique dela.
    Estoura(u8),
    Mudanca,
    SoKept,
    Ouvido(u32),
    Estrela(u32),
}

/// O destino dos cliques desta pintura.
pub type Alvos = Vec<(AlvoE, Rect)>;

struct P<'a> {
    scene: &'a mut VectorScene,
    ts: &'a mut TextSystem,
    theme: Theme,
    hover: Option<AlvoE>,
    alvos: &'a mut Alvos,
}

impl P<'_> {
    fn cor(&self, t: T) -> Color {
        resolve(t, self.theme)
    }

    fn caixa(&mut self, r: Rect, t: T) {
        let c = self.cor(t);
        fill_rounded_rect(self.scene, r, RAIO, c);
    }

    fn texto(&mut self, s: &str, x: f32, r: Rect, px: f32, t: T) {
        let alto = self.ts.layout("Ag", px, f32::INFINITY).height();
        let y = r.y + ((r.h - alto) / 2.0).max(0.0);
        let c = self.cor(t);
        paint_text_elided(
            self.ts,
            self.scene,
            s,
            x,
            y,
            px,
            (r.x + r.w - x).max(0.0),
            c,
        );
    }

    fn largura(&mut self, s: &str, px: f32) -> f32 {
        self.ts.prefix_width(s, px)
    }

    fn botao(&mut self, alvo: AlvoE, rotulo: &str, x: f32, y: f32, ativo: bool) -> f32 {
        let w = self.largura(rotulo, TEXTO_PX) + 2.0 * BOTAO_PAD;
        let r = Rect::new(x, y, w, BOTAO_H);
        let fundo = if self.hover == Some(alvo) && ativo {
            T::Bg3
        } else {
            T::Bg2
        };
        self.caixa(r, fundo);
        let tinta = if ativo { T::Text1 } else { T::TextDisabled };
        self.texto(rotulo, x + BOTAO_PAD, r, TEXTO_PX, tinta);
        self.alvos.push((alvo, r));
        x + w + GAP
    }
}

/// O nome que um som tem na tela: nunca um número gigante.
#[must_use]
pub fn nome_de(como: Como) -> String {
    match como {
        Como::Mao => tr("sfxlab.heard.edit").to_owned(),
        Como::Roll(n) => tr_with("sfxlab.heard.roll", &[("n", &n)]),
        Como::Mutate(n) => tr_with("sfxlab.heard.mutate", &[("n", &n)]),
        Como::Variacao(n) => tr_with("sfxlab.heard.variation", &[("n", &n)]),
    }
}

/// A onda fixa reamostrada para o retângulo: um par mínimo-máximo por px.
pub fn onda(scene: &mut VectorScene, o: &Onda, r: Rect, ganho: f32, cor: Color) {
    let n = o.colunas.len();
    let px = r.w.floor() as usize;
    if n == 0 || px == 0 {
        return;
    }
    let meio = r.y + r.h / 2.0;
    let meia = (r.h / 2.0 - ONDA_PAD).max(0.0);
    let nos: Vec<[f32; 2]> = (0..px)
        .map(|i| {
            let (de, ate) = (i * n / px, ((i + 1) * n / px).max(i * n / px + 1).min(n));
            o.colunas[de.min(n - 1)..ate]
                .iter()
                .fold([f32::INFINITY, f32::NEG_INFINITY], |[a, b], c| {
                    [a.min(c[0]), b.max(c[1])]
                })
        })
        .collect();
    let y = |v: f32| f64::from(meio - (v * ganho).clamp(-1.0, 1.0) * meia);
    let mut path = BezPath::new();
    let x = |i: usize| f64::from(r.x) + i as f64;
    path.move_to((x(0), y(nos[0][1])));
    for (i, c) in nos.iter().enumerate() {
        path.line_to((x(i), y(c[1]) - f64::from(TRACO) / 2.0));
    }
    for (i, c) in nos.iter().enumerate().rev() {
        path.line_to((x(i), y(c[0]) + f64::from(TRACO) / 2.0));
    }
    path.close_path();
    scene.fill_path(&path, &Brush::Solid(cor), Affine::IDENTITY);
}

/// Tudo o que a barra precisa saber do som atual.
pub struct Barra {
    pub como: Como,
    /// Há uma camada escolhida para o Lock.
    pub lock_ativo: bool,
    pub nivelado: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn barra(
    b: &Barra,
    r: Rect,
    hover: Option<AlvoE>,
    alvos: &mut Alvos,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let mut p = P {
        scene,
        ts,
        theme,
        hover,
        alvos,
    };
    let y = r.y + (r.h - BOTAO_H) / 2.0;
    let mut x = r.x + PAD;
    x = p.botao(AlvoE::Roll, tr("sfxlab.explore.roll"), x, y, true);
    x = p.botao(AlvoE::Mutate, tr("sfxlab.explore.mutate"), x, y, true);
    x = p.botao(AlvoE::Next, tr("sfxlab.explore.next"), x, y, true);
    x = p.botao(AlvoE::Keep, tr("sfxlab.explore.keep"), x, y, true);
    x = p.botao(AlvoE::Lock, tr("sfxlab.explore.lock"), x, y, b.lock_ativo);
    x = p.botao(AlvoE::Explore, tr("sfxlab.explore.open"), x, y, true);
    // Um som feito à mão não tem nome de sorteio: a barra só fala quando tem o que dizer.
    let dito = match (b.como, b.nivelado) {
        (Como::Mao, false) => return,
        (Como::Mao, true) => tr("sfxlab.explore.same_volume_only").to_owned(),
        (c, _) => tr_with("sfxlab.explore.same_volume", &[("name", &nome_de(c))]),
    };
    p.texto(&dito, x + GAP, r, TEXTO_PX, T::Text2);
}

/// O que a faixa mostra.
pub struct Faixa {
    pub so_kept: bool,
    pub rolagem: f32,
}

/// As linhas, da mais nova para a mais velha. Devolve o máximo da rolagem.
#[allow(clippy::too_many_arguments)]
pub fn heard(
    x: &Exploracao,
    f: &Faixa,
    r: Rect,
    hover: Option<AlvoE>,
    alvos: &mut Alvos,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) -> f32 {
    let mut p = P {
        scene,
        ts,
        theme,
        hover,
        alvos,
    };
    p.caixa(r, T::Bg2);
    let lista = &x.sessao.ouvidos;
    let cabeca = Rect::new(r.x, r.y, r.w, CABECA_H);
    let titulo = tr_with("sfxlab.heard.title", &[("n", &lista.len())]);
    p.texto(&titulo, r.x + PAD, cabeca, TEXTO_PX, T::Text1);
    let rotulo = tr("sfxlab.heard.kept_only");
    let w = p.largura(rotulo, PEQUENO_PX) + 2.0 * BOTAO_PAD;
    let botao = Rect::new(r.x + r.w - PAD - w, r.y, w, CABECA_H);
    p.texto(
        rotulo,
        botao.x + BOTAO_PAD,
        botao,
        PEQUENO_PX,
        if f.so_kept { T::Accent } else { T::Text2 },
    );
    p.alvos.push((AlvoE::SoKept, botao));

    let vistos: Vec<usize> = (0..lista.len())
        .rev()
        .filter(|i| !f.so_kept || lista[*i].estrela)
        .collect();
    let corpo = Rect::new(r.x, r.y + CABECA_H, r.w, (r.h - CABECA_H).max(0.0));
    if vistos.is_empty() {
        let dica = if f.so_kept {
            tr("sfxlab.heard.empty_kept")
        } else {
            tr("sfxlab.heard.empty")
        };
        p.texto(
            dica,
            r.x + PAD,
            Rect::new(r.x, corpo.y, r.w, LINHA_H),
            TEXTO_PX,
            T::Text3,
        );
        return 0.0;
    }
    let maximo = (vistos.len() as f32 * LINHA_H - corpo.h).max(0.0);
    let rolagem = f.rolagem.clamp(0.0, maximo);
    p.scene.push_clip(&rect_to_vello(corpo));
    for (k, i) in vistos.iter().enumerate() {
        let y = corpo.y + k as f32 * LINHA_H - rolagem;
        if y + LINHA_H < corpo.y || y > corpo.y + corpo.h {
            continue;
        }
        linha(
            &mut p,
            *i as u32,
            lista[*i].estrela,
            lista[*i].como,
            r,
            y,
            corpo,
        );
    }
    p.scene.pop_layer();
    maximo
}

fn linha(p: &mut P<'_>, i: u32, estrela: bool, como: Como, r: Rect, y: f32, corpo: Rect) {
    let toda = Rect::new(r.x, y, r.w, LINHA_H);
    if p.hover == Some(AlvoE::Ouvido(i)) {
        p.caixa(toda, T::Bg3);
    }
    let caixa = Rect::new(r.x + PAD, y + (LINHA_H - ESTRELA) / 2.0, ESTRELA, ESTRELA);
    desenha_estrela(p, caixa, estrela);
    let nome = nome_de(como);
    p.texto(&nome, caixa.x + ESTRELA + GAP, toda, TEXTO_PX, T::Text1);
    // Só o que está à vista é clicável: a lista rola por baixo do cabeçalho.
    let dentro = |q: Rect| {
        let (topo, fim) = (q.y.max(corpo.y), (q.y + q.h).min(corpo.y + corpo.h));
        Rect::new(q.x, topo, q.w, (fim - topo).max(0.0))
    };
    p.alvos.push((AlvoE::Ouvido(i), dentro(toda)));
    p.alvos.push((AlvoE::Estrela(i), dentro(caixa)));
}

fn desenha_estrela(p: &mut P<'_>, r: Rect, cheia: bool) {
    let mut path = BezPath::new();
    let ponto = |(a, b): (f64, f64)| {
        (
            f64::from(r.x) + a * f64::from(r.w),
            f64::from(r.y) + b * f64::from(r.h),
        )
    };
    path.move_to(ponto(PONTAS[0]));
    for q in &PONTAS[1..] {
        path.line_to(ponto(*q));
    }
    path.close_path();
    let cor = p.cor(if cheia { T::Accent } else { T::Text3 });
    if cheia {
        p.scene
            .fill_path(&path, &Brush::Solid(cor), Affine::IDENTITY);
    } else {
        p.scene.stroke_path(
            &Stroke::new(f64::from(TRACO)),
            Affine::IDENTITY,
            &Brush::Solid(cor),
            &path,
        );
    }
}

/// O que a grade mostra além da exploração.
pub struct VistaGrade<'a> {
    pub atual: Option<&'a Onda>,
    /// A variação a ser arrastada e onde a mão está.
    pub arrasto: Option<(usize, f32, f32)>,
}

/// A grade 3 × 3, com o som atual no centro, e o `Change` no pé.
#[allow(clippy::too_many_arguments)]
pub fn grade(
    g: &Grade,
    v: &VistaGrade<'_>,
    r: Rect,
    hover: Option<AlvoE>,
    alvos: &mut Alvos,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let mut p = P {
        scene,
        ts,
        theme,
        hover,
        alvos,
    };
    let alto = (r.h - PE_H).max(0.0);
    let cw = (r.w - 2.0 * GAP) / 3.0;
    let ch = (alto - 2.0 * GAP) / 3.0;
    let celula = |c: usize, l: usize| {
        Rect::new(
            r.x + c as f32 * (cw + GAP),
            r.y + l as f32 * (ch + GAP),
            cw,
            ch,
        )
    };
    let centro = celula(1, 1);
    p.caixa(centro, T::Selection);
    let cor = p.cor(T::Accent);
    stroke_rounded_rect(p.scene, centro, RAIO, TRACO_ATUAL, cor);
    if let Some(o) = v.atual {
        let tinta = p.cor(T::Accent);
        onda(p.scene, o, centro, 1.0, tinta);
    }
    let atual = tr("sfxlab.explore.current");
    p.texto(atual, centro.x + GAP, topo(centro), PEQUENO_PX, T::Text1);
    p.alvos.push((AlvoE::Atual, centro));

    for (i, (c, l)) in CELULAS.iter().enumerate() {
        filho(&mut p, g, i, celula(*c, *l));
    }
    if let Some((i, x, y)) = v.arrasto {
        let de = (centro.x + centro.w / 2.0, centro.y + centro.h / 2.0);
        let mut linha = BezPath::new();
        linha.move_to((f64::from(de.0), f64::from(de.1)));
        linha.line_to((f64::from(x), f64::from(y)));
        let cor = p.cor(T::Accent);
        p.scene.stroke_path(
            &Stroke::new(f64::from(TRACO_ATUAL)),
            Affine::IDENTITY,
            &Brush::Solid(cor),
            &linha,
        );
        let s = tr_with("sfxlab.explore.change_n", &[("n", &percent(g.tamanhos[i]))]);
        let q = Rect::new(x + GAP, y, FAIXA_W, LINHA_H);
        p.texto(&s, q.x, q, TEXTO_PX, T::Text1);
    }
    pe(&mut p, g, Rect::new(r.x, r.y + alto, r.w, PE_H));
}

fn topo(r: Rect) -> Rect {
    Rect::new(r.x, r.y, r.w, LINHA_H)
}

fn percent(t: f64) -> u32 {
    (t / TAMANHO_ROLL * 100.0).round() as u32
}

fn filho(p: &mut P<'_>, g: &Grade, i: usize, r: Rect) {
    let alvo = AlvoE::Filho(i as u8);
    let tocado = g.ultimo == Some(i);
    let fundo = if p.hover == Some(alvo) {
        T::Bg3
    } else {
        T::Bg2
    };
    p.caixa(r, fundo);
    if tocado {
        let cor = p.cor(T::Accent);
        stroke_rounded_rect(p.scene, r, RAIO, TRACO, cor);
    }
    if let Some(f) = &g.filhos[i] {
        let tinta = p.cor(if tocado { T::Accent } else { T::Text2 });
        onda(p.scene, &f.onda, r, 1.0, tinta);
    }
    let n = (i + 1).to_string();
    p.texto(&n, r.x + GAP, topo(r), TEXTO_PX, T::Text1);
    if g.filhos[i].as_ref().is_some_and(Filho::estoura) {
        let x = r.x + GAP + NUMERO_W;
        p.texto(tr("sfxlab.explore.clips"), x, topo(r), PEQUENO_PX, T::Warn);
        p.alvos.push((AlvoE::Estoura(i as u8), r));
    }
    if g.filhos[i].is_none() {
        let dito = if g.erro.is_some() {
            tr("sfxlab.explore.failed")
        } else {
            tr("sfxlab.explore.rendering")
        };
        p.texto(dito, r.x + GAP + NUMERO_W, topo(r), PEQUENO_PX, T::Text3);
    }
    p.alvos.push((alvo, r));
}

fn pe(p: &mut P<'_>, g: &Grade, r: Rect) {
    let rotulo = Rect::new(r.x, r.y, ROTULO_W, r.h);
    let s = tr_with("sfxlab.explore.change_n", &[("n", &percent(g.tamanho))]);
    p.texto(&s, r.x, rotulo, TEXTO_PX, T::Text1);
    let x = r.x + ROTULO_W + GAP;
    let meio = r.y + r.h / 2.0;
    let trilho = Rect::new(x, meio - FAIXA_H / 2.0, FAIXA_W, FAIXA_H);
    p.caixa(trilho, T::Bg0);
    let cheio = (g.tamanho / TAMANHO_ROLL).clamp(0.0, 1.0) as f32;
    p.caixa(Rect::new(x, trilho.y, trilho.w * cheio, FAIXA_H), T::Accent);
    p.alvos
        .push((AlvoE::Mudanca, Rect::new(x, r.y, FAIXA_W, r.h)));
    let dica = tr("sfxlab.explore.hint");
    let xd = x + FAIXA_W + 2.0 * GAP;
    p.texto(dica, xd, r, PEQUENO_PX, T::Text3);
}

/// De 0 a 1 ao longo do `Change`.
#[must_use]
pub fn tamanho_em(x: f32, trilho: Rect) -> f64 {
    let t = ((x - trilho.x) / trilho.w.max(1.0)).clamp(0.0, 1.0);
    f64::from(t) * TAMANHO_ROLL
}
