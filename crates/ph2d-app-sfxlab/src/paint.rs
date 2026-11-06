//! A tela: barra, régua, Mix e a pilha de camadas. Quem pinta também registra o que se clica (`Aba::alvos`).

use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_text_elided, paint_text_title_elided, rect_to_vello, resolve,
    stroke_rounded_rect,
};
use ph2d_editor_core::ruler::label_step;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_panel_sfxlab::{medidor, nome_da_camada};
use ph2d_sfx_lab::aprovar::EstadoDoAprovado;
use ph2d_sfx_recipe::{Layer, LayerId};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::{Affine, BezPath, Brush, Color, VectorScene};

use crate::aba::{Aba, Alvo, Gesto};
use crate::metrics as m;
use ph2d_sfx_lab::job::Onda;
use ph2d_sfx_lab::novo::Fonte;

struct P<'a> {
    scene: &'a mut VectorScene,
    ts: &'a mut TextSystem,
    theme: Theme,
}

impl P<'_> {
    fn c(&self, t: T) -> Color {
        resolve(t, self.theme)
    }

    fn fill(&mut self, r: Rect, t: T) {
        let cor = self.c(t);
        self.scene.fill_rect(rect_to_vello(r), cor);
    }

    fn caixa(&mut self, r: Rect, t: T) {
        let cor = self.c(t);
        fill_rounded_rect(self.scene, r, m::RADIUS, cor);
    }

    fn y_do_texto(&mut self, y: f32, h: f32, px: f32) -> f32 {
        let alto = self.ts.layout("Ag", px, f32::INFINITY).height();
        y + ((h - alto) / 2.0).max(0.0)
    }

    fn texto(&mut self, s: &str, x: f32, r: Rect, px: f32, t: T) {
        let y = self.y_do_texto(r.y, r.h, px);
        let cor = self.c(t);
        paint_text_elided(self.ts, self.scene, s, x, y, px, r.x + r.w - x, cor);
    }

    fn centrado(&mut self, s: &str, r: Rect, px: f32, t: T) {
        let w = self.ts.prefix_width(s, px);
        self.texto(s, r.x + ((r.w - w) / 2.0).max(0.0), r, px, t);
    }
}

pub(crate) fn aba(
    a: &mut Aba,
    area: Rect,
    over: Option<Rect>,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let mut p = P { scene, ts, theme };
    p.fill(area, T::Bg1);
    a.alvos.clear();

    let fundo = over.map_or(area.y + area.h, |o| o.y.min(area.y + area.h));
    a.corpo = Rect::new(area.x, area.y, area.w, fundo - area.y);
    let corpo = a.corpo;
    let esquerda = corpo.x + m::PAD;
    let x_pista = esquerda + m::STACK_W + m::GAP;
    let x_medidor = corpo.x + corpo.w - m::PAD - medidor::LARGURA;
    let w_pista = (x_medidor - m::GAP - x_pista).max(1.0);

    barra(a, &mut p, corpo);

    let y_regua = corpo.y + m::BAR_H;
    let regua_r = Rect::new(x_pista, y_regua, w_pista, m::RULER_H);
    a.pista = Rect::new(x_pista, y_regua, w_pista, fundo - y_regua);
    a.alvos.push((Alvo::Regua, regua_r));

    let y_mix = y_regua + m::RULER_H + m::ROW_GAP;
    let topo_linhas = y_mix + m::MIX_H + m::GAP;
    let passo = m::ROW_H + m::ROW_GAP;
    let conteudo = a.receita.layers.len() as f32 * passo;
    let base = fundo - m::PAD - m::ADD_H - m::GAP;
    let h_linhas = (base - topo_linhas).max(0.0);
    a.rolagem_max = (conteudo - h_linhas).max(0.0);
    a.rolagem = a.rolagem.min(a.rolagem_max);
    a.linhas = Rect::new(
        esquerda,
        topo_linhas,
        x_pista + w_pista - esquerda,
        h_linhas,
    );

    medidor::pinta(
        a.pronto.as_ref().map(|f| &f.medida),
        Rect::new(x_medidor, y_regua, medidor::LARGURA, fundo - y_regua),
        p.scene,
        p.ts,
        theme,
    );
    regua(a, &mut p, regua_r);
    mix(
        a,
        &mut p,
        esquerda,
        Rect::new(x_pista, y_mix, w_pista, m::MIX_H),
    );
    pilha(a, &mut p, esquerda, x_pista, w_pista);

    let y_add =
        topo_linhas + conteudo.min(h_linhas) + if conteudo > 0.0 { 0.0 } else { m::ROW_GAP };
    let add = Rect::new(esquerda, y_add, m::STACK_W, m::ADD_H);
    botao_add(a, &mut p, add);
    if conteudo == 0.0 {
        p.texto(
            tr("sfxlab.empty"),
            x_pista,
            Rect::new(x_pista, topo_linhas, w_pista, m::ROW_H),
            m::TEXT_PX,
            T::Text3,
        );
    }
    // A grade e o cursor descem da régua até a última camada, por cima das ondas.
    let y_fim = topo_linhas + (conteudo - m::ROW_GAP).clamp(0.0, h_linhas);
    grade(a, &mut p, regua_r, y_fim);
    cursor(a, &mut p, y_regua, y_fim);
    if a.menu_add {
        menu_add(a, &mut p, add, fundo);
    }
    if a.menu_objetos {
        menu_objetos(a, &mut p, corpo, fundo);
    }
}

