use crate::app::colors::UI_BACKGROUND;
use crate::app::file_picker::resources::{DirectoryEntry, FilePicker};
use crate::app::{AppState, Args};
use bevy::prelude::*;

#[derive(Component)]
pub struct FilePickerUi;

#[derive(Component)]
pub struct CurrentDirectoryLabel;

#[derive(Component)]
pub struct DirectoryListing;

#[derive(Component)]
pub struct DirectoryEntryListItem {
    entry: DirectoryEntry,
}

#[derive(Message)]
pub enum NavigateMessage {
    Up,
    Directory(DirectoryEntry),
    Refresh,
}

pub fn build_file_picker_ui(
    mut commands: Commands,
    mut writer: MessageWriter<NavigateMessage>,
) -> Result {
    commands.spawn((
        FilePickerUi,
        Camera2d,
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(10)),
            column_gap: px(10),
            ..default()
        },
        children![
            (
            Node {
                width: percent(100),
                height: px(50),
                display: Display::Flex,
                padding: UiRect::all(px(5)),
                row_gap: px(15),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            BackgroundColor(UI_BACKGROUND),
            children![
                (
                    Button,
                    Node {
                        padding: UiRect::all(px(5)),
                        ..default()
                    },
                    BackgroundColor(Color::linear_rgb(0.2, 0.2, 0.2)),
                    Observer::new(|mut event: On<Pointer<Click>>, mut navigate_writer: MessageWriter<NavigateMessage>| {
                        event.propagate(false);
                        navigate_writer.write(NavigateMessage::Up);
                        info!("go up");
                    }),
                    children![(Text::new("Up"),)],
                ),
                (CurrentDirectoryLabel, Text::new(""),)
            ]
        ),
        (
            DirectoryListing,
            Node {
            width: percent(100),
            flex_grow: 1.0,
            overflow: Overflow::scroll_y(),
            padding: UiRect::all(px(5)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
            BackgroundColor(UI_BACKGROUND),
        )
        ],
    ));
    writer.write(NavigateMessage::Refresh);
    Ok(())
}

#[allow(clippy::type_complexity)]
pub fn handle_list_item_interaction(
    mut commands: Commands,
    mut navigate_writer: MessageWriter<NavigateMessage>,
    mut query: Query<
        (&Interaction, &DirectoryEntryListItem, &mut BackgroundColor),
        (With<DirectoryEntryListItem>, Changed<Interaction>),
    >,
) -> Result {
    for (interaction, item, mut bg_color) in query.iter_mut() {
        match *interaction {
            Interaction::Hovered => {
                *bg_color = Color::linear_rgba(0.0, 0.0, 0.0, 0.5).into();
            }
            Interaction::Pressed => {
                *bg_color = Color::linear_rgba(0.0, 0.0, 0.0, 0.75).into();
            }
            Interaction::None => {
                *bg_color = Color::linear_rgba(0.0, 0.0, 0.0, 0.1).into();
            }
        }
        if *interaction == Interaction::Pressed {
            if item.entry.is_dir {
                navigate_writer.write(NavigateMessage::Directory(item.entry.clone()));
            } else {
                commands.insert_resource(Args {
                    input: Some(item.entry.path.clone()),
                    output: None,
                    analyze_only: false,
                    ..Default::default()
                });
                commands.set_state(AppState::Analyze);
            }
        }
    }
    Ok(())
}

pub fn handle_navigate_message(
    mut commands: Commands,
    mut reader: MessageReader<NavigateMessage>,
    mut file_picker: ResMut<FilePicker>,
    list_container_query: Query<Entity, With<DirectoryListing>>,
    mut current_dir_label: Query<&mut Text, With<CurrentDirectoryLabel>>,
) -> Result {
    let list_container = list_container_query.single()?;
    for (event, _) in reader.par_read() {
        match event {
            NavigateMessage::Up => {
                if let Some(parent) = file_picker.current_dir.parent() {
                    file_picker.current_dir = parent.to_owned();
                    file_picker.refresh()?;
                }
            }
            NavigateMessage::Directory(entry) => {
                file_picker.current_dir = entry.path.clone();
                file_picker.refresh()?;
            }
            NavigateMessage::Refresh => {
                file_picker.refresh()?;
            }
        }
    }
    commands.entity(list_container).despawn_children();
    for entry in file_picker.entries.iter() {
        let entry = entry.clone();
        let file_name = entry.path.file_name().unwrap().to_str().unwrap().to_owned();
        commands.entity(list_container).with_children(|builder| {
            builder.spawn((
                Button,
                DirectoryEntryListItem {
                    entry: entry.clone(),
                },
                Node {
                    width: percent(100),
                    padding: UiRect::all(px(5)),
                    ..default()
                },
                BackgroundColor(Color::linear_rgba(0.0, 0.0, 0.0, 0.1)),
                children![(Text::new(file_name),),],
            ));
        });
    }
    let mut current_dir_label = current_dir_label.single_mut()?;
    current_dir_label.0 = file_picker.current_dir.to_str().unwrap().to_owned();
    Ok(())
}

pub fn teardown_file_picker_ui(
    mut commands: Commands,
    query: Query<Entity, With<FilePickerUi>>,
) -> Result {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    Ok(())
}
