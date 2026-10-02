//! ⭐⭐⭐⭐ **AS SONDAS DO RICOCHETE, GUARDADAS NA PLACA ENTRE QUADROS** — o travão de girar a peça.
//!
//! Report do dono (2026-09-24): *«travamentos ao rotacionar a tela continuam»*. Medido
//! (`diag_o_quadro_assente_partido`, `1920×1080`): o quadro ASSENTE custa `231,9 ms` no nó de toro da
//! cena `=28` contra `82,2` sem o ricochete — e o de MOVIMENTO no mesmo tamanho custa `79,1`. ⇒ o que
//! o assente paga a mais é **todo** o ricochete, e dele a parte fixa é a assadura das sondas
//! (`PROBE_GRID³` grupos, `256` direcções cada, cada uma a marchar a árvore). Quando a mão HESITA um
//! quadro a meio de uma órbita, o app pede o assente, a placa não o cancela, e o quadro de movimento
//! seguinte espera por ele — é o travão.
//!
//! ⭐⭐⭐ **As sondas não dependem da ORIENTAÇÃO da câmera** — o comentário do próprio despacho já o
//! dizia (*«uma cache por cena é a wave seguinte»*). A assadura lê: a fita e as constantes da peça ·
//! as lâmpadas e a radiância delas · as gémeas foscas e a lei do dono · a bola da peça · a
//! tolerância de acerto e o passo da normal (que dependem do ZOOM abaixo do clamp, e por isso
//! ENTRAM) · o orçamento e o passo da marcha · a grade de longe. O único eixo da câmera que lá
//! chega é a base da vista (`mundo_para_vista`), e só como ROTAÇÃO de vectores de que a lei lê
//! produtos internos — invariante em aritmética exacta, e a `≤ 1` ULP em `f32` (a mesma conta que a
//! cache do campo do chão já fez: orbitar move `6e-9`).
//!
//! ⛔ **Uma peça com ESCULTURA não é guardada** — a mesma cerca da cache do chão: a fita não
//! inlina a escultura, e duas esculturas diferentes dariam a mesma chave.

use crate::trace::MarchSetup;

/// ⏱️⭐⭐⭐⭐ **Quantas CÉLULAS da grade as sondas guardadas podem estar deslocadas** e ainda servir a
/// um quadro que não pode esperar — ver [`crate::FieldPipelines::sondas_a_mexer`].
///
/// ⭐ **O número sai do CRUZAMENTO das duas saídas** (`diag_as_sondas_velhas_a_mexer`, cinco toros
/// com o do meio arrastado, `--release`, carga `~2,5`): contra a imagem do MESMO quadro com as
/// sondas acabadas de assar, erro por canal em bytes, média / p99 —
///
/// | deslocamento (células) | sondas VELHAS | SEM ricochete |
/// |---:|---:|---:|
/// | `0,036` | `0,08 / 1` | `1,20 / 14` |
/// | `0,232` | `0,25 / 3` | `1,18 / 13` |
/// | `0,408` | `0,40 / 5` | `1,18 / 13` |
/// | `0,656` | `0,63 / 11` | `1,22 / 14` |
/// | `0,939` | `0,83 / 16` | `1,29 / 14` |
/// | `1,617` | `1,08 / 14` | `1,51 / 16` |
///
/// As velhas erram LINEARMENTE com o deslocamento (`~1` byte de média por célula) e ir sem
/// ricochete erra `~1,2` sempre; o p99 delas passa o da outra saída entre `0,66` e `0,94` (`≈ 0,83`
/// interpolado) ⇒ `0,75`, abaixo do cruzamento. ⚠️ Medido numa cena; a peça do dono pode
/// deslocar-se mais depressa por célula, e o que o número protege é a ORDEM das duas saídas.
pub const TOLERANCIA_EM_CELULAS: f32 = 0.75;

