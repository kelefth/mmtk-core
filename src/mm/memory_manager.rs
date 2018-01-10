use std::ptr::null_mut;
use libc::c_void;

use plan::plan::Plan;
use ::plan::mutator_context::MutatorContext;

use ::plan::tracelocal::TraceLocal;

#[cfg(feature = "jikesrvm")]
use ::vm::JTOC_BASE;

use ::util::{Address, ObjectReference};

use ::plan::selected_plan;
use self::selected_plan::{SelectedPlan, SelectedMutator};

use env_logger;

#[no_mangle]
#[cfg(feature = "jikesrvm")]
pub extern fn jikesrvm_gc_init(jtoc: *mut c_void, heap_size: usize) {
    env_logger::init().unwrap();
    unsafe { JTOC_BASE = Address::from_mut_ptr(jtoc); }
    selected_plan::PLAN.gc_init(heap_size);
    ::vm::scheduler::test1();
    info!("{}", ::vm::scheduler::test(44));
    info!("{}", ::vm::scheduler::test2(45, 67));
    info!("{}", ::vm::scheduler::test3(21, 34, 9, 8));
}

#[no_mangle]
#[cfg(not(feature = "jikesrvm"))]
pub extern fn jikesrvm_gc_init(_jtoc: *mut c_void, _heap_size: usize) {
    panic!("Cannot call jikesrvm_gc_init when not building for JikesRVM");
}

#[no_mangle]
#[cfg(feature = "jikesrvm")]
pub extern fn start_control_collector(thread_id: usize) {
    selected_plan::PLAN.control_collector_context.run(thread_id);
}

#[no_mangle]
#[cfg(not(feature = "jikesrvm"))]
pub extern fn start_control_collector(rvm_thread: *mut c_void) {
    panic!("Cannot call start_control_collector when not building for JikesRVM");
}

#[no_mangle]
pub extern fn gc_init(heap_size: usize) {
    if cfg!(feature = "jikesrvm") {
        panic!("Should be calling jikesrvm_gc_init instead");
    }
    env_logger::init().unwrap();
    selected_plan::PLAN.gc_init(heap_size);
}

#[no_mangle]
pub extern fn bind_mutator(thread_id: usize) -> *mut c_void {
    SelectedPlan::bind_mutator(&selected_plan::PLAN, thread_id)
}

#[no_mangle]
pub fn alloc(mutator: *mut c_void, size: usize,
             align: usize, offset: isize) -> *mut c_void {
    let local = unsafe { &mut *(mutator as *mut SelectedMutator) };
    local.alloc(size, align, offset).as_usize() as *mut c_void
}

#[no_mangle]
#[inline(never)]
pub fn alloc_slow(mutator: *mut c_void, size: usize,
                  align: usize, offset: isize) -> *mut c_void {
    let local = unsafe { &mut *(mutator as *mut SelectedMutator) };
    local.alloc_slow(size, align, offset).as_usize() as *mut c_void
}

#[no_mangle]
#[inline(never)]
pub extern fn alloc_large(_mutator: *mut c_void, _size: usize,
                          _align: usize, _offset: isize) -> *mut c_void {
    unimplemented!();
}

#[no_mangle]
pub extern fn mmtk_malloc(size: usize) -> *mut c_void {
    alloc(null_mut(), size, 1, 0)
}

#[no_mangle]
pub extern fn mmtk_free(_ptr: *const c_void) {}

#[no_mangle]
pub extern fn will_never_move(object: ObjectReference) -> bool {
    selected_plan::PLAN.will_never_move(object)
}

#[no_mangle]
pub extern fn report_delayed_root_edge(trace_local: *mut c_void, addr: *mut c_void) {
    let local = unsafe { &mut *(trace_local as *mut selected_plan::SelectedTraceLocal) };
    local.process_root_edge(unsafe { Address::from_usize(addr as usize) }, true);
    unimplemented!();
}

#[no_mangle]
pub extern fn will_not_move_in_current_collection(trace_local: *mut c_void, obj: *mut c_void) -> bool {
    unimplemented!();
}

#[no_mangle]
pub extern fn process_interior_edge(trace_local: *mut c_void, target: *mut c_void, slot: *mut c_void, root: bool) {
    unimplemented!();
}