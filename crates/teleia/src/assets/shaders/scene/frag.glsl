uniform sampler2D texture_data;

void main()
{
    vec4 texel = texture(texture_data, vertex_texcoord);
    if (texel.a != 1.0) {
        discard;
    }

    frag_color = vec4(texel.rgb, texel.a);
} 
