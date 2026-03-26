use crate::app::file_picker::resources::FilePicker;
use crate::app::file_picker::ui;
use crate::app::file_picker::ui::{
    handle_list_item_interaction, handle_navigate_message, NavigateMessage,
};
use crate::app::AppState;
use bevy::app::{App, Plugin, Update};
use bevy::prelude::{in_state, IntoScheduleConfigs, OnEnter, OnExit};

pub struct FilePickerPlugin;

impl Plugin for FilePickerPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<NavigateMessage>()
            .insert_resource(FilePicker::default())
            .add_systems(OnEnter(AppState::FilePicker), ui::build_file_picker_ui)
            .add_systems(OnExit(AppState::FilePicker), ui::teardown_file_picker_ui)
            .add_systems(
                Update,
                (handle_navigate_message, handle_list_item_interaction)
                    .run_if(in_state(AppState::FilePicker)),
            );
    }
}
