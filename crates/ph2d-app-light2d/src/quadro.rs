//! **A luz do quadro** — o dono do solver e da grade; a shell só chama as três portas.

use std::collections::BTreeSet;

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use kurbo::Affine;
use ph2d_ecs::PresentWorld;
use ph2d_gpu::GpuContext;
use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_light2d_gpu::{Config, ErroSolver, Grade, RelatorioC0, Solver};
use ph2d_render::tonemap::Tonemap;
use ph2d_render::{BandBlit, LuzRecebida, SpriteRenderer};

use crate::grade::{DEGRAU_PADRAO, Degrau, grade_do_quadro, tela_para_mundo};

struct Resolvido {
    solver: Solver,
    /// A vista da fluência (a textura é fixa enquanto o solver vive).
    vista: wgpu::TextureView,
}

/// A luz 2D do quadro do editor. Ver o cabeçalho da crate para as três portas.
pub struct LuzDoQuadro {
    degrau: Degrau,
    cena: CenaDeLuz,
    sem_luz: BTreeSet<Entity>,
    /// `Some` = a cena tem `Luz` neste quadro.
    grade: Option<Grade>,
    mundo_para_tela: Affine,
    resolvido: Option<Resolvido>,
    relatorio: Option<RelatorioC0>,
    erro: Option<ErroSolver>,
    /// `(por texel, sprites da cena)` no último quadro — os outros ficaram com o quad.
    sprites_por_texel: (usize, usize),
    /// ⭐ Os caminhos vetoriais que bloqueiam, desenhados na grade (W3).
    vetores: crate::vetor::CamadaVetorial,
    /// A cena de smoke armada já foi encenada (uma vez por sessão).
    smoke_encenado: bool,
}

impl Default for LuzDoQuadro {
    fn default() -> Self {
        Self::new()
    }
}

impl LuzDoQuadro {
    #[must_use]
    pub fn new() -> Self {
        Self::com_degrau(DEGRAU_PADRAO)
    }

    /// Com um degrau explícito — a sonda mede as variantes NO MESMO processo.
    #[must_use]
    pub fn com_degrau(degrau: Degrau) -> Self {
        Self {
            degrau,
            cena: CenaDeLuz::default(),
            sem_luz: BTreeSet::new(),
            grade: None,
            mundo_para_tela: Affine::IDENTITY,
            resolvido: None,
            relatorio: None,
            erro: None,
            sprites_por_texel: (0, 0),
            vetores: crate::vetor::CamadaVetorial::default(),
            smoke_encenado: false,
        }
    }

    /// **Porta 1** — coleta a cena, decide a GUARDA e monta a grade ancorada para a vista
    /// `mundo_para_tela` (o afim que o Vello usa) sobre `alvo_px`. Devolve se a luz está acesa: sem
    /// nenhuma `Luz` na cena é `false`, o solver não corre e o quadro é o de sempre.
    pub fn preparar(
        &mut self,
        world: &World,
        mundo_para_tela: Affine,
        alvo_px: (u32, u32),
    ) -> bool {
        self.cena = cena_de_luz(world);
        self.mundo_para_tela = mundo_para_tela;
        self.grade = if self.cena.luzes == 0 {
            None
        } else {
            grade_do_quadro(mundo_para_tela, alvo_px, self.degrau)
        };
        self.sem_luz.clear();
        if self.grade.is_some() {
            self.sem_luz.extend(self.cena.nao_recebem.iter().copied());
        }
        self.grade.is_some()
    }

    /// ⭐ **As portas 1 e 1b numa chamada** — o que a codificação da shell faz por quadro: encena a
    /// cena de smoke armada (uma vez), prepara e desenha os vetores que bloqueiam. Devolve se a luz
    /// está acesa (a shell força então o documento às faixas).
    pub fn codificar(
        &mut self,
        world: &mut World,
        superficie: &ph2d_gpu::SurfaceContext,
        mundo_para_tela: Affine,
        vetores: (&ph2d_vec_scene::VecScene, &ph2d_vec_scene::VecXforms),
    ) -> bool {
        if !self.smoke_encenado {
            self.smoke_encenado = true;
            if let Some(n) = crate::smoke::nivel_armado() {
                crate::smoke::encenar(world, n);
            }
        }
        let alvo = superficie.size();
        let acesa = self.preparar(world, mundo_para_tela, (alvo.width, alvo.height));
        self.desenhar_vetores(superficie.gpu(), vetores.0, vetores.1);
        acesa
    }

    /// **Porta 1b** — desenha na grade os caminhos VETORIAIS que bloqueiam (os afins `xforms` do
    /// quadro). Apagada, ou sem nenhum, não faz nada.
    pub fn desenhar_vetores(
        &mut self,
        gpu: &GpuContext,
        doc: &ph2d_vec_scene::VecScene,
        xforms: &ph2d_vec_scene::VecXforms,
    ) {
        if let Some(g) = self.grade {
            let fator = self.degrau.vetor_px_por_texel;
            self.vetores
                .desenhar(gpu, (g, fator), &self.cena, doc, xforms);
        }
    }

