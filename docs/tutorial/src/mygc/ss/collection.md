# Collection: Implement garbage collection

We need to add a few more things to get garbage collection working. 
Specifically, we need to config the `GCWorkerCopyContext`, which a GC worker uses for 
copying objects, and GC work packets that will be scheduled for a collection.

## CopyConfig

`CopyConfig` defines how a GC plan copies objects.
Similar to the `MutatorConfig` struct, you would need to define `CopyConfig` for your plan.

In `impl<VM: VMBinding> Plan for MyGC<VM>`, override the method `create_copy_config()`.
The default implementation provides a default `CopyConfig` for non-copying plans. So for non-copying plans,
you do not need to override the method. But
for copying plans, you would have to provide a proper copy configuration.

In a semispace GC, objects will be copied between the two copy spaces. We will use one
`CopySpaceCopyContext` for the copying, and will rebind the copy context to the proper tospace
in the preparation step of a GC (which will be discussed later when we talk about preparing for collections).

We use `CopySemantics::DefaultCopy` for our copy
operation, and bind it with the first `CopySpaceCopyContext` (`CopySemantics::DefaultCopy => CopySelector::CopySpace(0)`).
Other copy semantics are unused in this plan. We also provide an initial space
binding for `CopySpaceCopyContext`. However, we will flip tospace in every GC, and rebind the
copy context to the new tospace in each GC, so it does not matter which space we use as the initial
space here.

```rust
{{#include ../../../code/mygc_semispace/global.rs:create_copy_config}}
```

## Introduce collection to MyGC plan

Add a new method to `Plan for MyGC`, `schedule_collection()`. This function 
runs when a collection is triggered. It schedules GC work for the plan, i.e.,
it stops all mutators, runs the
scheduler's prepare stage and resumes the mutators. The `StopMutators` work
will invoke code from the bindings to scan threads and other roots, and those 
scanning work will further push work for a transitive closure.

Though you can add those work packets by yourself, `GCWorkScheduler` provides a
method `schedule_common_work()` that will add common work packets for you.

