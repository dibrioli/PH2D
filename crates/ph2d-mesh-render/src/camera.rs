//! A câmera **orbital** — a única forma de olhar uma escultura.
//!
//! ⚠️ **Ela mora aqui e o GESTO mora no shell.** Este tipo é estado + matrizes,
//! sem saber o que é um ponteiro; quem traduz *arrastar* em `orbit` é a shell,
//! nunca uma `Tool` (ADR-0150: o contrato congelado não é tocado, e navegar não
//! é esculpir — o artista gira o modelo com a ferramenta de pincel na mão).
//!
//! ⚠️ **Órbita, não voo livre.** Uma câmera 3D genérica tem 6 graus de liberdade
//! e um artista de escultura usa três: girar em torno do objeto, aproximar e
//! deslocar o centro. Os outros três são maneiras de se perder. É o que
//! ZBrush, Nomad e o SculptGL entregam, e o que este tipo torna *inexprimível*
//! em vez de meramente desencorajado.

use glam::{Mat4, Vec3};
use ph2d_mesh::Ray;

use crate::{Lens, ViewRegion};

/// Quão perto do polo a órbita pode chegar. Exatamente no polo a direção da
/// vista fica paralela ao `up` e a `look_at` **degenera** (produz `NaN`); esta
/// margem torna o caso impossível em vez de tratado.
const PITCH_LIMIT: f32 = core::f32::consts::FRAC_PI_2 - 0.01;

/// Fração do enquadramento que sobra de folga em volta do modelo.
const FRAME_MARGIN: f32 = 1.15;

/// A câmera que olha uma escultura.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera3d {
    /// O ponto em torno do qual se gira.
    pub target: Vec3,
    /// Distância do olho ao alvo. Sempre > 0.
    pub distance: f32,
    /// Azimute, em radianos, em torno do `+Y` do mundo.
    pub yaw: f32,
    /// Elevação, em radianos. Clampada longe dos polos.
    pub pitch: f32,
    /// Campo de visão VERTICAL, em radianos.
    ///
    /// ⚠️ **Ele é lido pelas DUAS lentes**, e é isso que faz a
    /// [`Camera3d::view_height`] querer dizer a mesma coisa nas duas: sob a
    /// paralela ele já não é um ângulo de convergência, é a régua que converte
    /// [`Camera3d::distance`] na extensão do quadro **no plano do alvo**.
    pub fov_y: f32,
    /// ⭐ **O que o olho faz com o que está longe** — ver [`Lens`].
    ///
    /// ⚠️ **Ela entra no [`crate::FormStamp`] por ser campo desta struct**, e isso
    /// é o que faz o G-buffer ser re-rasterizado quando o artista troca a lente.
    /// *Uma escolha de vista que não entra no carimbo é uma escolha que o assado
    /// ignora até alguém orbitar.*
    pub lens: Lens,
}

impl Default for Camera3d {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            distance: 3.0,
            yaw: 0.5,
            pitch: 0.4,
            fov_y: core::f32::consts::FRAC_PI_4,
            lens: Lens::Perspective,
        }
    }
}

impl Camera3d {
    /// ⭐⭐⭐ **O CIMA DO MUNDO** — o eixo em torno do qual esta câmera roda.
    ///
    /// ⚠️ **É uma PORTA e não uma preferência**, e a razão é que ela tinha
    /// **quatro** cópias neste ficheiro (`view`, `frame`, `ray_through`,
    /// `screen_basis`) e **nenhum nome** — logo, nada no repo a que outra
    /// metade do app pudesse perguntar *«para que lado é cima aqui?»*.
    ///
    /// ⛔⛔ **E a pergunta foi feita e respondida errado** (report do Enio,
    /// 2026-09-08: *«a gravidade está em z mas neste app deve ser em y»*): o
    /// filtro de tecido nasceu com o «baixo» em `−Z`, que é a convenção do
    /// ALVO da espec, não a desta casa. *Uma convenção sem nome é copiada da
    /// última coisa que se leu.*
    pub const UP: Vec3 = Vec3::Y;

