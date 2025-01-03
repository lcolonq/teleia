in vec4 joint;
in vec4 weight;

uniform mat4 joint_matrices[128];

void main()
{
    vertex_texcoord = texcoord;
    vertex_normal = (normal_matrix * vec4(normal, 1.0)).xyz;
    mat4 skin
        = weight.x * joint_matrices[int(joint.x)]
        + weight.y * joint_matrices[int(joint.y)]
        + weight.z * joint_matrices[int(joint.z)]
        + weight.w * joint_matrices[int(joint.w)];
    vec3 pos = (position * skin * vec4(vertex, 1.0)).xyz;
    vertex_view_vector = camera_pos - pos;
    gl_Position = projection * view * vec4(pos, 1.0);
}
