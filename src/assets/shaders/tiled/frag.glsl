#version 300 es
precision highp float;

uniform sampler2D texture_data;

in vec2 vertex_texcoord;
out vec4 frag_color;

void main() {
    vec4 texel = texture(texture_data, vertex_texcoord);
    frag_color = texel;
} 
