//! ⭐ W5q — os LADRILHOS 8×8 da grade fina que o anel (`c3_anel.wgsl`) resolve, um bit por ladrilho
//! (`wgsl/ladrilhos.wgsl` lê-o): os que tocam um corpo da captura fina (a caixa de cada forma que ela
//! desenha, com a folga) ou a janela com a moldura de fora. Fora deles o anel escrevia −1 (fora dos
//! corpos) na grade inteira: agora salta-os, e o `miolo_puxa` do nível 1 lê um ladrilho apagado como
//! esse −1 — o mesmo bit, sem o teste de corpo em cada texel nem a escrita.

/// O lado de um ladrilho (o grupo 8×8 do anel).
pub(crate) const LADO: u32 = 8;

fn por_lado(fina: [u32; 2]) -> [u32; 2] {
    fina.map(|x| x.div_ceil(LADO).max(1))
}

/// Os bytes do buffer (a `PassList` declara-o com o miolo).
pub(crate) fn bytes(fina: [u32; 2]) -> u64 {
    let [x, y] = por_lado(fina);
    4 * u64::from((x * y).div_ceil(32))
}

pub(crate) struct Ladrilhos {
    pub(crate) buffer: wgpu::Buffer,
    fina: [u32; 2],
    /// Texels da captura por texel da fina.
    escala: f64,
    /// A folga em texels da fina: a vizinha do anel e o `textureGather` da captura.
    folga: f64,
    bits: Vec<u32>,
    enviados: Vec<u32>,
    janela: ([u32; 2], [u32; 2]),
    /// Instrumento (o controlo da régua): os corpos não acendem nada.
    pub(crate) so_a_janela: bool,
}

impl Ladrilhos {
    pub(crate) fn novo(buffer: wgpu::Buffer, fina: [u32; 2], captura: [u32; 2]) -> Self {
        let escala = f64::from(captura[0]) / f64::from(fina[0]);
        let palavras = (bytes(fina) / 4) as usize;
        Self {
            buffer,
            fina,
            escala,
            folga: 1.0 + (1.0 / escala).max(1.0).ceil(),
            bits: vec![0; palavras],
            enviados: Vec::new(),
            janela: ([0; 2], fina),
            so_a_janela: false,
        }
    }

    pub(crate) fn janela(&mut self, inicio: [u32; 2], fim: [u32; 2]) {
        self.janela = (inicio, fim);
    }

    /// O quadro começa: `todos` = cada ladrilho aceso (o anel na grade inteira, o antes); senão só a
    /// janela com a moldura de fora (o `classifica` e o `miolo_final` leem-nas).
    pub(crate) fn comecar(&mut self, todos: bool) {
        self.bits.fill(if todos { u32::MAX } else { 0 });
        let ([x0, y0], [x1, y1]) = self.janela;
        self.retangulo(
            [f64::from(x0) - 1.0, f64::from(y0) - 1.0],
            [f64::from(x1) + 1.0, f64::from(y1) + 1.0],
        );
    }

    /// A caixa `x0, y0, x1, y1` de uma forma, em texels da CAPTURA.
    pub(crate) fn caixa_da_captura(&mut self, c: [f32; 4]) {
        if self.so_a_janela {
            return;
        }
        let f = |v: f32| f64::from(v) / self.escala;
        self.retangulo(
            [
                (f(c[0])).floor() - self.folga,
                (f(c[1])).floor() - self.folga,
            ],
            [(f(c[2])).ceil() + self.folga, (f(c[3])).ceil() + self.folga],
        );
    }

    /// Acende os ladrilhos que tocam `[min, max)` (texels da fina).
    fn retangulo(&mut self, min: [f64; 2], max: [f64; 2]) {
        let n = por_lado(self.fina);
        let lado = f64::from(LADO);
        let de = |v: f64, k: usize| (v / lado).floor().clamp(0.0, f64::from(n[k])) as u32;
        let ate = |v: f64, k: usize| (v / lado).ceil().clamp(0.0, f64::from(n[k])) as u32;
        for y in de(min[1], 1)..ate(max[1], 1) {
            for x in de(min[0], 0)..ate(max[0], 0) {
                let i = (y * n[0] + x) as usize;
                self.bits[i >> 5] |= 1 << (i & 31);
            }
        }
    }

    /// Sobe os bits se mudaram desde o último envio.
    pub(crate) fn enviar(&mut self, queue: &wgpu::Queue) {
        if self.bits != self.enviados {
            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.bits));
            self.enviados.clone_from(&self.bits);
        }
    }
}
