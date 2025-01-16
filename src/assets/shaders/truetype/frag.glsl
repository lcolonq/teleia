#version 300 es
precision highp float;

uniform sampler2D texture_data;
uniform int text[256];
uniform int atlas_width;
uniform int cell_width;
uniform int text_width;

in vec2 vertex_texcoord;
out vec4 frag_color;

void main()
{
    vec2 inverted_texcoord = vec2(vertex_texcoord.x, 1.0 - vertex_texcoord.y);
    float texcoord_pixels_x = inverted_texcoord.x * float(text_width);
    int char_idx = int(floor(texcoord_pixels_x)) / cell_width;
    int offset = text[char_idx];
    float cbase = float(offset);
    float coff = mod(texcoord_pixels_x, float(cell_width));
    float val = texture(texture_data, vec2((cbase + coff) / float(atlas_width), inverted_texcoord.y)).r;
    frag_color = vec4(val, val, val, 1.0);
} 
