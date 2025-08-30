//! Shows how to create graphics that snap to the pixel grid by rendering to a texture in 2D

use bevy::{
    color::palettes::css::GRAY,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
    window::WindowResized,
};

/// In-game resolution width.
const RES_WIDTH: u32 = 512;

/// In-game resolution height.
const RES_HEIGHT: u32 = 256;

/// Spacing between numbers.
const NUMBER_SPACING: f32 = 20.;

/// Default render layers for pixel-perfect rendering.
/// You can skip adding this component, as this is the default.
const PIXEL_PERFECT_LAYERS: RenderLayers = RenderLayers::layer(0);

/// Render layers for high-resolution rendering.
const HIGH_RES_LAYERS: RenderLayers = RenderLayers::layer(1);


fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, (setup_numbers, setup_camera, setup_bins))
        .add_systems(FixedUpdate, fit_canvas)
        .run();
}

/// Low-resolution texture that contains the pixel-perfect world.
/// Canvas itself is rendered to the high-resolution world.
#[derive(Component)]
struct Canvas;

/// Camera that renders the pixel-perfect world to the [`Canvas`].
#[derive(Component)]
struct InGameCamera;

/// Camera that renders the [`Canvas`] (and other graphics on [`HIGH_RES_LAYERS`]) to the screen.
#[derive(Component)]
struct OuterCamera;

#[derive(Component)]
struct Rotate;

#[derive(Component)]
struct Number(u32);

#[derive(Component)]
struct Bin;

fn setup_numbers(mut commands: Commands) {
    // Create a new entity with a single component.
    for j in 0..50 {
        for i in 0..50 {
            let nw = i % 10;
            commands.spawn((
                Number(i),
                Transform::from_xyz(-(RES_WIDTH as f32 / 2.) + i as f32 * NUMBER_SPACING, -(RES_HEIGHT as f32 / 2.) + j as f32 * NUMBER_SPACING, 0.),
                Text2d::new(nw.to_string()),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
            ));
        }
    }
}

fn setup_bins(mut commands: Commands) {
    // Create bins at the bottom of the screen
    let bin_width = 80.0;
    let bin_height = 40.0;
    let bin_count = 5;
    let bin_spacing = 10.0;
    let bar_height = 20.0;
    let bar_spacing = 5.0;
    
    // Sample percentages for each bin
    let percentages = [0.75, 0.45, 0.90, 0.30, 0.60];
    
    for i in 0..bin_count {
        let x_pos = -(RES_WIDTH as f32 / 2.) + 60.0 + i as f32 * (bin_width + bin_spacing);
        let y_pos = -(RES_HEIGHT as f32 / 2.) + bin_height / 2.0 + 40.0;
        
        // Main bin with cyan/teal color
        commands.spawn((
            Bin,
            Sprite {
                color: Color::srgba(0.0, 0.7, 0.8, 0.9),  // Cyan/teal color
                custom_size: Some(Vec2::new(bin_width, bin_height)),
                ..default()
            },
            Transform::from_xyz(x_pos, y_pos, 1.0),
            PIXEL_PERFECT_LAYERS,
        ));
        
        // Bin number label (01-05)
        commands.spawn((
            Text2d::new(format!("{:02}", i + 1)),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(x_pos, y_pos, 1.3),
            PIXEL_PERFECT_LAYERS,
        ));
        
        // Percentage bar background (dark cyan)
        commands.spawn((
            Sprite {
                color: Color::srgba(0.0, 0.2, 0.25, 0.8),
                custom_size: Some(Vec2::new(bin_width, bar_height)),
                ..default()
            },
            Transform::from_xyz(x_pos, y_pos - bin_height / 2.0 - bar_spacing - bar_height / 2.0, 1.0),
            PIXEL_PERFECT_LAYERS,
        ));
        
        // Percentage bar fill (bright cyan)
        let fill_width = bin_width * percentages[i];
        commands.spawn((
            Sprite {
                color: Color::srgba(0.0, 0.9, 1.0, 0.9),  // Bright cyan
                custom_size: Some(Vec2::new(fill_width, bar_height)),
                ..default()
            },
            Transform::from_xyz(
                x_pos - (bin_width - fill_width) / 2.0, 
                y_pos - bin_height / 2.0 - bar_spacing - bar_height / 2.0, 
                1.1
            ),
            PIXEL_PERFECT_LAYERS,
        ));
        
        // Percentage text
        commands.spawn((
            Text2d::new(format!("{}%", (percentages[i] * 100.0) as i32)),
            TextFont {
                font_size: 10.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(x_pos, y_pos - bin_height / 2.0 - bar_spacing - bar_height / 2.0, 1.2),
            PIXEL_PERFECT_LAYERS,
        ));
    }
}

fn setup_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let canvas_size = Extent3d {
        width: RES_WIDTH,
        height: RES_HEIGHT,
        ..default()
    };

    // This Image serves as a canvas representing the low-resolution game screen
    let mut canvas = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: canvas_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // Fill image.data with zeroes
    canvas.resize(canvas_size);

    let image_handle = images.add(canvas);

    // This camera renders whatever is on `PIXEL_PERFECT_LAYERS` to the canvas
    commands.spawn((
        Camera2d,
        Camera {
            // Render before the "main pass" camera
            order: -1,
            target: RenderTarget::Image(image_handle.clone().into()),
            clear_color: ClearColorConfig::Custom(GRAY.into()),
            ..default()
        },
        Msaa::Off,
        InGameCamera,
        PIXEL_PERFECT_LAYERS,
    ));

    // Spawn the canvas
    commands.spawn((Sprite::from_image(image_handle), Canvas, HIGH_RES_LAYERS));

    // The "outer" camera renders whatever is on `HIGH_RES_LAYERS` to the screen.
    // here, the canvas and one of the sample sprites will be rendered by this camera
    commands.spawn((Camera2d, Msaa::Off, OuterCamera, HIGH_RES_LAYERS));
}

/// Scales camera projection to fit the window (integer multiples only).
fn fit_canvas(
    mut resize_events: EventReader<WindowResized>,
    mut projection: Single<&mut Projection, With<OuterCamera>>,
) {
    let Projection::Orthographic(projection) = &mut **projection else {
        return;
    };
    for event in resize_events.read() {
        let h_scale = event.width / RES_WIDTH as f32;
        let v_scale = event.height / RES_HEIGHT as f32;
        projection.scale = 1. / h_scale.min(v_scale).round();
    }
}
