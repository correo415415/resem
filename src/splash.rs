//! Startup splash screens, replicating the original SWF root timeline:
//! first the sponsor animation (gamesfre_pre_dragon_animation_406, dragon +
//! GAMESFREE.com PRESENTS on a slate-blue background), then the developer
//! intro (IntroLittleGiant_423, pixel-art Little Giant World logo on black).
//!
//! Frames were exported from the SWF renders and deduplicated; the original
//! per-frame timing is preserved as an RLE timeline at 30 fps (the SWF frame
//! rate). Clicking skips to the next phase like the original click handlers
//! (the original opened sponsor sites; we just skip).

use crate::data::AppState;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// Original Flash stage size.
const STAGE_W: f32 = 640.0;
const STAGE_H: f32 = 480.0;

/// Slate-blue background of the dragon splash (measured on reference).
const DRAGON_BG: Color = Color::srgb(59.0 / 255.0, 75.0 / 255.0, 98.0 / 255.0);

/// Dragon art is 359x89; content is centered at (331, 224) on the stage.
const DRAGON_SIZE: (f32, f32) = (359.0, 89.0);
const DRAGON_POS: (f32, f32) = (331.0 - 359.0 / 2.0 + 4.0, 224.0 - 89.0 / 2.0);

/// Intro art (cropped to 656x484) draws its logo so that it matches the
/// reference when placed at (0, -8) relative to the stage.
const INTRO_SIZE: (f32, f32) = (656.0, 484.0);
const INTRO_POS: (f32, f32) = (0.0, -8.0);

const FPS: f32 = 30.0;

/// RLE timelines: (unique frame index, hold in 30fps frames).
/// Generated from the original SWF frame sequence (dedup by content hash).
const DRAGON_TL: [(usize, u32); 34] = [
    (0, 14),
    (1, 1),
    (2, 1),
    (3, 1),
    (4, 1),
    (5, 1),
    (6, 1),
    (7, 1),
    (8, 1),
    (9, 1),
    (10, 1),
    (11, 2),
    (12, 2),
    (10, 15),
    (13, 2),
    (14, 2),
    (15, 2),
    (16, 2),
    (15, 2),
    (14, 1),
    (17, 9),
    (18, 5),
    (19, 1),
    (20, 1),
    (21, 1),
    (22, 1),
    (23, 1),
    (24, 1),
    (25, 1),
    (8, 1),
    (9, 1),
    (10, 1),
    (26, 1),
    (27, 8),
];

const INTRO_TL: [(usize, u32); 49] = [
    (0, 1),
    (1, 1),
    (2, 1),
    (3, 1),
    (4, 1),
    (5, 1),
    (6, 1),
    (7, 1),
    (8, 1),
    (9, 1),
    (10, 1),
    (11, 1),
    (12, 3),
    (13, 1),
    (14, 1),
    (15, 1),
    (16, 1),
    (17, 1),
    (18, 1),
    (19, 1),
    (20, 1),
    (21, 1),
    (22, 1),
    (23, 1),
    (24, 17),
    (25, 1),
    (26, 1),
    (27, 1),
    (28, 1),
    (29, 1),
    (30, 1),
    (31, 1),
    (32, 1),
    (33, 1),
    (34, 1),
    (35, 1),
    (36, 1),
    (37, 1),
    (38, 1),
    (39, 1),
    (40, 1),
    (41, 1),
    (42, 1),
    (43, 1),
    (44, 1),
    (45, 1),
    (46, 1),
    (47, 1),
    (48, 2),
];

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Dragon,
    Intro,
}

#[derive(Component)]
struct SplashRoot;

#[derive(Component)]
struct SplashImage;

#[derive(Component)]
struct SplashBackdrop;

#[derive(Resource)]
struct SplashState {
    phase: Phase,
    /// Elapsed 30fps frames within the current phase.
    frame: f32,
    dragon: Vec<Handle<Image>>,
    intro: Vec<Handle<Image>>,
}

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Splash), spawn_splash)
            .add_systems(OnExit(AppState::Splash), despawn_splash)
            .add_systems(
                Update,
                (splash_scale, splash_tick, splash_skip).run_if(in_state(AppState::Splash)),
            );
    }
}

