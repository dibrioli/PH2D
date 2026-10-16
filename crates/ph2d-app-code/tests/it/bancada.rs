//! ⏱ **E14 — a bancada tecla→pixel** (MEDIÇÃO, não é gate). Tabela e orçamentos:
//! `docs/CodeEditor/02_onda1.md` §Medições (E14).
//!
//! Por TECLA, o caminho do app: `DocumentSurface::key` (buffer + `AutoEdit` + reparse incremental) →
//! [cena a tocar: `scene_sync` + `scene_tick` do `ScriptHost`] → `ph2d_app_code::frame` → `paint` da
//! aba numa `VectorScene` (problemas, dobras, realce das linhas visíveis) → `VelloPass` na placa →
//! cópia de UMA faixa do alvo (a linha do cursor) para a memória, onde se confirma o glifo novo. Fora
//! fica o compositor do app e o ecrã do sistema (espera do vsync, apresentação).
//!
//! Variantes no MESMO processo, intercaladas em blocos rodados (`docs/DevOps/MEDIR_VELOCIDADE.md`):
//! cena parada × a tocar × 51 linhas (`bob.luau`) × 7 500 (o mesmo, juntado) × vista de ~43 linhas
//! (a janela do dono) × 100 linhas (o critério de morte). À parte, na mesma rodada: Ctrl+S → selo
//! (N = 3), e o que a gramática faz por tecla (reparse, dobras, erros, realce).
//!
//! Corre (compila no `smoke` e corre na placa, com a exclusão dela):
//! `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-app-code --test it bancada -- --ignored --nocapture`

use crate::Mesa;
use ph2d_code_buffer::{Buffer, Range as Cursor, Selection};
use ph2d_code_syntax::Syntax;
use ph2d_editor_core::doc_surface::{DocumentSurface, Key};
use ph2d_editor_core::zones::Rect;
use ph2d_gpu::GpuContext;
use ph2d_host::Modifiers;
use ph2d_panel_code::Badge;
use ph2d_render::VelloPass;
use ph2d_script::ScriptId;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::{Color, VectorScene};
use std::hint::black_box;
use std::time::Instant;

const BOB_FIXTURA: &str = include_str!("../../../ph2d-code-syntax/tests/it/fixtures/bob.luau");
const GRANDE: usize = 7_500;
/// A linha (0-based) onde se escreve: `  self.t = self.t + dt` do `update`; no grande, a mesma linha
/// a meio do ficheiro (cópia 72 de 147).
const ALVO_BOB: usize = 30;
const ALVO_GRANDE: usize = ALVO_BOB + 51 * 72;
const RODADAS: usize = 10;
const BLOCO: usize = 25;
/// Quadros sem tecla entre duas teclas (a 60 Hz e ~10 teclas/s há ~6); pintados e na placa.
const ENTRE: usize = 2;
const LETRAS: [char; 5] = ['x', 'w', 'm', 'z', 'k'];
const LARGURA: f32 = 1600.0;
const JANELA: usize = 43;
const MORTE: usize = 100;
/// Diferença (soma dos três canais) a partir da qual um píxel tem tinta contra o fundo da linha.
const TINTA: i32 = 60;

fn bob() -> String {
    BOB_FIXTURA
        .lines()
        .skip(2)
        .map(|l| format!("{l}\n"))
        .collect()
}

/// `bob.luau` juntado até `n` linhas. O motor recusa uma propriedade declarada duas vezes (medido:
/// `Refused` na linha 58, a 1.ª declaração da 2.ª cópia), então só a 1.ª cópia as declara; nas
/// outras a linha fica comentada — o mesmo número de linhas e o mesmo texto à volta do alvo.
fn juntado(n: usize) -> String {
    let b = bob();
    let por = b.lines().count();
    b.lines()
        .cycle()
        .take(n)
        .enumerate()
        .map(|(i, l)| {
            if i >= por && l.starts_with("ph2d.property(") {
                format!("-- {l}\n")
            } else {
                format!("{l}\n")
            }
        })
        .collect()
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1e3
}

fn loadavg() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .unwrap_or_default()
        .split_whitespace()
        .take(3)
        .collect::<Vec<_>>()
        .join(" ")
}

/// `min / p50 / p99 / p99.9` em ms.
fn resumo(v: &[f64]) -> String {
    if v.is_empty() {
        return "—".into();
    }
    let mut s = v.to_vec();
    s.sort_by(f64::total_cmp);
    let q = |p: f64| s[((s.len() - 1) as f64 * p).round() as usize];
    format!(
        "{:.2} / {:.2} / {:.2} / {:.2}",
        s[0],
        q(0.5),
        q(0.99),
        q(0.999)
    )
}

