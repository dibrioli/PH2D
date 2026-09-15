//! ⭐⭐⭐ **O MAPA imagem-px → ecrã que a ARTE DOBRADA manda** — a porta de quem desenha CHROME por
//! cima do canvas do Painter.
//!
//! # ⏸️ **DORMENTE desde 2026-09-15 — e a decisão é do dono, não um acidente**
//!
//! ⛔⛔ **Enquanto pintar ACHATAR a arte, esta porta não tem sujeito.** Por ordem do dono
//! (2026-09-15, [`ph2d_app_painter::skin_suspend`]) a sprite que o Painter edita não recebe malha,
//! e **todo o chrome deste módulo volta ao afim do quad — byte a byte** (é o caminho medido e
//! gateado por `a_segment_over_a_flat_quad_is_still_one_straight_line`).
//!
//! ⚠️⚠️ **Leia o resto deste cabeçalho com isso na mão:** tudo o que ele afirma sobre *«o chrome
//! segue a arte dobrada»* está CERTO e hoje **não acontece**, porque debaixo do pincel não há
//! dobra nenhuma. *Um doc que descreve um programa que não corre é a forma mais cara de mentira
//! deste repo.*
//!
//! ⭐ **Por que fica, medido:** sem malha ela é **exactamente** o que restaria se fosse apagada, logo
//! manter custa `0` em tempo de execução; apagar custaria os gates que PROVAM que o caminho recto é
//! exacto, e uma semana no dia em que pintar sobre a dobra voltar. ⛔ E ela **não pode ser apagada
//! por inteiro de qualquer forma**: a maquinaria por baixo (`ph2d_render::mesh_uv`,
//! `drawn_instance_of`, `ph2d_sprite_screen::uv_sob_o_ponteiro`) serve a **Remoção de fundo**, que
//! NÃO é achatada — o conta-gotas dela, o pincel de proteção, o *Add area* e a tinta da máscara.
//!
//! ⚠️ **A nota tem INSTRUMENTO**: o gate `the_dormant_door_says_so_and_the_note_dies_with_the_flattening`
//! ata-a ao achatamento nas DUAS direcções — tirar o achatamento sem tirar esta nota reprova.
//!
//! ⛔⛔ **O ponteiro foi curado em 2026-09-14 e o chrome não** (medido 2026-09-15): a entrega do
//! ponteiro passou a resolver o clique pela malha posada, e o editor de curva continuou a pintar os
//! pontos de controlo pelo afim do **quad de repouso**. ⇒ o artista clicava num sítio e o ponto
//! aparecia noutro, longe da tinta que ele próprio acabara de pousar.
//!
//! ⚠️⚠️ **E a metade que torna isto obrigatório é a do DEDO, não a do olho:** as alças do editor são
//! agarradas pelo `deliver_canvas_pointer`, que hoje resolve pela MALHA. Curar só uma das duas
//! direcções deixa um controlo **desenhado por um mapa e agarrado por outro** — a espécie de
//! controlo morto que o §5.0 diz que nenhuma sonda deste repo apanha. *As duas viajam juntas ou
//! nenhuma viaja.*
//!
//! ⚠️ **O afim FICA, e não é dívida:** ele é a lei de uma sprite que se desenha como QUAD, e carrega
//! a grelha da folha desdobrada que esta porta não conhece. A malha só responde onde ela existe.
//!
//! ⭐⭐ **E ele SUBDIVIDE desde 2026-09-15** ([`CanvasMap::segment`]). O cabeçalho anterior declarava
//! isto como limite — *«um ponto atravessa exactamente; um SEGMENTO é desenhado recto»* — e para a
//! espinha da curva era inofensivo **por construção** (ela já chega achatada em muitos pontos).
//! ⛔ Para a GRELHA não era: uma linha que atravessa o canvas inteiro é UM segmento, e sobre uma
//! dobra ela saía recta por cima de arte curva. ⇒ o segmento é agora partido até o desvio caber
//! na tolerância, e **num quad de repouso ele continua a emitir um ponto só, ao bit**: ali o mapa
//! é AFIM, e um afim leva recta em recta por definição.

use ph2d_render::Camera2d;
use ph2d_vector::{Affine, Point};

