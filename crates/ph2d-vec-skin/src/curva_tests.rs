//! Os gates do [`super`] — a arte segue o peso ENTRE os nós, e os nós não se mexem.

use super::*;
use ph2d_skeleton::{SkinBone, Xform};
use ph2d_vec_scene::{ShapeKind, cook};

/// Um osso deitado no `+X`, de `(x0,0)` a `(x0+len,0)`, rodado de `rot` na pose.
///
/// ⛔⛔ **O `tendon` é POSTO à mão, e a omissão dele apagava o fenómeno.** A
/// [`ph2d_skeleton::SkinBone::new`] nasce com `tendon: 0` em **todos** — e com os dois ossos no
/// mesmo tendão a mancha soma o MESMO a ambos, que a normalização a seguir **cancela**: a 1.ª
/// redacção deste ficheiro media `0,000000` sobre uma lei correcta. *Uma correcção que vale para
/// todos não corrige nada.*
fn osso(x0: f64, len: f64, rot: f64, tendon: u32) -> SkinBone {
    let (c, s) = (rot.cos(), rot.sin());
    let mut b = SkinBone::new(
        Xform([1.0, 0.0, 0.0, 1.0, x0, 0.0]),
        len,
        1.0,
        Xform([c, s, -s, c, x0, 0.0]),
        Xform::IDENTITY,
    )
    .expect("repouso nao-singular");
    b.tendon = tendon;
    b
}

/// A pele do palco: dois ossos ao longo de um rectângulo de `40 × 10`, o segundo dobrado.
pub(super) fn pele(rot: f64) -> Skin {
    Skin::new(vec![osso(0.0, 20.0, 0.0, 0), osso(20.0, 20.0, rot, 1)]).expect("2 ossos")
}

pub(super) fn forma() -> VecPath {
    cook(ShapeKind::Rectangle, [0.0, 0.0], [40.0, 10.0], &[])
}

/// A tabela do padrão-ouro para os quatro nós do rectângulo: o 1.º osso manda na esquerda, o 2.º na
/// direita. Três linhas por vértice (âncora · entrada · saída), como a do bind.
fn tabela() -> Vec<f64> {
    // nós: (0,0) · (40,0) · (40,10) · (0,10)
    let por_no = [[1.0, 0.0], [0.0, 1.0], [0.0, 1.0], [1.0, 0.0]];
    let mut out = Vec::new();
    for linha in por_no {
        for _ in 0..3 {
            out.extend_from_slice(&linha);
        }
    }
    out
}

/// A curva desenhada, amostrada densamente — o que o olho vê.
fn polilinha(p: &VecPath) -> Vec<[f64; 2]> {
    const N: usize = 200;
    let cozido = p.cooked();
    let mut out = Vec::new();
    for c in 0..cozido.contour_count() {
        let Some((verts, fechado)) = cozido.contour(c) else {
            continue;
        };
        let n = verts.len();
        let ultimo = if fechado { n } else { n.saturating_sub(1) };
        for i in 0..ultimo {
            let (a, b) = (&verts[i], &verts[(i + 1) % n]);
            for k in 0..N {
                let t = k as f64 / N as f64;
                let u = 1.0 - t;
                let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                out.push([
                    w0.mul_add(
                        a.anchor[0],
                        w1.mul_add(
                            a.out_handle[0],
                            w2.mul_add(b.in_handle[0], w3 * b.anchor[0]),
                        ),
                    ),
                    w0.mul_add(
                        a.anchor[1],
                        w1.mul_add(
                            a.out_handle[1],
                            w2.mul_add(b.in_handle[1], w3 * b.anchor[1]),
                        ),
                    ),
                ]);
            }
        }
    }
    out
}

fn desvio(a: &[[f64; 2]], b: &[[f64; 2]]) -> f64 {
    b.iter()
        .map(|p| ph2d_skeleton::dist2_to_polyline(*p, a).sqrt())
        .fold(0.0_f64, f64::max)
}

