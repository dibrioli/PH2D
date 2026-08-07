//! **COMO O BARRO É MOSTRADO** — o desenho e as opções de vista.
//!
//! ⚠️ **Corte por RESPONSABILIDADE, e a linha é entre *o que a cena É* (o irmão)
//! e *como ela APARECE* (aqui).** As duas metades crescem por motivos
//! diferentes: a de lá ganha estado quando a cena ganha um verbo, esta ganha uma
//! linha quando o SOMBREAMENTO ganha um canal — e foi um canal novo (o AO de
//! tela) que cruzou o teto de 600 LOC do arquivo pai.

use super::Sculpt3dScene;

impl Sculpt3dScene {
    /// Desenha a malha sobre o que já está no alvo. O upload acontece na
    /// primeira passagem — é aqui que o device é conhecido.
    pub(crate) fn render(
        &mut self,
        gpu: &ph2d_gpu::GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        color: &wgpu::TextureView,
        size: (u32, u32),
    ) {
        self.viewport = size;
        // ⚠️ O viewport é atualizado ANTES da recusa: ele é o que converte um
        // clique em raio, e um viewport parado faria o pincel cair no lugar
        // errado no instante em que o artista voltasse ao barro.
        if !self.shows_clay() {
            return;
        }
        self.sync_mesh(&gpu.device, &gpu.queue);
        // O rig é RESOLVIDO por frame, não guardado resolvido: a resolução é
        // barata (quatro lâmpadas) e uma cópia resolvida seria uma segunda
        // verdade sobre onde a luz está — a que fica velha no frame seguinte ao
        // artista mexer no card.
        let resolved = ph2d_light::resolve(&self.rig);
        // **A OCLUSÃO DE TELA, medida ANTES da cor** (`ph2d_mesh_render::ssao`).
        //
        // ⚠️ A ordem é o desenho: este passe rasteriza normais + profundidade e
        // um passe de tela cheia marcha o horizonte, e o `render` seguinte
        // AMOSTRA o resultado. Rodá-lo depois seria escurecer a imagem pronta —
        // barato, e errado, porque apagaria o REALCE junto (oclusão ambiente não
        // apaga brilho especular, ela apaga a luz que vem do céu).
        //
        // ⚠️ E a frescura é CONSUMIDA pelo `render`: pular esta chamada num frame
        // desenha sem oclusão, nunca com a do frame passado — uma medição de tela
        // descreve uma câmera, e a de ontem descreve outra.
        if self.ssao > 0.0 {
            self.renderer.render_ssao(
                &gpu.device,
                &gpu.queue,
                encoder,
                &self.camera,
                self.ssao_params(),
                size,
            );
        }
        self.renderer.render(
            &gpu.device,
            &gpu.queue,
            encoder,
            color,
            &self.camera,
            resolved.as_ref(),
            self.shade(),
            size,
        );
    }

    /// **COMO O BARRO É MOSTRADO** — a porta única das opções de vista.
    ///
    /// ⚠️ Ela existe porque as três viajam juntas para o device E para o painel,
    /// e montá-las no sítio de chamada faria a `render` e o `panel_snapshot`
    /// darem duas respostas a *"como está a vista"* — o par que diverge no dia
    /// em que a quarta opção chegar por um dos dois.
    pub(crate) fn shade(&self) -> ph2d_mesh_render::Shade {
        ph2d_mesh_render::Shade {
            cavity: self.cavity,
            ao: self.ao,
            ssao: self.ssao,
            // ⚠️ **A força é do artista e o alcance é da PEÇA** — o mesmo corte do
            // raio do AO de tela, e pelo mesmo motivo: um `scatter` guardado
            // seria uma segunda verdade sobre o tamanho da escultura.
            // ⚠️ **A força e a FRAÇÃO são autoradas; o comprimento é derivado.**
            // O `for_bounds` continua sendo quem sabe medir a peça — o que o
            // slider move é a fração que ele aplica, então o alcance nunca fica
            // velho quando a escultura cresce e o artista mesmo assim decide o
            // look.
            sss: ph2d_mesh_render::SssParams {
                strength: self.sss,
                ..ph2d_mesh_render::SssParams::for_bounds_with(
                    self.world_bounds(),
                    self.sss_scatter,
                )
            },
            matcap: self.matcap,
            wireframe: self.wireframe,
        }
    }

