out vec3 vertex_texcoord_3d;

void main() {
    vertex_texcoord_3d = vertex;
    vec4 pos = projection * view * vec4(vertex, 1.0);
    gl_Position = pos.xyww;
}
