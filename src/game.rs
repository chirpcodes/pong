// Dependencies

use crate::structs::{Vec2, Object, ObjectType};

use glium::Display;

// Create a struct representing our game state.
// This will store object states, scores, etc, and be responsible for simulating each frame update.

pub struct GameState {
	pub objects: Vec<Object>,
	pub control_id: usize,
	pub ai_accuracy: f32,
	pub paused: bool
}

impl GameState {
	pub fn new() -> Self {
		Self {
			objects: vec![],
			control_id: 0,
			ai_accuracy: 0.75,
			paused: true
		}
	}

	// Event loop for game physics and simulation.
	pub fn update(&mut self, delta_time: f32, width: f32, height: f32) {
		// Do not simulate if game is paused.
		if self.paused {return};

		// Build a list of colliders and track ball movement.

		let mut colliders = vec![];
		let mut ball_track: Option<(Vec2,Vec2,)> = None;
		for obj in &self.objects {
			if obj.obj_type == ObjectType::Ball {
				ball_track = Some((obj.position, obj.velocity,));
			}

			colliders.push(obj.get_collider());
		}

		// Behaviour & Logic Loop
		for i in 0..self.objects.len() {
			let obj = &mut self.objects[i];
			let mut obj_collider = colliders[i];

			// Handle simulation and physics for this object.

			// How much to move the object by this frame.
			let mut delta = Vec2 {
				x: obj.velocity.x * delta_time,
				y: obj.velocity.y * delta_time
			};

			match obj.obj_type {
				// Behaviour for ball movement and collision.
				ObjectType::Ball => {
					let center = obj.get_center();
					// Check if ball is out of bounds.
					if center.x < 0.0 || center.x > width {
						// If it is, reset to its original position.
						obj.reset(width, height);
					} else {
						// Check if next position update will cause a collision.

						obj_collider.min += delta;
						obj_collider.max += delta;

						// Check if ball will hit the horizontal edges of the screen.
						if center.y < obj.size.y / 2.0 || center.y > height - obj.size.y / 2.0 {
							// Flip y velocity.
							obj.velocity.y = -obj.velocity.y;
							delta.y = -(delta.y * 1.2);
						} else {
							// Otherwise, iterate through each collider to check for a collision.
							for o in 0..colliders.len() {
								if o == i {
									// Don't collide with self
									continue;
								}

								// Check if this object is colliding with the ball.
								let other = &colliders[o];
								if obj_collider.is_colliding(other) {
									// Increase x velocity of the ball and flip it in the other direction.
									obj.velocity.x = -(obj.velocity.x * 1.15).clamp(-obj.max_velocity.x, obj.max_velocity.x);

									// Increase and flip y velocity based on where the ball hit the paddle.
									// Ball travels upwards if it hit the upper half, and downwards if it hit the lower half.
									// Velocity increases the further away from the center it was hit.
									let angle = center.y - other.center.y;
									let traj = ((angle.abs() * 2.0) / center.y).clamp(0.0, 1.0);
									obj.velocity.y = if angle >= 0.0 { traj } else { -traj };

									// Update position delta.
									delta.x = -delta.x;
									delta.y = -delta.y;
								}
							}
						}
					}
				},
				// AI behaviour for non-controlled paddle.
				ObjectType::PaddleLeft => if let Some(track) = ball_track {
					let (pos, vel) = track;

					// Check if ball is moving towards this paddle.
					let is_incoming = if obj_collider.center.x < pos.x {
						vel.x < 0.0
					} else {
						vel.x > 0.0
					};

					// Y co-ordinate to move towards, center of screen by default.
					let mut y_tar = (height / 2.0) - (obj.size.y / 2.0);

					// Calculate y co-ordinate the ball will intercept at
					if is_incoming && y_tar > 0.0 && y_tar < height {
						let x_diff = obj_collider.center.x - pos.x;
						let time = x_diff / vel.x;
						let y_move = vel.y * time;

						let mut y_pos = pos.y + y_move;
						if y_pos < 0.0 {
							y_pos = height * 0.25;
						} else if y_pos > height {
							y_pos = height * 0.75;
						}
						y_tar = y_pos;
					}

					// Interpolate position towards target co-ordinate.
					// Accuracy affects the speed of this movement.
					obj.position.y = (obj.position.y + (
						y_tar - obj.size.y / 2.0 - obj.position.y
					) * (delta_time * 0.0015 * self.ai_accuracy))
					.clamp(0.0, height - obj.size.y);
				},
				_ => ()
			}

			obj.position += delta;
		}
	}

	// Reset all objects to their starting state.
	pub fn reset_objects(&mut self, width: f32, height: f32) {
		for obj in &mut self.objects {
			obj.reset(width, height);
		}
	}

	// Get player-controlled object.
	pub fn get_control(&mut self) -> &mut Object {
		&mut self.objects[self.control_id]
	}

	// Pause or unpause the game.
	pub fn pause(&mut self, display: &Display, pause: bool) {
		let gl_window = display.gl_window();
		let window = gl_window.window();
		window.set_cursor_grab(!pause).ok();
		window.set_cursor_visible(pause);
		self.paused = pause;
	}
}