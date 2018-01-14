use std::sync::mpsc;

use super::Error;
use super::Result;

use super::SpanContext;
use super::span_context::BaggageItem;


/// TODO
#[derive(Clone, Debug)]
pub struct FinishedSpan {
    context: SpanContext,
    name: String,
    references: Vec<SpanReference>,
}

impl FinishedSpan {
    /// TODO
    pub fn context(&self) -> &SpanContext {
        &self.context
    }

    /// TODO
    pub fn name(&self) -> &String {
        &self.name
    }

    /// TODO
    pub fn references(&self) -> &Vec<SpanReference> {
        &self.references
    }
}


/// TODO
#[derive(Clone, Debug)]
pub struct Span {
    context: SpanContext,
    name: String,
    references: Vec<SpanReference>,
    sender: SpanSender,
}

impl Span {
    /// TODO
    pub fn new(name: &str, context: SpanContext, sender: SpanSender) -> Span {
        Span {
            context,
            name: String::from(name),
            references: Vec::new(),
            sender: sender
        }
    }
}

impl Span {
    /// TODO
    pub fn child_of(&mut self, parent: SpanContext) {
        self.reference_span(SpanReference::ChildOf(parent));
    }

    /// TODO
    pub fn context(&self) -> &SpanContext {
        &self.context
    }

    /// TODO
    pub fn finish(self) -> Result<()> {
        let finished = FinishedSpan {
            context: self.context,
            name: self.name,
            references: self.references
        };
        self.sender.send(finished).map_err(|e| Error::SendError(e))?;
        Ok(())
    }

    /// TODO
    pub fn follows(&mut self, parent: SpanContext) {
        self.reference_span(SpanReference::FollowsFrom(parent));
    }

    /// TODO
    pub fn get_baggage_item(&self, key: &str) -> Option<&BaggageItem> {
        self.context.get_baggage_item(key)
    }

    /// TODO
    pub fn references(&self) -> &[SpanReference] {
        &self.references
    }

    /// TODO
    pub fn set_baggage_item(&mut self, key: &str, value: &str) {
        self.context.set_baggage_item(BaggageItem::new(key, value));
    }
}

impl Span {
    /// TODO
    fn reference_span(&mut self, reference: SpanReference) {
        self.context.reference_span(&reference);
        match reference {
            SpanReference::ChildOf(ref parent) |
            SpanReference::FollowsFrom(ref parent) => {
                for item in parent.baggage_items() {
                    self.context.set_baggage_item(item.clone())
                }
            }
        }
        self.references.push(reference);
    }
}


/// TODO
#[derive(Clone, Debug)]
pub enum SpanReference {
    ChildOf(SpanContext),
    FollowsFrom(SpanContext)
}


/// TODO
pub type SpanReceiver = mpsc::Receiver<FinishedSpan>;

/// TODO
pub type SpanSender = mpsc::Sender<FinishedSpan>;


#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::super::ImplWrapper;
    use super::super::SpanContext;
    use super::super::SpanReferenceAware;
    use super::super::span_context::BaggageItem;

    use super::FinishedSpan;
    use super::Span;
    use super::SpanReference;


    #[derive(Debug, Clone)]
    struct TestContext {
        pub id: String
    }
    impl SpanReferenceAware for TestContext {
        fn reference_span(&mut self, _: &SpanReference) {}
    }

    #[test]
    fn start_span_on_creation() {
        let (sender, _) = mpsc::channel();
        let context = SpanContext::new(ImplWrapper::new(TestContext {
            id: String::from("test-id")
        }));
        let _span: Span = Span::new("test-span", context, sender);
    }

    #[test]
    fn send_span_on_finish() {
        let (sender, receiver) = mpsc::channel();
        let context = SpanContext::new(ImplWrapper::new(TestContext {
            id: String::from("test-id")
        }));
        let span: Span = Span::new("test-span", context, sender);
        span.finish().unwrap();
        let _finished: FinishedSpan = receiver.recv().unwrap();
    }

    #[test]
    fn span_child_of_another() {
        let (sender, _) = mpsc::channel();
        let context = SpanContext::new(ImplWrapper::new(TestContext {
            id: String::from("test-id-1")
        }));
        let mut span = Span::new("test-span", context, sender);
        let mut context = SpanContext::new(ImplWrapper::new(TestContext {
            id: String::from("test-id-2")
        }));
        context.set_baggage_item(BaggageItem::new("a", "b"));
        span.child_of(context.clone());
        match span.references().get(0).unwrap() {
            &SpanReference::ChildOf(ref context) => {
                let span = context.impl_context::<TestContext>().unwrap();
                assert_eq!(span.id, "test-id-2");
            },
            _ => panic!("Invalid span reference")
        }
        let item = span.get_baggage_item("a").unwrap();
        assert_eq!(item.value(), "b");
    }

    #[test]
    fn span_follows_another() {
        let (sender, _) = mpsc::channel();
        let context = SpanContext::new(ImplWrapper::new(TestContext {
            id: String::from("test-id-1")
        }));
        let mut span = Span::new("test-span", context, sender);
        let mut context = SpanContext::new(ImplWrapper::new(TestContext {
            id: String::from("test-id-2")
        }));
        context.set_baggage_item(BaggageItem::new("a", "b"));
        span.follows(context.clone());
        match span.references().get(0).unwrap() {
            &SpanReference::FollowsFrom(ref context) => {
                let span = context.impl_context::<TestContext>().unwrap();
                assert_eq!(span.id, "test-id-2");
            },
            _ => panic!("Invalid span reference")
        }
        let item = span.get_baggage_item("a").unwrap();
        assert_eq!(item.value(), "b");
    }
}
