uniform sampler2D texture_data;

uniform float transparency;

void main()
{
    float opacity = 1.0 - clamp(transparency, 0.0, 1.0);
    vec2 tcfull = vec2(vertex_texcoord.x, 1.0 - vertex_texcoord.y);
    vec4 texel = texture(texture_data, tcfull);
    texel.a *= opacity;
    frag_color = texel;
} 