/// Same 4:3 letterbox scaling as the title screen.
fn splash_scale(windows: Query<&Window, With<PrimaryWindow>>, mut scale: ResMut<UiScale>) {
    if let Ok(w) = windows.single() {
        let s = (w.width() / STAGE_W).min(w.height() / STAGE_H).max(0.1);
        if (scale.0 - s).abs() > 0.001 {
            scale.0 = s;
        }
    }
}

fn spawn_splash(mut commands: Commands, assets: Res<AssetServer>) {
    let dragon: Vec<Handle<Image>> = (0..28)
        .map(|i| assets.load(format!("sprites/splash/dragon_{i:02}.png")))
        .collect();
    let intro: Vec<Handle<Image>> = (0..49)
        .map(|i| assets.load(format!("sprites/splash/intro_{i:02}.png")))
        .collect();
    let first = dragon[0].clone();

    commands
        .spawn((
            SplashRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            GlobalZIndex(55),
        ))
        .with_children(|root| {
            root.spawn((
                SplashBackdrop,
                Node {
                    width: Val::Px(STAGE_W),
                    height: Val::Px(STAGE_H),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(DRAGON_BG),
            ))
            .with_children(|stage| {
                stage.spawn((
                    SplashImage,
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(DRAGON_POS.0),
                        top: Val::Px(DRAGON_POS.1),
                        width: Val::Px(DRAGON_SIZE.0),
                        height: Val::Px(DRAGON_SIZE.1),
                        ..default()
                    },
                    ImageNode::new(first),
                ));
            });
        });

    commands.insert_resource(SplashState {
        phase: Phase::Dragon,
        frame: 0.0,
        dragon,
        intro,
    });
}

fn despawn_splash(mut commands: Commands, roots: Query<Entity, With<SplashRoot>>) {
    for e in &roots {
        commands.entity(e).despawn();
    }
    commands.remove_resource::<SplashState>();
}

/// Resolve an RLE timeline: which unique frame is on screen at `frame`,
/// or None when the animation has ended.
fn tl_frame(tl: &[(usize, u32)], frame: f32) -> Option<usize> {
    let mut acc = 0u32;
    let f = frame as u32;
    for &(idx, hold) in tl {
        acc += hold;
        if f < acc {
            return Some(idx);
        }
    }
    None
}

fn splash_tick(
    time: Res<Time>,
    mut st: ResMut<SplashState>,
    mut img: Query<(&mut ImageNode, &mut Node), With<SplashImage>>,
    mut backdrop: Query<&mut BackgroundColor, With<SplashBackdrop>>,
    mut next: ResMut<NextState<AppState>>,
) {
    st.frame += time.delta_secs() * FPS;
    let (tl, frames): (&[(usize, u32)], &[Handle<Image>]) = match st.phase {
        Phase::Dragon => (&DRAGON_TL, &st.dragon),
        Phase::Intro => (&INTRO_TL, &st.intro),
    };
    match tl_frame(tl, st.frame) {
        Some(idx) => {
            if let Ok((mut node, _)) = img.single_mut() {
                if node.image != frames[idx] {
                    node.image = frames[idx].clone();
                }
            }
        }
        None => match st.phase {
            Phase::Dragon => {
                // Original frame68: main.gotoAndPlay("pre_app") -> intro.
                st.phase = Phase::Intro;
                st.frame = 0.0;
                if let Ok((mut node, mut n)) = img.single_mut() {
                    node.image = st.intro[0].clone();
                    n.left = Val::Px(INTRO_POS.0);
                    n.top = Val::Px(INTRO_POS.1);
                    n.width = Val::Px(INTRO_SIZE.0);
                    n.height = Val::Px(INTRO_SIZE.1);
                }
                if let Ok(mut bg) = backdrop.single_mut() {
                    bg.0 = Color::BLACK;
                }
            }
            Phase::Intro => next.set(AppState::Title),
        },
    }
}

/// Click / key skips the current splash phase (the original made the
/// splashes clickable; we skip instead of opening sponsor pages).
fn splash_skip(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut st: ResMut<SplashState>,
) {
    if mouse.just_pressed(MouseButton::Left)
        || keys.just_pressed(KeyCode::Space)
        || keys.just_pressed(KeyCode::Enter)
    {
        // Jump past the end of the current timeline; splash_tick advances.
        st.frame = 10_000.0;
    }
}
