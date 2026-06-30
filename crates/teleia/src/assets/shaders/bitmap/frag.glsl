#version 300 es
precision highp float;

uniform sampler2D texture_data;
uniform vec4 background;

in vec2 vertex_texcoord;
in vec3 vertex_color;
out vec4 frag_color;

void main() {
    vec4 texel = texture(texture_data, vertex_texcoord);
    // if (vertex_color == vec3(0.0, 0.0, 0.0)) {
    //     discard;
    // }
    if (texel.rgb == vec3(0.0, 0.0, 0.0)) {
        frag_color = background;
        return;
    }
    texel.rgb = vertex_color;
    frag_color = texel;
} 