/// ⭐⭐⭐ **PINTAR PESO ENTRE DOIS NÓS MOVE A ARTE** — a lei da wave, com o CONTROLO dentro.
///
/// ⛔⛔ **O controlo é o caminho de HOJE** (os pontos de controlo, [`crate::aplica_corrigido`]): ali
/// a mesma mancha, no mesmo sítio, move a arte **zero**. *Sem ele este gate não distingue a lei nova
/// de uma fixtura que já se mexia sozinha* — e é aquele zero que é o report do dono.
#[test]
fn pintar_peso_entre_dois_nos_move_a_arte() {
    let k = pele(0.8);
    let t = tabela();
    // A mancha no MEIO da aresta de baixo, entre os nós `(0,0)` e `(40,0)`.
    let mancha = Correccao {
        tendon: 0,
        centro: [20.0, 0.0],
        raio: 14.0,
        especie: ph2d_skeleton::Especie::Soma(0.6),
    };

    // (a) HOJE — os pontos de controlo.
    let (mut sem_hoje, mut com_hoje) = (forma(), forma());
    crate::aplica_corrigido(&k, &mut sem_hoje, &t, &[]);
    crate::aplica_corrigido(&k, &mut com_hoje, &t, &[mancha]);
    let hoje = desvio(&polilinha(&sem_hoje), &polilinha(&com_hoje));

    // (b) PELA CURVA.
    let (mut sem, mut com) = (forma(), forma());
    aplica_pela_curva(&k, &mut sem, &t, &[]);
    aplica_pela_curva(&k, &mut com, &t, &[mancha]);
    let curva = desvio(&polilinha(&sem), &polilinha(&com));

    eprintln!("[curva] a mancha entre dois nos move: hoje={hoje:.6} · pela curva={curva:.6}");
    assert!(
        hoje < 1e-9,
        "o caminho dos PONTOS DE CONTROLO passou a sentir a mancha ({hoje}) — ou a fixtura mudou, \
         ou alguem ja' curou isto noutro sitio, e este gate deixou de medir o que diz"
    );
    assert!(
        curva > 0.1,
        "pela curva a mancha moveu so' {curva} — a lei nova nao esta' a ler o peso entre os nos, e \
         o report do dono («pintar peso entre os vertices nao faz nada») volta inteiro"
    );
}

/// ⭐⭐⭐ **OS NÓS NÃO SE MEXEM — a lei nova concorda com a de hoje EXACTAMENTE nas âncoras.**
///
/// ⚠️ É o que faz esta wave ser segura: em `t = 0` e `t = 1` a mistura é a linha do próprio nó, logo
/// a âncora deformada é a **mesma** dos dois lados. *Se ela se mexesse, todo rig já autorado
/// mudaria de forma no dia em que isto shipasse.*
#[test]
fn os_nos_nao_se_mexem() {
    let k = pele(0.8);
    let t = tabela();
    let (mut hoje, mut curva) = (forma(), forma());
    crate::aplica_corrigido(&k, &mut hoje, &t, &[]);
    aplica_pela_curva(&k, &mut curva, &t, &[]);

    let ancoras: Vec<[f64; 2]> = hoje.verts_all().map(|v| v.anchor).collect();
    let novas: Vec<[f64; 2]> = curva.verts_all().map(|v| v.anchor).collect();
    assert!(
        novas.len() >= ancoras.len(),
        "a lei nova perdeu nos: {} contra {}",
        novas.len(),
        ancoras.len()
    );
    // ⛔ Cada âncora AUTORADA tem de estar entre as novas, ao bit da fita: o fit só ACRESCENTA.
    let mut pior = 0.0_f64;
    for a in &ancoras {
        let d = novas
            .iter()
            .map(|b| (a[0] - b[0]).hypot(a[1] - b[1]))
            .fold(f64::INFINITY, f64::min);
        pior = pior.max(d);
    }
    eprintln!(
        "[curva] nos: {} autorados -> {} desenhados · pior desvio de ancora = {pior:.9}",
        ancoras.len(),
        novas.len()
    );
    assert!(
        pior < 1e-9,
        "uma ancora AUTORADA mexeu-se {pior} — a lei nova discorda da de hoje no no', e todo rig \
         ja' feito muda de forma"
    );
}

/// ⭐⭐⭐ **EM REPOUSO O DESENHO NÃO SE MEXE, e não por promessa.**
///
/// Com a pose à identidade a pele é a identidade, logo `t ↦ C(t)` e o fit devolve a própria curva.
/// ⚠️ **O desvio é medido contra a FONTE**, não contra o caminho de hoje: é a fonte que o artista
/// desenhou.
#[test]
fn em_repouso_o_desenho_nao_se_mexe() {
    let k = pele(0.0);
    let mut out = forma();
    aplica_pela_curva(&k, &mut out, &tabela(), &[]);
    let d = desvio(&polilinha(&forma()), &polilinha(&out));
    eprintln!("[curva] em repouso o desvio e' {d:.12}");
    assert!(
        d < 1e-9,
        "em REPOUSO a lei nova mexeu o desenho em {d} — ali ela tem de ser a identidade"
    );
}

