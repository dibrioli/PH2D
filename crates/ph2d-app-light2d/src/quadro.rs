//! **A luz do quadro** — o dono do solver e da grade; a shell só chama as três portas.

use std::collections::BTreeSet;

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use kurbo::Affine;
use ph2d_ecs::PresentWorld;
use ph2d_editor_core::luz_edits::{AvisoDaLuz, NivelDaLuz, PreferenciaDaLuz};

use crate::qualidade::Governador;
use ph2d_gpu::GpuContext;
use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_light2d_gpu::{Config, ErroSolver, Grade, Motivo, Quem, RelatorioC0, Solver};
use ph2d_render::tonemap::Tonemap;
use ph2d_render::{BandBlit, LuzRecebida, SpriteRenderer};

use crate::grade::{
    DEGRAU_PADRAO, Degrau, captura_do_quadro, fina_do_quadro, grade_do_quadro, tela_para_mundo,
};

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
    /// ⭐ Texels finos por texel da grade (a luz direta, W4b).
    fina: u32,
    /// ⭐ Texels da captura fina dos opacos por texel da grade (W5a).
    captura: u32,
    mundo_para_tela: Affine,
    alvo_px: (u32, u32),
    resolvido: Option<Resolvido>,
    relatorio: Option<RelatorioC0>,
    erro: Option<ErroSolver>,
    /// `(por texel, sprites da cena)` no último quadro — os outros ficaram com o quad.
    sprites_por_texel: (usize, usize),
    /// ⭐ Os caminhos vetoriais que bloqueiam, desenhados na grade (W3).
    vetores: crate::vetor::CamadaVetorial,
    /// A cena de smoke armada já foi encenada (uma vez por sessão).
    smoke_encenado: bool,
    /// A recusa do solver já foi DITA ao artista (o aviso sai na transição, não a cada quadro).
    recusa_dita: bool,
    /// A re-coleta ([`Self::recolher`]) achou a cena SEM `Luz` depois da codificação: este quadro
    /// fica apagado (receptores neutros), nunca preto.
    sem_luz_no_quadro: bool,
    /// ⭐ A preferência do editor (`None` = degrau fixo, o da sonda/dos gates) e o governador.
    preferencia: Option<PreferenciaDaLuz>,
    governador: Governador,
    /// O último custo de placa lido (ms, o solver inteiro; da grade atual) e quantas leituras.
    custo_ms: Option<f64>,
    leituras: u32,
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
            fina: 1,
            captura: 1,
            mundo_para_tela: Affine::IDENTITY,
            alvo_px: (0, 0),
            resolvido: None,
            relatorio: None,
            erro: None,
            sprites_por_texel: (0, 0),
            vetores: crate::vetor::CamadaVetorial::default(),
            smoke_encenado: false,
            recusa_dita: false,
            sem_luz_no_quadro: false,
            preferencia: None,
            governador: Governador::novo(NivelDaLuz::Baixa),
            custo_ms: None,
            leituras: 0,
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
        self.sem_luz_no_quadro = false;
        self.mundo_para_tela = mundo_para_tela;
        self.alvo_px = alvo_px;
        self.grade = if self.cena.luzes == 0 {
            None
        } else {
            grade_do_quadro(mundo_para_tela, alvo_px, self.degrau)
        };
        self.fina = fina_do_quadro(mundo_para_tela, self.degrau);
        self.captura = captura_do_quadro(mundo_para_tela, self.degrau);
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
            // ⭐ Na resolução da captura FINA dos opacos (W5a): a C0 das cascatas lê-a também (cada
            // subamostra cai num píxel cuja área já é a cobertura).
            let fator = self.degrau.vetor_px_por_texel * self.captura;
            self.vetores
                .desenhar(gpu, (g, fator), &self.cena, doc, xforms);
        }
    }

    /// ⭐ **A preferência do editor** (Settings ▸ Light), a cada quadro: o nível e o
    /// «Automático». Muda o degrau no próximo `preparar` (a grade nova recria o solver).
    pub fn preferir(&mut self, p: PreferenciaDaLuz) {
        if self.preferencia != Some(p) {
            self.preferencia = Some(p);
            self.governador = Governador::novo(p.nivel);
        }
        let nivel = if p.automatica {
            self.governador.nivel
        } else {
            p.nivel
        };
        self.degrau = crate::qualidade::degrau_do(nivel);
    }

    /// O último custo de placa da grade atual (ms), com ou sem o «Automático»; `None` antes da 1.ª
    /// leitura dela ou sem `TIMESTAMP_QUERY` (aí o automático fica no teto).
    #[must_use]
    pub fn custo_ms(&self) -> Option<f64> {
        self.custo_ms
    }

    /// Quantas leituras de custo o «Automático» já colheu — o governador lê CONTINUAMENTE, não uma
    /// vez.
    #[must_use]
    pub fn leituras_de_custo(&self) -> u32 {
        self.leituras
    }

    /// O nível em uso e se é automático (`None` = degrau fixo).
    #[must_use]
    pub fn qualidade(&self) -> Option<(NivelDaLuz, bool)> {
        let p = self.preferencia?;
        Some((
            if p.automatica {
                self.governador.nivel
            } else {
                p.nivel
            },
            p.automatica,
        ))
    }

    /// ⭐ **Porta 2a** — re-coleta a cena logo antes do solver. As edições do quadro (o Inspector, o
    /// arrasto do gizmo) aplicam-se DEPOIS da codificação; sem isto a luz respondia um quadro
    /// atrasado. A guarda e a grade ficam as da codificação (o documento já foi às faixas por elas):
    /// uma `Luz` nova acende no quadro seguinte, e a última apagada deixa ESTE quadro neutro.
    pub fn recolher(&mut self, world: &World) {
        if self.grade.is_none() {
            return;
        }
        self.cena = cena_de_luz(world);
        self.sem_luz_no_quadro = self.cena.luzes == 0;
        self.sem_luz.clear();
        self.sem_luz.extend(self.cena.nao_recebem.iter().copied());
    }

    /// A luz está acesa neste quadro (a cena tem `Luz` e o solver não recusou)?
    #[must_use]
    pub fn acesa(&self) -> bool {
        self.grade.is_some() && self.erro.is_none() && !self.sem_luz_no_quadro
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
        let Some(g) = self.grade.filter(|_| !self.sem_luz_no_quadro) else {
            return;
        };
        let (fina, captura) = (self.fina, self.captura);
        let mesmas = self.resolvido.as_ref().is_some_and(|r| {
            let c = r.solver.config();
            (c.grade.largura, c.grade.altura, c.fina, c.captura)
                == (g.largura, g.altura, fina, captura)
        });
        let pronto = if mesmas {
            let r = self.resolvido.as_mut().expect("verificado acima");
            r.solver.reposicionar(g.origem_m, g.texel_m)
        } else {
            self.resolvido = None;
            // A grade nova (outro nível, outra janela) tem outro custo: o lido era da anterior.
            self.custo_ms = None;
            let cfg = Config {
                fina,
                captura,
                ..Config::padrao_para(&gpu.device, g)
            };
            Solver::novo(gpu, cfg).map(|solver| {
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
        let (inicio, fim) = crate::grade::regiao_da_janela(
            r.solver.grade_da_fluencia(),
            self.mundo_para_tela,
            self.alvo_px,
        );
        r.solver.regiao_fina(inicio, fim);
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
        // ⭐ W5b (F4): os caminhos que tapam FORA da grade — só para a luz direta.
        if let Some((vista, distantes)) = self.vetores.distantes() {
            sprites
                .texturas
                .push((vista, ph2d_light2d_gpu::Amostragem::Exata));
            let t = sprites.texturas.len() - 1;
            sprites
                .distantes
                .extend(distantes.iter().cloned().map(|s| ph2d_light2d_gpu::SpriteNaC0 { textura: t, ..s }));
        }
        self.sprites_por_texel = (sprites.substituidos.len(), self.cena.sprites.len());
        self.relatorio = Some(
            r.solver
                .gravar_com_sprites(gpu, &mut enc, &self.cena, &sprites),
        );
        gpu.queue.submit([enc.finish()]);
        // ⭐ O custo de placa lê-se SEM ESPERAR (de há poucos quadros) sempre que há preferência do
        // editor — o Inspector mostra-o em todo nível; com o Automático desligado ficava congelado
        // no último nível automático (smoke do dono, 10/10). Só o governador depende do Automático.
        if let Some(p) = self.preferencia
            && let Some(t) = r.solver.tempos_sem_esperar(gpu)
        {
            self.custo_ms = Some(t.total());
            self.leituras += 1;
            if p.automatica {
                let custo = crate::qualidade::Custo {
                    por_nivel: t.por_nivel(),
                    fixo: t.fixo(),
                };
                self.governador.ler(custo, p.nivel);
            }
        }
    }

    /// O que os receptores leem; `None` = apagada.
    #[must_use]
    pub fn recebida(&self) -> Option<LuzRecebida<'_>> {
        if !self.acesa() {
            return None;
        }
        let r = self.resolvido.as_ref()?;
        // ⭐ A grade FINA: a fluência que os receptores leem vive nela (W4b).
        let g = r.solver.grade_da_fluencia();
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

    /// ⭐ **O que o artista precisa de saber sobre a lâmpada `e`** no último quadro (W4), do mais
    /// grave para o mais brando: a recusa do solver (a cena inteira apagada), o que a captura fez
    /// com ESTA lâmpada, e os sprites que entraram pelo quad. Recusa que só o terminal vê é botão
    /// mudo — cada caso aqui era um `eprintln!` ou nada.
    #[must_use]
    pub fn avisos_de(&self, e: Entity) -> Vec<AvisoDaLuz> {
        let mut out = Vec::new();
        if let Some(erro) = &self.erro {
            out.push(match erro {
                ErroSolver::GradeGrandeDemais { lado, limite } => AvisoDaLuz::GradeGrandeDemais {
                    lado: *lado,
                    limite: *limite,
                },
                ErroSolver::SemShaderF16 | ErroSolver::ConfigInvalida(_) => {
                    AvisoDaLuz::SolverIndisponivel
                }
            });
        }
        let indice = self
            .cena
            .luzes_emissoras
            .iter()
            .find(|(quem, _)| *quem == e)
            .map(|&(_, i)| i);
        if let (Some(k), Some(r)) = (indice, &self.relatorio) {
            for f in r.fora.iter().filter(|f| f.quem == Quem::Emissor(k)) {
                out.push(match f.motivo {
                    Motivo::PontoSemArea => AvisoDaLuz::PontoSemTamanho,
                    Motivo::MenorQueOTexel => AvisoDaLuz::PequenaDemaisParaOZoom,
                    Motivo::ForaDaGrade => AvisoDaLuz::ForaDaVista,
                });
            }
        }
        let (por_texel, total) = self.sprites_por_texel;
        if self.acesa() && por_texel < total {
            out.push(AvisoDaLuz::SpritesPeloQuad {
                com_quad: total - por_texel,
                total,
            });
        }
        out
    }

    /// ⭐ **A recusa NOVA deste quadro**, já em palavras (o aviso que sobe na tela): `Some` só no
    /// quadro em que o solver passa a recusar; enquanto a recusa dura, `None`.
    pub fn recusa_nova(&mut self) -> Option<String> {
        let Some(erro) = &self.erro else {
            self.recusa_dita = false;
            return None;
        };
        if self.recusa_dita {
            return None;
        }
        self.recusa_dita = true;
        Some(match erro {
            ErroSolver::GradeGrandeDemais { lado, limite } => ph2d_i18n::tr_with(
                "light2d.toast.grid_too_big",
                &[("lado", lado), ("limite", limite)],
            ),
            ErroSolver::SemShaderF16 | ErroSolver::ConfigInvalida(_) => {
                ph2d_i18n::tr("light2d.toast.solver_unavailable").to_string()
            }
        })
    }

    /// O solver vivo (a sonda lê os tempos dele).
    #[must_use]
    pub fn solver(&self) -> Option<&Solver> {
        self.resolvido.as_ref().map(|r| &r.solver)
    }
}
