#version 300 es
precision highp float;

uniform sampler2D texture_data;
uniform vec3 text_color;

in vec2 vertex_texcoord;
out vec4 frag_color;

void main() {
    vec4 texel = texture(texture_data, vertex_texcoord);
    if (texel.rgb == vec3(0.0, 0.0, 0.0)) discard;
    texel.r = text_color.r;
    texel.g = text_color.g;
    texel.b = text_color.b;
    frag_color = texel;
} 
