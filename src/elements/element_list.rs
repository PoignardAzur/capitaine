use crate::glue::GlobalEventCx;
use crate::widgets::WidgetList;

use crate::element_tree::{ElementTree, VirtualDom};

pub struct ElementList<Child: ElementTree<ExplicitState>, ExplicitState = ()> {
    pub elements: Vec<(String, Child)>,
    pub _state: std::marker::PhantomData<ExplicitState>,
}

pub struct ElementListTarget<Comp: VirtualDom<ParentComponentState>, ParentComponentState> {
    pub elements: Vec<(String, Comp)>,
    pub _expl_state: std::marker::PhantomData<ParentComponentState>,
}

impl<ExplicitState, Child: ElementTree<ExplicitState>> ElementTree<ExplicitState>
    for ElementList<Child, ExplicitState>
{
    type Event = (usize, Child::Event);
    type AggregateComponentState = Vec<(String, Child::AggregateComponentState)>;
    type BuildOutput = ElementListTarget<Child::BuildOutput, ExplicitState>;

    fn build(
        self,
        prev_state: Self::AggregateComponentState,
    ) -> (Self::BuildOutput, Self::AggregateComponentState) {
        let mut prev_state = prev_state;
        let (elements, state): (Vec<_>, Vec<_>) = self
            .elements
            .into_iter()
            .map(|(key, comp)| {
                // FIXME, inefficient, and only works if items are only ever
                // appended at the end and keys are unique
                let existing = prev_state
                    .iter_mut()
                    .find(|(other_key, _)| key == *other_key);
                let (new_elem, new_state) = if let Some(comp_prev_state) = existing {
                    let (_, comp_prev_state) = std::mem::take(comp_prev_state);
                    comp.build(comp_prev_state)
                } else {
                    comp.build(Default::default())
                };
                ((key.clone(), new_elem), (key, new_state))
            })
            .unzip();
        (
            ElementListTarget {
                elements,
                _expl_state: Default::default(),
            },
            state,
        )
    }
}

impl<Comp: VirtualDom<ParentComponentState>, ParentComponentState> VirtualDom<ParentComponentState>
    for ElementListTarget<Comp, ParentComponentState>
{
    type Event = (usize, Comp::Event);
    type DomState = Vec<Comp::DomState>;
    type AggregateComponentState = Vec<(String, Comp::AggregateComponentState)>;

    type TargetWidgetSeq = WidgetList<Comp::TargetWidgetSeq>;

    fn update_value(&mut self, other: Self) {
        *self = other;
    }

    fn init_tree(&self) -> (Self::TargetWidgetSeq, Self::DomState) {
        let (widgets, dom_state): (Vec<_>, Vec<_>) = self
            .elements
            .iter()
            .map(|(_, elem)| elem.init_tree())
            .unzip();

        (WidgetList { children: widgets }, dom_state)
    }

    // FIXME
    // This only works if we assume that items are ever only added at the end of the list.
    // Sounds perfectly reasonable to me.
    // (seriously though, a serious implementation would try to do whatever crochet::List::run does)
    fn apply_diff(
        &self,
        other: &Self,
        prev_state: Self::DomState,
        widget: &mut Self::TargetWidgetSeq,
    ) -> Self::DomState {
        let mut updated_state: Vec<_> = other
            .elements
            .iter()
            .zip(prev_state)
            .map(|item| {
                let (other_id, other_elem) = item.0;
                let elem_prev_state = item.1;

                if let Some(((_, elem), ref mut widget)) = self
                    .elements
                    .iter()
                    .zip(widget.children.iter_mut())
                    .find(|((id, _), _)| id == other_id)
                {
                    let elem_state = elem.apply_diff(other_elem, elem_prev_state, widget);

                    Some(elem_state)
                } else {
                    None
                }
            })
            .flatten()
            .collect();

        let (mut new_widgets, mut new_state): (Vec<_>, Vec<_>) = self
            .elements
            .iter()
            .map(|(id, elem)| {
                if other
                    .elements
                    .iter()
                    .find(|(other_id, _)| id == other_id)
                    .is_none()
                {
                    Some(elem.init_tree())
                } else {
                    None
                }
            })
            .flatten()
            .unzip();

        updated_state.append(&mut new_state);
        widget.children.append(&mut new_widgets);

        updated_state
    }

    fn process_event(
        &self,
        explicit_state: &mut ParentComponentState,
        children_state: &mut Self::AggregateComponentState,
        dom_state: &mut Self::DomState,
        _cx: &mut GlobalEventCx,
    ) -> Option<(usize, Comp::Event)> {
        for (i, elem_data) in self
            .elements
            .iter()
            .zip(children_state)
            .zip(dom_state)
            .enumerate()
        {
            let (_key, element) = elem_data.0 .0;
            let elem_comp_state = elem_data.0 .1;
            let elem_dom_state = elem_data.1;
            if let Some(event) =
                element.process_event(explicit_state, &mut elem_comp_state.1, elem_dom_state, _cx)
            {
                return Some((i, event));
            }
        }
        return None;
    }
}
