use macroquad::rand::gen_range;
use macroquad::prelude::*;

pub const WINDOW_WIDTH: f32 = 960.0;
pub const WINDOW_HEIGHT: f32 = 600.0;

pub const CONSTRAINT_RADIUS: f32 = 300.0;

pub const GRAVITY: f32 = 1.0;

pub const PHYSICS_SUBSTEPS: usize = 4;

fn window_conf() -> Conf {
	Conf {
		window_title: "Verlet Integration Physics ~ v1.0.0".to_string(),
		window_width: WINDOW_WIDTH as i32,
		window_height: WINDOW_HEIGHT as i32,
		..Default::default()
	}
}

#[derive(Debug)]
pub struct VerletObject {
	pub position: Vec2,
	pub last_position: Vec2,
	pub acceleration: Vec2,
	pub radius: f32,
	pub color: Color,
}

impl VerletObject {
	pub fn new(position: Vec2, radius: f32) -> Self {
		Self {
			position,
			last_position: position,
			acceleration: Vec2::ZERO,
			radius,
			color: Color {
				r: gen_range(0.2, 1.0),
				g: gen_range(0.2, 1.0),
				b: gen_range(0.2, 1.0),
				a: 1.0,
			}
		}
	}

	pub fn update_position(&mut self, delta: f32) {
		let velocity = self.position - self.last_position;

		self.last_position = self.position;
		self.position += velocity + self.acceleration * delta * delta;

		self.acceleration = Vec2::ZERO;
	}

	pub fn accelerate(&mut self, acc: Vec2) {
		self.acceleration += acc;
	}
}

pub struct ChainLink {
	pub a: usize,
	pub b: usize,
	pub target_distance: f32,
}

pub struct Master {
	pub objects: Vec<VerletObject>,
	pub chain_links: Vec<ChainLink>,
}

impl Master {
	pub fn update(&mut self, delta: f32) {
		let sub_delta = delta / PHYSICS_SUBSTEPS as f32;
		for _ in 0..PHYSICS_SUBSTEPS {
			self.apply_gravity();
			self.apply_constraint();
			self.solve_collisions();
			self.apply_chain_links();

			// this is to keep the ends of the rope bridge thing static
			let old_color = self.objects[0].color;
			self.objects[0] = VerletObject::new(
				vec2(WINDOW_WIDTH * 0.5 - 210.0, WINDOW_HEIGHT * 0.5 + 100.0),
				10.0,
			);
			self.objects[0].color = old_color;

			let old_color = self.objects[14].color;
			self.objects[14] = VerletObject::new(
				vec2(WINDOW_WIDTH * 0.5 + 210.0, WINDOW_HEIGHT * 0.5 + 100.0),
				10.0,
			);
			self.objects[14].color = old_color;

			self.update_positions(sub_delta);
		}
	}

	pub fn apply_gravity(&mut self) {
		for object in self.objects.iter_mut() {
			object.accelerate(vec2(0.0, GRAVITY));
		}
	}

	pub fn apply_constraint(&mut self) {
		let position = vec2(WINDOW_WIDTH * 0.5, WINDOW_HEIGHT * 0.5);
		let radius = CONSTRAINT_RADIUS;
		for object in self.objects.iter_mut() {
			let to_object = object.position - position;
			let distance = to_object.length();
			if distance > radius - object.radius {
				let n = to_object / distance;
				object.position = position + n * (radius - object.radius);
			}
		}
	}

	pub fn solve_collisions(&mut self) {
		let object_count = self.objects.len();
		for i in 0..object_count {
			for j in 0..object_count {
				if i == j {
					continue;
				}

				let collision_axis = self.objects[i].position - self.objects[j].position;
				let distance = collision_axis.length();
				let object_distance = self.objects[i].radius + self.objects[j].radius;
				if distance < object_distance {
					let n = collision_axis / distance;
					let delta = object_distance - distance;
					self.objects[i].position += 0.5 * delta * n;
					self.objects[j].position -= 0.5 * delta * n;
				}
			}
		}
	}

	pub fn apply_chain_links(&mut self) {
		for chain_link in self.chain_links.iter_mut() {
			let axis = self.objects[chain_link.a].position - self.objects[chain_link.b].position;
			let distance = axis.length();
			let n = axis / distance;
			let delta = chain_link.target_distance - distance;
			self.objects[chain_link.a].position += 0.5 * delta * n;
			self.objects[chain_link.b].position -= 0.5 * delta * n;
		}
	}

	pub fn update_positions(&mut self, delta: f32) {
		for object in self.objects.iter_mut() {
			object.update_position(delta);
		}
	}
}

pub fn generate_objects() -> Vec<VerletObject> {
	let mut result = vec![];

	for i in 0..=14 {
		result.push(
			VerletObject::new(
				vec2(WINDOW_WIDTH * 0.5 - 210.0 + i as f32 * 30.0, WINDOW_HEIGHT * 0.5 + 100.0),
				10.0,
			)
		);
	}

	result
}

pub fn generate_chain_links() -> Vec<ChainLink> {
	let mut result = vec![];

	for i in 1..=14 {
		result.push(
			ChainLink {
				a: i - 1,
				b: i,
				target_distance: 30.0,
			}
		);
	}

	result
}

#[macroquad::main(window_conf)]
async fn main() {
	let mut master = Master {
		objects: generate_objects(),
		chain_links: generate_chain_links(),
	};

	let mut mouse_timer = 0.0;

	loop {
		if is_mouse_button_down(MouseButton::Left) {
			mouse_timer -= delta_time();
			if mouse_timer <= 0.0 {
				mouse_timer = 10.0;
				master.objects.push(
					VerletObject::new(
						vec2(WINDOW_WIDTH * 0.5 + 180.0, WINDOW_HEIGHT * 0.5),
						gen_range(10.0, 40.0),
					)
				);
			}
		} else {
			mouse_timer = 0.0;
		}

		master.update(delta_time());

		clear_background(Color {
			r: 0.09,
			g: 0.09,
			b: 0.12,
			a: 1.0,
		});

		draw_circle(
			WINDOW_WIDTH * 0.5,
			WINDOW_HEIGHT * 0.5,
			CONSTRAINT_RADIUS,
			BLACK,
		);

		for object in master.objects.iter() {
			draw_circle(
				object.position.x,
				object.position.y,
				object.radius,
				object.color,
			);
		}

		draw_text(
			&format!("FPS: {}", get_fps()),
			20.0,
			30.0,
			32.0,
			WHITE,
		);

		draw_text(
			&format!("OBJECTS: {}", master.objects.len()),
			20.0,
			80.0,
			32.0,
			WHITE,
		);

		next_frame().await
	}
}

fn delta_time() -> f32 {
	get_frame_time() * 60.0
}