#version 450

layout(location = 0) in vec3 position;

layout(set = 0, binding = 0) uniform Data {
  mat4 matrix;
} uniforms;

layout(location = 0) out vec3 vertex_color;

void main() {
    vertex_color = vec3(1.0, 0.0, 0.0);
    gl_Position = uniforms.matrix * vec4(position, 1.0);
}