fn botao(
    a: &mut Aba,
    p: &mut P<'_>,
    alvo: Alvo,
    rotulo: &str,
    x: f32,
    y: f32,
    ligado: Option<bool>,
) -> f32 {
    let marca = ligado.map_or(0.0, |_| m::CHECK + m::GAP);
    let w = p.ts.prefix_width(rotulo, m::TEXT_PX) + 2.0 * m::BUTTON_PAD_X + marca;
    let r = Rect::new(x, y, w, m::BUTTON_H);
    let cheio = if a.hover == Some(alvo) {
        T::Bg3
    } else {
        T::Bg2
    };
    p.caixa(r, cheio);
    if let Some(on) = ligado {
        let q = Rect::new(
            x + m::BUTTON_PAD_X,
            y + (m::BUTTON_H - m::CHECK) / 2.0,
            m::CHECK,
            m::CHECK,
        );
        let (dentro, borda) = if on {
            (T::Accent, T::Accent)
        } else {
            (T::Bg1, T::BorderStrong)
        };
        let cor = p.c(dentro);
        fill_rounded_rect(p.scene, q, m::CHECK_RADIUS, cor);
        let cor = p.c(borda);
        stroke_rounded_rect(p.scene, q, m::CHECK_RADIUS, m::STROKE, cor);
    }
    p.texto(rotulo, x + m::BUTTON_PAD_X + marca, r, m::TEXT_PX, T::Text1);
    a.alvos.push((alvo, r));
    x + w + m::GAP
}

fn barra(a: &mut Aba, p: &mut P<'_>, corpo: Rect) {
    let y = corpo.y + (m::BAR_H - m::BUTTON_H) / 2.0;
    let x0 = corpo.x + m::PAD;
    let linha = Rect::new(x0, corpo.y, m::NAME_W, m::BAR_H);
    let ty = p.y_do_texto(linha.y, linha.h, m::NAME_PX);
    let cor = p.c(T::Text1);
    let nome = a.nome.clone();
    paint_text_title_elided(p.ts, p.scene, &nome, x0, ty, m::NAME_PX, m::NAME_W, cor);
    // Os botões começam onde a pista começa: a barra e a pilha dividem a mesma grelha.
    let mut x = x0 + m::STACK_W + m::GAP;
    x = botao(a, p, Alvo::Play, tr("sfxlab.play"), x, y, None);
    x = botao(a, p, Alvo::Stop, tr("sfxlab.stop"), x, y, None);
    let on = a.play_on_change;
    x = botao(
        a,
        p,
        Alvo::PlayOnChange,
        tr("sfxlab.play_on_change"),
        x,
        y,
        Some(on),
    );
    let x_usar = x;
    x = botao(a, p, Alvo::UseInGame, tr("sfxlab.use_in_game"), x, y, None);
    a.x_usar = x_usar;
    aviso_da_barra(a, p, x, corpo);
}

