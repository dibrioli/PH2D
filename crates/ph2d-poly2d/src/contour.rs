//! **A FRONTEIRA da tinta** — os anéis de pixel que separam o coberto do vazio.

/// ⭐⭐⭐ **OS ANÉIS DE CONTORNO de uma grelha de cobertura**, ordenados do maior para o menor.
///
/// `alpha` é uma amostra por pixel em ordem de leitura (`y * width + x`); um pixel conta como
/// tinta quando `alpha >= threshold`. Devolve um anel **fechado** por ilha, em coordenadas de
/// pixel (o centro do pixel `(x, y)` é o ponto `(x, y)`).
///
/// # Rastreio de Moore, e o critério que o faz FECHAR
///
/// ⚠️⚠️ **O anel fecha quando DUAS células consecutivas se repetem**, e não quando se volta ao
/// pixel inicial. A diferença morde numa forma com um istmo de um pixel: o rastreio passa pelo
/// pixel inicial **duas** vezes, e parar na primeira deixa metade da ilha de fora.
///
/// ⛔⛔ **E a 1.ª redacção usou a outra forma clássica — *«voltei ao início entrando pela mesma
/// direcção»* — comparando com a entrada ARTIFICIAL do arranque (a casa vazia por cima).** Essa
/// casa não é por onde o rastreio de facto volta a entrar, logo o critério **nunca disparava**:
/// medido, um quadrado de `10×10` deu **3201** pontos em vez de 36 — 89 voltas até bater no tecto
/// de segurança. *Um critério de paragem que compara com um valor que o laço nunca produz é um
/// laço infinito com cara de algoritmo.*
///
/// ⛔ **Só o contorno EXTERIOR de cada ilha.** Um buraco no meio de uma forma não é traçado, e a
/// consequência é honesta: a malha cobre o buraco. Traçá-lo exige o [`crate::triangulate`] saber
/// aceitar anéis interiores, que ele hoje não sabe — as duas metades andam juntas ou nenhuma.
///
/// ⚠️ **Uma ilha de menos de três pixels não devolve anel**: um ponto e um par não têm interior,
/// e um triângulo degenerado a jusante é pior que uma ilha ausente.
#[must_use]
pub fn contour(alpha: &[u8], width: usize, height: usize, threshold: u8) -> Vec<Vec<[f64; 2]>> {
    if width == 0 || height == 0 || alpha.len() < width * height {
        return Vec::new();
    }
    let dentro = |x: isize, y: isize| -> bool {
        if x < 0 || y < 0 || x >= width as isize || y >= height as isize {
            return false;
        }
        #[expect(
            clippy::cast_sign_loss,
            reason = "os dois foram testados contra zero na linha acima"
        )]
        let i = y as usize * width + x as usize;
        alpha[i] >= threshold
    };
    // Já visitado como ARRANQUE de um anel. ⚠️ Não é «já visitado pelo rastreio»: um pixel de
    // fronteira é pisado várias vezes num istmo, e marcá-lo aí partiria o anel ao meio.
    let mut semeado = vec![false; width * height];
    let mut aneis: Vec<Vec<[f64; 2]>> = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let (xi, yi) = (x as isize, y as isize);
            if !dentro(xi, yi) || semeado[y * width + x] {
                continue;
            }
            // ⚠️ **O arranque tem de ser um pixel de FRONTEIRA**, e a varredura em ordem de
            // leitura garante-o: o primeiro pixel coberto de cada ilha tem o vizinho de cima
            // vazio, senão a ilha já teria sido semeada numa linha anterior.
            if dentro(xi, yi - 1) {
                continue;
            }
            let anel = trace(&dentro, xi, yi, 8 * width * height);
            for &[px, py] in &anel {
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "os pontos do anel são centros de pixel dentro da grelha, por construção"
                )]
                let (ix, iy) = (px as usize, py as usize);
                if iy < height && ix < width {
                    semeado[iy * width + ix] = true;
                }
            }
            if anel.len() >= 3 {
                aneis.push(anel);
            }
        }
    }
    // O maior primeiro — quem consome uma silhueta só quer a peça principal, e quem as quer todas
    // percorre a lista. ⚠️ Ordenar pelo MÓDULO da área: a orientação é do rastreio, não do tamanho.
    aneis.sort_by(|a, b| {
        crate::signed_area(b)
            .abs()
            .total_cmp(&crate::signed_area(a).abs())
    });
    aneis
}

