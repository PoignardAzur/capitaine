use crate::element_tree::{ElementTree, NoEvent, ReconcileCtx, VirtualDom};
use crate::glue::{DruidAppData, GlobalEventCx};
use crate::widgets::flex;

// TODO
use crate::elements::component_caller::ComponentCaller;

use druid::widget::prelude::*;
use druid::{widget, Point, Widget, WidgetPod};
use tracing::instrument;

pub type WidgetSeqOf<RootComponentState, RootComponentEvent, ReturnedTree> =
   <<ReturnedTree as ElementTree<RootComponentState, RootComponentEvent>>::BuildOutput as VirtualDom<RootComponentState, RootComponentEvent>>::TargetWidgetSeq;

pub struct RootHandler<
    RootComponentState: Default + std::fmt::Debug,
    RootComponentEvent,
    ReturnedTree: ElementTree<RootComponentState, RootComponentEvent>,
    Comp: Fn(&RootComponentState, ()) -> ReturnedTree,
> {
    pub root_component: ComponentCaller<
        RootComponentState,
        RootComponentEvent,
        (),
        ReturnedTree,
        Comp,
        (),
        NoEvent,
    >,
    pub component_state: (RootComponentState, ReturnedTree::AggregateChildrenState),
    pub vdom: Option<ReturnedTree::BuildOutput>,
    pub default_widget: WidgetPod<DruidAppData, widget::Flex<DruidAppData>>,
    pub widget: Option<
        WidgetPod<
            DruidAppData,
            flex::Flex<WidgetSeqOf<RootComponentState, RootComponentEvent, ReturnedTree>>,
        >,
    >,
}

impl<
        RootComponentState: Default + std::fmt::Debug,
        RootComponentEvent,
        ReturnedTree: ElementTree<RootComponentState, RootComponentEvent>,
        Comp: Fn(&RootComponentState, ()) -> ReturnedTree,
    > RootHandler<RootComponentState, RootComponentEvent, ReturnedTree, Comp>
{
    pub fn new(
        root_component: Comp,
        root_state: RootComponentState,
    ) -> RootHandler<RootComponentState, RootComponentEvent, ReturnedTree, Comp> {
        let default_widget = WidgetPod::new(widget::Flex::row());
        RootHandler {
            root_component: ComponentCaller {
                component: root_component,
                props: (),
                _parent_state: Default::default(),
                _parent_event: Default::default(),
                _child_state: Default::default(),
                _child_event: Default::default(),
                _returned_tree: Default::default(),
            },
            component_state: (root_state, Default::default()),
            vdom: None,
            default_widget,
            widget: None,
        }
    }

    #[instrument(level = "debug", skip(self, ctx, data, env))]
    pub fn run(&mut self, ctx: &mut EventCtx, data: &mut DruidAppData, env: &Env) {
        use tracing::debug_span;

        if let Some(prev_vdom) = self.vdom.as_mut() {
            let flex_widget = self.widget.as_mut().unwrap().widget_mut();
            let mut cx = GlobalEventCx::new(data);

            let _span_process_event = debug_span!("process_event");
            let _span_process_event = _span_process_event.enter();
            // TODO - use process_event's return?
            let _ = prev_vdom.process_event(
                &mut self.component_state.0,
                &mut self.component_state.1,
                &mut flex_widget.children_seq,
                &mut cx,
            );
            std::mem::drop(_span_process_event);
        }

        let (new_vdom, state) = debug_span!("build").in_scope(|| {
            (self.root_component.component)(&self.component_state.0, ())
                .build(std::mem::take(&mut self.component_state.1))
        });
        self.component_state.1 = state;

        if let Some(prev_vdom) = self.vdom.as_mut() {
            let flex_widget = self.widget.as_mut().unwrap().widget_mut();
            let mut reconcile_ctx = ReconcileCtx {
                event_ctx: ctx,
                data,
                env,
            };

            debug_span!("reconcile").in_scope(|| {
                new_vdom.reconcile(prev_vdom, &mut flex_widget.children_seq, &mut reconcile_ctx);
            });
            debug_span!("update_value").in_scope(|| {
                prev_vdom.update_value(new_vdom);
            });

            ctx.request_update();
        } else {
            let widget_seq = debug_span!("init_tree").in_scope(|| new_vdom.init_tree());
            // FIXME - Fix alignment to be consistent
            // (eg "Root(Button)" and "Root(Row(Button))" should be the same)
            let flex_widget = WidgetPod::new(flex::Flex {
                direction: flex::Axis::Vertical,
                flex_params: flex::FlexContainerParams {
                    cross_alignment: flex::CrossAxisAlignment::Center,
                    main_alignment: flex::MainAxisAlignment::Start,
                    fill_major_axis: false,
                },
                children_seq: widget_seq,
            });
            ctx.children_changed();
            self.widget = Some(flex_widget);
            self.vdom = Some(new_vdom);
        }

        ctx.request_paint();
    }
}

impl<
        RootComponentState: Default + std::fmt::Debug,
        RootComponentEvent,
        ReturnedTree: ElementTree<RootComponentState, RootComponentEvent>,
        Comp: Fn(&RootComponentState, ()) -> ReturnedTree,
    > Widget<DruidAppData>
    for RootHandler<RootComponentState, RootComponentEvent, ReturnedTree, Comp>
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut DruidAppData, env: &Env) {
        if let Some(widget) = &mut self.widget {
            widget.event(ctx, event, data, env);
        } else {
            self.default_widget.event(ctx, event, data, env);
        }

        self.run(ctx, data, env);
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &DruidAppData,
        env: &Env,
    ) {
        if let Some(widget) = &mut self.widget {
            widget.lifecycle(ctx, event, data, env);
        } else {
            self.default_widget.lifecycle(ctx, event, data, env);
        }
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        _old_data: &DruidAppData,
        data: &DruidAppData,
        env: &Env,
    ) {
        if let Some(widget) = &mut self.widget {
            widget.update(ctx, data, env);
        } else {
            self.default_widget.update(ctx, data, env);
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &DruidAppData,
        env: &Env,
    ) -> Size {
        let size;

        if let Some(widget) = &mut self.widget {
            size = widget.layout(ctx, bc, data, env);
            widget.set_layout_rect(ctx, data, env, (Point::ZERO, size).into());
        } else {
            size = self.default_widget.layout(ctx, bc, data, env);
            self.default_widget
                .set_layout_rect(ctx, data, env, (Point::ZERO, size).into());
        }

        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &DruidAppData, env: &Env) {
        if let Some(widget) = &mut self.widget {
            widget.paint(ctx, data, env);
        } else {
            self.default_widget.paint(ctx, data, env);
        }
    }
}
