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
    dono: Option<String>,
}

impl ChaveDasSondas {
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
        let mut marcha = bits(&[
            setup.hit_eps,
            setup.normal_eps,
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
            dono: lei_do_dono.map(|l| l.source.clone()),
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
