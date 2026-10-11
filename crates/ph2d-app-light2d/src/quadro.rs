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
    DEGRAU_PADRAO, Degrau, MARGEM_DO_ENTORNO, captura_do_quadro, fina_do_quadro, grade_do_quadro,
    tela_para_mundo,
};

struct Resolvido {
    solver: Solver,
    /// A vista da fluência (a textura é fixa enquanto o solver vive).
    vista: wgpu::TextureView,
}

/// A luz 2D do quadro do editor. Ver o cabeçalho da crate para as três portas.
pub struct LuzDoQuadro {
    /// O degrau EM USO: a [`Self::base`] com a fina engrossada o que os tetos e o governador pedem.
    degrau: Degrau,
    /// ⭐ W5e: o degrau que o nível (ou a sonda) pede, antes dos tetos do dispositivo.
    base: Degrau,
    /// ⭐ W5e: o lado de textura do dispositivo (o da última placa vista; antes, o padrão do wgpu).
    lado_max: u32,
    /// ⭐ W5e: os bytes que o dispositivo RECUSOU, e em que janela (outra janela esquece-os).
    recusou: Option<((u32, u32), u64)>,
    /// ⭐ W5e: a fina que os tetos impõem (`None` = nada cabe), pela chave de que ela depende.
    fina_dos_tetos: Option<(((u32, u32), Degrau, crate::teto::Tetos), Option<u32>)>,
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
    /// ⭐ W5d: o ENTORNO — o 2.º solver, mais grosso e mais largo, da luz que vem de FORA da grade.
    entorno: Option<Solver>,
    relatorio: Option<RelatorioC0>,
    erro: Option<ErroSolver>,
    /// `(por texel, sprites da cena)` no último quadro — os outros ficaram com o quad.
    sprites_por_texel: (usize, usize),
    /// ⭐ Os caminhos vetoriais que bloqueiam, desenhados na grade (W3).
    vetores: crate::vetor::CamadaVetorial,
    /// ⭐ A fábrica das pipelines, partilhada pelos solvers de cada grade: a janela redimensionada (o
    /// maximizar muda o tamanho a cada quadro) só realoca — recompilar custava ~2 s por quadro.
    fabrica: ph2d_light2d_gpu::Fabrica,
    /// A cena de smoke armada já foi encenada (uma vez por sessão).
    smoke_encenado: bool,
    /// ⭐ W5d: as formas que o smoke pôs no documento e cuja entidade ainda não foi marcada.
    semeadas: Vec<(ph2d_vec_scene::VecPathId, &'static str)>,
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
            base: degrau,
            lado_max: wgpu::Limits::default().max_texture_dimension_2d,
            recusou: None,
            fina_dos_tetos: None,
            cena: CenaDeLuz::default(),
            sem_luz: BTreeSet::new(),
            grade: None,
            fina: 1,
            captura: 1,
            mundo_para_tela: Affine::IDENTITY,
            alvo_px: (0, 0),
            resolvido: None,
            entorno: None,
            relatorio: None,
            erro: None,
            sprites_por_texel: (0, 0),
            vetores: crate::vetor::CamadaVetorial::default(),
            fabrica: ph2d_light2d_gpu::Fabrica::default(),
            smoke_encenado: false,
            semeadas: Vec::new(),
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
        if self.cena.luzes > 0 {
            self.caber(mundo_para_tela, alvo_px);
        }
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
        vetores: (&mut ph2d_vec_scene::VecScene, &ph2d_vec_scene::VecXforms),
    ) -> bool {
        if !self.smoke_encenado {
            self.smoke_encenado = true;
            if let Some(n) = crate::smoke::nivel_armado() {
                crate::smoke::encenar(world, n);
                self.semeadas = crate::smoke::semear_documento(vetores.0, n);
            }
        }
        if !self.semeadas.is_empty() {
            let pendentes = std::mem::take(&mut self.semeadas);
            self.semeadas = crate::smoke::marcar_semeadas(world, vetores.0, pendentes);
        }
        let alvo = superficie.size();
        self.lado_max = superficie.gpu().device.limits().max_texture_dimension_2d;
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
        self.base = crate::qualidade::degrau_do(nivel);
    }