/// Uma linha só, a mais urgente: o que impede de usar, o que mudou, e o estado do aprovado.
fn aviso_da_barra(a: &Aba, p: &mut P<'_>, x: f32, corpo: Rect) {
    let estouro = a.pronto.as_ref().is_some_and(|f| f.medida.clip());
    let (texto, tinta): (String, T) = if let Some(r) = &a.recusa {
        (r.clone(), T::Danger)
    } else if a.objeto_sumiu {
        (tr("sfxlab.use.object_gone").to_owned(), T::Warn)
    } else if estouro {
        (tr("sfxlab.warn.peak").to_owned(), T::Warn)
    } else {
        match a.estado_do_aprovado() {
            EstadoDoAprovado::Nenhum => return,
            EstadoDoAprovado::Aprovado => (tr("sfxlab.state.approved").to_owned(), T::Text2),
            EstadoDoAprovado::Mudou => (tr("sfxlab.state.changed").to_owned(), T::Warn),
            EstadoDoAprovado::MotorVelho => (tr("sfxlab.state.older_engine").to_owned(), T::Warn),
        }
    };
    let fim = corpo.x + corpo.w - m::PAD;
    p.texto(
        &texto,
        x,
        Rect::new(x, corpo.y, (fim - x).max(0.0), m::BAR_H),
        m::TEXT_PX,
        tinta,
    );
}

/// A lista de objetos do Use in Game, debaixo do botão; a roda anda quando ela não cabe.
fn menu_objetos(a: &mut Aba, p: &mut P<'_>, corpo: Rect, fundo: f32) {
    let vazia = a.objetos.is_empty();
    let titulo = tr("sfxlab.use.pick_object");
    let mut largura = p.ts.prefix_width(titulo, m::TEXT_PX);
    for (_, n) in &a.objetos {
        largura = largura.max(p.ts.prefix_width(n, m::TEXT_PX));
    }
    if vazia {
        largura = largura.max(p.ts.prefix_width(tr("sfxlab.use.no_objects"), m::TEXT_PX));
    }
    let largura = largura + 2.0 * m::BUTTON_PAD_X + 2.0 * m::MENU_PAD;
    let y = corpo.y + m::BAR_H;
    let linhas = a.objetos.len().max(1);
    let cabem = (((fundo - y - 2.0 * m::MENU_PAD) / m::MENU_ROW_H).floor() as usize)
        .saturating_sub(1)
        .max(1);
    let visiveis = linhas.min(cabem);
    a.menu_max = linhas - visiveis;
    a.menu_ini = a.menu_ini.min(a.menu_max);
    let h = (visiveis + 1) as f32 * m::MENU_ROW_H + 2.0 * m::MENU_PAD;
    let x = a.x_usar.min(corpo.x + corpo.w - largura).max(corpo.x);
    let caixa = Rect::new(x, y, largura, h);
    a.menu_caixa = caixa;
    p.caixa(caixa, T::BgElev);
    let cor = p.c(T::Border);
    stroke_rounded_rect(p.scene, caixa, m::RADIUS, m::STROKE, cor);
    let linha = |i: usize| {
        Rect::new(
            caixa.x + m::MENU_PAD,
            caixa.y + m::MENU_PAD + i as f32 * m::MENU_ROW_H,
            caixa.w - 2.0 * m::MENU_PAD,
            m::MENU_ROW_H,
        )
    };
    let r = linha(0);
    p.texto(titulo, r.x + m::BUTTON_PAD_X, r, m::TEXT_PX, T::Text3);
    a.alvos.push((Alvo::ObjetoOff, r));
    if vazia {
        let r = linha(1);
        let t = tr("sfxlab.use.no_objects");
        p.texto(t, r.x + m::BUTTON_PAD_X, r, m::TEXT_PX, T::TextDisabled);
        a.alvos.push((Alvo::ObjetoOff, r));
        return;
    }
    let vistos: Vec<(u64, String)> = a
        .objetos
        .iter()
        .skip(a.menu_ini)
        .take(visiveis)
        .cloned()
        .collect();
    for (i, (id, nome)) in vistos.into_iter().enumerate() {
        let r = linha(i + 1);
        let alvo = Alvo::Objeto(id);
        if a.hover == Some(alvo) {
            p.caixa(r, T::Bg3);
        }
        p.texto(&nome, r.x + m::BUTTON_PAD_X, r, m::TEXT_PX, T::Text1);
        a.alvos.push((alvo, r));
    }
}

