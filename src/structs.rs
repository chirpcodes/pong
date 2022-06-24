// Dependencies

use std::ops::{Add, AddAssign, Sub};

// Imports from the Glium library:
use glium::{
	// Struct macros.
	implement_vertex,
	// Imports for VBOs and VBO Indexing.
	VertexBuffer, IndexBuffer,
	Display,
	index::PrimitiveType
};

// Implement a Vertex struct used to represent vertices.

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
	pub position: [f32; 2]
}
implement_vertex!(Vertex, position);

// Implement a Vec2 (2D Vector) struct representing a co-ordinate in 2D space.

#[derive(Copy, Clone, Debug)]
pub struct Vec2 {
	pub x: f32,
	pub y: f32
}

impl Vec2 {
	pub fn new(x: f32, y: f32) -> Self {
		Self { x:x, y:y }
	}

	pub fn set(&mut self, x: f32, y: f32) {
		self.x = x;
		self.y = y;
	}
}

impl Add for Vec2 { // Implement + operator for this struct
	type Output = Self;

	fn add(mut self, other: Vec2) -> Self {
		self.x += other.x;
		self.y += other.y;
		self
	}
}

impl AddAssign for Vec2 { // Implement += operator for this struct
	fn add_assign(&mut self, other: Vec2) {
		*self = *self + other;
	}
}

impl Sub for Vec2 { // Implement - operator for this struct
	type Output = Self;

	fn sub(mut self, other: Vec2) -> Self {
		self.x -= other.x;
		self.y -= other.y;
		self
	}
}

// Implement a Rect struct representing a drawn rectangle.

pub struct Rect {
	pub vx_buf: VertexBuffer<Vertex>,
	pub ix_buf: IndexBuffer<u8>
}

impl Rect {
	pub fn new(display: &Display, width: f32, height: f32) -> Self {

		// Create a shape given the dimensions of the rect, construct a VBO out of it.
		let vx_buf = VertexBuffer::new(display, &[
			Vertex { position: [0.0, 0.0] },
			Vertex { position: [width, 0.0] },
			Vertex { position: [width, height] },
			Vertex { position: [0.0, height] }
		]).unwrap();

		// Build an index for the vertex buffer.
		let ix_buf = IndexBuffer::<u8>::new(display, PrimitiveType::TrianglesList, &[
			0, 1, 2,
			2, 3, 0
		]).unwrap();

		// Construct the Rect object.
		Self {
			vx_buf: vx_buf,
			ix_buf: ix_buf
		}
	}
}

// Implement an Object struct representing a game object.
// These objects have a type, they can be either a Ball or a Paddle.

#[derive(PartialEq)]
pub enum ObjectType {
	Ball,
	PaddleLeft,
	PaddleRight
}

pub struct Object {
	pub obj_type: ObjectType,
	pub position: Vec2,
	pub size: Vec2,
	pub velocity: Vec2,
	pub max_velocity: Vec2
}

impl Object {
	// Build a new Object with default properties.
	pub fn new(obj_type: ObjectType) -> Self {
		Self {
			obj_type: obj_type,
			position: Vec2 { x:0.0, y:0.0 },
			size: Vec2 { x:1.0, y:1.0 },
			velocity: Vec2 { x:0.0, y:0.0 },
			max_velocity: Vec2 { x:2.0, y:2.0 }
		}
	}

	// Set the size of this object inline.
	pub fn set_size(mut self, x: f32, y: f32) -> Self {
		self.size.set(x, y);
		self
	}

	// Reset this object to its starting state.
	pub fn reset(&mut self, width: f32, height: f32) {
		match self.obj_type {
			ObjectType::Ball => {
				self.velocity.set(
					width / 3200.0,
					0.0
				);
				self.max_velocity.set(
					width / 400.0,
					height / 400.0
				);
				self.position.set(
					(width / 2.0) - (self.size.x / 2.0),
					(height / 2.0) - (self.size.y / 2.0)
				);
			},
			ObjectType::PaddleLeft => {
				self.size.y = height * 0.25;
				self.position.set(
					width * 0.05,
					(height / 2.0) - (self.size.y / 2.0)
				);
			},
			ObjectType::PaddleRight => {
				self.size.y = height * 0.25;
				self.position.set(
					width * 0.95 - self.size.x,
					(height / 2.0) - (self.size.y / 2.0)
				);
			}
		}
	}

	// Build an ObjectCollider for this object.
	pub fn get_collider(&self) -> ObjectCollider {
		ObjectCollider::new(self)
	}

	// Get the center point for this object.
	pub fn get_center(&self) -> Vec2 {
		Vec2 {
			x: self.position.x + (self.size.x / 2.0),
			y: self.position.y + (self.size.y / 2.0)
		}
	}
}

// Implement object colliders.

// Function to check if line segments intersect.
fn ccw(a: &Vec2, b: &Vec2, c: &Vec2) -> bool {
	(c.y - a.y) * (b.x - a.x) > (b.y - a.y) * (c.x - a.x)
}

#[derive(Copy, Clone, Debug)]
pub struct ObjectCollider {
	pub min: Vec2,
	pub max: Vec2,
	pub center: Vec2
}

impl ObjectCollider {
	// Build a new ObjectCollider given an Object.
	pub fn new(obj: &Object) -> Self {
		Self {
			min: obj.position,
			max: obj.position + obj.size,
			center: obj.get_center()
		}
	}

	pub fn get_hitbox(&self) -> [[Vec2; 2]; 4] {
		[
			[Vec2::new(self.min.x, self.min.y), Vec2::new(self.max.x, self.min.y)],
			[Vec2::new(self.min.x, self.min.y), Vec2::new(self.min.x, self.max.y)],
			[Vec2::new(self.min.x, self.max.y), Vec2::new(self.max.x, self.max.y)],
			[Vec2::new(self.max.x, self.max.y), Vec2::new(self.max.x, self.min.y)]
		]
	}

	// Check if this object is intercepting another collider.
	pub fn is_colliding(&self, other: &Self) -> bool {
		let self_hitbox = self.get_hitbox();
		let other_hitbox = other.get_hitbox();

		let mut is_colliding = false;
		for [a, b] in &self_hitbox {
			for [c, d] in &other_hitbox {
				// Check if line segments intersect. If they are, then the objects are colliding.
				let intersect = ccw(a,c,d) != ccw(b,c,d) && ccw(a,b,c) != ccw(a,b,d);
				if !is_colliding {
					is_colliding = intersect;
				}
			}
		}
		is_colliding
	}
}