/// Os oito vizinhos, em sentido horário a partir de Oeste — a ordem de Moore.
const VIZINHOS: [(isize, isize); 8] = [
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
];

/// O anel de UMA ilha, a partir de um pixel de fronteira dela.
///
/// ⚠️⚠️ **O `backtrack` do passo seguinte é o vizinho que PRECEDE o achado na ordem de Moore, e
/// não *«o último vazio que se viu»*.** A diferença mordeu na 1.ª redacção: quando o PRIMEIRO
/// vizinho testado já é tinta, nenhum vazio foi visto, e o backtrack ficava a apontar para uma
/// casa que não é vizinha do pixel novo. A busca seguinte não o encontrava, recomeçava em Oeste
/// por omissão, e o rastreio **nunca fechava** — três testes acima de 60 s e um anel de milhões de
/// pontos que a simplificação (quadrática) depois tentava percorrer.
///
/// ⭐ O vizinho anterior na ordem de Moore é sempre adjacente ao achado — é isso que faz esta
/// escolha estar **sempre** definida, inclusive no primeiro teste.
fn trace(
    dentro: &impl Fn(isize, isize) -> bool,
    sx: isize,
    sy: isize,
    tecto: usize,
) -> Vec<[f64; 2]> {
    let mut anel: Vec<[f64; 2]> = Vec::new();
    // A casa vazia de onde entrámos — em cima, pela garantia do chamador.
    let mut backtrack = (sx, sy - 1);
    let (mut cx, mut cy) = (sx, sy);
    // ⚠️ Tecto de segurança derivado do TAMANHO da grelha (o rastreio pisa cada pixel no máximo
    // uma vez por vizinho). Sem ele uma grelha patológica pendura o app — o modo de falha caro
    // deste algoritmo, e o que de facto aconteceu na 1.ª redacção.
    let mut guarda = 0usize;
    loop {
        let atual = [
            f64::from(i32::try_from(cx).unwrap_or(0)),
            f64::from(i32::try_from(cy).unwrap_or(0)),
        ];
        // Roda os vizinhos a partir do backtrack, em sentido horário, até achar tinta.
        let inicio = VIZINHOS
            .iter()
            .position(|&(dx, dy)| (cx + dx, cy + dy) == backtrack)
            .unwrap_or(0);
        let mut achou = None;
        for k in 1..=8 {
            let (dx, dy) = VIZINHOS[(inicio + k) % 8];
            let (nx, ny) = (cx + dx, cy + dy);
            if dentro(nx, ny) {
                let (px, py) = VIZINHOS[(inicio + k - 1) % 8];
                achou = Some(((nx, ny), (cx + px, cy + py)));
                break;
            }
        }
        let Some(((nx, ny), entrada)) = achou else {
            // Uma ilha de um pixel só: não há anel.
            return Vec::new();
        };
        let proximo = [
            f64::from(i32::try_from(nx).unwrap_or(0)),
            f64::from(i32::try_from(ny).unwrap_or(0)),
        ];
        // ⭐⭐⭐ **O anel fecha quando DUAS células consecutivas se repetem** — estamos no primeiro
        // ponto e o passo seguinte é o segundo.
        if anel.len() >= 2 && atual == anel[0] && proximo == anel[1] {
            return anel;
        }
        anel.push(atual);
        (cx, cy) = (nx, ny);
        backtrack = entrada;
        guarda += 1;
        if guarda > tecto {
            return anel;
        }
    }
}
