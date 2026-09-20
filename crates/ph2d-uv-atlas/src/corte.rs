//! ⭐⭐⭐ **O CORTE — a ilha parte-se até deixar de se pintar duas vezes.**
//!
//! É o passo que **todo desenrolador tem** e que a W1 entregou em falta. A régua da
//! [`crate::sobreposicao`] atribuiu o vermelho antes de se escrever uma linha disto, e a
//! tabela decidiu o desenho:
//!
//! | peça (CRUA, o caminho do produto) | área cruzada | `dobra` | `mesma-carta` | `mesma-ilha` |
//! |---|---|---|---|---|
//! | `_base_sculpt` | `9,56 %` | `0,10 %` | `0,15 %` | **`9,31 %`** |
//! | esfera `24` | `0,05 %` | `0,04 %` | `0,01 %` | `0,00 %` |
//!
//! ⇒ **o que se pinta duas vezes é o ASSENTAMENTO a pôr duas cartas da mesma ilha uma em
//! cima da outra**, e não o solver a dobrar. *Sem a atribuição eu teria começado por
//! afinar o G3, que responde por um décimo de ponto.*
//!
//! # A lei, e porque ela TERMINA
//!
//! Uma peça cresce por vizinhança a partir de uma face semente e **só aceita uma face
//! que não cruze nenhuma das que já lá estão** — o teste é o da
//! [`crate::sobreposicao::area_de_interseccao`], exacto. ⭐ Daí saem as duas propriedades
//! que interessam: cada peça é injectiva **por construção e não por promessa**, e o laço
//! acaba porque toda passagem coloca pelo menos a semente.
//!
//! # ⛔⛔ A unidade é a FACE e nunca o triângulo
//!
//! O atlas guarda um `(u, v)` **por canto**, e os dois triângulos de um quad partilham
//! dois cantos: pô-los em peças diferentes pediria dois `(u, v)` no mesmo canto, que é
//! inexprimível. ⇒ *o que este corte não separa é uma face que se dobra sobre si mesma*,
//! e essa é a coluna `dobra` da tabela — outra cura, a montante.

use crate::{sobreposicao, topo};
use ph2d_mesh::Mesh;
use std::collections::{BTreeMap, VecDeque};

/// O que o corte fez.
#[derive(Debug, Clone, Default)]
pub struct Corte {
    /// Por face da malha, a peça. `u32::MAX` = a face não tem `(u, v)`.
    pub peca_da_face: Vec<u32>,
    /// Quantas peças saíram.
    pub pecas: usize,
    /// ⚠️ Quantas delas têm **uma face só** — o custo do corte, porque cada peça paga um
    /// vão inteiro no empacotador.
    pub pecas_de_uma_face: usize,
    /// Quantas vezes uma face foi RECUSADA por cruzar a peça que crescia. ⭐ É o trabalho
    /// que o corte de facto fez: `0` quer dizer que nenhuma ilha se dobrava.
    pub recusas: usize,
    /// Faces sem `(u, v)`.
    pub faces_sem_uv: usize,
    /// ⛔ **Faces que a vizinhança do atlas deixou SOZINHAS.** Uma delas é peça de uma
    /// face por construção e não por geometria — *se este número explicar as peças de uma
    /// face, o defeito é da adjacência e não do corte*.
    pub faces_sem_vizinho: usize,
    /// ⭐ Quantos pares de peças a fusão juntou — *o corte que o crescimento fez a mais*.
    pub fusoes: usize,
    /// A mediana e o máximo do tamanho de uma peça, em faces.
    pub tamanho_p50: usize,
    /// Ver [`Self::tamanho_p50`].
    pub tamanho_max: usize,
}

/// O lado de uma célula da grelha de busca, a partir da área média dos triângulos.
fn lado_da_celula(areas: &[f64]) -> f64 {
    let vivos: Vec<f64> = areas.iter().copied().filter(|&a| a > 0.0).collect();
    if vivos.is_empty() {
        return 1.0;
    }
    let media = vivos.iter().sum::<f64>() / vivos.len() as f64;
    (media.sqrt() * 2.0).max(1.0e-9)
}

