use bevy::{
    log::LogPlugin,
    prelude::*,
    render::settings::{WgpuFeatures, WgpuSettings},
    sprite::collide_aabb,
    window::close_on_esc,
};
use bevy_hanabi::prelude::*;
use iyes_loopless::prelude::*;
use rand::{thread_rng, Rng};

const PLAYER_WIDTH: f32 = 34.0;
const PLAYER_HEIGHT: f32 = 24.0;
const PLAYER_FRAMES: usize = 4;
const PIPE_WIDTH: f32 = 52.0;
const PIPE_HEIGHT: f32 = 320.0;
const PIPE_POS: f32 = 265.0;
const FLOOR_WIDTH: f32 = 336.0;
const FLOOR_HEIGHT: f32 = 112.0;
const FLOOR_POS: f32 = 200.0;
const SCROLL_SPEED: f32 = 2.0;
const BACKGROUND_WIDTH: f32 = 288.0;

const MENU_LAYER: f32 = 6.0;
const PLAYER_LAYER: f32 = 5.0;
const PIPE_LAYER: f32 = 3.0;
const FLOOR_LAYER: f32 = 4.0;
const BACKGROUND_LAYER: f32 = 1.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    MainMenu,
    InGame,
    GameOver,
}

fn create<T: Default + Component>() -> T {
    T::default()
}

#[derive(Component)]
struct MenuEntity;
impl Default for MenuEntity {
    fn default() -> Self {
        MenuEntity
    }
}
#[derive(Component)]
struct InGameEntity;
impl Default for InGameEntity {
    fn default() -> Self {
        InGameEntity
    }
}

#[derive(Component)]
struct GameOverEntity;
impl Default for GameOverEntity {
    fn default() -> Self {
        GameOverEntity
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Collidable(f32, f32);

#[derive(Component)]
struct Animation;

#[derive(Component)]
struct Pipe;

#[derive(Component)]
struct Floor;

#[derive(Component)]
struct Background;

#[derive(Component)]
struct Velocity {
    y: f32,
}

impl Velocity {
    fn default() -> Self {
        Velocity { y: 0.0 }
    }
}

#[derive(Resource, Deref)]
struct RainbowFart(Handle<EffectAsset>);

#[derive(Resource, Deref)]
struct FlapSoundEffect(Handle<AudioSource>);

#[derive(Resource, Deref)]
struct DieSoundEffect(Handle<AudioSource>);

#[derive(Component)]
struct Score(u32);

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component, Deref, DerefMut)]
struct ScoreTimer(Timer);

//#[derive(Component, Deref, DerefMut)]
//struct Random(rand_chacha::ChaCha8Rng);

fn global_setup(mut commands: Commands, asset_server: Res<AssetServer>, audio: Res<Audio>) {
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

    audio.play_with_settings(
        asset_server.load("Music.ogg"),
        PlaybackSettings::LOOP.with_volume(0.75),
    );
}

//Menu

fn menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let sprite = SpriteBundle {
        texture: asset_server.load("message.png"),
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, MENU_LAYER),
            ..Transform::default()
        },
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
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 3.0),
            ..Transform::default()
        },
        ..default()
    };

    commands.spawn(sprite).insert(GameOverEntity);
}

fn gameover_keyboard_input(keys: Res<Input<KeyCode>>, mut commands: Commands) {
    if keys.just_pressed(KeyCode::Space) {
        commands.insert_resource(NextState(GameState::MainMenu));
    }
}

