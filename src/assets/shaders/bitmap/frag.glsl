#version 300 es
precision highp float;

uniform sampler2D texture_data;

in vec2 vertex_texcoord;
in vec3 vertex_color;
out vec4 frag_color;

void main() {
    vec4 texel = texture(texture_data, vertex_texcoord);
    if (texel.rgb == vec3(0.0, 0.0, 0.0)) discard;
    texel.r = vertex_color.r;
    texel.g = vertex_color.g;
    texel.b = vertex_color.b;
    frag_color = texel;
} 