/// A obra em curso: as peças, a grelha de busca e o que já foi colocado.
///
/// ⚠️ É uma struct e não um punhado de closures porque o laço é **ronda-a-ronda** e cada
/// ronda lê e escreve os mesmos cinco vectores; com fechos, o verificador de empréstimos
/// obrigaria a copiá-los.
struct Obra<'a> {
    tris: &'a [[u32; 3]],
    tris_da_face: &'a [(u32, u32)],
    plano: &'a [[f32; 2]],
    areas: &'a [f64],
    celula: f64,
    /// `(peça, triângulo)` por célula — ⭐ **uma grelha só para TODAS as peças**: com uma
    /// por peça, abrir uma peça nova custaria uma tabela nova, e elas são centenas.
    grelha: BTreeMap<(i32, i32), Vec<(u32, u32)>>,
    peca_da_face: Vec<u32>,
    contagem: Vec<usize>,
    filas: Vec<VecDeque<u32>>,
    /// ⭐ Conjunto disjunto sobre as PEÇAS. Durante o crescimento é a identidade; a
    /// fusão usa-o para juntar duas peças **sem reescrever a grelha** — que é o que
    /// torna a fusão barata o bastante para correr em rondas.
    pai: Vec<u32>,
}

impl Obra<'_> {
    /// A raiz de uma peça. ⚠️ **Sem compressão de caminho, de propósito:** com ela o
    /// [`Self::cruza`] precisaria de `&mut self`, e ele é chamado dentro de um laço que
    /// já tem a obra emprestada. A profundidade é a de uma cadeia de fusões.
    fn raiz(&self, mut x: u32) -> u32 {
        while self.pai[x as usize] != x {
            x = self.pai[x as usize];
        }
        x
    }

    /// A face `f` cruza alguma coisa que já está na peça `p` (uma RAIZ)?
    fn cruza(&self, f: usize, p: u32) -> bool {
        let (t0, t1) = self.tris_da_face[f];
        for t in t0..t1 {
            let z = cantos(self.plano, self.tris[t as usize]);
            let ((x0, y0), (x1, y1)) = celulas(z, self.celula);
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let Some(lista) = self.grelha.get(&(x, y)) else {
                        continue;
                    };
                    for &(q, o) in lista {
                        if self.raiz(q) != p {
                            continue;
                        }
                        let piso = self.areas[t as usize].min(self.areas[o as usize])
                            * sobreposicao::RUIDO_RELATIVO;
                        let w = cantos(self.plano, self.tris[o as usize]);
                        if sobreposicao::area_de_interseccao(z, w) > piso {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Põe a face `f` na peça `p` e mete-a na fila dela.
    fn aceita(&mut self, f: usize, p: u32) {
        self.peca_da_face[f] = p;
        self.contagem[p as usize] += 1;
        let (t0, t1) = self.tris_da_face[f];
        for t in t0..t1 {
            let z = cantos(self.plano, self.tris[t as usize]);
            let ((x0, y0), (x1, y1)) = celulas(z, self.celula);
            for y in y0..=y1 {
                for x in x0..=x1 {
                    self.grelha.entry((x, y)).or_default().push((p, t));
                }
            }
        }
        self.filas[p as usize].push_back(u32::try_from(f).unwrap_or(0));
    }

    /// Abre uma peça nova semeada em `f`.
    fn abre(&mut self, f: usize) -> u32 {
        let p = u32::try_from(self.filas.len()).unwrap_or(u32::MAX);
        self.filas.push(VecDeque::new());
        self.contagem.push(0);
        self.pai.push(p);
        self.aceita(f, p);
        p
    }
}

/// ⭐⭐⭐ **Corta as ilhas em peças injectivas.**
///
/// `plano` é o `(u, v)` por canto **antes de empacotar** (no plano da ilha), e
/// `ilha_da_face` diz de que ilha cada face veio — ⛔ uma peça nunca atravessa ilhas, e
/// isso é imposto e não deduzido: duas ilhas assentam cada uma na sua origem e podem
/// concordar por acaso.
///
/// # ⛔⛔ Porque o laço é RONDA-A-RONDA, e a medição que o obrigou
///
/// A 1.ª redacção crescia **uma peça até ao fim** e só depois semeava a seguinte. Sobre a
/// escultura do dono isso deu `13` ilhas ⇒ **`485` peças, `361` delas de uma face só** —
/// e a causa não é a geometria: uma face recusada ficava para trás enquanto a peça que a
/// recusou **lhe comia todos os vizinhos**, e quando ela enfim era semeada já não tinha
/// para onde crescer. ⚠️ *Uma peça que cresce até ao fim antes de a seguinte nascer não
/// está a repartir a ilha — está a ficar com ela.*
///
/// ⇒ cada peça activa avança **uma face por ronda**, e uma face recusada é oferecida
/// primeiro a uma peça VIZINHA e só depois vira semente de uma peça nova, ainda dentro da
/// mesma corrida — logo ela compete pelos próprios vizinhos em vez de os perder.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn corta(mesh: &Mesh, plano: &[[f32; 2]], ilha_da_face: &[u32], tem_uv: &[bool]) -> Corte {
    let nfaces = mesh.faces().len();
    let tris = topo::triangulos(mesh);
    let face_do_tri = topo::faces_dos_triangulos(mesh);
    // Por face, a fatia de triângulos dela (são contíguos por construção do leque).
    let mut tris_da_face: Vec<(u32, u32)> = vec![(0, 0); nfaces];
    for (t, &f) in face_do_tri.iter().enumerate() {
        let t = u32::try_from(t).unwrap_or(0);
        let e = &mut tris_da_face[f as usize];
        if e.1 == 0 {
            *e = (t, t + 1);
        } else {
            e.1 = t + 1;
        }
    }

    let areas: Vec<f64> = tris.iter().map(|&t| tri_area(plano, t)).collect();
    let celula = lado_da_celula(&areas);

    // Vizinhança de FACES: aresta da peça em comum **e** `(u, v)` a concordar.
    let mut vizinhos: Vec<Vec<u32>> = vec![Vec::new(); nfaces];
    for (ta, tb, elo) in topo::elos(mesh, plano) {
        if !elo.no_atlas {
            continue;
        }
        let (fa, fb) = (face_do_tri[ta as usize], face_do_tri[tb as usize]);
        if fa == fb || ilha_da_face.get(fa as usize) != ilha_da_face.get(fb as usize) {
            continue;
        }
        vizinhos[fa as usize].push(fb);
        vizinhos[fb as usize].push(fa);
    }

    let mut obra = Obra {
        tris: &tris,
        tris_da_face: &tris_da_face,
        plano,
        areas: &areas,
        celula,
        grelha: BTreeMap::new(),
        peca_da_face: vec![u32::MAX; nfaces],
        contagem: Vec::new(),
        filas: Vec::new(),
        pai: Vec::new(),
    };
    let mut recusas = 0usize;
    let mut carimbo = vec![u32::MAX; nfaces];

    // ⚠️ A ordem de semeadura é a das FACES, e por isso é estável entre corridas: uma
    // travessia de tabela de dispersão daria outra partição a cada arranque.
    for semente in 0..nfaces {
        if obra.peca_da_face[semente] != u32::MAX || !tem_uv[semente] {
            continue;
        }
        obra.abre(semente);
        loop {
            let mut andou = false;
            let mut p = 0usize;
            while p < obra.filas.len() {
                let Some(g) = obra.filas[p].pop_front() else {
                    p += 1;
                    continue;
                };
                andou = true;
                let pu = u32::try_from(p).unwrap_or(u32::MAX);
                for &hv in &vizinhos[g as usize] {
                    let h = hv as usize;
                    if obra.peca_da_face[h] != u32::MAX || carimbo[h] == pu || !tem_uv[h] {
                        continue;
                    }
                    carimbo[h] = pu;
                    if !obra.cruza(h, pu) {
                        obra.aceita(h, pu);
                        continue;
                    }
                    // ⛔⛔ **Uma face recusada abre peça NOVA, e não se oferece antes a
                    // uma peça vizinha.** Eu escrevi essa oferta, e a prova de mutação
                    // disse que ela não é lei nenhuma: apagá-la deixava os 25 gates
                    // verdes. Medida na escultura do dono, a diferença é **`226` contra
                    // `227` peças** (F1: `93` contra `92`) — *a fusão absorve-a inteira*.
                    // ⇒ *uma linha que a mutação não consegue matar é um comentário com
                    // sintaxe de código*, e ela foi APAGADA com o número ao lado.
                    recusas += 1;
                    obra.abre(h);
                }
                p += 1;
            }
            if !andou {
                break;
            }
        }
    }

    // ── ⭐⭐⭐ A FUSÃO. O crescimento ronda-a-ronda reparte a ilha entre muitas frentes,
    // e **duas frentes que se encontram sem se cruzarem ficaram separadas por nada**: na
    // escultura do dono a maior peça caiu de `21 801` para `1 457` faces, o que é bom
    // para o empacotador e é um corte que a geometria não pediu. ⇒ duas peças que se
    // encostam e cujo conjunto continua injectivo passam a ser UMA.
    //
    // ⚠️ A prova de que isto termina é a monotonia: fundir só aumenta uma peça, logo um
    // par que falhou não volta a passar, e cada ronda que muda alguma coisa reduz o
    // número de peças em pelo menos uma.
    let npecas = obra.filas.len();
    let mut faces_da: Vec<Vec<u32>> = vec![Vec::new(); npecas];
    for (f, &q) in obra.peca_da_face.iter().enumerate() {
        if q != u32::MAX {
            faces_da[q as usize].push(u32::try_from(f).unwrap_or(0));
        }
    }
    let mut pares: Vec<(u32, u32)> = Vec::new();
    for (f, lista) in vizinhos.iter().enumerate() {
        let a = obra.peca_da_face[f];
        if a == u32::MAX {
            continue;
        }
        for &g in lista {
            let b = obra.peca_da_face[g as usize];
            if b != u32::MAX && a != b {
                pares.push((a.min(b), a.max(b)));
            }
        }
    }
    pares.sort_unstable();
    pares.dedup();
    // ⚠️ Os pares mais PEQUENOS primeiro: é o confete que precisa de casa, e um par
    // grande fundido cedo fecha a porta a três pequenos.
    pares.sort_by_key(|&(a, b)| faces_da[a as usize].len().min(faces_da[b as usize].len()));
    let mut fusoes = 0usize;
    for _ in 0..8 {
        let mut mexeu = false;
        for &(x, y) in &pares {
            let (a, b) = (obra.raiz(x), obra.raiz(y));
            if a == b {
                continue;
            }
            let (pequeno, grande) = if faces_da[a as usize].len() <= faces_da[b as usize].len() {
                (a, b)
            } else {
                (b, a)
            };
            if faces_da[pequeno as usize]
                .iter()
                .any(|&f| obra.cruza(f as usize, grande))
            {
                continue;
            }
            obra.pai[pequeno as usize] = grande;
            let tomadas = std::mem::take(&mut faces_da[pequeno as usize]);
            faces_da[grande as usize].extend(tomadas);
            fusoes += 1;
            mexeu = true;
        }
        if !mexeu {
            break;
        }
    }

    // ── Renumerar as raízes para `0..k`, na ordem em que aparecem.
    let mut rotulo = vec![u32::MAX; npecas];
    let mut contagem: Vec<usize> = Vec::new();
    let mut peca_da_face = obra.peca_da_face.clone();
    for q in &mut peca_da_face {
        if *q == u32::MAX {
            continue;
        }
        let r = obra.raiz(*q) as usize;
        if rotulo[r] == u32::MAX {
            rotulo[r] = u32::try_from(contagem.len()).unwrap_or(u32::MAX);
            contagem.push(0);
        }
        *q = rotulo[r];
        contagem[rotulo[r] as usize] += 1;
    }

    let mut c = Corte {
        pecas: contagem.len(),
        peca_da_face,
        recusas,
        fusoes,
        ..Corte::default()
    };
    c.faces_sem_uv = (0..nfaces).filter(|&f| !tem_uv[f]).count();
    c.pecas_de_uma_face = contagem.iter().filter(|&&n| n == 1).count();
    c.faces_sem_vizinho = (0..nfaces)
        .filter(|&f| tem_uv[f] && vizinhos[f].is_empty())
        .count();
    let mut ord = contagem;
    ord.sort_unstable();
    c.tamanho_p50 = ord.get(ord.len() / 2).copied().unwrap_or(0);
    c.tamanho_max = ord.last().copied().unwrap_or(0);
    c
}

fn tri_area(plano: &[[f32; 2]], t: [u32; 3]) -> f64 {
    let (Some(&a), Some(&b), Some(&cc)) = (
        plano.get(t[0] as usize),
        plano.get(t[1] as usize),
        plano.get(t[2] as usize),
    ) else {
        return 0.0;
    };
    let (ux, uy) = (f64::from(b[0] - a[0]), f64::from(b[1] - a[1]));
    let (vx, vy) = (f64::from(cc[0] - a[0]), f64::from(cc[1] - a[1]));
    (ux.mul_add(vy, -(uy * vx)) * 0.5).abs()
}

fn cantos(plano: &[[f32; 2]], t: [u32; 3]) -> [[f32; 2]; 3] {
    [
        *plano.get(t[0] as usize).unwrap_or(&[0.0, 0.0]),
        *plano.get(t[1] as usize).unwrap_or(&[0.0, 0.0]),
        *plano.get(t[2] as usize).unwrap_or(&[0.0, 0.0]),
    ]
}

#[allow(clippy::cast_possible_truncation)]
fn celulas(z: [[f32; 2]; 3], celula: f64) -> ((i32, i32), (i32, i32)) {
    let lo = |v: f32| (f64::from(v) / celula).floor() as i32;
    let x0 = lo(z[0][0].min(z[1][0]).min(z[2][0]));
    let x1 = lo(z[0][0].max(z[1][0]).max(z[2][0]));
    let y0 = lo(z[0][1].min(z[1][1]).min(z[2][1]));
    let y1 = lo(z[0][1].max(z[1][1]).max(z[2][1]));
    ((x0, y0), (x1, y1))
}