fn grade(a: &Aba, p: &mut P<'_>, r: Rect, y_fim: f32) {
    let ms = f64::from(a.receita.seconds) * 1000.0;
    if ms <= 0.0 {
        return;
    }
    let px_por_ms = f64::from(r.w) / ms;
    let passo = label_step(px_por_ms);
    let baixo = r.y + r.h;
    let mut i = 1u32;
    while f64::from(i) * passo <= ms {
        let x = r.x + (f64::from(i) * passo * px_por_ms) as f32;
        p.fill(Rect::new(x, baixo, m::STROKE, y_fim - baixo), T::GridLine);
        i += 1;
    }
}

fn regua(a: &Aba, p: &mut P<'_>, r: Rect) {
    let ms = f64::from(a.receita.seconds) * 1000.0;
    if ms <= 0.0 {
        return;
    }
    let px_por_ms = f64::from(r.w) / ms;
    let passo = label_step(px_por_ms);
    let menor = passo / f64::from(m::RULER_DIVISIONS);
    let baixo = r.y + r.h;
    let mut i = 0u32;
    loop {
        let t = f64::from(i) * menor;
        if t > ms {
            break;
        }
        let x = r.x + (t * px_por_ms) as f32;
        if i.is_multiple_of(m::RULER_DIVISIONS) {
            p.fill(
                Rect::new(x, baixo - m::RULER_TICK, m::STROKE, m::RULER_TICK),
                T::Text3,
            );
            let v = (t * 1000.0).round() / 1000.0;
            let s = tr_with("sfxlab.ms", &[("n", &v)]);
            let caixa = Rect::new(x + m::ROW_GAP, r.y, r.x + r.w - x - m::ROW_GAP, r.h);
            p.texto(&s, caixa.x, caixa, m::SMALL_PX, T::Text2);
        } else {
            p.fill(
                Rect::new(x, baixo - m::RULER_MINOR, m::STROKE, m::RULER_MINOR),
                T::Text3,
            );
        }
        i += 1;
    }
}

fn mix(a: &Aba, p: &mut P<'_>, esquerda: f32, r: Rect) {
    let rotulo = Rect::new(esquerda, r.y, m::STACK_W, r.h);
    // Nenhuma camada escolhida: os cartões são os do som inteiro, e a linha do Mix mostra isso.
    p.caixa(
        rotulo,
        if a.sel.is_none() {
            T::Selection
        } else {
            T::Bg2
        },
    );
    p.texto(
        tr("sfxlab.mix"),
        esquerda + m::BUTTON_PAD_X,
        rotulo,
        m::TEXT_PX,
        T::Text1,
    );
    p.caixa(r, T::Bg2);
    if let Some(erro) = &a.erro {
        p.texto(erro, r.x + m::BUTTON_PAD_X, r, m::TEXT_PX, T::Danger);
        return;
    }
    if let Some(f) = &a.pronto {
        onda(p, &f.mix, r, 1.0, T::Text2);
    }
}

fn pilha(a: &mut Aba, p: &mut P<'_>, esquerda: f32, x_pista: f32, w_pista: f32) {
    let linhas = a.linhas;
    let passo = m::ROW_H + m::ROW_GAP;
    p.scene.push_clip(&rect_to_vello(linhas));
    let camadas: Vec<Layer> = a.receita.layers.clone();
    for (i, l) in camadas.iter().enumerate() {
        let y = linhas.y + i as f32 * passo - a.rolagem;
        if y + m::ROW_H < linhas.y || y > linhas.y + linhas.h {
            continue;
        }
        linha(a, p, l, esquerda, y);
        faixa(a, p, l, x_pista, w_pista, y);
    }
    p.scene.pop_layer();
}

