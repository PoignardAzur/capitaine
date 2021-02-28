use panoramix::elements::{Button, ButtonPressed, ElementList, Label};
use panoramix::widgets::flex::{CrossAxisAlignment, FlexContainerParams, MainAxisAlignment};
use panoramix::{make_group, make_row, CompCtx, Element, ElementExt, NoEvent, RootHandler};
use panoramix_derive::component;

use druid::PlatformError;

const ROW_FLEX_PARAMS: FlexContainerParams = FlexContainerParams {
    cross_alignment: CrossAxisAlignment::Center,
    main_alignment: MainAxisAlignment::Center,
    fill_major_axis: true,
};

#[derive(Debug, Default, Clone, PartialEq)]
struct ListItem {
    text: String,
    id: i32,
}

#[derive(Debug, Default, Clone, PartialEq)]
struct AppState {
    data: Vec<ListItem>,
    selected_row: Option<usize>,
    next_id: i32,
}

type RowEvent = ButtonPressed;
// TODO - private type leak?
pub struct RowProps {
    list_item: ListItem,
    is_selected: bool,
}

#[component]
fn ListRow(ctx: &CompCtx, props: RowProps) -> impl Element<u16, RowEvent> {
    let age = ctx.use_local_state::<u16>();
    make_row!(
        Button::new("Select").map_event(|state: &mut u16, event| {
            *state += 1;
            Some(event)
        }),
        Label::new(if props.is_selected { " [*]" } else { " [ ]" }),
        Label::new(format!("{} - age={}", &props.list_item.text, age)),
        Label::new(format!("id={}", props.list_item.id)),
    )
    .with_flex_container_params(ROW_FLEX_PARAMS)
}

fn editable_list(state: &AppState, _props: ()) -> impl Element<AppState, NoEvent> {
    let button_create = Button::new("Create").on::<ButtonPressed, _>(|state: &mut AppState, _| {
        state.data.push(ListItem {
            text: "new item".to_string(),
            id: state.next_id,
        });
        state.next_id += 1;
    });
    let button_insert = Button::new("Insert").on::<ButtonPressed, _>(|state: &mut AppState, _| {
        state.data.insert(
            0,
            ListItem {
                text: "inserted item".to_string(),
                id: state.next_id,
            },
        );
        state.next_id += 1;
    });
    let button_delete = Button::new("Delete").on::<ButtonPressed, _>(|state: &mut AppState, _| {
        if let Some(row) = state.selected_row {
            state.data.remove(row as usize);
            state.selected_row = None;
        }
    });
    let button_update = Button::new("Update").on::<ButtonPressed, _>(|state: &mut AppState, _| {
        if let Some(row) = state.selected_row {
            state.data[row as usize].text = "updated".to_string();
        }
    });

    let list_view_data = state
        .data
        .iter()
        .enumerate()
        .map(|(i, list_item)| {
            let row_props = RowProps {
                list_item: list_item.clone(),
                is_selected: state.selected_row == Some(i),
            };

            let list_row =
                ListRow::new(row_props).on::<RowEvent, _>(move |state: &mut AppState, _| {
                    state.selected_row = Some(i);
                });

            (list_item.id.to_string(), list_row)
        })
        .collect();
    let list_view = ElementList {
        children: list_view_data,
        _comp_state: Default::default(),
        _comp_event: Default::default(),
    };

    make_group!(
        make_row!(button_create, button_insert, button_delete, button_update,)
            .with_flex_container_params(ROW_FLEX_PARAMS),
        list_view,
    )
}

fn main() -> Result<(), PlatformError> {
    let state = AppState {
        data: (0..8_i32)
            .map(|i| ListItem {
                text: "hello".to_string(),
                id: i,
            })
            .collect(),
        selected_row: None,
        next_id: 8,
    };

    RootHandler::new(&editable_list, state)
        .with_tracing(true)
        .launch()
}
