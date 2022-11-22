use bevy::{
    log::LogPlugin,
    prelude::*,
    render::{
        mesh::shape::Cube,
        settings::{WgpuFeatures, WgpuSettings},
    },
    sprite::collide_aabb,
    window::close_on_esc,
};
use bevy_hanabi::prelude::*;
use iyes_loopless::prelude::*;

const PLAYER_WIDTH: f32 = 34.0;
const PLAYER_HEIGHT: f32 = 24.0;
const PLAYER_FRAMES: usize = 4;
const PLAYER_LAYER: f32 = 4.0;
const PIPE_WIDTH: f32 = 52.0;
const PIPE_HEIGHT: f32 = 320.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    MainMenu,
    InGame,
    GameOver
}

#[derive(Component)]
struct MenuEntity;
#[derive(Component)]
struct InGameEntity;

#[derive(Component)]
struct GameOverEntity;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Animation;

#[derive(Component)]
struct Obstacle;

#[derive(Component)]
struct Background;

#[derive(Component)]
struct Velocity {
    y: f32,
}

#[derive(Resource, Deref)]
struct FlapSoundEffect(Handle<AudioSource>);

#[derive(Resource, Deref)]
struct DieSoundEffect(Handle<AudioSource>);

impl Velocity {
    fn default() -> Self {
        Velocity { y: 0.0 }
    }
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

//#[derive(Component, Deref, DerefMut)]
//struct Random(rand_chacha::ChaCha8Rng);

fn global_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    //Dont really need HDR - need to figure out how it works ;)
    let camera = Camera2dBundle {
        camera: Camera { ..default() },
        projection: OrthographicProjection {
            scale: 0.5,
            ..OrthographicProjection::default()
        },

        ..Camera2dBundle::default()
    };
    commands.spawn(camera);

    let flap: Handle<AudioSource> = asset_server.load("Wing.ogg");
    commands.insert_resource(FlapSoundEffect(flap));

    let die: Handle<AudioSource> = asset_server.load("die.ogg");
    commands.insert_resource(DieSoundEffect(die));
    //commands.insert_resource(Random(rand::thread_rng()));
}

//Menu

fn menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let sprite = SpriteBundle {
        texture: asset_server.load("message.png"),
        ..default()
    };

    commands.spawn(sprite).insert(MenuEntity);
}

fn menu_keyboard_input(keys: Res<Input<KeyCode>>, mut commands: Commands) {
    if keys.just_pressed(KeyCode::Space) {
        commands.insert_resource(NextState(GameState::InGame));
    }
}


// Game Over

fn gameover_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let sprite = SpriteBundle {
        texture: asset_server.load("gameover.png"),
        ..default()
    };

    commands.spawn(sprite).insert(GameOverEntity);
}


fn gameover_keyboard_input(keys: Res<Input<KeyCode>>, mut commands: Commands) {
    if keys.just_pressed(KeyCode::Space) {
        commands.insert_resource(NextState(GameState::MainMenu));
    }
}

fn obstacle_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    for i in 2..6 {
        let sprite = SpriteBundle {
            transform: Transform {
                translation: Vec3::new(i as f32 * 200.0, -250.0, 1.0),
                ..Transform::default()
            },
            texture: asset_server.load("pipe-green.png"),
            ..default()
        };

        commands.spawn(sprite).insert(Obstacle).insert(InGameEntity);

        let sprite = SpriteBundle {
            transform: Transform {
                translation: Vec3::new(i as f32 * 200.0, 250.0, 1.0),
                ..Transform::default()
            },
            texture: asset_server.load("pipe-green.png"),
            sprite: Sprite {
                flip_y: true,
                ..Sprite::default()
            },
            ..default()
        };

        commands.spawn(sprite).insert(Obstacle).insert(InGameEntity);
    }
}

fn background_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    for i in -3..3 {
        let sprite = SpriteBundle {
            transform: Transform {
                translation: Vec3::new(i as f32 * 288.0, 0.0, 0.0),
                ..Transform::default()
            },
            texture: asset_server.load("background-day.png"),
            ..default()
        };

        commands.spawn(sprite).insert(Background).insert(InGameEntity);
    }
}

fn rainbow_fart_setup(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

) {
    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::splat(1.0));
    color_gradient1.add_key(0.1, Vec4::new(1.0, 1.0, 0.0, 1.0));
    color_gradient1.add_key(0.4, Vec4::new(1.0, 0.0, 0.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::splat(0.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec2::splat(1.0));
    size_gradient1.add_key(0.5, Vec2::splat(5.0));
    size_gradient1.add_key(0.8, Vec2::splat(0.8));
    size_gradient1.add_key(1.0, Vec2::splat(0.0));

    let effect1 = effects.add(
        EffectAsset {
            name: "emit:rate".to_string(),
            capacity: 32768,
            spawner: Spawner::rate(1000.0.into()),
            ..Default::default()
        }
        .init(PositionSphereModifier {
            center: Vec3::ZERO,
            radius: 2.,
            dimension: ShapeDimension::Surface,
            speed: 6.0.into(),
        })
        .update(AccelModifier {
            accel: Vec3::new(-200., -3., 100.),
        })
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient1,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient1,
        }),
    );

    let cube = meshes.add(Mesh::from(Cube { size: 1.0 }));
    let mat = materials.add(Color::PURPLE.into());

    commands
        .spawn((
            Name::new("emit:rate"),
            ParticleEffectBundle {
                effect: ParticleEffect::new(effect1),
                transform: Transform::from_translation(Vec3::new(-10., 0., 2.)),
                ..Default::default()
            },
        ))
        .with_children(|p| {
            // Reference cube to visualize the emit origin
            p.spawn((
                PbrBundle {
                    mesh: cube.clone(),
                    material: mat.clone(),
                    ..Default::default()
                },
                Name::new("source"),
            ));
        })
        .insert(InGameEntity);
}

