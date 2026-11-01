import json, math, os, sys
FX=os.path.join(os.path.dirname(os.path.abspath(__file__)),'../../../crates/ph2d-pixel-core/tests/fixtures/oracle_trial_rotation.json')
D=json.load(open(FX))
RAD=math.pi/180
def compose(rot,sx,sy,skew):
    r=rot*RAD; cs,sn,tk=math.cos(r),math.sin(r),math.tan(skew*RAD)
    return [sx*cs, sx*sn, sy*(-tk*cs-sn), sy*(-tk*sn+cs)]
def around(lin,box):
    a,b,c,d=lin; bx,by,bw,bh=box; u,v=bw/2,bh/2; cx,cy=bx+u,by+v
    return [a,b,c,d,cx-(a*u+c*v), cy-(b*u+d*v)]
def apply(t,u,v): return (t[0]*u+t[2]*v+t[4], t[1]*u+t[3]*v+t[5])
def trial_rot(t,w,h):
    r=math.hypot(t[0],t[1]); c,s=t[0]/r,t[1]/r
    def up(v):
        k=v*65536.0; return (math.ceil(k) if k>0 else math.floor(k))/65536.0
    c2,s2=up(c),up(s)
    if c2==c and s2==s: return t
    q00,q01=c2*c+s2*s, c2*s-s2*c; q10,q11=s2*c-c2*s, s2*s+c2*c
    a=q00*t[0]+q01*t[1]; b=q10*t[0]+q11*t[1]; cc=q00*t[2]+q01*t[3]; d=q10*t[2]+q11*t[3]
    p=apply(t,w/2,h/2)
    return [a,b,cc,d,p[0]-(a*w/2+cc*h/2), p[1]-(b*w/2+d*h/2)]
def run(case, tie):
    blk=case['block']; bx,by,bw,bh=D['inputs'][blk]['box']
    f=case['fields']; ang=f.get('angle',0); sk=f.get('skew',0)
    t=around(compose(-ang,1,1,-sk),(bx,by,bw,bh))
    t=trial_rot(t,bw,bh)
    q=[apply(t,u,v) for u,v in [(0,0),(bw,0),(bw,bh),(0,bh)]]
    q=[(math.floor(x),math.floor(y)) for x,y in q]
    x0=min(p[0] for p in q); y0=min(p[1] for p in q); x1=max(max(p[0] for p in q),x0+1); y1=max(max(p[1] for p in q),y0+1)
    p0,p1,p2,p3=q
    assert p1[0]-p0[0]==p2[0]-p3[0] and p1[1]-p0[1]==p2[1]-p3[1], 'homog'
    a,b,c,d=p1[0]-p0[0],p1[1]-p0[1],p3[0]-p0[0],p3[1]-p0[1]
    det=a*d-b*c; den=2*det; w,h=bw,bh
    want=case['ids']; out=[]
    for y in range(y0,y1):
        ry=2*y+1-2*p0[1]
        for x in range(x0,x1):
            rx=2*x+1-2*p0[0]
            nu=w*(d*rx-c*ry); nv=h*(a*ry-b*rx)
            info=dict(x=x,y=y,nu=nu,nv=nv,den=den,a=a,b=b,c=c,d=d,w=w,h=h,rx=rx,ry=ry,q=q)
            u,v=tie(info)
            g=(v*w+u+1) if (0<=u<w and 0<=v<h) else 0
            wt=want[y][x] if 0<=y<len(want) and 0<=x<len(want[0]) else 0
            info.update(got=g,want=wt)
            out.append(info)
    return out
def left(i):
    def cell(num,dx):
        f=num//i['den']
        return f-1 if num%i['den']==0 and dx>0 else f
    return cell(i['nu'],i['d']), cell(i['nv'],-i['b'])
if __name__=='__main__':
    tot=0
    for k,c in D['cases'].items():
        if c['algorithm']!='fast' or k.endswith('a180'): continue
        o=run(c,left); n=sum(1 for i in o if i['got']!=i['want']); tot+=n
        if n: print(k,n)
    print('total',tot)
