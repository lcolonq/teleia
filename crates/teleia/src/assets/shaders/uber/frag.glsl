uniform highp int flags;

uniform vec3 camera_pos;
uniform float time;

uniform vec2 sprite_offset;
uniform vec2 sprite_dims;

uniform vec4 color;
uniform sampler2D texture_color;
uniform sampler2D texture_normal;
uniform vec3 light_ambient_color;
uniform vec3 light_dir;
uniform vec3 light_dir_color;
uniform int light_count;
uniform vec3 light_pos[5];
uniform vec3 light_color[5];
uniform vec2 light_attenuation[5];

uniform int effect_flip;
uniform float effect_flash;
uniform float effect_hueshift;
uniform float effect_huescale;

in vec2 vertex_texcoord;
in vec3 vertex_normal;
in vec3 vertex_fragpos;
in vec4 vertex_fragpos_shadow_dir;
in vec3 vertex_view_vector;

out vec4 frag_color;

bool flag(int mask) {
    return (flags & mask) != 0;
}

mat3 compute_tbn() {
    vec3 p = -vertex_view_vector;
    vec3 normal = normalize(vertex_normal);
    vec3 dpx = dFdx(p);
    vec3 dpy = dFdy(p);
    vec2 duvx = dFdx(vertex_texcoord);
    vec2 duvy = dFdy(vertex_texcoord);
    vec3 dpyperp = cross(dpy, normal);
    vec3 dpxperp = cross(normal, dpx);
    vec3 tangent = dpyperp * duvx.x + dpxperp * duvy.x;
    vec3 bitangent = dpyperp * duvx.y + dpxperp * duvy.y;
    float invmax = inversesqrt(max(dot(bitangent, bitangent), dot(bitangent, bitangent)));
    float flip = flag(FLIP_TEXTURE) ? 1.0 : -1.0;
    return mat3(-tangent * invmax, flip * bitangent * invmax, normal);

}

vec4 normal_as_color(vec3 n) {
    float r = (128.0 + 127.0 * n.r) / 255.0;
    float g = (128.0 + 127.0 * n.g) / 255.0;
    float b = (128.0 + 127.0 * n.b) / 255.0;
    return vec4(r, g, b, 1.0);
}

vec3 dir_light(vec3 normal) {
    return max(dot(normal, -normalize(light_dir)), 0.0) * light_dir_color;
}

vec3 point_light(vec3 normal, const int idx) {
    vec3 pos = light_pos[idx];
    vec3 color = light_color[idx];
    float linear = light_attenuation[idx].x;
    float quadratic = light_attenuation[idx].y;
    vec3 light_vector = pos - vertex_fragpos;
    float distance = length(light_vector);
    float attenuation = 1.0 / (1.0 + distance * linear + distance * distance * quadratic);

    float directional = max(dot(normal.xyz, normalize(light_vector)), 0.0);
    vec3 directional_light = color * directional;

    vec3 view_dir = normalize(camera_pos - vertex_fragpos);
    vec3 reflect_dir = reflect(-normalize(light_vector), normalize(normal.xyz));
    float specular = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
    vec3 specular_light = 0.5 * specular * color;
    // return (directional_light + specular_light) * attenuation;
    return directional_light * attenuation;
}

vec3 point_light_billboard(const int idx) {
    vec3 pos = light_pos[idx];
    vec3 color = light_color[idx];
    float linear = light_attenuation[idx].x;
    float quadratic = light_attenuation[idx].y;
    vec3 light_vector = pos - vertex_fragpos;
    float distance = length(light_vector);
    float attenuation = 1.0 / (1.0 + distance * linear + distance * distance * quadratic);

    return color * attenuation;
}

vec3 rgb_to_hsl(vec3 rgb) {
    vec3 ret;
    float min = min(min(rgb.r, rgb.g), rgb.b);
    float max = max(max(rgb.r, rgb.g), rgb.b);
    float lum = (max + min) / 2.0;
    ret.z = lum;
    if (max == min) {
        ret.x = ret.y = 0.0;
    } else {
        float chroma = max - min;
        ret.y = chroma / (1.0 - abs(2.0 * lum - 1.0));
        if (max == rgb.r) {
            ret.x = (rgb.g - rgb.b) / chroma + (rgb.g < rgb.b ? 6.0 : 0.0);
        } else if (max == rgb.g) {
            ret.x = (rgb.b - rgb.r) / chroma + 2.0;
        } else {
            ret.x = (rgb.r - rgb.g) / chroma + 4.0;
        }
        ret.x /= 6.0;
    }
    return ret;
}

float hue_to_rgb(float p, float q, float t) {
    if (t < 0.0) t += 1.0;
    if (t > 1.0) t -= 1.0;
    if (t < 1.0/6.0) return p + (q - p) * 6.0 * t;
    if (t < 1.0/2.0) return q;
    if (t < 2.0/3.0) return p + (q - p) * (2.0/3.0 - t) * 6.0;
    return p;
}

vec3 hsl_to_rgb(vec3 hsl) {
    vec3 ret;
    if (hsl.y == 0.0) {
        ret.r = ret.g = ret.b = hsl.z;
    } else {
        float q = hsl.z < 0.5 ? hsl.z * (1.0 + hsl.y) : hsl.z + hsl.y - hsl.z * hsl.y;
        float p = 2.0 * hsl.z - q;
        ret.r = hue_to_rgb(p, q, hsl.x + 1.0/3.0);
        ret.g = hue_to_rgb(p, q, hsl.x);
        ret.b = hue_to_rgb(p, q, hsl.x - 1.0/3.0);
    }
    return ret;
}

void main() {
    float tcy = flag(FLIP_TEXTURE) ? vertex_texcoord.y : 1.0 - vertex_texcoord.y;
    vec2 tc = vec2(vertex_texcoord.x, tcy);
    mat3 tbn = compute_tbn();
    if (flag(EFFECTS)) {
        float fbase = float(effect_flip);
        float fmul = 1.0 - 2.0 * fbase;
        tc = vec2(fbase + fmul * vertex_texcoord.x, tcy);
    }
    if (flag(SPRITE)) {
        tc *= sprite_dims;
        tc += sprite_offset;
    }
    frag_color = color;
    if (flag(TEXTURE_COLOR)) {
        frag_color = texture(texture_color, tc);
    }
    if (flag(EFFECTS)) {
        vec3 hsl = rgb_to_hsl(frag_color.rgb);
        hsl.x = mod(hsl.x * effect_huescale + effect_hueshift, 1.0);
        vec3 p = hsl_to_rgb(hsl);
        frag_color.rgb = vec3(p.r + effect_flash, p.g + effect_flash, p.b + effect_flash);
    }
    vec3 normal = vertex_normal;
    if (flag(TEXTURE_NORMAL)) {
        normal = normalize(tbn * (texture(texture_normal, tc).xyz * 2.0 - 1.0));
    }
    vec3 from_ambient = vec3(1.0, 1.0, 1.0);
    if (flag(LIGHT_AMBIENT)) {
        from_ambient = light_ambient_color;
    }
    vec3 from_dir = vec3(0.0, 0.0, 0.0);
    if (flag(LIGHT_DIR)) {
        from_dir = dir_light(normal);
    }
    vec3 from_points = vec3(0.0, 0.0, 0.0);
    if (flag(LIGHT_POINT)) {
        for (int i = 0; i < light_count; ++i) {
            vec3 pl = point_light(normal, i);
            from_points += pl;
        }
    }
    frag_color.rgb *= (from_ambient + from_dir + from_points);
    if (frag_color.a == 0.0) {
        discard;
    }
}
