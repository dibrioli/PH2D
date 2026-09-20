//! ⭐⭐⭐ **OS GATES DA CONJUGAÇÃO** — o que torna o gate de PIXEL (que é `#[ignore]`, logo o CI
//! nunca o corre) deixar de ser o único a defender a metade do dispositivo.
//!
//! ⚠️⚠️ **A lei que o shader constrói À MÃO é a rotação conjugada.** Os afins e as juntas chegam
//! conjugados pela CPU, mas `R(θ̄)` não pode: `θ̄` só nasce da mistura dos ângulos, já dentro do
//! *vertex shader*. ⇒ ele multiplica os termos fora da diagonal pelas duas razões do `size`, e
//! **é isso que estes gates prendem** — *uma aritmética escrita em dois sítios só é uma lei se
//! alguém medir que os dois dão o mesmo*.

use super::{SkinAfimGpu, conjuga_para_o_quad, conjuga_ponto_para_o_quad};
use crate::sprite_mesh::{MeshFrame, SpriteMesh};
use crate::sprite_mesh_skin::SpriteMeshSkin;

const ANCHOR: [f32; 2] = [0.25, -0.4];
const SIZE: [f32; 2] = [3.0, 1.5];

fn aplica(m: [f32; 6], p: [f32; 2]) -> [f32; 2] {
    [
        m[0] * p[0] + m[2] * p[1] + m[4],
        m[1] * p[0] + m[3] * p[1] + m[5],
    ]
}

/// ⭐⭐⭐ **CONJUGAR E APLICAR É APLICAR E CONJUGAR** — `A(Q(p)) = Q(M(p))`, que é a definição de
/// `A = Q ∘ M ∘ Q⁻¹` e a única razão de as duas conjugações (a do AFIM e a do PONTO) poderem viver
/// em funções separadas.
///
/// ⛔ **É o gate que apanha um `− anchor` esquecido na do ponto:** a junta é um SÍTIO, e conjugá-la
/// como direcção punha o centro de rotação no sítio errado em toda sprite cuja âncora não fosse a
/// origem — *e só nessas*, que é o que faz o defeito passar despercebido numa fixtura centrada.
#[test]
fn conjugar_e_aplicar_e_aplicar_e_conjugar() {
    // Um afim que NÃO é uma rotação: escala, cisalha e translada.
    let m = [0.8_f32, 0.3, -0.45, 1.1, 0.7, -0.2];
    let a = conjuga_para_o_quad(m, ANCHOR, SIZE);
    let mut pior = 0.0_f32;
    for p in [[0.0_f32, 0.0], [1.0, 0.5], [-2.0, 1.7], [0.25, -0.4]] {
        let esq = aplica(a, conjuga_ponto_para_o_quad(p, ANCHOR, SIZE));
        let dir = conjuga_ponto_para_o_quad(aplica(m, p), ANCHOR, SIZE);
        pior = pior.max((esq[0] - dir[0]).hypot(esq[1] - dir[1]));
    }
    assert!(
        pior < 1e-6,
        "as duas conjugacoes discordam em {pior:.3e} — o afim e o ponto deixaram de viver no mesmo \
         espaco, e o centro de rotacao cai fora da arte"
    );
    // ⛔ O CONTROLO: com a âncora ignorada (o defeito), a mesma medição TEM de acusar.
    let cego = |p: [f32; 2]| [p[0] / SIZE[0], p[1] / SIZE[1]];
    let errado = [[1.0_f32, 0.5], [-2.0, 1.7]]
        .into_iter()
        .map(|p| {
            let e = aplica(a, cego(p));
            let d = cego(aplica(m, p));
            (e[0] - d[0]).hypot(e[1] - d[1])
        })
        .fold(0.0_f32, f32::max);
    assert!(
        errado > 1e-3,
        "o controlo nao reproduz o defeito ({errado:.3e}) — esta fixtura nao distingue as duas"
    );
}

/// ⭐⭐⭐ **A ROTAÇÃO QUE O SHADER CONSTRÓI À MÃO É A MESMA QUE A PORTA CONJUGA.**
///
/// O shader não pode chamar a [`conjuga_para_o_quad`] — ele só sabe `θ̄` depois de misturar os
/// ângulos — e reconstrói `S⁻¹RS` com as duas razões que o [`SkinAfimGpu`] lhe entrega. ⇒ este gate
/// é o que impede as duas aritméticas de divergirem.
///
/// ⚠️ **`sx = sy` tornaria as razões INERTES** e o gate mediria o nada; há asserção a exigir o
/// contrário, pela mesma lei que a fixtura do gate de pixel já carrega.
#[test]
fn a_rotacao_que_o_shader_constroi_e_a_que_a_porta_conjuga() {
    assert!(
        (SIZE[0] - SIZE[1]).abs() > 0.1,
        "um `size` quadrado deixa as duas razoes inertes e este gate mede o nada"
    );
    let razao = [SIZE[0] / SIZE[1], SIZE[1] / SIZE[0]];
    let mut pior = 0.0_f32;
    for rad in [0.0_f32, 0.35, 1.2, -0.8, 3.0] {
        let (co, si) = (rad.cos(), rad.sin());
        // A porta, sobre a rotação PURA (translação nula ⇒ só a parte linear conta).
        let a = conjuga_para_o_quad([co, si, -si, co, 0.0, 0.0], [0.0, 0.0], SIZE);
        for d in [[1.0_f32, 0.0], [0.0, 1.0], [0.7, -1.3]] {
            let porta = aplica(a, d);
            // O que o `posa_pela_pele` faz, linha por linha.
            let shader = [
                si.mul_add(-razao[1] * d[1], co * d[0]),
                si.mul_add(razao[0] * d[0], co * d[1]),
            ];
            pior = pior.max((porta[0] - shader[0]).hypot(porta[1] - shader[1]));
        }
    }
    assert!(
        pior < 1e-6,
        "o shader e a porta constroem rotacoes conjugadas DIFERENTES ({pior:.3e}) — a arte torce \
         por uma lei e o ponteiro aponta pela outra"
    );
}