    /// A luz está acesa neste quadro (a cena tem `Luz` e o solver não recusou)?
    #[must_use]
    pub fn acesa(&self) -> bool {
        self.grade.is_some() && self.erro.is_none()
    }

    /// **Porta 2** — grava o solver num encoder próprio e submete-o (a fila põe-no antes do passe de
    /// sprite). Apagada, não faz nada. O solver só é RECRIADO quando as dimensões da grade mudam
    /// (redimensionar a janela); pan e zoom só o reposicionam. Os sprites que emitem ou bloqueiam
    /// entram pelo alfa de cada texel — das instâncias do `present`, com as texturas do `renderer`.
    pub fn gravar(
        &mut self,
        gpu: &GpuContext,
        present: &mut PresentWorld,
        renderer: &SpriteRenderer,
    ) {
        let Some(g) = self.grade else {
            return;
        };
        let mesmas = self.resolvido.as_ref().is_some_and(|r| {
            let c = r.solver.config().grade;
            (c.largura, c.altura) == (g.largura, g.altura)
        });
        let pronto = if mesmas {
            let r = self.resolvido.as_mut().expect("verificado acima");
            r.solver.reposicionar(g.origem_m, g.texel_m)
        } else {
            self.resolvido = None;
            Solver::novo(gpu, Config::padrao_para(&gpu.device, g)).map(|solver| {
                let vista = solver
                    .fluencia()
                    .create_view(&wgpu::TextureViewDescriptor::default());
                self.resolvido = Some(Resolvido { solver, vista });
            })
        };
        self.erro = pronto.err();
        if let Some(e) = &self.erro {
            // ⚠️ Fica DITO (a W4 leva-o à UI): a luz apaga-se neste quadro em vez de mentir.
            eprintln!("[luz2d] o solver recusou a grade {g:?}: {e:?}");
            return;
        }
        let r = self
            .resolvido
            .as_mut()
            .expect("criado ou reposicionado acima");
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("luz2d quadro do editor"),
            });
        let mut sprites = crate::captura::sprites_da_cena(&self.cena, present, renderer);
        for (vista, mut s) in self.vetores.sprites() {
            // A camada está alinhada à grade e o píxel dela já é a cobertura de área: lida EXATA.
            sprites
                .texturas
                .push((vista, ph2d_light2d_gpu::Amostragem::Exata));
            s.textura = sprites.texturas.len() - 1;
            sprites.sprites.push(s);
        }
        self.sprites_por_texel = (sprites.substituidos.len(), self.cena.sprites.len());
        self.relatorio = Some(
            r.solver
                .gravar_com_sprites(gpu, &mut enc, &self.cena, &sprites),
        );
        gpu.queue.submit([enc.finish()]);
    }

    /// O que os receptores leem; `None` = apagada.
    #[must_use]
    pub fn recebida(&self) -> Option<LuzRecebida<'_>> {
        if !self.acesa() {
            return None;
        }
        let r = self.resolvido.as_ref()?;
        let g = r.solver.config().grade;
        Some(LuzRecebida {
            fluencia: &r.vista,
            origem_m: g.origem_m,
            texel_m: g.texel_m,
            tamanho: [g.largura, g.altura],
            sem_luz: &self.sem_luz,
        })
    }

    /// **Porta 3** — acende (ou apaga) os dois receptores para este quadro.
    pub fn ligar(
        &self,
        gpu: &GpuContext,
        renderer: &mut SpriteRenderer,
        colagem: &mut BandBlit,
        tonemap: &Tonemap,
    ) {
        let r = self.recebida();
        renderer.receber_luz(r.as_ref());
        let mapa = tela_para_mundo(self.mundo_para_tela);
        colagem.receber_luz(gpu, r.as_ref().map(|l| (l, mapa, tonemap)));
    }

    /// A cena coletada neste quadro.
    #[must_use]
    pub fn cena(&self) -> &CenaDeLuz {
        &self.cena
    }

    /// A grade deste quadro (`None` = apagada).
    #[must_use]
    pub fn grade(&self) -> Option<Grade> {
        self.grade
    }

    /// O que a C0 deixou de fora no último quadro (a W4 leva-o à UI).
    #[must_use]
    pub fn relatorio(&self) -> Option<&RelatorioC0> {
        self.relatorio.as_ref()
    }

    /// `(capturados por texel, sprites que emitem ou bloqueiam)` no último quadro: a diferença ficou
    /// com o QUAD (malha, ladrilho, textura cozida ou instância fora do `present`).
    #[must_use]
    pub fn sprites_por_texel(&self) -> (usize, usize) {
        self.sprites_por_texel
    }

    /// A recusa do solver no último quadro, se houve.
    #[must_use]
    pub fn erro(&self) -> Option<&ErroSolver> {
        self.erro.as_ref()
    }

    /// O solver vivo (a sonda lê os tempos dele).
    #[must_use]
    pub fn solver(&self) -> Option<&Solver> {
        self.resolvido.as_ref().map(|r| &r.solver)
    }
}