    /// **COMO a oclusão de tela é medida nesta peça** — o raio semeado pelo
    /// tamanho dela, o resto no default.
    ///
    /// ⚠️ Derivado por frame e não guardado: a caixa muda a cada dab, e um raio
    /// guardado seria uma segunda verdade sobre o tamanho da peça — a que fica
    /// velha exatamente quando o artista a faz crescer.
    ///
    /// ⚠️ **E a caixa é a da CENA, não a da peça ativa** — a diferença é a razão
    /// de este passe existir ao lado do assado. O bake mede um corpo contra o
    /// próprio campo SDF: ele nunca vê o vizinho. O de tela vê o que está na
    /// tela, e a sombra que uma peça lança sobre outra é exatamente o que só ele
    /// consegue medir; semear o raio pela peça ativa faria essa sombra encolher
    /// quando o artista clicasse na menor das duas.
    fn ssao_params(&self) -> ph2d_mesh_render::SsaoParams {
        ph2d_mesh_render::SsaoParams::for_bounds(self.world_bounds())
    }

    /// **A CAVIDADE, um passo adiante** — devolve a quantidade nova.
    ///
    /// Ciclo e não par de teclas, o idioma do [`Self::cycle_role`] ao lado: é um
    /// canal de LEITURA, e o artista o escolhe uma vez e volta a esculpir. Os
    /// quatro degraus dão desligado, sutil, forte e o teto.
    ///
    /// ⚠️ **Um número e não dois.** O `docs/3D/05.1` §4 fala em *Cavity* e *Edge
    /// Wear*, e os dois são a UI de um MATERIAL — sujeira acumulada na fresta e
    /// tinta gasta na quina são histórias físicas diferentes, com quantidades
    /// diferentes. Para LER FORMA, que é o que esta wave entrega, a curvatura é
    /// *um* número com sinal e escurecer/clarear são as duas metades da mesma
    /// multiplicação. Inventar o segundo agora seria um knob que nenhum gesto
    /// alcança; se o smoke disser que os dois lados querem quantidades
    /// diferentes, é ele que parte este número em dois.
    /// **O ESPALHAMENTO, um passo adiante** — o irmão exato do
    /// [`Self::cycle_cavity`], e os degraus são os mesmos pela mesma razão: é um
    /// canal que o artista escolhe uma vez e volta a esculpir.
    ///
    /// ⚠️ **Só a FORÇA cicla, e o alcance vive no PAINEL** — as duas metades
    /// existem e são alcançáveis por caminhos diferentes de propósito. A tecla é
    /// o gesto de *ligar e continuar esculpindo*; o alcance é o número que
    /// decide o LOOK, e um veredito de aparência se procura arrastando, não
    /// batendo em quatro degraus.
    ///
    /// ⚠️ **E o painel é o dono do valor, não uma segunda cópia dele.** Este
    /// ciclo escreve o MESMO `self.sss` que o `apply_ui` escreve e que o
    /// `panel_snapshot` publica, então a tecla e a pista nunca podem discordar —
    /// apertar `Shift+S` move o slider na tela.
    pub(crate) fn cycle_sss(&mut self) -> f32 {
        const STEPS: [f32; 4] = [0.0, 0.35, 0.70, 1.0];
        let at = STEPS
            .iter()
            .position(|s| (s - self.sss).abs() < 1e-4)
            .unwrap_or(0);
        self.sss = STEPS[(at + 1) % STEPS.len()];
        self.sss
    }

    pub(crate) fn cycle_cavity(&mut self) -> f32 {
        const STEPS: [f32; 4] = [0.0, 0.35, 0.70, 1.0];
        let at = STEPS
            .iter()
            .position(|s| (s - self.cavity).abs() < 1e-4)
            .unwrap_or(0);
        self.cavity = STEPS[(at + 1) % STEPS.len()];
        self.cavity
    }
}
