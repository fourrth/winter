#version 320 es
layout (location = 0) in vec3 vertPosition;
layout (location = 1) in vec3 vertColor;

out vec4 vertexColor;

layout(location = 5) uniform mat4 uMVP;

void main()
{
    gl_Position = uMVP * vec4(vertPosition,1.0);
    vertexColor = vec4(vertColor,1.0);
}