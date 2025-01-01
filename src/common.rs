use std::io;

use bevy::utils::error;
use bevy::{diagnostic::DiagnosticsStore, prelude::*};
use bevy_ratatui::event::{KeyEvent, MouseEvent};
use bevy_ratatui::terminal::RatatuiContext;
use bevy_ratatui_camera::RatatuiCameraWidget;

use crate::Flags;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            draw_scene_system.map(error),
            handle_keyboard_system,
            handle_mouse_system,
        ),
    );
}

fn draw_scene_system(
    mut ratatui: ResMut<RatatuiContext>,
    widget: Query<&RatatuiCameraWidget>,
    _flags: Res<Flags>,
    _diagnostics: Res<DiagnosticsStore>,
) -> io::Result<()> {
    ratatui.draw(|frame| {
        if let Ok(widget) = widget.get_single() {
            frame.render_widget(widget, frame.area());
        }
    })?;

    Ok(())
}

fn handle_keyboard_system(
    mut ratatui_events: EventReader<KeyEvent>,
    mut exit: EventWriter<AppExit>,
) {
    for _ in ratatui_events.read() {
        exit.send_default();
    }
}

fn handle_mouse_system(mut events: EventReader<MouseEvent>, mut exit: EventWriter<AppExit>) {
    for _ in events.read() {
        exit.send_default();
    }
}
