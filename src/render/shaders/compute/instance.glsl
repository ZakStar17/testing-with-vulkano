#version 450
layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(push_constant) uniform PushConstantData {
  mat4 projection_view;
} pc;

layout(set = 0, binding = 0) readonly buffer InputData {
  mat4 model[];
}
inputData;

layout(set = 0, binding = 1) buffer OutputData {
  mat4 matrix[];
}
outputData;

void main() {
  uint idx = gl_GlobalInvocationID.x;
  outputData.matrix[idx] = pc.projection_view * inputData.model[idx];
}