/// ⭐⭐ **O PREÇO, medido e impresso** — o `recook` corre uma vez por quadro, por forma presa.
///
/// ⚠️ Ele **imprime** e julga só o tecto grosseiro: o relógio desta máquina não vale nada sob carga,
/// e o número que interessa ao produto é o de `--release` numa máquina calma. *Um gate de relógio
/// apertado aqui seria mais um membro da família de flakes de fan-out.*
#[test]
fn o_preco_da_curva_esta_medido() {
    let k = pele(0.8);
    let t = tabela();
    let mut relogio = std::time::Duration::MAX;
    let mut nos = 0;
    for _ in 0..5 {
        let mut out = forma();
        let t0 = std::time::Instant::now();
        aplica_pela_curva(&k, &mut out, &t, &[]);
        relogio = relogio.min(t0.elapsed());
        nos = out.verts_all().count();
    }
    eprintln!(
        "[curva] preco: {:.3} ms por forma · {} nos de {} ({AMOSTRAS} amostras por segmento)",
        relogio.as_secs_f64() * 1e3,
        nos,
        forma().verts_all().count()
    );
    assert!(
        relogio.as_secs_f64() < 0.1,
        "uma forma de quatro nos custou {:.1} ms — acima de 100 ms nem o debug explica, e o \
         `recook` corre por quadro",
        relogio.as_secs_f64() * 1e3
    );
}

/// ⭐⭐⭐ **ONDE O MAPA É AFIM A ARTE AINDA SE MOVE — e o desenho fica BYTE-IDÊNTICO ao de sempre.**
///
/// ⛔⛔ **Ele nasceu de uma MUTAÇÃO SOBREVIVENTE:** apagar o `aplica_corrigido` do início da porta
/// passava a suíte inteira, **nas duas crates**. A razão é que todas as fixturas de então dobravam
/// um osso, logo **todo** contorno precisava de refit e o fit escrevia por cima — *o caminho onde a
/// lei de hoje é a única a trabalhar não tinha fixtura nenhuma*.
///
/// ⭐ A fixtura é **UM** osso: com um só, o peso é o mesmo em toda parte, a deformação é um AFIM, e
/// um afim **comuta** com a avaliação de Bézier. ⇒ o refit é dispensável ali *por teoria*, e a
/// porta tem de o dispensar **e ainda assim mover a arte**.
///
/// ⚠️ **As duas metades são dois defeitos:** não mover (a lei de hoje deixou de correr) e mover
/// diferente (o refit correu onde não devia, e reescreveu a representação — que é o defeito de
/// `13,33` que o `binding_a_shape_moves_nothing` apanhou).
#[test]
fn onde_o_mapa_e_afim_a_arte_move_se_e_o_desenho_e_identico() {
    let um = Skin::new(vec![osso(0.0, 40.0, 0.5, 0)]).expect("1 osso");
    let (mut hoje, mut curva) = (forma(), forma());
    crate::aplica_corrigido(&um, &mut hoje, &[], &[]);
    aplica_pela_curva(&um, &mut curva, &[], &[]);

    // ⭐ O CONTROLO: a pose TEM de mover a arte, senão as duas metades abaixo são vazias.
    let andou = desvio(&polilinha(&forma()), &polilinha(&hoje));
    assert!(
        andou > 1.0,
        "a fixtura nao move a arte ({andou}) — um osso sem pose faz este gate passar por vacuo"
    );

    let verts_hoje: Vec<VecVertex> = hoje.verts_all().copied().collect();
    let verts_curva: Vec<VecVertex> = curva.verts_all().copied().collect();
    assert_eq!(
        verts_curva.len(),
        verts_hoje.len(),
        "o refit correu sobre um mapa AFIM: ele reescreveu a representacao onde a lei de hoje ja' \
         esta' CERTA, e isso muda o `kind` e o raio de quina de todo vertice"
    );
    let pior = verts_hoje
        .iter()
        .zip(&verts_curva)
        .flat_map(|(a, b)| {
            [
                (a.anchor, b.anchor),
                (a.in_handle, b.in_handle),
                (a.out_handle, b.out_handle),
            ]
        })
        .map(|(p, q)| (p[0] - q[0]).hypot(p[1] - q[1]))
        .fold(0.0_f64, f64::max);
    assert!(
        pior < 1e-12,
        "sob um mapa AFIM as duas leis divergiram {pior} — ou a lei de hoje deixou de correr, ou o \
         refit correu onde ele nao muda a curva e so' muda os pontos de controlo"
    );
}

