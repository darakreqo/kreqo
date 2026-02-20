use xilem::WidgetView;
use xilem::masonry::core::{HasProperty, Property};
use xilem::style::Style;
use xilem::view::Prop;

pub trait Class<State, Action, Inner> {
    type Styled;
    fn styled(self, inner: Inner) -> Self::Styled;
}

impl<State, Action, Inner, A, B> Class<State, Action, Inner> for (A, B)
where
    State: 'static,
    Action: 'static,
    Inner: Style<State, Action>,
    A: Property + PartialEq,
    B: Property + PartialEq,
    <Inner as WidgetView<State, Action>>::Widget: HasProperty<A> + HasProperty<B>,
{
    type Styled = Prop<B, Prop<A, Inner, State, Action>, State, Action>;
    fn styled(self, inner: Inner) -> Self::Styled {
        inner.prop(self.0).prop(self.1)
    }
}

impl<State, Action, Inner, A, B, C> Class<State, Action, Inner> for (A, B, C)
where
    State: 'static,
    Action: 'static,
    Inner: Style<State, Action>,
    A: Property + PartialEq,
    B: Property + PartialEq,
    C: Property + PartialEq,
    <Inner as WidgetView<State, Action>>::Widget: HasProperty<A> + HasProperty<B> + HasProperty<C>,
{
    type Styled = Prop<C, Prop<B, Prop<A, Inner, State, Action>, State, Action>, State, Action>;
    fn styled(self, inner: Inner) -> Self::Styled {
        inner.prop(self.0).prop(self.1).prop(self.2)
    }
}

impl<State, Action, Inner, A, B, C, D> Class<State, Action, Inner> for (A, B, C, D)
where
    State: 'static,
    Action: 'static,
    Inner: Style<State, Action>,
    A: Property + PartialEq,
    B: Property + PartialEq,
    C: Property + PartialEq,
    D: Property + PartialEq,
    <Inner as WidgetView<State, Action>>::Widget:
        HasProperty<A> + HasProperty<B> + HasProperty<C> + HasProperty<D>,
{
    type Styled = Prop<
        D,
        Prop<C, Prop<B, Prop<A, Inner, State, Action>, State, Action>, State, Action>,
        State,
        Action,
    >;
    fn styled(self, inner: Inner) -> Self::Styled {
        inner.prop(self.0).prop(self.1).prop(self.2).prop(self.3)
    }
}

impl<State, Action, Inner, A, B, C, D, E> Class<State, Action, Inner> for (A, B, C, D, E)
where
    State: 'static,
    Action: 'static,
    Inner: Style<State, Action>,
    A: Property + PartialEq,
    B: Property + PartialEq,
    C: Property + PartialEq,
    D: Property + PartialEq,
    E: Property + PartialEq,
    <Inner as WidgetView<State, Action>>::Widget:
        HasProperty<A> + HasProperty<B> + HasProperty<C> + HasProperty<D> + HasProperty<E>,
{
    type Styled = Prop<
        E,
        Prop<
            D,
            Prop<C, Prop<B, Prop<A, Inner, State, Action>, State, Action>, State, Action>,
            State,
            Action,
        >,
        State,
        Action,
    >;
    fn styled(self, inner: Inner) -> Self::Styled {
        inner
            .prop(self.0)
            .prop(self.1)
            .prop(self.2)
            .prop(self.3)
            .prop(self.4)
    }
}

impl<State, Action, Inner, A, B, C, D, E, F> Class<State, Action, Inner> for (A, B, C, D, E, F)
where
    State: 'static,
    Action: 'static,
    Inner: Style<State, Action>,
    A: Property + PartialEq,
    B: Property + PartialEq,
    C: Property + PartialEq,
    D: Property + PartialEq,
    E: Property + PartialEq,
    F: Property + PartialEq,
    <Inner as WidgetView<State, Action>>::Widget: HasProperty<A>
        + HasProperty<B>
        + HasProperty<C>
        + HasProperty<D>
        + HasProperty<E>
        + HasProperty<F>,
{
    type Styled = Prop<
        F,
        Prop<
            E,
            Prop<
                D,
                Prop<C, Prop<B, Prop<A, Inner, State, Action>, State, Action>, State, Action>,
                State,
                Action,
            >,
            State,
            Action,
        >,
        State,
        Action,
    >;
    fn styled(self, inner: Inner) -> Self::Styled {
        inner
            .prop(self.0)
            .prop(self.1)
            .prop(self.2)
            .prop(self.3)
            .prop(self.4)
            .prop(self.5)
    }
}

