//! ⭐⭐⭐ **A LEI DA REFERÊNCIA no pincel** — o adaptador entre a malha, o
//! [`Brush`]/[`Dab`] da casa e o gesto puro de [`ph2d_cloth::verlet_gesto`].
//!
//! ⭐⭐ **Ele é o caminho de OMISSÃO desde 2026-09-06** (`PH2D_CLOTH_LAW=vbd`
//! volta ao VBD de [`super::stroke_cloth`], para bissecar). A frase que estava
//! aqui — *«ligar por omissão antes de o gate de paridade existir seria trocar
//! um pincel reprovado por outro sem régua»* — continua verdadeira, e o gate
//! passou a existir: `a_paridade_com_o_oraculo_nao_regride` cobre os `53`
//! traços em duas listas, `28` dentro da barra e `25` abertos com o número
//! medido ao lado.
//!
//! # O que este ficheiro traduz, e só isso
//!
//! - **a malha** → posições em `f64`, normais actuais, o anel-1 de cada vértice
//!   (pelas ARESTAS das faces poligonais, como o alvo — confirmado pelo
//!   especificador em 06/09; `PH2D_CLOTH_ANEL=tri` bissecta) e a máscara;
//! - **o pincel** → o [`Pincel`] da lei: raio, força, curva, dureza, modo, área;
//! - **o dab** → o [`Passo`]: o cursor, o delta, a normal da área (a média das
//!   normais sob o pincel) e a pressão.
//!
//! A lei em si (a área, a banda, as forças, as âncoras, o solver) vive na
//! `ph2d-cloth`, gateada contra as 46 fixtures do oráculo — nada dela é
//! reescrito aqui.

use crate::{Brush, Dab, Falloff, SculptStroke, Symmetry};
use ph2d_cloth::V3;
use ph2d_cloth::verlet::{Solver, dist, norm, unit};
use ph2d_cloth::verlet_gesto::{Curva, Passo, Pincel, PincelTecido};
use ph2d_mesh::{Face, Mesh};

/// **O que muda de uma CÓPIA DE SIMETRIA para a outra**, num dab.
///
/// ⚠️ Existe para o dab não passar oito argumentos soltos: os quatro campos são
/// a mesma coisa — *qual das passagens de simetria é esta* —, e um deles
/// (`passagens`) é a CONTAGEM delas, que a área *Local* lê para saber quantas
/// vezes construir.
struct Copia {
    /// O centro do dab já espelhado.
    center: [f32; 3],
    /// O caminho do dab já espelhado — a diferença dos dois pontos 3D.
    path: V3,
    /// A direcção do OLHO já espelhada: o eixo da vista da projecção do `δ`.
    eye: V3,
    /// O índice desta passagem (a sessão dela vive em `SculptStroke::cloth_ref`).
    copy: usize,
    /// Quantas passagens o traço tem ao todo.
    passagens: u32,
}

/// **A sessão da lei da referência de UMA cópia de simetria, num traço.**
#[derive(Clone, Debug)]
pub(super) struct ClothRef {
    pub(super) tecido: PincelTecido,
    /// ⭐⭐⭐ **As normais da superfície que o TRAÇO ENCONTROU** (espec §4.2-ter).
    ///
    /// Dentro de um traço o pincel deforma a malha e **continua a ler estas** —
    /// só a normal da área do Push e a normal por vértice do Inflate as lêem, e
    /// as duas obedecem. ⛔ Recalculá-las por dab é o que fazia os dois errarem
    /// `0,24` contra o oráculo; com a fotografia, `0,002` e `0,008`.
    ///
    /// ⚠️ **É propriedade do TRAÇO, não do programa:** de um traço para o
    /// seguinte elas refrescam-se, e a superfície que o próximo encontra é a que
    /// este deixou. ⚠️ E o **filtro** de tecido faz o CONTRÁRIO (§7) — ali cada
    /// passo repete a preparação que o traço só corre ao começar.
    pub(super) normais: Vec<V3>,
}