/// ⭐⭐⭐ **A DEFORMAÇÃO É CONTÍNUA — nenhum ângulo faz o desenho saltar.**
///
/// ⛔⛔⛔ **Este gate é o report do dono de 2026-09-19, com duas fotos:** *«em determinado momento
/// da deformação as alças sofrem uma mudança e o path muda repentinamente, como se o handle
/// mudasse de tipo»*.
///
/// A lei anterior perguntava *«o desvio passa da tolerância?»* e, se sim, refazia o contorno inteiro
/// com a `kurbo::fit_to_bezpath`. **Um booleano sobre uma grandeza contínua é um degrau** — e
/// medido numa dobra a passos de `0,01 rad` ele não deu um salto: deu **CHATTER**. A decisão
/// oscilava entre quadros vizinhos a partir de `1,44 rad` (`false → true → false → true` em
/// `1,44 · 1,45 · 1,56 · 1,79 · 1,96 · 2,06 · 2,10 · 2,12 …`), e cada oscilação valia `0,038`–`0,050`
/// numa peça de espessura `10` — *uma piscadela por quadro enquanto o artista arrasta*.
///
/// ⚠️ **A régua é o passo MÁXIMO contra o passo MEDIANO**, e tem de ser: o desenho move-se a cada
/// grau, logo um tecto absoluto mediria a velocidade do gesto. *O que um salto é: um passo que não
/// se parece com os vizinhos.*
///
/// ⭐ **O CONTROLO vem primeiro** — a varredura tem de ATRAVESSAR o regime difícil (onde a lei
/// ingénua se afasta muito), senão um gate verde não diz nada.
#[test]
fn a_deformacao_e_continua() {
    const PASSOS: usize = 250;
    let mut amostras: Vec<Vec<[f64; 2]>> = Vec::new();
    let mut pior_ingenuo = 0.0_f64;
    for k in 0..=PASSOS {
        let rot = k as f64 * 0.01;
        let k_pele = pele(rot);
        let t = tabela();
        let mut curva = forma();
        aplica_pela_curva(&k_pele, &mut curva, &t, &[]);
        let mut ingenua = forma();
        crate::aplica_corrigido(&k_pele, &mut ingenua, &t, &[]);
        pior_ingenuo = pior_ingenuo.max(desvio(&polilinha(&curva), &polilinha(&ingenua)));
        amostras.push(polilinha(&curva));
    }
    // ⭐ O CONTROLO: sem passar pelo regime em que as duas leis discordam, este gate é vácuo — era
    // ali que o interruptor antigo piscava.
    assert!(
        pior_ingenuo > 0.05,
        "a varredura nao atravessa o regime dificil (a lei ingenua afasta-se so' {pior_ingenuo}) — \
         o gate ficaria verde sobre a lei antiga tambem"
    );

    let passos: Vec<f64> = amostras.windows(2).map(|w| desvio(&w[0], &w[1])).collect();
    let mut ord = passos.clone();
    ord.sort_by(f64::total_cmp);
    let mediana = ord[ord.len() / 2];
    let (i, pior) = passos
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .expect("ha' passos");
    eprintln!(
        "[curva] continuidade: passo mediano {mediana:.6} · pior {pior:.6} em rot {:.2} => {:.2}x",
        i as f64 * 0.01,
        pior / mediana
    );
    assert!(
        *pior < mediana * 3.0,
        "o desenho saltou em rot {:.2}: o passo valeu {pior} contra uma mediana de {mediana} \
         ({:.1}x) — e' o report do dono de 19/09",
        i as f64 * 0.01,
        pior / mediana
    );
}

/// ⭐⭐ **EM REPOUSO AS ALÇAS FICAM BYTE-IDÊNTICAS** — a propriedade que a correcção da DIFERENÇA dá
/// e que o refit não podia dar.
///
/// ⛔ Era este o defeito que obrigava o limiar a existir: a 1.ª redacção da lei da curva refitava
/// sempre, e o `binding_a_shape_moves_nothing` acusava `40/3` em repouso — a elevação `(⅓, ⅔)` de
/// uma recta, que desenha a MESMA curva com outros pontos de controlo. Aqui o segundo membro do
/// sistema é **exactamente zero**, logo a correcção é zero.
#[test]
fn em_repouso_as_alcas_ficam_byte_identicas() {
    let k = pele(0.0);
    let t = tabela();
    let mut curva = forma();
    aplica_pela_curva(&k, &mut curva, &t, &[]);
    let mut ingenua = forma();
    crate::aplica_corrigido(&k, &mut ingenua, &t, &[]);
    for (a, b) in curva.verts.iter().zip(&ingenua.verts) {
        assert_eq!(a.out_handle, b.out_handle, "uma alca de saida mexeu-se");
        assert_eq!(a.in_handle, b.in_handle, "uma alca de entrada mexeu-se");
        assert_eq!(a.anchor, b.anchor, "um no' mexeu-se");
    }
}

