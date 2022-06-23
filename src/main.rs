// Dependencies

// Imports from standard rustc libraries.
// https://docs.rs/rustc-std-workspace-std/1.0.1/std/index.html

use std::time::{Instant, Duration};

// Glium is the library being used as an OpenGL wrapper.
// https://crates.io/crates/glium

use glium::{
	Program,
	Display, Surface,
	uniform
};

// Glutin is the library used by Glium for OpenGL context creation.
// https://crates.io/crates/glutin

use glium::glutin::{
	event,
	event_loop::{EventLoop, ControlFlow},
	window::WindowBuilder,
	ContextBuilder
};

// Import structs.rs from codebase

mod structs;
use structs::{Vec2, Rect, Object, ObjectType};

// Import basic shaders from file.

const VERTEX_SHADER_SRC: &'static str = include_str!("./shaders/vertex_shader.vsh");
const FRAGMENT_SHADER_SRC: &'static str = include_str!("./shaders/fragment_shader.frag");

// Main function.
// This will create the window, declare game variables, then run the event loop.

pub fn main() {
	// Create a handler for the event loop.

	let event_loop = EventLoop::new();

	// Initialise the display window.

	let win_build = WindowBuilder::new();
	let ctx_build = ContextBuilder::new();
	let display = Display::new(win_build, ctx_build, &event_loop).expect("Failed to create Display");

	{
		/*let window = display.gl_window();
		window.window().set_cursor_grab(true);
		window.window().set_cursor_visible(false);*/
	}

	// Build a program from GLSL source code.
	// This compiles the shaders and links them together for rendering.

	let program = Program::from_source(&display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None).unwrap();

	// Create the objects to be rendered in the game.
	// Since the game only uses basic rect shapes, it's more performant to build a single VBO here and manipulate it to fit each rendered object.

	let rect = Rect::new(&display, 1.0, 1.0);

	let mut objects = vec![
		Object::new(ObjectType::Ball).set_size(25.0, 25.0),
		Object::new(ObjectType::PaddleLeft).set_size(25.0, 100.0),
		Object::new(ObjectType::PaddleRight).set_size(25.0, 100.0)
	];

	// Control inputs will affect the PaddleRight object.
	let control_id = 2;

	// Store the window dimension and perspective matrix here so that it doesn't have to be recalculated every frame.
	// Only recalculate on the initial frame or on a window resize, otherwise it isn't necessary.

	let (mut width, mut height) = (0.0, 0.0);

	let mut perspective: Option<[[f32; 4]; 4]> = None;
	let mut perspective_update = true;

	// Initialise a variable used to implement delta time. This is a representation of the amount of time elapsed since the last frame.
	// This is done to update the scene independently of framerate, instead fixed by time.

	let mut last_frame = Instant::now();

	// Start running the event loop.
	// This will keep the display window open until the event loop exits.

	event_loop.run(move |event, _, control_flow| {
		// Calculate the delta time, then set a timer for the next frame to be drawn.
		// Delta time will be used later when calculating movements for on-screen objects.

		let now = Instant::now();
		let delta_time = (now - last_frame).as_nanos() as f32 / 1_000_000.0;
		last_frame = now;

		let next_frame_time = now + Duration::from_nanos(16_666_667);
		*control_flow = ControlFlow::WaitUntil(next_frame_time);

		// Start drawing this frame.

		let mut frame = display.draw();
		frame.clear_color(0.0, 0.0, 0.0, 1.0);

		// Calculate the perspective matrix. This achieves 3 things:
		// It stops the content of the window from stretching to match the screen.
		// It moves the origin point (0,0) to the top-left of the window instead of the center.
		// It allows co-ordinates to be calculated by pixel, giving a screen space equivalent to the dimensions of the window rather than a range of -1 to 1.

		if perspective_update {
			// Get the width and height dimensions of the display window.
			let (_width, _height) = frame.get_dimensions();
			(width, height) = (_width as f32, _height as f32);

			// Reset all objects to their initial positions.
			// This first happens when the game starts, and also prevents unintended behaviour if the window resizes.

			for obj in &mut objects {
				obj.reset(width, height);
			}
			
			// Build the perspective matrix.
			perspective = Some({
				[
					[2.0 / width, 0.0, 0.0, 0.0],
					[0.0, -2.0 / height, 0.0, 0.0],
					[0.0, 0.0, 1.0, 0.0],
					[-1.0, 1.0, 0.0, 1.0]
				]
			});
			perspective_update = false;
		}

		// Iterate through every object and update them for this frame.

		{
			let mut colliders = vec![];
			for obj in &objects {
				colliders.push(obj.get_collider());
			}

			for i in 0..objects.len() {
				let obj = &mut objects[i];
				let mut obj_collider = colliders[i];

				// Handle simulation and physics for this object.

				let mut delta = Vec2 {
					x: obj.velocity.x * delta_time,
					y: obj.velocity.y * delta_time
				};

				match obj.obj_type {
					ObjectType::Ball => {
						// Check if ball is out of bounds.
						if obj.is_out_of_bounds(width, height) {
							// If it is, reset to its original position.
							obj.reset(width, height);
						} else {
							// Check if next position update will cause a collision.

							obj_collider.min += delta;
							obj_collider.max += delta;

							for o in 0..colliders.len() {
								if o == i {
									// Don't collide with self
									continue;
								}

								let other = &colliders[o];
								if obj_collider.is_colliding(other) {
									obj.velocity.x = -(obj.velocity.x * 1.15).clamp(-obj.max_velocity.x, obj.max_velocity.x);

									let new_y = (obj.velocity.y * 1.15).clamp(-obj.max_velocity.y, obj.max_velocity.y).abs();
									let angle = obj.position.y + (obj.size.y / 2.0) - (other.min.y + ((other.max.y - other.min.y) / 2.0));
									obj.velocity.y = if angle >= 0.0 {
										new_y
									} else {
										-new_y
									};
									
									delta.x = -delta.x;
									delta.y = -delta.y;
								}
							}
						}
					},
					_ => ()
				}

				obj.position += delta;

				// Render this object.

				let uniforms = uniform!{
					perspective: perspective.unwrap(),
					matrix: [
						[obj.size.x, 0.0, 0.0, 0.0],
						[0.0, obj.size.y, 0.0, 0.0],
						[0.0, 0.0, 1.0, 0.0],
						[obj.position.x, obj.position.y, 1.0, 1.0]
					]
				};

				frame.draw(&rect.vx_buf, &rect.ix_buf, &program, &uniforms, &Default::default()).unwrap();
			}
		}

		frame.finish().unwrap();

		// Handle input events from the system, such as keypresses or mouse movements.

		let control_obj = &mut objects[control_id];

		match event {
			// A window event has been received, check its type and handle it.
			event::Event::WindowEvent { event, .. } => match event {
				// The close button has been pressed, exit the program.
				event::WindowEvent::CloseRequested => {
					*control_flow = ControlFlow::Exit;
					return;
				},
				// The window was resized, recalculate the perspective on the next frame.
				event::WindowEvent::Resized(_size) => {
					perspective_update = true;
				},
				// Ignore anything else.
				_ => ()
			},
			// Received event from an input device (mouse, keyboard).
			event::Event::DeviceEvent { device_id: _, event, .. } => match event {
				// The player moved their mouse.
				event::DeviceEvent::MouseMotion { delta, .. } => {
					// Change position of the player controlled object according to how much the mouse moved.
					control_obj.position.y += delta.1 as f32 * 2.0 * delta_time;
				},
				// Ignore anything else.
				_ => ()
			},
			// Ignore events that aren't being listened for.
			_ => ()
		}
	});
}