/// A lei em vigor é a da referência? — lido UMA vez.
///
/// ⭐⭐ **É o caminho de OMISSÃO desde 2026-09-06**, e a razão é o portão de
/// paridade existir: `28` dos `53` traços do oráculo saem dentro da barra
/// (`a_paridade_com_o_oraculo_nao_regride`), sete deles ao bit, e os `25`
/// abertos estão nomeados com o número medido. ⚠️ *A lei «o que é novo nasce
/// desligado» valia enquanto não havia régua; ela existe.*
///
/// ⛔ **O que fica do outro lado não é «o antigo», é o REPROVADO:** a lei VBD
/// foi recusada pelo dono três vezes com foto (a agulha, o papel amassado, os
/// caroços) e nunca teve paridade medida contra o alvo em modo nenhum.
///
/// `PH2D_CLOTH_LAW=vbd` volta a ela, para bissecar.
pub(super) fn lei_referencia() -> bool {
    static ESCOLHA: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ESCOLHA.get_or_init(|| std::env::var("PH2D_CLOTH_LAW").as_deref() != Ok("vbd"))
}

/// **O cursor deste passo tem de ser RE-APANHADO na superfície?** (espec §4.3:
/// os modos de força e o Expand re-picam a cada passo; o Grab fica no pen-down
/// e o Snake Hook anda no plano de profundidade.) A shell pergunta isto para
/// escolher entre o passo re-picado e o `hook_step` de sempre.
///
/// ⚠️ **Ele pergunta ao PINCEL desde 2026-09-06.** Antes lia uma variável de
/// ambiente por [`std::sync::OnceLock`], o que o congelava na primeira leitura
/// do processo — *um modo escolhido no painel a meio da sessão nunca teria
/// chegado aqui*.
#[must_use]
pub fn cloth_repica(brush: &Brush) -> bool {
    lei_referencia() && brush.cloth_mode.repica()
}

/// O anel-1 vem da TRIANGULAÇÃO, em vez das arestas dos polígonos?
///
/// ⚠️ **A omissão é ARESTAS, e mudou em 06/09 por resposta do especificador:** o
/// alvo toma os vizinhos pelas faces POLIGONAIS (um quad interior tem 4, e as
/// diagonais entram só como restrições de PAR). A grelha triangulada tinha
/// casado o `Local` do oráculo por rigidez a mais, não por ser o mecanismo dele.
/// `PH2D_CLOTH_ANEL=tri` fica como bissecção.
fn anel_por_triangulacao() -> bool {
    static ESCOLHA: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ESCOLHA.get_or_init(|| std::env::var("PH2D_CLOTH_ANEL").as_deref() == Ok("tri"))
}

