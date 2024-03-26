const std = @import("std");
const builtin = @import("builtin");

/// This variable is `true` if an atomic reference-counter is used for `Arc`, `false` otherwise.
///
/// If the target is single-threaded, `Arc` is optimized to a regular `Rc`.
pub const atomic_arc = !builtin.single_threaded or (builtin.target.isWasm() and std.Target.wasm.featureSetHas(builtin.cpu.features, .atomics));

/// A single threaded, strong reference to a reference-counted value.
pub fn Rc(comptime T: type) type {
    return struct {
        value: *T,
        alloc: std.mem.Allocator,

        const Self = @This();
        const Inner = struct {
            strong: usize,
            weak: usize,
            value: T,

            fn innerSize() comptime_int {
                return @sizeOf(@This());
            }

            fn innerAlign() comptime_int {
                return @alignOf(@This());
            }
        };

        /// Creates a new reference-counted value.
        pub fn init(alloc: std.mem.Allocator, t: T) std.mem.Allocator.Error!Self {
            const inner = try alloc.create(Inner);
            inner.* = Inner{ .strong = 1, .weak = 1, .value = t };
            return Self{ .value = &inner.value, .alloc = alloc };
        }

        /// Constructs a new `Rc` while giving you a `Weak` to the allocation,
        /// to allow you to construct a `T` which holds a weak pointer to itself.
        pub fn initCyclic(alloc: std.mem.Allocator, comptime data_fn: fn (*Weak) T) std.mem.Allocator.Error!Self {
            const inner = try alloc.create(Inner);
            inner.* = Inner{ .strong = 0, .weak = 1, .value = undefined };

            // Strong references should collectively own a shared weak reference,
            // so don't run the destructor for our old weak reference.
            var weak = Weak{ .inner = inner, .alloc = alloc };

            // It's important we don't give up ownership of the weak pointer, or
            // else the memory might be freed by the time `data_fn` returns. If
            // we really wanted to pass ownership, we could create an additional
            // weak pointer for ourselves, but this would result in additional
            // updates to the weak reference count which might not be necessary
            // otherwise.
            inner.value = data_fn(&weak);

            std.debug.assert(inner.strong == 0);
            inner.strong = 1;

            return Self{ .value = &inner.value, .alloc = alloc };
        }

        /// Gets the number of strong references to this value.
        pub fn strongCount(self: *const Self) usize {
            return self.innerPtr().strong;
        }

        /// Gets the number of weak references to this value.
        pub fn weakCount(self: *const Self) usize {
            return self.innerPtr().weak - 1;
        }

        /// Increments the strong count.
        pub fn retain(self: *Self) Self {
            self.innerPtr().strong += 1;
            return self.*;
        }

        /// Creates a new weak reference to the pointed value
        pub fn downgrade(self: *Self) Weak {
            return Weak.init(self);
        }

        /// Decrements the reference count, deallocating if the weak count reaches zero.
        /// The continued use of the pointer after calling `release` is undefined behaviour.
        pub fn release(self: Self) void {
            const ptr = self.innerPtr();

            ptr.strong -= 1;
            if (ptr.strong == 0) {
                ptr.weak -= 1;
                if (ptr.weak == 0) {
                    self.alloc.destroy(ptr);
                }
            }
        }

        /// Decrements the reference count, deallocating the weak count reaches zero,
        /// and executing `f` if the strong count reaches zero.
        /// The continued use of the pointer after calling `release` is undefined behaviour.
        pub fn releaseWithFn(self: Self, comptime f: fn (T) void) void {
            const ptr = self.innerPtr();

            ptr.strong -= 1;
            if (ptr.strong == 0) {
                f(self.value.*);

                ptr.weak -= 1;
                if (ptr.weak == 0) {
                    self.alloc.destroy(ptr);
                }
            }
        }

        /// Returns the inner value, if the `Rc` has exactly one strong reference.
        /// Otherwise, `null` is returned.
        /// This will succeed even if there are outstanding weak references.
        /// The continued use of the pointer if the method successfully returns `T` is undefined behaviour.
        pub fn tryUnwrap(self: Self) ?T {
            const ptr = self.innerPtr();

            if (ptr.strong == 1) {
                ptr.strong = 0;
                const tmp = self.value.*;

                ptr.weak -= 1;
                if (ptr.weak == 0) {
                    self.alloc.destroy(ptr);
                }

                return tmp;
            }

            return null;
        }

        /// Total size (in bytes) of the reference counted value on the heap.
        /// This value accounts for the extra memory required to count the references.
        pub fn innerSize() comptime_int {
            return Inner.innerSize();
        }

        /// Alignment (in bytes) of the reference counted value on the heap.
        /// This value accounts for the extra memory required to count the references.
        pub fn innerAlign() comptime_int {
            return Inner.innerAlign();
        }

        inline fn innerPtr(self: *const Self) *Inner {
            return @fieldParentPtr(Inner, "value", self.value);
        }

        /// A single threaded, weak reference to a reference-counted value.
        pub const Weak = struct {
            inner: ?*Inner = null,
            alloc: std.mem.Allocator,

            /// Creates a new weak reference.
            pub fn init(parent: *Rc(T)) Weak {
                const ptr = parent.innerPtr();
                ptr.weak += 1;
                return Weak{ .inner = ptr, .alloc = parent.alloc };
            }

            /// Creates a new weak reference object from a pointer to it's underlying value,
            /// without increasing the weak count.
            pub fn fromValuePtr(value: *T, alloc: std.mem.Allocator) Weak {
                return .{ .inner = @fieldParentPtr(Inner, "value", value), .alloc = alloc };
            }

            /// Gets the number of strong references to this value.
            pub fn strongCount(self: *const Weak) usize {
                return (self.innerPtr() orelse return 0).strong;
            }

            /// Gets the number of weak references to this value.
            pub fn weakCount(self: *const Weak) usize {
                const ptr = self.innerPtr() orelse return 1;
                if (ptr.strong == 0) {
                    return ptr.weak;
                } else {
                    return ptr.weak - 1;
                }
            }

            /// Increments the weak count.
            pub fn retain(self: *Weak) Weak {
                if (self.innerPtr()) |ptr| {
                    ptr.weak += 1;
                }
                return self.*;
            }

            /// Attempts to upgrade the weak pointer to an `Rc`, delaying dropping of the inner value if successful.
            ///
            /// Returns `null` if the inner value has since been dropped.
            pub fn upgrade(self: *Weak) ?Rc(T) {
                const ptr = self.innerPtr() orelse return null;

                if (ptr.strong == 0) {
                    ptr.weak -= 1;
                    if (ptr.weak == 0) {
                        self.alloc.destroy(ptr);
                        self.inner = null;
                    }
                    return null;
                }

                ptr.strong += 1;
                return Rc(T){
                    .value = &ptr.value,
                    .alloc = self.alloc,
                };
            }

            /// Decrements the weak reference count, deallocating if it reaches zero.
            /// The continued use of the pointer after calling `release` is undefined behaviour.
            pub fn release(self: Weak) void {
                if (self.innerPtr()) |ptr| {
                    ptr.weak -= 1;
                    if (ptr.weak == 0) {
                        self.alloc.destroy(ptr);
                    }
                }
            }

            /// Total size (in bytes) of the reference counted value on the heap.
            /// This value accounts for the extra memory required to count the references,
            /// and is valid for single and multi-threaded refrence counters.
            pub fn innerSize() comptime_int {
                return Inner.innerSize();
            }

            /// Alignment (in bytes) of the reference counted value on the heap.
            /// This value accounts for the extra memory required to count the references,
            /// and is valid for single and multi-threaded refrence counters.
            pub fn innerAlign() comptime_int {
                return Inner.innerAlign();
            }

            inline fn innerPtr(self: *const Weak) ?*Inner {
                return @as(?*Inner, @ptrCast(self.inner));
            }
        };
    };
}