    /// A câmera que enquadra `bounds` inteiro, com folga.
    #[must_use]
    pub fn framing(bounds: ph2d_mesh::Aabb, fov_y: f32, aspect: f32) -> Self {
        let mut cam = Self {
            fov_y,
            ..Self::default()
        };
        cam.frame(bounds, aspect);
        cam
    }

    /// Reenquadra para `bounds` preservando os ângulos — o "F" de *fit*.
    ///
    /// ⚠️ **Enquadra a CAIXA vista deste ângulo, não a esfera que a contém.** A
    /// primeira versão usava a esfera envolvente (`d ≥ r/sin(meio-fov)`), que é
    /// orientação-invariante e por isso tentadora — e **desperdiça `√3`**: para
    /// uma esfera de raio 1 a caixa é `[-1,1]³`, cuja meia-diagonal é 1,73, e o
    /// modelo saía ocupando **16,9% da tela** (medido pelo gate de GPU). A
    /// invariância só valeria a pena se o enquadramento rodasse todo frame; ele
    /// roda quando o artista aperta *fit*, uma vez.
    ///
    /// A conta exata: com o olho a `d` na direção `v`, uma quina `p` (relativa
    /// ao alvo) fica a profundidade `d − p·v` e a `p·right` de lado, então cabe
    /// quando `d ≥ p·v + |p·right| / tan(meio-fov_h)` — e o mesmo para a
    /// vertical. O `d` é o **máximo sobre as oito quinas e os dois eixos**.
    pub fn frame(&mut self, bounds: ph2d_mesh::Aabb, aspect: f32) {
        if bounds.is_empty() {
            return;
        }
        self.target = Vec3::from(bounds.center());

        let half_v = (self.fov_y * 0.5).max(0.01);
        let tan_v = half_v.tan();
        let tan_h = (tan_v * aspect.max(0.01)).max(1e-3);

        // Os eixos da câmera em mundo. `eye()` já depende de `distance`, mas só
        // a DIREÇÃO importa aqui, e ela é função de yaw/pitch apenas.
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let v = Vec3::new(cp * sy, sp, cp * cy); // alvo → olho
        let right = Self::UP.cross(v).normalize_or(Vec3::X);
        let up = v.cross(right);

        let half = (Vec3::from(bounds.max) - Vec3::from(bounds.min)) * 0.5;
        let mut d: f32 = 0.0;
        for i in 0..8 {
            let p = Vec3::new(
                if i & 1 == 0 { -half.x } else { half.x },
                if i & 2 == 0 { -half.y } else { half.y },
                if i & 4 == 0 { -half.z } else { half.z },
            );
            // ⚠️ **O termo de PROFUNDIDADE é da lente convergente e SÓ dela.** Sob raios
            // paralelos o tamanho na tela não depende da distância, logo uma quina mais perto
            // do olho não obriga a recuar — pedi-lo enquadraria a peça `p·v` unidades longe de
            // mais, que é o mesmo `√3` de desperdício que o doc acima já mediu noutro eixo.
            let depth = match self.lens {
                Lens::Perspective => p.dot(v),
                Lens::Ortho => 0.0,
            };
            d = d
                .max(depth + p.dot(right).abs() / tan_h)
                .max(depth + p.dot(up).abs() / tan_v);
        }
        self.distance = (d * FRAME_MARGIN).max(1e-3);
    }

    /// Onde o olho está.
    #[must_use]
    pub fn eye(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        self.target + Vec3::new(cp * sy, sp, cp * cy) * self.distance
    }