    /// ⭐ W5e — o degrau em uso para esta janela: a base com a fina engrossada o MÍNIMO que cabe nos
    /// tetos do dispositivo ([`crate::teto`]) e, com o «Automático», o que o governador pede abaixo do
    /// Baixo. Nada cabe e o «Automático» ligado: desce o nível; manual, o solver recusa e o aviso diz.
    fn caber(&mut self, mundo_para_tela: Affine, alvo_px: (u32, u32)) {
        if self.recusou.is_some_and(|(alvo, _)| alvo != alvo_px) {
            self.recusou = None;
        }
        let tetos = crate::teto::Tetos {
            lado: self.lado_max,
            recusou: self.recusou.map(|(_, b)| b),
        };
        let automatica = self.preferencia.is_some_and(|p| p.automatica);
        let fina = loop {
            let chave = (alvo_px, self.base, tetos);
            let fina = match self.fina_dos_tetos {
                Some((c, f)) if c == chave => f,
                _ => {
                    let f = crate::teto::fina_que_cabe(mundo_para_tela, alvo_px, self.base, tetos);
                    self.fina_dos_tetos = Some((chave, f));
                    f
                }
            };
            if fina.is_some() || !automatica || !self.governador.descer() {
                break fina;
            }
            self.base = crate::qualidade::degrau_do(self.governador.nivel);
        };
        let max = self.base.fina_max();
        let minimo = fina.unwrap_or(max);
        let k = if automatica {
            self.governador.faixa_da_fina(minimo, max);
            self.governador.fina
        } else {
            minimo
        };
        self.degrau = self.base.com_fina_engrossada(k);
    }