fn pipe_setup<TEntity: Default + Component>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for i in 2..6 {
        let sprite = SpriteBundle {
            transform: Transform {
                translation: Vec3::new(i as f32 * 200.0, -PIPE_POS, PIPE_LAYER),
                ..Transform::default()
            },
            texture: asset_server.load("pipe-green.png"),
            ..default()
        };

        commands
            .spawn(sprite)
            .insert(Pipe)
            .insert(Collidable(PIPE_WIDTH, PIPE_HEIGHT))
            .insert(create::<TEntity>());

        let sprite = SpriteBundle {
            transform: Transform {
                translation: Vec3::new(i as f32 * 200.0, PIPE_POS, PIPE_LAYER),
                ..Transform::default()
            },
            texture: asset_server.load("pipe-green.png"),
            sprite: Sprite {
                flip_y: true,
                ..Sprite::default()
            },
            ..default()
        };

        commands
            .spawn(sprite)
            .insert(Pipe)
            .insert(Collidable(PIPE_WIDTH, PIPE_HEIGHT))
            .insert(create::<TEntity>());
    }
}

fn floor_setup<TEntity: Default + Component>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for i in 0..10 {
        let sprite = SpriteBundle {
            transform: Transform {
                translation: Vec3::new(i as f32 * FLOOR_WIDTH, -FLOOR_POS, FLOOR_LAYER),
                ..Transform::default()
            },
            texture: asset_server.load("base.png"),
            ..default()
        };

        commands
            .spawn(sprite)
            .insert(Floor)
            .insert(Collidable(FLOOR_WIDTH, FLOOR_HEIGHT))
            .insert(create::<TEntity>());
    }
}

fn background_setup<TEntity: Default + Component>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for i in -3..3 {
        let sprite = SpriteBundle {
            transform: Transform {
                translation: Vec3::new(i as f32 * BACKGROUND_WIDTH, 0.0, BACKGROUND_LAYER),
                ..Transform::default()
            },
            texture: asset_server.load("background-day.png"),
            ..default()
        };

        commands
            .spawn(sprite)
            .insert(Background)
            .insert(create::<TEntity>());
    }
}