/// Ver o cabeçalho do módulo.
#[derive(Clone, Copy)]
pub struct CanvasMap<'a> {
    /// A lei do QUAD DE REPOUSO: imagem-px → ecrã (tamanho · escala · rotação · âncora · câmera).
    afim: Affine,
    iw: f64,
    ih: f64,
    /// A malha posada desta sprite, quando ela é desenhada como malha.
    malha: Option<(ph2d_render::DrawnMesh<'a>, Affine)>,
}

impl<'a> CanvasMap<'a> {
    /// O mapa desta sprite. `iw`/`ih` são o tamanho do canvas do Painter em pixels — é neles que os
    /// pontos autorados (as alças da curva) vivem.
    #[must_use]
    pub fn new(
        present: &'a ph2d_ecs::World,
        sim_entity_bits: u64,
        iw: u32,
        ih: u32,
        afim: Affine,
        camera: &Camera2d,
        window: ph2d_host::WindowSize,
    ) -> Self {
        let malha = (iw > 0 && ih > 0)
            .then(|| ph2d_render::drawn_mesh_of(present, sim_entity_bits))
            .flatten()
            .map(|m| (m, camera.world_to_screen_affine(window)));
        Self {
            afim,
            iw: f64::from(iw.max(1)),
            ih: f64::from(ih.max(1)),
            malha,
        }
    }

    /// O mapa de uma sprite **sem malha** — o afim de sempre, e nada mais. Para os chamadores que
    /// ainda não têm o mundo de apresentação à mão.
    #[must_use]
    pub fn rest(afim: Affine) -> Self {
        Self {
            afim,
            iw: 1.0,
            ih: 1.0,
            malha: None,
        }
    }

    /// **Um ponto autorado (imagem-px) no ECRÃ.** Pela malha onde ela o desenha; pelo afim do quad
    /// onde ela não o desenha (fora dela) ou não existe.
    #[must_use]
    pub fn point(&self, p: [f32; 2]) -> Point {
        if let Some((malha, mundo_para_ecra)) = &self.malha {
            let uv = [
                (f64::from(p[0]) / self.iw) as f32,
                (f64::from(p[1]) / self.ih) as f32,
            ];
            if let Some(w) = malha.world_at_uv(uv) {
                return *mundo_para_ecra * Point::new(f64::from(w[0]), f64::from(w[1]));
            }
        }
        self.afim * Point::new(f64::from(p[0]), f64::from(p[1]))
    }

    /// ⭐⭐⭐ **UM SEGMENTO autorado, no ECRÃ** — emite os pontos DEPOIS de `a`, já mapeados, em
    /// número bastante para o desvio caber na tolerância.
    ///
    /// ⚠️⚠️ **Num quad de repouso ele emite UM ponto, ao bit** — e não por um atalho escrito à mão:
    /// ali o mapa é um **AFIM**, e um afim leva recta em recta por definição, logo partir seria
    /// gastar sem mover um pixel. É isso que faz toda a chrome que já existe continuar byte a byte
    /// igual sobre uma sprite plana.
    ///
    /// ⭐⭐ **E o `return` de cima é POUPANÇA, não correcção — medido por mutação:** apagá-lo deixa a
    /// suíte inteira verde, porque sobre um afim o desvio é `0` e o [`Self::pedacos`] devolve `1`
    /// pela conta geral. ⇒ *a exactidão do caso plano cai da ÁLGEBRA, e o curto-circuito só evita
    /// três travessias por segmento.* O gate que a guarda
    /// (`a_segment_over_a_flat_quad_is_still_one_straight_line`) afirma a PROPRIEDADE e não o
    /// caminho, que é por isso que ele continua a valer com o atalho apagado.
    ///
    /// ⭐ **A tolerância é `0,5 px` de ECRÃ, e o número não é novo:** é o mesmo, e pela mesma razão,
    /// do refinamento da malha da pele (`ph2d_poly2d::RefineOptions`) — *abaixo de meio pixel o
    /// anti-aliasing da própria arte é mais largo que o erro*. Aqui ele ganha uma segunda razão do
    /// mesmo tamanho: a chrome é desenhada com uma caneta de `1,25 px`, então meio pixel de desvio
    /// mora **dentro da linha que o desenha**.
    pub fn segment(&self, a: [f32; 2], b: [f32; 2], mut emit: impl FnMut(Point)) {
        if self.malha.is_none() {
            emit(self.point(b));
            return;
        }
        let n = self.pedacos(a, b);
        for k in 1..=n {
            let t = f64::from(k) / f64::from(n);
            emit(self.point(interpola(a, b, t)));
        }
    }