/// A multi-threaded, strong reference to a reference-counted value.
pub fn Arc(comptime T: type) type {
    if (!atomic_arc) {
        return Rc(T);
    }

    return struct {
        value: *T,
        alloc: std.mem.Allocator,

        const Self = @This();
        const Inner = struct {
            strong: usize align(std.atomic.cache_line),
            weak: usize align(std.atomic.cache_line),
            value: T,

            fn innerSize() comptime_int {
                return @sizeOf(@This());
            }

            fn innerAlign() comptime_int {
                return @alignOf(@This());
            }
        };

        /// Creates a new reference-counted value.
        pub fn init(alloc: std.mem.Allocator, t: T) std.mem.Allocator.Error!Self {
            const inner = try alloc.create(Inner);
            inner.* = Inner{ .strong = 1, .weak = 1, .value = t };
            return Self{ .value = &inner.value, .alloc = alloc };
        }

        /// Constructs a new `Arc` while giving you a `Aweak` to the allocation,
        /// to allow you to construct a `T` which holds a weak pointer to itself.
        pub fn initCyclic(alloc: std.mem.Allocator, comptime data_fn: fn (*Weak) T) std.mem.Allocator.Error!Self {
            const inner = try alloc.create(Inner);
            inner.* = Inner{ .strong = 0, .weak = 1, .value = undefined };

            // Strong references should collectively own a shared weak reference,
            // so don't run the destructor for our old weak reference.
            var weak = Weak{ .inner = inner, .alloc = alloc };

            // It's important we don't give up ownership of the weak pointer, or
            // else the memory might be freed by the time `data_fn` returns. If
            // we really wanted to pass ownership, we could create an additional
            // weak pointer for ourselves, but this would result in additional
            // updates to the weak reference count which might not be necessary
            // otherwise.
            inner.value = data_fn(&weak);

            std.debug.assert(@atomicRmw(usize, &inner.strong, .Add, 1, .Release) == 0);
            return Self{ .value = &inner.value, .alloc = alloc };
        }

        /// Gets the number of strong references to this value.
        pub fn strongCount(self: *const Self) usize {
            return @atomicLoad(usize, &self.innerPtr().strong, .Acquire);
        }

        /// Gets the number of weak references to this value.
        pub fn weakCount(self: *const Self) usize {
            return @atomicLoad(usize, &self.innerPtr().weak, .Acquire) - 1;
        }

        /// Increments the strong count.
        pub fn retain(self: *Self) Self {
            _ = @atomicRmw(usize, &self.innerPtr().strong, .Add, 1, .AcqRel);
            return self.*;
        }

        /// Creates a new weak reference to the pointed value.
        pub fn downgrade(self: *Self) Weak {
            return Weak.init(self);
        }

        /// Decrements the reference count, deallocating if the weak count reaches zero.
        /// The continued use of the pointer after calling `release` is undefined behaviour.
        pub fn release(self: Self) void {
            const ptr = self.innerPtr();

            if (@atomicRmw(usize, &ptr.strong, .Sub, 1, .AcqRel) == 1) {
                if (@atomicRmw(usize, &ptr.weak, .Sub, 1, .AcqRel) == 1) {
                    self.alloc.destroy(ptr);
                }
            }
        }

        /// Decrements the reference count, deallocating the weak count reaches zero,
        /// and executing `f` if the strong count reaches zero.
        /// The continued use of the pointer after calling `release` is undefined behaviour.
        pub fn releaseWithFn(self: Self, comptime f: fn (T) void) void {
            const ptr = self.innerPtr();

            if (@atomicRmw(usize, &ptr.strong, .Sub, 1, .AcqRel) == 1) {
                f(self.value.*);
                if (@atomicRmw(usize, &ptr.weak, .Sub, 1, .AcqRel) == 1) {
                    self.alloc.destroy(ptr);
                }
            }
        }

        /// Returns the inner value, if the `Arc` has exactly one strong reference.
        /// Otherwise, `null` is returned.
        /// This will succeed even if there are outstanding weak references.
        /// The continued use of the pointer if the method successfully returns `T` is undefined behaviour.
        pub fn tryUnwrap(self: Self) ?T {
            const ptr = self.innerPtr();

            if (@cmpxchgStrong(usize, &ptr.strong, 1, 0, .Monotonic, .Monotonic) == null) {
                ptr.strong = 0;
                const tmp = self.value.*;
                if (@atomicRmw(usize, &ptr.weak, .Sub, 1, .AcqRel) == 1) {
                    self.alloc.destroy(ptr);
                }
                return tmp;
            }

            return null;
        }

        /// Total size (in bytes) of the reference counted value on the heap.
        /// This value accounts for the extra memory required to count the references.
        pub fn innerSize() comptime_int {
            return Inner.innerSize();
        }

        /// Alignment (in bytes) of the reference counted value on the heap.
        /// This value accounts for the extra memory required to count the references.
        pub fn innerAlign() comptime_int {
            return Inner.innerAlign();
        }

        inline fn innerPtr(self: *const Self) *Inner {
            return @fieldParentPtr(Inner, "value", self.value);
        }

        /// A multi-threaded, weak reference to a reference-counted value.
        pub const Weak = struct {
            inner: ?*Inner = null,
            alloc: std.mem.Allocator,

            /// Creates a new weak reference.
            pub fn init(parent: *Arc(T)) Weak {
                const ptr = parent.innerPtr();
                _ = @atomicRmw(usize, &ptr.weak, .Add, 1, .AcqRel);
                return Weak{ .inner = ptr, .alloc = parent.alloc };
            }

            /// Creates a new weak reference object from a pointer to it's underlying value,
            /// without increasing the weak count.
            pub fn fromValuePtr(value: *T, alloc: std.mem.Allocator) Weak {
                return .{ .inner = @fieldParentPtr(Inner, "value", value), .alloc = alloc };
            }

            /// Gets the number of strong references to this value.
            pub fn strongCount(self: *const Weak) usize {
                const ptr = self.innerPtr() orelse return 0;
                return @atomicLoad(usize, &ptr.strong, .Acquire);
            }

            /// Gets the number of weak references to this value.
            pub fn weakCount(self: *const Weak) usize {
                const ptr = self.innerPtr() orelse return 1;
                const weak = @atomicLoad(usize, &ptr.weak, .Acquire);

                if (@atomicLoad(usize, &ptr.strong, .Acquire) == 0) {
                    return weak;
                } else {
                    return weak - 1;
                }
            }

            /// Increments the weak count.
            pub fn retain(self: *Weak) Weak {
                if (self.innerPtr()) |ptr| {
                    _ = @atomicRmw(usize, &ptr.weak, .Add, 1, .AcqRel);
                }
                return self.*;
            }

            /// Attempts to upgrade the weak pointer to an `Arc`, delaying dropping of the inner value if successful.
            ///
            /// Returns `null` if the inner value has since been dropped.
            pub fn upgrade(self: *Weak) ?Arc(T) {
                const ptr = self.innerPtr() orelse return null;

                while (true) {
                    const prev = @atomicLoad(usize, &ptr.strong, .Acquire);

                    if (prev == 0) {
                        if (@atomicRmw(usize, &ptr.weak, .Sub, 1, .AcqRel) == 1) {
                            self.alloc.destroy(ptr);
                            self.inner = null;
                        }
                        return null;
                    }

                    if (@cmpxchgStrong(usize, &ptr.strong, prev, prev + 1, .Acquire, .Monotonic) == null) {
                        return Arc(T){
                            .value = &ptr.value,
                            .alloc = self.alloc,
                        };
                    }

                    std.atomic.spinLoopHint();
                }
            }

            /// Decrements the weak reference count, deallocating if it reaches zero.
            /// The continued use of the pointer after calling `release` is undefined behaviour.
            pub fn release(self: Weak) void {
                if (self.innerPtr()) |ptr| {
                    if (@atomicRmw(usize, &ptr.weak, .Sub, 1, .AcqRel) == 1) {
                        self.alloc.destroy(ptr);
                    }
                }
            }

            /// Total size (in bytes) of the reference counted value on the heap.
            /// This value accounts for the extra memory required to count the references.
            pub fn innerSize() comptime_int {
                return Inner.innerSize();
            }

            /// Alignment (in bytes) of the reference counted value on the heap.
            /// This value accounts for the extra memory required to count the references.
            pub fn innerAlign() comptime_int {
                return Inner.innerAlign();
            }

            inline fn innerPtr(self: *const Weak) ?*Inner {
                return @as(?*Inner, @ptrCast(self.inner));
            }
        };
    };
}

