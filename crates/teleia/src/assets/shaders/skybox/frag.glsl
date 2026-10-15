uniform samplerCube skybox;

in vec3 vertex_texcoord_3d;

void main() {
    vec3 v = vec3(vertex_texcoord_3d.x, -vertex_texcoord_3d.z, vertex_texcoord_3d.y);
    if (v.y < 0.0) discard;
    frag_color = texture(skybox, v);
} 