fn linha(a: &mut Aba, p: &mut P<'_>, l: &Layer, x: f32, y: f32) {
    let r = Rect::new(x, y, m::STACK_W, m::ROW_H);
    let selecionada = a.sel == Some(l.id);
    p.caixa(r, if selecionada { T::Selection } else { T::Bg2 });
    a.alvos.push((Alvo::Linha(l.id), r));

    let alca = Rect::new(x + m::GAP, y, m::HANDLE_W, m::ROW_H);
    let meio = y + m::ROW_H / 2.0;
    for k in [-1.0, 0.0, 1.0] {
        p.fill(
            Rect::new(
                alca.x,
                meio + k * m::HANDLE_STEP - m::HANDLE_LINE / 2.0,
                m::HANDLE_W,
                m::HANDLE_LINE,
            ),
            T::Text3,
        );
    }
    a.alvos.push((Alvo::Alca(l.id), alca));

    let cor_nome = if l.muted { T::Text3 } else { T::Text1 };
    let x_nome = alca.x + alca.w + m::GAP;
    let ate = x + m::STACK_W - 2.0 * (m::TOGGLE + m::ROW_GAP) - m::GAP;
    let nome = nome_da_camada(&a.receita, l);
    p.texto(
        &nome,
        x_nome,
        Rect::new(x_nome, y, ate - x_nome, m::ROW_H),
        m::TEXT_PX,
        cor_nome,
    );

    let y_t = y + (m::ROW_H - m::TOGGLE) / 2.0;
    let s = Rect::new(
        x + m::STACK_W - m::GAP - m::TOGGLE,
        y_t,
        m::TOGGLE,
        m::TOGGLE,
    );
    let mu = Rect::new(s.x - m::ROW_GAP - m::TOGGLE, y_t, m::TOGGLE, m::TOGGLE);
    alterna(
        a,
        p,
        Alvo::Mudo(l.id),
        tr("sfxlab.layer.mute"),
        mu,
        l.muted,
        T::Warn,
    );
    let isolada = a.solo == Some(l.id);
    alterna(
        a,
        p,
        Alvo::Solo(l.id),
        tr("sfxlab.layer.solo"),
        s,
        isolada,
        T::Accent,
    );
}

fn alterna(a: &mut Aba, p: &mut P<'_>, alvo: Alvo, t: &str, r: Rect, on: bool, cor: T) {
    let cheio = if on {
        cor
    } else if a.hover == Some(alvo) {
        T::Bg3
    } else {
        T::Bg1
    };
    p.caixa(r, cheio);
    p.centrado(t, r, m::SMALL_PX, if on { T::AccentFg } else { T::Text2 });
    a.alvos.push((alvo, r));
}

fn faixa(a: &mut Aba, p: &mut P<'_>, l: &Layer, x_pista: f32, w_pista: f32, y: f32) {
    let pista = Rect::new(x_pista, y, w_pista, m::ROW_H);
    p.caixa(pista, T::Bg0);
    let pps = a.px_por_s();
    let x0 = x_pista + l.start * pps;
    let w = (a.receita.seconds - l.start).max(0.0) * pps;
    let bloco = Rect::new(x0, y, w, m::ROW_H);
    let silencio = a.solo.is_some_and(|s| s != l.id) || (l.muted && a.solo != Some(l.id));
    let selecionada = a.sel == Some(l.id);
    p.caixa(bloco, if selecionada { T::AccentSoft } else { T::Bg2 });
    if let Some(o) = a.pronto.as_ref().and_then(|f| f.camadas.get(&l.id)) {
        let tinta = if silencio { T::Text3 } else { T::Accent };
        onda(p, o, bloco, l.gain, tinta);
    }
    let falta = a
        .pronto
        .as_ref()
        .is_some_and(|f| f.pcm.missing.contains(&l.id));
    if falta {
        let x = x0 + m::BUTTON_PAD_X;
        p.texto(tr("sfxlab.missing"), x, bloco, m::TEXT_PX, T::Warn);
    }
    if !arrastando(a, l.id) {
        a.alvos.push((Alvo::Onda(l.id), bloco));
    } else {
        a.alvos.push((Alvo::Onda(l.id), pista));
    }
    if selecionada {
        let cor = p.c(T::Accent);
        stroke_rounded_rect(p.scene, bloco, m::RADIUS, m::STROKE, cor);
    }
}

