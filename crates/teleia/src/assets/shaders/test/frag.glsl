// uniform int has_normal_map;
// uniform sampler2D normal_map;

uniform sampler2D texture_data;

void main()
{
    vec2 inverted_texcoord = vec2(vertex_texcoord.x, 1.0 - vertex_texcoord.y);
    vec4 texel = texture(texture_data, inverted_texcoord);
    if (texel.a != 1.0) {
        discard;
    }

    // mat3 tbn = compute_tbn();
    // vec3 normal = has_normal_map != 0
    //     ? normalize(tbn * (texture(normal_map, inverted_texcoord).xyz * 2.0 - 1.0))
    //     : normalize(vertex_normal);
    vec3 normal = normalize(vertex_normal);

    vec3 lighting = compute_lighting_noshadow(normal);

    frag_color = vec4(texel.rgb * lighting, texel.a);
} 
