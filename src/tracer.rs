use std::io;

use super::MapCarrier;
use super::Result;
use super::Span;
use super::SpanContext;


/// TODO
pub trait TracerInterface {
    /// TODO
    fn extract_binary(
        &self, carrier: Box<&mut self::io::Read>
    ) -> Result<Option<SpanContext>>;

    /// TODO
    fn extract_http_headers(
        &self, carrier: Box<&MapCarrier>
    ) -> Result<Option<SpanContext>>;

    /// TODO
    fn extract_textmap(
        &self, carrier: Box<&MapCarrier>
    ) -> Result<Option<SpanContext>>;

    /// TODO
    fn inject_binary(
        &self, context: &SpanContext, carrier: Box<&mut self::io::Write>
    ) -> self::io::Result<()>;

    /// TODO
    fn inject_http_headers(
        &self, context: &SpanContext, carrier: Box<&mut MapCarrier>
    );

    /// TODO
    fn inject_textmap(
        &self, context: &SpanContext, carrier: Box<&mut MapCarrier>
    );

    /// TODO
    fn span(&self, name: &str) -> Span;
}


/// TODO
pub struct Tracer {
    tracer: Box<TracerInterface>
}

impl Tracer {
    /// TODO
    pub fn new<T: 'static + TracerInterface>(tracer: T) -> Tracer {
        Tracer {
            tracer: Box::new(tracer)
        }
    }
}

impl Tracer {
    pub fn extract_binary<Carrier: self::io::Read>(
        &self, carrier: &mut Carrier
    ) -> Result<Option<SpanContext>> {
        self.tracer.extract_binary(Box::new(carrier))
    }

    /// TODO
    pub fn extract_http_headers<Carrier: MapCarrier>(
        &self, carrier: &Carrier
    ) -> Result<Option<SpanContext>> {
        self.tracer.extract_http_headers(Box::new(carrier))
    }

    /// TODO
    pub fn extract_textmap<Carrier: MapCarrier>(
        &self, carrier: &Carrier
    ) -> Result<Option<SpanContext>> {
        self.tracer.extract_textmap(Box::new(carrier))
    }

    /// TODO
    pub fn inject_binary<Carrier: self::io::Write>(
        &self, context: &SpanContext, carrier: &mut Carrier
    ) -> self::io::Result<()> {
        self.tracer.inject_binary(context, Box::new(carrier))
    }

    /// TODO
    pub fn inject_http_headers<Carrier: MapCarrier>(
        &self, context: &SpanContext, carrier: &mut Carrier
    ) {
        self.tracer.inject_http_headers(context, Box::new(carrier));
    }

    /// TODO
    pub fn inject_textmap<Carrier: MapCarrier>(
        &self, context: &SpanContext, carrier: &mut Carrier
    ) {
        self.tracer.inject_textmap(context, Box::new(carrier));
    }