/// Creates a new `Rc` inferring the type of `value`
pub fn rc(alloc: std.mem.Allocator, value: anytype) std.mem.Allocator.Error!Rc(@TypeOf(value)) {
    return Rc(@TypeOf(value)).init(alloc, value);
}

/// Creates a new `Arc` inferring the type of `value`
pub fn arc(alloc: std.mem.Allocator, value: anytype) std.mem.Allocator.Error!Arc(@TypeOf(value)) {
    return Arc(@TypeOf(value)).init(alloc, value);
}
// const std = @import("std");
// const Allocator = std.mem.Allocator;
//
// pub fn Rc(comptime T: type) type {
//     return struct {
//         value: *T,
//         allocator: Allocator,
//
//         const Self = @This();
//         const Inner = struct {
//             strong: usize,
//             value: T,
//         };
//
//         pub fn init(allocator: Allocator, t: T) Allocator.Error!Self {
//             const inner = try allocator.create(Inner);
//             inner.* = Inner{ .strong = 1, .value = t };
//             return Self{ .value = &inner.value, .allocator = allocator };
//         }
//
//         pub fn retain(self: *Self) void {
//             self.innerPtr().strong += 1;
//         }
//
//         pub fn release(self: *Self) void {
//             const inner = self.innerPtr();
//             inner.strong -= 1;
//             if (inner.strong == 0) {
//                 self.allocator.destroy(inner);
//             }
//         }
//
//         pub fn strongCount(self: *const Self) usize {
//             return self.innerPtr().strong;
//         }
//
//         inline fn innerPtr(self: *const Self) *Inner {
//             return @fieldParentPtr(Inner, "value", self.value);
//         }
//     };
// }
//
// test "basic" {
//     const t = std.testing;
//
//     var five = try Rc(i32).init(t.allocator, 5);
//
//     // Test init
//     try t.expectEqual(5, five.value.*);
//     try t.expectEqual(1, five.strongCount());
//
//     // Test retain
//     var five_copy = five;
//     five_copy.retain();
//     try t.expectEqual(2, five.strongCount());
//
//     // Test that both pointer point to the same value
//     five.value.* += 1;
//     try t.expectEqual(6, five_copy.value.*);
//
//     // Test release
//     five_copy.release();
//     try t.expectEqual(1, five.strongCount());
//
//     // Cleanup
//     five.release();
// }