fn rainbow_fart_onetime_setup(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
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

    let effect = effects.add(
        EffectAsset {
            //name: "emit:rate".to_string(),
            capacity: 32768,
            spawner: Spawner::rate(100.0.into()),
            ..Default::default()
        }
        .init(PositionCircleModifier {
            radius: 0.05,
            speed: 6.0.into(),
            dimension: ShapeDimension::Surface,
            ..Default::default()
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

    commands.insert_resource(RainbowFart(effect));
}

fn rainbow_fart_setup(mut commands: Commands, effect: Res<RainbowFart>) {
    commands
        .spawn(ParticleEffectBundle {
            effect: ParticleEffect::new(effect.clone()).with_z_layer_2d(Some(FLOOR_LAYER)),
            ..Default::default()
        })
        .insert(InGameEntity);
}

fn score_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(
            // Create a TextBundle that has a Text with a single section.
            TextBundle::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "",
                TextStyle {
                    font: asset_server.load("FlappyBirdy.ttf"),
                    font_size: 90.0,
                    color: Color::WHITE,
                },
            ) // Set the alignment of the Text
            .with_text_alignment(TextAlignment::TOP_LEFT)
            // Set the style of the TextBundle itself.
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(Score(0))
        .insert(InGameEntity)
        .insert(ScoreTimer(Timer::from_seconds(0.5, TimerMode::Repeating)));
}

fn score_render_system(mut query: Query<(&mut Text, &Score)>) {
    let (mut text, score) = query.single_mut();

    text.sections[0].value = format!("Score : {:04}", score.0).to_string();
}

fn score_update_system(time: Res<Time>, mut query: Query<(&mut Score, &mut ScoreTimer)>) {
    let (mut score, mut timer) = query.single_mut();
    timer.tick(time.delta());
    if timer.just_finished() {
        score.0 = score.0 + 1;
    }
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

fn rotate(mut query: Query<(&Velocity, &mut Transform)>) {
    let (velocity, mut transform) = query.single_mut();
    *transform = transform.with_rotation(Quat::from_rotation_z(f32::clamp(
        velocity.y / 2.0,
        -1.0,
        1.0,
    )));
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
        if t.translation.x <= -BACKGROUND_WIDTH * 2.0 {
            t.translation.x = BACKGROUND_WIDTH * 2.0;
        } else {
            t.translation.x -= 0.5;
        }
    }
}

fn move_pipes(mut pipes: Query<&mut Transform, With<Pipe>>) {
    let mut rng = thread_rng();
    let random = rng.gen::<f32>() - 1.0;
    for mut t in pipes.iter_mut() {
        if t.translation.x <= -400.0 {
            t.translation.x = 400.0;
            if t.translation.y > 0.0 {
                t.translation.y = PIPE_POS + 20.0 * random;
            } else {
                t.translation.y = -PIPE_POS + 20.0 * random;
            }
        } else {
            t.translation.x -= SCROLL_SPEED;
        }
    }
}

fn move_floor(mut floor: Query<&mut Transform, With<Floor>>) {
    for mut t in floor.iter_mut() {
        if t.translation.x <= -FLOOR_WIDTH * 2.0 {
            t.translation.x = FLOOR_WIDTH * 2.0;
        } else {
            t.translation.x -= SCROLL_SPEED;
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
    player: Query<(&Transform, Entity), (With<Player>, Without<Collidable>)>,
    obstacles: Query<(&mut Transform, &Collidable), Without<Player>>,
) {
    let (player, player_entity) = player.single();
    let player_pos = player.translation;
    let player_size = Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT);

    for (obstacle, col) in obstacles.iter() {
        let obstacle_pos = obstacle.translation;
        let obstacle_size = Vec2::new(col.0, col.1);
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

fn cleanup<TEntity>(mut entities: Query<(Entity, With<TEntity>)>, mut commands: Commands)
where
    TEntity: Component,
{
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
        .insert_resource(ClearColor(Color::rgb_u8(255, 87, 51)))
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        width: 800.0,
                        title: "Flappy Bird".to_string(),
                        ..default()
                    },
                    ..default()
                })
                .set(LogPlugin {
                    level: bevy::log::Level::WARN,
                    filter: "bevy_hanabi=warn,spawn=trace".to_string(),
                }),
        )
        .add_plugin(HanabiPlugin)
        .add_startup_system(global_setup)
        .add_startup_system(rainbow_fart_onetime_setup)
        .add_loopless_state(GameState::MainMenu)
        .add_enter_system(GameState::MainMenu, menu_setup)
        .add_enter_system(GameState::MainMenu, pipe_setup::<MenuEntity>)
        .add_enter_system(GameState::MainMenu, floor_setup::<MenuEntity>)
        .add_enter_system(GameState::MainMenu, background_setup::<MenuEntity>)
        .add_exit_system(GameState::MainMenu, cleanup::<MenuEntity>)
        .add_enter_system(GameState::InGame, player_setup)
        .add_enter_system(GameState::InGame, score_setup)
        .add_enter_system(GameState::InGame, rainbow_fart_setup)
        .add_enter_system(GameState::InGame, pipe_setup::<InGameEntity>)
        .add_enter_system(GameState::InGame, floor_setup::<InGameEntity>)
        .add_enter_system(GameState::InGame, background_setup::<InGameEntity>)
        .add_exit_system(GameState::InGame, cleanup::<InGameEntity>)
        .add_enter_system(GameState::GameOver, gameover_setup)
        .add_exit_system(GameState::GameOver, cleanup::<GameOverEntity>)
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::MainMenu)
                .with_system(scroll_background)
                .with_system(move_pipes)
                .with_system(move_floor)
                .with_system(menu_keyboard_input)
                .into(),
        )
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::InGame)
                .with_system(animation)
                .with_system(move_particles)
                .with_system(scroll_background)
                .with_system(move_pipes)
                .with_system(move_floor)
                .with_system(gravity)
                .with_system(score_update_system)
                .with_system(rotate)
                .with_system(keyboard_input)
                .with_system(check_collisions)
                .with_system(play_flap)
                .with_system(score_render_system)
                .into(),
        )
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::GameOver)
                .with_system(gameover_keyboard_input)
                .into(),
        )
        .add_system(close_on_esc)
        .run();
}
