#version 300 es
precision highp float;

uniform vec3 camera_pos;
uniform float time;

uniform vec3 light_ambient_color;
uniform vec3 light_dir;
uniform vec3 light_dir_color;
uniform int light_count;
uniform vec3 light_pos[5];
uniform vec3 light_color[5];
uniform vec2 light_attenuation[5];
uniform highp sampler2DShadow light_shadowbuffer_dir;
uniform samplerCube light_shadowbuffer_point[5];

uniform int has_point_shadows;

in vec2 vertex_texcoord;
in vec3 vertex_normal;
in vec3 vertex_fragpos;
in vec4 vertex_fragpos_shadow_dir;
in vec3 vertex_view_vector;

out vec4 frag_color;

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
    return mat3(-tangent * invmax, -bitangent * invmax, normal);
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

float dir_shadow(vec3 normal) {
    vec3 proj = vertex_fragpos_shadow_dir.xyz / vertex_fragpos_shadow_dir.w;
    float bias = 0.002;
    // float current_depth = proj.z;
    // float bias = max(0.05 * (1.0 - dot(normal, -normalize(light_dir))), 0.005);
    proj.z -= bias;
    proj *= 0.5; proj += 0.5;
    if (proj.z > 1.0) return 0.0;
    return 1.0 - texture(light_shadowbuffer_dir, proj.xyz);
}

float point_shadow(vec3 normal, vec3 shadow_vector, float closest_depth) {
    closest_depth *= 25.0;
    float current_depth = length(shadow_vector);
    float bias = max(0.1 * (1.0 - dot(normal, normalize(shadow_vector))), 0.005);
    bias = min(bias + current_depth * 0.01, 0.2);
    float shadow = current_depth - bias > closest_depth ? 1.0 : 0.0; 
    return shadow;
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
    return (directional_light + specular_light) * attenuation;
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

vec3 compute_lighting(vec3 normal) {
    vec3 ambient_light = light_ambient_color;

    vec3 from_dir = dir_light(normal) * (1.0 - dir_shadow(normal));

    vec3 shadow_vector[5];
    for (int i = 0; i < light_count; ++i) {
        shadow_vector[i] = vertex_fragpos - light_pos[i];
        shadow_vector[i].x *= -1.0;
    }

    // cannot only index array of samplers with a constant, hence the weird setup
    #define SAMPLE_SHADOW(n) n < light_count ? texture(light_shadowbuffer_point[n], shadow_vector[n]).r : 1.0
    float shadow_depth[5];
    shadow_depth[0] = SAMPLE_SHADOW(0);
    shadow_depth[1] = SAMPLE_SHADOW(1);
    shadow_depth[2] = SAMPLE_SHADOW(2);
    shadow_depth[3] = SAMPLE_SHADOW(3);
    shadow_depth[4] = SAMPLE_SHADOW(4);

    vec3 from_points = vec3(0.0, 0.0, 0.0);
    for (int i = 0; i < light_count; ++i) {
        from_points += has_point_shadows != 0
            ? point_light(normal, i) * (1.0 - point_shadow(normal, shadow_vector[i], shadow_depth[i]))
            : point_light(normal, i);
    }

    return (ambient_light + from_dir + from_points);
}

vec3 compute_lighting_noshadow(vec3 normal) {
    vec3 ambient_light = light_ambient_color;

    vec3 from_dir = dir_light(normal);

    vec3 from_points = vec3(0.0, 0.0, 0.0);
    for (int i = 0; i < light_count; ++i) {
        from_points += point_light(normal, i);
    }

    return (ambient_light + from_dir + from_points);
}

vec3 compute_lighting_billboard(vec3 normal) {
    vec3 ambient_light = light_ambient_color;

    vec3 from_dir = light_dir_color / 2.0;

    vec3 from_points = vec3(0.0, 0.0, 0.0);
    for (int i = 0; i < light_count; ++i) {
        from_points += point_light_billboard(i);
    }

    return (ambient_light + from_dir + from_points);
}