/// ⭐⭐⭐ **Tudo o que a assadura das sondas lê, menos a orientação da câmera.**
///
/// ⚠️ **Comparada por IGUALDADE, nunca por hash** — uma colisão entregaria o ricochete de outra peça
/// sem erro nenhum, e o texto da fita são poucos KB por quadro.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ChaveDasSondas {
    fonte: String,
    consts: Vec<u32>,
    /// Os campos do [`MarchSetup`] que a marcha das sondas lê, em bits.
    marcha: Vec<u32>,
    /// A grade de longe inteira (a caixa e a resolução): a marcha salta por ela.
    longe: Option<crate::longe::Longe>,
    radiancia: Vec<u32>,
    foscas: Vec<u32>,
    /// ⚠️ **O texto E os números da lei do dono.** Desde 2026-10-01 o texto é o MESMO para toda
    /// peça (a lei interpretada), logo é nos números que mora *«de que cor é cada folha»*: só o
    /// texto deixaria as sondas assadas com os donos de outra peça.
    dono: Option<(String, Vec<u32>)>,
}

impl ChaveDasSondas {
    /// ⭐⭐⭐⭐ **Quanto a GRADE destas sondas se deslocou da de `outra`, em CÉLULAS.**
    ///
    /// A grade é a bola da peça (centro e raio) — ver o `sonda_canto`/`sonda_passo` do
    /// `paint_wgsl_sondas`: o canto é `centro − raio·margem` e o passo `2·raio·margem/(G − 1)`. O
    /// ponto da grade que mais anda é um canto, e anda no máximo `max|Δcentro| + |Δraio|·margem`.
    /// ⚠️ Num arrasto a bola muda SEMPRE (medido no gate `arrastar_nao_assa…`: `1,7e-4` no centro
    /// a `dy = 0,03`) ⇒ a pergunta não pode ser de igualdade.
    fn deslocamento_em_celulas(&self, outra: &Self) -> f32 {
        // `marcha` = `[step, cx, cy, cz, raio, …]` — ver [`Self::de`].
        let f = |k: &Self, i: usize| k.marcha.get(i).copied().map_or(f32::NAN, f32::from_bits);
        let margem = ph2d_field_render::probes::PROBE_MARGIN;
        let raio = f(self, 4).max(f(outra, 4)).max(1e-3) * margem;
        #[allow(clippy::cast_precision_loss)]
        let celula = 2.0 * raio / (ph2d_field_render::probes::PROBE_GRID - 1) as f32;
        let dc = (1..4)
            .map(|i| (f(self, i) - f(outra, i)).abs())
            .fold(0.0f32, f32::max);
        let dr = (f(self, 4) - f(outra, 4)).abs() * margem;
        (dc + dr) / celula
    }

    /// A chave deste quadro — `None` quando ele não pode ser guardado (há escultura).
    pub(crate) fn de(
        fita: &ph2d_field_eval::wgsl::TapeWgsl,
        sculpts: &[ph2d_field_eval::device::DeviceSculpt],
        setup: &MarchSetup,
        pintor: &crate::paint::PaintSetup<'_>,
        lei_do_dono: Option<&ph2d_field_eval::owners::wgsl::OwnersWgsl>,
    ) -> Option<Self> {
        if !sculpts.is_empty() {
            return None;
        }
        let bits = |v: &[f32]| v.iter().map(|f| f.to_bits()).collect::<Vec<u32>>();
        let n = setup.n_lamps as usize;
        // ⛔⛔ **NADA DA CÂMARA ENTRA AQUI** (2026-10-01, report do dono: *«se aproximar ainda fica
        // lento e perde resolução»*). Os dois limiares de PIXEL (`hit_eps`, `normal_eps`) estavam na
        // chave, e de perto eles seguem o zoom (`pixel/4`): cada quadro de aproximar re-assava as
        // sondas — `162,7 ms` no nó, medido pelo relógio da placa. Eles só deslocam a origem e a
        // diferença finita de um raio de sonda na ordem de UM pixel, e a sonda é uma grelha no MUNDO
        // com células de `diâmetro/PROBE_GRID` — o mesmo argumento da chave do céu
        // (`crate::ceu_tempo::ChaveDoCeu`).
        let mut marcha = bits(&[
            setup.step,
            setup.ball_center[0],
            setup.ball_center[1],
            setup.ball_center[2],
            setup.ball_radius,
        ]);
        marcha.push(setup.budget);
        marcha.push(setup.n_lamps);
        for l in setup.lamps.iter().take(n) {
            marcha.extend(bits(l));
        }
        Some(Self {
            fonte: fita.source.clone(),
            consts: bits(&fita.consts),
            marcha,
            longe: setup.longe,
            radiancia: pintor
                .lamp_radiance
                .iter()
                .take(n)
                .flat_map(|r| bits(r))
                .collect(),
            foscas: bits(pintor.matte),
            dono: lei_do_dono.map(|l| (l.source.clone(), bits(&l.consts))),
        })
    }
}

