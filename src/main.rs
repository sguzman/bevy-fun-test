use bevy::{
	app::AppExit,
	core_pipeline::clear_color::ClearColorConfig,
	prelude::*,
	sprite::{ColorMaterial, MaterialMesh2dBundle},
	DefaultPlugins,
};

use bevy_editor_pls::EditorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};

// Component for velocity
#[derive(Debug, Clone, Copy, PartialEq, Component)]
struct Velocity(Vec3);

// Component for mass
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct Mass(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct Pause(bool);

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugin(PanCamPlugin::default())
		.add_plugin(EditorPlugin)
		.add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
		.add_plugin(bevy::diagnostic::EntityCountDiagnosticsPlugin)
		.add_startup_system(setup)
		.add_system(update_from_velocity)
		.add_system(update_from_gravity)
		.add_system(handle_collision)
		.add_system(exit_on_escape_system)
		.add_system(pause_game)
		.run();
}

// Function to pause	the game
fn pause_game(keyboard_input: Res<Input<KeyCode>>, mut pause: Query<&mut Pause>) {
	if keyboard_input.just_pressed(KeyCode::Space) {
		println!("Paused toggle: {:#?}", pause);
		let mut pause = pause.iter_mut().next().unwrap();
		pause.0 = !pause.0;
	}
}

// Function that exits on escape
fn exit_on_escape_system(
	keyboard_input: Res<Input<KeyCode>>,
	mut app_exit_events: ResMut<Events<AppExit>>,
) {
	if keyboard_input.just_pressed(KeyCode::Escape) {
		app_exit_events.send(AppExit);
	}
}

// Function system to handle collision by reversing the velocity
fn handle_collision(pause: Query<&Pause>, mut query: Query<(&mut Velocity, &Transform)>) {
	if pause.iter().next().unwrap().0 {
		return;
	}

	let mut pairs = query.iter_combinations_mut::<2>();
	while let Some([(mut v1, t1), (mut v2, t2)]) = pairs.fetch_next() {
		if t1.translation.distance(t2.translation) < 10. {
			v1.0 = -v1.0;
			v2.0 = -v2.0;
		}
	}
}

// Update translation from velocity
fn update_from_velocity(
	time: Res<Time>,
	pause: Query<&Pause>,
	mut query: Query<(&Velocity, &mut Transform)>,
) {
	if pause.iter().next().unwrap().0 {
		return;
	}
	for (velocity, mut transform) in query.iter_mut() {
		transform.translation += velocity.0 * time.delta_seconds();
	}
}

// Given two vectors of two masses, return the new velocity of the first mass
fn calculate_new_velocity(m1: &Mass, m2: &Mass, t1: &Transform, t2: &Transform) -> Vec3 {
	let m1 = m1.0 as f32;
	let m2 = m2.0 as f32;

	let direction = t2.translation - t1.translation;
	let distance = direction.length();
	let force = 1.0 / distance.powi(2);
	let force = direction.normalize() * force;

	let force = force * m1 * m2;
	force
}

// Update velocity from the gravity of the other bodies
fn update_from_gravity(
	pause: Query<&Pause>,
	mut query: Query<(&Mass, &Transform, &mut Velocity)>,
) {
	if pause.iter().next().unwrap().0 {
		return;
	}
	// Get both entities and their components
	let mut pairs = query.iter_combinations_mut::<2>();
	while let Some([(m1, t1, mut v1), (m2, t2, mut v2)]) = pairs.fetch_next() {
		let force = calculate_new_velocity(m1, m2, t1, t2);
		v1.0 += force;
		v2.0 -= force;
	}
}

// Function to spawn the entities, given a location and color
fn spawn_entity(
	commands: &mut Commands,
	meshes: &mut ResMut<Assets<Mesh>>,
	materials: &mut ResMut<Assets<ColorMaterial>>,
	x: f32,
) {
	commands
		.spawn(MaterialMesh2dBundle {
			mesh: meshes.add(shape::Circle::new(5.).into()).into(),
			material: materials.add(ColorMaterial::from(Color::BLUE)),
			transform: Transform::from_translation(Vec3::new(x, 0., 0.)),
			..default()
		})
		.insert(Velocity(Vec3::new(0., 10., 0.)))
		.insert(Mass(10));
}

fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<ColorMaterial>>,
) {
	commands
		.spawn(Camera2dBundle {
			camera_2d: Camera2d {
				clear_color: ClearColorConfig::Custom(Color::rgb(0.0, 0.0, 0.0)), // black color
			},
			..Default::default()
		})
		.insert(PanCam {
			grab_buttons: vec![MouseButton::Left, MouseButton::Middle], // which buttons should drag the camera
			enabled: true,        // when false, controls are disabled. See toggle example.
			zoom_to_cursor: true, // whether to zoom towards the mouse or the center of the screen
			min_scale: 0.01,      // prevent the camera from zooming too far in
			max_scale: Some(40.), // prevent the camera from zooming too far out
			..Default::default()
		});

	for i in 0..20 {
		spawn_entity(&mut commands, &mut meshes, &mut materials, 100.0 * i as f32);
	}

	commands.spawn(Pause(false));
}
