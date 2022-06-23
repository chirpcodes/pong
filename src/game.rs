// Dependencies

use crate::structs::{Vec2, Object, ObjectType};

// Create a struct representing our game state.
// This will store object states, scores, etc, and be responsible for simulating each frame update.

pub struct GameState {
	pub objects: Vec<Object>,
	pub control_id: usize,
	pub ai_accuracy: f32
}

impl GameState {
	pub fn new() -> Self {
		Self {
			objects: vec![],
			control_id: 0,
			ai_accuracy: 0.5
		}
	}

	pub fn update(&mut self, delta_time: f32, width: f32, height: f32) {
		let mut ball_track: Option<(Vec2,Vec2,)> = None;

		let mut colliders = vec![];
		for obj in &self.objects {
			if obj.obj_type == ObjectType::Ball {
				ball_track = Some((obj.position, obj.velocity,));
			}

			colliders.push(obj.get_collider());
		}

		for i in 0..self.objects.len() {
			let obj = &mut self.objects[i];
			let mut obj_collider = colliders[i];

			// Handle simulation and physics for this object.

			let mut delta = Vec2 {
				x: obj.velocity.x * delta_time,
				y: obj.velocity.y * delta_time
			};

			match obj.obj_type {
				// Behaviour for ball movement and collision.
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
								let angle = obj_collider.center.y - other.center.y;
								obj.velocity.y = if angle >= 0.0 { new_y } else { -new_y };

								delta.x = -delta.x;
								delta.y = -delta.y;
							}
						}
					}
				},
				// AI behaviour for non-controlled paddle.
				ObjectType::PaddleLeft => if let Some(track) = ball_track {
					let (pos, vel) = track;

					let is_incoming = if obj_collider.center.x < pos.x {
						vel.x < 0.0
					} else {
						vel.x > 0.0
					};

					let mut y_tar = (height / 2.0) - (obj.size.y / 2.0);
					if is_incoming {
						let x_diff = obj_collider.center.x - pos.x;
						let eta = x_diff / vel.x;
						y_tar = pos.y + (vel.y * eta);
					}

					obj.position.y = obj.position.y + (
						(y_tar - obj.size.y / 2.0) - obj.position.y
					) * (delta_time * 0.0015 * self.ai_accuracy);
				},
				_ => ()
			}

			obj.position += delta;
		}
	}

	pub fn reset_objects(&mut self, width: f32, height: f32) {
		for obj in &mut self.objects {
			obj.reset(width, height);
		}
	}

	pub fn get_control(&mut self) -> &mut Object {
		&mut self.objects[self.control_id]
	}
}