/// O anel-1 de `v` pelas ARESTAS das faces (ou pela triangulação, para bissecar).
pub(super) fn anel_de(mesh: &Mesh, v: u32) -> Vec<u32> {
    let adj = mesh.adjacency();
    if !anel_por_triangulacao() {
        // ⚠️⚠️ **A ORDEM do anel é a das FACES à volta do vértice** (espec
        // §3.1): por cada face que contém `v`, os DOIS cantos adjacentes a ele
        // naquela face, deduplicados. Ela fixa a ordem em que as restrições
        // entram na lista, e a lista é resolvida em Gauss-Seidel, que não comuta.
        //
        // ⭐ **É exactamente o que o `vert_verts` da casa já devolve** — ele é
        // construído percorrendo as faces do vértice e tomando o anterior e o
        // seguinte —, e por isso não há aqui lei própria a reescrevê-lo.
        // ⛔ Reescrevê-la foi tentado em 06/09 e a mutação que a apagava
        // SOBREVIVEU: as duas davam o mesmo vector, vértice a vértice. O que
        // fica é o gate que amarra a dependência
        // (`o_anel_vem_pela_ordem_das_faces_e_nao_pela_dos_indices`), porque um
        // `sort` do outro lado partiria o tecido em silêncio.
        return adj.vert_verts.neighbours(v as usize).to_vec();
    }
    let mut out = Vec::new();
    for &f in adj.vert_faces.neighbours(v as usize) {
        let face = &mesh.faces()[f as usize];
        for k in 0..face.tri_count() {
            let t = face.tri_at(k);
            if t.contains(&v) {
                out.extend(t.iter().copied().filter(|w| *w != v));
            }
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// O [`Pincel`] da lei, derivado do [`Brush`] da casa.
///
/// ⚠️ **`passagens` são as cópias de SIMETRIA do traço**, e a área *Local*
/// constrói a lista de restrições `passagens + 1` vezes (espec, emenda Q8) — é
/// daí que sai a rigidez que separa a *Local* da *Global*. ⚠️ As fixtures do
/// oráculo correm todas sem simetria (`1`), então a lei `n + 1` vem da espec e
/// **só o degrau `n = 1` está medido aqui**.
///
/// ⚠️ A força é o `strength` CRU (a lei eleva-o ao quadrado, espec §4.1), e não
/// o [`Brush::weight`] com a curva de força do modo — essa curva é de outros
/// verbos. A curva de queda mapeia o que existe dos dois lados; o resto cai na
/// `Suave`, que é a omissão do alvo.
/// A MESMA porta, aberta para a sonda de paridade do neutro — ⛔ nenhum caminho
/// de produto a chama.
#[cfg(test)]
pub(crate) fn pincel_de_para_sonda(brush: &Brush, passagens: u32) -> Pincel {
    pincel_de(brush, passagens)
}

fn pincel_de(brush: &Brush, passagens: u32) -> Pincel {
    Pincel {
        modo: brush.cloth_mode.modo(),
        area: brush.cloth_area.area(),
        falloff_forca: brush.cloth_force_falloff.falloff(),
        curva: match brush.falloff {
            Falloff::Constant => Curva::Constante,
            Falloff::Sharper => Curva::Aguda,
            _ => Curva::Suave,
        },
        raio: f64::from(brush.radius),
        forca: f64::from(brush.strength.clamp(0.0, 1.0)),
        dureza: f64::from(brush.hardness.clamp(0.0, 1.0)),
        flip: if brush.invert { -1.0 } else { 1.0 },
        limite: f64::from(brush.cloth_limit.clamp(0.1, 10.0)),
        banda: f64::from(brush.cloth_falloff.clamp(0.0, 1.0)),
        // ⚠️ **A lei recusa o pino fora da *Local*** (espec §2.3), e a porta é
        // aqui e não no painel: o painel pergunta o mesmo para não pintar um
        // interruptor morto, mas quem honra a recusa é quem constrói o pincel.
        pino: brush.cloth_pin && brush.cloth_area.offers_pin(),
        solver: Solver {
            massa: f64::from(brush.cloth_mass.clamp(0.01, 2.0)),
            amortecimento: f64::from(brush.cloth_damping.clamp(0.01, 1.0)),
            plasticidade: f64::from(brush.cloth_plasticity.clamp(0.0, 1.0)),
            // ⭐ O alvo fixa isto; aqui é do artista. Ver `Brush::cloth_sweeps`.
            varreduras: brush.cloth_sweeps.clamp(
                crate::ClothFilterProps::SWEEPS.0,
                crate::ClothFilterProps::SWEEPS.1,
            ),
            // ⚠️⚠️ **O TRAÇO FICA COM A LEI DO ALVO, e é uma escolha medida.** O
            // limitador de esticão nasceu no FILTRO porque é lá que o defeito
            // vive: ali a força é sustentada por centenas de quadros e o esticão
            // acumula sem tecto. Um dab de pincel tem raio, banda e um número de
            // passos que o cursor limita — ⛔ e são as `86` fixtures dele que
            // provam a nossa paridade. *Ligar aqui sem o corpus que o meça seria
            // trocar o activo por um palpite.*
            estica_max: f64::INFINITY,
            passagens_limite: ph2d_cloth::verlet::PASSAGENS_LIMITE,
            // ⚠️ **O volume é do FILTRO**: o traço simula uma REGIÃO (área
            // *Local*), e o volume de uma peça inteira restringido por um punhado
            // de vértices livres num dab faria a peça respirar a cada pincelada.
            volume: 0.0,
        },
        passagens,
        ..Pincel::default()
    }
}

/// ⭐⭐⭐ **OS COLISORES, NO VOCABULÁRIO DA LEI** — a porta que o traço e o FILTRO
/// partilham.
///
/// ⚠️ **Ela nasceu porque a lei ganhou um SEGUNDO consumidor** (2026-09-08): a
/// espec §7 diz que o filtro de tecido **tem** colisões (*«idem §5.6, opção
/// nasce desligada»*) e nós passávamos-lhe uma lista vazia. Estas ~40 linhas
/// eram inline no traço; copiá-las daria duas ideias de *«onde este vértice
/// bate»*, e a que envelhecesse atravessaria o obstáculo em silêncio.
///
/// ⚠️ **O RAIO vai ao espaço LOCAL do colisor** e o acerto volta ao mundo:
/// transformar a malha inteira custaria uma cópia por peça e por gesto.
pub(crate) fn caixas_de(colisores: &[(ph2d_mesh::Mesh, ph2d_mesh::Pose)]) -> Vec<Caixa<'_>> {
    colisores
        .iter()
        .map(|(c, pose)| {
            let f = move |de: V3, ate: V3| -> Option<ph2d_cloth::verlet::Impacto> {
                let d = [ate[0] - de[0], ate[1] - de[1], ate[2] - de[2]];
                let comprimento = norm(d);
                if comprimento <= 0.0 {
                    return None;
                }
                // ⚠️ **O RAIO vai ao espaço LOCAL do colisor**, e o
                // acerto volta ao mundo: transformar a malha inteira
                // custaria uma cópia por peça e por traço.
                let mundo = ph2d_mesh::Ray::new(
                    [de[0] as f32, de[1] as f32, de[2] as f32],
                    [d[0] as f32, d[1] as f32, d[2] as f32],
                );
                let h = c.raycast(&pose.ray_to_local(&mundo))?;
                let ponto = v3(pose.point_to_world(h.point));
                // ⚠️ **Só conta DENTRO do comprimento do raio** (espec
                // §5.6), e ⛔ a comparação é feita no MUNDO: o `t` do
                // `Hit` mede em unidades LOCAIS, e o doc dele avisa
                // que comparar `t` entre escalas dá a resposta errada.
                if dist(ponto, de) > comprimento {
                    return None;
                }
                Some(ph2d_cloth::verlet::Impacto {
                    ponto,
                    // ⚠️ A normal do `Hit` **não é garantidamente
                    // unitária** (o doc dela di-lo), e a lei do §5.6
                    // afasta o vértice `0,005` ao longo dela.
                    normal: unit(v3(pose.vector_to_world(h.normal))),
                })
            };
            Box::new(f) as Caixa<'_>
        })
        .collect()
}

/// Um colisor já com dono — o `Vec` local tem de o segurar enquanto a lei corre.
type Caixa<'a> = Box<dyn Fn(V3, V3) -> Option<ph2d_cloth::verlet::Impacto> + 'a>;

fn v3(p: [f32; 3]) -> V3 {
    [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])]
}

