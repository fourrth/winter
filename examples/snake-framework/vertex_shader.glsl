#version 310 es
layout (location = 0) in vec3 vertPosition;
layout (location = 1) in vec3 vertColor;

out vec4 vertexColor;

void main()
{
    gl_Position = vec4(vertPosition,1.0);
    vertexColor = vec4(vertColor,1.0);
}