#version 300 es
precision highp float;

uniform sampler2D texture_data;

in vec2 vertex_texcoord;
in vec3 vertex_color;
out vec4 frag_color;

void main()
{
    float val = texture(texture_data, vertex_texcoord).r;
    if (val == 0.0) discard;
    vec4 texel = vec4(vertex_color, val);
    frag_color = texel;
} 