impl SculptStroke {
    /// **O DAB pela lei da referência** — a porta que [`super::stroke_cloth`]
    /// desvia para cá quando [`lei_referencia`] está ligada.
    pub(super) fn cloth_ref_dab(
        &mut self,
        mesh: &mut Mesh,
        brush: &Brush,
        dab: &Dab,
        sym: Symmetry,
    ) -> usize {
        let (signs, n) = sym.signs();
        self.moved.clear();
        for (copy, s) in signs.iter().take(n).enumerate() {
            let center = [
                dab.center[0] * s[0],
                dab.center[1] * s[1],
                dab.center[2] * s[2],
            ];
            let path = [
                f64::from(dab.path[0] * s[0]),
                f64::from(dab.path[1] * s[1]),
                f64::from(dab.path[2] * s[2]),
            ];
            // ⚠️ **O OLHO espelha com a cópia**, e é por isso que ele vive no
            // [`Dab`] e não no [`Brush`]: guardá-lo no pincel daria a MESMA
            // direcção às duas cópias, e a metade espelhada seria projectada
            // por um olho que não é o dela.
            let eye = [
                f64::from(dab.eye[0] * s[0]),
                f64::from(dab.eye[1] * s[1]),
                f64::from(dab.eye[2] * s[2]),
            ];
            self.cloth_ref_copy(
                mesh,
                brush,
                dab,
                &Copia {
                    center,
                    path,
                    eye,
                    copy,
                    passagens: u32::try_from(n).unwrap_or(1),
                },
            );
        }
        // O tecido move geometria — a janela de upload é a das posições.
        self.last_paints_mask = false;
        if self.moved.is_empty() {
            return 0;
        }
        mesh.refresh_region(&self.moved, &mut self.region);
        self.moved.len()
    }

