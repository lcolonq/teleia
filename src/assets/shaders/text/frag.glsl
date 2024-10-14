#version 300 es
precision highp float;

uniform sampler2D texture_data;
uniform int text_length;
uniform int text[256];
uniform int char_width;
uniform int char_height;
uniform int font_width;
uniform int font_height;
uniform int text_width;
uniform int text_height;
uniform vec3 text_color;

in vec2 vertex_texcoord;
out vec4 frag_color;

void main()
{
    vec2 inverted_texcoord = vec2(vertex_texcoord.x, 1.0 - vertex_texcoord.y);
    vec2 texcoord_pixels = inverted_texcoord * vec2(float(text_width), float(text_height));
    int texcoord_char_x = int(floor(texcoord_pixels.x)) / char_width;
    int texcoord_char_y = int(floor(texcoord_pixels.y)) / char_height;

    int x = 0;
    int y = 0;
    int i = 0;
    for (; i < text_length; ++i) {
        if (x == texcoord_char_x && y == texcoord_char_y) {
            break;
        }
        if (text[i] == 10) {
            x = 0;
            y += 1;
        } else {
            x += 1;
        }
    }
    if (i == text_length || text[i] == 10) discard;

    int entry = text[i] - 32;
    vec2 texcoord_base = vec2(
        float(entry % (font_width / char_width)) * float(char_width),
        float(entry / (font_width / char_width)) * float(char_height)
    );
    // vec2 texcoord_base = vec2(8.0, 0.0);
    vec2 texcoord_off = vec2(
        mod(texcoord_pixels.x, float(char_width)),
        mod(texcoord_pixels.y, float(char_height))
    );
    vec2 texcoord_final = (texcoord_base + texcoord_off) / vec2(float(font_width), float(font_height));

    vec4 texel = texture(texture_data, texcoord_final);
    if (texel.rgb == vec3(0.0, 0.0, 0.0)) discard;
    texel.r = text_color.r;
    texel.g = text_color.g;
    texel.b = text_color.b;
    frag_color = texel;
} 