    /// Em quantos pedaços este segmento tem de ser partido.
    ///
    /// ⭐⭐⭐ **A LEI NÃO É NOVA — é a do refinamento da malha da pele** (`ph2d_poly2d::refine`):
    /// mede-se o desvio uma vez, estima-se `n` por `√(d/tol)` (o desvio de uma corda cai com `h²`),
    /// e **confere-se UMA vez** sobre o que foi entregue. ⛔ *Um laço até convergir é trabalho por
    /// quadro sem tecto*, e a nota daquele ficheiro já o diz por escrito.
    ///
    /// ⚠️ **E a conferência não é zelo: ela foi exigida por um gate vermelho lá** — a lei `O(h²)`
    /// descreve a tendência, não o valor, e *um número que se chama tolerância e não é honrado é um
    /// número que mente ao artista*.
    ///
    /// ⛔ O tecto é o [`MAX_PEDACOS`], e o recurso dele é o **relógio do quadro**.
    fn pedacos(&self, a: [f32; 2], b: [f32; 2]) -> u32 {
        let n = estima(1, self.desvio(a, b, 1));
        if n <= 1 {
            return 1;
        }
        let d = self.desvio(a, b, n);
        if d <= TOLERANCIA_PX || n >= MAX_PEDACOS {
            return n;
        }
        estima(n, d).max(n + 1).min(MAX_PEDACOS)
    }

    /// O **pior desvio**, em px de ecrã, entre o meio de cada pedaço e a corda dele.
    fn desvio(&self, a: [f32; 2], b: [f32; 2], n: u32) -> f64 {
        let mut pior = 0.0_f64;
        for k in 0..n {
            let (t0, t1) = (f64::from(k) / f64::from(n), f64::from(k + 1) / f64::from(n));
            let (p0, p1) = (
                self.point(interpola(a, b, t0)),
                self.point(interpola(a, b, t1)),
            );
            let meio = self.point(interpola(a, b, f64::midpoint(t0, t1)));
            let corda = Point::new(f64::midpoint(p0.x, p1.x), f64::midpoint(p0.y, p1.y));
            pior = pior.max((meio.x - corda.x).hypot(meio.y - corda.y));
        }
        pior
    }

    /// ⭐⭐ **UMA POLILINHA autorada, no ECRÃ** — todos os pontos já mapeados, com cada troço
    /// subdividido pelo [`Self::segment`].
    ///
    /// `fechada` acrescenta o troço que volta ao primeiro ponto — ⚠️ **e ele é obrigatório para uma
    /// caixa**: quem a desenha chama `close_path`, que liga o último ao primeiro **a direito**, e
    /// sobre uma dobra essa aresta sairia recta enquanto as outras três seguem a arte. *Meia lei
    /// aplicada é pior que nenhuma, porque as três primeiras convencem o olho.*
    ///
    /// ⚠️ Num quad de repouso ela devolve exactamente os mesmos pontos que um `map` ponto a ponto
    /// sempre devolveu, sem um a mais — ver o [`Self::segment`].
    #[must_use]
    pub fn polyline(&self, pts: &[[f32; 2]], fechada: bool) -> Vec<Point> {
        let Some((&primeiro, resto)) = pts.split_first() else {
            return Vec::new();
        };
        let mut out = vec![self.point(primeiro)];
        let mut anterior = primeiro;
        for &p in resto {
            self.segment(anterior, p, |q| out.push(q));
            anterior = p;
        }
        if fechada && resto.len() >= 2 {
            // ⛔ O troço de fecho é emitido **menos o último ponto**: ele é o primeiro, e repeti-lo
            // punha um vértice duplicado no `close_path`.
            let mut volta = Vec::new();
            self.segment(anterior, primeiro, |q| volta.push(q));
            volta.pop();
            out.extend(volta);
        }
        out
    }

    /// O afim do quad de repouso — para quem precisa de uma ESCALA (o raio de uma alça em px de
    /// imagem) ou de desenhar uma IMAGEM, que não é um ponto e não atravessa esta porta.
    #[must_use]
    pub fn affine(&self) -> Affine {
        self.afim
    }

