// ORÁCULO: o rolamento no Box2D 3.1.1 (MIT, Erin Catto) sobre as NOSSAS entradas — doc 121 §9.24.
//
// O Box2D resolve a resistência ao rolamento DENTRO do solver (o impulso angular limitado em cada
// iteração). Corre-se, nunca se lê: só o cabeçalho público para o chamar.
//
//   rampa  — a bola da `=115` (raio 0,2 e 0,11) numa rampa de 12°, g = 4, 2 s: a distância contra a
//            de `rr = 0`, para cada `rr`; e o limiar em que ela fica presa (bissecção). Diz a UNIDADE
//            do `rollingResistance` (com ou sem o raio) e a lei perto do limiar.
//   pilha  — os discos da `=114` a partir das posições iniciais que a nossa cena exportou
//            (`pilha_XX.txt`), a taça de 95 lados, g = 4, dt 1/60, 4 sub-passos, 490 tiques: as
//            MESMAS réguas da sonda `rodada` (queda 120..180 mediana · pior; depois do 240 o pior
//            rodopio e o pior deslize por janela).
//
// uso: oraculo rampa | oraculo pilha <pasta-com-pilha_XX.txt> <escala> <mu_r>...
//      (rr do Box2D = escala · mu_r · [raio, se a rampa disse que a unidade leva o raio])
#include <box2d/box2d.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define G 4.0f
#define DT (1.0f / 60.0f)
#define SUB 4

static b2WorldId mundo(void) {
	b2WorldDef wd = b2DefaultWorldDef();
	wd.gravity = (b2Vec2){0.0f, -G};
	wd.enableSleep = false;
	return b2CreateWorld(&wd);
}

static b2BodyId disco(b2WorldId w, b2Vec2 p, float r, float atrito, float rr) {
	b2BodyDef bd = b2DefaultBodyDef();
	bd.type = b2_dynamicBody;
	bd.position = p;
	bd.enableSleep = false;
	b2BodyId b = b2CreateBody(w, &bd);
	b2ShapeDef sd = b2DefaultShapeDef();
	sd.density = 1.0f;
	sd.material.friction = atrito;
	sd.material.restitution = 0.0f;
	sd.material.rollingResistance = rr;
	b2Circle c = {{0.0f, 0.0f}, r};
	b2CreateCircleShape(b, &sd, &c);
	return b;
}

// A distância que a bola de raio `r` desce numa rampa de 12° em 2 s com `rr`.
static float rampa(float r, float rr) {
	b2WorldId w = mundo();
	float a = 12.0f * (float)M_PI / 180.0f;
	b2Vec2 t = {cosf(a), -sinf(a)}, n = {sinf(a), cosf(a)};
	b2BodyDef bd = b2DefaultBodyDef();
	b2BodyId chao = b2CreateBody(w, &bd);
	b2ShapeDef sd = b2DefaultShapeDef();
	sd.material.friction = 1.0f;
	b2Segment s = {{-20.0f * t.x, -20.0f * t.y}, {20.0f * t.x, 20.0f * t.y}};
	b2CreateSegmentShape(chao, &sd, &s);
	b2Vec2 p0 = {n.x * (r + 0.02f), n.y * (r + 0.02f)};
	b2BodyId b = disco(w, p0, r, 1.0f, rr);
	for (int k = 0; k < 120; k++) b2World_Step(w, DT, SUB);
	b2Vec2 p = b2Body_GetPosition(b);
	b2DestroyWorld(w);
	return hypotf(p.x - p0.x, p.y - p0.y);
}

static int cmp(const void* a, const void* b) {
	float x = *(const float*)a, y = *(const float*)b;
	return (x > y) - (x < y);
}

#define TIQUES 490
#define MAXP 64

