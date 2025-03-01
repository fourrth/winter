#version 310 es

out mediump vec4 outputF;

in mediump vec4 vertexColor;

layout (location = 2) uniform mediump float timeu;

void main()
{
	outputF = abs(sin(timeu)) * vertexColor;
}