    /// A matriz de vista (mundo → olho).
    #[must_use]
    pub fn view(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye(), self.target, Self::UP)
    }

    /// A matriz mundo → clip.
    ///
    /// `perspective_rh` e não `perspective_rh_gl`: o clip do wgpu tem `z` em
    /// `[0,1]`, e a variante GL (`[-1,1]`) faria metade da profundidade cair
    /// atrás do plano near — o modelo apareceria cortado ao meio.
    #[must_use]
    pub fn view_proj(&self, aspect: f32) -> Mat4 {
        self.proj(aspect) * self.view()
    }

    /// A matriz vista → clip, **sozinha**.
    ///
    /// ⚠️ Ela existe porque o AO de tela precisa da INVERSA dela: a profundidade
    /// guarda `z` em clip e a marcha do GTAO acontece em espaço de VISTA, então
    /// alguém tem de desfazer a perspectiva. Uma segunda montagem da mesma
    /// `perspective_rh` do outro lado dessa fronteira divergiria no dia em que os
    /// planos de corte mudassem — e o `clip_planes` deriva os dele da DISTÂNCIA,
    /// ou seja muda a cada dolly.
    #[must_use]
    pub fn proj(&self, aspect: f32) -> Mat4 {
        let (near, far) = self.clip_planes();
        let aspect = aspect.max(0.01);
        match self.lens {
            Lens::Perspective => Mat4::perspective_rh(self.fov_y, aspect, near, far),
            // ⭐ **A meia-extensão sai do [`Self::view_height`]**, que é a régua que as duas
            // lentes partilham — é isso que faz trocar de lente não mudar o enquadramento no
            // plano do alvo, e é a lei que o modelador implícito já ship (ver [`Lens`]).
            Lens::Ortho => {
                let hv = self.view_height() * 0.5;
                Mat4::orthographic_rh(-hv * aspect, hv * aspect, -hv, hv, near, far)
            }
        }
    }

    /// ⭐⭐⭐ **A matriz mundo → clip de um RECORTE da vista** — o frustum fora do eixo.
    ///
    /// `aspect` é o da **vista INTEIRA**, nunca o do recorte: o recorte diz que pedaço dessa
    /// vista este alvo desenha, e a forma do frustum é da vista. ⚠️ Passar o aspecto do
    /// sub-rectângulo aqui devolveria uma imagem esticada que *parece* certa sozinha e não casa
    /// com o que está na tela.
    ///
    /// ⚠️ **Com [`ViewRegion::FULL`] devolve a [`Self::view_proj`] SEM a multiplicar** — ver o
    /// `//!` da [`crate::view_region`]: a matriz seria a identidade e o produto por ela é exacto
    /// **quase** sempre.
    #[must_use]
    pub fn view_proj_in(&self, aspect: f32, region: ViewRegion) -> Mat4 {
        self.proj_in(aspect, region) * self.view()
    }

    /// A matriz vista → clip de um RECORTE, **sozinha** — o irmão da [`Self::proj`], e ele existe
    /// pela mesma razão que ela: o AO de tela precisa da INVERSA, e a marcha do GTAO acontece em
    /// espaço de VISTA.
    ///
    /// ⚠️ **Com [`ViewRegion::FULL`] devolve a [`Self::proj`] sem a multiplicar**, e é isso que
    /// torna o caminho de omissão byte-idêntico **por construção** em vez de por sorte.
    #[must_use]
    pub fn proj_in(&self, aspect: f32, region: ViewRegion) -> Mat4 {
        if region.is_full() {
            return self.proj(aspect);
        }
        region.to_clip() * self.proj(aspect)
    }

    /// ⭐⭐ **QUANTO MUNDO A ALTURA DO QUADRO ABRANGE, NO PLANO DO ALVO.**
    ///
    /// ⚠️ **É a régua que as DUAS lentes partilham**, e a definição diz *«no plano do alvo»* de
    /// propósito: é ela que faz o [`Lens`] ser só uma lente — o zoom, o *fit* e o pan não mudam
    /// de lei, e as duas imagens coincidem **exactamente** naquele plano.
    ///
    /// ⛔ **Sob a paralela ela é a ÚNICA fonte de escala** (a distância deixa de a afectar por
    /// projecção), logo um *dolly* continua a ser zoom — o que é o que a mão espera.
    #[must_use]
    pub fn view_height(&self) -> f32 {
        2.0 * self.distance * (self.fov_y * 0.5).tan()
    }

    /// Os planos near/far, **derivados da distância** em vez de constantes.
    ///
    /// Um `near` fixo de 0,1 m destrói a precisão de profundidade ao aproximar
    /// de um modelo pequeno, e um `far` fixo corta um modelo grande. Ancorando
    /// os dois na distância do olho, a razão `far/near` fica constante — que é
    /// a grandeza de que a precisão do depth-buffer de fato depende.
    ///
    /// ⚠️⚠️ **Sob a lente PARALELA o `near` é NEGATIVO, e não é um descuido:** ali não há ponto
    /// de fuga, o plano do olho é uma escolha arbitrária, e uma peça mais funda do que a
    /// distância a que se está a olhar ficaria **cortada ao meio** sem nada a explicar. A laje
    /// fica centrada no ALVO (`±100 · distância`), que é onde a peça de facto está — o Blender
    /// documenta a mesma coisa (*«em ortográfica o Clip Start pode ser negativo»*).
    #[must_use]
    pub fn clip_planes(&self) -> (f32, f32) {
        match self.lens {
            Lens::Perspective => {
                let near = (self.distance * 0.01).max(1e-4);
                (near, self.distance * 100.0)
            }
            Lens::Ortho => {
                let half = (self.distance * 100.0).max(1e-3);
                (-half, half)
            }
        }
    }

    /// O raio que sai do olho e passa pelo pixel `(px, py)` — **o pick**.
    ///
    /// `px` cresce para a direita e `py` para BAIXO (a convenção de janela), e a
    /// conversão para NDC (`y` para cima) mora aqui: fazê-la no chamador é como
    /// nasce um pincel espelhado na vertical que ninguém entende.
    ///
    /// ⚠️ **Os eixos e o `tan` saem das MESMAS grandezas que a `view_proj` usa.**
    /// Se o raio fosse derivado de uma segunda cópia do frustum, o cursor e a
    /// imagem discordariam por um ângulo pequeno e constante — o defeito clássico
    /// de *"o pincel pinta ao lado de onde eu aponto"*, que nenhum teste de
    /// nenhuma das duas metades enxerga. O gate do round-trip
    /// (projeta um ponto → pega o pixel → dispara o raio de volta) é o que prende
    /// as duas.
    #[must_use]
    pub fn ray_through(&self, px: f32, py: f32, size: (u32, u32)) -> Ray {
        let (w, h) = (size.0.max(1) as f32, size.1.max(1) as f32);
        let ndc_x = 2.0 * px / w - 1.0;
        let ndc_y = 1.0 - 2.0 * py / h;
        let tan_v = (self.fov_y * 0.5).max(0.01).tan();
        let tan_h = tan_v * (w / h);

        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let v = Vec3::new(cp * sy, sp, cp * cy); // alvo → olho
        let right = Self::UP.cross(v).normalize_or(Vec3::X);
        let up = v.cross(right);

        match self.lens {
            Lens::Perspective => {
                let dir = right * (ndc_x * tan_h) + up * (ndc_y * tan_v) - v;
                Ray::new(self.eye().into(), dir.into())
            }
            // ⭐⭐ **Sob raios paralelos o que varia com o pixel é a ORIGEM, não a direcção** —
            // e é por isso que quem consome um raio tem de ler as duas coisas. Um consumidor
            // que só leia `dir()` funciona na convergente e mede o mesmo raio em todo o ecrã
            // aqui; ver [`Self::world_radius_for_screen_px`], que foi reescrita por causa disso.
            Lens::Ortho => {
                let hv = self.view_height() * 0.5;
                let o = self.eye() + right * (ndc_x * hv * (w / h)) + up * (ndc_y * hv);
                Ray::new(o.into(), (-v).into())
            }
        }
    }

    /// Onde um ponto do mundo cai na tela, em pixels — **o inverso exato do
    /// [`Self::ray_through`]**.
    ///
    /// `None` quando o ponto está atrás do olho (não há pixel para ele).
    ///
    /// ⚠️ Existe como PORTA e não como conta no chamador porque ela tem três
    /// consumidores que precisam concordar: o gate de round-trip, o diagnóstico
    /// do gesto, e qualquer cursor 3D futuro. Uma segunda cópia da conversão
    /// NDC→pixel é exatamente o que faz o cursor e a tinta discordarem.
    #[must_use]
    pub fn project(&self, p: [f32; 3], size: (u32, u32)) -> Option<(f32, f32)> {
        let (w, h) = (size.0.max(1) as f32, size.1.max(1) as f32);
        let clip = self.view_proj(w / h) * Vec3::from(p).extend(1.0);
        if clip.w <= 0.0 {
            return None;
        }
        let ndc = clip.truncate() / clip.w;
        Some(((ndc.x + 1.0) * 0.5 * w, (1.0 - ndc.y) * 0.5 * h))
    }

    /// O raio de MUNDO que mede `px` pixels de tela no ponto `at` — a conversão
    /// que faz um pincel **manter o tamanho aparente** quando a câmera aproxima.
    ///
    /// É o `computeWorldRadius2` do SculptGL (`Picking.js:378-387`), passo a
    /// passo: projeta o ponto, anda `px` pixels para o lado **na mesma
    /// profundidade**, e mede a distância de mundo entre os dois.
    ///
    /// ⚠️ **Ela NÃO re-deriva o frustum, e isso é o desenho.** A forma fechada
    /// (`2·d·tan(fov/2)·px / altura`) é mais curta e seria uma **segunda cópia**
    /// das mesmas grandezas que o [`Self::ray_through`] usa — exatamente o que o
    /// doc dele adverte, porque as duas divergiriam por um fator pequeno e
    /// constante que nenhum teste de nenhuma das metades enxerga. Aqui só entram
    /// as portas (`project`, `ray_through`) e o eixo da vista, que é a definição
    /// de `eye`/`target`.
    ///
    /// ⚠️ **O `t` do segundo raio é resolvido, não copiado do primeiro:** o
    /// [`Ray::new`] **normaliza** a direção, então dois raios com o mesmo `t`
    /// pousam em profundidades diferentes (o de fora está mais inclinado). O
    /// que iguala é a profundidade ao longo do eixo, e é ela que a conta pede.
    ///
    /// Devolve `0.0` para um ponto que não está à frente do olho — não há pixel
    /// para ele, então não há raio a converter.
    #[must_use]
    pub fn world_radius_for_screen_px(&self, at: [f32; 3], px: f32, size: (u32, u32)) -> f32 {
        let Some((sx, sy)) = self.project(at, size) else {
            return 0.0;
        };
        let eye = self.eye();
        let axis = (eye - self.target).normalize_or(Vec3::Z); // alvo → olho
        let depth = (Vec3::from(at) - eye).dot(-axis);
        // ⚠️ **A guarda é da lente CONVERGENTE**: ali um ponto atrás do olho não tem pixel (e o
        // `project` acima já o disse). Sob raios paralelos a laje é centrada no alvo e um ponto
        // atrás do plano do olho **continua a ser desenhado** — recusá-lo deixaria o pincel
        // inerte na metade de trás da peça assim que o artista aproximasse.
        if matches!(self.lens, Lens::Perspective) && depth <= 0.0 {
            return 0.0;
        }
        let ray = self.ray_through(sx + px, sy, size);
        // ⚠️⚠️ **A ORIGEM do raio entra na conta, e é a cura que a lente paralela obrigou:** ali
        // ela varia com o pixel e a direcção não, logo a versão antiga — que lia só a direcção —
        // media a mesma coisa em todo o ecrã e devolvia `0`.
        let o = Vec3::from(ray.origin());
        let side = Vec3::from(ray.dir());
        let along = side.dot(-axis);
        if along <= 1e-6 {
            return 0.0;
        }
        let t = (Vec3::from(at) - o).dot(-axis) / along;
        (o + side * t - Vec3::from(at)).length()
    }

    /// **QUANTO MUNDO A ALTURA DA TELA ABRANGE, POR UNIDADE DE PROFUNDIDADE** —
    /// a razão que define o frustum, `2·tan(fov/2)`.
    ///
    /// ⚠️ **Ela sai da porta que já existe, e NÃO da forma fechada.** A conta
    /// direta é mais curta e seria a segunda cópia das mesmas grandezas — o que o
    /// doc da [`Self::world_radius_for_screen_px`] já adverte, e as duas
    /// divergiriam por um fator pequeno e constante que nenhum teste de nenhuma
    /// das metades enxerga. Aqui a régua é medida no alvo, cuja profundidade é
    /// **exatamente** [`Self::distance`] por definição de `eye`/`target`.
    ///
    /// ⚠️ **E é uma RAZÃO, então o ponto onde ela é medida não importa** — a
    /// régua é proporcional à profundidade —, que é precisamente o que a torna
    /// útil a quem não pode escolher um ponto: um estêncil preso ao viewport é
    /// lido por vértice, em profundidades diferentes, e nenhuma delas é *a* certa.
    ///
    /// ⏳⏳ **DÍVIDA DECLARADA — sob a lente PARALELA esta grandeza não existe.** Ali o quadro
    /// abrange a mesma altura em toda profundidade, logo *«por unidade de profundidade»* não é
    /// uma pergunta. Medida, ela devolve `2·tan(fov/2)` nas duas lentes (a extensão sobre a
    /// distância do alvo), ou seja o **ÚNICO consumidor** — o estêncil do alpha por imagem
    /// ([`ph2d_sculpt3d::AlphaStencil::height_per_depth`]) — continua a carimbar como se a
    /// lente fosse convergente. ⚠️ **É o comportamento de hoje, não uma regressão**, e curá-lo
    /// pede um campo a mais naquele estêncil (`span = base + depth × razão`, com a família dos
    /// nove procedurais a passar `base = 0` e a ficar byte-idêntica) — wave própria.
    #[must_use]
    pub fn view_height_per_depth(&self, size: (u32, u32)) -> f32 {
        let h = self.world_radius_for_screen_px(self.target.into(), size.1.max(1) as f32, size);
        h / self.distance.max(1e-6)
    }

    /// O deslocamento de MUNDO que corresponde a `(dx, dy)` pixels de tela, na
    /// profundidade de `at` — **o gesto do Grab**.
    ///
    /// Irmão do [`Self::world_radius_for_screen_px`], e pela mesma razão: o dedo
    /// anda na TELA e o barro tem de andar junto, na profundidade em que ele
    /// está. Um delta de mundo derivado de outra forma faria o barro escapar do
    /// cursor conforme a câmera aproxima ou afasta.
    ///
    /// ⚠️ `dy` cresce para BAIXO (a convenção de janela, a mesma do
    /// [`Self::ray_through`]) e o mundo tem `y` para cima — a negação mora aqui,
    /// porque fazê-la no chamador é como nasce um Grab que puxa para o lado
    /// errado só na vertical.
    #[must_use]
    pub fn screen_delta_to_world(
        &self,
        at: [f32; 3],
        dx: f32,
        dy: f32,
        size: (u32, u32),
    ) -> [f32; 3] {
        let (right, up) = self.screen_basis();
        // A régua é a MESMA do raio de tela: quantos mundos vale um pixel ali.
        let per_px = self.world_radius_for_screen_px(at, 1.0, size);
        (right * (dx * per_px) + up * (-dy * per_px)).into()
    }

    /// **ONDE ESTÁ A TELA, EM MUNDO** — a direita e o cima do viewport.
    ///
    /// ⚠️ **Porta única, e ela nasceu porque ganhou um SEGUNDO consumidor.** O
    /// Grab já derivava esta base para levar o barro atrás do dedo; o estêncil
    /// do alpha precisa dela para se prender ao viewport. Duas derivações do
    /// mesmo par divergiriam no dia em que a convenção de *para cima* mudasse —
    /// e a que mente é a que o artista está olhando.
    #[must_use]
    pub fn screen_basis(&self) -> (Vec3, Vec3) {
        let axis = self.view_axis();
        let right = Self::UP.cross(axis).normalize_or(Vec3::X);
        (right, axis.cross(right))
    }

    /// ⭐ **O EIXO ALVO → OLHO**, unitário — *para que lado o observador está*.
    ///
    /// ⚠️ **Porta, e ela nasceu com o SEGUNDO consumidor** (o gizmo de navegação
    /// da escultura, 2026-09-08): ele precisa da terceira perna da base que o
    /// [`Self::screen_basis`] já derivava por dentro, e derivá-la no chamador
    /// seria a fórmula do [`Self::eye`] escrita uma quarta vez.
    #[must_use]
    pub fn view_axis(&self) -> Vec3 {
        (self.eye() - self.target).normalize_or(Vec3::Z)
    }

    /// Gira. `dx`/`dy` em radianos — a shell decide quantos radianos vale um
    /// pixel, porque é ela que conhece a densidade da tela.
    ///
    /// ⚠️ **O SENTIDO é do chamador, e ele não é óbvio.** `yaw` positivo leva o
    /// OLHO para `+X`, e a câmera indo para a direita faz o modelo *parecer* ir
    /// para a esquerda. Manipulação direta — *o modelo segue a mão*, que é o que
    /// ZBrush, Blender e SculptGL entregam — exige portanto `yaw -= dx` e
    /// `pitch += dy`. O gate que prende isto é
    /// `dragging_right_turns_the_model_right`, e ele mede o modelo NA TELA em
    /// vez de argumentar sobre sinais.
    pub fn orbit(&mut self, dx: f32, dy: f32) {
        self.yaw += dx;
        self.pitch = Self::clamp_pitch(self.pitch + dy);
    }

    /// ⭐⭐ **APONTA A CÂMERA** — o enquadramento NOMEADO, sem passar pelo gesto.
    ///
    /// ⚠️ **É uma porta, e não `cam.yaw = …; cam.pitch = …` no chamador**, porque
    /// o `pitch` tem uma trava física ([`Self::clamp_pitch`]) e quem escreve os
    /// campos à mão não a paga: uma vista de **topo** pedida como `π/2` exacto
    /// põe a direcção da vista paralela ao [`Self::UP`] e a `look_at` produz
    /// `NaN`. *Uma trava que só o gesto honra é uma trava que o primeiro
    /// chamador não-gesto quebra.*
    pub fn aim(&mut self, yaw: f32, pitch: f32) {
        self.yaw = yaw;
        self.pitch = Self::clamp_pitch(pitch);
    }

    /// ⭐ **A TRAVA DO POLO** — o `pitch` que esta câmera de facto consegue guardar.
    ///
    /// ⚠️ Ela é **pública** porque quem quiser reconhecer *«a câmera está na vista
    /// de topo?»* tem de comparar contra o valor **preso**, nunca contra o ideal:
    /// a `Top` pede `π/2` e a câmera guarda `π/2 − 0,01`, e um reconhecedor que
    /// comparasse com o ideal diria *não* sobre a vista que ele próprio acabou de
    /// pedir.
    #[must_use]
    pub fn clamp_pitch(pitch: f32) -> f32 {
        pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT)
    }

    /// Aproxima/afasta. `steps` positivo aproxima.
    ///
    /// **Multiplicativo, nunca aditivo:** um passo aditivo dá saltos enormes
    /// longe e não chega perto; multiplicativo, o gesto tem o mesmo efeito
    /// *aparente* em qualquer distância, que é o que a mão espera.
    pub fn dolly(&mut self, steps: f32) {
        const PER_STEP: f32 = 1.1;
        self.distance = (self.distance / PER_STEP.powf(steps)).clamp(1e-3, 1e6);
    }

    /// Desloca o alvo no plano da tela. `dx`/`dy` em FRAÇÃO da altura da
    /// viewport, para que arrastar o mesmo tanto de tela mova o mesmo tanto de
    /// modelo em qualquer zoom.
    pub fn pan(&mut self, dx: f32, dy: f32) {
        let view = self.view();
        // As linhas de uma matriz de vista ortonormal são os eixos da câmera
        // em coordenadas de mundo.
        let right = Vec3::new(view.x_axis.x, view.y_axis.x, view.z_axis.x);
        let up = Vec3::new(view.x_axis.y, view.y_axis.y, view.z_axis.y);
        // A altura do mundo que a viewport cobre à distância do alvo — a PORTA, e não a
        // conta escrita outra vez: é exactamente a mesma expressão, e por isso o pan é
        // byte-idêntico ao de antes desta lente existir.
        let world_per_frac = self.view_height();
        self.target += (-right * dx + up * dy) * world_per_frac;
    }
}

#[cfg(test)]
#[path = "camera_tests.rs"]
mod tests;

/// **A LENTE, e o valor ABSOLUTO da paralela** — ver o irmão.
#[cfg(test)]
#[path = "camera_lente_tests.rs"]
mod lente_tests;