fn p50(v: &[f64]) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(f64::total_cmp);
    s[(s.len() - 1) / 2]
}

struct Placa {
    gpu: GpuContext,
    passes: Vec<((u32, u32), VelloPass)>,
}

impl Placa {
    fn pass(&mut self, size: (u32, u32)) -> usize {
        if let Some(i) = self.passes.iter().position(|p| p.0 == size) {
            return i;
        }
        let p =
            VelloPass::new(&self.gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, size).expect("vello");
        self.passes.push((size, p));
        self.passes.len() - 1
    }

    fn render(&mut self, i: usize, scene: &VectorScene) {
        let (size, pass) = &mut self.passes[i];
        pass.render_to_intermediate(&self.gpu, scene.inner(), *size, Color::TRANSPARENT)
            .expect("render");
    }

    fn espera(&self) {
        let _ = self.gpu.device.poll(wgpu::PollType::wait_indefinitely());
    }

    /// A faixa de linhas `y0..y0+h` do alvo, RGBA8 compacto (`w * 4` por linha).
    fn faixa(&self, i: usize, y0: u32, h: u32) -> (u32, u32, Vec<u8>) {
        let tex = self.passes[i].1.intermediate_texture();
        let w = tex.width();
        let h = h.min(tex.height().saturating_sub(y0)).max(1);
        let unpadded = w * 4;
        let padded = unpadded.div_ceil(256) * 256;
        let dev = &self.gpu.device;
        let buf = dev.create_buffer(&wgpu::BufferDescriptor {
            label: Some("e14 faixa"),
            size: u64::from(padded) * u64::from(h),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut enc = dev.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: tex,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: y0, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buf,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(h),
                },
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        self.gpu.queue.submit([enc.finish()]);
        let slice = buf.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.espera();
        rx.recv().expect("recv").expect("map");
        let view = slice.get_mapped_range();
        let mut out = Vec::with_capacity((unpadded * h) as usize);
        for r in 0..h as usize {
            let a = r * padded as usize;
            out.extend_from_slice(&view[a..a + unpadded as usize]);
        }
        drop(view);
        buf.unmap();
        (w, h, out)
    }
}

/// Píxeis com tinta nas colunas `x0..x1` da faixa, contra o píxel `x_fundo` de cada linha (o fundo
/// da MESMA linha: o realce da linha do cursor conta como fundo).
fn tinta(faixa: &(u32, u32, Vec<u8>), x0: f32, x1: f32, x_fundo: f32) -> usize {
    let (w, h, px) = faixa;
    let at = |r: u32, c: u32| {
        let i = ((r * w + c.min(w - 1)) * 4) as usize;
        [i32::from(px[i]), i32::from(px[i + 1]), i32::from(px[i + 2])]
    };
    let (c0, c1, cf) = (
        x0.ceil().max(0.0) as u32,
        x1.floor().max(0.0) as u32,
        x_fundo as u32,
    );
    let mut n = 0;
    for r in 0..*h {
        let f = at(r, cf);
        for c in c0..c1.min(*w) {
            let p = at(r, c);
            if (p[0] - f[0]).abs() + (p[1] - f[1]).abs() + (p[2] - f[2]).abs() > TINTA {
                n += 1;
            }
        }
    }
    n
}

fn tecla(
    m: &mut Mesa,
    ts: &mut TextSystem,
    id: ScriptId,
    key: Key,
    mods: Modifiers,
    text: Option<&str>,
) {
    let _ = m.docs().key(id.0, ts, key, mods, true, text, None);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cena {
    Parada,
    Tocando,
}

#[derive(Default)]
struct Amostras {
    total: Vec<f64>,
    key: Vec<f64>,
    quadro: Vec<f64>,
    paint: Vec<f64>,
    placa: Vec<f64>,
    a11y: Vec<f64>,
    quente: Vec<f64>,
}

struct Celula {
    cena: Cena,
    linhas: usize,
    vista: usize,
    m: Mesa,
    id: ScriptId,
    area: Rect,
    pass: usize,
    /// O x do cursor no fim da linha-alvo antes de escrever, a largura de um glifo e a faixa da linha.
    x0: f32,
    adv: f32,
    band: (u32, u32),
    k: usize,
    visiveis: usize,
    primeiro: f64,
    s: Amostras,
}

/// Um projeto com o documento de `linhas` aberto, os três objectos a correr ESSE script, e a VM.
fn mesa(linhas: usize) -> (Mesa, ScriptId) {
    let texto = if linhas == 51 { bob() } else { juntado(GRANDE) };
    let (m, ids) = Mesa::new(&[("bob.luau", &texto)]);
    let id = ids[0];
    let mut m = m.com_cena(id, 3);
    m.cena();
    m.pede(id);
    assert_eq!(
        m.docs().text_of(id).map(|t| t.lines().count()),
        Some(linhas)
    );
    (m, id)
}

fn pinta(
    m: &mut Mesa,
    id: ScriptId,
    area: Rect,
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    theme: &Theme,
) {
    scene.reset();
    m.docs().paint(id.0, area, scene, ts, theme);
}

/// A maior altura de área em que a vista do documento grande mostra `rows` linhas.
fn altura_para(rows: usize, ts: &mut TextSystem, theme: &Theme) -> f32 {
    let (mut m, id) = mesa(GRANDE);
    let mut scene = VectorScene::new();
    let (mut lo, mut hi) = (40.0_f32, 4000.0_f32);
    while hi - lo > 0.5 {
        let mid = ((lo + hi) * 0.5).floor();
        pinta(
            &mut m,
            id,
            Rect::new(0.0, 0.0, LARGURA, mid),
            ts,
            &mut scene,
            theme,
        );
        let n = m.docs().visible_lines(id).map_or(0, |r| r.len());
        if n <= rows {
            lo = mid;
        } else {
            hi = mid;
        }
        if hi - lo <= 1.0 {
            break;
        }
    }
    lo
}

impl Celula {
    fn nome(&self) -> String {
        format!(
            "{} · {} · {}",
            if self.cena == Cena::Tocando {
                "a tocar"
            } else {
                "parada"
            },
            if self.linhas == 51 {
                "bob.luau (51)"
            } else {
                "7 500"
            },
            if self.vista == MORTE {
                "100 linhas"
            } else {
                "~43 linhas"
            }
        )
    }

    fn nova(
        cena: Cena,
        linhas: usize,
        vista: usize,
        altura: f32,
        ts: &mut TextSystem,
        placa: &mut Placa,
        theme: &Theme,
    ) -> Self {
        let (mut m, id) = mesa(linhas);
        let area = Rect::new(0.0, 0.0, LARGURA, altura);
        let pass = placa.pass((LARGURA as u32, altura as u32));
        let mut scene = VectorScene::new();
        // O 1.º quadro (frio: layout de cada linha, glifos novos para o Vello) é um evento próprio.
        let t = Instant::now();
        pinta(&mut m, id, area, ts, &mut scene, theme);
        placa.render(pass, &scene);
        let _ = placa.faixa(pass, 0, 1);
        let primeiro = ms(t);
        let visiveis = m.docs().visible_lines(id).map_or(0, |r| r.len());
        // O cursor no fim da linha-alvo, pelo teclado.
        let alvo = if linhas == 51 { ALVO_BOB } else { ALVO_GRANDE };
        tecla(&mut m, ts, id, Key::Home, crate::ctrl(), None);
        for _ in 0..alvo {
            tecla(&mut m, ts, id, Key::Down, Modifiers::default(), None);
        }
        tecla(&mut m, ts, id, Key::End, Modifiers::default(), None);
        assert_eq!(m.docs().cursor_of(id).map(|c| c.0), Some(alvo));
        let mut c = Self {
            cena,
            linhas,
            vista,
            m,
            id,
            area,
            pass,
            x0: 0.0,
            adv: 0.0,
            band: (0, 0),
            k: 0,
            visiveis,
            primeiro,
            s: Amostras::default(),
        };
        // Controlo da régua: antes de escrever NÃO há tinta onde o glifo vai nascer; depois há.
        c.quadro(ts, placa, &mut scene, theme, false);
        let r0 = c.m.docs().caret_area(id.0).expect("cursor à vista");
        c.band = (r0.y.floor() as u32, r0.h.ceil() as u32);
        let antes = placa.faixa(pass, c.band.0, c.band.1);
        tecla(
            &mut c.m,
            ts,
            id,
            Key::Char('x'),
            Modifiers::default(),
            Some("x"),
        );
        c.quadro(ts, placa, &mut scene, theme, false);
        let depois = placa.faixa(pass, c.band.0, c.band.1);
        let r1 = c.m.docs().caret_area(id.0).expect("cursor à vista");
        c.adv = r1.x - r0.x;
        assert!(c.adv > 2.0, "um glifo avança o cursor: {} → {}", r0.x, r1.x);
        let fundo = r1.x + 4.0 * c.adv;
        let vazio = tinta(&antes, r0.x + 3.0, r1.x - 1.0, fundo);
        let cheio = tinta(&depois, r0.x + 1.0, r1.x - 1.0, fundo);
        assert!(
            vazio == 0 && cheio >= 4,
            "{}: a régua não distingue o glifo (antes {vazio}, depois {cheio})",
            c.nome()
        );
        tecla(&mut c.m, ts, id, Key::Backspace, Modifiers::default(), None);
        c.quadro(ts, placa, &mut scene, theme, false);
        c.x0 = r0.x;
        c
    }

    /// Um quadro sem leitura: [a cena dá um passo] → família → pintura → placa.
    fn quadro(
        &mut self,
        ts: &mut TextSystem,
        placa: &mut Placa,
        scene: &mut VectorScene,
        theme: &Theme,
        mede: bool,
    ) {
        if self.cena == Cena::Tocando {
            self.m.cena();
        }
        self.m.quadro();
        let t = Instant::now();
        pinta(&mut self.m, self.id, self.area, ts, scene, theme);
        if mede {
            self.s.quente.push(ms(t));
        }
        placa.render(self.pass, scene);
    }

    /// Uma tecla medida, do premir aos píxeis lidos.
    fn amostra(
        &mut self,
        ts: &mut TextSystem,
        placa: &mut Placa,
        scene: &mut VectorScene,
        theme: &Theme,
    ) {
        let ch = LETRAS[self.k % LETRAS.len()];
        let s = ch.to_string();
        let t0 = Instant::now();
        let _ = self.m.docs().key(
            self.id.0,
            ts,
            Key::Char(ch),
            Modifiers::default(),
            true,
            Some(&s),
            None,
        );
        let t_key = ms(t0);
        let t1 = Instant::now();
        if self.cena == Cena::Tocando {
            self.m.cena();
        }
        self.m.quadro();
        let t_quadro = ms(t1);
        let t2 = Instant::now();
        pinta(&mut self.m, self.id, self.area, ts, scene, theme);
        let t_paint = ms(t2);
        let t3 = Instant::now();
        placa.render(self.pass, scene);
        let faixa = placa.faixa(self.pass, self.band.0, self.band.1);
        let t_placa = ms(t3);
        let total = ms(t0);
        // Fora do relógio: o glifo novo está nos píxeis lidos.
        let x1 = self.x0 + (self.k + 1) as f32 * self.adv;
        let n = tinta(&faixa, x1 - self.adv + 1.0, x1 - 1.0, x1 + 4.0 * self.adv);
        assert!(
            n >= 4,
            "{}: o glifo {ch:?} não está nos píxeis ({n})",
            self.nome()
        );
        let c = self.m.docs().caret_area(self.id.0).expect("cursor");
        assert!(
            (c.x - x1).abs() < 1.0,
            "o cursor está onde a régua lê: {} × {x1}",
            c.x
        );
        self.s.total.push(total);
        self.s.key.push(t_key);
        self.s.quadro.push(t_quadro);
        self.s.paint.push(t_paint);
        self.s.placa.push(t_placa);
        let t = Instant::now();
        black_box(self.m.docs().a11y(self.id.0));
        self.s.a11y.push(ms(t));
        self.k += 1;
        for i in 0..ENTRE {
            self.quadro(ts, placa, scene, theme, i + 1 == ENTRE);
        }
        if self.k == LETRAS.len() {
            for _ in 0..LETRAS.len() {
                tecla(
                    &mut self.m,
                    ts,
                    self.id,
                    Key::Backspace,
                    Modifiers::default(),
                    None,
                );
                self.quadro(ts, placa, scene, theme, false);
            }
            self.k = 0;
        }
        placa.espera();
    }
}

/// Ctrl+S → o selo «Reloaded · 3 objects», com a cena a tocar.
struct Salvar {
    linhas: usize,
    m: Mesa,
    id: ScriptId,
    vai: bool,
    total: Vec<f64>,
    recarga: Vec<f64>,
}

impl Salvar {
    fn nova(linhas: usize, ts: &mut TextSystem, theme: &Theme) -> Self {
        let (mut m, id) = mesa(linhas);
        let mut scene = VectorScene::new();
        pinta(
            &mut m,
            id,
            Rect::new(0.0, 0.0, LARGURA, 900.0),
            ts,
            &mut scene,
            theme,
        );
        // O fim da 1.ª linha (um comentário): escrever lá não parte o script.
        tecla(&mut m, ts, id, Key::Home, crate::ctrl(), None);
        tecla(&mut m, ts, id, Key::End, Modifiers::default(), None);
        Self {
            linhas,
            m,
            id,
            vai: true,
            total: Vec::new(),
            recarga: Vec::new(),
        }
    }

    fn amostra(&mut self, ts: &mut TextSystem) {
        // Fora do relógio: o texto muda (uma letra a mais, a menos) e a cena dá um passo.
        if self.vai {
            tecla(
                &mut self.m,
                ts,
                self.id,
                Key::Char('x'),
                Modifiers::default(),
                Some("x"),
            );
        } else {
            tecla(
                &mut self.m,
                ts,
                self.id,
                Key::Backspace,
                Modifiers::default(),
                None,
            );
        }
        self.vai = !self.vai;
        self.m.cena();
        self.m.quadro();
        let t0 = Instant::now();
        tecla(
            &mut self.m,
            ts,
            self.id,
            Key::Char('s'),
            crate::ctrl(),
            Some("s"),
        );
        self.m.quadro();
        let antes = self.m.docs().badge_of(self.id);
        let t1 = Instant::now();
        self.m.cena();
        let t_recarga = ms(t1);
        self.m.quadro();
        let selo = self.m.docs().badge_of(self.id);
        let total = ms(t0);
        assert_eq!(antes, None, "o selo do salvar anterior apagou-se");
        assert_eq!(
            selo,
            Some(Badge::Reloaded { objects: 3 }),
            "{} linhas",
            self.linhas
        );
        self.m.hero.file_menu.save = false;
        self.total.push(total);
        self.recarga.push(t_recarga);
    }
}

/// O que a gramática faz por tecla, à parte: o reparse incremental e as derivações por revisão.
struct Gramatica {
    linhas: usize,
    buf: Buffer,
    syn: Syntax,
    k: usize,
    reparse: Vec<f64>,
    dobras: Vec<f64>,
    erros: Vec<f64>,
    realce: Vec<f64>,
}

impl Gramatica {
    fn nova(linhas: usize) -> Self {
        let texto = if linhas == 51 { bob() } else { juntado(GRANDE) };
        let mut buf = Buffer::new(&texto);
        let alvo = if linhas == 51 { ALVO_BOB } else { ALVO_GRANDE };
        let at = buf.text().line_end(alvo);
        buf.set_selection(Selection::single(Cursor::cursor(at)));
        let syn = Syntax::new(buf.text());
        Self {
            linhas,
            buf,
            syn,
            k: 0,
            reparse: Vec::new(),
            dobras: Vec::new(),
            erros: Vec::new(),
            realce: Vec::new(),
        }
    }

    fn amostra(&mut self) {
        let ch = LETRAS[self.k % LETRAS.len()].to_string();
        let edits = self.buf.insert(&ch);
        let t = Instant::now();
        self.syn.edit(&edits, self.buf.text());
        self.reparse.push(ms(t));
        let t = Instant::now();
        black_box(self.syn.folds(self.buf.text()));
        self.dobras.push(ms(t));
        let t = Instant::now();
        black_box(self.syn.errors());
        self.erros.push(ms(t));
        // O realce das 100 linhas à volta do cursor (o que a pintura pede por quadro).
        let t0 = self.buf.text();
        let alvo = t0.line_of_byte(self.buf.selection().primary().head);
        let (a, b) = (
            alvo.saturating_sub(50),
            (alvo + 50).min(t0.line_count() - 1),
        );
        let r = t0.byte_of_line(a)..t0.line_end(b);
        let t = Instant::now();
        black_box(self.syn.highlights(self.buf.text(), r));
        self.realce.push(ms(t));
        self.k += 1;
        if self.k == LETRAS.len() {
            for _ in 0..LETRAS.len() {
                let e = self.buf.delete_backward();
                self.syn.edit(&e, self.buf.text());
            }
            self.k = 0;
        }
    }
}

#[test]
#[ignore = "medição na placa: corre-se à mão (comando no topo do ficheiro)"]
fn tecla_pixel() {
    let load0 = loadavg();
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        eprintln!("E14: sem adaptador — NÃO medido");
        return;
    };
    let info = gpu.adapter.get_info();
    let mut placa = Placa {
        gpu,
        passes: Vec::new(),
    };
    let mut ts = TextSystem::new();
    let theme = Theme::default();
    let alturas = [
        (JANELA, altura_para(JANELA, &mut ts, &theme)),
        (MORTE, altura_para(MORTE, &mut ts, &theme)),
    ];
    let mut celulas = Vec::new();
    for &(vista, h) in &alturas {
        for linhas in [51, GRANDE] {
            for cena in [Cena::Parada, Cena::Tocando] {
                celulas.push(Celula::nova(
                    cena, linhas, vista, h, &mut ts, &mut placa, &theme,
                ));
            }
        }
    }
    let mut salvar = [
        Salvar::nova(51, &mut ts, &theme),
        Salvar::nova(GRANDE, &mut ts, &theme),
    ];
    let mut gram = [Gramatica::nova(51), Gramatica::nova(GRANDE)];
    // Aquecimento fora da régua: um bloco curto de cada.
    let mut scene = VectorScene::new();
    for c in &mut celulas {
        for _ in 0..LETRAS.len() {
            c.amostra(&mut ts, &mut placa, &mut scene, &theme);
        }
        c.s = Amostras::default();
    }
    for s in &mut salvar {
        s.amostra(&mut ts);
        s.amostra(&mut ts);
        s.total.clear();
        s.recarga.clear();
    }
    let n_tarefas = celulas.len() + salvar.len() + gram.len();
    let t_rodada = Instant::now();
    for r in 0..RODADAS {
        for i in 0..n_tarefas {
            let j = (r + i) % n_tarefas;
            for _ in 0..BLOCO {
                if j < celulas.len() {
                    celulas[j].amostra(&mut ts, &mut placa, &mut scene, &theme);
                } else if j < celulas.len() + salvar.len() {
                    salvar[j - celulas.len()].amostra(&mut ts);
                } else {
                    gram[j - celulas.len() - salvar.len()].amostra();
                }
            }
        }
    }
    let parede = t_rodada.elapsed().as_secs_f64();
    let load1 = loadavg();

    let perfil = if cfg!(debug_assertions) {
        "debug"
    } else {
        "otimizado (smoke/release)"
    };
    println!(
        "\nE14 — perfil {perfil} · placa {} ({:?}) · loadavg início {load0} · fim {load1} · rodada {parede:.1} s",
        info.name, info.backend
    );
    println!(
        "alturas da área: ~43 linhas = {:.0} px · 100 linhas = {:.0} px (largura {LARGURA:.0})",
        alturas[0].1, alturas[1].1
    );
    println!(
        "\nTecla→pixel, ms (min / p50 / p99 / p99.9); fases em p50; n por célula = {}",
        celulas[0].s.total.len()
    );
    println!(
        "| cena · documento · vista | linhas pintadas | 1.º quadro | tecla→pixel | tecla (buffer+AutoEdit+reparse) | quadro (+cena) | paint | placa + leitura | a11y por tecla | paint sem mudança |"
    );
    println!("|---|---|---|---|---|---|---|---|---|---|");
    for c in &celulas {
        println!(
            "| {} | {} | {:.1} | {} | {:.3} | {:.3} | {:.3} | {:.3} | {} | {} |",
            c.nome(),
            c.visiveis,
            c.primeiro,
            resumo(&c.s.total),
            p50(&c.s.key),
            p50(&c.s.quadro),
            p50(&c.s.paint),
            p50(&c.s.placa),
            resumo(&c.s.a11y),
            resumo(&c.s.quente),
        );
    }
    println!(
        "\nCtrl+S → selo «Reloaded · 3 objects» (cena a tocar), ms; n = {}",
        salvar[0].total.len()
    );
    println!("| documento | Ctrl+S → selo | dos quais a recarga (`scene_sync` + passo) |");
    println!("|---|---|---|");
    for s in &salvar {
        println!(
            "| {} linhas | {} | {} |",
            s.linhas,
            resumo(&s.total),
            resumo(&s.recarga)
        );
    }
    println!(
        "\nGramática por tecla, à parte, ms; n = {}",
        gram[0].reparse.len()
    );
    println!(
        "| documento | reparse incremental | dobras (`folds`) | erros | realce de 100 linhas |"
    );
    println!("|---|---|---|---|---|");
    for g in &gram {
        println!(
            "| {} linhas | {} | {} | {} | {} |",
            g.linhas,
            resumo(&g.reparse),
            resumo(&g.dobras),
            resumo(&g.erros),
            resumo(&g.realce)
        );
    }
}