    /// TODO
    pub fn span(&self, name: &str) -> Span {
        self.tracer.span(name)
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::io;
    use std::io::BufRead;
    use std::sync::mpsc;

    use super::super::Error;
    use super::super::Result;

    use super::super::ImplWrapper;
    use super::super::MapCarrier;
    use super::super::Span;
    use super::super::SpanContext;
    use super::super::SpanSender;
    use super::super::span_context::BaggageItem;

    use super::Tracer;
    use super::TracerInterface;


    #[derive(Debug, Clone)]
    struct TestContext {
        pub name: String
    }

    struct TestTracer {
        sender: SpanSender
    }
    impl TracerInterface for TestTracer {
        fn extract_binary(
            &self, carrier: Box<&mut self::io::Read>
        ) -> Result<Option<SpanContext>> {
            let mut reader = self::io::BufReader::new(carrier);
            let mut name = String::new();
            reader.read_line(&mut name).map_err(|e| Error::IoError(e))?;

            let mut context = SpanContext::new(ImplWrapper::new(TestContext {
                name: name.trim().to_owned()
            }));
            for line in reader.lines() {
                let line = line.map_err(|e| Error::IoError(e))?;
                let cells: Vec<&str> = line.split(':').collect();
                context.set_baggage_item(BaggageItem::new(cells[0], cells[1]));
            }
            Ok(Some(context))
        }

        fn extract_http_headers(
            &self, carrier: Box<&MapCarrier>
        ) -> Result<Option<SpanContext>> {
            let mut context = SpanContext::new(ImplWrapper::new(TestContext {
                name: carrier.get("Span-Name").unwrap()
            }));
            let items = carrier.find_items(Box::new(
                |k| k.starts_with("Baggage-")
            ));
            for (key, value) in items {
                context.set_baggage_item(BaggageItem::new(&key[8..], value));
            }
            Ok(Some(context))
        }

        fn extract_textmap(
            &self, carrier: Box<&MapCarrier>
        ) -> Result<Option<SpanContext>> {
            let mut context = SpanContext::new(ImplWrapper::new(TestContext {
                name: carrier.get("span-name").unwrap()
            }));
            let items = carrier.find_items(Box::new(
                |k| k.starts_with("baggage-")
            ));
            for (key, value) in items {
                context.set_baggage_item(BaggageItem::new(&key[8..], value));
            }
            Ok(Some(context))
        }

        fn inject_binary(
            &self, context: &SpanContext, carrier: Box<&mut self::io::Write>
        ) -> self::io::Result<()> {
            let inner = context.impl_context::<TestContext>().unwrap();
            carrier.write_fmt(format_args!("TraceId: {}\n", "123"))?;
            carrier.write_fmt(format_args!("Span Name: {}\n", &inner.name))?;
            for item in context.baggage_items() {
                carrier.write_fmt(
                    format_args!("Baggage-{}: {}\n", item.key(), item.value())
                )?;
            }
            Ok(())
        }

        fn inject_http_headers(
            &self, context: &SpanContext, carrier: Box<&mut MapCarrier>
        ) {
            let inner = context.impl_context::<TestContext>().unwrap();
            carrier.set("Trace-Id", "123");
            carrier.set("Span-Name", &inner.name);
            for item in context.baggage_items() {
                let key = format!("Baggage-{}", item.key());
                carrier.set(&key, item.value());
            }
        }

        fn inject_textmap(
            &self, context: &SpanContext, carrier: Box<&mut MapCarrier>
        ) {
            let inner = context.impl_context::<TestContext>().unwrap();
            carrier.set("trace-id", "123");
            carrier.set("span-name", &inner.name);
            for item in context.baggage_items() {
                let key = format!("baggage-{}", item.key());
                carrier.set(&key, item.value());
            }
        }

        fn span(&self, name: &str) -> Span {
            let context = SpanContext::new(ImplWrapper::new(TestContext {
                name: String::from("test-span")
            }));
            Span::new(name, context, self.sender.clone())
        }
    }


    #[test]
    fn create_span() {
        let (sender, _) = mpsc::channel();
        let tracer = Tracer::new(TestTracer {sender});
        let _span: Span = tracer.span("test-span");
    }

    #[test]
    fn extract_binary() {
        let mut buffer = io::Cursor::new("test-span\na:b\n");
        let (sender, _) = mpsc::channel();
        let tracer = Tracer::new(TestTracer {sender});
        let context = tracer.extract_binary(&mut buffer).unwrap().unwrap();
        let inner = context.impl_context::<TestContext>().unwrap();
        assert_eq!("test-span", inner.name);
        assert_eq!(context.baggage_items(), [BaggageItem::new("a", "b")]);
    }

    #[test]
    fn extract_http_headers() {
        let mut map = HashMap::new();
        map.insert(String::from("Span-Name"), String::from("2"));
        map.insert(String::from("Baggage-a"), String::from("b"));
        let (sender, _) = mpsc::channel();
        let tracer = Tracer::new(TestTracer {sender});
        let context = tracer.extract_http_headers(&map).unwrap().unwrap();
        let inner = context.impl_context::<TestContext>().unwrap();
        assert_eq!("2", inner.name);
        assert_eq!(context.baggage_items(), [BaggageItem::new("a", "b")]);
    }

    #[test]
    fn extract_textmap() {
        let mut map = HashMap::new();
        map.insert(String::from("span-name"), String::from("2"));
        map.insert(String::from("baggage-a"), String::from("b"));
        let (sender, _) = mpsc::channel();
        let tracer = Tracer::new(TestTracer {sender});
        let context = tracer.extract_textmap(&map).unwrap().unwrap();
        let inner = context.impl_context::<TestContext>().unwrap();
        assert_eq!("2", inner.name);
        assert_eq!(context.baggage_items(), [BaggageItem::new("a", "b")]);
    }

    #[test]
    fn inject_binary() {
        let (sender, _) = mpsc::channel();
        let tracer = Tracer::new(TestTracer {sender});
        let mut span = tracer.span("test-span");
        span.set_baggage_item("a", "b");

        let mut buffer: Vec<u8> = Vec::new();
        tracer.inject_binary(span.context(), &mut buffer).unwrap();
        assert_eq!(
            String::from_utf8(buffer).unwrap(),
            "TraceId: 123\nSpan Name: test-span\nBaggage-a: b\n"
        );
    }

    #[test]
    fn inject_http_headers() {
        let (sender, _) = mpsc::channel();
        let tracer = Tracer::new(TestTracer {sender});
        let mut span = tracer.span("test-span");
        span.set_baggage_item("a", "b");

        let mut map = HashMap::new();
        tracer.inject_http_headers(span.context(), &mut map);

        let mut items: Vec<(String, String)> = map.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        items.sort();
        assert_eq!(items, [
            (String::from("Baggage-a"), String::from("b")),
            (String::from("Span-Name"), String::from("test-span")),
            (String::from("Trace-Id"), String::from("123"))
        ]);
    }

    #[test]
    fn inject_textmap() {
        let (sender, _) = mpsc::channel();
        let tracer = Tracer::new(TestTracer {sender});
        let mut span = tracer.span("test-span");
        span.set_baggage_item("a", "b");

        let mut map = HashMap::new();
        tracer.inject_textmap(span.context(), &mut map);

        let mut items: Vec<(String, String)> = map.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        items.sort();
        assert_eq!(items, [
            (String::from("baggage-a"), String::from("b")),
            (String::from("span-name"), String::from("test-span")),
            (String::from("trace-id"), String::from("123"))
        ]);
    }
}