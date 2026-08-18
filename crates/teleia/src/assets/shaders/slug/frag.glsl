uniform sampler2D texture_data;
uniform int curves_len;

struct curve { vec2 p1, p2, p3; };
curve get_curve(int i) {
    float tx = float(i * 2) / 4096.0;
    float txn = float(i * 2 + 1) / 4096.0;
    vec4 t1 = texture(texture_data, vec2(tx, 0.0));
    vec4 t2 = texture(texture_data, vec2(txn, 0.0));
    curve ret;
    ret.p1 = t1.xy;
    ret.p2 = t1.zw;
    ret.p3 = t2.xy;
    return ret;
}
vec2 compute_curve(curve c, float t) {
    float invt = 1.0 - t;
    return invt * invt * c.p1 + 2.0 * t * invt * c.p2 + t * t * c.p3;
}
ivec2 lookup_table_curve(curve c) {
    bool yp1 = c.p1.y > 0.0;
    bool yp2 = c.p2.y > 0.0;
    bool yp3 = c.p3.y > 0.0;
    int shift = (yp1 ? 2 : 0) + (yp2 ? 4 : 0) + (yp3 ? 8 : 0);
    int res = 0x2e74 >> shift;
    return ivec2(res & 1, (res >> 1) & 1);
}
vec2 roots_curve(curve c) {
    vec2 a = c.p1 - c.p2 * 2.0 + c.p3;
    vec2 b = c.p1 - c.p2;
    float ra = 1.0 / a.y;
    float rb = 0.5 / b.y;
    float d = sqrt(max(b.y * b.y - a.y * c.p1.y, 0.0));
    float t1 = (b.y - d) * ra;
    float t2 = (b.y + d) * ra;
    if (abs(a.y) < 1.0 / 65536.0) t1 = t2 = c.p1.y * rb;
    return vec2(t1, t2);
}

void main() {
    vec2 c = vertex_texcoord - vec2(0.5, 0.5);
    int winding = 0;
    for (int i = 0; i < curves_len; ++i) {
        curve b = get_curve(i);
        if (distance(c, b.p1) < 0.01 || distance(c, b.p2) < 0.01 || distance(c, b.p3) < 0.01) {
            frag_color = vec4(0.0, 1.0, 0.0, 1.0);
            return;
        }
        b.p1 -= c;
        b.p2 -= c;
        b.p3 -= c;
        ivec2 code = lookup_table_curve(b);
        vec2 roots = roots_curve(b);
        if (code.x > 0) {
            vec2 horiz = compute_curve(b, roots.x);
            if (horiz.x >= 0.0) {
                winding += 1;
            }
        }
        if (code.y > 0) {
            vec2 horiz = compute_curve(b, roots.y);
            if (horiz.x >= 0.0) {
                winding -= 1;
            }
        }
    }
    if (winding != 0) {
        frag_color = vec4(1.0, 1.0, 1.0, 1.0);
    } else {
        discard;
    }
} 
