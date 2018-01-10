use ::plan::transitive_closure::TransitiveClosure;
use ::util::address::{Address, ObjectReference};
use ::plan::tracelocal::TraceLocal;

pub struct NoGCTraceLocal {}

impl TransitiveClosure for NoGCTraceLocal {
    fn process_edge(&mut self, slot: Address) {
        unimplemented!();
    }

    fn process_node(&mut self, object: ObjectReference) {
        unimplemented!()
    }
}

impl TraceLocal for NoGCTraceLocal {
    fn process_roots(&mut self) {
        unimplemented!();
    }
    fn process_root_edge(&mut self, slot: Address, untraced: bool) {
        unimplemented!();
    }
    fn trace_object(&mut self, object: ObjectReference) -> ObjectReference {
        unimplemented!();
    }
    fn complete_trace(&mut self) {
        unimplemented!();
    }

    fn release(&mut self) {
        unimplemented!();
    }
}