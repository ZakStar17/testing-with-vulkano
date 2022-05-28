#version 450

// vertex data
layout(location = 0) in vec3 position;

// instance data
layout(location = 1) in mat4 matrix;


layout(location = 0) out vec3 vertex_color;

void main() {
    if (gl_InstanceIndex % 2 == 0) {
        if (gl_InstanceIndex % 3 == 0) {
            vertex_color = vec3(0.8, 0.0, 0.0);
        } else {
            vertex_color = vec3(0.27);
        }
    } else {
        if (gl_InstanceIndex % 3 == 0) {
            vertex_color = vec3(0.0, 0.8, 0.0);
        } else {
            vertex_color = vec3(0.0, 0.0, 0.8);
        }
    }
    gl_Position = matrix * vec4(position, 1.0);
}