fn player_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("flappy.png");
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT),
        PLAYER_FRAMES,
        1,
        None,
        None,
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let animation = SpriteSheetBundle {
        texture_atlas: texture_atlas_handle,
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, PLAYER_LAYER),
            ..Transform::default()
        },

        ..default()
    };

    /* 
    
        */

    commands
        .spawn(animation)
        .insert(Player)
        .insert(InGameEntity)
        .insert(Velocity::default())
        .insert(AnimationTimer(Timer::from_seconds(
            0.10,
            TimerMode::Repeating,
        )));
}

fn keyboard_input(keys: Res<Input<KeyCode>>, mut query: Query<&mut Velocity>) {
    let mut player = query.single_mut();
    if keys.just_pressed(KeyCode::Space) {
        player.y = 3.0;
    }
}

fn play_flap(keys: Res<Input<KeyCode>>, audio: Res<Audio>, sound_effect: Res<FlapSoundEffect>) {
    if keys.just_pressed(KeyCode::Space) {
        audio.play(sound_effect.0.clone_weak());
    }
}

fn gravity(mut query: Query<(&mut Velocity, &mut Transform)>) {
    let (mut velocity, mut transform) = query.single_mut();
    velocity.y -= 0.05;
    transform.translation.y += velocity.y;
}

fn animation(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = (sprite.index + 1) % texture_atlas.textures.len();
        }
    }
}

fn scroll_background(mut pipes: Query<&mut Transform, With<Background>>) {
    for mut t in pipes.iter_mut() {
        if t.translation.x <= -500.0 {
            t.translation.x = 500.0;
        } else {
            t.translation.x -= 0.5;
        }
    }
}

fn move_pipes(mut pipes: Query<&mut Transform, With<Obstacle>>) {
    for mut t in pipes.iter_mut() {
        if t.translation.x <= -400.0 {
            t.translation.x = 400.0;
        } else {
            t.translation.x -= 2.0;
        }
    }
}

fn move_particles(
    player: Query<&mut Transform, (With<Player>, Without<ParticleEffect>)>,
    mut particles: Query<&mut Transform, (With<ParticleEffect>, Without<Player>)>,
) {
    let player = player.single();
    let mut particles = particles.single_mut();
    particles.translation.y = player.translation.y;
}

fn check_collisions(
    mut commands: Commands,
    audio: Res<Audio>,
    die: Res<DieSoundEffect>,
    player: Query<(&Transform, Entity), (With<Player>, Without<Obstacle>)>,
    obstacles: Query<&mut Transform, (With<Obstacle>, Without<Player>)>,
) {
    let (player, player_entity) = player.single();
    let player_pos = player.translation;
    let player_size = Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT);

    for obstacle in obstacles.iter() {
        let obstacle_pos = obstacle.translation;
        let obstacle_size = Vec2::new(PIPE_WIDTH, PIPE_HEIGHT);
        let collision = collide_aabb::collide(player_pos, player_size, obstacle_pos, obstacle_size);
        match collision {
            Some(_) => {
                audio.play(die.clone());
                commands.entity(player_entity).despawn();
                commands.insert_resource(NextState(GameState::GameOver));
            }
            None => {}
        }
    }
}

fn cleanup<TEntity>(mut entities: Query<(Entity, With<TEntity>)>,
mut commands: Commands,)  where TEntity : Component{
    for (entity, _) in entities.iter_mut() {
        commands.entity(entity).despawn();
    }
}

fn main() {
    let mut options = WgpuSettings::default();
    options
        .features
        .set(WgpuFeatures::VERTEX_WRITABLE_STORAGE, true);

    App::new()
        .insert_resource(options)
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        width: 400.0,
                        ..default()
                    },
                    ..default()
                })
                .set(LogPlugin {
                    level: bevy::log::Level::WARN,
                    filter: "bevy_hanabi=warn,spawn=trace".to_string(),
                }),
        )
        //.add_plugin(HanabiPlugin)
        .add_startup_system(global_setup)
        .add_loopless_state(GameState::MainMenu)

        .add_enter_system(GameState::MainMenu, menu_setup)
        .add_exit_system(GameState::MainMenu, cleanup::<MenuEntity>)

        .add_exit_system(GameState::InGame, cleanup::<InGameEntity>)
        .add_enter_system(GameState::InGame, player_setup)
        //.add_enter_system(GameState::InGame, rainbow_fart_setup)
        .add_enter_system(GameState::InGame, obstacle_setup)
        .add_enter_system(GameState::InGame, background_setup)

        .add_enter_system(GameState::GameOver, gameover_setup)
        .add_exit_system(GameState::GameOver, cleanup::<GameOverEntity>)

        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::MainMenu)
                .with_system(menu_keyboard_input)
                .into()
        )

        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::InGame)
                .with_system(animation)
                //.with_system(move_particles)
                .with_system(scroll_background)
                .with_system(move_pipes)
                .with_system(gravity)
                .with_system(keyboard_input)
                .with_system(check_collisions)
                .with_system(play_flap)
                .into()
        )

        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::GameOver)
                .with_system(gameover_keyboard_input)
                .into()
        ) 

        .add_system(close_on_esc)

        
        .run();
}