/// ⭐⭐ **A AGULHA: o shader ainda usa a razão CRUZADA em cada linha.**
///
/// ⚠️ O gate acima reproduz a aritmética, logo **não sangra** se alguém trocar `razao.x` por
/// `razao.y` no `.wgsl` — ele é uma CÓPIA. Esta metade nomeia o endereço de fiação e falha ALTO,
/// que é a sorte desta família. ⛔ *Um gate que copia o sujeito precisa de um segundo que o aponte.*
#[test]
fn o_shader_cruza_as_razoes_e_a_struct_tem_o_tamanho_do_registo() {
    let wgsl = include_str!("shaders/sprite.wgsl");
    for agulha in [
        "co * d.x - si * a0.razao.y * d.y + base.x",
        "si * a0.razao.x * d.x + co * d.y + base.y",
    ] {
        assert!(
            wgsl.contains(agulha),
            "o shader deixou de conter `{agulha}` — se as razoes foram trocadas ou a linha mudou \
             de forma, o gate irmao nao o ve porque ele e' uma copia da aritmetica"
        );
    }
    // ⛔ O registo tem de ter o passo que o `array<SkinAfim>` do WGSL assume (múltiplo de 16).
    assert_eq!(
        size_of::<SkinAfimGpu>(),
        64,
        "o registo do osso mudou de tamanho — o WGSL le'-lo-ia desalinhado, e isso nao estoura: \
         desenha a arte com os numeros do vizinho"
    );
    assert_eq!(size_of::<SkinAfimGpu>() % 16, 0);
}

/// ⭐⭐⭐ **O QUE A COSTURA ESCREVE EM CADA REGISTO** — o gate que fecha a fiação do payload sem
/// pedir um adaptador.
///
/// ⛔⛔ **Ele existe porque o gate de PIXEL é `#[ignore]`, logo o CI nunca o corre.** Sem esta
/// metade, trocar as duas razões ou perder a base da tabela de juntas passava por todo portão que
/// corre sem placa — *e o sintoma seria a arte a torcer para o lado errado, num sítio onde nada
/// estoura*.
///
/// ⚠️ **A DUAS malhas de propósito:** com uma só, a `base` dos afins e a das juntas são ambas `0` e
/// um `base = 0` cravado passaria. *Uma concatenação com um elemento não testa a concatenação.*
#[test]
fn a_costura_escreve_a_base_o_indice_local_e_as_duas_razoes() {
    let (anchor, size) = ([0.25_f32, -0.4], [3.0_f32, 1.5]);
    // Duas malhas, com contagens de osso DIFERENTES — senão a base da segunda seria adivinhável.
    let malhas = [quadrado(2, anchor, size), quadrado(3, anchor, size)];
    let mut f = MeshFrame::default();
    for m in &malhas {
        assert!(f.push(m, anchor, size) > 0, "a malha nao entrou no quadro");
    }
    assert_eq!(f.afins.len(), 2 + 3, "a concatenacao dos afins");
    assert_eq!(f.juntas.len(), 2 * 2 + 3 * 3, "a concatenacao das juntas");

    let esperado = [(0_usize, 2_u32, 0_u32), (2, 3, 4)];
    for (malha, (base, n, base_j)) in malhas.iter().zip(esperado) {
        let ossos = malha.skin.as_ref().expect("pele").afins.len();
        for k in 0..ossos {
            let r = f.afins[base + k];
            assert_eq!(
                r.info,
                [u32::try_from(k).expect("k cabe"), n, base_j, 0],
                "o registo do osso {k} da malha de {n} ossos perdeu o indice LOCAL, a contagem ou \
                 a base da tabela de juntas"
            );
            // ⚠️ `sx/sy` na PRIMEIRA e `sy/sx` na segunda — trocá-las torce a arte ao contrário.
            assert!(
                (r.razao[0] - size[0] / size[1]).abs() < 1e-6
                    && (r.razao[1] - size[1] / size[0]).abs() < 1e-6,
                "as razoes do osso {k} saem {:?}, e o shader le' a primeira na linha do `y`",
                r.razao
            );
        }
    }
}

/// Um quadrado de dois triângulos com uma pele de `n` ossos — a fixtura do gate acima.
fn quadrado(n: usize, anchor: [f32; 2], size: [f32; 2]) -> SpriteMesh {
    let local: Vec<[f32; 2]> = [[-0.5_f32, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]]
        .into_iter()
        .map(|c| [anchor[0] + c[0] * size[0], anchor[1] + c[1] * size[1]])
        .collect();
    let uv = local
        .iter()
        .map(|p| SpriteMesh::uv_at(*p, anchor, size).expect("size nao nulo"))
        .collect();
    let mut pesos = [0.0_f32; crate::sprite_mesh_skin::OSSOS_POR_VERTICE];
    pesos[0] = 1.0;
    SpriteMesh {
        local,
        uv,
        tris: vec![[0, 1, 2], [0, 2, 3]],
        skin: Some(SpriteMeshSkin {
            pesos: vec![pesos; 4],
            ossos: vec![[0; crate::sprite_mesh_skin::OSSOS_POR_VERTICE]; 4],
            afins: vec![[1.0, 0.0, 0.0, 1.0, 0.0, 0.0]; n],
            juntas: vec![[0.0, 0.0]; n * n],
            angulos: vec![[1.0, 0.0]; n],
        }),
    }
}