To use `schedule_common_work()`, first we need to create a type `MyGCWorkContext`
and implement the trait `GCWorkContext` for it. We create `gc_work.rs` and add the
following implementation. Note that we do not set a specific `ProcessEdgesWorkType`
and we will use the default [`SFTProcessEdges`](https://www.mmtk.io/mmtk-core/mmtk/scheduler/gc_work/struct.SFTProcessEdges.html),
which is a general work packet that a plan can use to trace objects. For plans
like semispace, `SFTProcessEdges` is sufficient. For more complex GC plans,
one can create and write their own work packet that implements the `ProcessEdgesWork` trait.

```rust
{{#include ../../../code/mygc_semispace/gc_work.rs:workcontext}}
```

Then we implement `schedule_collection()` using `MyGCWorkContext` and `schedule_common_work()`.

```rust
{{#include ../../../code/mygc_semispace/global.rs:schedule_collection}}
```

Delete `handle_user_collection_request()`. This function was an override of 
a Common plan function to ignore user requested collection for NoGC. Now we 
remove it and allow user requested collection.

## Prepare for collection

The collector has a number of steps it needs to perform before each collection.
We'll add these now.

### Prepare plan

In `mygc/global.rs`, find the method `prepare`. Delete the `unreachable!()` 
call, and add the following code:

```rust
{{#include ../../../code/mygc_semispace/global.rs:prepare}}
```

This function is called at the start of a collection. It prepares the two 
spaces in the common plan, flips the definitions for which space is 'to' 
and which is 'from', then prepares the copyspaces with the new definition.

Note that we call `set_copy_for_sft_trace()` for both spaces. This step is required
when using `SFTProcessEdges` to tell the spaces which copy semantic to use for copying.
For fromspace, we use the `DefaultCopy` semantic, which we have defined earlier in our `CopyConfig`.
So for objects in fromspace that need to be copied, the policy will use the copy context that binds with
`DefaultCopy` (which allocates to the tospace) in the GC worker. For tospace, we set its
copy semantics to `None`, as we do not expect to copy objects from tospace, and if that ever happens,
we will simply panic.

### Prepare worker

As we flip tospace for the plan, we also need to rebind the copy context
to the new tospace. We will override `prepare_worker()` in our `Plan` implementation.
`Plan.prepare_worker()` is executed by each GC worker in the preparation phase of a GC. The code
is straightforward -- we get the first `CopySpaceCopyContext`, and call `rebind()` on it with
the new `tospace`.

```rust
{{#include ../../../code/mygc_semispace/global.rs:prepare_worker}}
```

### Prepare mutator

Going back to `mutator.rs`, create a new function called 
`mygc_mutator_prepare(_mutator: &mut Mutator <MyGC<VM>>, _tls: OpaquePointer,)`. 
This function will be called at the preparation stage of a collection 
(at the start of a collection) for each mutator. Its body can stay empty, as 
there aren't any preparation steps for the mutator in this GC.
In `create_mygc_mutator()`, find the field `prep_func` and change it from
`mygc_mutator_noop()` to `mygc_mutator_prepare()`.


## Scan objects

Next, we'll add the code to allow the plan to collect garbage - filling out 
functions for work packets.

In `gc_work.rs`, add a new method to `ProcessEdgesWork for MyGCProcessEdges`,
`trace_object(&mut self, object: ObjectReference)`.
This method should return an ObjectReference, and use the 
inline attribute.
Check if the object passed into the function is null 
(`object.is_null()`). If it is, return the object.
Otherwise, check which space the object is in, and forward the call to the 
policy-specific object tracing code. If it is in neither space, forward the 
call to the common space and let the common space to handle object tracing in 
its spaces (e.g. immortal or large object space):

```rust
{{#include ../../../code/mygc_semispace/gc_work.rs:trace_object}}
```

Add two new implementation blocks, `Deref` and `DerefMut` for 
`MyGCProcessEdges`. These allow `MyGCProcessEdges` to be dereferenced to 
`ProcessEdgesBase`, and allows easy access to fields in `ProcessEdgesBase`.

```rust
{{#include ../../../code/mygc_semispace/gc_work.rs:deref}}
```

## Release

Finally, we need to fill out the functions that are, roughly speaking, 
run after each collection.

### Release in plan

Find the method `release()` in `mygc/global.rs`. Replace the 
`unreachable!()` call with the following code.

```rust
{{#include ../../../code/mygc_semispace/global.rs:release}}
```

This function is called at the end of a collection. It calls the release 
routines for the common plan spaces and the fromspace.

### Release in mutator

Go back to `mutator.rs`. In `create_mygc_mutator()`, replace 
`mygc_mutator_noop()` in the `release_func` field with `mygc_mutator_release()`.
Leave the `release()` function in the `CopyContext` empty. There are no 
release steps for `CopyContext` in this collector.

Create a new function called `mygc_mutator_release()` that takes the same 
inputs as the `prepare()` function above. This function will be called at the 
release stage of a collection (at the end of a collection) for each mutator. 
It rebinds the allocator for the `Default` allocation semantics to the new 
tospace. When the mutator threads resume, any new allocations for `Default` 
will then go to the new tospace.
 
```rust
{{#include ../../../code/mygc_semispace/mutator.rs:release}}
```

Delete `mygc_mutator_noop()`. It was a placeholder for the prepare and 
release functions that you have now added, so it is now dead code.

## Summary

You should now have MyGC working and able to collect garbage. All three
benchmarks should be able to pass now. 

If the benchmarks pass - good job! You have built a functional copying
collector!

If you get particularly stuck, the code for the completed `MyGC` plan
is available [here](https://github.com/mmtk/mmtk-core/tree/master/docs/tutorial/code/mygc_semispace).