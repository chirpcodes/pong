#version 150

in vec2 position;

uniform mat4 perspective;
uniform mat4 matrix;

void main() {
	gl_Position = perspective * matrix * vec4(position, 0.0, 1.0);
}