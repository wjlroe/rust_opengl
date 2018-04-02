#version 150 core

in vec4 a_Pos;
in vec3 a_Color;
out vec3 v_Color;

layout(std140)
uniform Locals {
  mat4 u_View;
  mat4 u_Projection;
};

void main() {
  v_Color = a_Color;
  gl_Position = u_Projection * u_View * a_Pos;
}