    /// ⭐ W5e: quantas vezes a fina está engrossada neste quadro (0 = a do nível) e se foi pelo
    /// dispositivo (o aviso do Inspector).
    #[must_use]
    pub fn fina_engrossada(&self) -> (u32, bool) {
        let pelos_tetos = self
            .fina_dos_tetos
            .and_then(|(_, f)| f)
            .is_some_and(|f| f > 0);
        let k = (self.degrau.captura_px / self.base.captura_px)
            .log2()
            .round() as u32;
        (k, pelos_tetos)
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
        self.lado_max = gpu.device.limits().max_texture_dimension_2d;
        let Some(g) = self.grade.filter(|_| !self.sem_luz_no_quadro) else {
            return;
        };
        // ⭐ W5e: nada cabe abaixo do que a placa já recusou nesta janela — não se tenta alocar de novo
        // a cada quadro (outra janela ou outro nível tentam).
        if let (Some((_, None)), Some((_, bytes))) = (self.fina_dos_tetos, self.recusou) {
            self.resolvido = None;
            self.erro = Some(ErroSolver::SemMemoria { bytes });
            return;
        }
        let (fina, captura) = (self.fina, self.captura);
        // ⭐ W5c: a cascata holográfica (o degrau escolhe-a; a troca recria o solver).
        let holografica = self
            .degrau
            .holografica
            .then_some(ph2d_light2d_gpu::HOLOGRAFICA_PADRAO);
        let mesmas = self.resolvido.as_ref().is_some_and(|r| {
            let c = r.solver.config();
            (
                c.grade.largura,
                c.grade.altura,
                c.fina,
                c.captura,
                c.holografica,
                c.rebotes_por_quadro,
            ) == (
                g.largura,
                g.altura,
                fina,
                captura,
                holografica,
                self.degrau.rebotes_por_quadro,
            )
        });
        let pronto = if mesmas {
            let r = self.resolvido.as_mut().expect("verificado acima");
            r.solver.reposicionar(g.origem_m, g.texel_m)
        } else {
            self.resolvido = None;
            // A grade nova (outro nível, outra janela) tem outro custo: o lido era da anterior.
            self.custo_ms = None;
            let rebote = rebote_ligado();
            let cfg = Config {
                fina,
                captura,
                holografica,
                rebote,
                rebotes_por_quadro: self.degrau.rebotes_por_quadro,
                ..Config::padrao_para(&gpu.device, g)
            };
            Solver::novo_com(gpu, cfg, &self.fabrica).map(|solver| {
                let vista = solver
                    .fluencia()
                    .create_view(&wgpu::TextureViewDescriptor::default());
                self.resolvido = Some(Resolvido { solver, vista });
            })
        };
        self.erro = pronto.err();
        // ⭐ W5e: o dispositivo recusou a memória — o próximo `preparar` engrossa a fina abaixo disto.
        if let Some(ErroSolver::SemMemoria { bytes }) = self.erro {
            self.recusou = Some((self.alvo_px, bytes));
        }
        self.preparar_entorno(gpu, holografica);
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
                .extend(distantes.iter().map(|s| ph2d_light2d_gpu::SpriteNaC0 {
                    textura: t,
                    ..s.clone()
                }));
        }
        self.sprites_por_texel = (sprites.substituidos.len(), self.cena.sprites.len());
        // ⭐ W5d: o ENTORNO grava ANTES (no mesmo codificador): a c0 dele soma-se à desta.
        if let Some(e) = self.entorno.as_mut() {
            let gv = r.solver.config().grade;
            e.mascarar_miolo(
                gpu,
                Some([
                    gv.origem_m[0],
                    gv.origem_m[1],
                    gv.origem_m[0] + f64::from(gv.largura) * gv.texel_m,
                    gv.origem_m[1] + f64::from(gv.altura) * gv.texel_m,
                ]),
            );
            let sem_ceu = CenaDeLuz {
                ceu: [0.0; 3],
                ..self.cena.clone()
            };
            e.gravar_com_sprites(gpu, &mut enc, &sem_ceu, &sprites);
        }
        r.solver.somar_entorno(gpu, self.entorno.as_ref());
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
            // ⭐ W5d: o entorno faz parte do custo (fixo: não muda com o nível).
            let extra = self
                .entorno
                .as_mut()
                .and_then(|e| e.tempos_sem_esperar(gpu))
                .map_or(0.0, |t| t.total());
            self.custo_ms = Some(t.total() + extra);
            self.leituras += 1;
            if p.automatica {
                let custo = crate::qualidade::Custo {
                    por_nivel: t.por_nivel(),
                    fixo: t.fixo() + extra,
                };
                self.governador.ler(custo, p.nivel);
            }
        }
    }

    /// ⭐ W5d — o ENTORNO deste quadro: criado na grade dele ([`Degrau::entorno`] × o texel,
    /// [`MARGEM_DO_ENTORNO`] de cada lado), reposicionado se ela só andou, apagado sem ele (o degrau, a
    /// holográfica, `PH2D_LIGHT2D_ENTORNO=0` para comparar).
    fn preparar_entorno(
        &mut self,
        gpu: &GpuContext,
        holografica: Option<ph2d_light2d_gpu::Holografica>,
    ) {
        let ligado = self.degrau.entorno > 0.0
            && holografica.is_some()
            && std::env::var("PH2D_LIGHT2D_ENTORNO").map_or(true, |v| v.trim() != "0");
        let de = Degrau {
            c0_px: self.degrau.c0_px * self.degrau.entorno,
            margem: MARGEM_DO_ENTORNO,
            ..self.degrau
        };
        let Some(ge) = ligado
            .then(|| grade_do_quadro(self.mundo_para_tela, self.alvo_px, de))
            .flatten()
        else {
            self.entorno = None;
            return;
        };
        let mesma = self.entorno.as_ref().is_some_and(|e| {
            let c = e.config();
            (c.grade.largura, c.grade.altura) == (ge.largura, ge.altura)
        });
        if mesma {
            let e = self.entorno.as_mut().expect("verificado acima");
            if e.reposicionar(ge.origem_m, ge.texel_m).is_err() {
                self.entorno = None;
            }
            return;
        }
        let cfg = Config {
            holografica,
            rebote: rebote_ligado(),
            ..Config::padrao_para(&gpu.device, ge)
        };
        self.entorno = Solver::novo_com(gpu, cfg, &self.fabrica)
            .map(|mut s| {
                // Só as cascatas e a fonte do rebote: a grade fina dele não se lê.
                s.regiao_fina([0; 2], [0; 2]);
                s
            })
            .map_err(|e| eprintln!("[luz2d] o entorno recusou a grade {ge:?}: {e:?}"))
            .ok();
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
                ErroSolver::SemMemoria { bytes } => AvisoDaLuz::SemMemoria {
                    mb: bytes / 1_000_000,
                },
            });
        }
        if let (k @ 1.., true) = self.fina_engrossada()
            && self.acesa()
        {
            out.push(AvisoDaLuz::SombraEngrossada { vezes: 1 << k });
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
            ErroSolver::SemMemoria { bytes } => ph2d_i18n::tr_with(
                "light2d.toast.out_of_memory",
                &[("mb", &(bytes / 1_000_000))],
            ),
        })
    }

    /// O solver vivo (a sonda lê os tempos dele).
    #[must_use]
    pub fn solver(&self) -> Option<&Solver> {
        self.resolvido.as_ref().map(|r| &r.solver)
    }
}

/// ⭐ W5c: o rebote em todo nível (sem nenhum corpo que reflita, os passes dele saltam);
/// `PH2D_LIGHT2D_REBOTE=0` desliga-o (a comparação da cena 3).
fn rebote_ligado() -> bool {
    std::env::var("PH2D_LIGHT2D_REBOTE").map_or(true, |v| v.trim() != "0")
}