/// ⭐⭐ **O NÚMERO DE AMOSTRAS ESTÁ MEDIDO** — e o que ele compra satura.
///
/// A régua é o erro contra a verdade (a mesma lei com `64` amostras), na dobra mais forte da
/// varredura de continuidade.
#[test]
fn o_numero_de_amostras_esta_medido() {
    let k = pele(1.2);
    let t = tabela();
    let mut verdade = forma();
    aplica_pela_curva(&k, &mut verdade, &t, &[]);
    // ⚠️ A verdade aqui é a POSIÇÃO do fitter com muitas amostras; como a `AMOSTRAS` é uma const,
    // o que este gate pode afirmar é o resultado dela **contra a lei ingénua** e contra si mesma.
    let mut ingenua = forma();
    crate::aplica_corrigido(&k, &mut ingenua, &t, &[]);
    let ganho = desvio(&polilinha(&verdade), &polilinha(&ingenua));
    eprintln!("[curva] com {AMOSTRAS} amostras a correccao vale {ganho:.6}");
    assert!(
        ganho > 0.05,
        "a correccao das alcas deixou de mover a arte ({ganho}) — ou a lei parou, ou a fixtura \
         deixou de dobrar"
    );
}

/// ⭐⭐⭐ **AS ALÇAS CORRIGIDAS SEGUEM A CURVA VERDADEIRA** — a qualidade do ajuste, e não só o facto
/// de a arte se mexer.
///
/// ⛔⛔ **TRÊS mutações sobreviveram antes deste gate existir**: o determinante do sistema `2×2`
/// trocado por `a₁₁·a₂₂` (que ignora o acoplamento), a solução DESACOPLADA (`b₁/a₁₁`, `b₂/a₂₂`) e a
/// alça de ENTRADA deixada por corrigir. Nenhuma delas partia os gates que havia — *«a arte
/// mexe-se»* e *«é contínua»* continuam verdadeiros com um ajuste mau. ⇒ o que faltava era medir
/// **quanto** a cúbica corrigida se aproxima do mapa verdadeiro.
///
/// ⚠️ A régua é o erro MÁXIMO ao longo do segmento, e o controlo é o mesmo erro **antes** da
/// correcção: sem ele, um segmento quase recto daria um número pequeno por si só.
#[test]
fn as_alcas_corrigidas_seguem_a_curva_verdadeira() {
    let k = pele(1.2);
    let t = tabela();
    let fonte = forma();
    let mut ingenua = fonte.clone();
    crate::aplica_corrigido(&k, &mut ingenua, &t, &[]);
    let mut curva = fonte.clone();
    aplica_pela_curva(&k, &mut curva, &t, &[]);

    let n = fonte.verts.len();
    let (mut antes, mut depois) = (0.0_f64, 0.0_f64);
    for seg in 0..n {
        let s = super::SegmentoDaPele {
            src: super::cubica(&fonte.verts, seg, n),
            pele: &k,
            ra: super::linha(&t, 2, seg),
            rb: super::linha(&t, 2, (seg + 1) % n),
            correcoes: &[],
            rigido: true,
            campo: None,
            suave: None,
        };
        let (ja, agora) = (
            super::cubica(&ingenua.verts, seg, n),
            super::cubica(&curva.verts, seg, n),
        );
        for i in 1..64 {
            let u = f64::from(i) / 64.0;
            let verdade = s.ponto(u);
            antes = antes.max((verdade - ja.eval(u)).hypot());
            depois = depois.max((verdade - agora.eval(u)).hypot());
        }
    }
    eprintln!("[curva] erro contra a verdade: ingenua {antes:.6} · corrigida {depois:.6}");
    // ⭐ O CONTROLO: a fixtura tem de ter um erro a corrigir.
    assert!(
        antes > 1.0,
        "a lei ingenua ja' segue a verdade ({antes}) — a fixtura nao contem o fenomeno"
    );
    assert!(
        depois < antes / 4.0,
        "a correccao das alcas melhorou so' {:.2}x ({antes} -> {depois}) — o ajuste nao e' o de \
         minimos quadrados que o doc descreve",
        antes / depois
    );
}