    /// Uma cópia de simetria: a sessão nasce no 1.º dab, corre um passo, escreve.
    fn cloth_ref_copy(&mut self, mesh: &mut Mesh, brush: &Brush, dab: &Dab, copia: &Copia) {
        let Copia {
            center,
            path,
            eye,
            copy,
            passagens,
        } = *copia;
        if self.cloth_ref.len() <= copy {
            self.cloth_ref.resize_with(copy + 1, || None);
        }
        let cursor = v3(center);
        let pos: Vec<V3> = mesh.positions().iter().map(|p| v3(*p)).collect();
        if self.cloth_ref[copy].is_none() {
            let caras: Vec<&[u32]> = mesh.faces().iter().map(Face::verts).collect();
            let tecido = PincelTecido::pen_down(
                pincel_de(brush, passagens),
                &pos,
                cursor,
                ph2d_cloth::particao::ordem_de_visita(&pos, &caras),
            );
            // ⭐⭐⭐ **A ORDEM DE VISITA da malha** (espec §3.1-bis) — a partição em
            // células da árvore espacial. ⛔ **Não é uma optimização: é metade da
            // lei.** Ela fixa a ordem em que as restrições entram na lista, a
            // lista é resolvida em Gauss-Seidel, e Gauss-Seidel não comuta —
            // medido nas 65 fixtures do oráculo, varrer por índice crescente
            // deixa `30` traços acima da barra e pela célula ficam `15`.
            //
            // ⚠️ **É propriedade da MALHA, e por isso é derivada UMA vez, no
            // pen-down**, sobre as posições que passam a ser o repouso do traço.
            let normais = mesh.normals().iter().map(|n| v3(*n)).collect();
            let mut tecido = tecido;
            // ⭐⭐⭐ **A BASE PERSISTENTE entra no PEN-DOWN e em mais lado nenhum**
            // (espec §6.4): as quatro leituras que a lêem vivem todas na
            // construção das restrições, que acontece aqui.
            //
            // ⚠️ **A validação é o COMPRIMENTO**, não uma bandeira: um remesh muda
            // a contagem de vértices e a base deixa de descrever a malha ⇒ a lei
            // cai no repouso do traço sozinha. E ligar a opção **sem** base
            // gravada é um no-op exacto, que é o que a espec §6.4 mede.
            if brush.cloth_persistent && self.persistent_base.len() == pos.len() {
                tecido.sim.base = self.persistent_base.iter().map(|p| v3(*p)).collect();
            }
            self.cloth_ref[copy] = Some(ClothRef { tecido, normais });
        }
        let Some(mut ses) = self.cloth_ref[copy].take() else {
            return;
        };
        // ⚠️ **A máscara E o alpha entram pela MESMA porta, e a cada dab.** A lei
        // da casa é que *o alpha é mais um peso por-vértice, como a máscara*
        // ([`super::stroke_cloth`]), e o carimbo do alpha anda com o traço —
        // por isso isto não pode ser feito uma vez no pen-down. `1` = imóvel.
        //
        // ⚠️ **Sem isto o pincel de tecido era o ÚNICO verbo que ignorava o
        // alpha**, e o censo `every_verb_reads_the_alpha` diz-o pelo nome: o
        // buraco só apareceu quando esta lei passou a ser a de omissão.
        let frame = brush.alpha_frame();
        let usa_alpha = brush.alpha.is_some();
        if mesh.masks().is_some() || usa_alpha {
            let livre = mesh.masks().map(<[f32]>::to_vec);
            ses.tecido.mascara = (0..pos.len())
                .map(|v| {
                    let m = livre
                        .as_ref()
                        .map_or(1.0, |l| f64::from(crate::mask_ops::free_weight(l[v])));
                    let a = if usa_alpha {
                        f64::from(brush.alpha_weight(mesh.positions()[v], &frame))
                    } else {
                        1.0
                    };
                    1.0 - m * a
                })
                .collect();
        }

        // As normais ACTUAIS por vértice — o Inflate lê-as uma a uma, e a NORMAL
        // DA ÁREA sai delas dentro da lei (espec §4.2-bis).
        //
        // ⛔ **Ela deixou de ser calculada aqui em 2026-09-06**, e não por
        // arrumação: a média das normais no disco INTEIRO — que era o que este
        // sítio fazia — não é a lei. O alvo amostra num disco de **metade** do
        // raio, pesa cada normal, e reparte os vértices em dois baldes pelo lado
        // a que estão virados, ficando com o PRIMEIRO que seja não-vazio e de
        // soma não-nula. *Nenhuma dessas três coisas cabe num chamador que só
        // sabe somar.*
        // ⭐ A fotografia do pen-down, e ⛔ **nunca** `mesh.normals()` de agora —
        // ver [`ClothRef::normais`] e a espec §4.2-ter.
        let normais: &[V3] = &ses.normais;
        // ⭐⭐⭐ **`δ` é a PROJECÇÃO do caminho no plano do ECRÃ** (espec §4.3,
        // emenda Q12), e o eixo da vista é a direcção do olho deste dab. As duas
        // des-projecções do alvo são feitas à mesma profundidade, logo a
        // componente ao longo do olho é descartada por construção. ⚠️ Numa folha
        // plana vista de frente as duas coisas são a MESMA ao bit; numa
        // superfície curva separam-se `15,83°` nas fixtures do oráculo.
        // ⛔ Só o ARRASTO lê o caminho 3D — dele sai a direcção dele.
        // ⛔⛔⛔ **O AGARRAR LEVA O `δ` TOTAL, e o `Dab` só sabe dar o incremento**
        // (espec §4.3: *«para o Grab o delta ACUMULA desde o pen-down; para os
        // outros sete modos é incremental»*).
        //
        // ⚠️⚠️ **Até 07/09 o produto entregava-lhe o incremento**, e o Grab do
        // artista movia `0,0147` onde a lei move `0,1690` — **`11,5×` menos**. E
        // nenhum gate o via: a bancada constrói o delta total no laço dela, e o
        // gate de costura `o_produto_corre_a_lei_do_oraculo` não tinha nenhum
        // traço de Agarrar na lista. *Cinco traços a `10⁻⁶` não dizem nada sobre
        // o sexto modo.*
        //
        // ⚠️ O `Dab::path` é o incremento **por definição** (o doc dele diz-o), e
        // é a coisa certa para os outros sete — quem sabe que este modo quer
        // outra é quem tem a sessão, porque só ela guarda o pen-down.
        let path = if brush.cloth_mode.leva_o_delta_total() {
            let (c, i) = (cursor, ses.tecido.inicio);
            [c[0] - i[0], c[1] - i[1], c[2] - i[2]]
        } else {
            path
        };
        let k = path[0] * eye[0] + path[1] * eye[1] + path[2] * eye[2];
        let delta = [
            path[0] - eye[0] * k,
            path[1] - eye[1] * k,
            path[2] - eye[2] * k,
        ];
        let passo = Passo {
            cursor,
            delta,
            delta_3d: path,
            parado: norm(delta) == 0.0,
            // ⚠️ **A direcção da SUPERFÍCIE PARA O OLHO** — o `Dab` guarda o olho
            // a apontar para dentro da peça, e a convenção da espec §4.2-bis é a
            // oposta (`n̂ · v̂ > 0` é o balde da frente).
            vista: [-eye[0], -eye[1], -eye[2]],
            normais,
            pressao: f64::from(dab.pressure.clamp(0.0, 1.0)),
        };
        // ⭐⭐ **OS COLISORES** (espec §5.6) — as outras peças da cena, na pose em
        // que a simulação nasceu. ⚠️ **Cada um é uma FUNÇÃO** e não uma malha: a
        // busca é de quem tem a cena, a correcção é da lei.
        //
        // ⛔⛔ **DIVERGÊNCIA DECLARADA:** a espec dá ao *cast* uma **espessura de
        // raio** de `0,3` em unidades de mundo, e o `Mesh::raycast` desta casa
        // lança um raio FINO. Num colisor fino e visto de raspão o nosso passa e
        // o do alvo apanharia. *Fica nomeado em vez de arrumado como igual* — e
        // ⚠️ não há fixture de colisor no corpus, logo nada disto tem lado
        // aprovado (o gate 53 declara-se de ESPEC pela mesma razão).
        // ⚠️ **Os colisores saem do `self` durante o passo**, como a sessão logo
        // acima: as funções que a lei recebe emprestam-nos, e o `capture` do undo
        // logo a seguir precisa do `&mut self`. *Tirar e repor é o que deixa as
        // duas coisas coexistirem sem uma cópia.*
        let colisores = std::mem::take(&mut self.cloth_colliders);
        let simulou = {
            let anel = |v: u32| anel_de(mesh, v);
            if brush.cloth_collisions && !colisores.is_empty() {
                let fs = caixas_de(&colisores);
                let refs: Vec<ph2d_cloth::verlet::Colisor> =
                    fs.iter().map(std::convert::AsRef::as_ref).collect();
                ses.tecido.passo_com_colisores(&pos, &anel, &passo, &refs)
            } else {
                ses.tecido.passo(&pos, &anel, &passo)
            }
        };
        self.cloth_colliders = colisores;
        if simulou {
            // ⚠️ Todo vértice ACTIVO é capturado antes de ser escrito: o `pre` é
            // o que o undo devolve (a mesma lei do `build_cloth` do VBD).
            for v in 0..ses.tecido.sim.activo.len() {
                if ses.tecido.sim.activo[v] {
                    self.capture(mesh, u32::try_from(v).unwrap_or(u32::MAX));
                }
            }
            let out = mesh.positions_mut();
            for (v, act) in ses.tecido.sim.activo.iter().enumerate() {
                if !*act {
                    continue;
                }
                let p = ses.tecido.sim.x[v];
                let novo = [p[0] as f32, p[1] as f32, p[2] as f32];
                if out[v] != novo {
                    out[v] = novo;
                    self.moved.push(u32::try_from(v).unwrap_or(u32::MAX));
                }
            }
        }
        self.cloth_ref[copy] = Some(ses);
    }
}