static int pilha(const char* pasta, float escala, float mu_r) {
	float queda[32], rod = 0.0f, des = 0.0f;
	int nq = 0;
	for (int k = 0; k < 32; k++) {
		char nome[1024];
		snprintf(nome, sizeof nome, "%s/pilha_%02d.txt", pasta, k);
		FILE* f = fopen(nome, "r");
		if (!f) break;
		float cx, cy, R, r;
		if (fscanf(f, "# taca %f %f %f raio_disco %f", &cx, &cy, &R, &r) != 4) return 2;
		b2Vec2 p0[MAXP];
		int n = 0;
		while (n < MAXP && fscanf(f, "%f %f", &p0[n].x, &p0[n].y) == 2) n++;
		fclose(f);
		b2WorldId w = mundo();
		b2BodyDef bd = b2DefaultBodyDef();
		b2BodyId taca = b2CreateBody(w, &bd);
		b2ShapeDef sd = b2DefaultShapeDef();
		sd.material.friction = 0.6f;
		sd.material.restitution = 0.05f;
		// Os lados da nossa taça (`fixo::lados`: flecha 1e-3).
		int lados = (int)ceilf((float)M_PI / sqrtf(2.0f * 1e-3f / R));
		for (int i = 0; i < lados; i++) {
			float a0 = 2.0f * (float)M_PI * i / lados, a1 = 2.0f * (float)M_PI * (i + 1) / lados;
			b2Segment s = {{cx + R * cosf(a0), cy + R * sinf(a0)}, {cx + R * cosf(a1), cy + R * sinf(a1)}};
			b2CreateSegmentShape(taca, &sd, &s);
		}
		b2BodyId b[MAXP];
		for (int i = 0; i < n; i++) b[i] = disco(w, p0[i], r, 0.5f, escala * mu_r);
		static float rot[TIQUES + 1][MAXP], px[TIQUES + 1][MAXP], py[TIQUES + 1][MAXP];
		float ang[MAXP];
		for (int i = 0; i < n; i++) {
			ang[i] = 0.0f;
			rot[0][i] = 0.0f;
			px[0][i] = p0[i].x;
			py[0][i] = p0[i].y;
		}
		b2Rot antes[MAXP];
		for (int i = 0; i < n; i++) antes[i] = b2Body_GetRotation(b[i]);
		for (int k = 1; k <= TIQUES; k++) {
			b2World_Step(w, DT, SUB);
			for (int i = 0; i < n; i++) {
				b2Rot q = b2Body_GetRotation(b[i]);
				ang[i] += b2RelativeAngle(antes[i], q) * 180.0f / (float)M_PI;
				antes[i] = q;
				b2Vec2 p = b2Body_GetPosition(b[i]);
				rot[k][i] = ang[i];
				px[k][i] = p.x;
				py[k][i] = p.y;
			}
		}
		b2DestroyWorld(w);
		float qd = 0.0f;
		for (int i = 0; i < n; i++) qd = fmaxf(qd, fabsf(rot[179][i] - rot[120][i]));
		queda[nq++] = qd;
		for (int de = 240; de <= 420; de += 60)
			for (int i = 0; i < n; i++) {
				rod = fmaxf(rod, fabsf(rot[de + 60][i] - rot[de][i]));
				des = fmaxf(des, hypotf(px[de + 60][i] - px[de][i], py[de + 60][i] - py[de][i]));
			}
	}
	if (nq == 0) return 2;
	qsort(queda, nq, sizeof(float), cmp);
	printf("BOX2D discos %g (rr %g) | %.1f · %.1f | %.1f · %.3f | %d realizacoes\n", mu_r, escala * mu_r,
	       queda[nq / 2], queda[nq - 1], rod, des, nq);
	return 0;
}

int main(int argc, char** argv) {
	if (argc >= 2 && strcmp(argv[1], "rampa") == 0) {
		float tg = tanf(12.0f * (float)M_PI / 180.0f);
		float raios[2] = {0.2f, 0.11f};
		for (int j = 0; j < 2; j++) {
			float r = raios[j], solta = rampa(r, 0.0f);
			// O limiar: o menor rr em que a bola fica (desce menos de 2 % da solta).
			float lo = 0.0f, hi = 1.0f;
			for (int it = 0; it < 30; it++) {
				float m = 0.5f * (lo + hi);
				if (rampa(r, m) < 0.02f * solta) hi = m; else lo = m;
			}
			printf("RAMPA raio %g: solta %.4f · limiar rr %.4f (tg 12 = %.4f; tg·raio = %.4f)\n", r, solta, hi, tg,
			       tg * r);
			float mus[6] = {0.05f, 0.1f, 0.15f, 0.2f, 0.21f, 0.5f};
			printf("RAMPA raio %g, rr = mu·(limiar/tg):", r);
			for (int i = 0; i < 6; i++) {
				float rr = mus[i] * hi / tg;
				printf(" · %g: %.3f (%.3f)", mus[i], rampa(r, rr) / solta, fmaxf(0.0f, (tg - mus[i]) / tg));
			}
			printf("\n");
		}
		return 0;
	}
	if (argc >= 4 && strcmp(argv[1], "pilha") == 0) {
		float escala = strtof(argv[3], NULL);
		for (int i = 4; i < argc; i++)
			if (pilha(argv[2], escala, strtof(argv[i], NULL))) return 2;
		return 0;
	}
	fprintf(stderr, "uso: oraculo rampa | oraculo pilha <pasta> <escala> <mu_r>...\n");
	return 2;
}