    /// **O mesmo mapa, deslocado no ECRÃ** — o que o *Repeat Image* precisa: o mesmo desenho
    /// autorado repetido nos ladrilhos vizinhos.
    ///
    /// ⚠️ **O deslocamento entra nas DUAS metades** (o afim do quad *e* a projecção da malha), senão
    /// um ladrilho desenharia as alças da malha por cima do ladrilho central enquanto a linha que as
    /// liga viajava — *meia lei aplicada é pior que nenhuma, porque as duas concordam no centro*.
    #[must_use]
    pub fn deslocado(&self, dx: f64, dy: f64) -> Self {
        let t = Affine::translate((dx, dy));
        Self {
            afim: t * self.afim,
            malha: self.malha.map(|(m, mundo)| (m, t * mundo)),
            ..*self
        }
    }

    /// `true` quando a arte por baixo é desenhada como MALHA — o que separa *«o afim está certo»* de
    /// *«o afim é o que sobra»*.
    #[must_use]
    pub fn is_mesh(&self) -> bool {
        self.malha.is_some()
    }
}

/// Quantos pedaços um desvio de `d` px pede, partindo de `n`: a lei `√(d/tol)` do
/// `ph2d_poly2d::refine`, saturada no [`MAX_PEDACOS`].
fn estima(n: u32, d: f64) -> u32 {
    if d <= TOLERANCIA_PX {
        return n;
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a razão é finita e positiva, e o `min` é o tecto"
    )]
    let k = (f64::from(n) * (d / TOLERANCIA_PX).sqrt()).ceil() as u32;
    k.clamp(n, MAX_PEDACOS)
}

/// Um ponto autorado entre `a` e `b`.
fn interpola(a: [f32; 2], b: [f32; 2], t: f64) -> [f32; 2] {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "os pontos autorados são f32; a conta faz-se em f64 e volta"
    )]
    [
        f64::from(a[0]).mul_add(1.0 - t, f64::from(b[0]) * t) as f32,
        f64::from(a[1]).mul_add(1.0 - t, f64::from(b[1]) * t) as f32,
    ]
}

/// ⭐ **Meio pixel de ECRÃ** — ver [`CanvasMap::segment`] para as duas razões.
const TOLERANCIA_PX: f64 = 0.5;

/// ⭐ **O TECTO DE PEDAÇOS de um segmento — e ele é uma REDE, não um orçamento.**
///
/// O recurso é o **relógio do quadro**: a chrome do Painter redesenha-se toda a cada quadro, e cada
/// pedaço custa uma travessia da malha ([`ph2d_render::DrawnMesh::world_at_uv`] varre triângulos).
///
/// Medido por `measure_the_price_of_a_subdivided_grid` (`--release`, `load 1,5`), uma grelha de
/// **40 linhas** que atravessam a dobra:
///
/// | malha | pedaços pedidos | ms | % de um quadro de 16,7 ms | µs por pedaço |
/// |---:|---:|---:|---:|---:|
/// | 32 tris | 1 400 | 0,136 | 0,8 % | 0,10 |
/// | 128 tris | 1 040 | 0,321 | 1,9 % | 0,31 |
/// | 512 tris | 720 | 1,035 | 6,2 % | 1,44 |
/// | 1 152 tris | 600 | 2,070 | 12,4 % | 3,45 |
///
/// ⭐⭐ **O achado é que a LEI DO DESVIO já se auto-limita: ela pediu `15`–`35` pedaços por linha em
/// todos os casos, e o tecto NUNCA chegou a morder.** Quem cresce quando a malha é grossa é o número
/// de pedaços; quem cresce quando ela é fina é o preço de cada um — e os dois puxam em sentidos
/// opostos. ⇒ este número existe para o caso em que a lei **não converge** (um mapa dobrado sobre si
/// mesmo), e não para apertar o caso normal.
///
/// **Onde o `64` vem:** a `3,45 µs` por pedaço (a malha mais fina medida), `40` linhas no tecto
/// custariam `8,8 ms` — **53 %** de um quadro. É o pior caso possível desta rede, e é ele que impede
/// o número de ser maior.
///
/// ⏳ **E a medição nomeia a obra seguinte:** o custo é `pedaços × triângulos` porque a travessia é
/// uma varredura LINEAR. Um índice de UV na [`ph2d_render::DrawnMesh`] tornaria cada pedaço `O(1)` e
/// a tabela acima ficaria plana — é a mesma forma da grelha de aceleração que o resto do app já usa,
/// e não foi construída aqui porque nenhum número desta tabela a exige ainda.
const MAX_PEDACOS: u32 = 64;