fn arrastando(a: &Aba, id: LayerId) -> bool {
    matches!(a.gesto, Some(Gesto::Inicio { id: g, .. }) if g == id)
}

/// A onda fixa reamostrada para o retângulo: um par mínimo-máximo por px.
fn onda(p: &mut P<'_>, o: &Onda, r: Rect, ganho: f32, t: T) {
    let n = o.colunas.len();
    let px = r.w.floor() as usize;
    if n == 0 || px == 0 {
        return;
    }
    let meio = r.y + r.h / 2.0;
    let meia = (r.h / 2.0 - m::WAVE_PAD).max(0.0);
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
        path.line_to((x(i), y(c[1]) - f64::from(m::STROKE) / 2.0));
    }
    for (i, c) in nos.iter().enumerate().rev() {
        path.line_to((x(i), y(c[0]) + f64::from(m::STROKE) / 2.0));
    }
    path.close_path();
    let cor = p.c(t);
    p.scene
        .fill_path(&path, &Brush::Solid(cor), Affine::IDENTITY);
}

fn botao_add(a: &mut Aba, p: &mut P<'_>, r: Rect) {
    let on = a.menu_add || a.hover == Some(Alvo::Add);
    p.caixa(r, if on { T::Bg3 } else { T::Bg2 });
    p.texto(
        tr("sfxlab.add_layer"),
        r.x + m::BUTTON_PAD_X,
        r,
        m::TEXT_PX,
        T::Text1,
    );
    a.alvos.push((Alvo::Add, r));
}

fn cursor(a: &Aba, p: &mut P<'_>, y0: f32, y1: f32) {
    let x = a.pista.x + a.cursor * a.px_por_s();
    p.fill(
        Rect::new(x - m::CURSOR_W / 2.0, y0, m::CURSOR_W, y1 - y0),
        T::Accent,
    );
}

fn menu_add(a: &mut Aba, p: &mut P<'_>, add: Rect, fundo: f32) {
    let tem_material = a.takes.iter().next().is_some();
    let itens: Vec<(Fonte, &'static str, bool)> = Fonte::TODAS
        .iter()
        .map(|f| match f {
            Fonte::Noise => (*f, tr("sfxlab.layer.noise"), true),
            Fonte::Tone => (*f, tr("sfxlab.layer.tone"), true),
            Fonte::Body => (*f, tr("sfxlab.layer.body"), true),
            Fonte::Recording if tem_material => (*f, tr("sfxlab.layer.recording"), true),
            Fonte::Recording => (*f, tr("sfxlab.add.no_recording"), false),
        })
        .collect();
    let largura = itens
        .iter()
        .map(|(_, t, _)| p.ts.prefix_width(t, m::TEXT_PX))
        .fold(0.0, f32::max)
        + 2.0 * m::BUTTON_PAD_X
        + 2.0 * m::MENU_PAD;
    let h = itens.len() as f32 * m::MENU_ROW_H + 2.0 * m::MENU_PAD;
    let y = if add.y + add.h + m::ROW_GAP + h <= fundo {
        add.y + add.h + m::ROW_GAP
    } else {
        add.y - m::ROW_GAP - h
    };
    let caixa = Rect::new(add.x, y, largura, h);
    p.caixa(caixa, T::BgElev);
    let cor = p.c(T::Border);
    stroke_rounded_rect(p.scene, caixa, m::RADIUS, m::STROKE, cor);
    for (i, (f, t, ativo)) in itens.into_iter().enumerate() {
        let r = Rect::new(
            caixa.x + m::MENU_PAD,
            caixa.y + m::MENU_PAD + i as f32 * m::MENU_ROW_H,
            caixa.w - 2.0 * m::MENU_PAD,
            m::MENU_ROW_H,
        );
        let alvo = if ativo {
            Alvo::Item(f)
        } else {
            Alvo::ItemOff(f)
        };
        if ativo && a.hover == Some(alvo) {
            p.caixa(r, T::Bg3);
        }
        let tinta = if ativo { T::Text1 } else { T::TextDisabled };
        p.texto(t, r.x + m::BUTTON_PAD_X, r, m::TEXT_PX, tinta);
        a.alvos.push((alvo, r));
    }
}