/// ⭐ **As sondas residentes** — a chave que as produziu e o armazém.
pub(crate) struct SondasNaPlaca {
    pub(crate) chave: ChaveDasSondas,
    pub(crate) buffer: wgpu::Buffer,
}

impl crate::FieldPipelines {
    /// ⭐⭐⭐ **O armazém das sondas deste quadro, e se é preciso ASSÁ-LAS.**
    ///
    /// Com a mesma chave devolve o armazém guardado e `false`; com outra (ou sem chave) devolve um
    /// armazém novo e `true` — e só o GUARDA quando há chave. ⚠️ **Quem recebe `true` tem de
    /// despachar a assadura no MESMO envio que lê o armazém**: a chave é gravada aqui, antes do
    /// despacho, e é a espera do envio que a torna verdadeira.
    pub(crate) fn sondas(
        &mut self,
        device: &wgpu::Device,
        chave: Option<ChaveDasSondas>,
        bytes: u64,
    ) -> (wgpu::Buffer, bool) {
        if let (Some(c), Some(guardadas)) = (&chave, &self.sondas)
            && guardadas.chave == *c
        {
            return (guardadas.buffer.clone(), false);
        }
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sondas"),
            size: bytes,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        self.assaduras_de_sondas += 1;
        if let Some(chave) = chave {
            self.sondas = Some(SondasNaPlaca {
                chave,
                buffer: buffer.clone(),
            });
        }
        (buffer, true)
    }

    /// ⏱️⭐⭐⭐⭐ **As sondas de um quadro que NÃO PODE ESPERAR** — o movimento (report do dono,
    /// 2026-10-01: *«ainda com delay de 1 ou 2 segundos»* · §10.3 *«arrastar objetos tem um delay
    /// absurdo»*). Medido na cena dele (`diag_o_preco_de_uma_forma_nova_ao_lado_do_no`): cada
    /// quadro de ARRASTO de uma caixa re-assava as sondas — `129 ms` de placa num quadro de `171`,
    /// contra `14` com elas guardadas.
    ///
    /// ⭐ **A lei W73 — grosso a mexer, nítido ao assentar — aplicada à luz que ricocheteia:** com a
    /// mesma chave são as de sempre; com outra chave mas a MESMA [`ChaveDasSondas::grade`] servem
    /// as guardadas (o ricochete fica a dever a última edição enquanto a mão mexe) e a chave NÃO é
    /// reescrita — o assente seguinte vê-a diferente e re-assa. `None` quando não há sondas que
    /// sirvam: o chamador desenha esse quadro sem ricochete.
    pub(crate) fn sondas_a_mexer(&self, chave: Option<&ChaveDasSondas>) -> Option<wgpu::Buffer> {
        let (c, guardadas) = (chave?, self.sondas.as_ref()?);
        (guardadas.chave == *c
            || guardadas.chave.deslocamento_em_celulas(c) <= self.tolerancia_das_sondas)
            .then(|| guardadas.buffer.clone())
    }

    /// ⭐⭐ **Quantas vezes as sondas foram ASSADAS** — a régua da cache.
    ///
    /// ⚠️ **A economia é INVISÍVEL a toda régua de valor** (as duas rotas dão o mesmo ricochete, a
    /// `≤ 1` ULP) ⇒ o gate mede a CONTA. ⭐ E ela é por TRAÇADOR e não global: um irmão a pintar
    /// noutro traçador não entra nela.
    #[must_use]
    pub fn sondas_assadas(&self) -> usize {
        self.assaduras_de_sondas
    }

    /// Esquece as sondas guardadas — a porta pela qual um gate compara a rota FRIA com a guardada,
    /// e uma sonda de relógio mede as duas no mesmo processo.
    pub fn esquece_as_sondas(&mut self) {
        self.sondas = None;
    }
}