impl<State, Action, Inner, A, B, C, D, E, F, G> Class<State, Action, Inner>
    for (A, B, C, D, E, F, G)
where
    State: 'static,
    Action: 'static,
    Inner: Style<State, Action>,
    A: Property + PartialEq,
    B: Property + PartialEq,
    C: Property + PartialEq,
    D: Property + PartialEq,
    E: Property + PartialEq,
    F: Property + PartialEq,
    G: Property + PartialEq,
    <Inner as WidgetView<State, Action>>::Widget: HasProperty<A>
        + HasProperty<B>
        + HasProperty<C>
        + HasProperty<D>
        + HasProperty<E>
        + HasProperty<F>
        + HasProperty<G>,
{
    type Styled = Prop<
        G,
        Prop<
            F,
            Prop<
                E,
                Prop<
                    D,
                    Prop<C, Prop<B, Prop<A, Inner, State, Action>, State, Action>, State, Action>,
                    State,
                    Action,
                >,
                State,
                Action,
            >,
            State,
            Action,
        >,
        State,
        Action,
    >;
    fn styled(self, inner: Inner) -> Self::Styled {
        inner
            .prop(self.0)
            .prop(self.1)
            .prop(self.2)
            .prop(self.3)
            .prop(self.4)
            .prop(self.5)
            .prop(self.6)
    }
}

impl<State, Action, Inner, A, B, C, D, E, F, G, H> Class<State, Action, Inner>
    for (A, B, C, D, E, F, G, H)
where
    State: 'static,
    Action: 'static,
    Inner: Style<State, Action>,
    A: Property + PartialEq,
    B: Property + PartialEq,
    C: Property + PartialEq,
    D: Property + PartialEq,
    E: Property + PartialEq,
    F: Property + PartialEq,
    G: Property + PartialEq,
    H: Property + PartialEq,
    <Inner as WidgetView<State, Action>>::Widget: HasProperty<A>
        + HasProperty<B>
        + HasProperty<C>
        + HasProperty<D>
        + HasProperty<E>
        + HasProperty<F>
        + HasProperty<G>
        + HasProperty<H>,
{
    type Styled = Prop<
        H,
        Prop<
            G,
            Prop<
                F,
                Prop<
                    E,
                    Prop<
                        D,
                        Prop<
                            C,
                            Prop<B, Prop<A, Inner, State, Action>, State, Action>,
                            State,
                            Action,
                        >,
                        State,
                        Action,
                    >,
                    State,
                    Action,
                >,
                State,
                Action,
            >,
            State,
            Action,
        >,
        State,
        Action,
    >;
    fn styled(self, inner: Inner) -> Self::Styled {
        inner
            .prop(self.0)
            .prop(self.1)
            .prop(self.2)
            .prop(self.3)
            .prop(self.4)
            .prop(self.5)
            .prop(self.6)
            .prop(self.7)
    }
}

impl<State, Action, Inner, A, B, C, D, E, F, G, H, I> Class<State, Action, Inner>
    for (A, B, C, D, E, F, G, H, I)
where
    State: 'static,
    Action: 'static,
    Inner: Style<State, Action>,
    A: Property + PartialEq,
    B: Property + PartialEq,
    C: Property + PartialEq,
    D: Property + PartialEq,
    E: Property + PartialEq,
    F: Property + PartialEq,
    G: Property + PartialEq,
    H: Property + PartialEq,
    I: Property + PartialEq,
    <Inner as WidgetView<State, Action>>::Widget: HasProperty<A>
        + HasProperty<B>
        + HasProperty<C>
        + HasProperty<D>
        + HasProperty<E>
        + HasProperty<F>
        + HasProperty<G>
        + HasProperty<H>
        + HasProperty<I>,
{
    type Styled = Prop<
        I,
        Prop<
            H,
            Prop<
                G,
                Prop<
                    F,
                    Prop<
                        E,
                        Prop<
                            D,
                            Prop<
                                C,
                                Prop<B, Prop<A, Inner, State, Action>, State, Action>,
                                State,
                                Action,
                            >,
                            State,
                            Action,
                        >,
                        State,
                        Action,
                    >,
                    State,
                    Action,
                >,
                State,
                Action,
            >,
            State,
            Action,
        >,
        State,
        Action,
    >;
    fn styled(self, inner: Inner) -> Self::Styled {
        inner
            .prop(self.0)
            .prop(self.1)
            .prop(self.2)
            .prop(self.3)
            .prop(self.4)
            .prop(self.5)
            .prop(self.6)
            .prop(self.7)
            .prop(self.8)
    }
}
